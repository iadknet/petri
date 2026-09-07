# Runtime config panel apply audit, 2026-09-07

Empirical audit of every **runtime** (non-startup) setting in the config panel against a
live `v3-server`, answering one question per field: *does editing it and pressing Apply
actually save?*

Method: `target/release/v3-server` on a local port, frontend dev server proxying to it.
All runtime controls were extracted programmatically from `RUNTIME_PATCH_FIELDS`
(`frontend/src/components/config-panel/runtime/RuntimeConfigPanel.tsx`), then each was
probed in isolation from a freshly seeded baseline: `PATCH /v3/simulation/config` with that
one field, `GET` to confirm persistence, restore, re-seed on any restore failure. Numeric
fields were probed at three points — a typical nudge (`current + 3·step`), the panel's `min`,
and the panel's `max`; booleans at their inverse. Eighteen further probes bisected the exact
accept/reject thresholds for the fields that failed. The UI path was then exercised in a real
browser for the headline cases.

**Re-verified against `main` at `4e217dad`**, after the complementary-nutrition removal
(`ae96957f`) landed. That work changed config shape — `NutritionConfig` deleted,
`metabolic_energy_yield` / `reproductive_reserve_yield` dropped from `FoodTypeConfig`, the two
default food types collapsed to one `"Primary Food"`, and a new `energy.costs.eat_reward_per_food`
runtime field with a panel row of its own. The panel now carries **55** runtime controls, and the
matrix was re-run in full against it. Every line reference below is against `4e217dad`.

Raw per-probe results: [`config-panel-runtime-apply-audit-2026-09-07.results.json`](config-panel-runtime-apply-audit-2026-09-07.results.json).

## Results

Every one of the 55 controls saves correctly at a typical mid-range value, including the new
`eat_reward_per_food`. Two defects remain, both silent:

1. **8 of the 55 are rejected outright at values their own sliders offer** — the panel's
   `min`/`max` bounds disagree with `normalize()`'s constraints.
2. **Every rejection is invisible.** Nothing is rendered, so a refused Apply looks like a
   successful one: the edited value stays on screen and the server keeps the old one.

Combined with all-or-nothing patching, one out-of-range slider silently discards every other
edit in the same Apply. Measured on `4e217dad`: patching `move_cost 0.35`,
`mutation_probability 0.6`, `growth_rate 0.12` and `max_creatures 5000` together → 422, none of
the three valid values stored, nothing shown to the user.

### Resolved incidentally by the nutrition removal

The original headline was worse: *on a freshly started backend, every runtime setting silently
failed to save*. `patch_config` rejects the whole patch when `normalize()` changes anything, and
`SimulationConfig::default()` was not a fixed point of its own `normalize()` — shared food
carried `initial_coverage = 0.54` while both default food types carried `0.27`, and
`sync_shared_from_canonical_food` (`crates/v3-core/src/config/simulation.rs:1033`) rewrites shared
from `types[0]`. `AppState::new()` stores the un-normalized default
(`crates/v3-server/src/app_state.rs:18-20`), so even `PATCH {}` returned 422 until a Restart
(which normalizes, `crates/v3-server/src/http/lifecycle.rs:84`).

Collapsing to a single food type at `initial_coverage = 0.54` — matching
`FoodResourceConfig::default()` (`simulation.rs:38`, `:69`) — made the sync a no-op, so the
default is now a fixed point. Confirmed on `4e217dad`: `PATCH {}` against a fresh server returns
200.

**Nothing guards this.** No test asserts that `normalize()` is idempotent or that
`normalize(default()) == default()`, so the next default-value edit can silently restore the
total-failure mode. That guard is the deliverable, not a value change. Low risk to world
generation: the only production reader of `initial_coverage` is the per-type seeder
(`crates/v3-core/src/kernel/ordinary_food/ecology.rs:244`, reading `entry.config` from the type
catalog), so normalizing at construction does not change the seeded world.

## Root causes

### 1. Rejections are invisible — frontend/server error shapes disagree

The server returns `{"error": "validation_rejected", "message": ..., "field_errors": [...]}`
(`crates/v3-server/src/error.rs:68-71`) — `error` is a **string**.

`ApiRequestError` reads `body.error.message` (`frontend/src/api/rest.ts:157`), which is
`undefined` on that shape, so the thrown `Error` carries an empty message. `ConfigPanel`
then does `setError(e.message)` → `""`, and renders `{error && <p …>}`
(`frontend/src/components/ConfigPanel.tsx:55`, `:88`) — an empty string is falsy, so nothing
is shown. Confirmed live: zero error paragraphs in the panel after a 422.

The declared `ApiError` type (`frontend/src/types/errors.ts:15-22`) describes a nested
`error: {code, message, details}` object that the server never sends, so this is a type-level
lie rather than a runtime type error. It affects every endpoint, not just config.

### 2. The server never says which field was rejected

`patch_config` (`crates/v3-server/src/http/status.rs:149-176`) merges the patch, normalizes the
result, and rejects it if normalization changed anything — collapsing every disagreement into a
single `FieldError` with `field: "config"` and the reason "runtime config patch contains values
outside canonical constraints". Even with the rendering fixed, the user would not learn which
edit was at fault.

### 3. All-or-nothing patching discards valid edits alongside one invalid one

Apply sends every changed field in a single patch, and one invalid field rejects the whole
thing (measured above).

### 4. Panel `min`/`max` bounds disagree with `normalize()` constraints

Eight controls offer values that `normalize()` will rewrite, which under the all-or-nothing
rule means a 422. Exact thresholds confirmed by bisection against the live server at `4e217dad`
— unchanged from the pre-nutrition run:

| Field | Panel range | Actually accepted | Constraint |
| --- | --- | --- | --- |
| `population.max_creatures` | 1 – 100000 | ≥ `initial_creatures` (10000 by default) | `simulation.rs:969-971` |
| `world.food.shared.max_density` | 0.1 – 1 | ≥ every `types[i].initial_density` (1.0 by default) | `simulation.rs:1024-1025` |
| `runtime.max_actions_per_turn` | 1 – 20 | ≥ `mutation.action_queue_cap` (4) | `simulation.rs:921` |
| `mutation.action_queue_cap` | 1 – 16 | ≤ `runtime.max_actions_per_turn` (10) | `simulation.rs:921` |
| `mutation.per_birth_mutation_events_min` | 0 – 20 | 1 … `events_max` (10) | `simulation.rs:912-917` |
| `mutation.per_birth_mutation_events_max` | 0 – 20 | ≥ `events_min` (1) | `simulation.rs:915-917` |
| `action_log.capacity` | 0 – 5000 | ≥ 1 | `simulation.rs:953-955` |
| `energy.costs.failed_action_penalty` | 0 – 20 | rejected while the startup ramp is active | `status.rs:134-147` |

Note `max_density` is effectively immovable downward at defaults: any value below `1.0` is
rejected because the single food type sits at `initial_density = 1.0`.

The `failed_action_penalty` row is different in kind: the server's ramp lock is deliberate and
the panel *does* show it as locked with a reason. The default `target_tick` is `62680`, so it
stays locked for the first 62,680 ticks — correct, but easily read as broken.

### 5. Restart silently drops runtime settings on the same failure

`handleRestart` (`frontend/src/components/ControlBar.tsx:145-173`) re-applies the previous
runtime config as one patch to survive the reseed, and its `catch` only calls `console.error`.
Any rejection there loses every runtime setting the user had applied, with nothing shown in the UI.

## Fix plan

Ordered by user impact.

**1. Guard the normalization fixed point.** Normalize in `AppState::new()` / `from_config` so the
server can never start from an un-normalized config, and add the tests that are missing: a
`v3-server` regression test that a fresh server accepts `PATCH {}`, and a `v3-core` property test
that `normalize` is idempotent *and* `normalize(default()) == default()`. The current default
already satisfies this, so the tests are the deliverable.

**2. Make failures visible.** Fix `ApiRequestError` to read the server's real shape
(`body.message`, with `body.field_errors`), and give `ConfigPanel` a fallback so an empty
message never renders as no message. Render `field_errors` per field. This is a small change
with repo-wide benefit — the same constructor backs every endpoint. Align
`frontend/src/types/errors.ts` with what `error.rs` actually serializes (or change the server
to emit the nested shape the type already describes — pick one and make the other match).

**3. Report which field was rejected.** Diff `merged_value` against `normalized_value` and emit
a `FieldError` per differing JSON path with both the requested and the canonical value. This
alone would have made this whole audit unnecessary.

**4. Policy: reject, never clamp, and keep the patch atomic.** *(Decided 2026-09-07.)* The server
must never silently rewrite a submitted value and must never partially apply a patch: an
out-of-constraint patch changes nothing and returns 422 naming every offending path and its
constraint, which the panel renders per field so the user can fix the flagged control and
re-apply.

**5. Derive panel bounds from the real constraints.** Fix the eight rows above so the UI cannot
offer a value the server will refuse: clamp `max_creatures` to `≥ initial_creatures`,
`max_density` to `≥ max(types[i].initial_density)`, couple `max_actions_per_turn` and
`action_queue_cap`, couple `events_min` / `events_max`, and set `action_log.capacity` min to 1.
Static `min`/`max` in the field definitions cannot express cross-field constraints, so this
needs a per-field bound resolver taking the current draft. Prefer computing bounds from the
draft so the slider physically cannot reach an invalid value.

**6. Surface restart re-apply failures.** Replace the `console.error` in `handleRestart` with
a visible error, so a dropped runtime config after Restart is not silent.

## Out of scope / follow-ups

- This audit tested **save** (PATCH accepted + GET reflects it), not whether the running
  simulation honors each value at tick time. `world.food.shared.*` is explicitly re-applied via
  `reconfigure_food`; fields read through cached or per-creature state — `action_log.capacity`
  (log buffer sizing) and `population.max_creatures` are the ones worth verifying separately.
- `startup.*` and world topology are restart-only by design, rejected by `patch_config` with a
  clear per-field reason, and are not rendered in the runtime panel. Not defects.
- Floats round-trip as `f32` widened to `f64` (`growth_rate` reads `0.09000000357627869` in the
  API response). The panel displays them fine and comparisons tolerate it; noted only because
  it shows up in raw responses.

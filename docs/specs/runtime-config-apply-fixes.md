# Runtime Config Apply Fixes

**Status**: Complete
**Last updated**: 2026-09-07
**Scope**: Maintenance; no roadmap feature ID, dependency row, or closure changes

## Goal

The runtime config panel saves, applies, and reports its settings correctly:
every value the panel can offer is one the server accepts, a rejected Apply
tells the user which field was refused and why, and a rejected patch never
changes anything on the server.

## Non-Goals

- Changing any production default value, `normalize()` constraint, or the
  startup-only field set. `normalize()` keeps its clamping semantics for
  deserialized and programmatically built configs; only the PATCH endpoint
  rejects instead of clamping.
- Removing or relaxing the `energy.costs.failed_action_penalty` startup-ramp
  lock (server rule and panel lock both stay as they are).
- Verifying that the running simulation honors each runtime value at tick time
  (the audit's stated follow-up), partial application of patches, or any new
  dependency, abstraction, or configuration surface beyond what the six items
  below need.

## Inputs and Invariants

The user maintenance request of 2026-09-07 is authoritative. Follow
[the workflow](../workflow.md) with the feature-template sections adapted here;
roadmap ownership, checkbox, dependency, and rollup requirements do not apply
(`make roadmap-check` must still pass). Work starts from `main` at
`4e217dad` in `.claude/worktrees/runtime-config-apply-fixes`, branch
`worktree-runtime-config-apply-fixes`. The audit
[config-panel-runtime-apply-audit-2026-09-07.md](../strategy/config-panel-runtime-apply-audit-2026-09-07.md)
and its results JSON were staged but uncommitted on `main` when the goal
started; they are committed on this branch unchanged, together with their
`docs/README.md` entry. `main` had not advanced past `4e217dad` at planning
time, so every line reference and threshold in the audit stands as written.

Source-of-truth code, read at planning:

- `crates/v3-core/src/config/simulation.rs` `normalize()` (line 846 onward)
  and its cross-field rules at 912-921, 953-955, 969-971, 1024-1025, 1033.
- `crates/v3-server/src/http/status.rs` `patch_config` (149-176) and
  `crates/v3-server/src/error.rs` `IntoResponse` (68-71), which emits a flat
  `{error: <code string>, message, field_errors}` body.
- `docs/reference/v3-server-api-protocol-spec.md` §6, the normative error
  envelope: `{protocol_version, error: {code, message, details: {endpoint,
  field_errors, expected_state, current_state}}}`. `frontend/src/types/errors.ts`
  already declares this shape and `ApiRequestError` already reads
  `body.error.message`. The server is the one party that disagrees.
- `frontend/src/components/ConfigPanel.tsx`, `config-panel/runtime/RuntimeFieldGroup.tsx`,
  `config-panel/shared/FieldRow.tsx`, `config-panel/shared/types.ts`,
  `frontend/src/components/ControlBar.tsx` `handleRestart` (145-173).

Decisions (settled by the user, 2026-09-07):

1. **Reject, never clamp; atomic patch.** An out-of-constraint patch changes
   nothing and returns `422 validation_rejected` naming every offending path
   and its constraint. The server never rewrites a submitted value and never
   applies part of a patch.
2. **Item 1 is a regression guard, not a value change.** `normalize(default())`
   is already a fixed point after the single-food collapse; the deliverable is
   the tests plus normalizing at server construction.
3. **Item 5 resolves bounds from the current draft**, because static
   `min`/`max` cannot express cross-field constraints.
4. **Error envelope**: the server changes to emit the nested envelope the
   normative reference and the frontend type already describe. The frontend
   type is not rewritten to a flat shape.

Required contracts:

1. **Normalization fixed point (audit item 1).** `AppState::from_config`
   normalizes the config before storing it as `startup_defaults` and before
   seeding, so a server can never start from an un-normalized config. Tests:
   a `v3-server` integration test that a fresh `AppState::new()` server
   accepts `PATCH /v3/simulation/config` with `{}` (200, config unchanged); a
   `v3-core` example test that `normalize(default()) == default()` compared
   through serialized JSON; and a `v3-core` proptest that `normalize` is
   idempotent (`normalize(normalize(c)) == normalize(c)`) over a strategy that
   starts from the default and perturbs at least the cross-constrained fields
   (`initial_creatures`/`max_creatures`, `max_actions_per_turn`/`action_queue_cap`,
   `events_min`/`events_max`, `action_log.capacity`, shared `max_density` and
   each type's `initial_density`/`initial_coverage`) plus a sample of the
   clamped floats, including non-finite draws. No assertion depends on which
   cases are drawn.

2. **Error envelope (audit item 2, server side).** `AppError::into_response`
   emits `{protocol_version, error: {code, message, details}}` where `details`
   carries `endpoint` and `field_errors` for `validation_rejected`, and
   `expected_state`/`current_state` for `invalid_state_transition`; `details`
   is omitted when empty. Existing server tests that assert on the flat shape
   move to the nested one. The reference §6 needs no edit; §4.8 gains one
   line stating that a rejected patch names every offending path in
   `details.field_errors` and applies nothing.

3. **Visible failures (audit item 2, frontend side).** `ApiRequestError`
   exposes `fieldErrors: FieldError[]` (from `body.error.details.field_errors`,
   default `[]`) and never carries an empty message: fall back to
   `<code> (HTTP <status>)` and then to `Request failed (HTTP <status>)`.
   `ConfigPanel` keeps the panel-level message (fallback "Config update
   failed" for non-API errors) and additionally holds a per-path error map
   that `RuntimeFieldGroup` passes into `FieldRow`, which renders the reason
   under the affected row with `data-testid="field-error-<path with dots as
   dashes>"` and a visible, non-empty text. Field errors whose path matches no
   runtime row (for example `config`) render in the panel-level message. Both
   clear on the next Apply attempt and on Reset.

4. **Per-field rejection (audit items 3 and 4, server).** `patch_config` keeps
   the existing early rejections (state, startup-only paths, food shape,
   ramp lock, deserialization). After deep-merging and normalizing, if the
   normalized JSON differs from the merged JSON the patch is rejected whole
   with one `FieldError` per offending path, determined as follows:
   - **Attribution.** For each leaf path in the submitted patch, merge that
     leaf alone into the current config, normalize, and compare. A leaf whose
     isolated application changes any path is offending; its `field` is the
     leaf's dotted path (array indices as `.0.`), and its `reason` names the
     requested value, the canonical value `normalize()` would produce, and,
     when the changed path is a different field, that path (for example
     `requested 15; canonical constraints move mutation.per_birth_mutation_events_max to 15`).
   - **Fallback.** If no leaf is offending in isolation but the whole patch
     still differs after normalization, emit one `FieldError` per differing
     path with the requested and canonical values.
   - The response is `422 validation_rejected`; the config, world, fertility
     cache, and websocket frame are untouched. The endpoint applies the
     normalized config only when it equals the merged config exactly.
   The audit's mixed patch (`move_cost 0.35`, `mutation_probability 0.6`,
   `growth_rate 0.12`, `max_creatures 5000`) must yield exactly one
   `FieldError`, for `population.max_creatures`, and a following `GET` must
   show none of the four values stored. The seven bound-constrained fields
   (`population.max_creatures`, `world.food.shared.max_density`,
   `runtime.max_actions_per_turn`, `mutation.action_queue_cap`,
   `mutation.per_birth_mutation_events_min`,
   `mutation.per_birth_mutation_events_max`, `action_log.capacity`) each
   produce a `FieldError` naming that exact path when patched alone at an
   invalid value.

5. **Draft-derived bounds (audit item 5, frontend).** A pure
   `resolveRuntimeBounds(field, draft): {min, max}` in
   `config-panel/runtime/` returns the static bounds for every field except:
   `population.max_creatures` min = `draft.population.initial_creatures`;
   `world.food.shared.max_density` min = max over
   `draft.world.food.types[i].initial_density`; `runtime.max_actions_per_turn`
   min = `draft.mutation.action_queue_cap`; `mutation.action_queue_cap` max =
   `draft.runtime.max_actions_per_turn`; `mutation.per_birth_mutation_events_min`
   min = 1 and max = `draft.mutation.per_birth_mutation_events_max`;
   `mutation.per_birth_mutation_events_max` min =
   `draft.mutation.per_birth_mutation_events_min`; `action_log.capacity`
   min = 1. When a resolved min exceeds the static max (or vice versa) the
   resolved pair is still ordered (`min <= max`). `RuntimeFieldGroup` passes
   resolved bounds to both inputs of `FieldRow`. The slider clamps on every
   change; the number input clamps when the edit is committed (blur or
   Enter), so a user can still type a multi-digit value, and a click on Apply
   blurs the input first. In both cases the value reaching `updateDraft` is
   inside the resolved bounds. The `failed_action_penalty` ramp lock is
   untouched.

6. **Restart re-apply failure (audit item 6).** `handleRestart` surfaces a
   rejected re-apply patch as visible text in the control bar
   (`data-testid="control-restart-error"`, containing the error message and
   each `field: reason`), cleared on the next restart attempt. The startup
   itself still completes and the config store is still refreshed from the
   server so the panel shows what actually applied.

## Implementation Tasks

- [x] Red tests first for each contract: server envelope shape, fresh-server
      `PATCH {}`, per-field rejection and atomicity, core fixed-point and
      idempotence, `ApiRequestError` field errors and fallback message,
      ConfigPanel per-field rendering, bound resolver and clamping, and the
      restart error surface.
- [x] Server: normalize in `from_config`; nested envelope; attribution-based
      `FieldError` list; protocol reference §4.8 line.
- [x] Frontend: `ApiRequestError`, `ConfigPanel`, `RuntimeFieldGroup`,
      `FieldRow`, bound resolver, `ControlBar`.
- [x] Live-server re-run of the audit's failing cases (see Verification); the
      orchestrator performs the browser check, since the implementer has no
      browser tool.
- [x] Simplify pass, fresh mutation-survivor triage, independent review, and
      any permitted remediation. Simplify pass and fresh mutation run done by
      the implementer; independent review found P1 0 / P2 1 / P3 5, and one
      remediation pass addressed the P2 and two P3s (below).

### Red evidence

- `normalize_leaves_the_default_config_unchanged` fails when the audited
  default mismatch is reintroduced (`FoodResourceConfig::default().initial_coverage`
  temporarily set to `0.27` against `types[0]` at `0.54`):
  `assertion left == right failed`, shared `initial_coverage`
  `0.5400000214576721` vs `0.27000001072883606`. The default was reverted
  immediately; no production default changed.
- Seven server tests failed before the implementation:
  `error_envelope_nests_code_message_and_details`,
  `error_envelope_omits_details_when_there_are_none`,
  `error_envelope_reports_state_transition_details`,
  `from_config_normalizes_before_storing_and_seeding`,
  `patch_config_names_each_bound_constrained_field_patched_alone`,
  `patch_config_reason_names_the_cross_field_path_it_would_move`,
  `patch_config_rejects_mixed_patch_atomically_naming_only_the_offender`
  (`test result: FAILED. 82 passed; 7 failed`), with the flat body
  `{"error":"validation_rejected", ..., "field_errors":[{"field":"config", ...}]}`
  in the failure output. `fresh_default_server_accepts_an_empty_config_patch`
  and `patch_config_applies_a_fully_valid_mixed_patch` passed from the start,
  as the audit predicted for the current default.
- Frontend: `npx vitest run src/api/rest.test.ts
  src/components/config-panel/runtime/bounds.test.ts
  src/components/ConfigPanel.test.tsx src/components/ControlBar.test.tsx`
  reported `Test Files 4 failed (4) / Tests 14 failed | 27 passed (41)` before
  the implementation.

### Reading recorded for the number input

Contract 5's "the value reaching `updateDraft` is inside the resolved bounds"
is implemented as: the slider clamps on every change, and the number input
passes the raw typed value through while an edit is in progress and clamps on
commit (blur, Enter, or the blur that Apply performs first). A strict reading
that clamped every keystroke would prevent typing any multi-digit value whose
prefix is below the minimum, which contract 5 explicitly rules out.

## Verification

- [x] `cargo test -p v3-core --lib config` → `ok. 122 passed; 0 failed`;
      `cargo test -p v3-server` → `ok. 46 passed` (lib), `ok. 2 passed` (bin),
      `ok. 89 passed` (integration), `ok. 0 passed` (doc); all exit 0.
      `cargo check --workspace --all-targets` → `Finished dev profile`, exit 0
      (also run by the post-edit hook after each `.rs` edit).
      `cargo fmt --all -- --check` and `cargo clippy -p v3-server --all-targets`
      are clean.
- [x] `cargo test -p v3-core --test viability`: not required first, since no
      default, founder, or tick mechanic changes; runs inside `make check`.
- [x] Frontend `npm run lint` → 3 pre-existing warnings, 0 errors;
      `npm run test` → `Test Files 57 passed (57) / Tests 304 passed (304)`;
      `npm run build` → `✓ built in 6.13s`; all exit 0. Counts are from the
      post-remediation rerun. New tests:
      `frontend/src/api/rest.test.ts` (4),
      `frontend/src/components/config-panel/runtime/bounds.test.ts` (12),
      8 added to `ConfigPanel.test.tsx`, 2 added to `ControlBar.test.tsx`.
- [x] Live-server re-run against `target/release/v3-server` built from this
      worktree, `V3_SERVER_BIND_ADDR=127.0.0.1:18731`, killed afterwards
      (`pgrep -fl v3-server` → none; port free). All requests were
      `curl -s -X PATCH http://127.0.0.1:18731/v3/simulation/config
      -H 'content-type: application/json' -d '<payload>'`; the trailing
      `[HTTP nnn]` is `curl -w '%{http_code}'`.

  Fresh server, `-d '{}'` → `[HTTP 200]`, body
  `{"config":{"action_log":{"capacity":500}, ...},"protocol_version":"v3alpha2","state":"idle"}`
  (config unchanged).

  Each bound-constrained field alone at the audit's invalid value, all
  `[HTTP 422]` with `error.code = "validation_rejected"` and
  `error.details.endpoint = "patch_config"`; `error.details.field_errors` was
  exactly one entry in every case:

  | payload | `field` | `reason` |
  | --- | --- | --- |
  | `{"population":{"max_creatures":5000}}` | `population.max_creatures` | `requested 5000; canonical value is 100000` |
  | `{"world":{"food":{"shared":{"max_density":0.5}}}}` | `world.food.shared.max_density` | `requested 0.5; canonical constraints move world.food.shared.initial_density to 0.5; canonical constraints move world.food.types.0.initial_density to 0.5` |
  | `{"runtime":{"max_actions_per_turn":2}}` | `runtime.max_actions_per_turn` | `requested 2; canonical constraints move mutation.action_queue_cap to 2` |
  | `{"mutation":{"action_queue_cap":12}}` | `mutation.action_queue_cap` | `requested 12; canonical value is 10` |
  | `{"mutation":{"per_birth_mutation_events_min":15}}` | `mutation.per_birth_mutation_events_min` | `requested 15; canonical constraints move mutation.per_birth_mutation_events_max to 15` |
  | `{"mutation":{"per_birth_mutation_events_max":0}}` | `mutation.per_birth_mutation_events_max` | `requested 0; canonical value is 1` |
  | `{"action_log":{"capacity":0}}` | `action_log.capacity` | `requested 0; canonical value is 500` |

  The audit's mixed patch
  `{"energy":{"costs":{"move_cost":0.35}},"mutation":{"mutation_probability":0.6},"world":{"food":{"shared":{"growth_rate":0.12}}},"population":{"max_creatures":5000}}`
  → `[HTTP 422]`, body
  `{"protocol_version":"v3alpha2","error":{"code":"validation_rejected","message":"validation failed for patch_config","details":{"endpoint":"patch_config","field_errors":[{"field":"population.max_creatures","reason":"requested 5000; canonical value is 100000"}]}}}`
  — exactly one entry. The following `curl -s
  http://127.0.0.1:18731/v3/simulation/config` showed all four values
  unchanged at their defaults: `energy.costs.move_cost 0.20000000298023224`,
  `mutation.mutation_probability 0.44`,
  `world.food.shared.growth_rate 0.09000000357627869`,
  `population.max_creatures 100000`.
- [x] Browser check by the orchestrator through the frontend dev server: each of the seven rows
      cannot be driven to the invalid value from the panel (draft value is
      clamped to the resolved bound); an Apply rejected by the live server
      renders visible error text naming the field.

  Orchestrator run, 2026-09-07, release `v3-server` rebuilt from this
  worktree on `127.0.0.1:18732`, Vite on `localhost:5174` proxying to it, the
  in-app browser driving the page. Independent curl re-run first: `PATCH {}`
  → 200; the seven single-field invalid patches → 422 naming exactly the
  patched path; the mixed patch → 422 naming only `population.max_creatures`,
  `GET` afterwards showing all four defaults. In the panel, the number
  inputs reported resolved bounds `max_creatures min=10000`,
  `max_density min=1`, `max_actions_per_turn min=4`, `action_queue_cap max=10`,
  `events_min min=1 max=10`, `events_max min=1`, `capacity min=1`. Entering
  each audit value (5000, 0.5, 2, 12, 15, 0, 0) and committing (focusout)
  clamped every row to its resolved bound, with coupled rows re-resolving
  (`max_actions_per_turn` followed `action_queue_cap` to 12, `events_max`
  followed `events_min`); a real keyboard entry of `5000` then Tab on Max
  Creatures committed `10000`. Apply of the clamped draft → 200 and the
  server stored all seven bound values. Bypassing the clamp by injecting
  `max_creatures 5000` into the draft and clicking Apply → 422; the panel
  rendered `field-error-population-max_creatures` = `requested 5000;
  canonical value is 100000` under the row and `config-error` =
  `validation failed for patch_config`, both visible in a screenshot, and
  the server still held 10000. Servers stopped afterwards.
- [x] `make roadmap-check` → `roadmap-check: validation passed`, exit 0;
      `git diff --check` clean.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`, run twice.
      Run mode both times: `rust-mutants: fresh run; no prior mutant results
      reused`; diff against `4e217dad`.
      Output path both times:
      `/Users/istefanek/.local/share/petri-tools/mutants/runtime-config-apply-fixes/mutants.out`.

  First run summary: `32 mutants tested in 3m: 1 missed, 28 caught, 3 unviable`.
  Full survivor list (one entry, from `missed.txt`; `timeout.txt` empty):

  - `crates/v3-server/src/http/config_patch.rs:155:54: replace match guard
    left.len() == right.len() with true in collect_differences` — **killed**.
    With the guard always true, arrays of unequal length were zipped over the
    shorter prefix, so a resized array reported no difference at all. Added
    `json_differences_reports_a_resized_array_at_its_own_path`, which asserts
    that `[1, 2]` normalized to `[1]` is reported as one difference at the
    array's own path. No production code was changed.

  Second run summary after that test: `32 mutants tested in 3m: 29 caught,
  3 unviable`, `rust-mutants: no survivors`. The 2026-09-07 remediation pass
  changed only TypeScript and this spec, so that run still covers the Rust
  diff and was not repeated.
- [x] Benchmark report: Not applicable: this change touches only the server
      config endpoint, construction-time normalization of an already-normal
      default, the error envelope, and the frontend; it cannot change
      simulation cost or the seeded world (the only production reader of the
      normalized food fields is the per-type seeder, which reads the same
      values before and after).
- [x] Second goal-profile determinism run: Not applicable per the workflow's
      2026-09-05 decision.
- [x] Independent review completed and findings recorded (fresh
      `roadmap-reviewer`: P1 0, P2 1, P3 5; the P2 startup-row clamp and the
      stale-sentence and total-resolver P3s were remediated, the remaining
      three P3s are deferred in Notes); orchestrator `make check` result for
      the reviewed content is recorded in the closure note below.

## Performance and Goal Impact

No simulation mechanism changes. No natural analog applies; this is a
maintenance fix to the control surface. Not measured, per the Verification
item above.

## Success Criteria

- [x] The seven bound-constrained rows cannot reach an invalid value from the
      panel, and a direct PATCH at such a value is rejected naming that path.
      Bounds come from `resolveRuntimeBounds`, covered by
      `frontend/src/components/config-panel/runtime/bounds.test.ts`; the
      rejections are recorded under Verification.
- [x] The audit's mixed patch is rejected whole, naming only
      `population.max_creatures`, with nothing stored (live evidence above and
      `patch_config_rejects_mixed_patch_atomically_naming_only_the_offender`).
- [x] A rejected Apply shows visible error text naming the field, and a
      rejected restart re-apply shows visible error text in the control bar
      (`field-error-<path>` and `control-restart-error`, covered by
      `ConfigPanel.test.tsx` and `ControlBar.test.tsx`).
- [x] A fresh server accepts `PATCH {}`, and the fixed-point and idempotence
      tests guard the default.

## Notes for AI Agents

- Deviation recorded at start: `main` was not clean when the goal began; the
  only changes were the staged audit files and their README entry, which this
  goal names as its inputs. They were carried into the worktree as a patch and
  committed here; the orchestrator discards the identical staged copy on
  `main` only after verifying it matches the committed content, immediately
  before the fast-forward.
- Attribution runs one extra normalize per submitted leaf (at most the 55
  runtime controls); this is a control-path cost only, reached only when a
  patch is already known to be rejected.
- `patch_config_names_each_bound_constrained_field_patched_alone` and the
  mixed-patch test use `population.max_creatures: 32`, not the audit's `5000`:
  the server test harness `test_config()` sets `initial_creatures = 64`, so
  `5000` is a valid value there. The other six fields use the audit's literal
  values, and all seven are exercised at the audit's literals against the live
  server (see Verification).
- No other consumer of the old flat error body exists: `grep` over `scripts/`,
  `frontend/e2e/`, `crates/v3-cli/`, and the frontend sources finds only the
  audit note, which records the pre-fix shape as history and is left as
  written.
- Simplification pass (single-pass inline review, run without the Agent
  fan-out): dropped a hand-rolled `json_pointer` helper by carrying the
  submitted value inside `JsonDifference`; removed a `String` clone in the
  attribution loop; extracted `describeApiFailure` in
  `frontend/src/types/errors.ts`, now shared by `ConfigPanel` and `ControlBar`;
  and replaced the twelve hand-written `RuntimeFieldGroup` blocks in
  `RuntimeConfigPanel` with a `RUNTIME_GROUPS` table plus a prop spread, which
  is what made threading `fieldErrors` a one-line change.
- `FieldRow` clamps a committed number edit (blur or Enter) only for rows that
  are passed resolved `bounds`, which today are the runtime rows. Startup rows
  get no commit handler and pass typed values through exactly as before, guarded
  by `keeps a typed startup value above the static max after commit` in
  `ConfigPanel.test.tsx`. The slider still routes every change through
  `clampToBounds`, for startup rows against the field's own static pair, which
  is a no-op for a range input already constrained to that pair.
- Deferred review findings (P3, not addressed here):
  - `FieldRow` imports `FieldBounds`/`clampToBounds` from `runtime/bounds.ts`;
    move the generic pair to `shared/`.
  - The `"min" in field` duck test is duplicated in `ConfigPanel.tsx` and
    `bounds.test.ts`; add one `isNumericField` guard in `shared/types.ts`.
  - `canonicalize_patch` in `status.rs` duplicates the merge/normalize/serialize
    block of `patch_config`; have `patch_config` call the helper.
- Remediation pass ran in a fresh implementer context because SendMessage is
  unavailable in this desktop session.
- Closure: orchestrator `make check` exited 0 in the worktree on the reviewed
  and remediated content (log
  `scratchpad/make-check-1.log` of the orchestrating session); the closing
  commit below is that tested tree plus this note.
- `AppState::from_config` normalizing means a caller that passes a config with
  `max_creatures < initial_creatures` now gets the canonical `100000` rather
  than its own value. `lifecycle.rs` already normalized on startup, so only the
  construction path changed.
- Cost record: `/usage` totals not collected (the user was not present
  during this autonomous run to issue the command); implementer advisor
  consults 2 (initial pass) + 2 (remediation pass, fresh context); reviewer
  findings P1 0 / P2 1 / P3 5. Telemetry only.

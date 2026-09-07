# Runtime Config Apply Fixes

**Status**: In Progress
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

- [ ] Red tests first for each contract: server envelope shape, fresh-server
      `PATCH {}`, per-field rejection and atomicity, core fixed-point and
      idempotence, `ApiRequestError` field errors and fallback message,
      ConfigPanel per-field rendering, bound resolver and clamping, and the
      restart error surface.
- [ ] Server: normalize in `from_config`; nested envelope; attribution-based
      `FieldError` list; protocol reference §4.8 line.
- [ ] Frontend: `ApiRequestError`, `ConfigPanel`, `RuntimeFieldGroup`,
      `FieldRow`, bound resolver, `ControlBar`.
- [ ] Live-server re-run of the audit's failing cases (see Verification); the
      orchestrator performs the browser check, since the implementer has no
      browser tool.
- [ ] Simplify pass, fresh mutation-survivor triage, independent review, and
      any permitted remediation.

## Verification

- [ ] `cargo test -p v3-core --lib config` and `cargo test -p v3-server` pass
      with the new tests; `cargo check --workspace --all-targets` after each
      coherent Rust edit.
- [ ] `cargo test -p v3-core --test viability`: not required first, since no
      default, founder, or tick mechanic changes; runs inside `make check`.
- [ ] Frontend `npm run lint`, `npm run test`, `npm run build` pass, including
      new tests in `ConfigPanel.test.tsx`, `ControlBar.test.tsx`, a bound
      resolver test, and a `rest.ts` error test.
- [ ] Live-server re-run against a release `v3-server` built from the
      worktree (`V3_SERVER_BIND_ADDR=127.0.0.1:<port>`), recorded here with
      the exact requests and responses: each of the seven bound-constrained
      fields patched alone at the audit's invalid value returns 422 with a
      `field_errors` entry naming that path; the mixed patch returns 422 with
      exactly one entry, `population.max_creatures`, and `GET` shows all four
      values unchanged; a fresh server accepts `PATCH {}` with 200.
- [ ] Browser check by the orchestrator through the frontend dev server: each of the seven rows
      cannot be driven to the invalid value from the panel (draft value is
      clamped to the resolved bound); an Apply rejected by the live server
      renders visible error text naming the field.
- [ ] `make roadmap-check` on document changes; `git diff --check`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      run mode, and the full survivor list with each resolution recorded here.
- [ ] Benchmark report: Not applicable: this change touches only the server
      config endpoint, construction-time normalization of an already-normal
      default, the error envelope, and the frontend; it cannot change
      simulation cost or the seeded world (the only production reader of the
      normalized food fields is the per-type seeder, which reads the same
      values before and after).
- [ ] Second goal-profile determinism run: Not applicable per the workflow's
      2026-09-05 decision.
- [ ] Independent review completed and findings recorded; orchestrator
      `make check` exited 0 for the reviewed content.

## Performance and Goal Impact

No simulation mechanism changes. No natural analog applies; this is a
maintenance fix to the control surface. Not measured, per the Verification
item above.

## Success Criteria

- [ ] The seven bound-constrained rows cannot reach an invalid value from the
      panel, and a direct PATCH at such a value is rejected naming that path.
- [ ] The audit's mixed patch is rejected whole, naming only
      `population.max_creatures`, with nothing stored.
- [ ] A rejected Apply shows visible error text naming the field, and a
      rejected restart re-apply shows visible error text in the control bar.
- [ ] A fresh server accepts `PATCH {}`, and the fixed-point and idempotence
      tests guard the default.

## Notes for AI Agents

- Deviation recorded at start: `main` was not clean when the goal began; the
  only changes were the staged audit files and their README entry, which this
  goal names as its inputs. They were carried into the worktree as a patch and
  committed here; the orchestrator discards the identical staged copy on
  `main` only after verifying it matches the committed content, immediately
  before the fast-forward.
- Attribution runs one extra normalize per submitted leaf (at most the 55
  runtime controls); this is a control-path cost only.
- Cost record: to be added at closure.

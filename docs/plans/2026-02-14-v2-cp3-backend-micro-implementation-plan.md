# Petri V2 CP-3 Backend Micro-Implementation Plan

**Goal:** Replace synthetic `v2-server` world/status behavior with deterministic `v2-core`-backed world initialization and tick updates using small, test-gated slices.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2/crates/v2-core` world-state/bootstrap pieces, `v2/crates/v2-server` integration/state mapping, and backend test coverage; excludes `v2-cli`, frontend UX/layout work, and protocol-version changes.
**Docs Impact:** Adds a CP-3 backend execution plan with discrete slices; Stage 4 and CP-3 spec docs should reference this plan as execution guidance.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: startup and runtime payloads reflect real world state rather than synthetic placeholders.
- `GP-02`: simulation policy moves into `v2-core`, while `v2-server` remains transport/orchestration.
- `GP-03`: every behavior shift is introduced through a red-green test slice.
- `GP-04`: status, frame, and health telemetry become grounded in actual world evolution.
- `GP-04`: seeded founders start from one shared phenotype baseline and evolve through runtime reproduction/mutation, not startup randomness.

## Boundary Impact

- `v2-core` gains/extends runtime world-state primitives used by server integration.
- `v2-server` removes synthetic derivations and maps API payloads from `v2-core` state.
- `v2-web` and `v2-cli` contracts stay unchanged during this plan.
- Legacy crates remain reference-only.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-server/src/state.rs` | change | Current reset/tick paths keep empty food and synthetic counters; must be replaced with core-backed state. |
| `v2/crates/v2-server/src/api.rs` | change | Frame/status/health currently derive from placeholders and must map to runtime state. |
| `v2/crates/v2-core/src/ecology/*` | keep | Ecology config/telemetry semantics remain source-of-truth and should be consumed, not duplicated in server logic. |
| `v2/web/src/features/protocol/*` | keep | Wire contract remains stable; this plan changes payload truthfulness, not schema shape. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `startup` request add new fields immediately (for example `initial_food_density`)? | No. Keep `v2alpha1` wire schema stable in this slice; use deterministic internal defaults and revisit in follow-up if required. | user+agent | resolved |
| Should server keep synthetic `action_counts` until full runtime parity? | No. Action counts must come from applied world actions in this plan. | user+agent | resolved |
| Should this plan include snapshot endpoints? | No. Snapshot endpoints remain out of scope for `v2alpha1`. | user+agent | resolved |
| Should startup-seeded creatures begin with heterogeneous phenotypes? | No. Startup-seeded founders must share one phenotype baseline; diversity emerges through reproduction mutation. | user+agent | resolved |
| What should happen if startup creates a population that immediately collapses? | Startup must enforce a viability guard (or reject startup) so the run begins with at least one viable creature. | user+agent | resolved |
| Should v1 food tuning defaults be carried forward before wire-level tuning fields exist? | Yes. Carry forward `initial_food_density`, `food_growth_rate`, `food_spawn_rate`, `food_spread_threshold`, and `food_spawn_floor_density` as deterministic internal defaults, then expose via API in a follow-up slice. | user+agent | resolved |

## Slice Execution Rules

1. One slice per commit.
2. Every slice starts with a failing test in the same area.
3. A slice is complete only when targeted tests pass and no new synthetic fallback path is introduced.
4. Keep file touch scope narrow: one behavior seam per slice.

## Micro-Slice Task List

### Slice B1: Startup food realism guard tests

Files:
- Create: `v2/crates/v2-server/tests/startup_world_init.rs`
- Modify: `v2/crates/v2-server/tests/lifecycle.rs`

Steps:
1. Add failing tests asserting startup frame includes non-empty food by default.
2. Add failing determinism test: same startup request + seed yields same seeded food cells.
3. Add failing variation test: different seed changes seeded food placement.

### Slice B2: Deterministic food seeding primitive in `v2-core`

Files:
- Create: `v2/crates/v2-core/src/world_seed.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/tests/world_seed.rs`

Steps:
1. Implement deterministic seeded food-cell generation helper.
2. Export helper from `v2-core` for server integration.
3. Add unit tests for bounds safety and deterministic output.

### Slice B3: Wire startup/reset food seeding in `v2-server`

Files:
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/startup_world_init.rs`

Steps:
1. Replace empty food initialization on reset/startup with `v2-core` seeded food output.
2. Keep existing paint semantics unchanged.
3. Green B1 tests.

### Slice B4: Creature-frame realism guard tests

Files:
- Create: `v2/crates/v2-server/tests/frame_creatures.rs`

Steps:
1. Add failing tests showing repeated `GET /frame` without ticks does not reshuffle creature coordinates.
2. Add failing tests showing creature IDs are stable across adjacent frames unless population changes.

### Slice B5: Introduce minimal core world-state struct

Files:
- Create: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/tests/world_state.rs`

Steps:
1. Add `WorldState` container for creatures, food, barriers, and tick.
2. Add deterministic startup construction from seed + startup dimensions/population.
3. Add unit tests for occupancy bounds and stable startup seeding.

### Slice B6: Remove synthetic creature generator from server API

Files:
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/frame_creatures.rs`

Steps:
1. Replace `creature_snapshots(...)` synthetic frame generation with data from stored world state.
2. Keep payload schema identical.
3. Green B4 tests.

### Slice B7: Tick/action realism guard tests

Files:
- Create: `v2/crates/v2-server/tests/tick_dynamics.rs`
- Modify: `v2/crates/v2-server/tests/ws_stream.rs`

Steps:
1. Add failing tests asserting `tick_running` mutates world/telemetry beyond tick counter.
2. Add failing tests asserting action counts are derived from applied actions, not modulo formulas.
3. Add websocket expectations for status/frame changes after real ticks.

### Slice B8: Implement `WorldState::tick` and action accounting

Files:
- Modify: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-core/tests/world_state.rs`

Steps:
1. Add deterministic tick transition logic for movement/food consumption/reproduction/death bookkeeping.
2. Return per-tick action counts and aggregate counters needed by status.
3. Add tests for monotonic tick and action-count invariants.

### Slice B9: Server tick loop integration with core world state

Files:
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/server.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/tick_dynamics.rs`

Steps:
1. Replace `advance_ticks` synthetic path with `WorldState::tick` loop calls.
2. Update status/health derivation from runtime state outputs.
3. Green B7 tests.

### Slice B10: Remove synthetic fallbacks and dead helpers

Files:
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`

Steps:
1. Delete modulo-based action helper and synthetic creature helper paths.
2. Remove noncollapse-baseline status derivation from server runtime loop.
3. Ensure no server runtime path fabricates frame entities independent of world state.

### Slice B11: Backend checkpoint hardening

Files:
- Modify: `v2/crates/v2-server/tests/payloads.rs`
- Modify: `v2/crates/v2-server/tests/lifecycle.rs`
- Modify: `v2/crates/v2-server/tests/ws_stream.rs`

Steps:
1. Add regression checks for startup/world/tick realism assumptions added above.
2. Confirm protocol envelope/version/ordering contracts remain unchanged.

### Slice B12: Founder phenotype uniformity guard tests

Files:
- Create: `v2/crates/v2-server/tests/startup_founder_phenotype.rs`
- Modify: `v2/crates/v2-server/tests/startup_world_init.rs`

Steps:
1. Add failing tests asserting all seeded startup creatures share the same `phenotype_rgb` at tick `0`.
2. Add failing tests asserting founder phenotype is deterministic for the same startup seed.
3. Add failing tests asserting phenotype diversity appears only after tick/reproduction paths.

### Slice B13: Founder phenotype baseline implementation

Files:
- Modify: `v2/crates/v2-core/src/world_state.rs`
- Create: `v2/crates/v2-core/src/phenotype.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Modify: `v2/crates/v2-server/src/api.rs`

Steps:
1. Add a single founder phenotype baseline constant set in `v2-core` (RGB + mutation metadata if needed).
2. Ensure startup seeding applies the shared founder phenotype to all initial creatures.
3. Keep frame serialization unchanged while removing any startup-time phenotype randomization.
4. Green B12 tests.

### Slice B14: Startup viability guard tests

Files:
- Create: `v2/crates/v2-server/tests/startup_viability.rs`
- Modify: `v2/crates/v2-server/tests/lifecycle.rs`

Steps:
1. Add failing tests for viability guard behavior on startup/start.
2. Add failing tests proving default startup config yields a surviving population over a short deterministic horizon.
3. Add failing tests for non-viable startup handling (reject or deterministic viable fallback, per finalized contract).

### Slice B15: Startup viability guard implementation

Files:
- Create: `v2/crates/v2-core/src/viability.rs`
- Modify: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`

Steps:
1. Add a deterministic short-horizon viability probe helper in `v2-core`.
2. Integrate viability checking into startup/reset flow.
3. Ensure startup guarantees at least one viable creature (or returns protocol-valid rejection).
4. Green B14 tests.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test world_seed`
3. `cd v2 && cargo test -p v2-core --test world_state`
4. `cd v2 && cargo test -p v2-core --test phenotype_evolution`
5. `cd v2 && cargo test -p v2-core --test startup_viability_gate`
6. `cd v2 && cargo test -p v2-server --test startup_world_init`
7. `cd v2 && cargo test -p v2-server`
8. Run the `CP-3` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-3 exit`).

## Risks and Rollback

- Risk: introducing world-state logic in large jumps can reintroduce synthetic shortcuts under time pressure.
- Risk: behavior drift can break frontend expectations if payload semantics shift without contract tests.
- Rollback:
1. Revert the most recent slice commit only (never batch-revert multiple slices blindly).
2. Re-introduce failing guard tests first, then re-implement the slice with narrower scope.

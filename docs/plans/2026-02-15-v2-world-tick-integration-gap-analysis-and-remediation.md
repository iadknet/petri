# V2 World-Tick Integration Gap Analysis and Remediation

**Goal:** Close the gap between v2 functionality already implemented in core modules and what is actually integrated into the live world tick and protocol surfaces.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** Analysis of v1-v2 runtime parity gaps plus concrete remediation tasks for v2-core/v2-server/v2-web integration; excludes legacy crate changes.
**Docs Impact:** Adds this active plan as integration source-of-truth; no canonical strategy/standards changes in this slice unless remediation reveals new invariant changes.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: restores meaningful creature/world behavior in the running simulation loop.
- `GP-03`: converts integration assumptions into explicit, failing-first regression tests.
- `GP-04`: ensures runtime telemetry/payload fields reflect actual world state transitions.

## Boundary Impact

- No dependency-direction changes.
- `v2-core` remains owner of world-tick simulation policy.
- `v2-server` remains owner of transport mapping and startup orchestration.
- `v2-web` remains owner of startup controls and protocol-driven rendering.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/world_state.rs` | change | Current tick path is integrated but omits key lifecycle behavior (variable food density and reproduction). |
| `v2/crates/v2-server/src/state.rs` | change | Server startup/runtime currently hold seed/tick configs but do not expose enough tuning knobs through startup request. |
| `v2/crates/v2-server/src/api.rs` | change | Frame/status schema supports density/reproduce fields but values are partially synthetic or hardcoded in practice. |
| `v2/web/src/features/startup/*` | change | Startup UI currently exposes only a minimal subset of tuning controls compared to v1 capabilities. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should remediation port all v1 behavior at once? | No; integrate the highest-impact runtime gaps first (food density realism, reproduction, startup tuning knobs), then iterate. | user+agent | resolved |
| Should world-tick reuse full legacy controller/tick logic directly? | No; keep v2 tick model minimal but behavior-complete for current checkpoint intent. | user+agent | resolved |
| Should request schema remain backward-compatible for startup knobs? | Additive compatibility only: new startup tuning fields should be optional/defaultable. | user+agent | resolved |

## Integration Gap Inventory

1. Food density realism gap:
- v2 world stores food as occupancy-only and frame maps all food cells to density `255`.
- v1 food model tracks per-cell density and growth/spread/spawn update density values.

2. Reproduction integration gap:
- v2 world tick currently never increments births/reproduce actions and does not create offspring.
- v1 tick includes reproduction gating, energy transfer, and occupancy-aware offspring placement.

3. Tuning surface gap:
- v2 startup request exposes only world size/population/tick budget knobs.
- v1 exposes broader ecology/energy/reproduction tuning needed to shape emergent behavior.

4. Integration-orphaned module gap:
- v2 has substantial tested modules (`runtime`, `evolution`, `ecology`) whose semantics are not yet wired into live server world tick.
- Result: test-covered contracts exist without equivalent live execution-path behavior.

## Task List

### Task 1: Lock gaps with failing integration tests first

Files:
- Modify: `v2/crates/v2-core/tests/world_state.rs`
- Modify: `v2/crates/v2-server/tests/tick_dynamics.rs`
- Modify: `v2/crates/v2-server/tests/startup_world_init.rs`

Steps:
1. Add failing test for variable food density values over ticks.
2. Add failing test proving reproduction can occur and births/action counts reflect it.
3. Add failing test proving startup tuning fields influence seeded/tick behavior.

### Task 2: Integrate food density model into live v2 world tick

Files:
- Modify: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`

Steps:
1. Replace occupancy-only food representation with per-cell density state.
2. Port density-aware growth/spread/spawn behavior (v2-adapted from v1).
3. Ensure frame payload reports real per-cell density bytes.

### Task 3: Integrate reproduction into live v2 world tick

Files:
- Modify: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-server/src/state.rs`

Steps:
1. Add deterministic reproduction decision path with energy/space gating.
2. Add offspring spawning and energy transfer semantics.
3. Wire births/reproduce action counts into status windows.

### Task 4: Expose additional startup tuning knobs in v2 API + UI

Files:
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/web/src/features/protocol/models.ts`
- Modify: `v2/web/src/features/startup/StartupPanel.tsx`
- Modify: `v2/web/src/features/simulation/store/simulationStore.ts`

Steps:
1. Add optional/defaultable startup tuning fields for food and reproduction dynamics.
2. Map startup knobs into world seed/tick configuration at reset.
3. Add startup panel controls for the new knobs and keep defaults deterministic.

### Task 5: Integration coverage and matrix alignment

Files:
- Modify: `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

Steps:
1. Confirm CP-3 criteria explicitly include variable food-density and reproduction-path assertions.
2. Keep matrix command list aligned with actual regression suite names.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test world_state`
3. `cd v2 && cargo test -p v2-server --test tick_dynamics`
4. `cd v2 && cargo test -p v2-server --test startup_world_init`
5. `cd v2 && cargo test -p v2-server`
6. `cd v2/web && npm run test`

## Risks and Rollback

- Risk: reproduction defaults could cause runaway populations without adequate gating.
- Risk: startup schema expansion may require broad fixture/test updates.
- Risk: density model may increase frame payload size in dense worlds.
- Rollback:
1. Revert startup schema/UI additions first if protocol churn blocks progress.
2. Keep density/reproduction runtime integration with server-side defaults while UI knobs are iterated separately.

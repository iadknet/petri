# V3 Stage 4: Tick + Seeding + Viability E2E Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire stages 1–3b into a runnable simulation: world seeding, per-tick orchestration (Phase 0 updates + turn queue + action application), and end-to-end viability confirming creatures eat, move, and reproduce over multiple ticks without panicking.

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** New `mutation/` and `simulation/` modules in `v3/crates/v3-core/src/`; extend `creature/state.rs`; new integration tests in `tests/viability.rs`. Status updates to master plan.

**Docs Impact:** `docs/plans/2026-02-21-v3-stage-4-tick-seeding-viability.md` (this plan created); `docs/plans/2026-02-21-v3-implementation-master-plan.md` Stage 4 row updated on completion.

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-21-v3-implementation-master-plan.md`

---

## Goal Alignment

- **GP-01:** Completes the simulation execution substrate — creatures can now be seeded, aged, fed, moved, and reproduced in a full tick loop.
- **GP-02:** All new modules follow the existing dependency hierarchy; `simulation/` sits at the top, importing from `creature/`, `kernel/`, `sensors/`, and `runtime/` but not vice versa.
- **GP-03:** All semantics are tested at unit and integration level; viability tests confirm emergent behavior (creatures survive, eat, reproduce) end-to-end.

## Boundary Impact

- New `mutation/` module imports `creature/genome.rs` and `config/simulation.rs` only.
- New `simulation/` module sits at the top of the v3-core dependency chain; imports all existing modules.
- `creature/state.rs` gains two new fields — backward-compatible struct change (internal tests updated).
- No changes to `contracts/`, `config/`, `kernel/`, `sensors/`, or `runtime/` internals.
- `lib.rs` gains `pub mod mutation;` and `pub mod simulation;`.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `creature/state.rs` | change | Add `phenotype_channel_weights` and `phenotype_channel_polarity`; update `new()` constructor and existing tests |
| `runtime/mesh.rs` | keep | `execute_creature_mesh` signature unchanged; simulation/tick.rs calls with extracted fields |
| `kernel/world.rs` | keep | All utilities reused as-is |
| `sensors/static_inputs.rs` | keep | `assemble_static_inputs` reused as-is; internal tests updated for new `CreatureState::new` signature |
| `runtime/vm.rs` | keep | Internal tests updated for new `CreatureState::new` signature only |
| `lib.rs` | change | Add `pub mod mutation; pub mod simulation;` |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `apply_reproduce` own the RNG or borrow from `sim.rng`? | Borrow from `sim.rng` via a separate `rng: &mut impl Rng` parameter to avoid double-borrow of `sim` | agent | resolved |
| How to split-borrow `sim` in `run_tick` for mesh execution? | Extract creature fields into a block scoped `let action = { ... }` to release borrow before action dispatch | agent | resolved |
| Does `CreatureId` implement `Ord` for `queue.sort()`? | Yes — slotmap `new_key_type!` derives `Ord` and `PartialOrd` | agent | resolved |
| Should `apply_reproduce` handle `energy_transfer_request <= 0` as `RejectedEnergyConstraints`? | Yes — zero or negative transfer means no viable offspring energy | agent | resolved |

---

## Context and File Layout

- Worktree: `.worktrees/v3-implementation`
- All cargo commands run from the `v3/` subdirectory
- Key specs: `docs/reference/v3-tick-orchestration-spec.md`, `docs/reference/v3-startup-seeding-spec.md`, `docs/reference/v3-reproduction-spec.md`, `docs/reference/v3-runtime-config-spec.md`, `docs/reference/v3-phenotype-spec.md`, `docs/reference/v3-mutation-spec.md`

Expected final file tree after this stage:
```
v3/crates/v3-core/src/
├── creature/state.rs     ← extended: phenotype_channel_weights + phenotype_channel_polarity
├── mutation/
│   ├── mod.rs            ← NEW
│   ├── types.rs          ← NEW: MutationSummary, MutationSkipReason
│   └── engine.rs         ← NEW: MutationEngine stub
├── simulation/
│   ├── mod.rs            ← NEW
│   ├── simulation.rs     ← NEW: Simulation struct
│   ├── seeding.rs        ← NEW: seed_simulation
│   ├── tick.rs           ← NEW: run_phase_0, run_tick
│   └── actions.rs        ← NEW: apply_noop, apply_eat, apply_move, apply_reproduce
└── lib.rs                ← extended: pub mod mutation; pub mod simulation;
v3/crates/v3-core/tests/viability.rs  ← NEW: E2E integration tests
```

---

## Tasks

- [x] **Task 1:** Extend `CreatureState` with `phenotype_channel_weights`/`phenotype_channel_polarity`; create `mutation/types.rs`, `mutation/engine.rs`, `mutation/mod.rs`; update all `CreatureState::new(...)` call sites.
  - [x] `creature/state.rs` has new fields and updated constructor
  - [x] All existing `CreatureState::new(...)` calls in `state.rs`, `sensors/static_inputs.rs`, `runtime/vm.rs` updated to pass `[1.0f32; 3]` and `[true; 3]`
  - [x] `MutationSummary::zero()` has all fields zero and accounting invariant holds
  - [x] `MutationEngine::apply_mutations` stub returns zero summary for any genome
  - [x] `cargo test -p v3-core` passes (all previous + new mutation tests)

- [x] **Task 2:** Create `simulation/simulation.rs` (`Simulation` struct) and `simulation/seeding.rs` (`seed_simulation` function).
  - [x] `seed_simulation(default_config, 42).creature_count()` > 0
  - [x] All creature positions are unique after seeding
  - [x] `sim.world.total_food() > 0` after seeding
  - [x] Two calls with same seed produce same `creature_count()` and `world.total_food()`
  - [x] First creature has `phenotype_rgb == [204, 61, 61]`, weights `[1.0; 3]`, polarity `[true; 3]`

- [x] **Task 3:** Create `simulation/tick.rs` with `run_phase_0`.
  - [x] Creature age increments by 1 after `run_phase_0`
  - [x] Creature energy decreases by decay amount
  - [x] Creature with energy <= 0 after decay removed from slotmap and world occupancy
  - [x] Food total increases when `growth_rate = 1.0`
  - [x] Dead creature position is no longer occupied after removal

- [x] **Task 4:** Create `simulation/actions.rs` with `apply_noop`, `apply_eat`, `apply_move`, `apply_reproduce`.
  - [x] `apply_eat` increases energy and clears food
  - [x] `apply_eat` caps energy at max
  - [x] `apply_move` updates position and occupancy grid
  - [x] `apply_move` into barrier: no position change, cost still deducted
  - [x] `apply_move` into occupied cell: no position change
  - [x] `apply_reproduce` creates child with inherited genome
  - [x] `apply_reproduce` fails when target is occupied
  - [x] `apply_reproduce` fails when parent energy insufficient
  - [x] `apply_reproduce` deducts reproduce_cost from parent
  - [x] `apply_reproduce` returns `RejectedPopulationCap` at population cap
  - [x] `apply_reproduce` child has `generation = parent.generation + 1`
  - [x] `apply_reproduce` child inherits parent memory

- [x] **Task 5:** Extend `simulation/tick.rs` with `run_tick` (turn queue + full tick execution).
  - [x] `run_tick` increments `sim.tick` by 1
  - [x] Dead creatures removed mid-tick are skipped in queue
  - [x] Newborn children are NOT in the current tick's queue
  - [x] After tick on world with food + founder creature, energy changed or food consumed

- [x] **Task 6:** Create `v3/crates/v3-core/tests/viability.rs` with 6 E2E integration tests.
  - [x] `seed_simulation(default_config, 42)` + 20 `run_tick` calls completes without panic
  - [x] After 5 ticks, `sim.creature_count() > 0`
  - [x] Total food after 1 tick differs from initial
  - [x] All founders have energy matching `config.energy.lifecycle.initial_energy`
  - [x] Place founder on cell with food, run 1 tick, verify food reduced or energy increased
  - [x] Two sims with same seed produce same `creature_count()` and `world.total_food()` after seeding

- [x] **Task 7:** Wire modules in `lib.rs`, verify `simulation/mod.rs` exports, run quality gates, update master plan and create evidence matrix.
  - [x] `lib.rs` has `pub mod mutation;` and `pub mod simulation;`
  - [x] `cargo fmt --all -- --check` passes
  - [x] `cargo clippy --workspace --all-targets -- -D warnings` passes (zero warnings)
  - [x] `cargo test --workspace` passes (all tests green, 215 tests: 209 unit + 6 integration)
  - [x] `scripts/check-plan-harness.sh --mode strict` violations=0
  - [x] `scripts/check-doc-harness.sh --mode warn` no errors
  - [x] `scripts/check-architecture-harness.sh --mode warn` no errors
  - [x] Master plan Stage 4 row updated to `[x]`
  - [x] Evidence matrix created at `docs/plans/archive/reconciliation/2026-02-22-v3-stage-4-evidence-matrix.md`

---

## Verification Commands (full gate)

Run from worktree root (`.worktrees/v3-implementation`):

```bash
cd v3 && cargo test --workspace
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
cd v3 && cargo fmt --all -- --check
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode warn
```

---

**Review cycles:** 1

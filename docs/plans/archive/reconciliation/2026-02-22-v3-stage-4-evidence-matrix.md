# V3 Stage 4 Evidence Matrix

**Stage:** 4 — Tick + Seeding + Viability E2E
**Date verified:** 2026-02-22
**Subplan:** `docs/plans/2026-02-21-v3-stage-4-tick-seeding-viability.md`
**Test count:** 215 (209 unit + 6 integration viability)
**Verification gate:** cargo test + clippy + fmt + harness scripts all pass

---

## Task 1: CreatureState extension + Mutation module

| Item | Code Ref | Test |
| --- | --- | --- |
| `phenotype_channel_weights: [f32; 3]` field | `src/creature/state.rs:22` | `creature::state::tests::creature_state_position_stored` |
| `phenotype_channel_polarity: [bool; 3]` field | `src/creature/state.rs:24` | `creature::state::tests::new_creature_has_zero_age_and_empty_memory` |
| `CreatureState::new` updated constructor | `src/creature/state.rs:29` | call sites in `src/sensors/static_inputs.rs`, `src/runtime/vm.rs` |
| `MutationSkipReason` enum | `src/mutation/types.rs:4` | `mutation::types::tests::mutation_summary_zero_has_zero_counts` |
| `MutationSummary::zero()` | `src/mutation/types.rs:24` | `mutation::types::tests::zero_summary_accounting_invariant` |
| `MutationEngine::apply_mutations` stub | `src/mutation/engine.rs:15` | `mutation::engine::tests::stub_engine_returns_zero_summary` |
| `mutation/mod.rs` re-exports | `src/mutation/mod.rs` | (compile-time) |

## Task 2: Simulation struct + seeding

| Item | Code Ref | Test |
| --- | --- | --- |
| `Simulation` struct | `src/simulation/simulation.rs:14` | (used throughout) |
| `Simulation::new` constructor | `src/simulation/simulation.rs:28` | `tests/viability.rs:creatures_can_eat_food` |
| `Simulation::creature_count` | `src/simulation/simulation.rs:46` | `simulation::seeding::tests::seeded_simulation_has_creatures` |
| `seed_simulation` function | `src/simulation/seeding.rs:27` | `simulation::seeding::tests::seeded_simulation_has_creatures` |
| Unique founder positions | `src/simulation/seeding.rs:40-60` | `simulation::seeding::tests::founders_placed_at_distinct_positions` |
| Food seeded after world init | `src/simulation/seeding.rs:32` | `simulation::seeding::tests::world_has_food_after_seeding` |
| Deterministic seeding | `src/simulation/seeding.rs:27` | `simulation::seeding::tests::seeding_is_deterministic` |
| Founder phenotype baseline | `src/simulation/seeding.rs:16-18` | `simulation::seeding::tests::founder_phenotype_baseline` |
| Founder energy from config | `src/simulation/seeding.rs:57` | `simulation::seeding::tests::founder_energy_matches_config` |

## Task 3: Phase 0 world updates

| Item | Code Ref | Test |
| --- | --- | --- |
| `run_phase_0` — food growth | `src/simulation/tick.rs:12` | `simulation::tick::tests::phase_0_grows_food` |
| `run_phase_0` — creature aging | `src/simulation/tick.rs:15-16` | `simulation::tick::tests::phase_0_increments_creature_age` |
| `run_phase_0` — energy decay | `src/simulation/tick.rs:17` | `simulation::tick::tests::phase_0_decays_energy` |
| `run_phase_0` — death removal | `src/simulation/tick.rs:20-32` | `simulation::tick::tests::phase_0_removes_dead_creatures` |
| Occupancy cleared on death | `src/simulation/tick.rs:30` | `simulation::tick::tests::dead_creature_removed_from_occupancy` |

## Task 4: Action application

| Item | Code Ref | Test |
| --- | --- | --- |
| `apply_noop` | `src/simulation/actions.rs:22` | (trivial, tested via integration) |
| `apply_eat` energy increase | `src/simulation/actions.rs:27-36` | `simulation::actions::tests::apply_eat_increases_energy_and_clears_food` |
| `apply_eat` energy cap | `src/simulation/actions.rs:34` | `simulation::actions::tests::apply_eat_caps_energy_at_max` |
| `apply_move` position update | `src/simulation/actions.rs:41-59` | `simulation::actions::tests::apply_move_updates_position_and_occupancy` |
| `apply_move` barrier rejection | `src/simulation/actions.rs:48-50` | `simulation::actions::tests::apply_move_into_barrier_no_position_change_cost_deducted` |
| `apply_move` occupied rejection | `src/simulation/actions.rs:48-50` | `simulation::actions::tests::apply_move_into_occupied_cell_no_position_change` |
| `apply_reproduce` child spawn | `src/simulation/actions.rs:128-144` | `simulation::actions::tests::apply_reproduce_creates_child_with_inherited_genome` |
| `apply_reproduce` occupied target | `src/simulation/actions.rs:80-82` | `simulation::actions::tests::apply_reproduce_fails_when_target_occupied` |
| `apply_reproduce` energy check | `src/simulation/actions.rs:93-95` | `simulation::actions::tests::apply_reproduce_fails_when_energy_insufficient` |
| `apply_reproduce` cost deduction | `src/simulation/actions.rs:90` | `simulation::actions::tests::apply_reproduce_deducts_cost_from_parent` |
| `apply_reproduce` pop cap | `src/simulation/actions.rs:85-87` | `simulation::actions::tests::apply_reproduce_at_pop_cap_returns_rejected` |
| `apply_reproduce` generation | `src/simulation/actions.rs:115` | `simulation::actions::tests::apply_reproduce_child_has_correct_generation` |
| `apply_reproduce` memory copy | `src/simulation/actions.rs:114` | `simulation::actions::tests::apply_reproduce_child_inherits_memory` |

## Task 5: Turn queue + full tick

| Item | Code Ref | Test |
| --- | --- | --- |
| `run_tick` — Phase 0 | `src/simulation/tick.rs:54` | `simulation::tick::tests::run_tick_increments_tick_counter` |
| `run_tick` — queue sort+shuffle | `src/simulation/tick.rs:57-59` | (determinism tested via viability) |
| `run_tick` — mid-tick skip | `src/simulation/tick.rs:70-72` | (soft default, covered by no-panic test) |
| `run_tick` — mesh execution | `src/simulation/tick.rs:78-88` | `simulation::tick::tests::run_tick_increments_tick_counter` |
| `run_tick` — tick counter | `src/simulation/tick.rs:116` | `simulation::tick::tests::run_tick_increments_tick_counter` |
| `run_tick` — action dispatch | `src/simulation/tick.rs:91-113` | integration viability tests |

## Task 6: Viability E2E tests

| Item | Code Ref | Test |
| --- | --- | --- |
| 20-tick no-panic | `tests/viability.rs:34` | `sim_runs_20_ticks_without_panic` |
| Population survives 5 ticks | `tests/viability.rs:44` | `population_survives_5_ticks` |
| Food consumed and regrows | `tests/viability.rs:58` | `food_gets_consumed_and_regrows` |
| Founders start with correct energy | `tests/viability.rs:72` | `seeded_founders_start_with_correct_energy` |
| Creatures can eat food | `tests/viability.rs:89` | `creatures_can_eat_food` |
| Deterministic seeding | `tests/viability.rs:159` | `deterministic_seeding_reproducible` |

## Task 7: Quality gates

| Gate | Result |
| --- | --- |
| `cargo test --workspace` | 215 tests: 209 unit + 6 integration — all pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| `cargo fmt --all -- --check` | clean |
| `scripts/check-plan-harness.sh --mode strict` | violations=0, warnings=0 |
| `scripts/check-doc-harness.sh --mode warn` | violations=0 |
| `scripts/check-architecture-harness.sh --mode warn` | violations=0 (5 pre-existing warnings in petri-core, unrelated to Stage 4) |

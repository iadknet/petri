# Fix Food Test Fixtures

**Status**: Implemented
**Last updated**: 2026-09-24
**Scope**: Test-only follow-up to `remove-shared-food-config-copy.md`; no
roadmap feature ID, benchmark, or mutation gate. No production code changes.
Base: `main` at `a9f59893`.

## Goal

Commit `4e1dfbb5` deleted test writes to the retired `world.food.shared`
copy (`cfg.world.food.initial_coverage` / `initial_density` through
`FoodConfig: Deref`). Those writes were already dead: the kernel seeds from
`world.food.types[i].initial_coverage/initial_density`
(`kernel/ordinary_food/ecology.rs` `seed_density`), whose defaults are
coverage `0.54`, density `1.0` (`config/simulation.rs` `FoodTypeConfig::default`;
`max_density` defaults to `1.0`, so every deleted `initial_density = 1.0` write
already matched the effective value). The deleted values record the food the
fixture's author meant to have. Each listed fixture now gets that food through
`types[0]`, or is left alone where its food is never seeded from config or its
assertions do not depend on food.

Initial food is read only by `WorldState::seed_food` (direct calls) and
`seed_simulation` (`simulation/seeding.rs:66`). `WorldState::new`,
`reconfigure_food`, and `Simulation::new` do not seed. `seed_density` shuffles
the candidate list before checking the coverage target, so a coverage change
alters which cells are seeded but not the RNG draw count at seeding.

## Non-Goals

- Production code, including `neighborhood/recruitment_paths/mod.rs`
  `task_config` (its deleted `0.0` write is shadowed by the per-type loop that
  already sets `food.initial_coverage = 0.0` on every type).
- Fixtures already migrated to `types[0]` in `4e1dfbb5` (`world.rs`
  `seed_food_uses_exact_coverage_count`, `sensors/static_inputs.rs` ×2) and
  `bench/profiles/tests.rs` (assertion removal only).
- Weakening any assertion, or explaining behavior shifts beyond recording them.
- Historical records (`docs/progress/**`, readings).

## Per-test decisions

"Deleted write" is the exact line `git show 4e1dfbb5` removed. "Seeded?"
says whether the fixture's food comes from config at all.

| # | Test (file) | Deleted write | Seeded? | Intended food and evidence | Planned change | Pin movement |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `kernel/world.rs` `seed_food_skips_barrier_cells` | `food_cfg.initial_coverage = 1.0; // guarantee all non-barrier cells are seeded` | yes, `seed_food` | coverage 1.0: the deleted comment. At 0.54 the barrier assertion is vacuous for a barrier-seeding bug about half the time | `food_cfg.types[0].initial_coverage = 1.0` with the original comment | none |
| 2 | `runtime/tests/vm_io_memory.rs` `vm_eats_when_food_here` | `initial_coverage = 1.0; initial_density = 1.0` | yes, `seed_food` | 1.0/1.0: comment "Build a world with food everywhere"; asserts `food_here > 0` at (1,1), which at 0.54 holds only by seed luck | `types[0].initial_coverage = 1.0; types[0].initial_density = 1.0` | none |
| 3–4 | `simulation/actions/predation.rs` `make_sim_two_creatures`, `make_sim_one_creature` | `cfg.world.food.initial_coverage = 0.0` | no: `WorldState::new` then a struct-literal `Simulation`; food is never seeded | empty world, already true | no change: food is zero by construction; `apply_steal_energy` never reads food | none |
| 5 | `simulation/tick/tests/phase0.rs` `phase_0_grows_food` | `cfg.world.food.initial_coverage = 0.0` | yes, `seed_simulation` | 0.0: start empty so the before/after difference is pure growth | `types[0].initial_coverage = 0.0`. Measured: passes (growth from recovery spawns with `growth_rate = 1.0`) | none |
| 6–8 | `simulation/tick/tests/support.rs` `make_sim_two_creatures`, `make_sim_with_one_creature`, `make_sim_with_custom_genome` | `cfg.world.food.initial_coverage = 0.0` | no: `reconfigure_food` only | empty world, already true | no change: never seeded | none |
| 9 | `v3-core/tests/applied_trajectory.rs` `accounting_preserves_pre_feature_sampled_trajectories_and_actions` | `initial_coverage = 1.0; initial_density = 1.0` | yes, `seed_simulation` | 1.0/1.0: explicit override among the fixture's size/population overrides | `types[0].initial_coverage = 1.0; types[0].initial_density = 1.0` | digest moves (below) |
| 10 | `v3-core/tests/applied_trajectory.rs` `mutation_on_applied_trajectory_guard_is_pinned` | same | yes | same | same | digest moves (below) |
| 11 | `v3-core/tests/common/mod.rs` `test_config` | `cfg.world.food.initial_coverage = 0.0` | no: all users (`temporal_fixtures.rs`, `creature_workflow_e2e/*`) build worlds with `reconfigure_food` and never seed | empty world, already true | no change: never seeded | none |
| 12 | `v3-core/tests/reproducibility.rs` `reproducibility_config` | `initial_coverage = 1.0; initial_density = 1.0` | yes | 1.0/1.0: module doc "full initial food coverage and density" | `types[0]` 1.0/1.0 | no pin (two-simulation self-comparison). Coverage asserts measured: paired 247→264, backward+forward 151→156, births 1361→1443 |
| 13 | `v3-core/tests/viability.rs` `viability_config` | `initial_coverage = 1.0; initial_density = 1.0` (under the kept comment "Full food coverage to compensate for small world") | yes | 1.0/1.0: the kept comment; file rule 1 permits food coverage/density overrides | `types[0]` 1.0/1.0 | `founder_only_trajectory_digest_is_pinned` moves (below) |
| 14 | `viability.rs` `high_coverage_economics_probe_config` | `cfg.world.food.initial_coverage = 1.0` | yes | 1.0: name and doc "intentionally raises startup food coverage" | `types[0].initial_coverage = 1.0` | none |
| 15 | `viability.rs` `creatures_can_eat_food` | outer `c.world.food.initial_coverage = 0.0`; inner `food_cfg.initial_coverage = 1.0; initial_density = 1.0` | yes, but broken: the world is seeded from `food_cfg` (`types[0].initial_coverage = 1.0`), then `Simulation::new` (`simulation/simulation.rs:48`) reapplies the outer config (`types[0]` 0.54). The catalog compares full `FoodTypeConfig`s (`ordinary_food/mod.rs` `apply_config_transition`), sees a change, and `ecology.rs` `apply_config_transition` clears all density. The assertion `food_after < food_before` then passes because construction erased the food, not because the founder ate | a pre-seeded food cell under the founder (doc: "Place a founder on a cell pre-seeded with food"); the outer 0.0 was never seeded from and only diverged from the seeding config | put `types[0].initial_coverage = 1.0` on the one `cfg`, seed from `cfg.world.food.clone()` so construction sees no config change, and read `food_before` from `sim.world` after `Simulation::new` with a precondition `assert!(food_before > 0.0)`. Strengthens, does not loosen. Measured: the precondition fails on the base fixture (food erased by construction) and passes with the fix; the eat assertion passes | none |
| 16 | `viability.rs` `founder_eats_primary_type_and_preserves_other_food` | `= 0.0` | no: `reconfigure_food` + `set_food_type` | empty world except the set cell, already true | no change | none |
| 17 | `viability.rs` `founder_reproduces_when_energy_allows_and_target_is_open` | `= 0.0` | no: `WorldState::new` + `Simulation::new` | "With no food", already true | no change | none |
| 18 | `viability.rs` `founder_does_not_attempt_reproduce_when_below_min_reproduce_age` | `= 0.0` | no | empty, already true | no change | none |
| 19 | `viability.rs` `founder_moves_when_no_food_and_below_reproduce_threshold` | `= 0.0` | no | "With no food", already true | no change | none |
| 20 | `viability.rs` `mutation_skip_reason_tracking_accumulates_correctly` | `initial_coverage = 0.8; initial_density = 1.0` | yes | 0.8/1.0: set with `growth_rate = 0.5` and `reproduce_cost = 1.0` to get births; asserts `mutation_events_attempted_total > 0` | `types[0]` 0.8/1.0 | none |
| 21 | `v3-core/tests/terrain.rs` `production_sized_startup_has_no_runtime_pattern_area_cap` | `cfg.world.food.initial_coverage = 0.0` | yes | 0.0, likely for speed on a 1600² world | no change: the only assertion is non-empty barriers; food never enters it, and the seeding shuffle (the dominant cost) runs regardless of coverage | none |
| 22 | `v3-core/tests/vm_all_opcodes_e2e.rs` `build_simulation` | `cfg.world.food.initial_coverage = 0.0` | no: `reconfigure_food` + `set_food(pos, food_here)` | only the set cell has food, already true | no change | none |
| 23 | `v3-cli/tests/cli.rs` `default_config` | `initial_coverage = 0.6; initial_density = 1.0` | yes (`run_simulation`) | 0.6/1.0; `tick_sample_mean_generation_rises_once_the_population_reproduces` needs population > 10 at tick 60 | `types[0]` 0.6/1.0 | none (`run_started_identifies_applied_config` digests the same config it runs) |
| 24 | `v3-server/tests/server.rs` `health_payload_contains_mutation_skip_by_reason` | `initial_coverage = 0.8; initial_density = 1.0` | yes | 0.8/1.0: same births-for-mutations setup as #20; its reconciliation assertions are only non-vacuous if mutations occur | `types[0]` 0.8/1.0 | none |

## Pin movement (Rust 1.93.0)

| Pin | Old | New | Cause |
| --- | --- | --- | --- |
| `applied_trajectory.rs` first digest | `dd28049c8c37ae68f52750c300090870190c3b4e98e6acc9328d4eb78a0bee19` | `93ba762bb64d24032df8821f5f0b3f50b5b9c3a29f340aabf7a29186f03fda65` | row 9: coverage 0.54 → 1.0 |
| `applied_trajectory.rs` second digest | `e0f75b16be1581697eb54fe5cae69fe281e5c98fe76fb8db40774dbfcf629f37` | `44ead4e700027eb43138d562a88a3581beb84f002432e37f98bc12b0f97d67f0` | row 10: coverage 0.54 → 1.0 |
| `viability.rs` founder-only tuple, macOS aarch64 | `(323, 0, 54db30adef8c1c9e047f3eeb1e988e2581c6e43b28183d01c5ea23b7800e529c)` | `(199, 0, ad2f93e100d1f7962b68dcb6f35b65d3d6b4972ef0efee15de5b0e4df06dfc52)` | row 13 |
| `viability.rs` founder-only tuple, GNU/Linux aarch64 (`rust:1.93.0` container) | `(323, 0, 57986127fb8baa4fc3859ecf23f4941a0691473c9490f3a3e4b052e3eb9e019a)` | `(199, 0, ad2f93e100d1f7962b68dcb6f35b65d3d6b4972ef0efee15de5b0e4df06dfc52)` | row 13 |

The applied-trajectory pins and the macOS viability tuple were measured on
macOS aarch64: two agreeing runs each, and dropping only the fixture's
`types[0]` lines reproduces all three old values exactly. On GNU/Linux aarch64
the base tree reproduces the old Linux pin, and the fixed tree gives
`ad2f93e1…` on two runs, the same value as macOS. Because the platform values
now coincide, the `cfg!(linux/gnu)` branch is collapsed to one literal
(identical branches would also trip `clippy::if_same_then_else`); the
docstring says to reintroduce it if a split reappears. x86_64 (the CI runner
architecture): see Verification.

Finding (not caused by this change): `applied_trajectory.rs` has single,
macOS-measured pins and is not in the CI matrix (`rust-test-reproducibility` is
absent from `.github/workflows/ci.yml`). On GNU/Linux aarch64 the base tree
already fails its first digest (`dd053d39fa29d094b7c066de174ede203501f8f2a45ac8020dd1055aaa25afee`
vs the pinned `dd28049c…`). Under the fix both digests differ on Linux
(`cd0b1086c2fd20cbfcd3071db9496d660630fd9c5b057f8f387699b600afa721`,
`bf836c460b795808814cb01701b1b6b059bfdfb7d817a683a341d5c271eaac59`). No Linux
branch is added there: the test was already Linux-red and is run only on
macOS.

Finding for the user (row 13): under the intended full coverage, the
founder-only 2,000-tick run on `viability_config` seed 2026 has fewer births
(323 → 199) and goes extinct earlier (tick 890 → 448); peak population is
nearly unchanged (150 → 149) and all reproduce rejections stay `RejectedInvalidTarget`
(272 → 995, crowding). The old pin already recorded extinction
(`final_population = 0`), and every viability behavior test passes under the
intended food. This is recorded, not investigated.

Every re-pin is confirmed by two agreeing runs and by reverting only the
fixture line, which restores the old value.

## Invariants

- No production (non-test) source changes; the only non-test edit is this PRD.
- No assertion is removed, loosened, or re-thresholded. Pinned literals change
  only in the pins listed above, with exact equality kept: the digests, and in
  the founder-only viability tuple the births element too (all three tuple
  elements go through the two-run and revert checks). The one added assertion
  (row 15 precondition) strengthens a test. Pin provenance comments are
  updated beside each re-pin.
- Row 24 keeps its assertions: an added mutation-activity precondition would be
  a new test requirement the author did not write, outside this fixture fix.
- Each fixture change writes the deleted value to `types[0]`, the field the
  seeder reads, and nothing else.
- Deterministic tests stay deterministic: re-pins come from two agreeing runs.

## Verification

1. `cargo test -p v3-core --test viability` (before any edit: passed, 28/28).
2. Targeted runs of every changed test; revert check per re-pinned digest.
3. GNU/Linux aarch64 `rust:1.93.0` container runs of the viability pin and
   `applied_trajectory` (base and fixed trees, separate target volumes).
4. x86_64: unverified. A `--platform linux/amd64` container run failed before
   building (`rustc -vV` crashed with SIGSEGV under emulation). The old
   comment's claim that the aarch64 Linux pin matched x86_64 CI is dropped;
   CI's `rust-viability` job is the first x86_64 measurement of the new pin.
5. `make check` once, logged, exit 0 (log kept in the implementing session's scratchpad as `make-check.log`).

## Review

| Round | Codex job | Verdict |
| --- | --- | --- |
| PRD 1 | `task-mufzkw25-zvwis1` | not-ready: `creatures_can_eat_food` food erased by `Simulation::new` (accepted, confirmed by probe, row 15); advisory tuple re-pin wording (applied); advisory server mutation-activity precondition (rebutted: new requirement outside the fixture fix) |
| PRD 2 | `task-mufzs34k-hcg0i5` | ready; advisory "unchanged" → "nearly unchanged" peak population (applied) |
| Implementation 1 | `task-mug11k8z-gnogmm` | no findings; could not verify runtime results (test outcomes, digests, x86_64 CI) |

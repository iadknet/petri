# T24.F01 — Order-Free Food Spread

**Status**: In Progress
**Last updated**: 2026-10-07
**Feature**: T24.F01
**Track**: [T24 — Simulation Throughput](../../roadmaps/t24-simulation-throughput.md)

## Goal

Seeds disperse evenly in every direction. Food spreading from a dense cell
keeps its share whichever neighbour it lands on: in `grow`
(`crates/v3-core/src/kernel/ordinary_food/ecology.rs`) a cell that already
holds food adds its own growth to the spread it has already received in the
tick instead of overwriting it, so a north or west neighbour's contribution
is no longer discarded and food inside patches no longer drifts north-west.
This is the rule `docs/reference/v3-world-grid-spec.md` §5 already states
(local growth and spread both add). The natural analog is dispersal without a
compass bias; it reaches creatures through the world's standing food, not
through a sensor. The repair's ecological effect is read on the gate and goal
profiles on its own, before T24.F02 parallelizes the pass.

## Non-Goals

- No change to which random draws the tick makes or in what order; the food
  stream, the gather form and parallelism are T24.F02.
- No fixed summation order and no change to how draws are assigned to cells:
  both order dependences of invariant 2 stay until T24.F02 replaces them.
- No new `SimulationConfig` or `RuntimeConfig` field, default, telemetry
  family or frontend change.
- No edit to the reference spec (§5 is already the target rule) and no
  recalibration of the T02.F04 grazing settings or the T01.F11 persistence
  numbers, both measured with the bug (track Notes).
- No change to the `suppressed_by_*` telemetry sums or inhibition flags; their
  arithmetic does not touch the overwritten value.

## Inputs and Invariants

| Input | Where | What F01 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t24-simulation-throughput.md`, row T24.F01, Notes "T24.F01, the spread bug" and "Every T24 feature runs the ordinary gate…" | The goal; the diagnosis (assign overwrites spread added by every neighbour with a lower row-major index); the fix (accumulate, no stream change mixed in); the TDD fixture; the predeclaration that gate and goal trajectories move; no Benchmark gate exemption. |
| Reference rule | `docs/reference/v3-world-grid-spec.md` §5 "Growth" | `food_density[cell] = clamp(food_density[cell] + delta * occupancy_multiplier)` and spread "add … (clamped to `max_density`)". |
| The pass | `ecology.rs` `grow`, growth and spread loop (lines 459–550 at 4743416a): `next_density[idx] = (source + local_delta).clamp(...)` at line 495 is the assignment; spread adds at 547–548; recovery adds at 597; write-back at 614–624 | The one line that changes and the two that already accumulate. Reads of density come from `state` (the pre-growth snapshot), never from `next_density`. |
| Visit order | The loop is `for y { for x }` with `idx = y * width + x`; `resolve_neighbor_static` wraps under `WorldEdgeMode::Wrap` (the default) | Today a target keeps spread only from sources visited after it (its south and east neighbours, and a west or north neighbour across a wrapped edge); it loses spread from sources visited before it (north, west, and an east or south neighbour across a wrapped edge), because its own assignment at line 495 runs after theirs. From the source's side: spread sent south or east, or west or north across a wrapped edge, lands on a higher-index target and is lost. Spread into an empty cell (`source <= 0.0`, `continue` at 471) is never lost, and a lost contribution is observable only where the target has headroom below `max_density` after its own growth. |
| Term signs | `growth_rate.max(0.0)`, fertility and inhibition `.max(0.0)`, `spread_density_ratio.clamp(0.0, 1.0)`, depletion and grazing multipliers in `[0, 1]`, `spawn_delta = max_density * growth_rate` | Every contribution is non-negative, so clamping each partial sum to `max_density` equals clamping the total (invariant 2). |
| Contribution size | `spread_delta = source × growth_rate × cell_fertility × spread_density_ratio × neighbour_fertility × target_multiplier` (lines 487–541); `crates/v3-core/src/config/simulation.rs`: `growth_rate` 0.09, `spread_threshold_ratio` 0.8, `spread_density_ratio` 0.25, `max_density` 1.0; fertility enabled by default with `min_fertility` 0.0 and `max_fertility` 2.0 (`FertilityConfig::default`, and the three goal worlds); goal worlds set per-type `growth_rate` 0.05 and 0.12 | One contribution is at most `4 × growth_rate × spread_density_ratio` of the source per tick before depletion and inhibition: 9% at the shared defaults, 5% and 12% at the goal worlds' per-type rates; it is sent only by a cell at or above 0.8 × `max_density`. |
| Call sites and stats | `crates/v3-core/src/simulation/tick.rs` `run_phase_0_food_growth`; `stats.rs` `record_food_growth_summary` | `grow_food` is called once per tick with `sim.rng`; the summary's `per_type.total_density` and `occupied_cells` feed `last_tick_food_total_density_by_type` and the benchmark's `food_density_total` samples. |
| Comparison rule | T10 track Notes (thresholds: flag above +10% work per creature-tick or +25% wall; severe above +50% work or a doubling of wall; a predeclared, justified cost re-pins the epoch in the closing commit); `crates/v3-cli/src/bench/comparison.rs` (`COUNTER_NAMES`, only counters can be severe); `crates/v3-cli/tests/bench.rs` `gate_profile_has_no_severe_regression_against_series_references` (inside `make check`) | The Performance predeclaration and the closure order (re-pin before the mutation gate). |
| References | Previous closure T24.F03: `docs/progress/features/t24-f03-food-growth-phase-timing.json` (gate: seeds 11/22/33 final population 2226/2170/2163) and `-goal.json` (Orchards 11: final 4088, plateau 3706.5, minimum 18; Canyon 22: 3221; Confluence 33: 1219; no extinction); epochs T11.F25 (gate) and T11.F27 (goal) in `docs/progress/benchmark-series.json` | The readings this feature's summaries are compared with. |
| Determinism and viability tests | `crates/v3-core/tests/reproducibility.rs` (two seeded runs byte-identical, with operator-coverage assertions `paired > 0`, `backward + forward > 0`, `compared_mutated_descendant`; terrain across thread counts), `tests/viability.rs` (header rule: fix the config or behavior, never the tests; `founder_only_trajectory_digest_is_pinned` hashes the founder trajectory and the food grid and pins `(199, 0, digest)`) | Byte-identity and survival assertions are unchanged; the trajectory digest is expected to change and is re-pinned once with evidence (task 1); the operator-coverage assertions may need a new seed or horizon. |
| Trajectory pins | `crates/v3-core/tests/applied_trajectory.rs` (`accounting_preserves_pre_feature_sampled_trajectories_and_actions`, `mutation_on_applied_trajectory_guard_is_pinned`), `tests/baseline_worlds.rs` (`legacy_default_short_run_identity`), `crates/v3-cli/src/bench/tests.rs` (`the_real_run_path_carries_a_full_occupancy_grid_at_every_checkpoint`: seed 1, horizon 250, 32-by-32, 8 founders) | Each file's own rule: a pin is updated only after an intentional behavior change has been reproduced across two agreeing runs; the bench fixture reselects its seed by its comment's rule (the lowest seed in 0..=25 that keeps at least 2 creatures over at least 2 cells at the horizon). All four move with the repaired rule and are re-pinned once (task 5). |

Invariants:

1. Per-cell rule. For each food type and each non-barrier cell, the density
   written back is `clamp(own + Σ spread + Σ recovery, 0, max_density)`, where
   `own = source + local_delta` when `source > 0` and 0 otherwise, `Σ spread`
   sums every `spread_delta` whose drawn target is the cell, and `Σ recovery`
   sums every recovery attempt that landed on it. No contribution is
   discarded. Barrier cells are written 0 and are never a target.
2. Accumulation-order independence. Within a tick every density read comes
   from the pre-growth state, and `next_density` is only ever added to, so for
   the targets as drawn (invariant 3) the written value does not depend on
   the order in which contributions are accumulated, except for `f32`
   rounding of the partial sums; incremental clamping is exact because every
   term is non-negative. Two order dependences remain and are kept until
   T24.F02: the draws are assigned to spreading cells in row-major visit
   order (a different visit order would change targets, not sums), and the
   partial sums are accumulated in that order (rounding-level). T24.F02
   replaces both with a keyed per-cell stream and a fixed neighbour order.
3. Draws unchanged. For a given pre-growth state the tick makes the same
   `rng` draws in the same order as the parent build: for each food type in
   catalog order, one `gen_range(0..count)` per spreading cell in row-major
   order, then that type's recovery draws. Trajectories diverge across ticks
   only because the kept spread changes later states.
4. Telemetry meaning unchanged. `growth_suppressed_by_*`, `cells_with_type_inhibition`
   and the grazing fields are computed as before; `total_density` and
   `occupied_cells` now include the kept spread.
5. No field, default or digest change: the effective-config digests of both
   summaries equal T24.F03's; the pinned recipe digests do not move.
6. Reproducibility: seeded runs stay byte-identical across processes and thread
   counts; the diff adds no thread-dependent path.
7. Cost: one `f32` load and add per occupied cell per type per tick, and
   nothing else on the timed path.

## Telemetry

No new signal, configuration or trace kind. The existing food gauges and the
`food_growth` phase derive from the repaired behavior and move with it:

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.food_total_density{petri.food_type}` (existing) | gauge, sampled at the snapshot tick | `last_tick_food_total_density_by_type` from `FoodGrowthSummary.per_type.total_density`, which now includes kept spread | the `food_growth` span's config group (`world.food.shared.growth_rate`, `max_density`, `grazing.enabled`, `occupancy_depletion.enabled`, per-type `growth_rate`); no switch is added |
| `petri.tick.food_grazed_cell_share{petri.food_type}`, `food_grazing_modifier_mean`, `food_growth_suppressed_by_*`, `food_cells_with_type_inhibition` (existing) | gauges | unchanged arithmetic on a changed trajectory | as above |
| `petri.run.phase_wall_clock{petri.phase="food_growth"}` and the `growth_spread` pass spans (existing, T24.F03) | sum; span | the same pass with one extra add per occupied cell | as above |

Benchmark summaries carry `food_density_total` per sample and per type in the
persistence samples of each goal case and the gate (T01.F11 cadence, every
100 ticks); these are the readings the Performance section compares.

## Implementation Tasks

- [x] Run `cargo test -p v3-core --test viability` first (tick-loop mechanics
      change) and record its result. The survival tests keep their seeds,
      horizons and assertions under the file's header rule; a survival
      failure on the repaired rule is a behavior finding reported under the
      blocker rule, never a re-seed. `founder_only_trajectory_digest_is_pinned`
      is expected to fail on the repaired rule because it hashes the food
      grid: after the fixture, oracle and property tests are green it is
      re-pinned once, with the before and after `(births, population, digest)`
      triples in the readings.
- [x] TDD in `ecology.rs` tests, with fertility, inhibition
      (`growth_inhibitor` 0), depletion, grazing and recovery off and
      `max_density` leaving headroom (the `spread_target_delta_with_grazing`
      helper's settings): a growth pass on a `Bounded` 2-by-1 world where the
      west cell is dense and the east cell holds food below the spread
      threshold fails today (the east cell ends at `target × (1 + g)`) and
      passes after the fix (`target × (1 + g) + source × g ×
      spread_density_ratio`); the mirrored layout (dense east cell, occupied
      west cell) gives the same target value, so the two orientations agree.
- [x] Per-cell oracle test in `ecology.rs` tests: on a `Wrap` world with two
      food types, a barrier cell and recovery spawns enabled, and fertility,
      inhibition, depletion and grazing off so the contributions are the
      plain `source × g`, `source × g × ratio` and `max_density × g` terms,
      one growth pass with a seeded `StdRng` writes the grid that a
      test-side gather computes from the same draws (for each food type in
      catalog order, one `gen_range(0..count)` per spreading cell in row-major
      order, then that type's recovery draws) under invariant 1, with the
      clamp; the fixture is chosen so that a delivery
      across the wrapped edge and a binding clamp occur, and the test asserts
      that they did. Fails today on the lost deliveries.
- [x] Property test (proptest, v3-core): with fertility, depletion, grazing,
      inhibition and recovery off, `max_density` large enough that no clamp
      binds, and a spread threshold some drawn densities exceed, one growth
      pass on a random grid (at least 2 by 2, no barriers) with random
      densities ends with a total equal to `Σ source × (1 + g) + Σ_{source ≥ threshold} source × g ×
      spread_density_ratio` within `f32` tolerance, whatever the seed draws
      for each spreading cell's target; its assertions do not depend on which
      cases were drawn. The property fails today whenever a draw lands on an
      occupied higher-index target (one visited later).
- [x] Fix: at line 495 accumulate, `next_density[idx] = (next_density[idx] +
      source + local_delta).clamp(0.0, max_density)`; nothing else in the loop
      changes. Existing growth tests in `world.rs` and `ecology.rs` keep
      passing or have their expected values re-derived from the §5 rule, never
      loosened. The operator-coverage assertions in `reproducibility.rs` are
      re-seeded or given a longer horizon only if the repaired trajectory no
      longer satisfies them; the byte-identity assertions are never weakened,
      and any such change is recorded in the readings. The three trajectory
      hashes in `applied_trajectory.rs` and `baseline_worlds.rs` are re-pinned
      once to the value two runs on the repaired rule agree on, with the
      before and after values in the readings and a one-line note in each
      file's pin comment naming T24.F01 as the move. The bench fixture
      `the_real_run_path_carries_a_full_occupancy_grid_at_every_checkpoint`
      reselects its seed by its comment's rule (lowest seed in 0..=25 keeping
      at least 2 creatures over at least 2 cells at horizon 250), its comment
      and the readings record the new seed and how many of the 26 seeds go
      extinct or end on one cell under the repaired rule (5 and 11 at
      T19.F04); no assertion, horizon or world size changes.
- [ ] Benchmark gate and goal profiles, the readings table (food totals and
      persistence per world against T24.F03), the series entry and the epoch
      step of the Performance table, then the mutation gate.

## Verification

- [x] `cargo test -p v3-core --test viability` passes before any edit and,
      after the one digest re-pin, after the fix; both results and the
      before/after pin triples in the readings.
- [x] The 2-by-1 fixture, the per-cell oracle test and the property test
      each fail before the fix and pass after; `cargo test -p v3-core` passes.
      Recorded outputs in [`docs/progress/readings/t24-f01.md`](../../progress/readings/t24-f01.md).
- [x] `cargo test -p v3-core --test reproducibility` passes.
- [ ] `make check` exits 0, including
      `gate_profile_has_no_severe_regression_against_series_references`
      against the series as left by the epoch step below.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The
      survivor list stays here.
- [x] Benchmark summaries stored at
      `docs/progress/features/t24-f01-order-free-food-spread.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** A correctness repair of an
existing mechanism. Natural analog: dispersal with no compass bias; the
repaired food reaches creatures through the world's standing food, through no
new sensor or field.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure T24.F03 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| What moves and why | Both trajectories move from the first tick in which a spreading cell's draw lands on an occupied higher-index target (one visited later) that still has headroom below `max_density` after its own growth. Mechanically, such a cell now keeps the spread of all four neighbours instead of the two whose sources are visited after it (Visit order row), so regrowth inside and at the edge of grazed patches is faster; a saturated cell and an empty cell are unaffected. One-tick bound (Contribution size row): a kept contribution is at most 9% of its source at the shared defaults, 5% or 12% at the goal worlds' per-type rates, sent only by cells at or above 0.8 × `max_density`, so standing food in a world changes by percent-scale amounts per tick of regrowth, and the trajectory effect compounds through consumption and births over 2,000 ticks. |
| Forecast bands | Numerical forecasts for the readings, stated as bands with an explicit miss rule, not thresholds. Food: `food_density_total` per type at the final sample of each goal world and gate seed within −10% to +30% of T24.F03's. Persistence: plateau and final population per world within −30% to +50% of T24.F03's; `minimum_population` not read against a band (Orchards' trough is 18 at T24.F03 and seed-fragile). Extinction: `extinction_tick` stays null in all three worlds. Miss rule: a reading outside its band, or a food decrease at the final sample, is a recorded miss in the Measured verdict and the readings and does not block; an extinction in a world that persisted at T24.F03 is a world-compatibility question for the user under the blocker rule. |
| Counters | The diff touches no creature code path, so every `per_creature_tick` move is a population-composition effect of the trajectory. Expected: `mesh_hops`, `vm_steps`, `graph_relax_iters`, `actions_applied`, `passes` within ±25% of T24.F03 on both profiles (work of the typical creature-tick under a modest composition shift); `births`, `plasticity_updates`, `decided_passes`, `pass_cap_hits` may flag or go severe on either profile (small denominators and the evolved-lineage share; `plasticity_updates` went severe at the T19.F02, T11.F19 and T17.F02 trajectory moves), and the gate's `decided_passes` (0.000000 at T24.F03) goes severe under the zero-baseline rule (`comparison.rs:542`) if the repaired gate run makes any decided pass. **Acceptance scope and epoch step:** a severe level on any counter, named above or not, is this feature's predeclared and justified cost only when the closing diff's production change is confined to the accumulate at `grow` line 495 (with tests, the digest re-pin, summaries and documents), which the reviewer checks against the diff; that profile's epoch (`gate` or `goal_worlds` `epoch_baseline`) then re-pins to this feature's summary in the closing commit, before the mutation gate, so `gate_profile_has_no_severe_regression_against_series_references` is green at the mutation baseline. A severe level with any other production change in the diff is unexplained and a blocker under the gate's P1 rule. A profile with no severe level keeps its epoch (T11.F25 or T11.F27) and its summary is only appended to `closed`. A band miss on a non-severe counter is recorded. No user decision is needed on either branch. |
| Wall clock | Two effects, predeclared apart. (a) Phase cost: the repaired pass does one extra add per occupied cell per type, so `food_growth_ms` per tick, read per seed from `environment.phase_wall_clock_ms_per_seed` over the seed's ticks, is expected within ±10% of the stored T24.F03 summaries on the same host: 0.351 ms per gate tick (26.35 / 26.21 / 26.44 ms over 75 ticks) and 53.6 ms per goal tick (Orchards 65.6, Canyon 38.9, Confluence 56.2 ms per tick over 2,000 ticks). (b) Denominator: the comparison is wall per creature-tick, and the food pass scans the grid whatever the population, so a trajectory with fewer creature-ticks raises the ratio with no change in phase cost; a flag (above +25%) is possible on that route if the population falls. A wall flag or doubling is attributed before it is classified: with (a) within ±10% and `creature_ticks` per seed accounting for the ratio, it is the trajectory's and is recorded as this feature's justified cost, and an accepted wall doubling re-pins that profile's epoch in the closing commit under the T10 rule even when every counter is non-severe; with (a) beyond +10%, the profile is rerun under `scripts/bench-wait` on an idle host, and a phase-cost rise that persists is a blocker, not a predeclared cost. |
| Indicators | No diversity or cognition measure is introduced; none remains `Undefined` because of this feature. Observation caps unchanged. |
| Overhead check | Not run. The workflow's Benchmark gate introduces the overhead check as one of three track checks that "stand in for the skipped gates" of an exempt T21 or T23 feature; T24.F01 is exempt from nothing and runs the gate and goal profiles, which measure its cost directly, and the ceiling ratio compares two builds of the same commit that both carry the add, so it cannot read this feature. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements; the existing `petri-telemetry` stack
is not a competing server and stays up throughout (user ruling, 2026-10-07):

```sh
make bench PROFILE=gate FEATURE=t24-f01-order-free-food-spread
make bench PROFILE=goal FEATURE=t24-f01-order-free-food-spread
```

**Measured verdict** (2026-10-07, code 924ef77b, production diff confined to
`ecology.rs`, host load 12–15, `bench-wait` not blocking). The predeclaration
above is unedited; each row applies its rules to the stored summaries.

| Profile | Result |
| --- | --- |
| Gate | CLI exit 0, outer exit 0, `severe` false vs T11.F25 and T24.F03. `vm_steps` flagged +33.87% (0.048411 vs 0.036163): a band miss on a non-severe counter, recorded. `decided_passes` 0 on both sides (zero-baseline case not triggered). Wall −0.58% per creature-tick; `food_growth_ms` per tick +0.9% to +3.3% (within ±10%). Epoch kept at T11.F25; summary appended to `closed`. |
| Goal | CLI exit 3 (severe), outer exit 0, `severe` true vs T11.F27 and T24.F03: `plasticity_updates` +89.19% (0.852171 vs 0.450435), predeclared as possibly severe; `pass_cap_hits` flagged +40.55%, predeclared; `vm_steps` −58.24% (1.004918 vs 2.406467), a band miss on a non-severe counter (a fall is never severe), recorded; the other four counters within ±25%. Wall −16.59% per creature-tick with `creature_ticks` up 20–29%; `food_growth_ms` per tick −3.8% to −4.6% (within ±10%): the wall rule's (a) holds and (b) explains the ratio. Severe accepted under the acceptance scope (diff confined to the one accumulate, reviewer-checked): `goal_worlds` `epoch_baseline` re-pins to this feature's goal summary in the closing commit, before the mutation gate. |
| Forecast bands | Hits: no extinction anywhere; gate persistence +3.5% to +5.2%; gate food seed 22 +10.9%; Orchards final population +21.8% and plateau +30.9%; Confluence plateau +19.3%; type-1 food unchanged (saturated). Misses, recorded under the miss rule and not blocking: goal type-0 food at the final sample Orchards −17.9%, Canyon −43.9%, Confluence −4.1%; gate food seeds 11 −12.9% and 33 −8.4%; final population Canyon +61.9% and Confluence +67.4%; plateau Canyon +110.5% (all above the +50% band). Reading, not a claim: the forecast took "more kept spread" to mean more standing food, but the kept regrowth is eaten; populations rose 22–67% and standing type-0 food fell in step, the direction the plateau rise implies. The paired per-world readings are the reference T24.F02 holds its trajectories against. |

The user decision clauses of the predeclaration (extinction in a persisting
world, a phase-cost rise beyond +10%, a severe with any other production
change) were not reached; no user decision was needed at closure.

- Gate: `make bench` exit 0, `severe=false` vs T11.F25 and T24.F03; flag `vm_steps` +33.87%
  (outside the +-25% band, recorded); no wall flag. Epoch T11.F25 kept; summary appended to `closed`.
- Goal: `make bench` make 2 (v3-cli exit 3), `severe=true` vs T11.F27 and T24.F03; severe
  `plasticity_updates` +89.19% (predeclared), flag `pass_cap_hits` +40.55%, `vm_steps` -58.24%
  (outside the +-25% band, recorded); no wall flag. Goal-worlds epoch not yet re-pinned (orchestrator
  step, after the reviewer confirms the production change is confined to the accumulate).
- Bands: food misses (decreases) Orchards -17.9%, Canyon -43.9%, Confluence -4.1%, gate seeds 11 and 33
  -12.9% and -8.4%; persistence misses Canyon final +61.9% and plateau +110.5%, Confluence final +67.4%;
  all other persistence readings hit; `extinction_tick` null everywhere (hit).
- Wall: `food_growth_ms` per tick within +-10% on every seed (gate +0.9% to +3.3%, goal -3.8% to -4.6%).

- Summaries: [gate](../../progress/features/t24-f01-order-free-food-spread.json),
  [goal](../../progress/features/t24-f01-order-free-food-spread-goal.json).
- Full readings: [`docs/progress/readings/t24-f01.md`](../../progress/readings/t24-f01.md).

## Success Criteria

- [ ] `grow` implements invariant 1: the fixture, the per-cell oracle test
      and the property test pass, and the two orientations of the fixture
      agree.
- [ ] Draws, config digests and the reference spec are unchanged (invariants
      3 and 5); reproducibility and viability tests pass.
- [ ] Gate and goal summaries are stored with the readings against T24.F03,
      the epoch step is applied as predeclared, and `make check` and the
      mutation gate pass on the closing commit.
- [ ] Track criterion 2 holds: every spread contribution reaches its target
      whatever order cells are visited in, and the ecological effect is read
      on both profiles on its own.

## Notes for AI Agents

- Decision: the user ordered T24 as T24.F03, then T24.F01, then T24.F02
  (2026-10-06); T24.F02 builds its gather form on this repaired rule.

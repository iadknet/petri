# T26.F02 — Evolvable Energy Reserves

**Status**: In Progress
**Last updated**: 2026-10-21
**Feature**: T26.F02
**Track**: [T26 — Body Traits](../../roadmaps/t26-body-traits.md)

## Goal

Fat: a creature inherits how much energy it can store. The body record
carries a `capacity` in energy units, bounded to [50, 800], mutated in log
space at birth; the energy cap is the creature's own capacity, a larger body
pays more per move and per tick, a smaller one less, and at the founder's
200 both rules are today's. The goal and goal-long runs report the capacity
distribution, boundary mass and trend per world; the server and inspector
report each creature's own capacity.

## Non-Goals

- No new sensor input key (invariant 6 changes only the denominator).
- No change to the litter rule, `min_reproduce_energy`, gestation, the eat
  and predation cap sites' order and bookings, or the noop floor.
- No lab assay (invariant 9's arms are goal-world runs) and no per-creature
  capacity in the detail-view snapshot payload.

## Inputs and Invariants

Inputs: the T26 row and Notes (design note §3 D; review findings 5, 7, 11);
T26.F01's Goal, Inputs and Invariants and predeclaration; the options and
audit in [`docs/progress/readings/t26-f02.md`](../../progress/readings/t26-f02.md).
The code changed, on main at `08b1584c`:

| Site | Today |
| --- | --- |
| `creature/body.rs` | `HeritableBody { phenotype, identity }`, `Copy + Eq`; `founder(index, seed)`; `inherit(&parent, mutated, &PhenotypeConfig, rng)` draws phenotype then kin tag when `mutated`, nothing otherwise |
| `creature/state.rs:293-303` | `energy_cap` returns `lifecycle.max_energy`; `move_effort` returns `action_effort(costs.move_cost, complexity, age)` |
| `simulation/tick.rs:149-216` | Phase 0 charges `decay + carry × genome_size` as one f32 subtraction, books it by `decay_partition` (decay first), attributes a death to `LifecycleDecay` when `before − decay ≤ 0`, else `GenomeCarrying` |
| `simulation/actions/reproduction.rs:255-262,532-561` | the litter is `min(fraction × after_cost, default_offspring_energy)`, refused under `initial_energy`; the child is constructed with it uncapped and `offspring_energy_credit` books it |
| `sensors/static_inputs.rs:35-66,148-149`, `sensors/perception.rs:141,167`, `runtime/inputs.rs:87-90`, `sensors/reducers.rs:386` | `EnergyCurrent`, `EnergyConsumedThisTick`, the two `PreviousOutcome` energy channels and nearby `ENERGY_RATIO` divide by `lifecycle.max_energy` (200) |
| `config/simulation.rs:414,468,994` | `max_energy` default 200, normalized finite `≥ 1`, fallback 200; the goal recipes set none, the root `world-recipe-*.json` set 200, `blobs-and-fruit-substrate.json` 400 |
| `v3-lab/src/arena.rs:74,93`, `tests/exploration_probe.rs:794` | the birth sentinel `min_reproduce_energy = max_energy + 1`; `start_energy ≤ max_energy`; a refund capped at `max_energy` |
| `simulation/seeding.rs:89-101` | founders are provisioned `initial_energy`, normalized finite `≥ 0` with no upper bound |
| `v3-server/src/http/creature.rs:484` | detail `max_energy` is `creature.energy_cap(..)` |
| `frontend/src/components/WorldViewport.tsx:120` | the canvas energy bar divides by the server config's `energy.lifecycle.max_energy` |
| `v3-cli/src/bench/tracking.rs:773-800,1127-1160` | `PersistenceSample` carries population means per sampled tick from `PopulationReadings` |
| `v3-telemetry/src/metrics.rs:571-640`, `trace.rs:249-257` | flows export as `petri.run.energy_flow{petri.flow}`, deaths as `petri.run.deaths_by_cause{petri.cause}` over `DeathCause::ALL`; the `world_update` phase group carries the decay and carrying values |

Invariants:

1. **Configuration.** New `energy.capacity: EnergyCapacityConfig` and
   `mutation.capacity: CapacityMutationConfig`; `energy.lifecycle.max_energy`
   becomes the range's upper bound and stays the v3-lab birth sentinel.
   Normalization runs in the table's order; the `sensor_scale`, β and u
   defaults are the comparison's and the sweep's picks (invariants 6 and 9,
   ruled 2026-10-21).

   | Field | Default | Normalization (else the fallback) |
   | --- | --- | --- |
   | `energy.lifecycle.max_energy` | 800.0 | finite `≥ 1`; else 800 |
   | `energy.capacity.reference` (the neutral point) | 200.0 | finite in `[1, max_energy]`; else `min(200, max_energy)` |
   | `energy.capacity.min` | 50.0 | finite in `[1, reference]`; else `min(50, reference)` |
   | `energy.capacity.founder` | 200.0 | finite in `[min, max_energy]`; else `reference` |
   | `energy.lifecycle.initial_energy` | 20.0 | finite in `[0, founder]`; else `min(20, founder)`: a founder is never provisioned above its capacity |
   | `energy.capacity.move_cost_slope` (β) | 1.0 (sweep pick) | finite in `[0, 1]`; else the default |
   | `energy.capacity.upkeep_per_unit` (u) | 0.002 (sweep pick) | finite `≥ 0`; else the default |
   | `energy.capacity.sensor_scale` | `Reference` (comparison ruling) | `Reference` or `Own` |
   | `mutation.capacity.chance` | 0.1 | in `[0, 1]`; else the default |
   | `mutation.capacity.log_step` | 0.25 | finite `≥ 0`; else the default |

2. **The trait.** `HeritableBody` gains `capacity: f32` and drops `Eq`.
   `founder(index, seed, capacity)` takes the founder capacity; seeding and
   the lab pass `energy.capacity.founder`. `inherit(&parent, mutated,
   &MutationConfig, &EnergyConfig, rng)` keeps F01's gated
   phenotype-then-kin-tag draws, then on every birth draws
   `rng.gen::<f32>() < mutation.capacity.chance`; when it fires it draws
   `δ = rng.gen_range(-log_step..=log_step)` and sets
   `capacity = (parent.capacity × δ.exp()).clamp(min, max_energy)`; a miss
   draws nothing more. The child's capacity is drawn at delivery (step 11),
   after the litter was deducted at conception.
3. **The cap rule.** `energy_cap` returns `self.body.capacity`; the three
   credit sites and the detail response read it unchanged. Capacity stays in
   `[min, max_energy]`, so `energy ≤ max_energy` and the lab sentinel hold.
4. **The move rule.** `move_effort` returns
   `action_effort(costs.move_cost × factor, complexity, age)` with the
   factor `HeritableBody::move_factor(&self, &EnergyCapacityConfig)` =
   `(1 − β) + β·(capacity/reference)`, computed in `f64` and converted to
   `f32`: at `capacity = reference` it is exactly 1.0 for every `f32` β, and
   over the normalized ranges it is positive, which the `f32` form
   `1 + β·(ratio − 1)` is not. A property test over the normalized ranges,
   including `reference = 2²⁴, min = 1, β = 1`, asserts positivity, the noop
   floor, and today's effort bit for bit at the reference.
5. **Upkeep.** Phase 0 computes today's `f32` sum `base = decay + carry ×
   genome_size`, the signed upkeep `HeritableBody::upkeep(&self,
   &EnergyCapacityConfig)` = `u·(capacity − reference)`, and charges one
   `f32` subtraction of `max(base + upkeep, 0)`; at the reference the upkeep
   is `0.0` and the charge is today's bit for bit. `Phase0Charge { base,
   upkeep }` replaces the inline charge and attribution.
   `lifecycle_decay` and `genome_carrying` are booked by today's
   `decay_partition(before, before − base, decay)`, byte for byte today's at
   the reference; the new signed flow `capacity_upkeep` takes
   `applied − decay − carrying` (negative for a small body; the zero floor's
   shortfall lands here). Attribution uses the charge's rounded
   intermediates: `LifecycleDecay` when `before − decay ≤ 0` (today's test),
   else `GenomeCarrying` when `before − base ≤ 0`, else the new
   `DeathCause::CapacityUpkeep`, so a creature at the reference is never
   attributed to upkeep; the fixture `before = 1`, `decay = 2⁻²⁵`,
   `carry × size = 1 − 2⁻²⁴` pins it.
6. **Sensor scale.** Every energy read — `EnergyCurrent`,
   `EnergyConsumedThisTick`, the `EnergyDelta` and `DamageDelta` channels of
   `PreviousOutcome`, nearby `ENERGY_RATIO` — divides by one per-creature
   denominator, `StaticInputs::energy_scale` (the nearby read takes the
   target's): under `sensor_scale = Reference` it is
   `energy.capacity.reference` for every creature, so the trait changes
   absolute storage only (the founder's 0.16 gate opens at 32 energy at
   every capacity; above the reference reads 1.0); under `Own` it is the
   creature's capacity (the gate opens at 8 energy at C = 50, 128 at
   C = 800). The absolute reproductive condition is unchanged under both.
   The rule, fixed before invariant 9's comparison: `Own` becomes the
   default only if, at both bounds in all three worlds, the `Own` arm
   persists to tick 2,000 with births per reproduce attempt at or above half
   the C = 200 arm's (zero attempts or births in any arm or control world
   fails); otherwise `Reference` does, the other value staying as an
   attribution control. Ruling (2026-10-21, readings file): `Reference` is
   the default; the C = 50 `Own` arm persisted but reproduced at 1.7% /
   3.9% / 1.7% of the C = 200 arm's births per attempt (Orchards / Canyon /
   Confluence), the C = 800 `Own` arm at 103% / 116% / 112%, and the C = 800
   `Reference` arm matched C = 200 in every number (mean energy 22–29).
7. **Birth.** `cap_energy` becomes one cap primitive that applies the body
   cap and returns the applied excess; the three credit sites book it to
   `maximum_energy_clamp_loss` in their present order, and `spawn_child`,
   after constructing the child with the litter, books it to the new flow
   `offspring_capacity_overflow`; `offspring_energy_credit` books the capped
   energy. At any tick
   `parental_transfer_debit = offspring_energy_credit + offspring_capacity_overflow + gestation_lost + pending litter stock`
   (living creatures' `pending_birth.litter_energy`), within the `f32`
   rounding of each conception's debit; exactly
   representable fixtures pin it exactly for a delivered, a lost and an
   in-gestation birth. The required fixture: a parent at the reference
   delivers a litter of 100 to a child of capacity 50; the child starts at
   50, the overflow books 50, the credit 50.
8. **Determinism and pins.** The capacity draws are the only new draws,
   after the kin-tag draw; no tick-phase order changes. Predeclared by
   artifact: every pin that depends on a trajectory with births moves
   (`applied_trajectory.rs`, `reproducibility.rs` fingerprints, the gate's
   deterministic blocks, the viability and baseline-world digests, seeded
   fixtures) and the config digests move with the new fields; a re-pin is
   accepted only when disabling the capacity draw restores the old value,
   which its comment records. Mutation-engine draws and the recruitment-paths
   slow-tier readings (one-tick scenes, no births) are unchanged.
   `tests/viability.rs` passes with its assertions unweakened; its GNU/Linux
   x86_64 digests are measured on the `ubuntu-24.04` CI runner before
   closure, never left stale.
9. **Coefficients.** The comparison runs first, invariant 6's ruling
   follows, then the sweep under the ruled scale. Choice rule for β and u:
   among pairs at which all three worlds persist to tick 2,000 with plateau
   population at or above 70% of the T27.F01 goal epoch's, the largest
   `β × u` wins, ties to the larger β; if none qualifies, (0.5, 0.0005) is
   taken and the breach recorded. Result (readings file, `81202517`): only
   (1.0, 0.002) qualified in all three worlds; the other pairs fell under
   the Orchards bar and (1.0, 0.0005) went extinct in Canyon at tick 658.

   | Stage | Arms | Fixed | Readings per world |
   | --- | --- | --- | --- |
   | Arm mechanics (both stages) | the three goal worlds at 2,000 ticks, one seed each, as scratch copies of the goal recipes carrying the arm's overrides through `PROFILE=sweep --config` (a scratch build through `PROFILE=goal` only where the sweep profile lacks a reading) | — | the readings file records each arm's overrides, command and local summary hash (`.bench-artifacts/t26-f02/sweep/`, never committed) |
   | Unchanged-brain comparison | `founder` 200; `founder` 50 and 800 each under `Reference` and `Own` (five arms) | `mutation.per_unit_rate` 0 and `mutation.capacity.chance` 0, so no brain, body or capacity mutation (each arm's summary must show zero applied mutation events); β = u = 0 | persistence, plateau and trough, births per attempt, attempts rejected on energy and on the litter (reproduction failures), and separately the storage benefit: mean energy and `maximum_energy_clamp_loss` per creature-tick against the founder-200 arm |
   | β/u sweep | β ∈ {0.5, 1.0} × u ∈ {0.0005, 0.002} (four arms) | the ruled `sensor_scale`, capacity evolvable at the defaults | persistence, plateau and trough, the capacity block, `capacity_upkeep` and move flows per creature-tick |
10. **Readings.** `PersistenceSample` gains `capacity:
    Option<CapacityDistribution { min, p10, median, p90, max, mean,
    at_min_fraction, at_max_fraction }>` over living creatures, `null` at
    extinction; the boundary fractions count capacities equal to `min` and
    `max_energy`.
11. **Lab, server, frontend.** `validate` requires
    `start_energy ≤ energy.capacity.founder`; the exploration probe's
    refund caps through `energy_cap`; the detail `max_energy` is the
    creature's capacity; the canvas energy bar divides by
    `energy.capacity.reference`; the config types and panel carry the new
    fields.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.energy_flow{petri.flow="capacity_upkeep"}` | counter (`f64` non-monotonic sum, signed) | the Phase 0 upkeep remainder of invariant 5 | `energy.capacity.*`, `energy.lifecycle.*` on `run.started` |
| `petri.run.energy_flow{petri.flow="offspring_capacity_overflow"}` | counter (`f64` sum) | the birth cap step of invariant 7 | `mutation.capacity.*`, `energy.lifecycle.default_offspring_energy` |
| `petri.run.deaths_by_cause{petri.cause="capacity_upkeep"}` | counter (`u64` sum) | the attribution of invariant 5 | as above |
| `petri.config.energy.capacity.reference`, `petri.config.energy.capacity.upkeep_per_unit` | span attributes on the `world_update` phase group beside the decay and carrying values | invariant 5 | — |
| `petri.config.energy.capacity.reference`, `petri.config.energy.capacity.sensor_scale` | span attributes on the `sensor_assembly` phase group; `sensor_scale` is the mechanism switch, recorded configuration in the `run.started` body | invariant 6 | — |
| `petri.config.energy.capacity.{reference,min,move_cost_slope}`, `petri.config.energy.lifecycle.max_energy`, `petri.config.mutation.capacity.{chance,log_step}` | span attributes on the `actions` phase group (moves and births happen there) | invariants 2, 4, 7 | — |
| creature detail `max_energy` | API field | the creature's `body.capacity` | — |
| `capacity` block per persistence sample | goal and goal-long summary | invariant 10 | per world and sampled tick |

## Verification Plan

**Question.** Capacity is a heritable, bounded trait with linear costs
neutral at 200, so the founder's economy is unchanged while capacity can
move under selection; a changed founder charge, a capacity outside the
bounds, or a goal-long distribution fixed at a bound with negligible cost
flows would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal + goal-long` | gate reads the added per-creature-tick arithmetic and re-pins its deterministic blocks; goal reads persistence and the 2,000-tick distribution in the three worlds and feeds the sweep; goal-long is the track's one carrier and criterion 3 names its capacity-distribution trend. |
| Mutation gate | `run` | new production logic in `v3-core` (body rules, Phase 0 partition, birth cap, sensor scale) and `v3-cli` (the distribution block) whose behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | `crates/v3-lab` changes (founder signature, `validate`, the probe's cap) put the tier's lab end-to-end runs in question; the recruitment-paths readings are a pins-unchanged check (invariant 8). |

## Implementation Tasks

- [x] Config: `EnergyCapacityConfig`, `CapacityMutationConfig`, `max_energy`
      800, normalization and its tests.
- [x] Body: `capacity`, `founder(.., capacity)`, `inherit` with the
      capacity draws, `move_factor` and `upkeep`, property tests for the
      neutral point, the bounds, positivity and the draw order.
- [x] Cap and move rules read the body (invariants 3–4); characterization
      tests at the reference; the lab oracles re-derived.
- [x] Phase 0 `Phase0Charge`, `capacity_upkeep`, `DeathCause::CapacityUpkeep`
      and the booking and attribution tests (invariant 5).
- [x] Birth cap step, `offspring_capacity_overflow`, the required fixture and
      the ledger identity test (invariant 7).
- [x] Sensor scale: the per-creature `energy_scale` under both
      `sensor_scale` values at the read sites and their tests (invariant 6).
- [x] Lab, server detail doc, frontend bar scale, config types and panel
      group (invariant 11).
- [x] Telemetry: the two flows, the cause, the three phase groups'
      attributes, the family and dashboards-check rows.
- [x] Bench: `CapacityDistribution` in `PopulationReadings` and
      `PersistenceSample`; the two flows in `EnergyFlowTracking`'s mapping;
      schema and tests.
- [x] Reference docs: runtime-config §4, sensor, tick-orchestration §3,
      reproduction steps 11–14, startup-seeding §5.1, server-protocol.
- [x] `cargo test -p v3-core --test viability` first; re-pin the trajectory
      and reproducibility pins as predeclared.
- [x] Comparison, the `sensor_scale` ruling recorded in invariant 6, then
      the sweep (invariant 9); the chosen scale, β and u set as defaults.
- [ ] gate, goal and goal-long on the final defaults; mutation gate; slow tier.

## Verification

- [x] `cargo test -p v3-core --test viability` -> 30 passed on macOS
      aarch64 at the sweep defaults (re-pinned). GNU/Linux x86_64 digests:
      an x86_64 test binary under emulated Ubuntu 24.04 (glibc 2.39), which
      reproduced the `08b1584c` pins, measured the new ones; CI's
      `ubuntu-24.04` confirmation is pending.
- [x] `cargo test --workspace --no-fail-fast` -> all pass but v3-cli
      `gate_profile_has_no_severe_regression_against_series_references`
      (awaits the gate epoch re-pin); `cargo fmt --all --check`,
      `make rust-clippy`, `make frontend-build` and `npx vitest run`
      (675 passed) clean.
- [ ] `make check` -> `<result>`; `make rust-test-slow` -> `<result>`.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, summary
      line, output path, every survivor resolved as killed, equivalent, or
      deferred.
- [x] Comparison and sweep tables and local summary hashes in the readings
      file.
- [ ] gate, goal and goal-long summaries at
      `docs/progress/features/t26-f02-evolvable-energy-reserves{,-goal,-goal-long}.json`;
      raw hash and byte count checked; no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: fat reserves
survive lean times but cost to carry; the trait reaches creatures through
the body record's cap, the move charge and the Phase 0 upkeep. Compute: a
multiply-add per creature-tick, a multiply per move, one or two draws per
birth and an O(population) pass per persistence sample; gate wall-clock is
expected within the ordinary thresholds against the T27.F01 epoch, and the
deterministic blocks change because every birth draws. The gate and goal
epochs are re-pinned in the closing commit. Expected directions are in the
table; no other indicator has a predeclared direction. The ruled scale, β
and u, set before the final runs: `Reference`, 1.0, 0.002 (the only pair
qualifying under invariant 9's rule; readings file, commit `81202517`).

| Reading | Expected | A different reading says |
| --- | --- | --- |
| 2,000-tick persistence, three worlds | matches the T27.F01 goal epoch within its ordinary variation (the founder's economy is identical at the reference) | a changed founder economy: a defect |
| Mean capacity and mass at 50 over the goal-long run, under `Reference` | mean falls, mass at 50 rises: the founder's absolute thresholds leave a store above 200 unused while the cost is real | rising mean capacity or mass at 800: the storage benefit is real |
| The same under `Own` | mean capacity may rise: a larger store is filled before the gate opens | — |
| Distribution still centered at 200 with negligible `capacity_upkeep` and move flows | not expected | the tradeoff is effectively free |

**Measured verdict.** `<one line per profile>`

- Summaries of the declared profiles:
  [gate](../../progress/features/t26-f02-evolvable-energy-reserves.json),
  [goal](../../progress/features/t26-f02-evolvable-energy-reserves-goal.json),
  [goal-long](../../progress/features/t26-f02-evolvable-energy-reserves-goal-long.json).
- Full readings: [`docs/progress/readings/t26-f02.md`](../../progress/readings/t26-f02.md).

## Success Criteria

- [ ] Capacity is heritable and bounded to [50, 800]; move effort and upkeep
      are linear in it and neutral at 200 (property test, byte-identical
      booking test).
- [ ] A newborn above its capacity starts at it, the excess booked to
      `offspring_capacity_overflow`; the ledger identity holds.
- [ ] Every energy read divides by the ruled `sensor_scale`; the server and
      inspector report the creature's own capacity.
- [ ] The comparison and sweep tables exist; the defaults are the sweep's pick
      under the predeclared rule.
- [ ] The goal and goal-long summaries report the capacity distribution,
      boundary mass and trend per world, and the readings file names which
      predeclared outcome each world shows.

## Notes for AI Agents

- Decision: the energy-read denominator (`sensor_scale`) is ruled by
  invariant 6's predeclared rule after the unchanged-brain comparison
  (design note §3 D); the other value stays as an attribution control.
- Decision: the capacity draws are on every birth, after the gated
  phenotype-then-kin-tag draws; a newborn's excess provisioning is booked as
  `offspring_capacity_overflow`, not refunded.
- Decision: in-feature refactors R1–R6, R8 and R9 of the preparatory audit
  in `docs/progress/readings/t26-f02.md` are this feature's scope.
- Deferred: the lab's scripted stepper (`v3-lab/src/eval.rs`) charges decay
  only by its header contract, so no capacity upkeep (zero at the founder
  capacity the lab seeds); a lab feature seeding other capacities adds it.
  The root `world-recipe-*.json` presets keep their explicit `max_energy`,
  which bounds their range by invariant 1; they are not goal recipes.

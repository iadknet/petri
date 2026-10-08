# T03.F12 — Genome-Length-Dependent Reproductive Time

**Status**: In Progress
**Last updated**: 2026-10-08
**Feature**: T03.F12
**Track**: [T03 — Functional Traits and Metabolism](../../roadmaps/t03-functional-traits-and-metabolism.md)

## Goal

Copying a longer genome takes longer. A `Reproduce` that passes every gate at
tick *T* conceives a litter instead of spawning a child: the parent pays the
charge and the litter's energy at *T*, commits nothing further from that
moment through tick *T*+h (its brain runs and is charged), and the child is
born at the parent's Phase 2 slot on tick *T*+h, where
h = ⌈`genome_size()` × rate / `FOUNDER_GENOME_SIZE_UNITS`⌉. At the default
rate 1.0 the 97-unit founder gestates one tick and a genome twice its size two.
A parent that dies before *T*+h leaves no child. Natural analog: genome size
sets cell-cycle time through replication (Gregory 2001) and Avida's gestation
time, where the offspring exists only when the serial copy completes and a
partial copy dies with its parent. It reaches creatures through the body only:
time not spent eating, moving or reproducing, and no sensor. Intended outcome:
a smaller mean genome size over a goal run than the T03.F11 surcharge alone
leaves, with the population kept.

## Non-Goals

- A reduced action rate during the span (option C of the research note), a
  rejected-`Reproduce` path, or a penalty decision for one.
- An additional energy drain during the span; the existing decay, carrying and
  cognition charges continue and are the only drain.
- A gestation sensor, input, or score; the brain's input set is unchanged.
- Concurrent gestations; a held parent commits nothing, so a second commit
  cannot happen.
- Changes to T03.F08, T03.F10 or T03.F11 rates, to the age gate, the litter
  sizing (T17.F01) or the goal recipes; every standard world carries the
  feature through the default.
- A same-binary control arm for the long Canyon run; the row asks for
  persistence past tick 36,652, read against E2 descriptively.

## Inputs and Invariants

Sources of truth: the T03.F12 row and the track's 2026-10-06 note; the
[reproductive time design research note](../../strategy/reproductive-time-research-2026-10-06.md)
(the N1 cell, Sections 2 to 5, the user's five answers); the T03.F11 spec
(founder anchor, Step 5 charge, pin procedure) and the T14.F12 spec (the
neighborhood read and its standing floors); the T24.F02 goal summary and raw
report (previous closure); `crates/v3-core/src/simulation/actions/reproduction.rs`
(`apply_reproduce`, Steps 1 to 14), `crates/v3-core/src/simulation/tick.rs`
(`execute_reproduce`, `run_phase_2`), `tick/helpers.rs` and
`simulation/simulation.rs` (`remove_creature`, the one removal path),
`creature/state.rs`, `simulation/energy_accounting.rs` (`EnergyFlows`),
`simulation/stats.rs`, `config/simulation.rs` (`EnergyLifecycleConfig`,
`normalize`); the N1 prototype on branch `worktree-evolvability-diagnosis-2`
(`held_until_tick`, `reproduce_hold_ticks`, commit `dcbee45e`).

What the code does today, verified 2026-10-08: `apply_reproduce` resolves and
validates the target (Steps 1, 2), checks the cap (3) and age (4), computes the
charge with the T03.F11 multiplier (5), rejects on `min_reproduce_energy` (6)
and a litter under `initial_energy` (7), deducts charge then transfer (8),
clones and mutates the genome with `reproduce_rng` (9 to 11), builds the
child's plasticity weights (12) and spawns it (13, 14), incrementing
`offspring_spawned_count`, `offspring_energy_credit` and
`reproduction_actions_spawned_total`. `run_phase_2` walks the tick's decisions
in order and runs each creature's whole action queue; `execute_reproduce`
records the outcome channels (`record_action_result`, `record_offspring`) and
the failed-action penalty. Every death, in Phase 0, Phase 2 or predation, goes
through `Simulation::remove_creature`. `CreatureState` is not serialized
anywhere; there is no checkpoint. `FOUNDER_GENOME_SIZE_UNITS` is 97, pinned
by `the_canonical_founder_genome_is_one_hundred_eleven_units` (the name
predates T19.F04). The goal profile runs 2,000 ticks on seeds 11 / 22 / 33;
the gate 75 ticks.

Research summary (the note, 2026-10-06, Avida and Tierra documentation,
Gregory 2001, Ginther et al. 2024 read for it): three forms were weighed. A,
the N1 post-birth hold, is validated (Canyon seeds 1022 and 2022, 20,000
ticks: size 4.57× and 9.14× the founder against 7.09× and 22.38×, population
0.55× and 2.19×). B, pre-birth gestation, is rate-equivalent to A in steady
state and adds what N1 did not test: the litter lost with a dying parent (the
Avida analog exactly), the litter lost to crowding at delivery, and the end
of the commit tick's queue at conception. C, a reduced action rate, is untested and
weaker by construction. B is adopted as the user's framing with A as the
predeclared retreat; the span's drain is the existing metabolism (decay plus
carry over the span is 2% of a Canyon birth at founder size, 70% at 22×,
smaller than the surcharge at every measured size above the founder, yet the hold bounded size
where the surcharge, 91% of a birth's energy at 22×, did not; that the channel
is time is the note's inference, not a separated measurement).

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Rate | `energy.lifecycle.gestation_ticks_per_founder_genome`, `f32`, default `1.0` through `#[serde(default = "default_gestation_ticks_per_founder_genome")]` (a bare `#[serde(default)]` would read a missing field as `0.0` and disable the span; a test deserializes a lifecycle without the field and asserts `1.0`); finite and non-negative, invalid values fall back to `1.0`; `0.0` disables the span. Enters `config_digest`. |
| Span | `gestation_ticks(size, rate) = ((f64::from(size) * f64::from(rate)) / f64::from(FOUNDER_GENOME_SIZE_UNITS)).ceil() as u64`, one helper beside `genome_replication_cost_multiplier`, read from `cached_genome_size`. At rate 1.0 it equals `size.div_ceil(97)` for every `u32` size (proptest): founder 1, 98 to 194 units 2. Non-decreasing in size at a fixed rate. `deliver_tick = T.saturating_add(h)` and delivery fires when `sim.tick >= deliver_tick`, so a rate large enough to saturate the cast holds the parent for the rest of its life instead of wrapping (a configuration pathology, documented in the config spec; the frontend row caps at 10). Not a config field for the anchor: the founder's size is a property of the founder (T03.F11). |
| Commit, tick *T* | Steps 1 to 8 of `apply_reproduce` run as today, so a rejected attempt and its stats, penalty and action-log entry are unchanged. When both gates pass the parent stores one pending birth on `CreatureState`: the chosen `Direction`, the litter energy (`transfer`, `f32`) and `deliver_tick = T + h`. The result variant `Spawned` is renamed `Conceived` (`as_key` `"Conceived"`); the action log records `Success`; `record_action_result(true)` is recorded at commit as the action's outcome for Phase 2.5, while `record_offspring` is recorded at delivery, since `OutcomeAccumulator.offspring_spawned` and the window seam's `offspring_spawned` count children that exist (the parent is alive and holds a decision slot on the delivery tick). `reproduction_actions_attempted_total`, `last_tick_reproduce` and the per-creature attempt counts are unchanged. `gestations_started_total += 1`. When h is at least 1 the conception ends the parent's action queue for tick *T*: the remaining queued actions are discarded with no penalty and no log entry, so a second `Reproduce` in the same queue cannot conceive concurrently (N1 let the rest of the queue run; its child already existed). When h is 0 the delivery step runs at once inside `apply_reproduce` and the queue continues, so rate `0.0` is the pre-feature engine. |
| Hold, ticks *T*+1 … *T*+h | At the top of the parent's Phase 2 slot, after `compute.record` and `record_cognition` and after the `contains_key` check, a creature with a pending birth whose energy is at or below zero is removed there through `remove_creature_if_dead` (its litter lost; today Phase 1 charges can exhaust a creature without removing it, and an acting creature is removed after its first action, which a held one never takes); otherwise it delivers the birth if `sim.tick >= deliver_tick` (below) and then skips its whole action queue (`continue`, the N1 prototype's site), `gestation_creature_ticks_total += 1`; so the parent commits nothing on *T*+1 through *T*+h inclusive and acts again from *T*+h+1, exactly N1's span (the founder: one tick). Phase 0 decay and carrying, Phase 1 cognition and its charges, and Phase 2.5 run as today; no action-log entry is written for a held tick. |
| Delivery, tick *T*+h | At the parent's Phase 2 slot, before the skip: if `creatures.len() >= max_creatures` the birth is lost (`gestations_lost_population_cap_total`); else the stored direction is resolved from the parent's position (unchanged since the commit, since a held parent cannot move; the cell's occupancy can have changed); if that cell is not a valid target, the first valid cell in `Direction::ALL` order from the parent's position is taken; none valid loses the birth (`gestations_lost_no_cell_total`). A delivered birth runs Steps 9 to 14 as today, every inherited field read from the parent at delivery (genome, shared memory, plasticity weights, phenotype, identity, `generation + 1`), mutations drawn from `reproduce_rng` at this point, the child's energy the stored litter energy; `offspring_spawned_count`, `offspring_energy_credit`, `reproduction_actions_spawned_total`, `birth_mutation_operators` and the mutation funnel keep their birth-time meaning; the delivered cell joins `successful_spawn_targets`. `gestations_delivered_total += 1`; the pending birth is cleared; the parent is never held past *T*+h by crowding or anything else. |
| Loss | `Simulation::remove_creature` on a creature with a pending birth adds the litter energy to `energy_flows.gestation_lost` and `gestations_lost_parent_death_total += 1`; the two delivery losses add to the same flow. A lost birth is not a death: no `DeathCause`, no mortality record, no child. |
| Energy identity | `parental_transfer_debit` equals `offspring_energy_credit + gestation_lost + Σ pending litter energy over living creatures` at every tick boundary up to the Step 8 rounding: the debit is the realized `f32` difference while the litter carries the nominal `transfer`, so each birth contributes at most one `f32` ulp of the parent's energy at Step 8. That energy is at most `max(E_max, C_max)`, where `E_max` is the largest creature energy observed at any tick boundary of the run and `C_max` the largest `max_energy` in force at any time during it (within a tick energy rises only through eat, steal and the kill bonus, which clamp to the cap then in force; a runtime patch that lowers the cap does not re-clamp living creatures, and a parent that gains energy, conceives and spends it before the boundary is covered by `C_max`). A fixture with births, deaths during gestation and crowded deliveries records `E_max` and `C_max`, asserts the identity within `gestations_started × max(E_max, C_max) × 2^-23`, repeats it in a variant that lowers `max_energy` through `sim.config` mid-run and asserts at the lowering tick that some creature holding a pending birth has energy above the new cap (a held parent only loses energy, so it conceived above the lowered cap before the lowering: the B6 sequence) and that some creature holds energy above the new cap, and asserts the count identities exactly. |
| Count identities | `reproduction_actions_attempted_total == reproduction_actions_rejected_total + gestations_started_total` replaces today's `attempted == spawned + rejected` (births are delayed and can be lost), and `gestations_started_total == gestations_delivered_total + the three lost counters + pending births`. `stats_accounting_invariant_holds_over_50_ticks` in `crates/v3-core/tests/viability.rs` and `v3-reproduction-spec.md` Section 7 state the new identities; at rate `0.0` started equals spawned, so the old identity is implied. |
| Natural-analog rule | No input, sensor or score reads the span; the parent feels it through energy and age only. The held parent's cognition outputs are discarded, so reward learning sees no action outcomes on held ticks (the prototype's behavior). |
| Determinism | Delivery runs at the parent's slot in the tick's decision order and draws from `reproduce_rng` there, so cross-process reproducibility is untouched and thread counts do not matter. Every birth moves by h ticks, so the gate and goal `deterministic` blocks differ from every prior report; the founder-neighborhood, drift-walk and `recruitment_paths` blocks run no reproduction action and are predeclared byte-identical to T24.F02's. The new field enters every `config_digest`: the goal recipe digests in `crates/v3-cli/tests/bench_artifacts.rs`, the `applied_trajectory` digests, the `baseline_worlds` short-run identity hash and the viability suite's `founder_only_trajectory_digest_is_pinned` are re-pinned once after two agreeing runs (the T03.F11 procedure), in the first implementer brief. At rate `0.0` each of these fixtures reproduces its pre-feature digest exactly; this is the equivalence evidence and a kept test. Existing reproduction fixtures that assert the immediate-spawn sequence (`tick/tests/support.rs` and `actions/mod.rs` `small_config`, the `reproduction.rs` newborn-credit test, `neutral_connection_tests/refinement.rs`) set the rate to `0.0` deliberately; the gestation tests and the viability suite run at the default. |
| Viability | `cargo test -p v3-core --test viability` runs first. The founder's first delivery lands one tick after its first commit, so the suite's 20-tick birth horizons become `BIRTH_HORIZON_TICKS = 21` (the founder's gestation) wherever a test counts births within the horizon (`population_survives_and_reproduces_20_ticks`, `high_coverage_economics_survive_and_reproduce_short_horizon`, the two `viability_still_passes_with_*_mutations` tests), and `founder_reproduces_when_energy_allows_and_target_is_open` checks the pending birth after tick 1 and the child after tick 2; no assertion is weakened, and the moves are recorded in the readings. |
| Telemetry and reports | Six `u64` counters on `SimStats` and one `f64` flow, exported per the wiring convention (Telemetry table); the rate reaches `run.started` with no edit and joins the `actions` phase group in `crates/v3-telemetry/src/trace.rs` beside `genome_replication_cost_per_unit`. `SimStats` is not serialized: the goal report projects counters and flows explicitly (`crates/v3-cli/src/bench/tracking.rs`, `WorldTracking`, `EnergyFlowTracking`), so the flow joins `EnergyFlowTracking` (the `energy_flows` case block is already kept whole in the summary) and the six counters plus the end-of-run `pending_births` and `pending_litter_energy` form a `gestation` block on `WorldTracking`, which `GoalCaseObservation` flattens so the JSON key is `gestation` on the case, added to `KEPT_CASE_BLOCKS` in `crates/v3-cli/src/bench/artifacts.rs` so the committed summary carries them. |
| Frontend | `LifecycleEnergyConfig` gains the field; `EnergyLifecycleSection.tsx` gains one row beside "Replication Cost / Unit" (label "Gestation Ticks / Founder Genome", min 0, max 10, step 0.1, default 1, tooltip naming the span); `test/fixtures.ts` and the `ControlBar.test.tsx` fixture carry it. |
| Reference docs | `v3-runtime-config-spec.md` Section 4 gains the row and a "Gestation" note (formula, anchor, commit/hold/delivery, losses), and its existing replication-surcharge text stops naming the founder as 111 units (97 since T19.F04; `FOUNDER_GENOME_SIZE_UNITS`), so the two anchors in one document agree; `v3-reproduction-spec.md` Section 6 gains the commit, hold and delivery steps, the `Conceived` variant and the loss rule. |
| Retreat (predeclared) | If the goal population floor below fails on any world under this form, the form becomes A, exactly N1: Steps 9 to 14 run at *T* as today, the rest of the commit tick's queue runs (a second spawning `Reproduce` in it resets the hold to its own deadline), the parent is held *T*+1 … *T*+h, no pending state, loss counters or loss flow exist, and the count identity is today's. The pins, fixtures, telemetry rows, reference docs and both profiles are refreshed for the form in force and this spec is edited to record it. The retreat needs no new decision; a failure under A is a blocker for the user. |

Invariants: energy is conserved and behavior-backed (the identity above);
the genome is immutable after birth, so `cached_genome_size` is current and no
path walks a genome; a creature holds at most one pending birth; a held parent
commits no action; the span is a function of size and rate only, never of
crowding; rate `0.0` is byte-identical to the pre-feature engine.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.gestations_started` | counter | `SimStats.gestations_started_total`, one per commit | `energy.lifecycle.gestation_ticks_per_founder_genome` |
| `petri.run.gestations_delivered` | counter | `SimStats.gestations_delivered_total`, one per birth at *T*+h | same |
| `petri.run.gestations_lost_parent_death` | counter | `SimStats.gestations_lost_parent_death_total`, from `remove_creature` | same |
| `petri.run.gestations_lost_no_cell` | counter | `SimStats.gestations_lost_no_cell_total`, delivery with no valid cell | same |
| `petri.run.gestations_lost_population_cap` | counter | `SimStats.gestations_lost_population_cap_total`, delivery at the cap | same, `population.max_creatures` |
| `petri.run.gestation_creature_ticks` | counter | `SimStats.gestation_creature_ticks_total`, one per held Phase 2 slot | same |
| `petri.run.energy_flow{petri.flow="gestation_lost"}` | `f64` non-monotonic sum | `EnergyFlows.gestation_lost`, the litter energy of every lost birth | same |
| `energy.lifecycle.gestation_ticks_per_founder_genome` | recorded configuration | `run.started` body (no edit); `petri.config.energy.lifecycle.gestation_ticks_per_founder_genome` on the `actions` phase span | — |

Each of the six counters is a family in `crates/v3-telemetry/src/metrics.rs`,
a row in the expected families of `metrics/tests.rs`, and a name in
`scripts/telemetry-dashboards-check` `always_names`; the flow joins the
existing `petri.run.energy_flow` family's point list and its test row. The
implicit drain is an estimate, not a stored flow: `gestation_creature_ticks` ×
(`energy_decay_per_tick` + `genome_carry_cost_per_unit` × the terminal
population mean genome size), read beside `lifecycle_decay` and
`genome_carrying`; it ignores the correlation between size and held exposure
and the charges on parents removed in Phase 0 before a held slot.

## Implementation Tasks

- [x] Config field, `normalize`, `gestation_ticks` helper and its tests
      (unit, proptest equality to `div_ceil(97)` at rate 1.0, monotone,
      rate `0.0` gives 0); the `Conceived` rename; recipe-digest,
      `applied_trajectory` and `baseline_worlds` pins moved after two agreeing
      runs.
- [x] Pending birth on `CreatureState`; commit, hold, delivery and loss in
      `reproduction.rs`, `tick.rs` and `simulation.rs`; the six counters, the
      flow, the `gestation` report block and the viability count identity;
      TDD fixtures for the hold, the delivery (moved parent,
      fallback cell, no cell, cap), the loss on death, the identity, and
      rate `0.0` equivalence.
- [x] Telemetry wiring (families, test rows, dashboards-check names, the
      `actions` phase attribute).
- [x] Frontend row and fixtures; `v3-runtime-config-spec.md` and
      `v3-reproduction-spec.md`.
- [ ] Verification runs below: viability first, `make check`, gate and goal
      profiles, the long Canyon run, the mutation gate.

## Verification

- [x] `cargo test -p v3-core --test viability` (first) -> result in
      [`docs/progress/readings/t03-f12.md`](../../progress/readings/t03-f12.md).
- [x] Focused tests: `cargo test -p v3-core gestation` and the fixtures named
      above -> names and results in the readings file.
- [x] `cargo test -p v3-core --test applied_trajectory` twice (digests agree,
      re-pinned; the rate-`0.0` case reproduces the pre-feature digests),
      `--test baseline_worlds`, `cargo test -p v3-cli --test bench_artifacts`
      -> readings file.
- [ ] `make check` -> exit 0; `scripts/telemetry-dashboards-check` passes
      with the six new counter names; `scripts/telemetry-overhead` on the existing
      stack under the 1.10 ceiling -> readings file.
- [ ] Long Canyon run (predeclaration below): raw `canyon.ndjson` and
      `canyon-config.json` under the main checkout's ignored
      `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/`;
      committed under `docs/progress/sweeps/t03-f12/` only `run.sh`,
      `timing.txt` and `canyon-samples.md` (population, `mean_genome_size`,
      `mean_generation`, `mean_mesh_nodes` at every 2,500th tick and at the
      first sample past 36,652, with the raw file's SHA-256 and byte count).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The
      full survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t03-f12-genome-length-dependent-reproductive-time.json`
      and `-goal.json`, local raw hash/byte count and verification time
      checked, series entries point to the summaries, no new full report
      staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: replication time
proportional to genome length (Avida's gestation time; the nucleotypic effect
on cell-cycle time), reaching creatures through the body as ticks on which a
parent commits nothing and takes in nothing, and through the loss of a litter
whose parent dies first. Expected compute cost: one `u64` comparison per
creature per Phase 2 slot and one small struct per commit; wall time per
creature-tick flat; no compute severe and no epoch re-pin budgeted for either
profile. Gate reference: the T24.F02 gate, the gate epoch. Goal references: the
T24.F01 goal (the pinned goal-worlds epoch, for the comparator) and the T24.F02
goal (the previous closure, for the row's readings). The work counters are
ecology counters under the usual thresholds (flag +10%, severe +50%):
`actions_applied` per creature-tick is expected down by about the held share
(births were 1.7% of creature-ticks at T24.F02; at h of 1 to 2 the held share
is 2% to 4%), `births` per creature-tick not reliably lower (N1), and
`plasticity_updates` has read severe from clade composition before; a severe
is read against the per-world evidence and, if it is composition, reported
for the user's decision, not absorbed and not attributed to compute. Every
indicator not in the table is reported with no direction.

| Reading | Reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Goal persistence | T24.F02 final 2,831 / 9,165 / 4,062; plateau 6,214.7 / 6,760.2 / 5,798.3; minimum 56 / 838 / 257 | **Floor:** no world extinct, and final and plateau each not under half of T24.F02's on every world. A world under the floor triggers the predeclared retreat to form A and a goal re-run; A under the floor is a blocker. Minimum reported (Orchards' 56 is the sterility-sensitive trough). |
| Goal `mean_genome_size`, `mean_mesh_nodes` at the 20 persistence checkpoints (every 100 ticks) | T24.F02 at tick 2,000: 127.06 / 151.54 / 136.00 units (1.310× / 1.562× / 1.402× the founder); 4.06 / 5.70 / 3.82 nodes | The row's intended outcome: a smaller multiple of the founder than T24.F02's at tick 2,000 on each world, with the checkpoint trajectories tabulated in the readings. A world at or above T24.F02's tick-2,000 size is a contrary result: reported to the user under the blocker rule with its trajectory and its gestation counters, not absorbed and not rejected by the spec owner. |
| Units per generation `(size − 97) / mean_generation` (raw report, last persistence sample) | T24.F02 0.615 / 0.957 / 0.780 (generations 48.89 / 57.01 / 50.01) | Expected lower on each world; a higher value on any world is reported with the size row's contrary result. |
| Long Canyon run, goal seed 22 | E2 default (pre-T24 binary): extinct at tick 36,652; 16× the founder at 20,000 | `target/release/v3-cli run --ticks 40000 --sample-every 500 --seed 22 --config experiments/worlds/canyon-country.json --save-config <raw dir>/canyon-config.json > <raw dir>/canyon.ndjson`, release binary under `scripts/bench-wait`, alone, wall seconds to `timing.txt`. **Decisive check:** population above zero at every sample through 40,000; evidence is the committed `canyon-samples.md` with the raw file's SHA-256. `mean_genome_size` at 20,000 reported against E2's 16× descriptively (different substrate, no same-binary control). Budget 4 hours wall (N1's 20,000-tick Canyon arms ran about 1,400 s each); a run over budget is stopped and reported with its last sample, and the check is then unmet, not waived. |
| `neighborhood_read.changed_per_all_births` | Standing floors 0.146400 / 0.130000 / 0.143400 (T14.F12, strict not-below); T24.F02 read 0.142800 / 0.161800 / 0.156600 | Not below the standing floor on any world. T24.F02's Orchards reading already sat below its floor with no recorded disposition; that pre-existing breach is reported to the user with this closure. An Orchards reading below the floor here is a breach of a standing indicator and goes to the user under the blocker rule; the spec owner neither waives nor rebases it. |
| `neighborhood_read.mean_executed_nodes`, `dead_per_all_births` (raw report) | T24.F02 2.76 / 3.72 / 2.68; 0.0048 / 0.0092 / 0.0038 | Reported; a world below 2.5 executed nodes whose size is also below T24.F02's is investigated as core shrinkage. T14.F12's funnel read descriptively, as the row says. |
| Gestation counters per world | none | `gestations_started` above zero on every world (the applied-behavior evidence the span is on); delivered, the three lost counters and their shares of started, `gestation_creature_ticks / creature_ticks` (expected 2% to 5%), and the derived implicit drain reported. |
| `action_charges.reproduce`, `lifecycle_decay`, `genome_carrying` | T24.F02 59,965 / 57,430 / 70,984; 8,195,361 / 6,435,707 / 8,972,820; 174,756 / 144,410 / 195,049 | Reported, no direction (the per-attempt charge is unchanged). |
| Gate (75 ticks, founders) | T24.F02 gate | Founders' births shift one tick; counters within flag thresholds; `deterministic` differs; no severe budgeted. |
| Founder neighborhood, drift walk, `recruitment_paths` blocks | T24.F02 goal | Byte-identical, field for field. |
| Observation budgets | Workflow caps (`measurement_evidence.wall_caps`) | Founder neighborhood below 10 s; summed evolved below 180 s; neighborhood read below 10 s summed; drift walk below 30 s; `recruitment_paths` below 120 s per report (T24.F02 measured 240.2 s against it with no recorded disposition, T24.F01 13.0 s; the breach is reported to the user with this closure and this run's value is read against host load); goal run below 15 minutes. |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t03-f12-genome-length-dependent-reproductive-time.json),
  [goal](../../progress/features/t03-f12-genome-length-dependent-reproductive-time-goal.json).
- Full readings: [`docs/progress/readings/t03-f12.md`](../../progress/readings/t03-f12.md).

## Success Criteria

- [ ] A parent whose `Reproduce` passes the gates at *T* commits no action on
      *T*+1 … *T*+h while its cognition runs and is charged, and its child
      appears at *T*+h with the litter energy paid at *T*; a parent that dies
      first leaves no child and the litter energy is in `gestation_lost`.
- [ ] Rate `0.0` reproduces the pre-feature trajectory digests; the energy
      identity and the gestation count identity hold in the fixture.
- [ ] Viability passes first; gate and goal profiles run; the population floor
      holds on every world (or the retreat is recorded); the long Canyon run
      persists past tick 36,652; the tick-2,000 genome size is a smaller
      multiple of the founder than T24.F02's on every world, or the contrary
      world is recorded as a user decision.
- [ ] The six counters and the flow are exported, checked and carried in the
      goal summary, the rate is recorded configuration, the frontend row and
      the two reference docs exist.

## Notes for AI Agents

- Decision: (user, 2026-10-06, via the track note and the research note) the
  form is a pre-birth gestation with the N1 post-birth hold as the predeclared
  retreat; cognition is charged during the span; the span is anchored on
  `FOUNDER_GENOME_SIZE_UNITS`; there is no additional energy drain; there are
  no concurrent gestations.
- Exception: (user, 2026-10-08) the spec is committed with challenge finding B6
  upheld after round 3; the fix adopted is Codex's own round-3 proposal, not
  confirmed by a further round (record in `docs/progress/readings/t03-f12.md`).

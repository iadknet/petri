# T03.F12 — Genome-Length-Dependent Reproductive Time

**Status**: In Progress
**Last updated**: 2026-10-08
**Feature**: T03.F12
**Track**: [T03 — Functional Traits and Metabolism](../../roadmaps/t03-functional-traits-and-metabolism.md)

## Goal

Copying a longer genome takes longer. A `Reproduce` that spawns a child at
tick *T* holds the parent afterwards: on ticks *T*+1 … *T*+h its brain runs
and is charged but it commits no action, where
h = ⌈`genome_size()` × rate / `FOUNDER_GENOME_SIZE_UNITS`⌉. At the default
rate 1.0 the 97-unit founder is held one tick and a genome twice its size two.
This is the N1 post-birth hold the follow-up diagnosis run validated, the
form the spec's predeclared retreat named; the pre-birth gestation form was
built and measured first and failed the goal population floor on Canyon
(Non-Goals). Natural analog: genome size sets cell-cycle time through
replication (Gregory 2001) and Avida's gestation time, where a longer serial
copy occupies the organism longer. It reaches creatures through the body only:
time not spent eating, moving or reproducing, and no sensor. Intended outcome:
a smaller mean genome size over a goal run than the T03.F11 surcharge alone
leaves, with the population kept.

## Non-Goals

- A reduced action rate during the span (option C of the research note), a
  rejected-`Reproduce` path, or a penalty decision for one.
- An additional energy drain during the span; the existing decay, carrying and
  cognition charges continue and are the only drain.
- A hold sensor, input, or score; the brain's input set is unchanged.
- The pre-birth gestation form (conception at *T*, delivery at *T*+h, litters
  lost on death, no cell or cap): built and measured at `f380899e`, retired
  by the predeclared retreat when Canyon's final population read 0.441× the
  previous closure's; not kept behind a config switch. Its readings stay in
  the readings file as the form-B record.
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
weaker by construction. B was built first as the user's framing with A as the
predeclared retreat; its goal run (`f380899e`, 2026-10-08) read Canyon's
final population at 4,042 against T24.F02's 9,165 (0.441×, under the half
floor; plateau 0.862×), so the retreat fired and A is the form in force. The
span's drain is the existing metabolism (decay plus
carry over the span is 2% of a Canyon birth at founder size, 70% at 22×,
smaller than the surcharge at every measured size above the founder, yet the hold bounded size
where the surcharge, 91% of a birth's energy at 22×, did not; that the channel
is time is the note's inference, not a separated measurement).

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Rate | `energy.lifecycle.reproductive_hold_ticks_per_founder_genome`, `f32`, default `1.0` through `#[serde(default = "default_reproductive_hold_ticks_per_founder_genome")]` (a bare `#[serde(default)]` would read a missing field as `0.0` and disable the span; a test deserializes a lifecycle without the field and asserts `1.0`); finite and non-negative, invalid values fall back to `1.0`; `0.0` disables the span. Enters `config_digest`. Renamed from the form-B `gestation_ticks_per_founder_genome` because nothing gestates: the child exists at *T*. |
| Span | `reproductive_hold_ticks(size, rate) = ((f64::from(size) * f64::from(rate)) / f64::from(FOUNDER_GENOME_SIZE_UNITS)).ceil() as u64`, one helper beside `genome_replication_cost_multiplier`, read from `cached_genome_size`. At rate 1.0 it equals `size.div_ceil(97)` for every `u32` size (proptest): founder 1, 98 to 194 units 2. Non-decreasing in size at a fixed rate. `held_until_tick = (T + 1).saturating_add(h)`, so a rate large enough to saturate the cast holds the parent for the rest of its life instead of wrapping (a configuration pathology, documented in the config spec; the frontend row caps at 10). Not a config field for the anchor: the founder's size is a property of the founder (T03.F11). |
| Birth, tick *T* | `apply_reproduce` runs Steps 1 to 14 exactly as before this feature: the child is spawned at *T*, `Spawned` keeps its name, `record_action_result` and `record_offspring` are recorded at *T*, and every reproduction statistic, flow and rejection path is untouched. After a `Spawned` result with h at least 1, `execute_reproduce` sets `held_until_tick = (T + 1).saturating_add(h)` on the parent (a `u64` on `CreatureState`, 0 at birth) and `reproductive_holds_started_total += 1`. The rest of the tick's queue runs as today (N1): a later spawning `Reproduce` in the same queue overwrites `held_until_tick` with its own deadline. With h equal to 0 nothing is set, so rate `0.0` is the pre-feature engine. |
| Hold, ticks *T*+1 … *T*+h | At the top of the parent's Phase 2 slot, after `compute.record` and `record_cognition` and after the `contains_key` check, a creature with `held_until_tick > sim.tick` skips its whole action queue (`continue`, the N1 prototype's site) and `reproductive_hold_creature_ticks_total += 1`; so the parent commits nothing on *T*+1 through *T*+h inclusive and acts again from *T*+h+1, exactly N1's span (the founder: one tick). Before the skip, a held creature whose energy is at or below zero is removed through `remove_creature_if_dead`, the one deviation from N1: Phase 1 charges can exhaust a creature without removing it, and an acting creature is removed after its first action, which a held one never takes. Phase 0 decay and carrying, Phase 1 cognition and its charges, and Phase 2.5 run as today; no action-log entry is written for a held tick. |
| Count identities | `attempted == spawned + rejected` holds as today (`stats_accounting_invariant_holds_over_50_ticks`, `v3-reproduction-spec.md` Section 7, both restored to the pre-feature statement). `reproductive_holds_started_total == reproduction_actions_spawned_total` whenever the rate is above zero (h is at least 1 for every genome of at least one unit) and is 0 at rate `0.0`; a fixture asserts both. No in-flight energy exists, so no energy identity is added; the form-B identity fixture and its cap-lowering variant are removed with the form. |
| Natural-analog rule | No input, sensor or score reads the span; the parent feels it through energy and age only. The held parent's cognition outputs are discarded, so reward learning sees no action outcomes on held ticks (the prototype's behavior). |
| Determinism | The hold is a per-creature `u64` compared with `sim.tick` at the creature's Phase 2 slot, so cross-process reproducibility is untouched and thread counts do not matter. Held parents commit nothing, so the gate and goal `deterministic` blocks differ from every prior report and from the form-B run; the founder-neighborhood, drift-walk and `recruitment_paths` blocks run no reproduction action and are predeclared byte-identical to T24.F02's. The new field enters every `config_digest`: the goal recipe digests in `crates/v3-cli/tests/bench_artifacts.rs`, the `applied_trajectory` digests, the `baseline_worlds` short-run identity hash and the viability suite's `founder_only_trajectory_digest_is_pinned` are re-pinned once after two agreeing runs (the T03.F11 procedure) in the retreat brief, since they moved for form B and move again for A. At rate `0.0` each of these fixtures reproduces its pre-feature digest exactly; this is the equivalence evidence and a kept test. Existing reproduction fixtures that run several ticks and assert on later actions may set the rate to `0.0` deliberately where the hold would change what they assert (`tick/tests/support.rs` and `actions/mod.rs` `small_config`, `neutral_connection_tests/refinement.rs`); the hold tests and the viability suite run at the default. |
| Viability | `cargo test -p v3-core --test viability` runs first. Births happen at *T* as before this feature, so the suite's 20-tick birth horizons and the founder's one-tick reproduction check are restored to their pre-feature values (the form-B `BIRTH_HORIZON_TICKS = 21` is removed); no assertion is weakened. |
| Telemetry and reports | Two `u64` counters on `SimStats`, exported per the wiring convention (Telemetry table); the rate reaches `run.started` with no edit and joins the `actions` phase group in `crates/v3-telemetry/src/trace.rs` beside `genome_replication_cost_per_unit`. `SimStats` is not serialized: the goal report projects counters explicitly (`crates/v3-cli/src/bench/tracking.rs`, `WorldTracking`), so the two counters form a `reproductive_hold` block on `WorldTracking`, which `GoalCaseObservation` flattens so the JSON key is `reproductive_hold` on the case, kept in `KEPT_CASE_BLOCKS` in `crates/v3-cli/src/bench/artifacts.rs`. The form-B `gestation` block, its six counters and the `gestation_lost` flow are removed from `SimStats`, `EnergyFlows`, `EnergyFlowTracking`, the metrics families, their test rows and `always_names`. |
| Frontend | `LifecycleEnergyConfig` carries the renamed field; `EnergyLifecycleSection.tsx` has one row beside "Replication Cost / Unit" (label "Reproductive Hold Ticks / Founder Genome", min 0, max 10, step 0.1, default 1, tooltip naming the post-birth hold); `test/fixtures.ts` and the `ControlBar.test.tsx` fixture carry it. |
| Reference docs | `v3-runtime-config-spec.md` Section 4 carries the renamed row and a "Reproductive hold" note (formula, anchor, hold semantics, the dead-held-creature removal), and its replication-surcharge text names the founder as 97 units (`FOUNDER_GENOME_SIZE_UNITS`, since T19.F04); `v3-reproduction-spec.md` Section 6 adds the post-spawn hold step and Section 7 keeps `attempted = spawned + rejected`; the form-B commit, delivery, `Conceived` and loss text is removed from both. |
| Retreat (executed) | The predeclared retreat fired on the form-B goal run (`f380899e`: Canyon final 4,042 against T24.F02's 9,165, 0.441×). The form above is A, exactly N1 except the dead-held-creature removal. The pins, fixtures, telemetry rows, reference docs, both profiles, the long Canyon run, the overhead check and the mutation gate are refreshed for it. A floor failure under A is a blocker for the user, not a further retreat. |

Invariants: energy is conserved and behavior-backed (no new flow; the hold
moves no energy); the genome is immutable after birth, so
`cached_genome_size` is current and no path walks a genome; a held parent
commits no action; the span is a function of size and rate only, never of
crowding; rate `0.0` is byte-identical to the pre-feature engine.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.reproductive_holds_started` | counter | `SimStats.reproductive_holds_started_total`, one per spawning birth with h at least 1 | `energy.lifecycle.reproductive_hold_ticks_per_founder_genome` |
| `petri.run.reproductive_hold_creature_ticks` | counter | `SimStats.reproductive_hold_creature_ticks_total`, one per held Phase 2 slot | same |
| `energy.lifecycle.reproductive_hold_ticks_per_founder_genome` | recorded configuration | `run.started` body (no edit); `petri.config.energy.lifecycle.reproductive_hold_ticks_per_founder_genome` on the `actions` phase span | — |

Each counter is a family in `crates/v3-telemetry/src/metrics.rs`, a row in
the expected families of `metrics/tests.rs`, and a name in
`scripts/telemetry-dashboards-check` `always_names`. The
implicit drain is an estimate, not a stored flow: `reproductive_hold_creature_ticks` ×
(`energy_decay_per_tick` + `genome_carry_cost_per_unit` × the terminal
population mean genome size), read beside `lifecycle_decay` and
`genome_carrying`; it ignores the correlation between size and held exposure
and the charges on parents removed in Phase 0 before a held slot.

## Implementation Tasks

- [x] Config field renamed to `reproductive_hold_ticks_per_founder_genome`
      (default function, `normalize`, missing-field test); the span helper
      renamed with its tests (unit, proptest equality to `div_ceil(97)` at
      rate 1.0, monotone, rate `0.0` gives 0); `Spawned` restored; the four
      pin sets re-pinned after two agreeing runs, each reproducing its
      pre-feature digest at rate `0.0`.
- [x] Form A in `tick.rs` and `creature/state.rs`: `held_until_tick` set in
      `execute_reproduce` after a spawn, the skip at the top of the Phase 2
      slot with the dead-held-creature removal, the two counters; the pending
      birth, delivery, loss paths, six counters, `gestation_lost` flow and
      `gestation` block removed; TDD fixtures for the hold span (founder one
      tick, two-founder genome two), the same-queue reset, cognition charged
      while held, no action log while held, the dead-held removal, the count
      identity, and rate `0.0` equivalence.
- [x] Telemetry wiring for the two counters (families, test rows,
      dashboards-check names, the `actions` phase attribute renamed); the
      `reproductive_hold` report block in `tracking.rs` and
      `KEPT_CASE_BLOCKS`.
- [x] Frontend row and fixtures renamed; `v3-runtime-config-spec.md` and
      `v3-reproduction-spec.md` rewritten for the hold (97-unit founder text).
- [ ] Verification runs below on the form-A code: viability first,
      `make check`, gate and goal profiles, the long Canyon run, the overhead
      check, the mutation gate.

## Verification

Every item below is read on the form-A code; the form-B results
(`f380899e`) are the "Build verification (form B)" and "Benchmark readings
(form B)" sections of the readings file and verify nothing here. `make check`
(rerun after the self-review refactor) fails one test, the gate series check, against the form-B gate summary still
in the series (readings, "Build verification (form A)").

- [x] `cargo test -p v3-core --test viability` (first) -> result in
      [`docs/progress/readings/t03-f12.md`](../../progress/readings/t03-f12.md).
- [x] Focused tests: `cargo test -p v3-core --lib reproductive_hold` and the
      fixtures named above -> names and results in the readings file.
- [x] `cargo test -p v3-core --test applied_trajectory` twice (digests agree,
      re-pinned; the rate-`0.0` case reproduces the pre-feature digests),
      `--test baseline_worlds`, `--test viability` founder-only digest,
      `cargo test -p v3-cli --test bench_artifacts` -> readings file.
- [ ] `make check` -> exit 0; `scripts/telemetry-dashboards-check` passes
      with the two counter names and without the six removed ones;
      `scripts/telemetry-overhead` on the existing stack under the 1.10
      ceiling, run once more on a host whose load allows the pair count the
      spread demands (the form-B attempt was inconclusive at spread 0.1971:
      32 pairs at 5.4 s each exceed the 300 s cap); a second inconclusive
      verdict is recorded as such and reported to the user, never rounded to
      a pass -> readings file.
- [ ] Long Canyon run (predeclaration below) on the form-A code: raw
      `canyon.ndjson` and `canyon-config.json` under the main checkout's
      ignored `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/`
      (the form-B raw files move to a `form-b/` sibling); committed under
      `docs/progress/sweeps/t03-f12/` only `run.sh`, `timing.txt` and
      `canyon-samples.md` (population, `mean_genome_size`, `mean_generation`,
      `mean_mesh_nodes` at every 2,500th tick and at the first sample past
      36,652, with the raw file's SHA-256 and byte count), the form-B table
      kept in the same file under its own heading as a reference.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The
      full survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t03-f12-genome-length-dependent-reproductive-time.json`
      and `-goal.json` (overwriting the form-B summaries, whose readings stay
      in the readings file), local raw hash/byte count and verification time
      checked, series entries point to the summaries, no new full report
      staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: replication time
proportional to genome length (Avida's gestation time; the nucleotypic effect
on cell-cycle time), reaching creatures through the body as ticks on which a
parent commits nothing and takes in nothing (the form-B litter loss is gone
with the retreat; the thresholds, references and directions below stand
unchanged for the form-A re-run). Expected compute cost: one `u64` comparison
per creature per Phase 2 slot and one store per spawn; wall time per
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
| Hold counters per world | form B read held share 2.69% / 2.69% / 2.96% | `reproductive_holds_started` above zero and equal to `births` on every world (the applied-behavior evidence the span is on); `reproductive_hold_creature_ticks / creature_ticks` (expected 2% to 5%) and the derived implicit drain reported. |
| `action_charges.reproduce`, `lifecycle_decay`, `genome_carrying` | T24.F02 59,965 / 57,430 / 70,984; 8,195,361 / 6,435,707 / 8,972,820; 174,756 / 144,410 / 195,049 | Reported, no direction (the per-attempt charge is unchanged). |
| Gate (75 ticks, founders) | T24.F02 gate | Founders' births shift one tick; counters within flag thresholds; `deterministic` differs; no severe budgeted. |
| Founder neighborhood, drift walk, `recruitment_paths` blocks | T24.F02 goal | Byte-identical, field for field. |
| Observation budgets | Workflow caps (`measurement_evidence.wall_caps`) | Founder neighborhood below 10 s; summed evolved below 180 s; neighborhood read below 10 s summed; drift walk below 30 s; `recruitment_paths` below 120 s per report (T24.F02 measured 240.2 s against it with no recorded disposition, T24.F01 13.0 s; the breach is reported to the user with this closure and this run's value is read against host load); goal run below 15 minutes. |

**Measured verdict.** Form B (`f380899e`, 2026-10-08, run once): gate exit 0
not severe; goal CLI exit 3, severe against both references; the persistence
floor failed on Canyon (final 0.441×), so the predeclared retreat fired and
every other form-B reading (contrary Canyon size, Orchards and Confluence
neighborhood-read floors, Orchards executed nodes 2.38, `recruitment_paths`
230.8 s, overhead inconclusive, long Canyon run persisting at all 80 samples)
is the form-B record in the readings file, superseded by the form-A re-run and
not put to the user. Form A: pending.

- Summaries: [gate](../../progress/features/t03-f12-genome-length-dependent-reproductive-time.json),
  [goal](../../progress/features/t03-f12-genome-length-dependent-reproductive-time-goal.json).
- Full readings: [`docs/progress/readings/t03-f12.md`](../../progress/readings/t03-f12.md).

## Success Criteria

- [ ] A parent whose `Reproduce` spawns at *T* commits no action on
      *T*+1 … *T*+h while its cognition runs and is charged, and acts again
      from *T*+h+1; the founder is held one tick, a 194-unit genome two.
- [ ] Rate `0.0` reproduces the pre-feature trajectory digests; the count
      identities hold in the fixture.
- [ ] Viability passes first; gate and goal profiles run; the population floor
      holds on every world (or the retreat is recorded); the long Canyon run
      persists past tick 36,652; the tick-2,000 genome size is a smaller
      multiple of the founder than T24.F02's on every world, or the contrary
      world is recorded as a user decision.
- [ ] The two counters are exported, checked and carried in the goal
      summary, the rate is recorded configuration, the frontend row and the
      two reference docs describe the hold.

## Notes for AI Agents

- Decision: (user, 2026-10-06, via the track note and the research note) the
  form is a pre-birth gestation with the N1 post-birth hold as the predeclared
  retreat; cognition is charged during the span; the span is anchored on
  `FOUNDER_GENOME_SIZE_UNITS`; there is no additional energy drain; there are
  no concurrent spans.
- Decision: (predeclared retreat, fired 2026-10-08 on the form-B goal run at
  `f380899e`: Canyon final 4,042 = 0.441× T24.F02's 9,165) the form in force is
  the N1 post-birth hold; the gestation form is retired, not kept behind
  config, and its readings, the challenge finding B6 that concerned its energy
  identity, and the B6 exception the user accepted stay in
  `docs/progress/readings/t03-f12.md` as the form-B record.

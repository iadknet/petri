# T11.F01 — Mutational Neighborhood Indicator

**Status**: In Progress
**Last updated**: 2026-09-05
**Feature**: T11.F01
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Every benchmark report says how often one mutation leaves a brain acting like
its parent (silent), acting differently (changed), or not acting at all (dead),
per operator at one event and per birth through the production mutation engine,
on a fixed sensor battery of single-tick snapshots and short multi-tick
sequences. The founder half runs in the gate profile; the goal profile adds the
same reading on a predeclared sample of evolved genomes with structural
companions. The track floors are fixed here, before any repair lands.
Observation only: no production probability, operator, founder, or runtime
semantics change.

## Non-Goals

- No change to mutation operators, `MutationConfig` defaults, the founder, the
  VM, the graph backend, or tick mechanics. The repairs are T11.F02 onward.
- No automated no-regression comparison of neighborhood components between
  reports; each closing spec records the comparison in its Performance section
  (see Notes). No world-trajectory (survival or reproduction) battery.
- No new non-production operator (a single-field VM perturbation is T11.F02);
  the raw-versus-single-field contrast uses the existing operator pairs named
  below.
- No T11.F05 temporal motifs (they do not exist yet), no floor enforcement,
  and no pass/fail assertion on any founder or evolved fraction.
- No integration, remote mutation, worktree removal, or branch deletion by the
  implementer.

## Inputs and Invariants

- Sources: the T11 track's T11.F01 note and "Floors and the no-regression
  rule"; the [brain evolvability audit](../../strategy/brain-evolvability-audit-2026-09-04.md)
  (Appendix A is the draft of this indicator; Appendix B is not lifted);
  [T01.F12](t01-f12-goal-profile-basic-indicators-and-progress-table.md)
  for the goal profile, `Indicator<T>`, six-decimal formatting, and the
  final-state observation precedent; [T10.F10](t10-f10-deterministic-benchmark-harness.md)
  for the gate profile and the byte-identical `deterministic` block.
- Existing seams to reuse, not recreate: `v3_core::runtime::execute_creature_mesh_with_reserve`
  (the production cognition path), `MutationEngine::apply_mutations_with_food_type_count`,
  the four operator families' `apply` entry points and `ALL` constants
  (`VmOperator`, `GraphOperator`, `TopologyOperator`, `InputRefOperator`),
  `mesh_reachable_nodes` and `functional_complexity` in
  `creature/genome/analysis.rs`, the sensor snapshot types used by
  `observe_final_actions`, and `crates/v3-cli/src/bench.rs` for profile
  parameters, `GoalIndicators`, `Indicator<T>`, `six`, and `run_deterministic`.
  Food type count comes from the profile's `SimulationConfig`, not a literal.
- The per-tick shared-memory bookkeeping between sequence ticks (snapshot to
  `prev_shared_memory`, then decay by `shared_memory.decay_rate`) is the rule
  in `simulation/tick.rs` `run_phase_0`. Extract it into one core function the
  tick and the indicator both call, or call an equivalent existing seam; do
  not duplicate the rule.
- Determinism: every trial and birth draws from a `SmallRng` seeded by a fixed
  constant plus its index; tallies are integer counts summed in a fixed order,
  so results are independent of thread count. Parallel evaluation over trials
  with rayon is expected and must leave the `deterministic` block byte-identical
  across runs and thread counts (the existing bench test covers this).
- Report compatibility: new fields carry serde defaults (T01.F11 precedent) so
  every stored report still loads, and gate parameters and all prior
  deterministic fields are unchanged. Fractions are six-decimal strings.

### Battery (neighborhood-v1)

- Subject: a genome, evaluated from zeroed shared memory, zeroed previous
  memory, and fresh `GraphRuntimeState`, so the reading is a property of the
  genome alone. Sensor scenarios follow Appendix A's generator: local food of
  both types, the two neighbor food rings, occupancy, age, energy, and
  reproductive reserve, with zero barriers and zeroed perception.
- Single-tick snapshots: 48 scenarios from seed 7. Multi-tick sequences: 8
  sequences of 4 ticks from seed 8, each tick drawing a fresh scenario; shared
  memory and graph state persist across the ticks of a sequence under the
  production bookkeeping above, and energy and reserve are reset to the tick's
  scenario values. The signature is the ordered list of action queues
  (`Vec<WorldAction>`, complete, including direction and payload) over all 48 +
  32 executions.
- Classes, against the unmutated subject's signature: silent (identical on
  every execution), dead (differs somewhere and every action on every
  execution is `NoOp`), changed (otherwise). Companion counts per tally:
  `changed_only_in_sequences` (identical on all 48 snapshots, different in a
  sequence) and the mean fraction of executions that differ among changed and
  dead subjects.
- Per-operator treatment: for every operator in the four `ALL` lists, apply
  once to a fresh copy of the subject with the production reachability bias
  (the per-family `MutationConfig::reachable_bias` values, read from the
  production config rather than restated) and record applied, skipped, silent, changed, dead. The
  diagnostic contrasts are read from this table: raw redraw versus single-field
  step is `VmInstructionRawFieldMutation` versus `VmConstantMutation` and
  `GraphRawFieldMutation` versus `AlterGraphEdgeWeight` and
  `MutateGraphOperatorParam`.
- Per-birth treatment: run `MutationEngine::apply_mutations_with_food_type_count`
  with the production `MutationConfig` on fresh copies; bucket by
  `applied_events` (0 through the configured maximum, plus an "any events"
  total). Zero-event births are counted, not evaluated. Burst versus
  single-event supply is the bucket-1 row against the rest.
- Predeclared sizes and seeds. Founder half (gate and goal): 50 trials per
  operator (seed bases 1000, 2000, 3000, 4000 plus trial index by family) and
  500 births (seed base 9000 plus birth index). These are the once-halved
  values of the originally predeclared 100 and 1,000, applied under the
  compute-limit fallback below; the measured reason is recorded there. Evolved half (goal only):
  per seed, 12 genomes from the final living population, chosen as the
  creatures at ranks `floor(i * n / 12)` for `i` in `0..12` of the id-sorted
  population (all of them when `n < 12`, none when the seed went extinct);
  per genome, 20 trials per operator and 200 births, with seed bases offset by
  `100_000 * (genome_index + 1)` where `genome_index` runs over the sampled
  genomes of that seed in rank order. Record all constants in the report's
  battery block.
- Structural companions per sampled evolved genome: functional complexity,
  reachable node count, whether any reachable VM instruction or graph edge or
  sink reads shared memory, whether any writes it, whether any stateful compute
  node kind is present, and whether any plasticity is present. These attribute
  a zero reading to absent structure, inert structure, or a probe blind spot.
- Compute limits. Founder half: at most 10 seconds of release wall time per
  profile run on the recording host, and the two debug-build gate tests in
  `make check` may each grow by at most 10 seconds. Evolved half: at most 90
  seconds added to the goal run. If a limit is exceeded, halve the trial and
  birth counts once, record the measured reason here, and rerun; never choose
  sizes for better readings. Neighborhood wall time goes in the `environment`
  block, never in `deterministic`.
  Measured (orchestrator, 2026-09-05, debug build, 8 threads, after the first
  implementer had already halved to 50/500): the byte-identical gate test
  took 41 s and the series-regression gate test 15 s, against a pre-feature
  total of about 30 s for both, so the debug growth limit is exceeded even at
  the halved sizes and would be roughly doubled at 100/1,000. Release wall
  time for the founder half was 0.69 s (gate) and 0.83 s (goal), far inside
  its limit. Decision: production sizes stay at the once-halved 50/500 (a
  second halving is not permitted and would thin the per-birth buckets
  further), and the two debug-build gate tests run the same report path at
  the reduced fixture sizes (`NeighborhoodSizes::default()`), since in-process
  byte-identity and the work-counter regression check do not depend on trial
  count; `make bench` alone produces the production reading. The production
  founder half at 50/500 yields about 50 mutated births and about 5
  single-event births, which is coarse for floors (d) and (e); this is
  recorded as a deferred finding for T11.F04, which owns the supply change.

### Floors (final values)

Fixed from the audit's founder battery; none is set below the audit's value,
so no reason is required. They are met by the close of T11.F10, not by this
feature, and no floor is loosened after this spec closes.

- (a) Inserting one `Noop` with jump repair, or copying a block into an
  unreachable span, leaves the battery trace identical for any program whose
  jumps target in-range instructions and whose execution stays under the step
  cap (property tests, landing with T11.F02).
- (b) Memory-motif insertions are at least 80% silent on the founder.
- (c) Graph add-node, copy-node, and input-reference addition are at least
  95% silent on the founder.
- (d) Mutated births at production defaults are at most 5% dead.
- (e) Single-event births are at least 60% silent.

## Implementation Tasks

- [x] Add a core module (under `v3_core`, beside the genome analysis or
      mutation code, whichever it depends on least) with the battery, signature,
      classification, per-operator and per-birth tallies, and structural
      companions, using only the production seams above. Extract the shared
      memory bookkeeping rather than duplicating it.
- [x] Lift Appendix A into a maintained `v3-core` integration test that runs
      the founder half at reduced sizes and asserts determinism across two
      runs and the structural facts the audit records (for example
      `VmCopyConstantBlock` and `CopySubgraph` are fully silent on the founder);
      it asserts no floor.
- [x] Add `mutational_neighborhood` to `GoalIndicators` as
      `Indicator<MutationalNeighborhood>` with a serde default of `Undefined`:
      a battery block (version `neighborhood-v1`, seeds, sizes), the founder
      half (per-operator rows with family and operator name, per-birth buckets),
      and an evolved half that is `Undefined` outside the goal profile and
      otherwise carries per-seed sampled genomes (rank, creature id, per-birth
      tally, companions) plus per-operator and per-birth tallies pooled across
      the sample. Gate and goal run the founder half; sweep stays `Undefined`.
- [x] Record neighborhood wall time per profile run in the `environment`
      block, separate from simulation and final-state observation timings.
- [x] Update `docs/progress.md`: state that the mutational neighborhood
      indicator begins with T11.F01, and add the T11.F01 row with its gate and
      goal evidence and headline readings. Historical rows are unchanged.
- [x] Generate the gate and goal reports through `make bench` at the committed
      implementation, store them at
      `docs/progress/features/t11-f01-mutational-neighborhood-indicator.json`
      and `...-goal.json`, and append both to their series in
      `docs/progress/benchmark-series.json` as the T01.F12 closure did.

## Verification

- [x] TDD: classification (silent, changed, dead, `changed_only_in_sequences`)
      on constructed signatures; battery generation determinism; per-birth
      buckets sum to the mutated total; evolved-sample rank selection for
      `n < 12`, `n = 12`, `n > 12`, and `n = 0`; companions on constructed
      genomes with and without memory instructions, stateful nodes, and
      plasticity; report round-trip with the new field defaulted for a stored
      report lacking it. Coverage: `crates/v3-core/src/neighborhood/classify.rs`
      (`identical_signatures_classify_as_silent`,
      `a_snapshot_difference_with_a_non_noop_action_is_changed`,
      `every_execution_noop_and_differing_is_dead_even_with_an_empty_action_queue`,
      `changed_only_in_sequences_requires_identical_snapshots_and_a_differing_sequence`),
      `battery.rs` (`generate_is_deterministic_for_a_fixed_food_type_count`,
      `signature_is_a_pure_function_of_the_genome_and_battery`), `births.rs`
      (`zero_event_births_are_counted_but_not_folded_into_any_bucket` plus the
      `bucket_totals_equal_the_overall_any_events_tally` proptest), `sample.rs`
      (all four population-size cases), `companions.rs` (eight constructed-genome
      cases), and `crates/v3-cli/src/bench.rs`
      (`goal_indicators_defaults_mutational_neighborhood_to_undefined_when_the_field_is_absent`
      for the round-trip).
- [x] Property tests for the pure invariants: `classify.rs`
      (`a_signature_equal_to_its_base_is_always_silent`,
      `an_all_noop_differing_signature_is_always_dead`,
      `tally_merge_is_commutative_and_associative`) and `births.rs`
      (`bucket_totals_equal_the_overall_any_events_tally`, which also proves
      every birth is accounted for exactly once). No `proptest-regressions/`
      file was generated — every case proptest drew passed on the first run,
      so there is none to commit.
- [x] `cargo check --workspace --all-targets` passed after every Rust edit
      (enforced by the implementer-compile-check hook on every save). Focused:
      `cargo test -p v3-core` (1051 lib + 4 `mutational_neighborhood` + 25
      viability + others, all passed), `cargo test -p v3-cli` (23 lib + 9 bin +
      17 bench-integration + 7 config-integration, all passed).
      `make roadmap-check` passed after every document edit. The shared-memory
      bookkeeping extraction (`advance_shared_memory`) touches `run_phase_0`,
      so `cargo test -p v3-core --test viability` ran first: 25 passed, 0
      failed.
- [x] Debug-build timing of the two gate tests, three points: pre-feature
      (orchestrator, before T11.F01 existed) ~30s combined; at
      `NeighborhoodSizes::PRODUCTION` (orchestrator, 2026-09-05) 41s +
      15s ≈ 56s, exceeding the 10-second growth budget even at the once-halved
      50/500 sizes; at the fixed `NeighborhoodSizes::default()` used by these
      two tests from this closure on (this implementer, same host class)
      `gate_profile_deterministic_block_is_byte_identical_across_two_runs`
      14.57s and `gate_profile_has_no_severe_regression_against_series_references`
      7.27s, combined 21.84s — inside budget (any apparent decrease versus the
      ~30s pre-feature figure is host/session wall-clock noise, not a claim
      this feature made anything faster).
- [x] `simplify` pass on the diff (single implementer pass; see report to the
      orchestrator for the itemized changes), then `make rust-mutants`;
      summary line, output path, and full survivor list with resolutions
      recorded in the mutation record below.
- [ ] Benchmark reports stored as above; a second goal run to scratch output,
      sequential and without concurrent builds, with full `deterministic`
      equality, both elapsed times, and the neighborhood wall times. **Skipped
      by user decision (2026-09-05):** per-feature performance benchmarking is
      too expensive; a second `make bench PROFILE=goal` run was not performed.
      The stored gate and goal reports were produced once, at commit
      `bb557f1c`, release build, at the production 50/500 (founder) and 20/200
      (evolved) sizes; every commit in this feature after bb557f1c changes no
      production reading (tests, docs, and a doc-comment-plus-refactor
      simplify pass, covered by the report-shape and conversion tests above
      and by a temporary release-mode check (not committed, deleted after use)
      that called `evaluate_genome` at the production 50/500 sizes on the
      worktree's pre-simplify state and asserted the result matched the stored
      bb557f1c founder-half tallies exactly (spot-checked operator rows and
      birth buckets); see the implementer's report to the orchestrator for the
      comparison method).
- [ ] Reviewer findings recorded; `make check` exit 0 on the closing commit.
      Pending the orchestrator's independent review and `make check` run; this
      implementer ran the equivalent Rust-only checks above but does not run
      the frontend/policy/dependency portions of `make check` as part of
      implementation.

Mutation record. Output path for every run:
`/Users/istefanek/.local/share/petri-tools/mutants/t11-f01/mutants.out`
(the directory holds the latest run; the earlier runs' figures are recorded
here). First run (2026-09-05T18:59:35Z to 19:42:32Z, cargo-mutants 27.1.0):
`187 mutants tested: 43 missed, 105 caught, 0 timeout, 39 unviable`. Its 43
missed mutants, each resolved as **killed** by a test-only change unless
marked otherwise:

- `crates/v3-cli/src/bench.rs` `neighborhood_battery_execution_count` (return
  0, return 1, `+`→`-`, `+`→`*`, `*`→`+`, `*`→`/`; six mutants): killed by a
  bench unit test asserting the exact 48 + 8 × 4 = 80 count.
- `bench.rs` `to_neighborhood_operator_rows` → `vec![]`: killed by
  `to_neighborhood_operator_rows_preserves_every_row_in_order`.
- `bench.rs` `evolved_neighborhood_for_seed` seed arithmetic (`*`→`+`,
  `*`→`/`, `+`→`*`; three mutants): killed by
  `evolved_neighborhood_for_seed_offsets_each_sampled_genome_by_its_rank_order`.
- `crates/v3-core/src/neighborhood/battery.rs` `nonzero_or_zero` (return 1.0,
  0.0, -1.0; three mutants): killed by
  `nonzero_or_zero_is_always_zero_when_p_zero_is_one` and
  `nonzero_or_zero_is_in_range_when_p_zero_is_zero`.
- `battery.rs:66` `draw_scenario` `==`→`!=`: killed by
  `draw_scenario_uses_a_lower_zero_probability_for_the_first_food_type_than_others`.
- `battery.rs:174` `Battery::execute_single_tick` → `vec![]`: killed by
  `execute_single_tick_never_returns_an_empty_action_queue`.
- `births.rs:57` `per_birth_result` seed arithmetic (three mutants) and
  `births.rs:65` `==`→`!=`: killed by
  `per_birth_result_seeds_each_birth_by_offset_plus_base_plus_index` and the
  strengthened zero-event bucket test.
- `classify.rs` `Tally::record` (`+=`→`*=` five times, deleted `!`; six
  mutants) and `Tally::merge` (`+=`→`*=` six times): killed by
  `record_accumulates_every_field_by_addition_not_multiplication` and
  `merge_adds_every_field_rather_than_multiplying`.
- `classify.rs` `Tally::mean_fraction_differing` (return 0.0, `/`→`*`,
  `/`→`%`; three mutants): killed by
  `mean_fraction_differing_divides_the_recorded_totals`.
- `operators.rs:63` `OperatorKind::family` (`"xyzzy"`, `""`) and
  `operators.rs:81` `OperatorKind::seed_base` (0, 1): killed by
  `operator_catalog_family_labels_match_the_four_all_list_boundaries` and
  `per_operator_rows_seeds_trials_by_offset_plus_family_base_plus_trial`,
  which also kills the `operators.rs:168` seed arithmetic (three mutants).
- `sample.rs:17` `evolved_sample_ranks` `<`→`<=`: **equivalent**. At a
  population of exactly 12 the take-every-rank branch and the rank formula
  `i * 12 / 12` produce the same list, so the boundary change cannot alter
  any output.
- `simulation/tick.rs:124` `advance_shared_memory` `>`→`>=`: **equivalent**.
  A zero decay rate gives a factor of exactly 1.0, and multiplying an f32 by
  1.0 is the identity for every value, so the guard's boundary cannot change
  any slot.

A second run started by the implementer at 20:07:02Z was cut off when that
agent ended (92 of 187 evaluated, 0 missed, 57 caught, 35 unviable) and is
superseded. Final run, executed by the orchestrator uncontended
(2026-09-05T20:19:17Z to 20:50:43Z, 31 minutes): `187 mutants tested in 31m:
2 missed, 146 caught, 39 unviable`, 0 timeout. `missed.txt` holds exactly
the two equivalent mutants above; `timeout.txt` is empty. No production code
was edited to kill a mutant, and no `#[mutants::skip]` or `exclude_re` entry
was added. Deviation: the orchestrator ran the final target and wrote this
record because the implementer that started the rerun ended before it
finished and could not be resumed.

## Performance and Goal Impact

Predeclared compute cost: no simulation work-counter change; every nonzero
counter must read exactly 0.000000% against T01.F12 (previous closure) and
T10.F10 (gate epoch); plasticity remains both-zero. No gate epoch re-pin. The
gate wall clock gains the founder-half neighborhood time, reported separately
in the environment block and expected under 10 seconds release; the goal run
gains the founder and evolved halves, expected under 90 seconds and inside the
15-minute investigation threshold. This is not a mechanism feature; it has no
natural analog to name and reaches no creature.

Indicator wiring: `mutational_neighborhood` is defined for gate and goal from
this closure on. Its first readings are the T11 baseline that every later
closure compares against under the no-regression rule; they are not evidence
of improvement. Record here at closure: the founder per-operator table's
headline rows (memory motifs, graph add-node and copy-node, input-reference
add, raw-field operators), the per-birth buckets, the evolved-sample pooled
fractions with companions, the goal lineage and memory readings against the
T01.F12 baseline, and the second-run determinism check.

Measured on 2026-09-05 (orchestrator's timing run) and at implementation
completion, producer revision `bb557f1c` (release build, Apple M1 Pro, 8
logical threads); every commit in this feature after `bb557f1c` is tests,
docs, and a behavior-preserving refactor, so no production reading changes
(confirmed by the temporary release-mode parity check recorded in
Verification):

- Compute: every nonzero work counter read exactly 0.000000% against both
  gate epoch T10.F10 and the previous gate closure T01.F12, and exactly
  0.000000% (including plasticity, which is nonzero here for the first time
  since the goal profile runs a full population) against the goal series'
  only entry, T01.F12's goal report. Method: each stored report's own
  `comparison.references` block, computed by `compare_against` at generation
  time from the six `per_creature_tick` counters and the profile-matched
  reference reports; no separate comparison script was needed. Gate wall
  clock was -11.298164% vs T10.F10 and +5.274450% vs T01.F12 (both `ok`,
  wall-clock is a secondary signal). Full detail and the goal-side simulation
  trajectory/lineage/memory equality with T01.F12 are in the `docs/progress.md`
  T11.F01 row.
- Neighborhood wall time (environment block, outside `deterministic`): gate
  founder half 690.468 ms; goal founder half 827.116 ms, evolved half
  27,916.647 ms total across three seeds — both comfortably inside the
  10-second (gate, release) and 90-second (goal evolved-half) budgets.
- Founder per-operator headline rows (applied 50/50 for each; silent/changed/
  dead fractions): memory motifs `VmInsertReadStoreMotif` 0.12/0.88/0.00,
  `VmInsertReadBidMotif` 0.12/0.84/0.04, `VmInsertLoadCompareMotif`
  0.06/0.94/0.00 — the audit's 13-24% silent range for memory motifs is
  corroborated in character (well under floor (b)'s 80%, as expected before
  any repair). Graph `AddInternalGraphNode` 0.02/0.98/0.00, matching the
  audit's "1.0% silent" reading, and `CopyInternalNode` 0.58/0.42/0.00 — both
  far below floor (c)'s 95%, met by T11.F10, not here. Note: the roadmap's
  "graph add-node and copy-node" refers to these CGP-graph-backend operators,
  not the topology-family `AddNode`/`CopyNode` operators (both 1.00/0.00/0.00
  here), which are a different operator class over the mesh's node list, not
  the graph-node internals; citing the topology rows for floor (c) would
  misread it as nearly met. Input-reference `Add` 0.40/0.60/0.00. Raw-field
  contrasts: `VmInstructionRawFieldMutation` 0.02/0.98/0.00 vs the
  single-field `VmConstantMutation` 0.62/0.38/0.00; `GraphRawFieldMutation`
  0.10/0.90/0.00 vs the single-field `AlterGraphEdgeWeight` 0.74/0.26/0.00 and
  `MutateGraphOperatorParam` 0.74/0.26/0.00 — in both families the raw redraw
  is markedly less silent than the single-field step, as the audit predicted.
- Founder per-birth: 500 births, 456 zero-event (identical to the base by
  construction), 44 mutated (silent 0.068182, changed 0.704545, dead
  0.227273); bucket 1 (single-event, 4 births) is 0.5 silent / 0.5 changed /
  0.0 dead. Deferred finding (T11.F04, recorded in the Battery bullet and
  again here): 44 mutated and 4 single-event births at the production 50/500
  sizes is too coarse to size floors (d) and (e) precisely; T11.F04 owns the
  supply change that makes this reading tighter.
- Evolved-sample pooled per-birth fractions (12 genomes/seed sampled by rank,
  255 mutated births pooled per seed at 20/200 evolved sizes): seed 11
  (final population 5,291) silent 0.086275/changed 0.713725/dead 0.200000;
  seed 22 (10,997) silent 0.156863/changed 0.721569/dead 0.121569; seed 33
  (8,130) silent 0.086275/changed 0.713725/dead 0.200000 — the same order of
  magnitude as the founder's pooled reading, no evolved-population collapse
  or explosion in sensitivity.
- Structural companions: of the 36 sampled evolved genomes (12 per seed), a
  minority read shared memory (for example seed 11 ranks 0 and 2645) and
  three carry plasticity (seed 22 rank 916; seed 33 ranks 2032 and 4742),
  none has a stateful compute node. T01.F12's memory-sensitivity fraction is
  0.000000 for every seed despite this: the companions block attributes that
  zero to a probe blind spot or inert wiring rather than to memory structure
  being absent from the evolved population, which is exactly what this
  bullet in the Battery section was written to distinguish.
- Goal lineage and memory readings against the T01.F12 baseline: byte-for-byte
  identical (lineage count/entropy 11=142/2.780730, 22=144/2.947968,
  33=140/2.668014; memory sensitivity 0/0/0, fraction 0.000000 for every
  seed; persistence, births/100 ticks 7994.200000, and structure distribution
  1/95/97/124/798/116.771071 all equal), confirming the simulation trajectory
  is unchanged, as an observation-only feature requires.
- Second-run determinism: not run as a second `make bench` process (skipped
  by user decision, recorded in Verification); in-process determinism across
  thread counts is instead covered by the committed
  `founder_neighborhood_is_identical_across_thread_counts` integration test
  and by `gate_profile_deterministic_block_is_byte_identical_across_two_runs`.

## Success Criteria

- [ ] Gate and goal reports carry a byte-reproducible `mutational_neighborhood`
      reading; historical reports still load with it `Undefined`.
- [ ] The founder reading is consistent with the audit's causes (the audit's
      headline operators keep their character) and the floors above are fixed.
- [ ] The evolved-genome sample reports fractions with structural companions,
      so a zero memory reading is attributed rather than assumed.
- [ ] No production behavior changed: viability, reproducibility, and the
      compute counters are unchanged, and `make check` passes.

## Notes for AI Agents

- Planning readiness review (orchestrator, 2026-09-05): checked against the
  template, the T11.F01 track note, the floors bullet, T01.F12's goal-profile
  budget, and the gate tests' debug-build budget. One revision: the automated
  no-regression comparison was moved out of scope (each closing spec records
  it; T11.F02 is the first that needs it, and a `compare_against` extension is
  a bounded later addition if the manual record proves error-prone). Ready.
- Later features must not read the founder half from the goal report as the
  gate reading, and must not compare evolved-sample fractions across features
  whose goal populations differ without saying so.
- Implementation history: the first implementer assigned to this feature lost
  its context to a usage limit before completing it; the commit history and
  this spec's amendments are the only record of that session's decisions
  (notably the once-halved 50/500 founder sizes). A later implementer
  finished the feature by reading forward from the committed state
  (`bb557f1c`) and this spec rather than from any preserved implementer
  transcript.
- Deferred finding for T11.F04: the production founder half at 50/500 sizes
  yields only about 44 mutated births and about 4 single-event births per
  report run (measured; the spec's Battery bullet estimated "about 50" and
  "about 5" before this closure's numbers were in hand). This is too coarse
  to size floors (d) (at most 5% dead) and (e) (at least 60% silent)
  precisely — a handful of births flipping class changes the fraction by
  several points. T11.F04 owns the mutation-supply change and should either
  raise the per-birth sample size or accept the coarseness explicitly when it
  re-reads this indicator.

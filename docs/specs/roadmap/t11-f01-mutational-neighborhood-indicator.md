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
- Predeclared sizes and seeds. Founder half (gate and goal): 100 trials per
  operator (seed bases 1000, 2000, 3000, 4000 plus trial index by family) and
  1,000 births (seed base 9000 plus birth index). Evolved half (goal only):
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

- [ ] Add a core module (under `v3_core`, beside the genome analysis or
      mutation code, whichever it depends on least) with the battery, signature,
      classification, per-operator and per-birth tallies, and structural
      companions, using only the production seams above. Extract the shared
      memory bookkeeping rather than duplicating it.
- [ ] Lift Appendix A into a maintained `v3-core` integration test that runs
      the founder half at reduced sizes and asserts determinism across two
      runs and the structural facts the audit records (for example
      `VmCopyConstantBlock` and `CopySubgraph` are fully silent on the founder);
      it asserts no floor.
- [ ] Add `mutational_neighborhood` to `GoalIndicators` as
      `Indicator<MutationalNeighborhood>` with a serde default of `Undefined`:
      a battery block (version `neighborhood-v1`, seeds, sizes), the founder
      half (per-operator rows with family and operator name, per-birth buckets),
      and an evolved half that is `Undefined` outside the goal profile and
      otherwise carries per-seed sampled genomes (rank, creature id, per-birth
      tally, companions) plus per-operator and per-birth tallies pooled across
      the sample. Gate and goal run the founder half; sweep stays `Undefined`.
- [ ] Record neighborhood wall time per profile run in the `environment`
      block, separate from simulation and final-state observation timings.
- [ ] Update `docs/progress.md`: state that the mutational neighborhood
      indicator begins with T11.F01, and add the T11.F01 row with its gate and
      goal evidence and headline readings. Historical rows are unchanged.
- [ ] Generate the gate and goal reports through `make bench` at the committed
      implementation, store them at
      `docs/progress/features/t11-f01-mutational-neighborhood-indicator.json`
      and `...-goal.json`, and append both to their series in
      `docs/progress/benchmark-series.json` as the T01.F12 closure did.

## Verification

- [ ] TDD: classification (silent, changed, dead, `changed_only_in_sequences`)
      on constructed signatures; battery generation determinism; per-birth
      buckets sum to the mutated total; evolved-sample rank selection for
      `n < 12`, `n = 12`, `n > 12`, and `n = 0`; companions on constructed
      genomes with and without memory instructions, stateful nodes, and
      plasticity; report round-trip with the new field defaulted for a stored
      report lacking it.
- [ ] Property tests for the pure invariants: fractions sum to the applied
      count, a signature equal to its base is silent, an all-`NoOp` differing
      signature is dead, and bucket totals equal the overall tally, with
      assertions independent of the drawn cases. Commit any
      `proptest-regressions/` file.
- [ ] `cargo check --workspace --all-targets` after coherent Rust edits; the
      focused core and CLI tests; `make roadmap-check` after document edits.
      Viability-first is not applicable unless defaults, founders, or tick
      mechanics change, which this feature must not do; if the memory
      bookkeeping extraction touches `run_phase_0`, run
      `cargo test -p v3-core --test viability` first and record it.
- [ ] Debug-build timing of the two gate tests before and after, against the
      10-second growth limit.
- [ ] `simplify` pass on the diff, then `make rust-mutants`; record the
      summary line, output path, and full survivor list with each resolution.
- [ ] Benchmark reports stored as above; a second goal run to scratch output,
      sequential and without concurrent builds, with full `deterministic`
      equality, both elapsed times, and the neighborhood wall times.
- [ ] Reviewer findings recorded; `make check` exit 0 on the closing commit.

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

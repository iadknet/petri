# T11.F14 — Mesh Execution Observability

**Status**: Complete
**Last updated**: 2026-09-06
**Feature**: T11.F14
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Every gate closure reports what the founder's mesh actually executes, and every
goal closure reports the same measurements on the existing evolved-genome
sample with its actual generation depth. Distinguish structural reachability,
executed structure, input-dependent route choices, hop-cap termination, and
executed nodes whose bypass leaves the battery's action queues unchanged.

## Non-Goals

- Change no production execution, mutation, founder, energy, memory, learning,
  or reproduction behavior. Routing repairs and the single-visit rule belong
  to T11.F15; legal loops and their existing termination behavior remain.
- Do not change gate/goal parameters, neighborhood scenarios, trial counts,
  evolved sampling, mutation streams, or any existing indicator definition.
- No whole-population mesh battery census, mutation-only lineage walk,
  extended evolution run, UI, new dependency, or generic observer framework.
- These are bounded battery measurements, not proof of cognition, ecological
  neutrality, or useful redundancy. No new fitness signal or indicator floor.

## Inputs and Invariants

- Sources: the owning track's T11.F14 row and note;
  [T11.F01](t11-f01-mutational-neighborhood-indicator.md);
  [T01.F12](t01-f12-goal-profile-basic-indicators-and-progress-table.md);
  [mesh research note](../../strategy/mesh-evolvability-research-2026-09-06.md)
  Sections 3, 7 (C2 and C11), and 8. Dependencies remain in the track.
- Reuse `neighborhood::Battery`, `Signature`, `evolved_sample_ranks`,
  `structural_companions`, `mesh_reachable_nodes`, the production
  `runtime::mesh::execute_creature_mesh_impl` / `MeshExecutionMode` seam,
  production gated routing, and `crates/v3-cli/src/bench.rs` report assembly.
  Runtime must not depend on neighborhood or CLI types.
- Research, 2026-09-06: extending the existing compile-time mesh execution
  mode with compact hop observations is preferred to collecting full backend
  traces, which also allocate VM instruction and graph-pass detail the
  indicator does not consume. Both preserve the production executor; a
  separate simulator would duplicate its routing and termination rules.
  The existing neighborhood and report types supply the rest without a new
  package. [Serde's field-default documentation](https://serde.rs/attr-default.html)
  supports the required missing-field behavior. The abstract of
  [Dang, Kalkreuth, and Opris (2026)](https://arxiv.org/abs/2606.15923)
  distinguishes connected non-contributing material; this motivates a
  knockout measurement, not a claim that Petri satisfies that CGP proof.
- A normal observed execution must match unobserved production execution in
  actions, priority bid, energy, shared memory, graph runtime state, and work
  counters. Collection must not introduce hot-path allocations in the
  unobserved mode. No full backend traces are needed.

The new measurement is versioned `mesh-execution-v1`. It runs once on the
unmutated founder and once per existing sampled evolved genome, not on each
mutation trial. It reuses all 48 snapshots and 8 four-tick sequences of
`neighborhood-v1`, with the existing fresh-state and per-world-tick bookkeeping.
The report records the version, the 80 executions per genome, the 48 snapshot
route probes, and the knockout method below. Existing battery metadata and
seeds are unchanged.

- `total_node_count`: genome node count. `reachable_node_count`: existing
  structural reachability count. `executed_node_count`: number of distinct
  node IDs dispatched over the complete baseline battery, including repeated
  visits only once. Missing nodes are not dispatched; an attempted dispatch
  that exhausts energy is still a dispatch. These are mesh nodes, not VM
  instructions, graph-internal nodes, or `functional_complexity`.
- `hop_cap_hits`: number of baseline battery executions whose actual
  termination is `MaxHopsReached`, out of the reported 80. A terminal node
  reached on the last permitted hop is not a cap hit. Do not infer a hit
  from hop count alone or assume cap termination always emits `NoOp`.
- `route_varies_with_input`: compare only the 48 independently reset snapshot
  scenarios, so sequence history is not attributed to current input. For each
  node and scenario collect the set of selected target positions when runtime
  actually applies its routing decision after terminal/exhaustion checks.
  The flag is true when a node has different nonempty sets in two scenarios
  and their union contains at least two positions. Differences in whether a
  node was visited alone do not establish a conditional route; neither do
  repeated choices within one scenario. Terminal/exhausted nodes' diagnostic
  gate winners do not count. Target positions, not destination IDs, identify
  branches. This finite battery can miss conditional behavior.
- `knockout_count`: for each distinct baseline-executed node, separately
  bypass it in a fresh genome clone and rerun the complete battery from fresh
  state. Count the nodes whose complete `Signature` is identical to baseline.
  The bypass successor is the removed node's winner under the existing
  production resolver with zero runtime gate scores (highest bias, position
  ties). Redirect every incoming target and the entry ID, when applicable,
  to that successor, preserving incoming slot/bias/order and all other
  genome content; remove the node. No successor leaves incoming references
  dangling at its removed ID; a self-successor does the same. A missing
  successor remains missing. All use existing production soft termination.
  The successor receives the predecessor's unchanged upstream slots; the
  removed backend, its dynamic gates, effects, and cost do not execute.
  This is `static-successor-bypass-v1`, not a routing repair. It tests action
  queues including direction and payload, not energy/priority/state equality
  as independent outcomes. Indirect changes to those values that alter a
  later battery action still make the knockout contributing.
- Counts satisfy `0 <= knockout <= executed <= reachable <= total` for
  well-formed unique-ID genomes. Baseline signatures and all existing
  neighborhood tallies remain exactly unchanged. Ordered collections and
  integer aggregation preserve repeatability and thread-count independence.

Add a serde-defaulted `mesh_execution` reading to the founder half and beside
each sampled genome's structural companions. Missing historical readings are
`Undefined`, never a measured zero. Include actual `CreatureState::generation`
on sampled genomes (missing historical generation is absent/undefined).
Founder depth is explicitly zero. This metadata must not change the battery's
sensor values, including its existing zero generation input.

Each goal seed also records the generation distribution of **all final living
creatures**, alongside its existing final population count: median and maximum
as `u64`, with the upper middle observation for an even population, and
`Undefined` when extinct. This is not the distribution of the 12 sampled
genomes, ticks elapsed, or the maximum depth of an extinct ancestor. Old
reports default this field to `Undefined`. Gate has the founder reading;
goal has founder and evolved readings; sweep and other profiles acquire no
new measurement. All readings live in `deterministic`; observation timing
stays in the existing neighborhood/final-observation `environment` timings.

## Implementation Tasks

- [x] Add compact applied mesh observations through the existing executor
      seam and core tests proving observation preserves production behavior.
- [x] Extend the neighborhood battery with the defined mesh measurements and
      isolated structural knockouts, sharing scenario/sequence bookkeeping;
      add constructed fixtures and property tests before implementing results.
- [x] Wire founder, sampled-genome, sampled-generation, and whole-final-
      population depth readings into the existing bench report, with historical
      defaults and profile/serialization tests.
- [x] Store fresh gate and single goal reports at
      `docs/progress/features/t11-f14-mesh-execution-observability.json` and
      `...-goal.json`; append them to `docs/progress/benchmark-series.json`.
      Add the closure row and concise measurement definitions to
      `docs/progress.md`, including per-seed depth beside evolved mesh readings.
      Keep historical report content and epoch baselines unchanged.

## Verification

- [x] TDD fixtures cover static and sensor-conditional routes, multiple target
      positions sharing a destination, an unreachable node, a reachable losing
      branch, repeated visits, missing entry/target, an actual hop-cap loop,
      terminal completion at the cap, and unused terminal/exhaustion route
      scores. A node selecting the same multi-position set in every snapshot
      does not establish input variation.
- [x] Knockout fixtures cover a silent pass-through, a contributing action or
      upstream/memory producer, a contributing conditional router, entry
      bypass, bias/position winner selection, no/missing/self successor, and
      sequence-only effects. Assert source genome isolation and complete
      signature comparison; independently exercise VM and graph nodes.
- [x] Property tests in `v3-core` establish the pure count/subset bounds and
      source-preserving bypass behavior on generated well-formed genomes;
      assertions do not depend on which cases proptest drew. A production-
      versus-observed parity fixture compares complete applied outcomes and
      runtime state, including repeated graph visits and exhaustion.
- [x] Bench tests cover known odd/even and empty population depths, generations
      above `u32::MAX`, actual sample generation, whole-population rather than
      sample-only aggregation, defaulted historical fields, profile presence,
      and byte-identical deterministic output across runs/thread counts at
      existing reduced test trial sizes. Run focused core/CLI tests and
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [x] Run the explicit diff self-review for reuse, simplification, and
      efficiency, then fresh `MUTANTS_ITERATE=0 make rust-mutants`; record its
      summary, output path, full missed/timeout list and each resolution.
- [x] Run `make bench` for the gate and once for the goal, using the stored
      paths above; record compute comparisons, observation cost, full prior-
      field deterministic equality with T11.F07, and readings below.
- [x] A second goal run is not applicable: the 2026-09-05 user decision in
      `docs/workflow.md` uses the single closure reading and the reproducibility
      tests in `make check`. The gate's two-run check remains required.
- [x] `make roadmap-check` passes on document edits; final review findings and
      resolutions and the reviewed-feature `make check` result are recorded.
      Final closure-content verification and its tested commit are recorded
      by the orchestrator in the closure conversation before integration.

Verification evidence, 2026-09-06 (all commands from the feature worktree with
`~/.local/share/petri-tools/bin` and `~/.local/share/aquaproj-aqua/bin` on PATH):

- `cargo check --workspace --all-targets`: exit 0 after final Rust edits,
  `/tmp/t11-f14-check-complete.log`.
- `cargo test -p v3-core --lib neighborhood`: exit 0, 51 passed,
  `/tmp/t11-f14-test-neighborhood.log`.
- `cargo test -p v3-core --lib compact_observation`: exit 0, 2 passed,
  `/tmp/t11-f14-test-runtime.log`.
- `cargo test -p v3-cli --lib bench::tests`: exit 0, 34 passed,
  `/tmp/t11-f14-test-cli.log`; includes reduced gate/goal deterministic
  byte equality across thread counts. Production/default/founder/tick-loop
  mechanics did not change, so the viability-first rule is not applicable.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0,
  `/tmp/t11-f14-clippy.log`; `cargo fmt --all` and `git diff --check` passed.
- Final `MUTANTS_ITERATE=0 make rust-mutants`: exit 0,
  **49 mutants tested in 4m: 1 missed, 19 caught, 29 unviable**.
  `/tmp/t11-f14-mutants-final.log`; output
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f14/mutants.out`;
  adjacent `run-mode.txt` reads `fresh`. Full survivor list (no timeouts):
  `crates/v3-core/src/runtime/mesh.rs:265:40: replace || with && in
  <impl MeshExecutionMode for ObservedMeshExecution>::record_hop` —
  **equivalent**: `RECORDS_HOPS=false` already makes the executor pass `None`
  for terminal/exhausted routes; ordinary dispatches have both flags false
  and preserve the same position. No skips, exclusions, or production
  changes were introduced to kill mutants. The first fresh run had the same
  list; final evidence followed the test strengthening and repeated self-review.
- `make bench PROFILE=gate FEATURE=t11-f14-mesh-execution-observability
  OUT=docs/progress/features/t11-f14-mesh-execution-observability.json` and
  `make bench PROFILE=goal FEATURE=t11-f14-mesh-execution-observability
  OUT=docs/progress/features/t11-f14-mesh-execution-observability-goal.json`:
  both exit 0, serialized after mutations with no competing work. Logs:
  `/tmp/t11-f14-bench-gate.log`, `/tmp/t11-f14-bench-goal.log`. Exactly one
  goal run. Host-process inspection required authorized sandbox escalation;
  no guard was bypassed.
- Full recursive equality of every pre-existing gate and goal `deterministic`
  field against T11.F07 passed. Only added founder/sample `mesh_execution`
  and `generation`, and per-seed `generation_distribution`, were removed;
  the feature label is outside `deterministic`. Comparison script/log:
  `/tmp/t11-f14-compare.py`, `/tmp/t11-f14-comparison.log`. All old battery
  metadata, operator rows, birth buckets, structural companions, population
  samples, and other indicators are unchanged, not merely their headlines.
- `make roadmap-check`: exit 0 after document edits,
  `/tmp/t11-f14-roadmap-check.log`. Final review is complete and its sole P1
  is resolved by the explicit user decision. The orchestrator's reviewed-
  feature `make check` exited 0; it will run `make check` again immediately
  after this documentation release and record final closure evidence and the
  tested commit in the closure conversation before integration.

## Performance and Goal Impact

Predeclared cost: no simulation behavior or deterministic work-counter change
against T11.F07, the previous closure. Compare all gate counters and wall time
per creature-tick against both T11.F07 and the pinned T11.F04 epoch; retain
the existing threshold and its cross-definition qualifications. No severe
simulation cost or epoch re-pin is justified by this observation feature.

Extra work is one observed baseline battery plus one ordinary battery per
executed node, for one founder and at most 36 evolved genomes; generation
aggregation scans the already available final population. Measure this within
existing observation timers outside simulation phases. The plan expected the
small bounded sample to keep the 10-second release founder and then-current
90-second evolved-neighborhood budgets; the original failed reading and the
subsequent user-authorized permanent cap adjustment are recorded below.
Investigate an excess without reducing samples, skipping knockouts, or changing
goal parameters. No claim about the theoretical compute cost of a new
biological mechanism applies.

At closure, record dated founder mesh counts and, per goal seed, whole-
population median/maximum generation, sample generation range, total/reachable/
executed/knockout counts, route-variable genomes and cap-hit executions with
denominators. Record all existing goal indicator readings and compare every
existing deterministic field with T11.F07, excluding only the new fields and
the feature label. No neighborhood component is predeclared to move. These
are the first mesh readings under this version; do not claim improvement or
compare them numerically to the research note's whole-population 48-snapshot
census without disclosing the changed sample and battery. Insufficient depth
is a finding for T01, not permission to extend this goal run.

Measured 2026-09-06: [gate report](../../progress/features/t11-f14-mesh-execution-observability.json)
and [single goal report](../../progress/features/t11-f14-mesh-execution-observability-goal.json).
Both are appended to the existing benchmark series; historical reports and
the T11.F04 epoch remain unchanged.

| Measurement | T11.F14 | Against T11.F07 | Against T11.F04 epoch |
| --- | ---: | --- | --- |
| Gate ms/creature-tick | 0.004426850 | +5.933561%, ok | +5.415966%, ok |
| Goal ms/creature-tick | 0.007140584 | +0.079017%, ok | -0.205263%, ok |
| Gate six work counters | unchanged | all 0.000000% | all unchanged except graph -66.666133% across definitions |
| Goal six work counters | unchanged | all 0.000000% | mesh +13.470830% (flag), VM -48.989220%, graph -60.595625% across definitions, plasticity +8.030172%, actions -2.107840%, births -1.079622% |

Neither report has a severe comparison. The existing epoch mesh flag and
graph-definition qualification are retained; no normalized compute threshold
was weakened.
Gate founder observation is 0.251187 seconds (T11.F07 0.465682); goal founder
is 0.484453 seconds, both below 10 seconds. Goal simulation is 709.646496
seconds and final-state observation is 3.664995 seconds. Summed simulation
and observation timers are 852.274173 seconds (14m12.274s), against T11.F07
851.609682 seconds. This exceeds the rough eleven-minute workflow estimate,
not the single-run requirement.

The **original 90-second evolved-neighborhood budget was exceeded**: 138.478229
seconds versus T11.F07's already-over-budget 138.421668 (+0.040862%). Per seed,
current/prior seconds are 6.316832/6.796962, 2.182635/2.420632, and
129.978763/129.204074. Investigation identifies the same seed33 concentration
as the existing report, now 93.9% of the total; all genomes' old structural
companions, trial counts, outcomes and deterministic fields are identical.
The additional baseline/knockout readings add no material measured total
regression (56.562 ms net difference), but this timer covers the complete
neighborhood and does not isolate their cost. The inherited budget excess is
recorded, not claimed to pass or used to justify smaller samples, skipped
knockouts, new profiling runs, or a longer evolution run.

**Original closure blocker, 2026-09-06:** 138.478229 seconds exceeded the
then-binding 90-second evolved-neighborhood cap by 48.478229 seconds. The
[T11.F07 budget resolution](t11-f07-reward-trace-clock.md#performance-and-goal-impact)
explicitly limits its 180-second allowance to F07; later features do not
inherit it. Matching that prior workload and staying below the overall
profile investigation threshold did not satisfy that separate component cap.
This was the independent review's sole P1 finding and blocked closure.

**Authorized permanent resolution, 2026-09-06:** in response to the proposed
180-second allowance, the user said, "You can permanantly adjust the cap and
merge". This authorizes a permanent evolved-neighborhood cap of **180 seconds
per goal-profile run, summed across seeds**, for T11.F14 and future closures.
The live rule is recorded in [the shared workflow](../../workflow.md#review).
The existing guarded measurement of 138.478229 seconds meets the new cap;
its original 90-second failure remains recorded. This is an explicit
operational budget adjustment after measurement, not a predeclared cost or a
claim that the old cap passed. The 10-second founder cap, 15-minute total
profile investigation threshold, profile parameters, sample/trial counts,
mutation floors, normalized compute thresholds, and historical reports and
baselines remain unchanged. No new runtime measurement is needed for this
documentation-only adjustment. The budget blocker is resolved; the spec stays
In Progress until closure updates and required final checks are complete.

Founder depth is 0: total/reachable/executed/knockout = **2/2/2/0**,
route-variable genomes **0/1**, cap hits **0/80** executions. Goal sample
counts below are sums across each seed's 12 genomes; individual readings and
actual generations remain in the report.

| Seed | Final population | Whole-population median/max generation | Sample generation range | Total/reachable/executed/knockout | Route-variable genomes | Cap hits/executions |
| --- | ---: | --- | --- | --- | --- | --- |
| 11 | 10350 | 23/43 | 8–31 | 31/30/26/2 | 0/12 | 34/960 |
| 22 | 11646 | 22/44 | 13–36 | 27/27/25/1 | 0/12 | 0/960 |
| 33 | 10464 | 22/44 | 15–29 | 34/34/27/4 | 0/12 | 23/960 |

This is the first `mesh-execution-v1` reading, at tens of generations, not
2000 generations merely because the profile ran 2000 ticks. The three
samples span only 8–36 generations; depth beyond this remains unmeasured.
No sampled genome varies its route on the 48 independent snapshot probes.
The executed-count union and cap/knockout measurements use all 80 executions,
including sequences. These finite observations do not establish cognition,
ecological neutrality, or useful redundancy. The research note used a
whole-population 48-snapshot census, a different sample and battery, so no
numerical improvement over that census is claimed. Broader depth evidence
belongs to T01; this goal was not extended.

Existing goal readings (all exactly equal to T11.F07): births/100 ticks
13354.716667; final populations 10350/11646/10464 and births
267777/266319/267187, no extinction; clades 214/170/195 and entropy
4.335792/4.233764/4.015665. Structure min/p25/median/p75/max/mean is
1/96/101/119/489/117.196334. Current-memory either counts are 0/1/0;
temporal persisted-output either counts 25/11/7, operator-state 14/25/9,
previous-slot 0/2/0, with all component counts/fractions unchanged in the
report. Founder any-event silent/changed/dead counts are 83/116/9 of 208;
evolved pooled counts are 605/452/43, 589/471/40, and 670/391/39 of 1100,
including unchanged sequence-only changes 9/7/6. Every per-operator and
per-event bucket remains in the report and passed full equality. Strategy
count, strategy causal distinctness, evolutionary activity, adaptive novelty,
memory dependence, learning dependence, prediction dependence, information
integration, and reciprocal interaction all remain `Undefined`.

## Success Criteria

- [x] Fresh gate/goal reports carry all six defined mesh quantities; historical
      fields default truthfully and existing deterministic content is unchanged.
- [x] Each evolved reading carries actual sampled generation and its goal
      seed's whole-final-population median/maximum generation; extinct seeds
      have explicit undefined distributions.
- [x] Tests demonstrate applied execution, input variation, and the precise
      knockout intervention; observations never mutate simulation subjects.
- [x] Required mutation, benchmark, review, and reviewed-feature `make check`
      evidence is recorded; T11.F14 is checked and this spec is Complete for
      integration. The final closure-content check remains the orchestrator's
      required integration gate, with its result recorded in the conversation.

## Notes for AI Agents

- Planning started from `795cbb144b21856dabd6b962ce80d583ffc4ab80` in
  `/Users/istefanek/projects/petri/.worktrees/t11-f14`, branch `codex/t11-f14`.
  Track is already In Progress and master already Active; no promotion needed.
- Readiness self-review, 2026-09-06: 0 P1, 0 P2, 0 P3; Ready, no revision.
  Checked the template, owning roadmap including knockout/depth requirements,
  dependency contract, runtime boundaries, and testability. Static bypass,
  snapshot-only route variation, and final-living-population depth are explicit
  measurement definitions; bounded-battery limitations remain. This is a spec
  review, not runtime validation or the independent final review.
- Model record: Astra (`gpt-6-astra`) medium orchestrator, persistent xhigh
  spec owner/advisor, persistent low implementer, fresh high final reviewer.
  Planning/readiness self-review is not an advisor consultation or independent
  validation. Consultation count, decisive guidance, final finding counts,
  remediation passes, requirement corrections, user interventions, and usage
  are recorded below; exact aggregate closure usage remains unavailable.

- Implementation consultations: 6. (1) Accepted the private compact
  execution mode reusing the untraced backend dispatcher with `RECORDS_HOPS`
  false; exact shared battery bookkeeping, isolated static bypass, and whole-
  population `u64` generation aggregation. (2) Accepted removing an unused
  observation equality derive and using exact termination-variant matches,
  retaining the existing trace enum. (3) Accepted fixture-only
  `lamarckian: false` for the required plasticity initializer; no inheritance
  occurs in the parity test. The repeated compile errors in consultations 2
  and 3 came from queued check/test batches; later commands stop on failure.
  Consultations 4 through 6 are recorded below. No optional scope additions.
- Diff self-review, 2026-09-06: checked reuse, simplicity, and efficiency.
  Production dispatch/routing loop is unchanged; the compact mode delegates
  backend execution and adds no normal-mode allocations. Battery scenario
  drawing and tick bookkeeping remain shared and unchanged. Existing
  `Indicator`, resolver, signature, reachability, rank sampler, ordered
  standard collections, and serde defaults suffice; no dependencies,
  configuration, observer framework, or production cleanup were added.
  No further simplification was warranted. Focused verification and fresh
  mutation evidence are recorded below when complete.

- Consultation 4 accepted two pre-review test strengthenings: assert the
  generated observed-ID set is a subset of structurally reachable IDs, and
  include a sampled generation above `u32::MAX` while retaining distinct
  unsampled depths. These close explicit verification coverage, not production
  defects. No spec revision or scope change. Repeated the diff self-review
  after these test-only edits; no additional simplification needed.
- Consultation 5 found implementation and mutation evidence sufficient for
  independent review, including the equivalent survivor, but identified the
  then-unresolved observation-cap blocker above. No code remediation or additional
  goal run is recommended from the available evidence; independent final
  review and the orchestrator's `make check` remain required.
- Requirement correction 1, 2026-09-06: the prior inference that an inherited,
  materially unchanged observation cost could carry F07's allowance forward
  was invalid. F07 explicitly scoped its adjustment to that closure. The
  90-second requirement remained binding until the explicit authorization
  below; the correction itself changed no cap or acceptance criterion.
- Consultation 6 / authorized requirement adjustment, 2026-09-06: apply the
  user's permanent 180-second evolved-neighborhood cap in the shared live
  workflow, preserving original measurements and all other limits. The
  existing report satisfies the new cap and resolves the budget P1; no code
  remediation or further benchmark run is needed for this adjustment.
- User intervention, 2026-09-06: after the orchestrator offered an F14-only
  180-second allowance or preserving the blocked worktree, the user explicitly
  authorized permanently adjusting the cap and merging. The permanent scope
  applies to F14 and future closures, as recorded in the resolution above;
  final closure checks and the authorized local integration remain pending.
- TDD evidence: initial runtime and battery/CLI red compilation logs are
  `/tmp/t11-f14-red-runtime.log`, `/tmp/t11-f14-red-battery.log`, and
  `/tmp/t11-f14-red-cli.log`. Fixture development exposed empty-graph sink
  nonexecution and the action-type discriminator; fixtures now include a
  compute node and explicitly emit Move for direction comparison.
- Initial fresh mutation run: `MUTANTS_ITERATE=0 make rust-mutants`, exit 0;
  `49 mutants tested in 4m: 1 missed, 19 caught, 29 unviable`; no timeouts.
  Log `/tmp/t11-f14-mutants-fresh.log`. Full survivor list: only
  `crates/v3-core/src/runtime/mesh.rs:265:40: replace || with && in
  <impl MeshExecutionMode for ObservedMeshExecution>::record_hop` —
  **equivalent**: this mode sets `RECORDS_HOPS=false`, so the executor already
  passes no route on terminal/exhausted dispatches; ordinary dispatches have
  both flags false and retain the same selected position. No production code
  was changed to kill a mutant, no skips/exclusions added. Final fresh evidence
  follows the pre-review test strengthening; incremental evidence is not used.

- Independent final review, 2026-09-06: **1 P1, 0 P2, 0 P3**. The sole P1
  is the measured evolved-observation cost of 138.478229 seconds exceeding
  the then-binding 90-second cap. **Resolved** by the explicit user-authorized
  permanent 180-second cap above, which the existing guarded report meets.
  The original finding count and original failed measurement are retained.
  The reviewer found no code correctness or maintainability issues and
  independently confirmed full prior-field deterministic equality and the
  mutation survivor's equivalence. Post-review code remediation passes: **0**.
- Current workflow record: **6 advisor consultations**, **1 requirement
  correction**, **1 user-authorized requirement adjustment**, **1 user
  intervention**, and **exact aggregate closure usage unavailable**. The spec
  is Complete and the feature row is checked for integration. The orchestrator
  ran `make check` on the reviewed feature
  content on 2026-09-06: **exit 0**, log `/tmp/t11-f14-make-check.log`.
  This verifies the implementation before the authorized budget-document
  change. The orchestrator will run the final closure-content `make check`
  immediately after this documentation release; its result and tested commit
  must be shown in the closure conversation before integration. No result for
  that forthcoming run is claimed here.
- Available usage checkpoint: the orchestrator reported the native goal's
  blocked-checkpoint snapshot as **617,721 tokens** and **3,623 elapsed
  seconds**. That snapshot remained frozen while resumed work continued;
  it is a checkpoint reading, not total closure usage.

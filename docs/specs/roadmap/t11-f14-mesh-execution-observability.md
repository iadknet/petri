# T11.F14 — Mesh Execution Observability

**Status**: In Progress
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

- [ ] Add compact applied mesh observations through the existing executor
      seam and core tests proving observation preserves production behavior.
- [ ] Extend the neighborhood battery with the defined mesh measurements and
      isolated structural knockouts, sharing scenario/sequence bookkeeping;
      add constructed fixtures and property tests before implementing results.
- [ ] Wire founder, sampled-genome, sampled-generation, and whole-final-
      population depth readings into the existing bench report, with historical
      defaults and profile/serialization tests.
- [ ] Store fresh gate and single goal reports at
      `docs/progress/features/t11-f14-mesh-execution-observability.json` and
      `...-goal.json`; append them to `docs/progress/benchmark-series.json`.
      Add the closure row and concise measurement definitions to
      `docs/progress.md`, including per-seed depth beside evolved mesh readings.
      Keep historical report content and epoch baselines unchanged.

## Verification

- [ ] TDD fixtures cover static and sensor-conditional routes, multiple target
      positions sharing a destination, an unreachable node, a reachable losing
      branch, repeated visits, missing entry/target, an actual hop-cap loop,
      terminal completion at the cap, and unused terminal/exhaustion route
      scores. A node selecting the same multi-position set in every snapshot
      does not establish input variation.
- [ ] Knockout fixtures cover a silent pass-through, a contributing action or
      upstream/memory producer, a contributing conditional router, entry
      bypass, bias/position winner selection, no/missing/self successor, and
      sequence-only effects. Assert source genome isolation and complete
      signature comparison; independently exercise VM and graph nodes.
- [ ] Property tests in `v3-core` establish the pure count/subset bounds and
      source-preserving bypass behavior on generated well-formed genomes;
      assertions do not depend on which cases proptest drew. A production-
      versus-observed parity fixture compares complete applied outcomes and
      runtime state, including repeated graph visits and exhaustion.
- [ ] Bench tests cover known odd/even and empty population depths, generations
      above `u32::MAX`, actual sample generation, whole-population rather than
      sample-only aggregation, defaulted historical fields, profile presence,
      and byte-identical deterministic output across runs/thread counts at
      existing reduced test trial sizes. Run focused core/CLI tests and
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [ ] Run the explicit diff self-review for reuse, simplification, and
      efficiency, then fresh `MUTANTS_ITERATE=0 make rust-mutants`; record its
      summary, output path, full missed/timeout list and each resolution.
- [ ] Run `make bench` for the gate and once for the goal, using the stored
      paths above; record compute comparisons, observation cost, full prior-
      field deterministic equality with T11.F07, and readings below.
- [x] A second goal run is not applicable: the 2026-09-05 user decision in
      `docs/workflow.md` uses the single closure reading and the reproducibility
      tests in `make check`. The gate's two-run check remains required.
- [ ] `make roadmap-check` passes on document edits; final review findings and
      resolutions are recorded; the orchestrator runs `make check` on final
      feature content and records the tested commit in the closure conversation.

## Performance and Goal Impact

Predeclared cost: no simulation behavior or deterministic work-counter change
against T11.F07, the previous closure. Compare all gate counters and wall time
per creature-tick against both T11.F07 and the pinned T11.F04 epoch; retain
the existing threshold and its cross-definition qualifications. No severe
simulation cost or epoch re-pin is justified by this observation feature.

Extra work is one observed baseline battery plus one ordinary battery per
executed node, for one founder and at most 36 evolved genomes; generation
aggregation scans the already available final population. Measure this within
existing observation timers outside simulation phases. The small bounded
sample should keep the existing 10-second release founder and 90-second
evolved-neighborhood budgets; investigate an excess without reducing samples,
skipping knockouts, or changing goal parameters. No claim about the theoretical
compute cost of a new biological mechanism applies.

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

## Success Criteria

- [ ] Fresh gate/goal reports carry all six defined mesh quantities; historical
      fields default truthfully and existing deterministic content is unchanged.
- [ ] Each evolved reading carries actual sampled generation and its goal
      seed's whole-final-population median/maximum generation; extinct seeds
      have explicit undefined distributions.
- [ ] Tests demonstrate applied execution, input variation, and the precise
      knockout intervention; observations never mutate simulation subjects.
- [ ] Required mutation, benchmark, review, and `make check` evidence is
      recorded; T11.F14 is checked and this spec is Complete at integration.

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
  will be recorded before closure; task usage currently unavailable.

# T20.F05 — Structured Heritable Refinement

**Status**: In Progress
**Last updated**: 2026-09-26
**Feature**: T20.F05
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Repeated circuit motifs: an opt-in Graph mutation changes existing inherited
coefficients together along a declared semantic correspondence. Directional
relationships and homologous fields across nearby-creature slots both work,
while independent coefficient changes and persistent deletion retain local
exceptions. This establishes two engineering-supported layouts; F09 determines
whether either improves native inherited discovery.

## Non-Goals

- No learner, acquired state, permanent weight tying, pattern language, indirect
  genome encoding, new evaluator, sensor, ecological pressure or VM changes.
- No production availability switch, supply recalibration, founder changes,
  discovery/retention/ecological campaign or claim of general biological benefit.
- No automatic wiring, regrowth, new mesh nodes, routes, declarations, group
  metadata, persistent edge IDs or retrofit of dense connectivity.

## Inputs and Invariants

The owning row and F05 Notes in the [T20 roadmap](../../roadmaps/t20-input-evolvability-and-structured-variation.md)
govern scope. [F04](t20-f04-general-neutral-input-recruitment.md) provides bounded
neutral access through ordinary Graph edges and keeps both recruitment arms
opt-in. Its F02 consumer boundary supplies inherited action-vote coefficients;
compute-node plasticity and overwrite sinks are outside this candidate.
`contracts::WorldInputKey`, `Direction::ALL`, `VoteSink` and
[the nearby-creature field layouts](../../reference/v3-sensor-spec.md#54-nearbycreaturecore)
define meaning. A raw index, equal channel count or storage adjacency is not
homology. F01's ring opportunity remains inconclusive; that does not establish
a ring discovery benefit here.

**Evidence and choice (2026-09-26).** Inspected `mutation/graph`,
`mutation/engine`, `mutation/types`, `config/simulation`, `creature/genome/cgp`,
`creature/genome/vote` and `contracts/inputs`. Existing scalar weight mutation
and deletion retain independent coefficients but cannot express a single
coordinated event. Extending that dispatcher with semantic grouping meets the
contract without changing execution or inheritance. An indirect encoding with
direct offsets is a credible alternative: [Helms and Clune (2017), Offset-HybrID](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0174635)
shows how regularity and exceptions can coexist, with problem-dependent results.
Importing its encoding machinery is unnecessary for this bounded mutation;
ordinary coefficients already store local differences. No dependency is added.
The unresolved question is empirical usefulness, owned by F09.

**Prospective candidate contract.** One off-by-default configuration switch
admits one new Graph operator of weight 4 (refinement), with neutral complexity
effect. Omitted configuration means off. Existing operator weights, domain
probabilities, size-derived event supply, target biases, recruitment arms and
ordinary scalar mutation remain unchanged. Off excludes the candidate before
sampling, without extra RNG draws or changed operator ordering. The enabled
candidate is available under size pressure because it adds no genomic units.

| Boundary | Required behavior |
| --- | --- |
| Ring correspondence | Existing direct `InputLeaf` edges from one concrete `NeighborFoodRing(type)`, `NeighborBarrierRing` or `NeighborOccupiedRing` to one directional vote kind (Move, Reproduce or StealEnergy) form a group at a fixed relative offset `(sink_direction - source_direction) mod 8`. All eight offsets and all three directed kinds have selection support. Food types and input families never mix. Only canonical channels/directions 0–7 qualify. No preference for the aligned offset or Move is supplied by a task answer. |
| Repeated-slot correspondence | Within one existing valid action-vote sink occurrence, edges from the same field of one nearby-creature bank across its four ranked slots form a group: Core width 4 per slot, Vitals width 2, Identity width 3. Fields and banks never mix; rank is not direction and is never cyclically rotated. Every field and all 27 valid vote identities have support. A same-field sum is the supported relationship, not a claim that nearby entities are interchangeable in every task. |
| Sparse groups and duplicates | Only existing edges participate, with at least two different homologous positions and at most eight (ring) or four (slots). Missing positions stay missing. Match declared input meaning, including food identity, rather than declaration index. A candidate group with more than one edge occurrence for a homologous position is ambiguous and excluded as a whole; duplicates remain independently mutable through existing operators. Duplicate declarations alone do not disqualify a unique semantic edge. For ring groups, repeated sink identities producing multiple edges at one position are likewise ambiguous. No occurrence-number pairing or deduplication rewrites the genome. |
| Inapplicable shapes | Scalars, heterogeneous area summaries, introspection, upstream/shared-memory addresses, action-queue fields and decision-state vectors receive no invented grouping. Compute-node inputs, non-vote sinks, dangling references, out-of-range channels/directions and groups containing nonfinite weights are excluded. Other valid groups in the same node remain eligible. Ordinary mutation and F04 access remain available on excluded shapes. |
| Selection and event bounds | Use the existing Graph-domain target selector over nodes with a valid group, then a bounded random selection among that node's groups. Each valid group has positive support. One event selects one group, without retries until a useful effect occurs. Scan cost may depend on existing genome size; the edit is bounded to eight existing coefficients. Eligibility and application use the same semantic grouping logic, so inapplicable padding cannot steal a valid target. |
| Step | For `m` present members, draw one additive scalar `d` uniformly from `[-0.1, 0.1]`, and propose the same increment `d / sqrt(m)` to each weight. This caps the requested Euclidean vector step at 0.1, matching the ordinary near-zero additive scalar bound while avoiding an eightfold unreported step. Coefficients keep their pre-existing differences up to floating-point rounding; never replace them with a common absolute value or a mean. Preflight all finite results and require the actual f32 vector norm to be at most `0.1 + 1e-6`; reject a numeric proposal that fails atomically, with an observable skip and no retry. Report rounded-to-zero changes separately from attempted touches. |
| Independent exceptions and pruning | Existing independent mutation can change any participating coefficient without changing its neighbors. Subsequent coordinated steps preserve those local differences within f32 precision. Remove-edge and declaration pruning remain unchanged; a deleted member is never recreated by refinement, execution, copying or serialization. A one-member remainder is no longer a coordinated group. Explicit F04 recruitment may later add a new occurrence under its own contract. |
| Lifecycle | Only selected inherited edge weights change. Preserve sources, sink identities, ordering, unselected coefficients, compute/plasticity/birth state, declarations, routes, backend kinds and genome size. Copies, native offspring construction and serde retain the resulting independent coefficients. This operator adds no lifetime state and changes no learning/inheritance rule. |
| Applied accounting | Give the candidate a distinct operator identity in existing event/funnel records. Record selected node, semantic layout/group, selected and actually changed coefficients, requested and actual vector norm, numeric rejection and native cost through existing observation seams or the bounded reading. Observation consumes no RNG and cannot guide selection. Keep one event distinct from up to eight coefficient touches; no unbounded runtime telemetry or new goal indicator. |

**Bounded engineering comparison.** Use two authored Graph fixtures with
learning disabled: `NeighborFoodRing(0)` to Move at offset 0 across eight
positions, and `NearbyCreatureCore.present` to Eat across four slots. Both
start with zero coefficients except one inherited `0.025` local exception;
these are operator fixtures, not authored discovery successes. Exercise their
native input resolution and vote application with nonzero sources. Also verify
other supported families/kinds/offsets, sparse cases and exclusions through
focused tests and pure-invariant property coverage.

For each fixture, compare 256 independent one-event proposals per arm, seeds
`29_005_000 + i` for `i=0..256`: ordinary native scalar refinement; a diagnostic
matched additive control touching the same `m` members with independently drawn
signs and magnitude `abs(d)/sqrt(m)`; and the coordinated native operator.
The matched additive control is test/observation-only, never an additional
production operator. It uses the same scalar amplitude distribution and vector
budget as coordination; ordinary scalar is reported separately because its
existing scale-relative branch differs at nonzero weights. Do not change that
branch to make the comparison match. This is 1,536 directed operator proposals,
not 1,536 births under normal operator selection or independent lineages.

Report attempted/applied/skipped proposals, selected nodes/groups, touches,
actual changes, vector norms, preserved exceptions, input/vote effects,
genomic units, native work and host elapsed time for each arm. Report supported
layout verdicts separately; correctness on both layouts is the acceptance gate,
with no superiority margin or ecological success threshold. Preserve failures
and stop at 60 seconds release execution or 10 MiB retained evidence; no seed
replacement or extended null search. Reuse existing test/observation seams;
concise tables live in [readings](../../progress/readings/t20-f05.md). Engine
integration checks separately establish candidate availability, event accounting,
size-pressure admission and default-off RNG preservation.

## Implementation Tasks

- [x] Add the bounded semantic grouping and coordinated operator to existing
  Graph mutation, with off-by-default configuration and applied accounting.
- [x] Establish both layout contracts, sparse/ambiguous exclusions, vector
  bounds, independent exceptions, deletion and inherited lifecycle using TDD
  and property coverage; preserve default behavior and other mutation paths.
- [x] Complete the bounded comparison and document actual supported scope and
  settings in the existing mutation/runtime references and readings.
- [ ] Complete baseline evidence, final review, mutation gate and closure
  records; check the roadmap row only at completion.

## Verification

- [x] Focused TDD and proptest checks in existing Graph mutation, engine,
  runtime and inheritance seams cover the contract; exact test names, commands,
  outcomes and any regression seeds are recorded in [readings](../../progress/readings/t20-f05.md).
- [x] The 1,536-proposal release comparison records both layout verdicts,
  scalar/additive/coordinated denominators, vector bounds, applied effects and
  costs in the same readings. This is engineering qualification only.
- [ ] `cargo check --workspace --all-targets`, `make roadmap-check` and final
  `make check` pass. If defaults, founders or tick mechanics change,
  `cargo test -p v3-core --test viability` runs first.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path
  and every survivor here as killed, equivalent or explicitly user-deferred.
- [ ] Gate and one goal summary stored under `docs/progress/features/`; raw
  provenance/hash/byte counts and verification time checked, series entries
  point to summaries and no new full report is staged. A second goal
  determinism run is not applicable under the workflow's one-goal-run rule.

**Implementation self-review (2026-09-26):** complete; no necessary source
changes. Reuse, enum/serde/dependency choices and bounded work were checked;
readings retain the review rationale and documentation-check results. This does
not replace independent final review or the remaining closure gates.

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog is inherited
variation of repeated sensory-motor circuit motifs with locally different
strengths. It reaches creatures at birth through their existing input and motor
machinery. The candidate stays off pending later qualification. No environmental
pressure, new goal indicator or changed standard-world setting is introduced.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure F04 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Default simulation | Existing deterministic simulation counters, persistence, diversity, cognition, neighborhood, mutation-effect and input-use values equal F04. Additive zero-valued operator schema entries and effective-config identities may differ solely for the off-by-default schema; explicitly identify them and verify all existing values/trajectories. No favorable movement is sought. |
| Default compute | No expected production execution-cost increase. Small constant admission/config checks must stay within existing thresholds; they do not justify a severe regression. Existing +10%/+50% normalized-work and +25%/+100% wall flags apply. No epoch re-pin or relaxed floor is authorized. |
| Enabled candidate | Group selection scans existing genotype data and can cost more host mutation time than scalar refinement; at most eight weights change. No new genomic units, sensor declarations, carrying/replication units or size-derived event supply arise directly from this event. Direct vote edges retain native execution and zero separate per-edge physiological price. Changed votes can alter subsequent executed behavior and realized costs. No expected direction for usefulness, survival, diversity or cognition is asserted. |
| Observation caps | Founder neighborhood 10 s; evolved neighborhood 180 s summed across worlds; read 10 s; mutation effects 60 s; input use 60 s. Goal end-to-end investigation threshold 900 s. Controlled comparison: 60 s release execution / 10 MiB retained evidence. |
| Artifacts | Existing gate/goal summaries; full local reports under the main checkout's ignored `.bench-artifacts/t20-f05-structured-heritable-refinement/`. Bounded candidate tables in readings, with no new standard profile or campaign. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f05-structured-heritable-refinement
make bench PROFILE=goal FEATURE=t20-f05-structured-heritable-refinement
```

**Measured verdict.** Pending required gate and goal profiles. Record each CLI
and observed outer-process exit status with its source, `severe`, crossed
thresholds and epoch disposition at closure.

## Success Criteria

- [ ] One opt-in native event refines existing semantic groups in both ring and
  nearby-slot layouts with bounded additive steps and truthful accounting.
- [ ] Local exceptions and deletion persist; unsupported shapes remain outside
  coordination; ordinary access/refinement and default trajectories are intact.
- [ ] The bounded comparison and all closure gates pass, with engineering
  scope separated from F09's unresolved discovery/usefulness verdict.

## Notes for AI Agents

- Decision: Structured refinement remains opt-in pending the subsequent T20
  qualification and availability gates; these two engineering-supported layouts
  do not establish a general ecological coordination benefit.

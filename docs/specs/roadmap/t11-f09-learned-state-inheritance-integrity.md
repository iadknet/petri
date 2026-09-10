# T11.F09 — Learned-State Inheritance Integrity

**Status**: In Progress
**Last updated**: 2026-09-10
**Feature**: T11.F09
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Inherited neural adaptation. When learned weights are heritable, structural
mutation preserves their association with surviving connections and faithful
copies, and new connections start with explicitly defined weights. Ordinary
offspring retain the existing reset behavior.

## Non-Goals

- New inheritance modes, enabling Lamarckian defaults, recombination, persistent
  ancestry IDs, a new controller, or changes to mutation supply and targeting.
- Inheriting eligibility traces, temporal graph state, or dispatch history;
  changing learning mathematics, runtime clocks, or environmental pressures.
- Proving ecological usefulness of learning or adding goal indicators.

## Inputs and Invariants

- Source of truth: the owning T11.F09 row and note. Dependency outputs are
  [T11.F07](t11-f07-reward-trace-clock.md)'s once-per-world-tick eligibility and
  newborn trace reset, and
  [T11.F08](t11-f08-function-preserving-duplication-and-module-growth.md)'s
  reference-preserving graph/mesh copies and graph split. Dependencies remain
  owned by the track. Extend the [reproduction](../../reference/v3-reproduction-spec.md),
  [graph backend](../../reference/v3-graph-backend-spec.md), and
  [mutation](../../reference/v3-mutation-spec.md) references with the resulting
  inheritance rule.
- Existing seams: `simulation/actions/{reproduction,cgp_reproduction}.rs`,
  `creature/genome/cgp.rs` insertion/removal/duplication helpers,
  `mutation/graph/operators.rs`, `mutation/topology/structural.rs`, mutation
  engine event application and rollback, and
  `runtime/plasticity/hebbian.rs`'s lazy weight initialization. Genome mutation
  stays in the existing engine; reproduction transfers learned state; runtime
  continues to own learning. Keep the existing runtime weight vector layout.
- Local diagnosis, 2026-09-10: reproduction snapshots parent weights before
  mutation but reads them by the child's mesh index afterward. CGP inheritance
  repeats that lookup by compute-node index and clones the whole input-weight
  slice without checking edge correspondence or length. Mesh removal, graph
  interleaved insertion/copy, and edge removal can therefore attach learning
  to unrelated connections. Lazy initialization only fills empty slices, so a
  stale nonempty slice is not repaired at first execution.
- Research checked 2026-09-10: Stanley and Miikkulainen's
  [NEAT historical markings](https://nn.cs.utexas.edu/downloads/papers/stanley.cec02.pdf)
  distinguish connection ancestry from array position. That supports tracking
  origin, not adopting NEAT's population-wide innovation registry or crossover.
  Options: extend existing mutation edits with birth-local provenance
  (selected); persistent innovation IDs (unnecessary lifetime and schema
  scope); infer correspondence from final topology (ambiguous for identical
  edges and copies); bake learned values into the child genome before mutation
  (changes genotype/runtime separation and mutation inputs). The existing
  clone, splice, and rollback paths provide the needed information; no package
  improves this bounded repair. Exact internal representation is the
  implementer's choice, reviewed at the required advisor checkpoint.

**Connection correspondence.** Provenance describes an edge occurrence in the
parent, not merely an equal source or an array position. Track it across the
whole birth mutation sequence through the existing edits. Two equal parallel
edges can carry different learned weights and must remain distinguishable.
Failed or rolled-back edits restore correspondence with the genome. Tracking
must not consume RNG, change event selection, or affect semantic-change counts,
parseability, serialization, or persistent genotype identity.

| Applied edit | Learned-state rule |
| --- | --- |
| Surviving mesh/compute node or input edge moves through insertion/deletion | Retain its own parent edge origin after every index remap. |
| Faithful compute-node, subgraph, mesh-node/slice, or edge-bundle copy | Copy each source edge's origin; original and copy receive independent runtime storage. Intra-copy source remaps preserve that origin. |
| New node/edge, backend replacement, or unrelated replacement | No parent origin; initialize from the final child genomic edge weight. A reused mesh ID or index does not confer origin. |
| Identity split | The existing consumer edge keeps its origin and learned weight; the new identity edge starts at its authored genomic weight (currently 1). |
| Edge/source removal or direct rewiring | Removed edges disappear. A retained edge whose source is deleted or actually changed by explicit retargeting loses origin and starts from its child genomic weight; pure index repair and faithful-copy remaps are not rewiring. |
| Explicit edge-weight mutation | Reset that edge to its resulting genomic weight, so the requested mutation is not masked by an old learned value. Other surviving edges retain origin. |
| Compute-kind/parameter, input-reference meaning, routing, or plasticity-config changes | Preserve surviving edge origins; these do not replace edge occurrences. Final child plasticity configuration determines inheritance eligibility. |

The final child's existing `plasticity.lamarckian` flag is the inheritance
switch, as in the current implementation. A child compute node without
plasticity or with the flag false starts with an empty runtime weight slice;
its first plasticity execution uses child genomic weights. A child node with
the flag true receives each available corresponding parent learned value;
where origin or parent runtime value is absent, use the final child genomic
weight. The parent's flag does not add another eligibility condition: weights
learned by a parent in ordinary mode can be inherited if the child's existing
flag mutation enables it. No new clamping rule is added at birth.

Preserve lazy initialization for nodes with no available inherited values.
When a node has any inherited values, materialize exactly one runtime value
per final child input edge, mixing inherited values and genomic defaults as
specified. Missing, short, or uninitialized parent slices never justify reading
another edge. An unmutated eligible child retains the existing learned values.

The parent's genome and runtime state remain untouched; the child genome is
the usual mutated genetic copy, without learned values written back into it.
All newborn eligibility traces, decayed bases, graph temporal vectors and
dispatch history remain empty. Shared-memory inheritance and birth accounting
remain unchanged. First execution and first reward delivery must use the
aligned weights without importing parental credit. Arrays used for runtime
state remain an implementation layout, not the definition of homology.

## Implementation Tasks

- [ ] Add failing coverage for the correspondence and initialization contract
  using existing reproduction and mutation test seams.
- [ ] Carry birth-local correspondence through existing structural edits and
  connect it to ordinary/Lamarckian offspring weight construction.
- [ ] Update the three reference contracts, complete required checks and
  measured reports, and record closure evidence without broadening scope.

## Verification

- [ ] Focused reproduction/mutation tests prove insertion, deletion, all copy
  forms, split, rewiring, explicit weight mutation, config eligibility, absent
  parent values, multi-event composition/rollback, independent child storage,
  and newborn trace reset. Record exact test names, red/green commands and
  outcomes in the [readings](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md).
- [ ] Pure correspondence/shape invariants have proptest coverage; applied
  reproduction and first-runtime-use coverage establish integration. Results
  and any regression files are recorded in the readings.
- [ ] `make check` and `make roadmap-check` pass; command evidence in readings.
  Run viability first if implementation changes production defaults, founder
  behavior, or tick-loop mechanics.
- [ ] Store `make bench PROFILE=gate FEATURE=t11-f09-learned-state-inheritance-integrity`
  and one `make bench PROFILE=goal FEATURE=t11-f09-learned-state-inheritance-integrity`
  run at the report paths below; record threshold verdicts and comparisons in
  readings. Retain all three standard goal worlds and their existing pressures.
- [x] Second goal-profile determinism run: Not applicable under the standing
  2026-09-05 one-run rule; cross-process coverage and the gate two-run check
  remain in `make check`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path,
  and resolve every survivor here as killed, equivalent, or user-agreed deferred.

Mutation survivors: pending the required fresh run.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: inherited neural
adaptation is carried by the offspring's brain, affecting its existing actions
and bodily outcomes. No new sensor, reward, environmental pressure, or energy
tariff is introduced. This repairs an existing heritable-state mechanism.

Expected compute cost is birth-local provenance bookkeeping and construction
of correctly sized inherited weight slices, proportional to the graph
structure tracked during birth and inherited edges. Avoid persistent runtime bookkeeping or a
second mutation execution. No work-counter definition change, severe cost
allowance, threshold weakening, or epoch re-pin is preapproved.

For the gate, compare all six normalized work counters and wall time per
creature-tick with previous closure T11.F18 and pinned epoch
`remove-complementary-nutrition`, as recorded in
`docs/progress/benchmark-series.json`. Keep work warning/severe thresholds
+10%/+50% and wall warning/severe thresholds +25%/+100%. For the current
`goal-worlds-v1` series, compare each environment with previous/epoch
T12.F04. Preserve the historical `goal-v1` series and identify any comparison
limits; do not compare different world recipes as unchanged inputs.

Founder neighborhood operator and birth-bucket outcomes are expected unchanged:
this feature neither changes genomic mutation nor starts founders with learned
state. Keep the battery, samples, trials, seeds, floors and operator weights
fixed. Evolved neighborhood silent/changed/dead fractions and drift-depth
observations can move in either direction because corrected inherited weights
can change selection; report each decline and its denominator without asserting
paired causality. No directional gain is promised for population persistence,
births, lineage survival/entropy, shared or temporal memory sensitivity,
reachable/executed/contributing structure, routing, or any other existing goal
indicator. Learning dependence remains `Undefined`; inheritance correctness is
not evidence of evolved ecological learning. No new measure is introduced.

Retain founder observation's 10-second cap, the summed evolved observation
180-second cap per goal run, and the 15-minute goal investigation threshold.
Investigate unpredicted founder changes, crossed floors, and severe regressions;
record limitations rather than changing acceptance rules.

**Measured verdict.** Pending gate and goal runs; no epoch re-pin authorized.

- Reports: [gate](../../progress/features/t11-f09-learned-state-inheritance-integrity.json),
  [goal](../../progress/features/t11-f09-learned-state-inheritance-integrity-goal.json).
- Full readings: [t11-f09-learned-state-inheritance-integrity.md](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md).

## Success Criteria

- [ ] Surviving and faithfully copied connections retain the correct eligible
  learned value through structural mutation without position-based cross-talk.
- [ ] New and reset edges use the defined child genomic values; ordinary
  inheritance, independent storage and all newborn trace resets are preserved.
- [ ] Reference contracts, verification, measured evidence, and roadmap closure
  agree with the applied implementation.

## Notes for AI Agents

- Plan authored and self-reviewed on 2026-09-10 by the persistent spec owner,
  `gpt-6-astra` at `high`; orchestration uses Astra `low`, implementation and
  remediation use one persistent Astra `low`, and final independent review uses
  a fresh Astra `medium` agent. Self-review/advice is not independent validation.
- Readiness review, 2026-09-10: **Ready**, no P1/P2/P3 findings. One wording
  revision clarified that a retarget draw retaining the same source is not a
  new connection and that provenance may traverse existing graph structure.
  Reviewed template/status consistency, dependency invariants, bounded scope,
  and observable verification. Runtime correctness remains for implementation
  tests and the independent final review. `make roadmap-check` passed. Track
  already `In Progress` and master already `Active`; neither needs promotion.
  F07 and F08 are checked.
- Required advisor consultations, review findings/remediation, requirement
  corrections, and workflow interventions: none yet; record them as they occur.
  Planning itself does not count as advisor consultation.
- Cost record: token usage unavailable unless measured. Preserve actual role,
  review and command evidence; do not infer a token count.

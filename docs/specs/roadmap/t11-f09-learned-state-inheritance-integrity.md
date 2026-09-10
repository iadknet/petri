# T11.F09 — Learned-State Inheritance Integrity

**Status**: Complete
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
- Birth-local `Option<Vec<Vec<Option<f32>>>>` metadata carries each compute input
  occurrence's available parent value through existing edits. It is absent
  outside reproduction and excluded from serialization, equality, and debug
  genome identity. Newborn construction consumes it; absent parent values do
  not allocate tracking rows.
- Stanley and Miikkulainen's
  [NEAT historical markings](https://nn.cs.utexas.edu/downloads/papers/stanley.cec02.pdf)
  support distinguishing connection ancestry from array position. Birth-local
  correspondence is sufficient here; persistent innovation IDs add unnecessary
  schema scope, final-topology matching is ambiguous for parallel edges, and
  baking learning into genomic weights changes genotype/runtime separation.

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

- [x] Add failing coverage for the correspondence and initialization contract
  using existing reproduction and mutation test seams.
- [x] Carry birth-local correspondence through existing structural edits and
  connect it to ordinary/Lamarckian offspring weight construction.
- [x] Update the three reference contracts, complete required checks and
  measured reports, and record closure evidence without broadening scope.

## Verification

- [x] Focused reproduction/mutation tests prove insertion, deletion, all copy
  forms, split, rewiring, explicit weight mutation, config eligibility, absent
  parent values, multi-event composition/rollback, independent child storage,
  and newborn trace reset. Exact test names, red/green commands and outcomes
  are in the [readings](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md).
- [x] Pure correspondence/shape invariants have proptest coverage; applied
  reproduction and first-runtime-use coverage establish integration. Results
  and any regression files are recorded in the readings.
- [x] `make check` passes on code commit
  `2c5c0a32d823a8d472b5b340ca21d713143c41e5`; `make roadmap-check` passes.
  Command evidence is in the readings. Production defaults, founder behavior,
  and tick-loop mechanics are unchanged.
- [x] Gate and single goal reports exist at the paths below, produced by
  `make bench PROFILE=gate FEATURE=t11-f09-learned-state-inheritance-integrity`
  and `make bench PROFILE=goal FEATURE=t11-f09-learned-state-inheritance-integrity`.
  Threshold verdicts and comparisons are in the readings; all three standard
  worlds retain their existing pressures.
- [x] Second goal-profile determinism run: Not applicable under the standing
  2026-09-05 one-run rule; cross-process coverage and the gate two-run check
  remain in `make check`.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` coverage and test-only
  incremental kill confirmation are complete; every survivor is resolved below.

Mutation disposition across all 67 final-production mutants: 55 caught
(51 fresh plus 4 incrementally confirmed), 10 unviable, and 2 equivalent.
There are no timeouts, deferrals, new exclusions, weakened tests, or unresolved
survivors. The readings preserve the initial baseline failure, unchanged
fingerprint pin, and exact fresh/incremental command evidence.

Fresh output: `/tmp/t11-f09-mutants-second-fresh/` (67 tested: 51 caught,
6 missed, 10 unviable, 0 timeouts). Incremental confirmation:
`/tmp/t11-f09-mutants-incremental/` (6 tested: 4 caught, 2 missed/equivalent,
0 timeouts). Standard tool output:
`/Users/istefanek/.local/share/petri-tools/mutants/t11-f09/mutants.out` (now the
incremental result). Each preserved directory contains `outcomes.json`, full
survivor lists, logs, and diffs. Timeout lists are empty; no deferrals remain.

Full original survivor list, locations in `crates/v3-core/src/creature/genome/cgp.rs`:

| Location and mutant | Final disposition |
| --- | --- |
| `207:9: replace <impl PartialEq for CgpGraphBackendDef>::eq -> bool with true` | Killed incrementally by `birth_genome_equality_requires_every_genetic_field`. |
| `210:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed incrementally by the same property. |
| `208:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed incrementally by the same property. |
| `209:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed incrementally by the same property. |
| `388:39: replace + with * in CgpGraphBackendDef::duplicate_compute_nodes_in_place` | Equivalent: inserting an identical cloned row at `source` or `source + 1` produces the same adjacent values, including each descending insertion. |
| `414:29: replace > with >= in CgpGraphBackendDef::reindex_input_refs_after_removal` | Equivalent: equality already returns `false`, so the comparison is reached only for unequal indices. |

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

**Measured verdict.** Gate and one goal run passed (`comparison.severe=false`).
Gate work is unchanged from T11.F18, with no flags. Goal aggregate plasticity
work (+40.887%) and wall time (+41.720%) warn against T12.F04; the user reported
external CPU contention during that run, limiting timing attribution. All caps
remain satisfied (whole measured goal 639.969 s). Founder neighborhoods and
comparable drift readings are unchanged; evolved outcomes are mixed, including
Confluence final population -54.189%, fruit share 0.049737→0.000017, and shared
memory sensitivity 2/21818→0/9995. Per-case plasticity work rises 64.501% in
Orchards and 53.275% in Canyon; full comparisons and all neighborhood declines
with denominators are in the readings. No thresholds changed or epoch re-pin
was authorized. Learning dependence remains `Undefined`.

- Reports: [gate](../../progress/features/t11-f09-learned-state-inheritance-integrity.json),
  [goal](../../progress/features/t11-f09-learned-state-inheritance-integrity-goal.json).
- Full readings: [t11-f09-learned-state-inheritance-integrity.md](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md).

## Success Criteria

- [x] Surviving and faithfully copied connections retain the correct eligible
  learned value through structural mutation without position-based cross-talk.
- [x] New and reset edges use the defined child genomic values; ordinary
  inheritance, independent storage and all newborn trace resets are preserved.
- [x] Reference contracts, verification, measured evidence, and roadmap closure
  agree with the applied implementation.

## Notes for AI Agents

- Cost: Orchestration and persistent implementation/remediation use Astra
  `low`; the persistent spec owner/advisor uses Astra `high`; independent
  review uses a fresh Astra `medium`. Token usage is unavailable, not inferred.
- Cost: Advisor consultations 6; independent review findings 1 P1 resolved,
  0 P2/P3; correction review 0 P1/P2/P3. Documentation remediation 1,
  formatting correction 1, test-strengthening pass 1; requirement corrections 0.
  Detailed evidence is in the [readings](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md).
- Cost: User interventions 1: external CPU contention during the single goal
  run limits wall-time attribution. Thresholds and the one-goal-run rule remain
  unchanged; no waiver or epoch re-pin applies.

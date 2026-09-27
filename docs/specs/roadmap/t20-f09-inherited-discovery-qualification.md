# T20.F09 — Inherited Discovery Qualification

**Status**: In Progress
**Last updated**: 2026-09-26
**Feature**: T20.F09
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Measure whether native inherited variation discovers useful new Graph input
contributions with lifetime learning off. Qualify general access separately
from a ring-specialized coordinated-refinement candidate, preserve negative and
inconclusive results, and freeze only the scope supported by the measurements.

## Non-Goals

- Reproductive retention, free ecological transfer, VM qualification, learning,
  new ecology, production availability, production founder changes, or mutation-rate tuning.
- General coordination across repeated layouts: this bounded panel tests a
  ring only; repeated nearby-creature slots remain empirically unqualified.
- A new experiment framework, benchmark profile, dependency, or goal indicator.

## Inputs and Invariants

The owning row and F09, general-access, and evidence-gate Notes in the
[T20 roadmap](../../roadmaps/t20-input-evolvability-and-structured-variation.md)
govern. [F05](t20-f05-structured-heritable-refinement.md) supplies opt-in
structured steps with independent coefficients; its two-layout engineering
success is not discovery or ecological evidence.
[T13.F07](t13-f07-current-policy-recruitment-transitions.md) supplies native
per-parent supply, target-local exposure, ancestry and causal-control patterns.
Use current source, not F07's historical default settings or proposal counts.

**Evidence and choice (2026-09-26).** Existing `neighborhood/opportunity`
contains founder-relative family fixtures and authored controls;
`neighborhood/recruitment_paths` contains native sibling proposals, compact
records and lineage Wilson intervals; `input_use` contains semantic channel
inventory and causal observations. Extend these seams with a bounded
`neighborhood/input_discovery` assay and `v3-cli input-discovery` command,
following the existing opportunity/recruitment artifact paths. A fourth bench
profile would mix task selection with ecological baselines; a separate search
framework would duplicate native mutation and recording. Neither is needed.
[Offset-HybrID](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0174635)
provides a credible indirect-encoding alternative and shows problem-dependent
benefits from regularity plus exceptions; F05 already represents the bounded
candidate in ordinary edges, so no encoding replacement is justified here.

[F01's family verdicts](../../progress/readings/t20-f01.md) are positive for
`FoodHere(1)` and `AreaFoodSummary(0)`, and inconclusive for
`NeighborBarrierRing`. Accordingly, scalar and heterogeneous-vector families
are the two primary access shapes. The ring is a separate secondary refinement
case; a positive ring discovery result cannot upgrade its ecological opportunity
verdict. The roadmap explicitly permits a specialized ring add-on. Choosing
that scope prospectively avoids inventing an unscreened nearby-creature task
and makes no general coordination claim. No F05 fixture counts as a lineage.

**Frozen experiment.** All tables below are prospective. Record the exact
configuration, initial genome, ordered scene manifest and their digests before
the first measured proposal; implementation may encode these tables, not tune
them against outcomes. An invalid instrument is a blocker, not a negative result.

| Item | Contract |
| --- | --- |
| Starts | Scalar/vector use the current canonical V3Alpha1 founder unchanged. Ring uses that founder plus exactly F01’s existing authored AreaFoodSummary(0) controller as incumbent foraging competence, with its published constants unchanged; label this start `f09-ring-area-incumbent-v1` and record its provenance, full genome, size and digest. The focal NeighborBarrierRing family is absent from this start. All five arms of a family share its frozen genotype and fresh independent lineages. Every focal family is absent initially; no focal declaration, zero-edge scaffold, source/sink mapping or useful focal payload is supplied. No F05 fixture or focal authored control is a discovery start. Preserve and judge against that family’s actual frozen incumbent, not against a different canonical genome. |
| Five arms, fixed order | B: current separate declaration/connection, recruitment Off and coordination false. S: SingleChannel access, coordination false. W: WholeFamily access, coordination false. C: WholeFamily plus structured refinement. M: WholeFamily plus diagnostic matched additive refinement. All other production mutation settings and operator/domain weights are unchanged. Run every arm on each of the three families. |
| Native proposals | Each generation draws two siblings through the native mutation engine on the parent's own `genome_size()`, using its default per-unit rate, food-type count 2, recomputed reachability and executed-node set. All supported sources, sinks, backends, operator choices, deletions and target biases remain available. Never select a source, sink, node or correspondence using the task answer. Mixed-backend descendants are retained and attributed. |
| Matched control | M differs from C only at a selected structured event: same native eligible target/group selection and amplitude distribution, but independent signs with magnitude `abs(d)/sqrt(m)` per member. Bound actual f32 vector norm by `0.1 + 1e-6`, apply atomically before any later mutation event, and record requested/actual steps and touches. Sign draws use a separate diagnostic stream. This is an observation-only intervention, not a production config/operator; production C remains the native F05 operation. Ordinary scalar mutation stays available in every arm. |
| Learning off | Mutate the stored genotype normally. Evaluate fresh copies with every Graph compute-node plasticity option removed and fresh cognition state; never write that mask back into the inherited genotype or mutation stream. Record the mask and assert zero plasticity/reward updates. This controls lifetime learning without changing the mutation opportunity pool. VM has no newly implemented learner. |
| Independent units and seeds | Family order scalar/vector/ring; arm order above. Discovery panel: 32 lineages per family/arm. Validation: 16 new lineages per family/arm, never used to tune selection, tasks or thresholds. Generations 0–255, siblings 0–1; seed = `20_090_000_000 + panel*1_000_000_000 + family*100_000_000 + arm*1_000_000 + lineage*10_000 + generation*2 + sibling`. Panel 0 discovery, 1 validation. Distinct arms have distinct seeds. 368,640 proposals maximum; siblings/generations are not independent replicates. |
| Selection | Evaluate parent and both children on training scenes. Reject children that die or violate any incumbent-preservation predicate. Choose highest fraction of focal task scenes satisfied; ties prefer sibling 0 over sibling 1 over parent, allowing neutral additions to persist. Costs are reported, not a tie-breaker; this is T13 Selection rather than CostSelection. Retain parent when neither child is acceptable. No ablation result, held-out score, declaration/connection count or future outcome enters selection. |
| Task semantics | Scalar: consume actually present type-1 food when type-0 food is absent, preserve incumbent eating/reproduction otherwise. Vector: when not reproducing, with no local or cardinal-neighbor type-0 food, move toward visible distant type-0 food; preserve incumbent behavior otherwise. Ring: avoid moving into a present barrier while preserving incumbent non-move actions and ordinary movement in barrier-free cases. Focal score denominators contain only the family’s opportunity scenes: fruit-only/non-reproducing for scalar; distant-food/no local or cardinal food/non-reproducing for vector; the frozen ring incumbent moves into the focal barrier for ring. All remaining contexts are incumbent-preservation checks. Record both denominator sets. These are task scores, not reproductive fitness. Observe final applied action and world consequence through native ticks, not a weight or pre-application vote proxy. |
| Training scenes | Reuse F01's `competence_contexts` Cartesian factors: four local-food states × cardinal ring-food factor × low/high reproduction eligibility, crossed with four cardinal far-food positions for vector, four cardinal barrier positions for ring, and no extra factor for scalar. The ring-food factor is no/E for scalar and vector, and no/N/E/S/W for ring. For ring, add off-axis distant type-0 food keyed to the focal barrier direction: N=(+2,-3), E=(+3,+2), S=(-2,+3), W=(-3,-2), relative to the subject. This native visible-food cue drives the supplied area-food incumbent toward the adjacent cardinal barrier without placing the cue inside the barrier. Reconstruct in native 12×12 worlds with subject at (6,6), age 50, energy 20/80, two food types, vision 5, vector far food at distance 2, no food growth/recovery and no initial random food. Add barrier-free ring counterparts with the identical off-axis food cues. All other costs/budgets stay native. Compute predicates from world facts; authored controller outputs are not the oracle. |
| Held-out scenes | For scalar/vector, use the same factors with cardinal ring food N/S/W instead of E and far food distances 3 and 4 instead of 2. For ring, retain the no/N/E/S/W ring-food factor and all four cardinal barrier positions, with off-axis distant-food offsets N=(+3,-4), E=(+4,+3), S=(-3,+4), W=(-4,-3). For every family translate the subject to (5,5) and use energy 25/90 instead of 20/80; ring cases additionally occur both with and without the opposite diagonal barrier at `(-sign(food_dx), -sign(food_dy))` relative to the subject, so the extra barrier does not intentionally occlude the distant cue. These scene combinations never influence selection. Primary discovery is the first chosen candidate passing all training qualification predicates; test that candidate once on held-out scenes and count at most one success per lineage. Endpoints are reported separately and cannot rescue a failed primary validation. Freeze both before held-out evaluation. |
| Instrument validity and controls | Before any measured proposal, native ticks must demonstrate the ring start’s visible distant-food cue and blocked moves in all four directions separately on training and held-out sets. Use exactly the existing F01 four-edge cardinal ring inhibition overlay (weight −2.0) as the ring positive instrument control, changing only reference indices to compose with the area-food incumbent; label/digest it `f09-ring-area-incumbent-inhibition-control-v1`. Its all-four focal causal coverage, survival, mixed-scene preservation and matched focal ablation must pass the unchanged qualification predicates. Record the start and control separately: only the former enters discovery. Control output never defines the task oracle, guides mutation or counts as discovery. Other instrument controls remain labelled separately. If this one existing-controller construction fails native visibility or behavioral validity, stop ring-dependent measurement as an invalid instrument and escalate; do not tune constants, change the controller, weaken coverage or relabel the failure as a candidate verdict. |
| Causal discovery | A chosen candidate must improve focal task fraction by at least 1/8 over its start, satisfy at least 3/4 of focal scenes, preserve every incumbent predicate and remain alive on training and held-out scenes. Semantic focal-input ablation in all backends must lose at least 1/8 on both sets; Graph-only focal ablation must also lose at least 1/8 to count as Graph discovery. Mask semantic reads using the existing out-of-range reference ablation while preserving declarations, structure, genome size and native cost settings; report changed realized work/charges caused by changed behavior. Report VM-only and jointly carried effects separately; copied old computation or mere declarations do not qualify. |
| Channel coverage and mixed scenes | Report declared/connected/executed/causal focal channels and denominators. Scalar requires channel 0; vector requires causal direction use on both dx and dy axes; ring requires causal coverage across all four cardinal directions separately on training and held-out scenes. A lineage satisfying the score only in one direction fails qualification. Preservation is checked with competing local-food, ring-food and reproduction cues, including focal zero-signal scenes. Other channels/directions remain reported, not presumed qualified. |
| Exposure and censoring | Per lineage aggregate requested/applied/skipped events by operator, selected target and backend; focal declarations/connections; structured-eligible groups, selected/changed coefficients and step-norm bounds; causal stages; first qualifying generation; size/native work/charges; and terminal reason. Keep all-proposal and chosen-ancestry exposure separate without persisting every event. Report proposal discoveries separately from chosen-chain discoveries. No discovery by 256 is right-censored, not an observed discovery time. Missing rows, cap stops, no eligibility, no selected event, no causal benefit and lost competence are distinct. |

**Verdicts and frozen scope.** Use the existing lineage-level Wilson 95%
interval implementation. A conservative contrast interval is
`[candidate.lower-control.upper, candidate.upper-control.lower]`; independent
lineage counts, not proposal counts, determine it. Report absolute rates and
contrasts separately on discovery and validation panels; no pooled rescue.

| Decision | Rule fixed before outcomes |
| --- | --- |
| A family/arm positive | Complete valid panel; Graph discovery rate at least 25%; lower contrast bound against B greater than 0; observed gain at least 0.10; conditions hold separately in the 32-lineage discovery and 16-lineage validation panels. These are conservative engineering screening margins, not confirmatory population claims. |
| Access S and W | Judge each access-only arm independently on scalar and vector. One same arm must be positive on both to qualify general access. Preserve narrower per-family readings without labelling them general qualification. If both qualify, freeze both; no post-hoc winner selection. |
| Coordinated variation | Judge C on the ring separately against W and M using the same 25%, positive lower-bound and 0.10 margins in both panels. Require qualifying lineages to have an applied focal ring structured event and report C versus B. This yields at most a ring-specialized discovery verdict. No scalar/vector result is evidence of their semantic coordination, and no nearby-slot benefit is implied. |
| Negative versus inconclusive | A complete valid comparison is negative for the declared improvement margin when its contrast upper bound is below 0.10 in either panel with adequate exposure. Adequate exposure means at least 3/4 of lineages had a selected focal access event (S/W), or an applied focal structured event (C/M); report each denominator. Otherwise a non-positive comparison is inconclusive, including inadequate exposure, broad uncertainty or a resource stop. Neither means the input is inherently useless. |
| Artifacts to freeze | Freeze every first qualifying chosen candidate and endpoint by genotype digest, exact source revision, family/arm/panel/lineage/generation seed locator and shared start/config/scene manifest, with compact training/held-out causal, cost and backend facts. Deterministic replay must reconstruct the genome and match its digest. Persist full genomes only for downstream candidate representatives after favorable qualification: the lowest successful lineage identity per qualified family/arm scope (at most five: S/W × scalar/vector, and C × ring), labelled separately from controls; all other frozen identities remain exactly replayable. Do not dump every null endpoint genotype. Freeze positively qualified settings and family/channel scope in the summary. F13 receives valid inherited comparisons and their limits, even when unfavorable. |
| Downstream | F10 and F03 require favorable general access on both primary shapes; either may proceed with access alone if coordination fails. F11/F12 inherit only that supported scope through F10/F11. A positive ring coordination result may be carried as a specialized candidate only alongside favorable W general access, because C uses WholeFamily; it does not replace primary access or qualify ecological transfer. F13 requires valid comparisons, not a positive discovery verdict. |
| Blocking update | If no access arm qualifies both primary families, mark F10, F03, F11 and F12 rows **Blocked — [evidence link]**, keep boxes unchecked and remove those IDs from the master priority list. Mark F13 blocked and remove it only if the inherited comparison itself is invalid/unavailable. A coordination failure alone blocks no access-only successor; record its unavailable scope in track Notes. Conditional learning rows stay unscheduled. F09 may close with valid unfavorable/inconclusive evidence. |
| Disposal | At final negative/inconclusive stopping verdict, remove unqualified F04/F05 candidate production dispatch/configuration and corresponding references, preserving ordinary growth, F02 correctness and useful observation fixtures. S/W remain available only when that arm qualifies both primary families; C remains available only when its ring verdict and W general access qualify. A retained ring-only refinement result cannot leave repeated-slot refinement available. Per-family partial access success remains evidence, not retained experimental availability. A measured candidate's source revision and raw evidence preserve historical reproducibility; do not fake a removed arm in later builds. An assay command may explicitly reject retired arms. Any exception to disposal requires the user's explicit decision. No candidate becomes a default here. |

**Resource bound.** The recording-design failure documented in [readings](../../progress/readings/t20-f09.md) supplies partial observations only; it does not trigger the final disposal or downstream-blocking rules above. Those actions remain suspended. One corrective execution of the unchanged fixed panel requires explicit user approval, currently pending; do not start it or treat this amendment as approval. Any approved execution uses the original seeds, scenes, selection, margins, panel sizes and caps, and supersedes the defective-output attempt rather than adding independent samples. No repeated retry is authorized.

First run the first two discovery lineages of every
family/arm as a feasibility prefix; they remain part of the full panel. Stop
without extending the search if projected full execution exceeds two host-hours.
The combined measured prefix/full run has a two-host-hour release wall cap and
512 MiB raw on-disk cap, with bounded checks between generations and records;
write completed evidence and an explicit partial-lineage/cap reason. Summary
cap is 4 MiB. No seed replacement, retry-until-success or cap extension. Run
through `scripts/bench-wait`, alone on the host. Build time is separate.

## Telemetry

Reuse native mutation summaries, input-use semantics, execution observations and
applied tick work/energy flows. Detailed scene readings may exist temporarily
for evaluation; persisted evidence is a compact projection, never serialization
of the internal `Qualification`, `Reading` or proposal objects.

| Record | Sufficient persisted evidence |
| --- | --- |
| Shared manifest, once | Source revision, schema/seed rule, exact config and family start genomes, compact ordered scene descriptions/IDs, authored control identities, their digests and effective learning/candidate settings. Baseline and control results use compact facts below; do not repeat their full scene traces. |
| One row per lineage | Identity, seeds/generation locator, completion and censoring, first eligible primary, endpoint/candidate digests and sizes; exact score/opportunity and preserved/incumbent numerators/denominators, survival, learning-mask/update facts; intact/all-/Graph-/VM-ablation scores; per-channel declared/connected/executed/causal counts and score losses; separate training and held-out qualification/backend flags; exposure aggregates; requested/actual refinement-norm bounds and changed-coefficient totals; native work and action/perception/carrying/learning charges at the frozen primary/endpoint and instrument checkpoints, and total elapsed mutation time across proposals. Sum the existing checkpoint scene values; additional all-proposal physiological-cost collection is not required. Missing/unrun values remain absent explicitly. Keep primary and endpoint roles separate; no checkpoint scene arrays. |
| Deterministic transcript digest | Incremental SHA-256 per lineage over newline-delimited canonical JSON of deterministic proposal facts in generation/sibling order: seed, parent/child digest, chosen flag, post-mutation RNG observation, units, event outcomes/targets, access/refinement facts and task outcome. Use a fixed field projection and canonical key ordering. Exclude wall times, host paths, timing-based stop decisions and other nondeterminism. Hash online and discard the projected proposal; persist only the final digest and record count. Replay regenerates the same digest and frozen genotype digests. |
| Bounded audit examples | Only generation 0, siblings 0 and 1, lineage 0 of each family/arm/panel: at most 60 compact proposal witnesses, selected by identity rather than success. Each is at most 16 KiB, for a total at most 960 KiB; no genotype or full scene/ablation vectors. Any omitted/truncated detail is explicit and recoverable by replay. Examples never determine verdicts or replace complete aggregate facts. No endpoint genomes are retained merely because their lineage is selected for audit. |

Use the existing CLI writer, hashing and replay seams; add no experiment,
resume or storage framework. Observations and aggregation consume no mutation
RNG. Collection cost remains under the original assay cap; no always-on
telemetry or T21 dependency is added. The correlated compact records can later
map to T21 export.

Before requesting corrective-execution approval, verify compact projections
against the already-recorded partial observations and focused fixtures: all
scientific counts, causal losses, coverage, costs and verdict inputs agree;
reduced deterministic replay matches transcript and genotype digests; and an
explicit full-panel byte estimate from measured record sizes covers all 720
lineages, shared context, bounded examples and selected candidate genomes.
Report those components separately. The intended evidence is a few MiB plus
necessary selected genomes, not a file allowed to grow toward the 512 MiB
emergency limit. Fix an excessive estimate before execution; do not enlarge the
cap, weaken evidence or run new qualification proposals for this preflight.

## Implementation Tasks

- [ ] Add the bounded native discovery assay, fixed tasks/arms, causal and
  learning-off checks, lineage summaries and diagnostic additive control.
- [ ] Correct the evidence writer, verify compact facts/replay/byte estimate,
  and obtain explicit approval before one corrective fixed-panel execution;
  preserve the original scene/config/start manifest and experiment parameters.
- [ ] Record separate access and coordination verdicts and frozen candidates;
  dispose of unqualified candidate scope and apply downstream roadmap blocks.

## Verification

- [ ] Focused TDD checks cover applied scene predicates, no authored solution,
  native supply/targeting, learning mask, matched event order/vector bounds,
  Graph/VM causal attribution, coverage, incumbent preservation, uncertainty,
  cap/partial records and artifact provenance; commands/results in
  [readings](../../progress/readings/t20-f09.md). Pure invariants use proptest.
- [ ] Compact projection and full-panel byte preflight preserve every verdict
  input without exhaustive proposal/scene persistence. Reduced deterministic
  replay verifies transcript/endpoint digests, seed/lineage independence,
  observation-on/off mutation fingerprints and unchanged default trajectories;
  authored positive/ablated controls validate the instrument separately.
- [ ] `v3-cli input-discovery --feature t20-f09-inherited-discovery-qualification`
  through `scripts/bench-wait` produces a complete or explicitly capped raw
  record and `docs/progress/features/t20-f09-inherited-discovery-qualification-discovery.json`;
  counts, qualified scope and raw hash/bytes verified in readings.
- [ ] Final `make check` exits 0; `make roadmap-check` validates honest closure,
  downstream blocks and priority changes. No second goal run for determinism.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary/output path and every
  survivor resolved here as killed, equivalent or user-authorized deferred.
- [ ] Gate/goal summaries stored, raw hash/bytes/time checked, series entries
  point to summaries, and no new full report is staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement only; no new natural
mechanism or environmental pressure. Candidate removal retires unsuccessful
experimental availability without changing default behavior. The inherited
mechanisms under observation model sensory circuit growth and repeated circuit
variation, reaching creatures through native inherited inputs and motor votes.

| Item | Expected direction and acceptance |
| --- | --- |
| References | F05 gate/goal closure; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Default deterministic behavior | All existing simulation, persistence, diversity, cognition, neighborhood, mutation-effect and input-use values equal F05. If disposal removes disabled candidate config fields/operator-zero entries, list only those schema/digest differences explicitly; they do not excuse changed trajectories or unrelated measurements. No favorable indicator movement sought. |
| Default compute | No expected increase or justified severe regression. Existing +10%/+50% normalized-work and +25%/+100% wall flags remain; no floor relaxation or epoch re-pin. |
| Candidate costs | Report genome size, requested event supply, mutation time, native mesh/Graph/VM work, perception, carrying and action charges. Direct Graph vote edges have zero separate per-edge physiological price; disclose this. Fresh one-tick tasks pay their native costs but are not births or ecological lifetimes. No direction for discovery, reproduction or ecological benefit is assumed. |
| Observation caps | Founder neighborhood 10 s; evolved neighborhood 180 s summed across worlds; read 10 s; mutation effects 60 s; input use 60 s; goal investigation threshold 900 s. Discovery: two host-hours release / 512 MiB raw / 4 MiB summary. |
| Artifacts | Gate/goal summaries and bounded discovery summary; raw files under the main checkout's ignored `.bench-artifacts/t20-f09-inherited-discovery-qualification/`; concise tables in readings. Candidate source revision is preserved if disposal follows measurement. |

Run after disposal and all benchmark-affecting edits are final, sequentially
without competing builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f09-inherited-discovery-qualification
make bench PROFILE=goal FEATURE=t20-f09-inherited-discovery-qualification
```

**Measured verdict.** The discovery attempt stopped on an excessive-recording
error and remains partial evidence, with no final candidate disposition;
corrective execution awaits user approval. Pending gate and goal runs; record CLI/outer exit status,
`severe`, threshold crossings and epoch decision separately for each.

- Summaries: [gate](../../progress/features/t20-f09-inherited-discovery-qualification.json),
  [goal](../../progress/features/t20-f09-inherited-discovery-qualification-goal.json).
- Full readings: [t20-f09](../../progress/readings/t20-f09.md).

## Success Criteria

- [ ] Valid bounded native observations distinguish access-only and coordinated
  refinement results with learning off, causal Graph attribution, two primary
  family shapes, mixed-scene competence and honest uncertainty/censoring.
- [ ] Supported settings/candidates are frozen; unfavorable findings remain
  visible; rejected scope is disposed and downstream gates match the verdicts.
- [ ] Required checks, review, mutation and benchmark evidence are complete.

## Notes for AI Agents

- Decision: Ring-specialized coordination scope and its area-food incumbent
  fixture are fixed before measurement; the supplied competence contains no
  focal barrier input. This feature does not qualify repeated-slot coordination
  or production defaults.

- Decision: The user rejected the excessive evidence design. Candidate disposal
  and downstream blocking based on that storage-induced stop are suspended;
  one corrective unchanged-panel execution requires explicit user approval.

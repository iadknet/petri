# T20 — Input Evolvability and Structured Variation

**Status**: Planned
**Last updated**: 2026-09-23
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Creatures can acquire useful behavior from newly available sensors and internal
inputs, preserve established abilities, and retain the improvement across
generations. Qualify general access and structured inherited variation on Graph
first; rings are one test case. A later feature extends the qualified mechanisms
to VM. Lifetime learning is a conditional investigation with its own evidence
and rollout gates, not a prerequisite for inherited improvements.

## Track Success Criteria

- [ ] Existing observations distinguish declaration, connection, executed reads,
  causal action influence and retained usefulness by input family and channel.
- [ ] Early native-cost ecological checks establish usable opportunities for
  input-dependent behavior on at least two differently shaped families, including
  a non-ring family; negative and inconclusive cases are reported separately.
- [ ] Graph recruitment can declare and neutrally connect any supported input
  family through a bounded operation, without inventing a useful mapping or
  overwriting incumbent state; pruning and ordinary sparse wiring remain possible.
- [ ] Inherited discovery improves with learning disabled on at least two family
  shapes, one non-ring, against the current declaration-then-connection path.
  Benefits from coordinated variation are attributed separately from access.
- [ ] Structured variation respects actual repeated meanings, preserves
  independent exceptions, and is qualified on two different repeated layouts
  before making a general coordination claim; a ring-only benefit is labelled
  a specialized add-on.
- [ ] The qualified inherited Graph candidate is retained across descendants,
  transfers to reproducing ecological populations at native costs, and becomes
  ordinarily available without requiring VM changes or lifetime learning.
- [ ] A later VM feature qualifies access, inherited refinement and actual costs
  against the Graph contract, including mixed-backend populations and attribution.
- [ ] The learning investigation records whether a real within-life need remains;
  any activated learning branch must demonstrate added value over the qualified
  inherited alternative before its separate production gate.

## Executable Features

- [ ] **T20.F01 — Input-Use Baseline and Ecological Opportunity** — Depends on: T19.F06, T11.F20, T11.F22, T11.F24, T11.F25, T12.F04
  - Goal: Measure where input use stops between declaration and retained behavior, and whether competent use of differently shaped inputs can improve reproduction in the existing worlds at native costs.
- [ ] **T20.F02 — Graph Neutral-Connection Contract** — Depends on: T20.F01
  - Goal: Initially silent afferents: a Graph node can acquire a refinable input connection without changing its incumbent signal or overwriting state, using existing edges wherever they suffice.
- [ ] **T20.F03 — VM Input Recruitment and Refinement** — Depends on: T20.F09, T11.F25
  - Goal: Sensory circuit growth and repeated motifs: VM nodes can acquire and refine the qualified inherited input connections through native instructions with full channel access and measured costs.
- [ ] **T20.F04 — General Neutral Input Recruitment** — Depends on: T20.F02, T13.F05
  - Goal: Afferent circuit growth: one bounded Graph mutation can declare an input and connect it neutrally to an eligible consumer while preserving existing coefficients and ordinary separate growth paths.
- [ ] **T20.F05 — Structured Heritable Refinement** — Depends on: T20.F04
  - Goal: Repeated circuit motifs: inherited variation can change homologous relationships across directions or repeated entity slots while independent mutations preserve local exceptions.
- [ ] **T20.F06 — Applied Action Credit (Conditional)** — Depends on: T20.F13, T19.F05
  - Goal: Efference copies and eligibility traces: Graph input contributions can receive credit for actions the body actually performs and outcomes it experiences, without assuming a directional input layout.
- [ ] **T20.F07 — Independent Action-Conditioned Learning (Conditional)** — Depends on: T20.F06, T11.F07, T11.F09
  - Goal: Outcome-dependent synaptic adjustment: a creature can refine eligible Graph input contributions within its lifetime using experienced outcomes, with bounded acquired state and a heritable learning-off option.
- [ ] **T20.F08 — Shared Learning with Independent Exceptions (Conditional)** — Depends on: T20.F07
  - Goal: Generalization across repeated circuits: lifetime updates can transfer between semantically corresponding contributions with adjustable sharing and independently refinable exceptions.
- [ ] **T20.F09 — Inherited Discovery Qualification** — Depends on: T20.F05, T13.F07
  - Goal: Measure whether ordinary variation discovers useful Graph input contributions across different family shapes with lifetime learning disabled, separating access from coordinated refinement.
- [ ] **T20.F10 — Inherited Retention Qualification** — Depends on: T20.F09
  - Goal: Measure whether newly useful input contributions improve reproduction and remain useful across descendants without sacrificing established abilities.
- [ ] **T20.F11 — Inherited Ecological Transfer** — Depends on: T20.F10, T12.F04
  - Goal: Measure whether the qualified inherited Graph abilities persist in freely foraging, reproducing populations across the standard landscapes at native costs.
- [ ] **T20.F12 — Qualified Graph Availability** — Depends on: T20.F11
  - Goal: Adaptive circuit development: ordinarily born creatures can acquire the qualified inherited Graph mechanisms through evolution, retaining sparse, independently refined and pruned alternatives.
- [ ] **T20.F13 — Lifetime-Learning Need and Comparator** — Depends on: T20.F09
  - Goal: Determine whether an ecologically realizable within-life problem remains after inherited refinement, and whether a supplied learner offers enough added benefit to justify native learning work.
- [ ] **T20.F14 — Incremental Learning Qualification (Conditional)** — Depends on: T20.F08, T09.F03
  - Goal: Measure whether native lifetime learning adds useful input behavior and reproductive benefit beyond inherited competence under ordinary mutation, birth and ecological costs.
- [ ] **T20.F15 — Qualified Learning Availability (Conditional)** — Depends on: T20.F14
  - Goal: Experience-dependent adaptation: ordinary Graph lineages can acquire only the positively qualified learning mechanisms while retaining learning-off and independent alternatives.

## Notes for AI Agents

**Revision after adversarial review, 2026-09-23.** The purpose is general input
evolvability, with Graph qualification first and one later VM feature. Keep T20
as one track: general access is the core; topology-aware refinement and learning
have narrower claims and distinct gates. The original draft overgeneralized
ring experiments and coupled inherited qualification to learning. Pending IDs
are retained, with F03 moved out of the Graph chain, F09–F12 made inherited-only,
and F13–F15 added for the conditional learning decision and qualification.
No feature in this track has been executed.

**Evidence and alternatives.** The [ALife review](../strategy/alife-sensorimotor-learning-research-2026-09-23.md)
and four exploratory reports motivate candidates, not defaults:

| Evidence | Supported decision | Important limit |
| --- | --- | --- |
| [Wiring experiment](../strategy/ring-wiring-experiment-2026-09-23.md) | Coordinated changes reached all eight directions in 27/32 fresh searches versus 1/32 with matched additive scalar changes; inherited refinement is the first candidate. | Broader zero wiring alone showed little improvement. Restricted search does not establish native discovery or benefit for non-ring inputs. |
| [Learning experiment](../strategy/ring-learning-experiment-2026-09-23.md) | Coordinated inherited controllers survived stable trials in 83/128 episodes versus 1/128 scalar; preserve an inherited-only path and comparator. | Native hybrid reward learning reduced survival from 46/128 with learning off to 29/128 on; supplied action-conditioned learning is a different mechanism. |
| [Adaptation experiment](../strategy/ring-adaptation-experiment-2026-09-23.md) | Competent inherited starts mattered: 120/128 stable survivors with learning versus 19/128 weak starts. | Mid-life sensor remapping is a diagnostic perturbation, not a current ecological requirement. |
| [Group-learning experiments](../strategy/ring-group-experiments-2026-09-23.md) | Partial sharing tolerated exceptions better than full sharing; new-ring recruitment survived 88/128 episodes versus 1/128 independent. | The new-ring panel omitted coordinated inheritance, froze old weights and supplied an eating reflex. F13 must add that comparator before committing to native learning. |
| [Sensor audit](../strategy/post-t19-sensor-integration-audit-2026-09-23.md) and [mutation audit](../strategy/post-t19-mutation-audit-2026-09-23.md) | General declaration/connection costs, unexcited observation channels and structural-only census readings justify F01 and a generic recruitment experiment. | Sparse readers and poor stored steering are not causal proof of access failure, ecological indifference or a T19 regression. T11.F24/F25 repair confirmed mutation confounds first. |

The latest learning panels contain 16 source controllers with repeated episodes,
not thousands of independent lineages. Their injected feedback was **gross food
energy before the energy cap**, and full sharing tied updates, not absolute
inherited weights. Scripted food, recentering, supplied reflexes and credit limit
transfer. Arbitrary remapping remained mostly lethal and sufficiently expensive
learning erased the benefit. Neither 25% sharing nor 20% exploration is a default
or an acceptance threshold. The experiments do not establish a need for learning
over an adequate inherited controller in Petri's actual ecology.

Primary research supports comparing alternatives:

- [Clune et al. (2011)](https://jeffclune.com/publications/2011-CluneEtAl-IndirectEncodingAcrossRegularityContinuum-IEEE-TEC.pdf)
  and [Helms and Clune (2017), Offset-HybrID](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0174635)
  support inherited regularity plus independently evolved exceptions. Performance
  varies by problem; this motivates a small mutation extension, not importing
  HyperNEAT or assuming partial sharing requires lifetime learning.
- [Polyworld](https://shinyverse.org/larryy/Yaeger.ALife3.pdf),
  [EmEvo (2024)](https://arxiv.org/html/2406.15016v1), and
  [Arnold et al. (2024)](https://arxiv.org/abs/2404.12631) show viable combinations
  of evolution and learning under their own interfaces and objectives.
  [JaxLife (2024)](https://arxiv.org/html/2409.00853v1) supplies an inherited-controller
  alternative. The research review records recent follow-ups and access limits.
- [Evolved Hebbian rules (2020)](https://arxiv.org/abs/2007.02686) and
  [synaptic/structural plasticity (2024)](https://arxiv.org/abs/2406.09787) motivate
  conditional learner research, not arbitrary plain Hebbian updates or a ban on
  deletion.

Extend existing edges, input resolution, mutation and observation machinery.
Compare ordinary two-step sparse growth, one-event neutral recruitment, and
structured inherited changes with independent exceptions. Keep these as separate
arms. Universal dense wiring, a global sensor-addressing rewrite, permanent
weight tying and a replacement brain framework are not justified by this evidence.

**General access and semantic structure.** F01 covers the audited 19 named
non-upstream families plus `UpstreamSlot`; shared-memory sources are inventoried
separately because they need no input-reference declaration. F04 must address
every currently legal family/channel through the common contract; this does not
claim every input is useful in every world. Runtime/property fixtures cover legal
access broadly; empirical qualification uses at least two predeclared, differently
shaped families, including a non-ring family, selected by F01's opportunity check.

Homology follows meaning, never equal width. Ring directions, repeated nearby
creature slots, and the directional action kinds offer candidate repeated
structures. `NearbyCreatureVitals` has eight channels but is not a ring; its
values need Core position/presence for many useful behaviors. Area summaries mix
different quantities, and a scalar has no repeated layout. Do not force cyclic
sharing onto either. F05 must qualify two distinct repeated layouts before
claiming general coordination; otherwise report its supported subset while F04's
general access is assessed independently. No task answer selects a source, sink,
correspondence or mutation target in production.

Zero weight is not universally neutral: wiring `ClearSlot` clears memory, and
other write sinks can overwrite a carried value with zero. F02 must distinguish
additive contributions from presence-triggered/overwrite effects. Initially use
proven-neutral consumers such as action votes, across action kinds rather than
only Move; any additional sink needs a neutrality proof, not a width-based rule.
Signal neutrality at adequate budgets does not imply free storage, perception,
execution, reproduction or physiology.

**Ownership.** T20 owns general input recruitment, structured refinement and
the conditional learning extensions described here, within the existing runtime
and mutator. [T11](t11-brain-genotype-phenotype-map.md) retains general encoding,
supply, clocks and learned-state inheritance; F24/F25 own the prerequisite
target-identity and active-parameter repairs. T20.F03 owns the scoped VM fresh-read
and payload-draw corrections S1/M2 alongside its backend extension.
[T13](t13-neutral-module-recruitment.md) retains whole-module/contextual-copy work;
reuse its applied opportunity, ancestry and contribution observations.
[T09](t09-cognition-and-learning.md) retains general cognition and native-plasticity
qualification: its unstarted F03 is a prerequisite only for T20.F14's learning
comparison, not the inherited path. [T19](t19-mesh-action-selection-and-live-state.md)
continues to govern votes, revisits, live state and frozen external perception.
T18 founder redesign and unrelated audit findings are not silently absorbed.

**Feature handoffs.** The main path is F01 → F02 → F04 → F05 → F09 → F10 → F11
→ F12, entirely Graph and inherited. F03 may start after positive F09 discovery
and does not gate Graph rollout. F13 consumes valid F09 comparison evidence;
F06–F08 and F14–F15 stay conditional and outside the master priority order until
F13 justifies them. Each feature includes its necessary inspection, serialization
and applied accounting, without a separate observability framework.

| Feature | Bounded deliverable and verification boundary |
| --- | --- |
| F01 | Extend/version existing probes for family/channel declaration → connection → executed read → causal applied effect → retention, with nonzero barrier, extended and outcome scenes (S2/S3); retain structural census semantics. On 2–3 predeclared family shapes, compare competent authored use against founders and representative incumbents, plus matched ablations, in the standard worlds at native costs. Measure actual offspring and preserve competence; test exposure and controller adequacy before interpreting a null. Record Graph feasibility using ordinary edges, perception/carrying/work costs, the current two-step discovery baseline, and positive/negative/inconclusive opportunity by family. Preserve source identities; historical pre-repair results are not pooled into the baseline. Record VM cost concerns for F03 without making VM feasibility a gate. No new ecology or production mutation policy. |
| F02 | Establish the general Graph neutral-consumer contract with authored/property fixtures, stable references, copying, deletion, round trips and independent coefficient access. Use current Graph edges; representation changes are conditional on a demonstrated failure and require a prospective scope amendment. Exclude unsafe zero-write sinks unless the chosen semantics prove neutrality. Account for capacity/budget failure and extra work. This feature can close on validated existing behavior and fixtures; it does not require a new projection object or evaluator. |
| F04 | Add an explicit, bounded declare-and-connect mutation for any legal input family, using F02's eligible consumers. Preserve `InputRef.Add` as neutral declaration and the existing separate connection path; do not silently redefine either. Predeclare single-channel versus whole-family bundle arms and bounded destination selection, preserve present coefficients, and fail atomically. Existing shared-memory sources need only connection; new nodes remain subject to T13's routing contract and are not automatically wired to everything. Pruning persists until an explicit later recruitment event. Keep supply policy fixed and report actual target exposure, genomic growth and costs. |
| F05 | Add coordinated inherited steps over declared repeated meanings using ordinary coefficients, with independent residual refinement and deletion. Candidate cases include cyclic direction relationships and homologous fields across the four nearby-creature slots, without treating rank as direction. Compare scalar/additive versus coordinated steps under declared event and vector-step bounds, recording coefficients touched and costs. No new learner or generic pattern language. Verify applicable and inapplicable shapes; F09 decides the empirical scope that qualifies. |
| F09 | Use the native mutator and fresh independent lineages with learning off. Compare current two-step growth, one-event access alone, and access plus structured variation; keep matched additive controls and all source/sink targeting assumptions explicit. Starts may have incumbent competence but no authored new-input solution. Qualify added causal behavior, relevant channel coverage, mixed-scene usefulness and preserved competence on at least two family shapes, one non-ring. Report exposure, discovery time, costs and the backend carrying each contribution in mixed meshes. Give separate verdicts for access and coordinated variation, freeze supported candidates, and keep authored controls separate from discovery. |
| F10 | Test frozen inherited candidates in bounded finite populations with native births/mutation and actual reproductive outcomes. Use T13's ancestry/retention definitions and ancestral-payload/ablation controls across parent-to-child depth. Separate loss by deletion, extinction and inadequate exposure; do not count copied old computation as a new ability. Qualify retention separately from one successful lifetime. |
| F11 | Test qualified inherited Graph candidates with fresh seeds in Orchards, Canyon and Confluence, reusing T12 recipes. Remove recentering, scripted encounters, supplied reflexes and energy support. Observe reproduction, causal input use, incumbent behavior and realized costs; keep learning disabled for the causal inherited comparison. Distinguish landscape-specific benefit from broad transfer, and report backend composition. This is bounded engineering qualification, not a confirmatory cognition campaign. |
| F12 | Enable only positively qualified inherited Graph creation/refinement opportunities through ordinary mutation, with the same costs and declared scope. Check reachability from ordinary births and measure backend-mix shifts without changing node creation odds to equalize counts. Do not retrofit dense wiring or enable VM or learning by implication. Record default/epoch changes through the existing workflow. |
| F03 | Extend the frozen qualified Graph access/refinement contract to VM after F09. Correct all fresh-read generators/motifs to address supported references and compound channels, and payload draws to the actual 24-slot bus (S1/M2); declare pool limits explicitly. Reuse native instructions where feasible, with stable operands and fully charged execution, not an uncharged Graph evaluator. Test equivalent applied contributions at adequate budgets, actual exhaustion behavior and native evolutionary discovery, then reuse the bounded retention/ecological comparisons before VM availability. One naive dense translation used 201 instructions versus 33 for matched directions and crossed the step ramp; that is a warning, not a VM lower bound. VM lifetime learning is outside this feature. |
| F13 | Add coordinated inheritance with learning off to the new-input experiment, comparing against the strongest inherited candidate with matched starting competence, exposure and real costs. Separately identify a current ecological within-life need, such as changing resource value or social context only if the world actually supplies it. Test inherited, native-plasticity and supplied-learner controls where meaningful; report supplied credit/reflexes as limitations. Sensor-index rotation is a stress test, not the need. Return a positive need/feasibility verdict only for incremental benefit that survives a realistic opportunity/cost check; this does not implement a learner or qualify rollout. A negative result leaves the inherited path intact. |
| F06 | Conditional on positive F13. Define general input→executed-action eligibility, delayed outcomes, multiple contributors/actions, failures and trace expiry. Explicitly extend the existing Graph plasticity state/update lifecycle to the selected eligible effect contributions; do not add a parallel learner beside it. Current native reward learning covers compute-node inputs, effect edges use genome weights, and the outcome bank contains per-tick net energy change rather than action identity. Specify these boundary changes, bodily signals, saturation and T11.F09 inheritance before implementation. Add credit records without weight updates, with trace fixtures and priced storage/work. |
| F07 | Extend that lifecycle with the independent action-conditioned learner on Graph. Define error signal, normalization and bounds from native outcomes, not the prototype's gross-food/5 oracle. Test delayed credit and acquired versus inherited coefficients with live/frozen/reset controls, including T11 inheritance behavior. Learning rates include a mutable off state. Price actual trace/update work: both native learning cost defaults are currently zero, so merely saying “native costs” is insufficient; specify justified nonzero sensitivity cases and any proposed price before outcomes. No forced exploration percentage or subsidy. |
| F08 | Add bounded heritable sharing only over F05's justified correspondences, with independent/full/partial cases, local exceptions and no regrowth of pruned coefficients. Match the focal contribution's total update across sharing arms. Test wrong correspondence, asymmetric structure and irrelevant input controls; 25% is one experimental setting. Restrict claims to qualified layouts; scalar and heterogeneous channels do not acquire invented symmetry. |
| F14 | Separately qualify the incremental Graph learner against the frozen inherited candidate and T09.F03's native-plasticity comparator. Reuse the discovery, retention and ecological protocols with fresh lineages, reachable learning parameters, actual birth/reset/inheritance, paid credit/update work and matched initial competence. Supplied-learner success alone is insufficient. Report supported families, live/frozen/reset effects and a distinct positive/negative/inconclusive verdict; inherited production availability is unaffected. |
| F15 | Enable only the positively qualified Graph learning scope, with off and independent alternatives, verified ordinary-birth access, priced behavior and default/epoch records. Do not infer VM learning support from F03 or from Graph success. |

**Evidence gates and exits.** Prospective feature specs freeze family/scene
selection, source cohorts, independent units, seeds, horizons, cost treatment,
quantitative margins, uncertainty, compute/storage caps and stopping rules.
Use lineage-level uncertainty where appropriate, retain failures/censoring, and
hold out new lineages and scenes. F01's bounded ecology check is an opportunity
screen, not proof that a failed authored controller exhausts possible uses of a
sensor. If two different shapes cannot be qualified, record the limitation and
re-plan before claiming general input evolvability; do not invent new ecology
inside this track to produce a pass.

Required favorable verdicts are explicit: F01 opportunity/Graph feasibility for
F02–F05; F09 discovery for F10 and F03; F10 retention for F11; F11 transfer for
F12; F13 need/feasibility for F06–F08; F14 incremental qualification for F15.
F13 needs a valid inherited comparison from F09, not necessarily a favorable
discovery verdict. An access-only success can advance independently of a failed
coordination candidate when that contrast and its downstream scope were
predeclared; a learning failure never retracts a qualified inherited result.

The master selector only understands checkboxes and dependencies. Therefore a
negative or inconclusive required verdict must be recorded on affected rows as
**Blocked — [evidence link]**, with those IDs removed from the master priority
list in the same planning-state update. Keep their boxes unchecked; the failed
measurement itself can close honestly. F06–F08/F14/F15 begin unscheduled and are
added only after a positive F13 decision. This uses the existing roadmap and
workflow; it adds no alternate dispatcher or status machinery.

Each experimental candidate's spec includes its bounded disposal work. A final
negative verdict, or an inconclusive result at the predeclared stopping cap,
removes that candidate's unqualified production-path code/configuration as part
of the qualification feature's closure, unless the user explicitly chooses to
retain it. Preserve evidence, useful observation fixtures, independently
qualified candidates and correctness repairs. Do not leave abandoned opt-in
machinery indefinitely. Record a stopped/negative track or branch outcome in
the existing progress record and roadmap notes without marking biological
success criteria complete. A later revival needs a prospective scope and
priority decision, not rewritten historical evidence or relaxed thresholds.

Candidate Graph creation stays opt-in until F12, VM until F03's own positive
qualification, and new learning until F15. Disabled candidates consume no extra
mutation RNG draws or alter defaults. Enabled candidates pay their declared
costs; default zero prices must be disclosed rather than presented as measured
economic costs. Flat specs are written just in time, and all execution uses
[the existing workflow](../workflow.md) and [Codex substitutions](../workflow-codex.md).
Measurement features keep the same closure workflow; this review does not create
an exception.

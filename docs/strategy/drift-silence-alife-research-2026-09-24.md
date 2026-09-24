# Behavioral silence after drift: artificial-life research

**Status:** Research and proposed experiments, not executable roadmap guidance.
**Date:** 2026-09-24.
**Question:** What would distinguish useful robustness, inaccessible variation,
and limited observation when almost every mutation-bearing birth at drift depth
2,000 is silent on Petri's battery?

## Recommendation

Extend the existing neighborhood and drift probes for diagnosis before selecting
a production mechanism. A lower silent fraction is not itself a useful target:
we need to know which novel behaviors become accessible, at what mutational cost,
and whether useful changes survive inheritance and ecological selection.

The strongest near-term alternative is to proceed with the already planned
T20 input-recruitment experiments while treating deep-drift silence as an
unresolved diagnostic. That is reasonable if a quick census finds the problem
confined to unselected walks. A general mutation-policy or representation repair
should wait for a demonstrated bottleneck in selected genomes as well.

## Local context and limits

The [September 7 depth study](mesh-depth-research-2026-09-07.md) separated inactive
target dilution from silence within executed tissue on an older substrate. The
[September 14 supply study](per-unit-mutation-supply-research-2026-09-14.md) found
that per-unit supply alone could grow genomes explosively without selection;
its selected runs behaved differently. Neither finding automatically explains
the post-T19, post-T11.F24 observation.

The current [T11 roadmap](../roadmaps/t11-brain-genotype-phenotype-map.md) already
owns neutral-network characterization (F12), supply/targeting calibration (F13),
and memory discovery (F10). F25 already addresses mutation draws targeting action
parameters the body does not consume. [T13](../roadmaps/t13-neutral-module-recruitment.md)
owns module recruitment and retention; [T20](../roadmaps/t20-input-evolvability-and-structured-variation.md)
owns input recruitment, structured refinement, and its discovery/retention/transfer
qualification. New general diagnostics fit T11, reusing the existing observation
machinery. They should inform those owners rather than duplicate their mechanisms.

The depth walk retains descendants without ecological selection. It is not the
same experiment as moving only through phenotype-preserving mutations on a
neutral network, and its step count is not a count of applied mutation events.
Behavioral equality on a finite battery is also not ecological neutrality.

## Primary evidence

The search prioritized 2024–2026 research, with older digital-evolution and CGP
studies where they address the question directly. These are transferable
experimental ideas, not validation of a Petri change.

| Source and reading depth | Finding relevant to the decision | Transfer limit |
| --- | --- | --- |
| [Kumawat, Lalejini, Acosta and Zaman, *Evolution takes multiple paths to evolvability when facing environmental change*, PNAS, online December 2024 / 2025 issue](https://doi.org/10.1073/pnas.2413930121). Abstract, figures, and indexed full-text discussion inspected. | Avida populations under recurring environmental changes evolved neighborhoods richer in alternate useful phenotypes; mutation rate and neighborhood composition served distinct adaptive roles. Intermediate switching rates were especially effective in their treatments. | Fixed-length programs and explicit computational-task rewards differ from Petri ecology. Their roughly 30-generation switching interval is not a transferable parameter recommendation. |
| [Hardy, *How does robustness affect evolvability?*, Evolution, 2025](https://doi.org/10.1093/evolut/qpaf116). Author abstract inspected. | Formal models distinguish increasing mutational sensitivity from changing which phenotypes non-neutral mutations reach. Improving the latter can increase evolvability without the load associated with generalized sensitivity. | A theoretical result for discrete phenotypes; it supplies a distinction to measure, not a demonstrated Petri repair. |
| [Martin, Camargo and Louis, *Bias in the arrival of variation can dominate over natural selection in Richard Dawkins's biomorphs*, PLOS Computational Biology, 2024](https://doi.org/10.1371/journal.pcbi.1011893). Full primary article inspected, especially GP-map definitions and simulation design. | Frequently generated, moderately adaptive phenotypes can win over rarer, fitter alternatives. Genotype-level and neutral-set-level robustness/evolvability relationships differ. | A nine-parameter developmental morphology model, not recurrent cognition. It motivates measuring novelty identities and frequencies, not assuming that more changed births imply better search. |
| [Ferguson and Lalejini, *Identifying potentiating events in evolutionary search using replay experiments*, August 2026 preprint](https://arxiv.org/abs/2608.09833). Full HTML methods and limitations inspected; peer-review status not established. | Replicated restarts from historical states estimate changes in the probability of a named future outcome. Present fitness and future potential need not rise together. Coarse-to-fine replay can localize informative history; incomplete population samples can bias ecological interpretation. | Full replays can be expensive and retrospective. A small genome-only pilot estimates conditional controller potential, not whole-population ecological evolvability. |
| [Cui, Margraf and Hähner, *Analysing the Influence of Reorder Strategies for Cartesian Genetic Programming*, 2025](https://doi.org/10.1007/s42979-025-04296-4). Primary article's introduction, representation, and results summary inspected. | Representation-induced positional bias can make much structure rarely contribute; semantics-preserving reordering improved their benchmarks, with no universally best operator. | Petri's routed, recurrent mesh is not standard feed-forward CGP. This is evidence to inspect opportunity distributions, not to install a reorder operator. |
| [Cui, Margraf and Hähner, *Refining Mutation Variants in Cartesian Genetic Programming*, 2022](https://doi.org/10.1007/978-3-031-21094-5_14). [Author institution's abstract](https://publica.fraunhofer.de/entities/publication/27e45619-0c47-40f2-aeb0-4d7a0f377161) inspected via search; publisher full text not read. | Separate active/inactive rates and mutations conditioned on hitting active nodes improved tested supervised benchmarks, with extra mutation work and runtime. | Active graph membership is not causal contribution or ecological usefulness. Retry-until-active changes exposure; retry-until-behavior-changes would be an observer-driven intervention. Compare costs and dormant preparation before considering policy changes. |
| [Elena and Sanjuán, *The effect of genetic robustness on evolvability in digital organisms*, 2008](https://doi.org/10.1186/1471-2148-8-284). Primary article abstract and experiment description inspected. | In Avida, robust genotypes could adapt better over longer horizons while doing worse initially; results in more complex environments were less conclusive. | Does not establish that Petri's silent genomes are beneficially robust. It argues against interpreting a one-step silent fraction as a complete evolvability measure. |
| [*Meta-Learning an Evolvable Developmental Encoding*, 2024](https://arxiv.org/abs/2406.09020). Primary HTML introduction and method inspected. | A learned genotype-to-phenotype mapping made downstream search yield more diverse, high-quality maze artifacts. Representation can change search accessibility independently of raw expressivity. | Uses an outer optimization loop and quality-diversity objectives. Importing that machinery would alter Petri's ecological premise and is disproportionate before diagnosis. |

## Competing explanations

These are hypotheses to distinguish, not findings about today's genomes.

| Explanation | Discriminating observation | Likely owner if confirmed |
| --- | --- | --- |
| Inactive target or invalid/unused payload | Applied edits rarely reach executed, consumed computation; changes increase when exposure is conditioned on eligible tissue. | T11 general targeting; existing F25 for unused action parameters; T13 applicability for modules. |
| Expression or threshold masking | Edits change internal values, routing bids, or vote margins but rarely alter decoded action or parameters. | T11 representation or T20 refinement, depending on affected surface. |
| Observation blindness | The same mutant differs on longer histories, nonzero memory, broader supported inputs, or recorded ecological observations. | Existing neighborhood observation machinery; coordinate with T20.F01 opportunity readings. |
| Useful cryptic variation | Battery-silent descendants reach useful alternatives more often than their parents after additional mutations. | T11.F12, T13/T20 retention qualification. |
| Unselected-walk degeneration | The severe silence is specific to walked genomes; selected genomes of comparable size/depth preserve useful variation. | Keep drift diagnostic; selected-depth characterization in T11.F13 rather than a silence repair. |

Internal differences alone are not useful behavior. Equal actions alone do not
prove equal costs, future state, survival, or reproduction. Attribution needs an
explicit unclassified category when available traces cannot establish a cause.

## Four bounded exploratory probes

Use a single current revision, explicit recipes and seed banks, and freeze the
parent panel before comparing treatments. Save parent/child genomes and enough
mutation provenance to replay interesting cases. Keep all-birth, event-bearing,
single-applied-event, and per-operator denominators separate. Sampled siblings
are clustered by parent, so uncertainty should include independent parents or
lineages rather than treating every birth as an independent evolutionary replicate.
The limits below are proposed work caps, not estimates of measured runtime.

1. **Where does the effect disappear?** Start with 12 parents: four founder or
   shallow, four deep-walk, four available selected genomes. Produce 128 fresh
   births each under the unchanged engine, recording applied edits and existing
   execution/contribution observations. Classify exposure to executed tissue,
   state/value changes where observable, and final action differences. Repeat
   a smaller one-applied-event diagnostic by operator to avoid cancellation and
   multi-event attribution ambiguity. Report conditioning explicitly; do not
   present forced-event samples as production births. Time a two-parent pilot
   before expanding. This first separates targeting from downstream masking.

2. **Does the battery miss the effect?** Replay up to 200 sampled silent
   parent-child pairs on the original battery and a separate diagnostic extension:
   longer lawful input sequences, memory warmup, and supported contexts absent
   from the original sample. Keep costs and state trajectories alongside actions.
   Preserve the original metric and label the extension separately. If many
   pairs cease to be silent, improve the observation contract before changing
   mutation policy. Authored positive controls are needed to verify the extension
   can expose known temporal and directional effects.

3. **Does neutrality prepare a useful next step?** On eight parents, retain a
   small fixed sample of battery-silent one-event children. Give each parent and
   its children an equal second-step mutation budget (for example 64 attempts).
   Measure distinct behavior signatures and success on one predeclared existing
   input or temporal task, with incumbent behavior checked separately. A
   consistent advantage after a silent first step is evidence for accessible
   cryptic variation; a null is only a bound at this path length and sample size.
   Reuse recruitment-path machinery. Do not mistake a two-step diagnostic for
   proof of retained adaptation.

4. **Tiny continuation pilot, only after a discriminating result.** Select one
   candidate change or historical parent-child contrast from the preceding
   probes and compare equal-budget replicated continuations using an existing
   task harness. Start with eight seeds per arm and a fixed evaluation cap; read
   discovery, damage, retained incumbent function, and useful descendants.
   Pilot wall time first. Use existing ecological replays later if task results
   warrant them; controller-only continuations cannot establish population-level
   transfer. This applies replay methodology without immediately commissioning
   a long ecology sweep.

Do not expand all four automatically. Stop after the first useful causal
separation, record unresolved alternatives, and choose the next probe from it.
A short pilot can reject a promising-sounding repair or reveal a measurement
gap; it cannot certify open-ended cognition.

## Roadmap decision after the probes

Prefer one bounded general **mutation-effect attribution and coverage** addition
in T11 if existing instruments cannot supply probes 1–2. It should precede any
new repair and feed T20.F01, not absorb T20's input work. Expand or sequence
T11.F12 for multistep neutral potential rather than create a duplicate
neutral-network track. Keep mutation-policy comparisons in T11.F13. If the
diagnosis finds only an already scheduled input-recruitment gap, no additional
mechanism feature is needed: carry the finding into T20's existing comparisons.

No production default, roadmap priority, or feature contract is changed by this
note. The alternative of increasing sensitivity globally remains unattractive
until useful novelty and retained function improve together; the strongest
counterargument is that inactive variation may be preparatory and current
silence may be specific to the instrument. The probes are designed to test both.

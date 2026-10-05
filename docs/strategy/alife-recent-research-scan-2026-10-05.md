# Recent artificial-life research applicable to Petri (2025–2026 scan)

**Status:** Research note, not executable roadmap guidance. Nothing here changes
production, a default, a founder or a running exploration.
**Date:** 2026-10-05.
**Review:** reviewed by a Fable advisor before and after writing; not reviewed by
Codex.
**Question:** What is the most recent groundbreaking artificial-life research that
applies to Petri's standing problem: under Petri's own variation, new nodes and
new sensor reads almost never become load-bearing, and one-step helpful supply
does not convert into reach ([runs 1–5](evolvability-exploration-2026-10-run5.md),
[run 6 plan](evolvability-exploration-plan-2026-10-04-run6.md))?

## Answer in brief

No 2026 result breaks Petri's problem. The field's 2025–2026 centre of gravity
sits outside Petri's premise: foundation-model-guided search for interesting
simulations, LLM agents as artificial life, neural cellular automata and
Lenia-class substrates, and GPU ecologies of fixed-architecture networks.

The thread that does apply is older than 2026 and already in Petri's citations,
but no exploration run has tested it: **environmental fluctuation, not operator
design, is what selects for evolvable genotype–phenotype maps.** Kumawat,
Lalejini, Acosta and Zaman (PNAS 2025, Avida) and Ikeda, Kaneko and Hatakeyama
(2026, exhaustive genotype–phenotype map) reach that conclusion from opposite
ends: a cyclic environment switching about every 30 generations evolved
neighborhoods with a median of 45,802 alternate-phenotype mutants against 1 in a
constant environment, and frequent switching moves evolved genotypes from the
robust interior of a fitness region to its boundary, where alternative regions
are one or a few mutations away. Petri's goal worlds are static, and both lab
instruments are stationary: every generation draws fresh scenes, but always
from one fixed layout distribution, so no regime ever changes. T02.F01 Seasons
is unbuilt. Runs 1–6 varied operators, rates, jump
sizes and founders on fixed scene banks and returned nulls or small effects.

Three 2026 results support that reading or sharpen how to test it:

1. Mertan and Cheney (*Artificial Life*, online September 2026): selection
   consistently undervalues structurally novel offspring because their
   controllers are tuned to the parent structure; incumbents gain a first-mover
   advantage; three protection schemes failed to fix it. This is run 5's pattern
   (fired jumps helpful 1.6 %, harmful 20–22 %) stated as a general property of
   co-optimization, and it argues against importing protection mechanisms.
2. Walsh (ALIFE 2026, Quandary Den): duplicated elements become irremovable by
   masking accumulated interference, not by acquiring a function. This offers a
   second route by which Petri's silent tissue could become load-bearing, and
   T22.F03's ablation sentinels can read it without a new arm.
3. Bohm and Hintze (ALIFE 2026): computational universality is a poor predictor
   of evolvability; simple Class II rules evolve better than Rule 110. Abstract
   only (see read depth). It supports choosing substrate primitives for what
   evolution reliably discovers, which is the T19 vote-surface direction.

**Recommendation: more exploration**, one new instrument (details below), not a
roadmap feature, and nothing that touches run 6 while it runs.

## Method and read depth

Search on 2026-10-05: nine web searches, three arXiv export-API listings
(`"artificial life" AND evolvability`; `open-ended evolution OR digital
evolution OR artificial life`; `neuroevolution AND agents|ecology|foraging|
evolvability`, newest first), the ALIFE 2026 program (Waterloo, 17–21 August
2026; PDF read in full for titles), the *Artificial Life* journal online-early
list (browser), and the citation index of this repository to exclude papers the
docs already hold. Candidates were kept when they concern evolvability,
genotype–phenotype structure, neutral or silent variation, energy-coupled
computation or evolving agents in an ecology.

Read depth is marked per row. *Full* means the arXiv HTML or the open-access
page was read end to end by a reader whose report is quoted here; *abstract*
means the abstract and metadata only. Kumawat et al. was read in full by a
delegated reader from the bioRxiv v2 text, and its numbers below come from that
report. Bohm and Hintze's PDF sits behind a Cloudflare challenge on
`direct.mit.edu` and its replication repository was empty on the search date,
so it stays at abstract depth.

## Local context

- Runs 1–5 ([run 5 results](evolvability-exploration-2026-10-run5.md)): mutation
  arms M1–M5, A/A controls and the heavy-tailed weight jump J produced no reach
  gain over an A/A control at 100 generations; M4 roughly triples confirmed
  helpful one-step children without campaign gain; J births are harmful about
  twice as often as native twins on food. Run 6 tests 1,000-generation horizons,
  an exuberant silent-afferent start and a production check. All of these hold
  the environment fixed.
- Both instruments (`food-seeking-hunger` V2, `wall-v1`) draw each generation's
  training scenes from the scenes stream of one calibrated `SceneSpec`
  (`campaign.rs`, the per-generation draw), so samples vary but the layout
  distribution never does; `crates/v3-lab` has no schedule that changes the
  distribution across generations. `crates/v3-core` has no seasonal or fluctuating
  resource driver; [T02.F01 Seasons](../roadmaps/t02-environmental-dynamics.md)
  and T02.F03 regional offsets are unchecked.
- Kumawat et al. is cited five times (T11.F04 spec, population-decline note,
  incremental-recruitment note, drift-silence note, depth note), each at
  abstract or summary depth, as an argument about mutation rate. Its central
  finding, that the environment regime shapes the neighborhood, has not been
  carried into any experiment.
- A sibling note written the same day,
  [Why complex behavior is not evolving](evolvability-diagnosis-2026-10-05.md),
  reaches two of the same conclusions from the digital-evolution classics: the
  32-individual lab cannot carry a verdict on the production world, and the
  first step toward a function must pay in the world itself (its E7 names
  "regrowth that moves" as one world-side candidate). This note adds the
  recent fluctuation literature behind that candidate and a concrete lab
  instrument for it; the two notes should be read together and not merged
  into one roadmap row before either experiment has run.
- The lab runs populations of 32 over 8 replicates. The already-cited
  [large-scale ecology paper](https://arxiv.org/abs/2510.18221) reports new
  foraging and predation strategies only at 60,000 agents and 2-million-step
  horizons; arrival-of-the-frequent dynamics at quick sizes are a standing
  limit on what any lab arm can show.

## Primary evidence

| Source and read depth | Finding | Petri reading |
| --- | --- | --- |
| [Kumawat, Lalejini, Acosta, Zaman, *Evolution takes multiple paths to evolvability when facing environmental change*, PNAS 122(1) 2025](https://doi.org/10.1073/pnas.2413930121); full text (bioRxiv v2, delegated reader) | Avida, 22,500 organisms, 100-instruction genomes, six logic tasks in two mirror environments, 300,000 updates, 20 replicates. Cyclic switching every ~30 generations gave a median of 45,802.5 alternate-phenotype mutants among all single and double mutants against 1.0 for the constant environment and 4,556 for switching every ~300 generations; random switching gave 16. Evolved mutation rates: 9.19e-4 under Cyclic against 7.37e-6 constant. Neighborhood skew helped re-adaptation to seen environments; elevated rate helped adaptation to 127 novel tasks (42.3 against 30 for the ancestor), and transplanting rates reproduced the difference. Mechanism: populations "localize on phenotypic boundaries"; it "requires predictability" and an intermediate switching rate; "intermediate-rate environmental fluctuations are able to overcome the otherwise overwhelming selection for mutational robustness." | The only lever in the recent literature that a Petri experiment has not yet pulled. Avida's task rewards are not Petri's ecology, and "boundaries ... are central to the evolvability that we study, but their presence in every genotype-phenotype map is not guaranteed." A predictable A↔B food regime is the smallest Petri analog. |
| [Ikeda, Kaneko, Hatakeyama, 2026](https://arxiv.org/abs/2608.24704); full | Exhaustive Ising-spin genotype–phenotype map. Under fixed environment and phenotypic noise, ~90 % of trajectories end at high-penetrance "core" genotypes when switching every 100 generations, ~30 % when switching every 10; "Phenotypic noise ... breaks effective neutrality, and selection favors high-penetrance core genotypes over peripheral genotypes that provide greater mutational access to alternative adaptive regions." | Same conclusion as Kumawat from theory: static worlds select for robustness and interior positions; frequent change selects for accessibility. Petri's silent fraction and low reach are the interior signature. Already cited in the mesh note; now read in full. |
| [Mertan, Cheney, *Evolutionary Brain-Body Co-Optimization Consistently Fails to Select for Morphological Potential*, Artificial Life 2026 (arXiv 2508.17464)](https://arxiv.org/abs/2508.17464); full | 1,305,840 morphologies, controllers trained per morphology. Body mutations drop fitness far below the true inter-morphology difference; algorithms "consistently eliminate offspring that would have been selected if their true fitness were known"; 38 of 100 MAP-Elites runs stop at a point that "is not even a local maximum"; morphological innovation protection, MAP-Elites and AFPO all fail to fix it; "the ranking among high-fitness individuals in early generations does not correlate with their final ranking." | A structural mutation in Petri (new node, new read) is a body mutation whose weights were tuned for the old structure. Run 5's harm ratios are this effect. Protection schemes failed in their hands too, consistent with the Genesis reading. The measurable quantity is potential after k further steps, which replay (Ferguson and Lalejini 2026, already cited) estimates. |
| [Walsh, *Neutrally Evolving Interlocking Complexity in the Quandary Den*, ALIFE 2026](https://arxiv.org/abs/2604.18361); full | Teams of gene-encoded characters; offspring kept when score is at least the parent's; gene add/remove at 1 % per birth. With duplication, shared start and friendly fire, added genes become irremovable: "It becomes nearly impossible to remove an added gene because removing it will unmask those attacks." The ratchet needs duplication (not de novo genes) and negative selection against loss; robustness declines linearly per added gene. | Constructive neutral evolution as a second recruitment route: a copy or paralog becomes essential by cancelling interference, with no gain. Reading, not arm: on T22.F03 ablation sentinels, track whether removing each silent node turns deleterious over generations. Petri already has the copy and paralog operators the ratchet needs. |
| [Claret, Cotofrei, O'Neill, Stoffel, *Multi-Behavioral Evolved Substrates*, ALIFE 2026](https://arxiv.org/abs/2610.00148); full | Same 2→10→1 architecture: (μ+λ)-ES reaches 76.7 % five-task success with XOR pinned at 75 %, Adam reaches 100 %; "The activation barrier is an evolutionary search barrier, not a representational limitation." Neuromodulation plus per-task activation selection reaches 100 % in a median of 14 generations; neither alone converges. | The oracle-optimizer test separates search barrier from representation. Petri's calibrated authored comparators already play that role; the result says a reach null is a search statement, not an expressivity one. Boolean toys; no transfer of mechanism. |
| [Saad Saoud, *The Cost of Becoming*, 2026](https://arxiv.org/abs/2609.17606); full | Direct, static generative and zygotic developmental encodings, 30 paired runs: no fitness difference (p = 0.213); developmental encoding raises mutant-reachable diversity (+0.502, p = 5.6e-9) and lowers mutation viability (0.906 → 0.648), locality (0.574 → 0.394) and crossover viability. | A caution for T18 founder architecture and for run 6's ALT start: more reachable phenotypes arrive with lower locality and viability, and no reach gain by themselves. Report both sides on any encoding change. |
| [Chaturvedi, El-Gazzar, van Gerven, *Role Differentiation in a Coupled Resource Ecology under Multi-Level Selection*, ALIFE 2026](https://arxiv.org/abs/2604.00810); full | Boids with CTRNN controllers, energy depot, minimal-criteria birth and death; group-level CMA-ES evolves a shared substrate and a context-dependent mutation operator (MLP fed by activity and resource averages). Ablation: "most of the baseline performance is carried by the shared controller substrate, while the mutation operator provides a smaller but useful improvement." | An outer optimizer, so off-premise for production. The activity-conditioned mutation idea is Petri's executed-set targeting; their ablation puts the operator's contribution second to the substrate, matching runs 4–5. |
| [Jha et al., *Tapes Together Strong*, 2026](https://arxiv.org/abs/2609.10817); full; [Cicala et al., *Coevolution of self-replication and function in a digital primordial soup*, 2026](https://arxiv.org/abs/2607.09211); abstract | Z80 soups where every instruction costs one energy unit and execution probability equals energy share; stealing is suppressed because a defector "starves the shared energy pool before it can execute the L writes necessary to replicate"; scarcity raises task solving; spatial structure gives higher complexity and higher joint solve rates. | External confirmation of the T03.F10 direction (compute cost as a hard constraint) and of the hunger-regime instrument. No new mechanism for recruitment. |
| [Bohm, Hintze, *The Surprising Evolvability of Underappreciated Cellular Automata*, ALIFE 2026](https://doi.org/10.1162/ISAL.a.950); abstract | Fixed elementary CA rule as the brain's primitive, genomes place input writes and output reads; "Rule 110 is not optimal on any task and is far from optimal on two of the three, while several of the strongest performers come from Class II"; "what matters is not only what a substrate can compute in principle, but what evolution can reliably discover and exploit under constrained search." | Supports T19's move toward a plainer action surface; no mechanism until the full text is readable. |
| [Ge, Cheng, *Neuroevolution Arena*, 2026](https://arxiv.org/abs/2608.10323); full | 100×100 torus, up to 10,000 cells with independent MLPs, energy and hunger, 50,000 generations; EvoEvo, EvoRL and RLRL regimes. RL regimes double training fitness, yet pure evolution wins most frozen pairwise blocks on the small architecture; only 4 of 15 pairs keep direction across blocks; survival endpoint floored at 0. Authors list implementation defects. | Closest 2026 regime to Petri's, and the lesson is methodological: ranking under training and under ecological evaluation diverge, and three runs per condition do not settle it. Nothing to adopt. |
| [Charterton, Borg, Ekart, 2026](https://arxiv.org/abs/2609.35018); abstract | NEAT foragers with plastic networks: simple nearest-food topologies met the fitness goal so social information never evolved; "a greater degree of complexity in the environment" is needed. | The sterility-shortcut lesson in another system: an instrument that is solvable by a trivial rule selects that rule. |
| [de Bruin, Glette, Ellefsen, *Lamarckian Inheritance in Dynamic Environments*, 2026](https://arxiv.org/abs/2605.15769); abstract | "Lamarckian inheritance only underperforms Darwinian inheritance when the changes are both conflicting and unpredictable"; sensors restore the advantage. | Predictable change is again the condition under which inheritance of acquired structure pays; relevant to T09 if lifetime learning is ever inherited. |

Also seen, set aside with the reason: [ASAL](https://arxiv.org/abs/2412.17799)
and [VLM-guided evolution](https://arxiv.org/abs/2509.22447) (outer
foundation-model objective); Darwin Gödel Machine and
[OpenLife](https://arxiv.org/abs/2606.31046) (LLM agents);
[PBT-NCA](https://arxiv.org/abs/2604.11248),
[MSPD](https://arxiv.org/abs/2606.17091) and
[Hash Chemistry](https://arxiv.org/abs/2607.28219) (cellular or hash substrates
without agents); [Microcosmos](https://arxiv.org/abs/2607.02954) (engine);
[Self-Modifying LGP](https://doi.org/10.1162/ISAL.a.1044) (self-replication and
regression tasks); [genomic-bottleneck reservoirs](https://arxiv.org/abs/2606.28380)
(meta-learned hypernetwork); [Baldwin regimes in chess](https://arxiv.org/abs/2604.03565)
(self-play); [Escondo and Ferguson 2026](https://arxiv.org/abs/2608.09833) on
replay sampling (seen only through its citation in the already-cited replay
paper: full population snapshots are recommended over elite sampling, which
matters if the lab ever runs replays from stored elites).

## Transfer gap, stated

Kumawat's evolvability is a richer neighborhood of *alternate task phenotypes*
under switching task rewards in fixed-length programs. Petri's problem is a new
*sensor read or node* becoming load-bearing in a recurrent mesh under ecological
selection. The bridge is a hypothesis: a food rule that is periodically wrong
creates selection for genotypes sitting one step from the alternative rule, and
that neighborhood is where a second read pays. Ikeda et al. say the same thing
geometrically and warn that phenotypic noise pushes the other way, so the
instrument must hold Petri's noise sources fixed. Nothing in either paper shows
that boundary positions exist in Petri's genotype–phenotype map or that they are
reachable from the founder. That is exactly what the experiment measures.

## The experiment this recommends (run 7 candidate, not run 6)

Under the [T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
as an `instr:` commit on an exploration branch, after run 6 closes:

1. **Instrument.** `food-seeking-hunger` with two frozen food layouts A and B
   that are mirror images in the scene frame, switching A↔B every k
   generations with Gaussian jitter, as Kumawat did. A random-layout arm is the
   predictability control (their Random regime gave 16 alternate mutants against
   45,802): it redraws the layout distribution itself at each switch. The
   stationary V2 instrument is the reference, which is Kumawat's constant
   environment, not his Random one, because its per-generation scene draws
   come from one unchanging distribution. Calibration, comparator and the
   sterility repair stay as in V2.
2. **k grid.** Kumawat's optimum was about 30 Avida generations with 300 and 3
   on either side; Petri generations are not Avida generations, so k ∈ {3, 30,
   300} is a starting grid, predeclared, with the pilot choosing one for the
   main batches.
3. **Readings.** Reach on both layouts after a switch (lag, in Kumawat's sense);
   the share of one-step children that are helpful on the *other* layout
   (the alternate-neighborhood analog, which run 5's screen already computes
   per bank); final-best gaps; genome size.
4. **Not-forcing.** The rule names no sensor, sink or assay; both layouts are
   admissible food scenes; no founder or authored wiring changes.
5. **Outcome routing.** A positive result is a lead for
   [T02.F01 Seasons](../roadmaps/t02-environmental-dynamics.md) priority and
   for the T22 instrument set, never a production default; a supported negative
   at the predeclared effect size closes the environmental-fluctuation
   hypothesis for this instrument class. A positive result is evidence that the
   lever exists, not evidence for the driver's shape: the instrument's hard
   A↔B switch is Kumawat's design, while T02.F01 keeps its smooth,
   year-varying contract, and nobody should build a stepped season from a
   positive run 7.
6. **Companion reading, no arm.** On the existing T22.F03 ablation sentinels,
   record per generation whether removing each silent node is neutral or
   deleterious, to see whether Walsh's masking ratchet operates in Petri at all.

## Options considered

| Option | Verdict |
| --- | --- |
| Adopt a foundation-model or novelty objective (ASAL, VLM, QD) | Rejected: outer objective, violates the no-observer-selection premise already recorded in the mesh note. |
| Import protection for novel structure (speciation, innovation protection) | Rejected: Mertan and Cheney show all three tested schemes fail; Genesis needed a novelty archive plus speciation, both outside Petri's constraints. |
| More operator or founder arms on static instruments | Not before the environment arm: five runs of operator arms returned nulls, and two independent literatures say the environment regime is the lever. |
| Fluctuating-environment instrument on the lab (above) | **Recommended.** Cheap, inside the exploration contract, predeclared controls, routes to an unbuilt roadmap feature. |
| Build T02.F01 Seasons directly | Not yet: a roadmap feature costs a full workflow; the lab instrument decides whether the lever moves reach in Petri first. |

## Remaining uncertainty

- Whether Petri's map has boundary genotypes reachable from the founder is
  unknown; a null on the k grid would say the lever does not exist at this
  scale, not that fluctuation is useless.
- Lab population 32 is far below both Avida's 22,500 and the ecology paper's
  60,000; arrival-of-the-frequent dynamics may hide the effect regardless of k.
- Bohm and Hintze remains abstract-only; nothing here depends on its details.

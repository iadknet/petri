# T08 — Heredity and Recombination

**Status**: Planned
**Last updated**: 2026-09-05
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Ensure heritable change can be attributed along lineages, that lineages
inherit how much they mutate without collapsing supply, and that useful
function can eventually be exchanged through compatibility-aware mating and
homologous recombination, with asexual reproduction as the control.

## Track Success Criteria

- [ ] Along surviving lineages, heritable changes that altered behavior are distinguished from silent ones.
- [ ] Heritable mutation policies do not collapse mutation toward zero or remove access to adaptive novelty.
- [ ] The opt-in sexual mode stays disabled until its complete dual-parent and homologous-recombination contract is available, then exchanges homologous function without destructive arbitrary topology splicing.
- [ ] Controlled campaigns compare asexual and sexual treatments for adaptation, modularity, robustness, and diversity.

## Executable Features

- [ ] **T08.F02 — Lineage Mutation-Effect Attribution** — Depends on: T11.F01
  - Goal: Which mutations mattered. Along a surviving lineage, which heritable changes altered behavior and which were silent.
- [ ] **T08.F05 — Heritable Mutation Policy** — Depends on: T08.F02, T11.F12, T11.F13
  - Goal: Mutation rate evolves. Lineages inherit how much they mutate within bounds informed by rate characterization, preserving mutation supply and measured against fixed-rate controls.
- [ ] **T08.F06 — Homology and Mating Compatibility** — Depends on: T04.F05, T11.F08, T08.F05, T09.F08
  - Goal: Deferred until the T11 foundation and T08 heredity evidence are reviewed. Which genomes are similar enough to exchange parts, measured before any mating exists.
- [ ] **T08.F07 — Dormant Dual-Parent Mating Contract** — Depends on: T03.F07, T08.F06
  - Goal: Deferred heredity work. Two-parent reproduction semantics, present but disabled by default.
- [ ] **T08.F08 — Homologous Recombination and Sexual-Mode Enablement** — Depends on: T08.F07
  - Goal: Deferred heredity work. Turn on sex only after recombination provably exchanges homologous function.
- [ ] **T08.F09 — Heredity-Mode Comparison Confirmatory Campaign** — Depends on: T01.F09, T04.F07, T08.F05, T08.F08, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs comparing asexual and sexual populations.

## Notes for AI Agents

- Restructured 2026-09-04 after the [brain evolvability audit](../strategy/brain-evolvability-audit-2026-09-04.md): the mutation map moved to T11. T08.F01 became T11.F01, T08.F03 became T11.F12, T08.F04 became T11.F08, T08.F10 became T11.F02, T08.F11 became T11.F03, and T08.F12 became T11.F09. Those IDs are retired here and never reused. T08 adds no mutation operators of its own and builds on the corrected T11 map; the track was renamed from Evolvability and Heredity to match.
- T08.F02 attributes behavioral change along surviving lineages using the T11.F01 battery and its silent, changed, and dead classes, so that lineage readings and neighborhood readings share one definition of "changed".
- T08.F05 begins with an inherited scalar mutation rate, with bounds informed and justified by T11.F13 and fixed-rate controls from that characterization (user decision, 2026-09-05). T11.F04's approximately 0.55 requested events per birth is a provisional default, not a minimum or a claimed optimum. Resolve insufficient evidence for bounds explicitly before enabling inherited policy; never invent a floor from the existing default. [Clune et al. (2008)](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1000187) found that evolved rates could fall below the long-term adaptation optimum, while [Kumawat et al. (2025)](https://pubmed.ncbi.nlm.nih.gov/39739809/) found that environmental change could support elevated rates and useful alternate phenotypes. Evolved rates therefore need measured discovery/retention and supply outcomes, not an assumption of self-optimization. Lifetime responses to stress are a separate, unscheduled policy, and T08.F05 adds no new mutation operators.
- Sexual reproduction is committed as an opt-in experimental treatment, not as the production default and not as an assumed improvement. Asexual reproduction remains the control, and T08.F09 may truthfully find no benefit.
- T08.F06 through T08.F09 remain later work: do not plan their specs until T09.F08's diagnostic outcomes have been reviewed and T11.F01, T11.F08, T11.F12, T08.F02, and T08.F05 have produced the required evidence. A positive shared-memory sensitivity reading alone is not clearance for mating work.
- T08.F07 may add deterministic mate intent, pairing, contention, contribution, target, and draft semantics, but the creature-facing sexual mode remains unavailable by default. T08.F08 enables it only after recombination, viability, and inheritance tests pass.
- T08.F07 and T08.F08 explicitly supersede the one-parent portions of `docs/reference/v3-reproduction-spec.md`, `docs/reference/v3-creature-identity-spec.md`, `crates/v3-core/src/contracts/actions.rs` (`WorldAction::Reproduce { direction, energy_transfer }`), and `crates/v3-core/src/simulation/actions/reproduction.rs` (`apply_reproduce`). Their specs must define two-parent energy and reserve contributions, spawn targeting, turn-order contention, generation and ancestry, lineage and kin identity, shared memory, phenotype, graph state, learned weights, mutation ordering, and rejection telemetry while preserving the asexual path.
- Homology for recombination needs the structural identity that T11.F08 and T11.F09 define; positional array indices alone are not homology. If T11.F11 introduces labels, they are the natural homology markers for T08.F06.
- Research basis reviewed 2026-09-02: [The evolutionary origin of complex features](https://doi.org/10.1038/nature01568), [Sexual reproduction reshapes the genetic architecture of digital organisms](https://doi.org/10.1098/rspb.2005.3338), and [Homology and linkage in crossover for variable-length genomes](https://doi.org/10.1371/journal.pone.0209712).

| Option | Petri fit and evidence | Decision / exit criterion |
| --- | --- | --- |
| Mutation tuning without sex | Lowest contract risk and remains the required control, but cannot test exchange between lineages. | Retain as control, not as a substitute for the opt-in treatment. |
| Relative-position or arbitrary topology crossover | Demonstrated in simpler linear digital genomes, but Petri has variable nested VM, graph, and mesh structures. | Reject as the default because it does not establish functional homology. |
| Compatibility plus homology-aware module exchange | Matches Petri's topology and lets exchange quality be measured before evolutionary claims. | Adopt provisionally. T08.F06 must compare candidate mappings on homology, linkage, child viability, function retention, exchange rate, and cost; if none beats asexual mutation under the predeclared safety floor, block enablement rather than splice arbitrarily. |

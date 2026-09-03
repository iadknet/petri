# T08 — Evolvability and Heredity

**Status**: Planned
**Last updated**: 2026-09-03
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Ensure useful behavior can be reached through viable heritable steps, retained
under mutation, expanded through modular duplication, and eventually exchanged
through compatibility-aware mating and recombination.

## Track Success Criteria

- [ ] Mutational-neighborhood assays quantify viability, behavioral novelty, robustness, and access to future phenotypes around representative genomes.
- [ ] Duplication and structural mutation preserve useful function often enough to provide measured evolutionary stepping stones.
- [ ] Heritable mutation policies do not collapse mutation toward zero or remove access to adaptive novelty.
- [ ] The opt-in sexual mode stays disabled until its complete dual-parent and homologous-recombination contract is available, then exchanges homologous function without destructive arbitrary topology splicing.
- [ ] Controlled campaigns compare asexual and sexual treatments for adaptation, modularity, robustness, and diversity.

## Executable Features

- [ ] **T08.F01 — Mutational Neighborhood Assay** — Depends on: T01.F02, T10.F05
- [ ] **T08.F02 — Lineage Mutation-Effect Attribution** — Depends on: T01.F03, T08.F01
- [ ] **T08.F03 — Neutral Network and Robustness Characterization** — Depends on: T08.F01, T08.F02
- [ ] **T08.F04 — Function-Preserving Duplication and Module Growth** — Depends on: T08.F01, T08.F03
- [ ] **T08.F05 — Heritable Mutation Policy** — Depends on: T08.F02, T08.F03
- [ ] **T08.F06 — Homology and Mating Compatibility** — Depends on: T04.F05, T08.F04
- [ ] **T08.F07 — Dormant Dual-Parent Mating Contract** — Depends on: T03.F07, T08.F06
- [ ] **T08.F08 — Homologous Recombination and Sexual-Mode Enablement** — Depends on: T08.F07
- [ ] **T08.F09 — Heredity-Mode Comparison Confirmatory Campaign** — Depends on: T01.F09, T04.F07, T08.F05, T08.F08, T10.F08

## Notes for AI Agents

- Extend Petri's current mutation operators, parseability gate, reachability bias, and mutation observability. Parseability is not behavioral viability.
- Measure local mutational neighborhoods before tuning operator probabilities. Separate neutral structure that improves robustness from unreachable bloat that only absorbs mutations.
- Sexual reproduction is committed as an opt-in experimental treatment, not as the production default and not as an assumed improvement. Asexual reproduction remains the control, and T08.F09 may truthfully find no benefit.
- T08.F06 through T08.F09 are deferrable: do not plan their specs until the first-slice go/no-go in the master roadmap has passed and T08.F01 through T08.F05 have produced mutational-neighborhood evidence.
- T08.F07 may add deterministic mate intent, pairing, contention, contribution, target, and draft semantics, but the creature-facing sexual mode remains unavailable by default. T08.F08 enables it only after recombination, viability, and inheritance tests pass.
- T08.F07 and T08.F08 explicitly supersede the one-parent portions of `docs/reference/v3-reproduction-spec.md`, `docs/reference/v3-creature-identity-spec.md`, `crates/v3-core/src/contracts/actions.rs` (`WorldAction::Reproduce { direction, energy_transfer }`), and `crates/v3-core/src/simulation/actions/reproduction.rs` (`apply_reproduce`). Their specs must define two-parent energy and reserve contributions, spawn targeting, turn-order contention, generation and ancestry, lineage and kin identity, shared memory, phenotype, graph state, learned weights, mutation ordering, and rejection telemetry while preserving the asexual path.
- Research basis reviewed 2026-09-02: [The evolutionary origin of complex features](https://doi.org/10.1038/nature01568), [Sexual reproduction reshapes the genetic architecture of digital organisms](https://doi.org/10.1098/rspb.2005.3338), and [Homology and linkage in crossover for variable-length genomes](https://doi.org/10.1371/journal.pone.0209712).

| Option | Petri fit and evidence | Decision / exit criterion |
| --- | --- | --- |
| Mutation tuning without sex | Lowest contract risk and remains the required control, but cannot test exchange between lineages. | Retain as control, not as a substitute for the opt-in treatment. |
| Relative-position or arbitrary topology crossover | Demonstrated in simpler linear digital genomes, but Petri has variable nested VM, graph, and mesh structures. | Reject as the default because it does not establish functional homology. |
| Compatibility plus homology-aware module exchange | Matches Petri's topology and lets exchange quality be measured before evolutionary claims. | Adopt provisionally. T08.F06 must compare candidate mappings on homology, linkage, child viability, function retention, exchange rate, and cost; if none beats asexual mutation under the predeclared safety floor, block enablement rather than splice arbitrarily. |

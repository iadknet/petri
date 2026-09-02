# T06 — Niche Construction and Ecological Inheritance

**Status**: Planned
**Last updated**: 2026-09-02
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Allow creatures to create persistent, costly changes to resources and habitat
that alter selection for themselves and other lineages across organismal
lifetimes and generations.

## Track Success Criteria

- [ ] Creature-authored world changes are distinguishable from externally scheduled environment state and have explicit persistence or decay.
- [ ] Resource movement, transformation, and habitat modification create measurable opportunities or costs for other strategies.
- [ ] Transfer and erasure assays demonstrate ecological inheritance rather than mere correlation with constructor presence.
- [ ] Long-running treatments sustain multiple construction, exploitation, or avoidance strategies without direct niche rewards.

## Executable Features

- [ ] **T06.F01 — Durable Creature-Authored World State** — Depends on: T01.F01, T02.F01
- [ ] **T06.F02 — Resource Transport and Caching** — Depends on: T03.F05, T06.F01
- [ ] **T06.F03 — Resource Transformation and Byproduct Loops** — Depends on: T03.F04, T06.F01
- [ ] **T06.F04 — Habitat Modification Actions** — Depends on: T03.F01, T06.F01
- [ ] **T06.F05 — Persistence, Decay, and Ecological Inheritance** — Depends on: T06.F02, T06.F03, T06.F04
- [ ] **T06.F06 — Constructor Transfer and Erasure Assays** — Depends on: T06.F05, T10.F05
- [ ] **T06.F07 — Niche-Construction Diversification Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T04.F06, T06.F06, T10.F08

## Notes for AI Agents

- The applied world grid owns environmental truth. Construction actions must change that state through normal action resolution and energy accounting.
- A creature merely consuming or occupying a cell is not sufficient evidence of niche construction; the change must alter later selection and survive long enough to be assayed.
- Treat persistence as an experimental variable. Too little prevents inheritance, while excessive persistence can lock in stale or maladaptive structures.
- Research basis reviewed 2026-09-02: [Evolution of Complex Niche-Constructing Behaviors and Ecological Inheritance](https://doi.org/10.3389/frobt.2020.600387) and [Evolutionary consequences of niche construction](https://doi.org/10.1073/pnas.96.18.10242).
- Options considered were more developer-authored resources, ephemeral creature effects, and persistent organism-mediated change. Use the latter because it makes organisms part of one another's inherited selective environment.

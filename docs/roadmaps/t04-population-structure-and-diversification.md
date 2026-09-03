# T04 — Population Structure and Diversification

**Status**: Planned
**Last updated**: 2026-09-03
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Allow locally adapted strategies to originate, persist, migrate, compete, and
form distinguishable ecotypes without immediate global homogenization by one
successful generalist.

## Track Success Criteria

- [ ] Ancestry and ecotype evidence distinguishes persistent local adaptation from temporary phenotype variation.
- [ ] Dispersal, connectivity, and disturbance can be varied independently in controlled treatments.
- [ ] Reciprocal-transplant and invasion assays demonstrate environment-specific adaptation and stable coexistence.
- [ ] Reproductive isolation, when present, is measured as an evolved outcome rather than inferred from spatial separation alone.

## Executable Features

- [ ] **T04.F01 — Ancestry Graph and Ecotype Observation** — Depends on: T01.F03
- [ ] **T04.F02 — Local Dispersal and Reproduction Range** — Depends on: T01.F01
- [ ] **T04.F03 — Habitat Patch and Connectivity Model** — Depends on: T01.F12, T02.F01
- [ ] **T04.F04 — Migration and Patch Disturbance Regimes** — Depends on: T02.F05, T04.F02, T04.F03
- [ ] **T04.F05 — Reciprocal-Transplant and Invasion Assays** — Depends on: T04.F01, T04.F03, T10.F05
- [ ] **T04.F06 — Persistent Specialist Coexistence Confirmatory Campaign** — Depends on: T01.F09, T03.F09, T04.F04, T04.F05, T10.F08
- [ ] **T04.F07 — Reproductive-Isolation and Speciation Evidence** — Depends on: T04.F06, T08.F08

## Notes for AI Agents

- Petri's founder `lineage_id` is stable clade identity, not a complete parent-child ancestry graph and not evidence of an ecotype.
- Habitat patches are small worlds. On 2026-09-03 no world at or below 512-by-512 persisted under production defaults, so T04.F03 depends on T01.F12 and must size patches at or above the recorded minimum persistent population, or state the economics change that makes smaller patches viable.
- Keep geography, ecological strategy, ancestry, and mating compatibility as separate observations.
- Local reproduction and limited dispersal should precede explicit mating barriers; first determine whether ecology maintains differentiated populations.
- Research basis reviewed 2026-09-02: [Ecological and Mutation-Order Speciation in Digital Organisms](https://doi.org/10.1086/674359) and [Digital Evolution for Ecology Research](https://doi.org/10.3389/fevo.2021.750779).
- Options considered were global panmixia, hard-coded species labels, and spatially structured populations with measured ecotypes. Use spatial structure and assays because labels do not establish ecological or reproductive independence.

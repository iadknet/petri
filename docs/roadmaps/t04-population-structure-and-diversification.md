# T04 — Population Structure and Diversification

**Status**: Planned
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give the world geography: offspring born near their parents, patches joined by
corridors, and migration, so locally adapted ways of living can originate,
persist, and form distinguishable ecotypes without one successful generalist
homogenizing everything.

## Track Success Criteria

- [ ] Ancestry and ecotype evidence distinguishes persistent local adaptation from temporary phenotype variation.
- [ ] Dispersal, connectivity, and disturbance can be varied independently.
- [ ] Reciprocal-transplant and invasion assays demonstrate environment-specific adaptation and stable coexistence.
- [ ] Reproductive isolation, when present, is measured as an evolved outcome rather than inferred from spatial separation alone.

## Executable Features

- [ ] **T04.F01 — Ancestry Graph and Ecotype Observation** — Depends on: T01.F12
  - Goal: Family trees. Record parent-child ancestry so persistent local adaptation can be told apart from passing variation; observation only, no world change.
- [ ] **T04.F03 — Habitat Patch and Connectivity Model** — Depends on: T01.F12, T02.F01
  - Goal: Islands and valleys. The T02.F02 terrain forms patches joined by corridors, with connectivity as the knob, so local populations can diverge.
- [ ] **T04.F04 — Migration and Patch Disturbance Regimes** — Depends on: T02.F05, T04.F03
  - Goal: Corridors open and close. Migration between patches varies and the T02.F05 disturbances strike patches, so recolonization and gene flow can be studied.
- [ ] **T04.F05 — Reciprocal-Transplant and Invasion Assays** — Depends on: T04.F01, T04.F03, T10.F05
  - Goal: Deferred proof phase. Move creatures between patches and see whether they do worse away from home.
- [ ] **T04.F06 — Persistent Specialist Coexistence Confirmatory Campaign** — Depends on: T01.F09, T03.F09, T04.F04, T04.F05, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that specialists persist side by side.
- [ ] **T04.F07 — Reproductive-Isolation and Speciation Evidence** — Depends on: T04.F06, T08.F08
  - Goal: Deferred proof phase. Do separated populations stop interbreeding when they meet again?

## Notes for AI Agents

- Petri's founder `lineage_id` is stable clade identity, not a complete parent-child ancestry graph and not evidence of an ecotype. T04.F01 adds the ancestry graph; it is reachable after T01.F12 and can be pulled forward whenever the lineage-count indicator stops being informative enough.
- Habitat patches are small worlds. T01.F11 measured on 2026-09-04 that persistence below the default 1600-by-1600 world is seed-dependent (128-by-128 went extinct on every seed; one 256-by-256 seed survived as a single creature; one 512-by-512 seed was still falling at tick 2,000), so T04.F03 must size patches, or the total passable area behind them, at or above what persists, or state the economics change that makes smaller patches viable.
- Keep geography, ecological strategy, ancestry, and mating compatibility as separate observations.
- Local reproduction already exists: offspring spawn in a cell adjacent to the parent (`docs/reference/v3-reproduction-spec.md`), so the former T04.F02 was removed on 2026-09-04 as redundant. Ecology and terrain should be shown to maintain differentiated populations before any explicit mating barrier is added.
- Research basis reviewed 2026-09-02: [Ecological and Mutation-Order Speciation in Digital Organisms](https://doi.org/10.1086/674359) and [Digital Evolution for Ecology Research](https://doi.org/10.3389/fevo.2021.750779).
- Options considered were global panmixia, hard-coded species labels, and spatially structured populations with measured ecotypes. Use spatial structure and assays because labels do not establish ecological or reproductive independence.

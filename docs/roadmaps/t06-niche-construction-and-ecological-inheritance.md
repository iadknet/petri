# T06 — Niche Construction and Ecological Inheritance

**Status**: Planned
**Last updated**: 2026-09-08
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Let creatures leave lasting marks on the world: burrows, caches, trails,
dung, and cleared ground that persist or decay and change selection for
themselves and for other lineages across lifetimes and generations.

## Track Success Criteria

- [ ] Creature-authored world changes are distinguishable from weather and terrain and have explicit persistence or decay.
- [ ] Resource movement, transformation, and habitat modification create measurable opportunities or costs for other strategies.
- [ ] Transfer and erasure assays demonstrate ecological inheritance rather than mere correlation with constructor presence.
- [ ] Long-running treatments sustain multiple construction, exploitation, or avoidance strategies without direct niche rewards.

## Executable Features

- [ ] **T06.F01 — Material Carrying and Barrier Construction** — Depends on: T12.F01
  - Goal: Burrows and walls. Creatures use four shared storage slots to pick up and relocate barriers at an action energy cost and a movement penalty per occupied slot, leaving changes that persist beyond their makers.
- [ ] **T06.F02 — Food Transport and Caching** — Depends on: T06.F01
  - Goal: Caching. Creatures use the same four slots to carry food, eat it later, or deposit it for any creature to discover, preserving its properties and decay clock while paying the same movement penalty per occupied slot.
- [ ] **T06.F03 — Resource Transformation and Byproduct Loops** — Depends on: T03.F04, T06.F01
  - Goal: Dung and decay. Eating produces byproducts that become another food type, so one lineage's waste is another's meal.
- [ ] **T06.F04 — Habitat Modification Actions** — Depends on: T03.F01, T06.F01
  - Goal: Soil engineering. Creatures alter habitat properties such as fertility beyond barrier relocation, making places better or worse for everyone who comes after.
- [ ] **T06.F05 — Persistence, Decay, and Ecological Inheritance** — Depends on: T06.F02, T06.F03, T06.F04
  - Goal: Inheritance of place. Constructed changes outlast their makers and shape what their descendants and neighbors face.
- [ ] **T06.F06 — Constructor Transfer and Erasure Assays** — Depends on: T06.F05, T10.F05
  - Goal: Deferred proof phase. Erase or transplant constructions and see whether the advantage travels with them.
- [ ] **T06.F07 — Niche-Construction Diversification Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T04.F06, T06.F06, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that construction sustains more ways of living.

## Notes for AI Agents

- Advance designs requested 2026-09-04: [T06.F01 spec](../specs/roadmap/t06-f01-material-carrying-and-barrier-construction.md) and [T06.F02 spec](../specs/roadmap/t06-f02-food-transport-and-caching.md). Both remain Planned. F01 reuses T12.F01 seeded barriers (moved from T02.F02 on 2026-09-08; T02.F02 keeps their visibility behavior, which the sensor contract already provides); F02 transports uneaten food and does not require T03.F05 internal energy-storage traits. Trails are outside these two features. F04 covers habitat properties beyond F01's barrier relocation.
- The applied world grid owns environmental truth. Construction actions must change that state through normal action resolution and energy accounting.
- A creature merely consuming or occupying a cell is not sufficient evidence of niche construction; the change must alter later selection and survive long enough to be assayed.
- Treat persistence as an experimental variable. Too little prevents inheritance, while excessive persistence can lock in stale or maladaptive structures.
- Research basis reviewed 2026-09-02: [Evolution of Complex Niche-Constructing Behaviors and Ecological Inheritance](https://doi.org/10.3389/frobt.2020.600387) and [Evolutionary consequences of niche construction](https://doi.org/10.1073/pnas.96.18.10242).
- Options considered were more developer-authored resources, ephemeral creature effects, and persistent organism-mediated change. Use the latter because it makes organisms part of one another's inherited selective environment.

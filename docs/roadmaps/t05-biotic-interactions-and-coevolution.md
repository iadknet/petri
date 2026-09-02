# T05 — Biotic Interactions and Coevolution

**Status**: Planned
**Last updated**: 2026-09-02
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Make other evolving creatures a changing source of selection through
frequency-dependent exploitation, defense, and counter-adaptation that can
sustain arms races or cycling strategies without directly rewarding complexity.

## Track Success Criteria

- [ ] Interaction outcomes derive from both participants' applied traits, actions, and current state.
- [ ] Historical-opponent assays distinguish escalation, cycling, and one-sided adaptation.
- [ ] Coevolution treatments produce persistent reciprocal adaptation beyond matched no-coevolution controls.
- [ ] Multi-trophic interactions yield a measured ecological network with more than one durable interaction strategy.

## Executable Features

- [ ] **T05.F01 — Extensible Biotic Interaction Outcome Framework** — Depends on: T01.F01, T03.F01
- [ ] **T05.F02 — Predator-Prey Sensing and Action Loop** — Depends on: T03.F06, T05.F01
- [ ] **T05.F03 — Frequency-Dependent Interaction Telemetry** — Depends on: T01.F03, T05.F01
- [ ] **T05.F04 — Historical-Opponent Replay Assays** — Depends on: T05.F03, T10.F05
- [ ] **T05.F05 — Predator-Prey Coevolution Characterization Campaign** — Depends on: T04.F05, T05.F02, T05.F04, T10.F08
- [ ] **T05.F06 — Host-Parasite Exploitation Lifecycle** — Depends on: T05.F01, T05.F05
- [ ] **T05.F07 — Multi-Trophic Ecological Network Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T05.F05, T05.F06, T10.F08

## Notes for AI Agents

- Extend the existing `StealEnergy` interaction and predation accounting only where they provide sound primitives; do not treat kill counts as proof of coevolution.
- Compare contemporaries with archived ancestors and descendants from both roles. Reciprocal adaptation requires each population to change the other's adaptive landscape.
- T05.F05 is characterization that qualifies the interaction loop before the host-parasite extension. T05.F07 is the result-blind confirmatory campaign for this track.
- Preserve the distinction between antagonistic interactions here, persistent environment modification in T06, and signals or cooperation in T07.
- Research basis reviewed 2026-09-02: [Coevolution drives the emergence of complex traits and promotes evolvability](https://doi.org/10.1371/journal.pbio.1002023) and [Evolving Digital Ecological Networks](https://doi.org/10.1371/journal.pone.0056468).
- Options considered were stronger fixed predators, static payoff games, and evolving interacting populations. Use evolving populations because static opponents eventually become fixed optimization targets.

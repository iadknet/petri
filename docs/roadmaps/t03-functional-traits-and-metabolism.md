# T03 — Functional Traits and Metabolism

**Status**: Planned
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give creatures bodies that differ: heritable traits that change what they can
sense, eat, store, survive, and invest in, each with an applied cost, so
specialist ways of living can evolve instead of cosmetic variation or direct
fitness bonuses.

## Track Success Criteria

- [ ] Every body trait changes applied simulation behavior and has an observable ecological cost or tradeoff.
- [ ] Trait mutation preserves bounded viable steps and exposes its applied outcomes through the goal-profile report.
- [ ] Multiple specialist trait combinations can invade when rare and coexist under appropriate environments.
- [ ] No trait or cost directly rewards controller size, cognitive score, novelty, or diversity.

## Executable Features

- [ ] **T03.F01 — Heritable Body Traits with Costs** — Depends on: None
  - Goal: Bodies differ. Creatures inherit continuous body traits that change what they can do and what it costs them, through the same energy accounting that governs eating and reproduction, so specialist roles can evolve.
- [ ] **T03.F02 — Evolvable Perception Tradeoffs** — Depends on: T03.F01
  - Goal: Eyes cost. Sharper or wider senses cost more energy to maintain, compared through a nested sensor ladder (resource sense, plus direction, plus vision).
- [ ] **T03.F03 — Movement Capability and Efficiency Tradeoffs** — Depends on: T03.F01
  - Goal: Legs and fins. Speed, efficiency, and terrain handling trade off against each other and against upkeep.
- [ ] **T03.F04 — Resource Conversion and Digestion Specialization** — Depends on: T02.F01, T03.F01
  - Goal: Guts. A creature digests some food types better than others, and a generalist pays for breadth.
- [ ] **T03.F05 — Energy Storage and Allocation Tradeoffs** — Depends on: T03.F01
  - Goal: Fat. Larger reserves survive lean seasons but cost more to carry and to build.
- [ ] **T03.F06 — Attack, Defense, and Escape Traits** — Depends on: T03.F01, T05.F01
  - Goal: Claws and shells. Offensive, defensive, and escape traits trade off, and an encounter's outcome falls out of both bodies.
- [ ] **T03.F07 — Life-History and Offspring Investment Traits** — Depends on: T03.F01
  - Goal: Litters. Few well-provisioned offspring or many cheap ones, and when to mature, as heritable choices.
- [ ] **T03.F08 — Computational Capacity and Maintenance Tradeoffs** — Depends on: T03.F01
  - Goal: Brains are expensive. Larger or more active controllers cost more to maintain, so cognition has to pay for itself.
- [ ] **T03.F09 — Functional Specialization Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T03.F02, T03.F03, T03.F04, T03.F05, T03.F07, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that specialists coexist rather than one generalist winning.

## Notes for AI Agents

- The existing phenotype color walk is identity and visualization state, not functional morphology. Do not overload it as a causal trait system.
- Keep energy conserved and behavior-backed: sensing, motion, storage, digestion, defense, reproduction, and computation must affect the same applied accounting used for survival and reproduction. T03.F01 is a set of heritable numbers with costs wired into that accounting, not a lookup table or a generic framework.
- Introduce continuous or otherwise locally mutable trait spaces where practical so useful specializations have reachable stepping stones.
- Morphology can facilitate control as well as impose cost; do not automatically attribute behavior enabled by a body trait to controller cognition.
- Research basis reviewed 2026-09-02: [What Is Morphological Computation?](https://doi.org/10.1162/ARTL_a_00219) and [Evolving embodied intelligence from materials to machines](https://doi.org/10.1038/s42256-018-0009-9). Added 2026-09-03: [The Emergence of Complex Behavior in Large-Scale Ecological Environments](https://arxiv.org/abs/2510.18221) reports that adding a directional sense and then vision produced qualitatively new foraging and predation strategies; T03.F02 should adopt that nested sensor-ablation treatment design (resource sense only, plus direction, plus vision) so perception tradeoffs are compared against a known-effective control structure.
- Options considered were fixed creature capabilities, direct niche labels, and evolvable applied traits. Use applied traits because fixed capabilities constrain niche count and direct labels create developer-assigned roles rather than evolved specialization.

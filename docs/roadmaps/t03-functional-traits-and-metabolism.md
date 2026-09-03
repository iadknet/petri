# T03 — Functional Traits and Metabolism

**Status**: Planned
**Last updated**: 2026-09-03
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Make creatures heritably different in what they can sense, consume, store,
survive, and invest in, with applied tradeoffs that create viable specialist
roles instead of cosmetic phenotype variation or direct fitness bonuses.

## Track Success Criteria

- [ ] Every functional trait changes applied simulation behavior and has an observable ecological cost or tradeoff.
- [ ] Trait mutation preserves bounded viable steps and exposes its applied outcomes through experiment artifacts.
- [ ] Multiple specialist trait combinations can invade when rare and coexist under appropriate environments.
- [ ] No trait or cost directly rewards controller size, cognitive score, novelty, or diversity.

## Executable Features

- [ ] **T03.F01 — Heritable Functional Trait Framework** — Depends on: T01.F01
- [ ] **T03.F02 — Evolvable Perception Tradeoffs** — Depends on: T03.F01
- [ ] **T03.F03 — Movement Capability and Efficiency Tradeoffs** — Depends on: T03.F01
- [ ] **T03.F04 — Resource Conversion and Digestion Specialization** — Depends on: T02.F01, T03.F01
- [ ] **T03.F05 — Energy Storage and Allocation Tradeoffs** — Depends on: T03.F01
- [ ] **T03.F06 — Attack, Defense, and Escape Traits** — Depends on: T03.F01, T05.F01
- [ ] **T03.F07 — Life-History and Offspring Investment Traits** — Depends on: T03.F01
- [ ] **T03.F08 — Computational Capacity and Maintenance Tradeoffs** — Depends on: T03.F01
- [ ] **T03.F09 — Functional Specialization Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T03.F02, T03.F03, T03.F04, T03.F05, T03.F07, T10.F08

## Notes for AI Agents

- The existing phenotype color walk is identity and visualization state, not functional morphology. Do not overload it as a causal trait system.
- Keep energy conserved and behavior-backed: sensing, motion, storage, digestion, defense, reproduction, and computation must affect the same applied accounting used for survival and reproduction.
- Introduce continuous or otherwise locally mutable trait spaces where practical so useful specializations have reachable stepping stones.
- Morphology can facilitate control as well as impose cost; do not automatically attribute behavior enabled by a body trait to controller cognition.
- Research basis reviewed 2026-09-02: [What Is Morphological Computation?](https://doi.org/10.1162/ARTL_a_00219) and [Evolving embodied intelligence from materials to machines](https://doi.org/10.1038/s42256-018-0009-9). Added 2026-09-03: [The Emergence of Complex Behavior in Large-Scale Ecological Environments](https://arxiv.org/abs/2510.18221) reports that adding a directional sense and then vision produced qualitatively new foraging and predation strategies; T03.F02 should adopt that nested sensor-ablation treatment design (resource sense only, plus direction, plus vision) so perception tradeoffs are compared against a known-effective control structure.
- Options considered were fixed creature capabilities, direct niche labels, and evolvable applied traits. Use applied traits because fixed capabilities constrain niche count and direct labels create developer-assigned roles rather than evolved specialization.

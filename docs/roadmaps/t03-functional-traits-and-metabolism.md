# T03 — Functional Traits and Metabolism

**Status**: Planned
**Last updated**: 2026-09-07
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
- [ ] **T03.F08 — Computational Capacity and Maintenance Tradeoffs** — Depends on: T03.F01, T11.F10
  - Goal: Brains are expensive. Larger or more active controllers cost more to maintain, so cognition has to pay for itself.
- [ ] **T03.F09 — Functional Specialization Confirmatory Campaign** — Depends on: T01.F06, T01.F09, T03.F02, T03.F03, T03.F04, T03.F05, T03.F07, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that specialists coexist rather than one generalist winning.
- [ ] **T03.F10 — Activity-Ramped Compute Cost** — Depends on: None
  - Goal: Thinking burns fuel and fatigue sets in. A brain pays for each instruction it runs, cheaply for ordinary programs and at a rising rate for sustained execution within one dispatch, so a runaway loop starves its creature instead of being silently cut off at the step cap.

## Notes for AI Agents

- The existing phenotype color walk is identity and visualization state, not functional morphology. Do not overload it as a causal trait system.
- Keep energy conserved and behavior-backed: sensing, motion, storage, digestion, defense, reproduction, and computation must affect the same applied accounting used for survival and reproduction. T03.F01 is a set of heritable numbers with costs wired into that accounting, not a lookup table or a generic framework.
- Introduce continuous or otherwise locally mutable trait spaces where practical so useful specializations have reachable stepping stones.
- T03.F10 (added 2026-09-07 at the user's direction) makes runaway computation lethal instead of merely truncated. Evidence: the T11.F17 goal run's seed 33 ended with 12.05% of its 11,379 creatures running to the 10,000-step `max_vm_steps` cap every tick, holding 97.76% of that tick's VM steps, while `vm_steps` per creature-tick read +132.76% against the pinned epoch. A capped run costs 0.01 energy at the current `runtime.vm.opcode_cost_multiplier` of 1e-6, and because `crates/v3-core/src/runtime/vm.rs` subtracts each step's cost from the creature's `f32` energy one step at a time, a 1e-6 charge is below the float's resolution for any creature above about 16 energy and rounds to nothing; cognition is free today in fact as well as in effect. The hard cap has no ecological consequence: a looping creature keeps whatever its action queue held at the cap and reproduces as well as any other, so selection never sees the loop. The feature charges VM execution within one node dispatch on a ramp: ordinary programs and short bounded loops (tens to a few hundred steps, including a pass over the 16 memory slots) stay nearly free, and cost per step rises with the step index past a free allowance, so a run to the cap costs a lethal fraction of a creature's energy (illustration at a linear ramp: 100 steps about 0.005 energy, 1,000 steps about one tick of decay, 10,000 steps about 50 energy); the existing exhausted-energy path (queue discarded, `NoOp`, death at zero energy) does the rest. The spec fixes the allowance, the ramp shape and constants, and the interaction with `SetPriorityBid`'s own energy deduction, accumulates the dispatch's cost locally and subtracts it once so the ramp's early steps survive `f32` rounding, keeps `max_vm_steps` as the per-tick compute bound with energy binding well before it, and charges within a node dispatch only, since the hop cap and single-visit rule already bound the chain. Natural analog: neural activity is metabolically expensive in proportion to what fires, and sustained activity beyond a node's local supply draws on the body at rising cost (fatigue); the spec names the ramp shape as engineering on that analog rather than claiming the curve itself from nature. Its research step reads the activity-cost literature for the linear part and checks whether any measured superlinearity supports the ramp. It changes production ecology: `cargo test -p v3-core --test viability` runs first, persistence is re-read against the T11.F04 sweeps, `vm_steps` is expected to fall on the goal profile, and the gate and goal reports are compared under the usual thresholds with no severe cost budgeted. It adds no cost on program length, node count, or junk (those remain T03.F08's and T11.F13's questions), no new sensor, and no bonus for short programs. Sequencing: it closes before T11.F17 closes, so the paired long run that decides T11.F17 is read with loops lethal rather than blooming; T03.F08 later reads its realized cost beside the 2026-09-07 figure.
- T03.F08 follows basic controller qualification and founder accessibility. Measure realized energy deltas and executed work before adding maintenance costs; do not assume current cognition is too expensive or use a cost change to compensate for a runtime defect. First realized reading, 2026-09-07 ([depth note](../strategy/mesh-depth-research-2026-09-07.md), Section 3.1): on the user's 281,405-tick run brain compute charged 0.000152 energy per creature-tick, 0.03% of the 0.5 per-tick decay, so cognition is free today. The existing `energy.complexity_cost` multiplier (disabled by default since 2026-03-12) is this feature's lever and stays in the code untouched until then; it scales action costs by `complexity()`, which excludes unreachable junk by design, so it taxes the executed core and T11.F15's reachable-but-losing scaffold and cannot serve as the junk bound T11.F13 characterizes. If T11.F13 selects a per-node cost as that bound, this feature designs it from executed work per node, not from `complexity()`.
- Morphology can facilitate control as well as impose cost; do not automatically attribute behavior enabled by a body trait to controller cognition.
- Research basis reviewed 2026-09-02: [What Is Morphological Computation?](https://doi.org/10.1162/ARTL_a_00219) and [Evolving embodied intelligence from materials to machines](https://doi.org/10.1038/s42256-018-0009-9). Added 2026-09-03: [The Emergence of Complex Behavior in Large-Scale Ecological Environments](https://arxiv.org/abs/2510.18221) reports that adding a directional sense and then vision produced qualitatively new foraging and predation strategies; T03.F02 should adopt that nested sensor-ablation treatment design (resource sense only, plus direction, plus vision) so perception tradeoffs are compared against a known-effective control structure.
- Options considered were fixed creature capabilities, direct niche labels, and evolvable applied traits. Use applied traits because fixed capabilities constrain niche count and direct labels create developer-assigned roles rather than evolved specialization.

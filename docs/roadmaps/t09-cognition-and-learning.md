# T09 — Cognition and Learning

**Status**: Planned
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Show that Petri's creatures come to remember, learn, and predict because the
world rewards it, using the existing controller substrate, and change that
substrate only when a measured gap says it cannot get there.

## Track Success Criteria

- [ ] The T09.F08 memory probe reports, with effect sizes, whether any evolved lineage's actions depend on its shared memory while current observations are held constant.
- [ ] The existing VM, graph, mesh, shared-memory, stateful-node, plasticity, and reward-modulation mechanisms are qualified independently and in combination.
- [ ] Evolved cognition exceeds matched reactive controllers and behavior-preserving minimized controllers in held-out conditions.
- [ ] At least one persistent lineage requires temporally extended state and at least one additional learning, integration, predictive, or social mechanism under the T01 profile.
- [ ] Cognitive advantages persist across independent long-running populations rather than one selected champion or seed.
- [ ] Any cognition-engine extension is represented by a new bounded roadmap feature justified by a measured substrate gap.

## Executable Features

- [ ] **T09.F01 — Existing Controller Substrate Qualification Suite** — Depends on: T01.F02, T01.F05, T10.F05
  - Goal: Deferred proof phase. Prove each existing brain mechanism can carry memory, learning, and integration with fixtures before claiming evolved creatures use them.
- [ ] **T09.F02 — Memory-Motif Evolvability from Founders** — Depends on: T08.F01, T09.F08
  - Goal: Can memory be found? Whether founders in the seasonal, occluded world evolve circuits that read and write shared memory, and how many mutational steps away those circuits are.
- [ ] **T09.F03 — Plasticity and Reward-Learning Qualification** — Depends on: T02.F04
  - Goal: Learning within a lifetime. Whether the existing plasticity and reward modulation help a creature in the overgrazing world, measured with learning frozen versus live.
- [ ] **T09.F04 — Predictive Context and Transfer Assay** — Depends on: T02.F07, T09.F02, T09.F03
  - Goal: Deferred proof phase. Do evolved creatures anticipate the season rather than react to it, and does that hold under new weather?
- [ ] **T09.F05 — Reciprocal and Social Cognition Assay** — Depends on: T05.F04, T07.F06, T09.F01
  - Goal: Deferred proof phase. Does behavior toward another creature depend on that creature responding?
- [ ] **T09.F06 — Apply Controller Minimization and Reactive-Surrogate Protocol** — Depends on: T08.F04, T09.F04, T09.F05
  - Goal: Deferred proof phase. Could a much simpler reactive controller do the same job?
- [ ] **T09.F07 — Replicated Cognition Emergence Confirmatory Campaign** — Depends on: T01.F09, T03.F08, T05.F05, T07.F07, T09.F06, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that cognition emerges and persists.
- [ ] **T09.F08 — Memory-Reset Probe on Evolved Specimens** — Depends on: T01.F12, T02.F02
  - Goal: Does memory matter yet? Run the goal profile in the seasonal, occluded world and report per-lineage memory sensitivity with effect sizes; this is the go/no-go on the existing brain.

## Notes for AI Agents

- T09.F08 is the go/no-go named in the master roadmap and runs in process. Evolve the T01.F12 goal profile in the T02.F02 world for its predeclared horizon, then apply the T01.F12 memory-sensitivity probe to the final population grouped by lineage: re-run each creature's mesh on the same sensor snapshot with shared memory intact, zeroed, and scrambled, and report per-lineage effect sizes and uncertainty. No specimen serialization, manifest, or comparator protocol is needed; T01.F05 owns comparators in the deferred proof phase. A null result across seeds stops the roadmap until a measured substrate or economics gap is addressed by a new bounded feature.
- T09.F02 and T09.F03 depend on the mechanism features they test, not on the deferred T09.F01 qualification suite or the T02.F06 campaign.
- Controller size, executed operations, memory writes, or plasticity updates are not cognition by themselves. Use them only as supporting structural observations.
- Hold current observations constant while perturbing history when testing temporal dependence. Freeze or reset learned weights and reward traces when testing learning dependence.
- Evaluate newborn genomes and mature learned organisms separately, then compare them to minimized and reactive surrogates over held-out environments.
- T09.F06 applies the versioned T01.F05 protocol and reports its conclusive, negative, or inconclusive result; it must not redefine comparator access, search budget, fidelity, or acceptance margins after seeing evolved specimens.
- Final cognition evidence includes the T07 spatial-temporal signal medium and T03 computational maintenance costs through explicit dependencies; signal activity or extra computation without causal adaptive benefit does not count.
- Do not add a speculative replacement brain within T09.F01. If qualification exposes a representational or evolvability limit, add the smallest dependency-linked feature that addresses that measured gap.
- Research basis reviewed 2026-09-02: [Evolution of Integrated Causal Structures in Animats](https://doi.org/10.1371/journal.pcbi.1003966) and [The Evolutionary Origin of Associative Learning](https://doi.org/10.1086/706252).
- Options considered were raw structural complexity, integrated-information measures, benchmark fitness, and causal multi-axis profiling. Use causal profiling because it is controller-architecture neutral and tests whether internal mechanisms matter to adaptive behavior.

# T09 — Cognition and Learning

**Status**: Planned
**Last updated**: 2026-09-23
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Establish whether the world rewards remembering, learning, and predicting,
and whether evolved cognition is real: once T11 has made the brain's building
blocks reliable and reachable, measure the ecological benefit of memory, the
usefulness of within-lifetime learning, and the causal profile of evolved
controllers against reactive and minimized surrogates.

## Track Success Criteria

- [ ] T09.F08 reports separate and combined state interventions over short trajectories, including adaptive outcomes and uncertainty, without treating shared-memory sensitivity as a whole-brain verdict.
- [ ] The existing VM, graph, mesh, shared-memory, stateful-node, plasticity, and reward-modulation mechanisms are qualified independently and in combination.
- [ ] Evolved cognition exceeds matched reactive controllers and behavior-preserving minimized controllers in held-out conditions.
- [ ] At least one persistent lineage requires temporally extended state and at least one additional learning, integration, predictive, or social mechanism under the T01 profile.
- [ ] Cognitive advantages persist across independent long-running populations rather than one selected champion or seed.

## Executable Features

- [ ] **T09.F01 — Existing Controller Substrate Qualification Suite** — Depends on: T01.F02, T01.F05, T10.F05
  - Goal: Deferred proof phase. Extend the T11.F05 engineering fixtures into the full versioned comparator and replay protocol for memory, learning, and integration claims.
- [ ] **T09.F03 — Plasticity and Reward-Learning Qualification** — Depends on: T02.F04, T11.F07, T11.F09
  - Goal: Learning within a lifetime. Whether the existing plasticity and reward modulation help a creature in the overgrazing world, measured with learning frozen versus live.
- [ ] **T09.F04 — Predictive Context and Transfer Assay** — Depends on: T02.F07, T11.F10, T09.F03
  - Goal: Deferred proof phase. Do evolved creatures anticipate the season rather than react to it, and does that hold under new weather?
- [ ] **T09.F05 — Reciprocal and Social Cognition Assay** — Depends on: T05.F04, T07.F06, T09.F01
  - Goal: Deferred proof phase. Does behavior toward another creature depend on that creature responding?
- [ ] **T09.F06 — Apply Controller Minimization and Reactive-Surrogate Protocol** — Depends on: T11.F08, T09.F04, T09.F05
  - Goal: Deferred proof phase. Could a much simpler reactive controller do the same job?
- [ ] **T09.F07 — Replicated Cognition Emergence Confirmatory Campaign** — Depends on: T01.F09, T03.F08, T05.F05, T07.F07, T09.F06, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that cognition emerges and persists.
- [ ] **T09.F08 — Temporal-State and Adaptive-Benefit Diagnostics** — Depends on: T01.F12, T02.F02, T11.F10
  - Goal: Does memory help here? Compare intact and perturbed histories in the seasonal, occluded world to distinguish missing temporal behavior from behavior that brings no adaptive benefit.

## Notes for AI Agents

- Input-evolvability follow-up, 2026-09-23: [T20](t20-input-evolvability-and-structured-variation.md) owns general input recruitment and structured refinement, qualified first on Graph with learning off. Its learning branch is conditional on a demonstrated ecological need. T09 retains general cognition assays and F03's unstarted qualification of native plasticity; only T20.F14's incremental learning comparison depends on it. T20's inherited discovery, retention, ecological transfer and Graph availability do not wait for T09.F03.
- Restructured 2026-09-04 after the [brain evolvability audit](../strategy/brain-evolvability-audit-2026-09-04.md): brain architecture, execution clocks, and mutation accessibility moved to T11. T09.F02 became T11.F10, T09.F09 became T11.F05, T09.F10 became T11.F06, and T09.F11 became T11.F07. Those IDs are retired here and never reused. T09 tests what the repaired substrate does in the world; the track title reverts from Brain Architecture, Cognition, and Learning to Cognition and Learning.
- T09.F08 is a diagnostic checkpoint, not a shared-memory go/no-go. Retain T01.F12's named shared-memory-sensitivity indicator and add separate interventions for current plus previous shared memory, persistent graph state, learned weights, and reward traces, plus combined history interventions. Validate them on the T11.F05 reactive and temporal fixtures. Distinguish inherited parameters from acquired state, hold starting world/body state and RNGs fixed, and report action/state differences and subsequent survival, resource, and reproductive outcomes over bounded paired trajectories, grouped by lineage with uncertainty. Reuse in-process world cloning and replay of short trajectories; no generic specimen serialization or campaign infrastructure is required. Wire new indicators into the goal profile and version changed definitions instead of rewriting old readings.
- Interpret T09.F08 with the T11 and T20 readings: route execution, encoding, supply and inheritance defects to T11/T19's existing owners, and input recruitment or structured-refinement gaps to T20. Evolved temporal behavior without a demonstrated benefit motivates examination of ecological pressure and realized costs. An inadequate exposure or inconclusive comparator is recorded as such. Neither zero shared-memory sensitivity nor any positive action difference alone decides whether the brain is adequate. A demonstrated blocker pauses dependent cognition claims until its owning feature repairs it; create a bounded follow-up only where no existing feature owns the repair. It does not automatically halt unrelated ecology work.
- T09.F03 depends on the T11 engineering foundations, not on the deferred T09.F01 qualification suite or the T02.F06 campaign. T09.F01 reuses the T11.F05 fixtures if the proof phase is later requested; it does not duplicate their engineering role.
- Reused fixtures follow T19's live per-visit internal state, legal budgeted cycles, votes and frozen external perception. Historical T11.F06/F07 specifications cannot restore their retired clocks.
- Controller size, executed operations, memory writes, or plasticity updates are not cognition by themselves. Use them only as supporting structural observations.
- Hold current observations constant while perturbing history when testing temporal dependence. Freeze or reset learned weights and reward traces when testing learning dependence.
- Evaluate newborn genomes and mature learned organisms separately, then compare them to minimized and reactive surrogates over held-out environments.
- T09.F06 applies the versioned T01.F05 protocol and reports its conclusive, negative, or inconclusive result; it must not redefine comparator access, search budget, fidelity, or acceptance margins after seeing evolved specimens.
- Final cognition evidence includes the T07 spatial-temporal signal medium and T03 computational maintenance costs through explicit dependencies; signal activity or extra computation without causal adaptive benefit does not count.
- A substrate gap exposed by a T09 assay is recorded with matched observations, actions, task exposure and compute accounting, then handed to its existing owner: T11 for general representation, supply and inheritance, T19 for execution semantics, T13 for whole-module recruitment, or T20 for its input mechanisms and conditional learning extensions. Reuse a pending feature where its scope fits; do not duplicate it with a new T11 feature or repair it inside the assay. No replacement brain is added here.
- Research basis reviewed 2026-09-02: [Evolution of Integrated Causal Structures in Animats](https://doi.org/10.1371/journal.pcbi.1003966) and [The Evolutionary Origin of Associative Learning](https://doi.org/10.1086/706252). The 2026-09-04 architecture review (SignalGP module regulation, recurrent CGP) and the brain evolvability audit are recorded in the T11 notes and the companion document.
- Options considered were raw structural complexity, integrated-information measures, benchmark fitness, and causal multi-axis profiling. Use causal profiling because it is controller-architecture neutral and tests whether internal mechanisms matter to adaptive behavior.

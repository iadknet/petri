# T09 — Brain Architecture, Cognition, and Learning

**Status**: Planned
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Make the existing brain execute memory and learning reliably, establish whether
evolution can discover useful temporal behavior, and then test whether the world
rewards remembering, learning, and predicting. Extend or replace the controller
only to address a demonstrated gap.

## Track Success Criteria

- [ ] Small engineering fixtures characterize temporal capabilities independently of ecological emergence; graph state and reward traces have explicit update clocks and verified semantics.
- [ ] Founder memory-motif experiments distinguish representability, evolutionary accessibility, retention, and adaptive benefit within predeclared budgets.
- [ ] T09.F08 reports separate and combined state interventions over short trajectories, including adaptive outcomes and uncertainty, without treating shared-memory sensitivity as a whole-brain verdict.
- [ ] The existing VM, graph, mesh, shared-memory, stateful-node, plasticity, and reward-modulation mechanisms are qualified independently and in combination.
- [ ] Evolved cognition exceeds matched reactive controllers and behavior-preserving minimized controllers in held-out conditions.
- [ ] At least one persistent lineage requires temporally extended state and at least one additional learning, integration, predictive, or social mechanism under the T01 profile.
- [ ] Cognitive advantages persist across independent long-running populations rather than one selected champion or seed.
- [ ] Any cognition-engine extension is represented by a new bounded roadmap feature justified by a measured substrate gap.

## Executable Features

- [ ] **T09.F01 — Existing Controller Substrate Qualification Suite** — Depends on: T01.F02, T01.F05, T10.F05
  - Goal: Deferred proof phase. Extend the basic engineering fixtures into the full versioned comparator and replay protocol for memory, learning, and integration claims.
- [ ] **T09.F02 — Memory-Motif Evolvability from Founders** — Depends on: T08.F01, T08.F04, T08.F12, T09.F09, T09.F10, T09.F11
  - Goal: Can memory be found? In small temporal tasks, measure whether founders can reach and retain useful remembered decisions through viable mutations before testing seasonal ecology.
- [ ] **T09.F03 — Plasticity and Reward-Learning Qualification** — Depends on: T02.F04, T09.F11, T08.F12
  - Goal: Learning within a lifetime. Whether the existing plasticity and reward modulation help a creature in the overgrazing world, measured with learning frozen versus live.
- [ ] **T09.F04 — Predictive Context and Transfer Assay** — Depends on: T02.F07, T09.F02, T09.F03
  - Goal: Deferred proof phase. Do evolved creatures anticipate the season rather than react to it, and does that hold under new weather?
- [ ] **T09.F05 — Reciprocal and Social Cognition Assay** — Depends on: T05.F04, T07.F06, T09.F01
  - Goal: Deferred proof phase. Does behavior toward another creature depend on that creature responding?
- [ ] **T09.F06 — Apply Controller Minimization and Reactive-Surrogate Protocol** — Depends on: T08.F04, T09.F04, T09.F05
  - Goal: Deferred proof phase. Could a much simpler reactive controller do the same job?
- [ ] **T09.F07 — Replicated Cognition Emergence Confirmatory Campaign** — Depends on: T01.F09, T03.F08, T05.F05, T07.F07, T09.F06, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs that cognition emerges and persists.
- [ ] **T09.F08 — Temporal-State and Adaptive-Benefit Diagnostics** — Depends on: T01.F12, T02.F02, T09.F02
  - Goal: Does memory help here? Compare intact and perturbed histories in the seasonal, occluded world to distinguish missing temporal behavior from behavior that brings no adaptive benefit.
- [ ] **T09.F09 — Basic Temporal Controller Diagnostics** — Depends on: T01.F12
  - Goal: Check the brain's building blocks with small delayed-cue, memory-retention, and delayed-reward fixtures, reporting capability and measured gaps before interpreting evolution.
- [ ] **T09.F10 — Stable Graph Memory Timing** — Depends on: T09.F09
  - Goal: Neural timescales. Remembered state advances on an explicit world-tick clock, so extra internal settling passes or disconnected computation cannot silently speed up forgetting.
- [ ] **T09.F11 — Reward Trace Timing and Learning Calibration** — Depends on: T09.F09
  - Goal: Synaptic eligibility. Recent activity fades with elapsed world time and influences later learning through an explicit, calibrated reward-update rule.

## Notes for AI Agents

- T09.F09 is engineering characterization, separate from the deferred T09.F01 proof suite. Use existing in-process runtime/test seams and small constructed reactive, shared-memory, stateful-graph, and plasticity cases; include delayed cues with identical current observations, several delay lengths, disconnected-node perturbations, and rewards after skipped module visits. Record expected and observed behavior under production settings. It may close with explicitly recorded capability gaps assigned to the bounded repair features, but never by waiving a failing required check or calling a gap a pass. It changes no production behavior and needs no specimen storage, campaign scheduler, or replacement engine. Later repair features rerun the affected fixtures.
- T09.F10 separates persistent temporal updates from within-tick graph relaxation. Current `runtime/cgp/execute.rs` zeroes ordinary recurrent outputs per dispatch and advances stateful operators each relaxation pass, including passes prolonged by disconnected nodes. Define state visibility, repeated dispatch within a tick, and energy-exhaustion rollback; verify memory trajectories across pass limits and disconnected-node additions while preserving combinational computation. Audit realized energy and compute costs separately. The spec chooses the smallest extension of current graph state; this feature does not replace the VM or add a new controller family.
- T09.F11 defines the time unit and gain of eligibility traces and reward updates. Current `runtime/plasticity/traces.rs` decays only on module execution, while `simulation/tick.rs` applies rewards each tick; the learning rate enters both the trace and `runtime/plasticity/reward.rs`. Test skipped and repeated visits, immediate and delayed rewards, exact one-edge updates, and a small live-versus-frozen reversal task. Remove stale-credit timing and explicitly justify the gain rule rather than assuming the double application was accidental. This is runtime qualification; T09.F03 later measures ecological usefulness.
- T09.F02 uses the corrected substrate and T08 neighborhoods on bounded delayed-cue or alternation tasks where current observations alone cannot determine the correct response. First demonstrate a constructed memory controller's advantage over a matched reactive control. Then measure discovery from founders, viable intermediate changes, retention, and outcomes across predeclared seeds, births, mutation opportunities, and wall-clock limits. Hold observation/action access fixed and separate execution changes from mutation treatments. Hand-built success proves capability, not emergence; failure to evolve within budget is an accessibility result for that exposure, not proof of impossibility. Authored tasks are diagnostics, off in production. No seasonal-world or deferred-campaign dependency is needed.
- T09.F08 is a diagnostic checkpoint, not a shared-memory go/no-go. Retain T01.F12's named shared-memory-sensitivity indicator and add separate interventions for current plus previous shared memory, persistent graph state, learned weights, and reward traces, plus combined history interventions. Validate them on T09.F09 reactive and temporal fixtures. Distinguish inherited parameters from acquired state, hold starting world/body state and RNGs fixed, and report action/state differences and subsequent survival, resource, and reproductive outcomes over bounded paired trajectories, grouped by lineage with uncertainty. Reuse in-process world cloning and replay of short trajectories; no generic specimen serialization or campaign infrastructure is required. Wire new indicators into the goal profile and version changed definitions instead of rewriting old readings.
- Interpret T09.F08 with the earlier diagnostics: runtime failures require targeted repair; a capable controller that evolution does not find motivates mutation/accessibility work; evolved temporal behavior without a demonstrated benefit motivates examination of ecological pressure and realized costs. An inadequate exposure or inconclusive comparator is recorded as such. Neither zero shared-memory sensitivity nor any positive action difference alone decides whether the brain is adequate. A demonstrated blocker pauses dependent cognition claims and gets one bounded follow-up feature before their continuation; it does not automatically halt unrelated ecology work.
- T09.F02 and T09.F03 depend on engineering foundations, not on the deferred T09.F01 qualification suite or the T02.F06 campaign. T09.F01 reuses these fixtures if the proof phase is later requested; it does not duplicate their engineering role.
- Controller size, executed operations, memory writes, or plasticity updates are not cognition by themselves. Use them only as supporting structural observations.
- Hold current observations constant while perturbing history when testing temporal dependence. Freeze or reset learned weights and reward traces when testing learning dependence.
- Evaluate newborn genomes and mature learned organisms separately, then compare them to minimized and reactive surrogates over held-out environments.
- T09.F06 applies the versioned T01.F05 protocol and reports its conclusive, negative, or inconclusive result; it must not redefine comparator access, search budget, fidelity, or acceptance margins after seeing evolved specimens.
- Final cognition evidence includes the T07 spatial-temporal signal medium and T03 computational maintenance costs through explicit dependencies; signal activity or extra computation without causal adaptive benefit does not count.
- Do not add a speculative replacement brain within qualification or repair features. If corrected execution and local mutation still expose a measured gap, add one bounded comparison feature with matched observations, actions, task exposure, and compute accounting before choosing a replacement.
- Research basis reviewed 2026-09-02: [Evolution of Integrated Causal Structures in Animats](https://doi.org/10.1371/journal.pcbi.1003966) and [The Evolutionary Origin of Associative Learning](https://doi.org/10.1086/706252).
- Options considered were raw structural complexity, integrated-information measures, benchmark fitness, and causal multi-axis profiling. Use causal profiling because it is controller-architecture neutral and tests whether internal mechanisms matter to adaptive behavior.
- Architecture review 2026-09-04: [SignalGP module regulation](https://mmore500.com/pubs/lalejini2021tag) improved evolution on context-dependent tasks but could impede it when context was unnecessary; [recurrent Cartesian genetic programming](https://doi.org/10.1007/s10710-016-9276-6) supplies an alternative temporal representation. Extend Petri first because it already has shared memory, stateful graph nodes, plasticity, and structural mutation, and the audit identifies concrete execution concerns. A small recurrent-controller comparator or SignalGP-style module regulation remains conditional on a persistent measured gap; neither has demonstrated superiority in Petri. The [large-scale ecology paper](https://arxiv.org/html/2510.18221v1) uses memoryless MLPs, so its scale results do not qualify Petri's memory or learning.

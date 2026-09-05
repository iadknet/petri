# T08 — Evolvability and Heredity

**Status**: Planned
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Ensure useful behavior can be reached through viable heritable steps, retained
under mutation, expanded through gene duplication, and eventually exchanged
through compatibility-aware mating and recombination.

## Track Success Criteria

- [ ] Mutational-neighborhood checks quantify viability, behavioral novelty, robustness, and access to future phenotypes around representative genomes.
- [ ] Structural VM edits preserve references outside the intended change, and graph growth can introduce new computation without broadly rewiring existing behavior.
- [ ] Duplication and structural mutation preserve useful function often enough to provide measured evolutionary stepping stones.
- [ ] Learned-state inheritance follows surviving homologous nodes and edges, with explicit initialization for new or incompatible structure.
- [ ] Heritable mutation policies do not collapse mutation toward zero or remove access to adaptive novelty.
- [ ] The opt-in sexual mode stays disabled until its complete dual-parent and homologous-recombination contract is available, then exchanges homologous function without destructive arbitrary topology splicing.
- [ ] Controlled campaigns compare asexual and sexual treatments for adaptation, modularity, robustness, and diversity.

## Executable Features

- [ ] **T08.F01 — Mutational Neighborhood Assay** — Depends on: T01.F12
  - Goal: One step away. Measure function retention, novelty, and reproductive viability after individual mutations and matched-supply mutation bursts over short trajectories; observation only.
- [ ] **T08.F02 — Lineage Mutation-Effect Attribution** — Depends on: T08.F01
  - Goal: Which mutations mattered. Along a surviving lineage, which heritable changes altered behavior and which were silent.
- [ ] **T08.F03 — Neutral Network and Robustness Characterization** — Depends on: T08.F01, T08.F02
  - Goal: Slack in the genome. How much neutral structure absorbs mutation and keeps function intact, distinguished from unreachable bloat.
- [ ] **T08.F04 — Function-Preserving Duplication and Module Growth** — Depends on: T08.F01, T08.F10, T08.F11, T09.F10
  - Goal: Gene duplication. Copying a working module keeps it working, so complexity can grow by copy and divergence as in real genomes.
- [ ] **T08.F05 — Heritable Mutation Policy** — Depends on: T08.F02, T08.F03
  - Goal: Mutation rate evolves. Lineages inherit how much they mutate, bounded so it cannot collapse to zero.
- [ ] **T08.F06 — Homology and Mating Compatibility** — Depends on: T04.F05, T08.F04, T08.F05, T09.F08
  - Goal: Deferred until the foundation diagnostics are reviewed. Which genomes are similar enough to exchange parts, measured before any mating exists.
- [ ] **T08.F07 — Dormant Dual-Parent Mating Contract** — Depends on: T03.F07, T08.F06
  - Goal: Deferred heredity work. Two-parent reproduction semantics, present but disabled by default.
- [ ] **T08.F08 — Homologous Recombination and Sexual-Mode Enablement** — Depends on: T08.F07
  - Goal: Deferred heredity work. Turn on sex only after recombination provably exchanges homologous function.
- [ ] **T08.F09 — Heredity-Mode Comparison Confirmatory Campaign** — Depends on: T01.F09, T04.F07, T08.F05, T08.F08, T10.F08
  - Goal: Deferred proof phase. Replicated confirmatory runs comparing asexual and sexual populations.
- [ ] **T08.F10 — VM Structural Mutation Reference Integrity** — Depends on: T01.F12
  - Goal: Gene insertion and duplication. Adding, removing, or copying program instructions preserves surviving control-flow references outside the intended mutation.
- [ ] **T08.F11 — Localized Graph Growth** — Depends on: T08.F01
  - Goal: New neural connections. A new compute node can acquire inputs and join a working circuit through bounded local changes without scattering sensor edges across the existing controller.
- [ ] **T08.F12 — Learned-State Inheritance Integrity** — Depends on: T08.F04, T09.F11
  - Goal: Inherited neural adaptation. When learned weights are heritable, structural mutation preserves their association with surviving connections and initializes new connections explicitly.

## Notes for AI Agents

- Extend Petri's current mutation operators, parseability gate, reachability bias, and mutation observability. Parseability is not behavioral viability.
- T08.F01 runs in process on founders and genomes sampled from a goal-profile run, adding known temporal motifs when T09.F09 fixtures are available. Fixed sensor sequences measure action and state changes; paired short world trajectories measure survival and reproduction. A snapshot action match is not viability, and an operator-labeled semantic-change counter is not behavioral novelty. Predeclare samples, seeds, trajectory lengths, and compute limits. Compare individual operators and the current burst policy with isolated events at matched expected mutation supply; separate attempted events, applied edits, and observed behavioral changes. Include raw operand redraws versus single-field perturbations as diagnostic treatments. Do not change production probabilities in this measurement feature; propose a bounded tuning feature only if the evidence supports one. No specimen serialization or deferred campaign infrastructure is required.
- T08.F10 owns reference integrity for VM insertion, deletion, memory-motif insertion, and block/slice copying. The current remapped copy adds a positional delta to relative jumps, and insertions do not preserve crossed targets (`crates/v3-core/src/mutation/vm/operators.rs` and `runtime/vm.rs`). Its spec defines internal, external, deleted, and wrapped-target policies and uses property tests for surviving references plus behavioral fixtures. Correctness repairs do not wait for an emergence campaign; preserving references does not require every mutation to preserve behavior.
- T08.F11 uses the T08.F01 baseline to compare current graph growth with local growth at matched mutation opportunity. The current `add_compute_node` in `crates/v3-core/src/mutation/graph/operators.rs` adds edges for every existing sensor component across random existing surfaces. Bound the new node's integration and measure both retained behavior and subsequent activation; adding permanently disconnected code is not success.
- T08.F04 moves ahead of the longer T08.F02/F03 lineage and neutral-network characterization. Use T08.F01 neighborhoods and the corrected VM and graph semantics to qualify existing copy operators: preserve internal references, account for shared-memory and output interference, and demonstrate copy-and-divergence on short trajectories. Separate functional preservation from applied energy cost; structural disconnection alone does not establish neutrality.
- T08.F12 owns learned-weight correspondence through node and edge insertion, deletion, and copying in `simulation/actions/reproduction.rs` and `cgp_reproduction.rs`. Define which homologous weights survive and which reset; positional array indices alone are not homology. Cover ordinary and Lamarckian offspring without enabling new inheritance modes or changing the established reset of eligibility traces.
- Sexual reproduction is committed as an opt-in experimental treatment, not as the production default and not as an assumed improvement. Asexual reproduction remains the control, and T08.F09 may truthfully find no benefit.
- T08.F06 through T08.F09 remain later work: do not plan their specs until T09.F08's diagnostic outcomes have been reviewed and T08.F01 through T08.F05 have produced the required heredity evidence. A positive shared-memory sensitivity reading alone is not clearance for mating work.
- T08.F07 may add deterministic mate intent, pairing, contention, contribution, target, and draft semantics, but the creature-facing sexual mode remains unavailable by default. T08.F08 enables it only after recombination, viability, and inheritance tests pass.
- T08.F07 and T08.F08 explicitly supersede the one-parent portions of `docs/reference/v3-reproduction-spec.md`, `docs/reference/v3-creature-identity-spec.md`, `crates/v3-core/src/contracts/actions.rs` (`WorldAction::Reproduce { direction, energy_transfer }`), and `crates/v3-core/src/simulation/actions/reproduction.rs` (`apply_reproduce`). Their specs must define two-parent energy and reserve contributions, spawn targeting, turn-order contention, generation and ancestry, lineage and kin identity, shared memory, phenotype, graph state, learned weights, mutation ordering, and rejection telemetry while preserving the asexual path.
- Research basis reviewed 2026-09-02: [The evolutionary origin of complex features](https://doi.org/10.1038/nature01568), [Sexual reproduction reshapes the genetic architecture of digital organisms](https://doi.org/10.1098/rspb.2005.3338), and [Homology and linkage in crossover for variable-length genomes](https://doi.org/10.1371/journal.pone.0209712).
- Foundation review 2026-09-04: [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) supports incremental growth and protection of new structure in its tested setting; [The Evolutionary Origin of Associative Learning](https://www.journals.uchicago.edu/doi/full/10.1086/706252) reports learning assembled from earlier behavior and latent components. Extend Petri's mutation and copy machinery before replacing its evolutionary system. The alternatives are immediate rate tuning and a new controller representation; neither addresses the observed reference-integrity defects. Population-level benefits of local growth and burst changes remain hypotheses for the bounded comparisons, not promised outcomes. Do not import global fitness sharing or direct complexity rewards into ecological selection.

| Option | Petri fit and evidence | Decision / exit criterion |
| --- | --- | --- |
| Mutation tuning without sex | Lowest contract risk and remains the required control, but cannot test exchange between lineages. | Retain as control, not as a substitute for the opt-in treatment. |
| Relative-position or arbitrary topology crossover | Demonstrated in simpler linear digital genomes, but Petri has variable nested VM, graph, and mesh structures. | Reject as the default because it does not establish functional homology. |
| Compatibility plus homology-aware module exchange | Matches Petri's topology and lets exchange quality be measured before evolutionary claims. | Adopt provisionally. T08.F06 must compare candidate mappings on homology, linkage, child viability, function retention, exchange rate, and cost; if none beats asexual mutation under the predeclared safety floor, block enablement rather than splice arbitrarily. |

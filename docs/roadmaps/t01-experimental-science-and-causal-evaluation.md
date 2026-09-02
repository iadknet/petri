# T01 — Experimental Science and Causal Evaluation

**Status**: Planned
**Last updated**: 2026-09-02
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give Petri falsifiable, reproducible measures of ecological diversity,
behavioral diversity, cognitive complexity, adaptive novelty, and evolvability
that distinguish meaningful outcomes from genome growth, cosmetic variation,
or transient population noise.

## Track Success Criteria

- [ ] Every program-level outcome has a versioned definition, uncertainty treatment, and predeclared acceptance threshold.
- [ ] Reference reactive, stateful, learning-dependent, and socially contingent controllers produce the expected causal-evaluation profiles.
- [ ] Ecological strategies are distinguished by causal resource and interaction use, not genotype or phenotype distance alone.
- [ ] Cognitive claims report a profile of effective controller size, temporal dependence, learning dependence, information integration, social contingency, and transfer breadth.
- [ ] Baseline long-running experiments quantify current collapse, persistence, novelty, and cognitive dependence across independent replicates.
- [ ] A result-blind confirmatory protocol freezes hypotheses, effect thresholds, uncertainty, replication, exposure, exclusions, and stopping rules before confirmatory runs begin.

## Executable Features

- [ ] **T01.F01 — Outcome and Evidence Contract** — Depends on: None
- [ ] **T01.F02 — Reference Controller and Behavior Fixtures** — Depends on: T01.F01
- [ ] **T01.F03 — Population, Specimen, and Lineage Sampling Model** — Depends on: T01.F01, T10.F03
- [ ] **T01.F04 — Behavioral Strategy Descriptors** — Depends on: T01.F03
- [ ] **T01.F05 — Cognitive Comparator and Causal-Assay Protocol** — Depends on: T01.F02, T10.F05
- [ ] **T01.F06 — Ecological Diversity and Persistence Measures** — Depends on: T01.F03, T01.F04
- [ ] **T01.F07 — Adaptive Novelty and Change-Potential Measures** — Depends on: T01.F03, T01.F04
- [ ] **T01.F08 — Baseline Collapse and Ablation Characterization** — Depends on: T01.F05, T01.F06, T01.F07, T10.F08
- [ ] **T01.F09 — Confirmatory Protocol and Threshold Freeze** — Depends on: T01.F08, T10.F07

## Notes for AI Agents

- Extend Petri's applied-state telemetry and current reachability-aware functional-complexity analysis; do not treat either as sufficient evidence by itself.
- The cognitive profile is multi-axis rather than one weighted score. Report effect sizes and uncertainty for memory reset or scrambling, plasticity freeze, sensor lesions, live-versus-replayed agents, held-out transfer, and behavior-preserving controller minimization.
- T01.F05 owns the versioned comparator protocol and validates it against fixtures. It must give original and surrogate controllers observation/action parity, predeclare search and training budgets, define behavior-fidelity tolerance and held-out adaptive-performance margins, and return `Inconclusive` when the bounded search cannot establish equivalence or separation. T09 applies this protocol to evolved specimens; it does not define a second minimization method.
- T01 owns sampling and statistical policy, estimands, acceptance rules, and domain-neutral analysis definitions. T10 owns capture, serialization, replay mechanics, scheduling, provenance, and packaging; domain tracks own intervention semantics and conclusions.
- Archive and assay the inherited genome, newborn state, and mature learned state separately. Petri inherits shared memory while learned graph runtime normally resets at birth.
- Measures must be validated against constructed reference cases before they evaluate evolved organisms.
- T01.F08 is characterization evidence only. T01.F09 must seal a versioned protocol and its hashes before confirmatory data collection; an independent scientific reviewer, not the implementer, approves it using the predeclared smallest-effect, power or precision, and control-calibration rules. An unresolved scientific choice is a concrete blocker rather than permission to tune after seeing outcomes.
- Research basis reviewed 2026-09-02: [MODES Toolbox](https://doi.org/10.1162/artl_a_00280), [Evolution of Integrated Causal Structures in Animats](https://doi.org/10.1371/journal.pcbi.1003966), [Simplification of genetic programs: a literature survey](https://doi.org/10.1007/s10618-022-00830-7), [Registered Reports](https://www.cos.io/initiatives/registered-reports), and [ADEMP-PreReg for simulation studies](https://doi.org/10.1037/met0000695).
- Existing options considered were raw genome size, reachable structural size, information-theoretic brain scores, and causal behavioral assays. Use reachable size as one structural axis and causal assays as the primary evidence because architecture-specific or syntactic measures can misclassify bloat.

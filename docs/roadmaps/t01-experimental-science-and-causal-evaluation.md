# T01 — Experimental Science and Causal Evaluation

**Status**: In Progress
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give Petri falsifiable, reproducible measures of ecological diversity,
behavioral diversity, cognitive complexity, adaptive novelty, and evolvability
that distinguish meaningful outcomes from genome growth, cosmetic variation,
or transient population noise.

## Track Success Criteria

- [ ] A dependency-free baseline records the smallest persistent world, a bounded persistence gate, and an evolutionary-activity profile against a neutral shadow model before assay features are planned.
- [ ] Every feature closed after T10.F10 records a dated fixture-class reading of the currently defined ecosystem and cognition indicators, and every T01 measure feature wires its indicator into that report, so program direction is visible between campaigns.
- [ ] Every program-level outcome has a versioned definition, uncertainty treatment, and predeclared acceptance threshold.
- [ ] Reference reactive, stateful, learning-dependent, and socially contingent controllers produce the expected causal-evaluation profiles.
- [ ] Ecological strategies are distinguished by causal resource and interaction use, not genotype or phenotype distance alone.
- [ ] Cognitive claims report a profile of effective controller size, temporal dependence, learning dependence, information integration, social contingency, and transfer breadth.
- [ ] Baseline long-running experiments quantify current collapse, persistence, novelty, and cognitive dependence across independent replicates.
- [ ] A result-blind confirmatory protocol freezes hypotheses, effect thresholds, uncertainty, replication, exposure, exclusions, and stopping rules before confirmatory runs begin.
- [ ] A single progress ledger preserves dated evaluation snapshots and makes improvement, regression, and inconclusive evidence visible for every program-level outcome.

## Executable Features

- [ ] **T01.F01 — Outcome and Evidence Contract** — Depends on: None
- [ ] **T01.F02 — Reference Controller and Behavior Fixtures** — Depends on: T01.F01
- [ ] **T01.F03 — Population, Specimen, and Lineage Sampling Model** — Depends on: T01.F01, T10.F03
- [ ] **T01.F04 — Behavioral Strategy Descriptors** — Depends on: T01.F03
- [ ] **T01.F05 — Cognitive Comparator and Causal-Assay Protocol** — Depends on: T01.F02, T10.F05
- [ ] **T01.F06 — Ecological Diversity and Persistence Measures** — Depends on: T01.F03, T01.F04
- [ ] **T01.F07 — Adaptive Novelty and Change-Potential Measures** — Depends on: T01.F03, T01.F04, T01.F13
- [ ] **T01.F08 — Baseline Collapse and Ablation Characterization** — Depends on: T01.F05, T01.F06, T01.F07, T01.F13, T10.F08
- [ ] **T01.F09 — Confirmatory Protocol and Threshold Freeze** — Depends on: T01.F08, T10.F07
- [ ] **T01.F10 — Program Progress Ledger** — Depends on: T01.F09, T10.F08
- [ ] **T01.F11 — Baseline Persistence Characterization** — Depends on: T10.F10
- [ ] **T01.F12 — Minimum Persistent World and Persistence Gate** — Depends on: T01.F11, T10.F09
- [ ] **T01.F13 — Evolutionary Activity and Shadow-Model Baseline** — Depends on: T01.F01, T01.F12

## Notes for AI Agents

- T01.F11 runs entirely through the T10.F10 sweep profile so every number it reports is regenerable by one command; it adds no ad hoc scripts. It uses production defaults, sweeps world size (at least 128, 256, 512, and the 1600 default), density-matched founder counts, and at least three seeds for 2,000 to 5,000 ticks, and records extinction tick, peak and plateau population, births per 100 ticks, and mean energy. It may add sampled fields to the bench report but must not change simulation behavior. On 2026-09-03 every world at or below 512-by-512 went extinct by tick 300 while the default world plateaued near 5,000 creatures; T01.F11 must reproduce or supersede that result.
- T01.F12 chooses the smallest configuration that persists for a predeclared horizon over predeclared seeds and records it as the standard replicate world. It may change production economics only if no small world persists, and only through the viability suite. It adds a bounded persistence smoke to the fast test set, sized in seconds rather than minutes, and switches the T10.F10 gate profile to the standard replicate world, which starts a new report series and re-pins the epoch baseline in the same commit. The existing 20-tick viability gate is not persistence evidence and must not be cited as such.
- T01.F13 adopts evolutionary activity statistics (Bedau and Packard) with a neutral shadow model, following Channon's 2024 Tokyo Type 1 procedure: report total cumulative activity, median normalized cumulative activity, and new activity for the real run against a shadow run in which births and deaths occur at the same rate without selection. The feature spec declares the heritable component definition (for example reachable-structure signatures or operator-level genome features) before any run. Its output is a per-run summary that T01.F07 extends and the ledger cites, and it declares the minimum horizon below which the T10.F10 report records its statistics as `Undefined`. A result showing no activity above the shadow model is a program-level stop signal, not a metric to tune.
- Extend Petri's applied-state telemetry and current reachability-aware functional-complexity analysis; do not treat either as sufficient evidence by itself.
- The cognitive profile is multi-axis rather than one weighted score. Report effect sizes and uncertainty for memory reset or scrambling, plasticity freeze, sensor lesions, live-versus-replayed agents, held-out transfer, and behavior-preserving controller minimization.
- T01.F05 owns the versioned comparator protocol and validates it against fixtures. It must give original and surrogate controllers observation/action parity, predeclare search and training budgets, define behavior-fidelity tolerance and held-out adaptive-performance margins, and return `Inconclusive` when the bounded search cannot establish equivalence or separation. T09 applies this protocol to evolved specimens; it does not define a second minimization method.
- T01 owns sampling and statistical policy, estimands, acceptance rules, and domain-neutral analysis definitions. T10 owns capture, serialization, replay mechanics, scheduling, provenance, and packaging; domain tracks own intervention semantics and conclusions.
- Archive and assay the inherited genome, newborn state, and mature learned state separately. Petri inherits shared memory while learned graph runtime normally resets at birth.
- Measures must be validated against constructed reference cases before they evaluate evolved organisms.
- T01.F08 is characterization evidence only. T01.F09 must seal a versioned protocol and its hashes before confirmatory data collection; an independent scientific reviewer, not the implementer, approves it using the predeclared smallest-effect, power or precision, and control-calibration rules. An unresolved scientific choice is a concrete blocker rather than permission to tune after seeing outcomes.
- The independent reviewer is defined concretely so T01.F09 cannot block forever: a separate agent session (a different model or a fresh session with no access to the implementing session's context) that receives only the sealed protocol document and its committed hash, with no result data. The user may substitute a human reviewer. The approval record names the reviewer session, the protocol hash, and the date, and is committed before any confirmatory run starts.
- T01.F10 links the per-feature reports produced by T10.F10 rather than restating them, and remains the only place where campaign-grade evidence advances an outcome. T01.F10 creates `docs/progress.md` and appends one compact snapshot after each designated evaluation campaign. For every program-level outcome, record the baseline, current estimate and uncertainty, direction (`Improving`, `Unchanged`, `Regressing`, or `Inconclusive`), evidence class, date, code revision, metric or protocol version, and supporting T10 result bundle. Preserve null results and regressions. Characterization evidence may advance track success criteria; only confirmatory evidence may advance a master final success criterion. When a metric definition changes, start a new clearly labeled series rather than implying direct comparability.
- Keep the ledger as a plain Markdown document backed by compact result summaries. Do not add a dashboard, database, service, third-party tracking dependency, or composite success score.
- Research basis reviewed 2026-09-02: [MODES Toolbox](https://doi.org/10.1162/artl_a_00280), [Evolution of Integrated Causal Structures in Animats](https://doi.org/10.1371/journal.pcbi.1003966), [Simplification of genetic programs: a literature survey](https://doi.org/10.1007/s10618-022-00830-7), [Registered Reports](https://www.cos.io/initiatives/registered-reports), and [ADEMP-PreReg for simulation studies](https://doi.org/10.1037/met0000695). Added 2026-09-03 for T01.F13: Channon, "A Procedure for Testing for Tokyo Type 1 Open-Ended Evolution", Artificial Life 30(3), in the [2024 open-ended evolution special issue](https://doi.org/10.1162/artl_e_00445), and Bedau and Packard, "Measurement of evolutionary activity, teleology, and life" (Artificial Life II, 1992).
- Existing options considered were raw genome size, reachable structural size, information-theoretic brain scores, and causal behavioral assays. Use reachable size as one structural axis and causal assays as the primary evidence because architecture-specific or syntactic measures can misclassify bloat.

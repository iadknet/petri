# Diverse Ecosystems and Complex Cognition

**Status**: Planning
**Last updated**: 2026-09-03

## Success Definition

Petri succeeds when replicated, long-running evolutionary experiments sustain
multiple causally distinct ecological strategies and persistent lineages evolve
behavior that depends on temporally extended state, learning, prediction,
information integration, or reciprocal interaction. The demonstrated behavior
must remain adaptive in held-out conditions and must not be reproducible by a
substantially simpler reactive controller.

## Track Roadmaps

- [ ] **T01 — Experimental Science and Causal Evaluation** — [Roadmap](roadmaps/t01-experimental-science-and-causal-evaluation.md) — Depends on: None
- [ ] **T02 — Environmental Dynamics** — [Roadmap](roadmaps/t02-environmental-dynamics.md) — Depends on: None
- [ ] **T03 — Functional Traits and Metabolism** — [Roadmap](roadmaps/t03-functional-traits-and-metabolism.md) — Depends on: None
- [ ] **T04 — Population Structure and Diversification** — [Roadmap](roadmaps/t04-population-structure-and-diversification.md) — Depends on: None
- [ ] **T05 — Biotic Interactions and Coevolution** — [Roadmap](roadmaps/t05-biotic-interactions-and-coevolution.md) — Depends on: None
- [ ] **T06 — Niche Construction and Ecological Inheritance** — [Roadmap](roadmaps/t06-niche-construction-and-ecological-inheritance.md) — Depends on: None
- [ ] **T07 — Communication and Social Evolution** — [Roadmap](roadmaps/t07-communication-and-social-evolution.md) — Depends on: None
- [ ] **T08 — Evolvability and Heredity** — [Roadmap](roadmaps/t08-evolvability-and-heredity.md) — Depends on: None
- [ ] **T09 — Cognition and Learning** — [Roadmap](roadmaps/t09-cognition-and-learning.md) — Depends on: None
- [ ] **T10 — Evolutionary Scale and Experiment Infrastructure** — [Roadmap](roadmaps/t10-evolutionary-scale-and-experiment-infrastructure.md) — Depends on: None

## Final Success Criteria

- [ ] Independent long-running treatments meet the evolutionary-opportunity, replication, persistence, and stopping thresholds fixed and independently approved by T01 before confirmatory runs begin.
- [ ] Multiple ecological strategies exceed the T01 coexistence and causal-distinctness thresholds across independent replicates rather than appearing only as transient genome variation.
- [ ] Persistent evolved lineages exceed the T01 cognitive-complexity thresholds through controlled ablations, reactive-controller comparisons, and held-out transfer assays.
- [ ] Adaptive novelty continues through the declared observation window without relying on direct rewards for genome size, controller size, diversity, novelty, or cognitive-complexity metrics.
- [ ] Archived manifests, checkpoints, specimens, and result bundles reproduce every program-level claim within declared tolerances.
- [ ] A single version-controlled progress ledger shows how every program-level outcome has changed from baseline and links each entry to its supporting result bundle.
- [ ] The smallest persistent world, its throughput budget, and its evolutionary-activity baseline are recorded in the progress ledger before any confirmatory campaign begins.

## Notes for AI Agents

- Track IDs organize durable areas of responsibility; they are not a ten-phase implementation order. The feature dependency graph is the authoritative execution order.
- First slice (added 2026-09-03 after a baseline measurement): the following eleven features form a closed dependency set, listed in a valid execution order; complete them before planning any feature outside the set: T10.F10, T01.F11, T10.F09, T01.F12, T01.F01, T01.F13, T10.F01, T10.F03, T02.F01, T02.F02, T09.F08. T10.F10 is first so that every later feature, including the baseline sweep itself, is measured by the same deterministic checked-in script. The slice ends with a go/no-go on the existing controller substrate. If T01.F13 shows no adaptive activity above the shadow model or T09.F08 finds no memory-dependent lineage, stop and add the smallest substrate or economics feature that addresses the measured gap before continuing.
- Baseline measured 2026-09-03 at commit 98f09322 with production defaults through `v3-cli run`: 256-by-256 and 512-by-512 worlds with density-matched founders went extinct by tick 180 to 300 across seeds, including with full initial food coverage; the default 1600-by-1600 world with 10,000 founders peaked near 35,000 creatures, fell to about 5,000 by tick 500, and held there through tick 1000 with roughly 3,500 births per 100 ticks; the default world ran at about 3 ticks per second single-threaded. The 20-tick viability gate does not detect any of this. Re-measure before citing these numbers; T01.F11 supersedes them.
- Characterization evidence may advance track success criteria. Only confirmatory evidence advances the final success criteria above.
- Every feature closed after T10.F10 stores a compact benchmark report with a compute-performance half and a goal-indicator half, both measured on the same cheap deterministic benchmark run. Any feature that introduces a diversity or cognition measure wires its indicator into that report as part of the same feature. The performance half gates that feature: a severe compute regression against either the previous closed feature or the pinned epoch baseline blocks closure unless the feature predeclared and justified the cost, in which case its closing commit re-pins the epoch baseline. The goal half is fixture-class trend data only and never advances a success criterion. Neither half is a dashboard, a database, or a composite score.
- T08.F06 through T08.F09 (mating, recombination, and the heredity-mode campaign) are deferrable until the first-slice go/no-go passes; their dependencies already place them last, and no earlier feature may depend on them.
- Long evolution campaigns establish emergence and persistence. Short paired replays of archived organisms establish causal mechanisms.
- Every experiment is labeled fixture, characterization, or confirmatory in its manifest. Fixture and characterization runs may inform design but cannot support final program claims; every confirmatory campaign depends on T01.F09 and uses its result-blind frozen protocol.
- A tick count alone is not evolutionary exposure. Experiment manifests must bound births, mutation opportunities, lineage or generation depth, persistence windows, ticks, and wall-clock time as appropriate.
- Confirmatory campaigns require an explicit repository-external artifact root and manifest-declared wall-clock, concurrency, and storage caps. Exceeding those caps requires renewed user authorization.
- Energy is ecological accounting, not a standalone track or a direct complexity reward. Every new capacity must have an applied cost or tradeoff owned by the feature that introduces it.
- Qualify the existing VM, graph, shared-memory, plasticity, and reward-modulation substrate before proposing a replacement cognition engine.
- Options considered were a richer fixed environment, an external curriculum, direct novelty or complexity rewards, and endogenous ecological feedback. The roadmap uses bounded fixed environments as assays and endogenous feedback as the long-term pressure because direct proxies can reward bloat or indefinite but trivial change.
- Research basis reviewed 2026-09-02: [MODES Toolbox](https://doi.org/10.1162/artl_a_00280), [Open-Endedness for the Sake of Open-Endedness](https://doi.org/10.1162/artl_a_00289), [The evolutionary origin of complex features](https://doi.org/10.1038/nature01568), and [Coevolution drives the emergence of complex traits and promotes evolvability](https://doi.org/10.1371/journal.pbio.1002023). Added 2026-09-03: Channon, "A Procedure for Testing for Tokyo Type 1 Open-Ended Evolution", Artificial Life 30(3), in the [2024 open-ended evolution special issue](https://doi.org/10.1162/artl_e_00445), and [The Emergence of Complex Behavior in Large-Scale Ecological Environments](https://arxiv.org/abs/2510.18221), whose 60,000-agent runs at roughly 190 steps per second and 2-million-step horizons set the scale reference that T10.F09 budgets against.
- Platforms considered as replacements on 2026-09-03 were MABE2 (no release, linear Markov-brain genomes), Avida (linear self-replicating programs without spatial energy ecology), and JAX grid-world simulators (GPU-batched fixed-topology networks). None replaces Petri's evolvable topology; the roadmap borrows their evaluation methods instead.
- Exact numerical thresholds and replicate counts remain deliberately unset until T01 and T10 produce pilot variance and throughput evidence. T01.F09 freezes them from predeclared smallest-effect and precision rules and requires result-blind independent approval before any confirmatory outcome is generated.
- T01.F10 keeps program tracking deliberately simple: one Markdown ledger in Git, backed by T10 result bundles, with no dashboard, database, service, or composite success score.

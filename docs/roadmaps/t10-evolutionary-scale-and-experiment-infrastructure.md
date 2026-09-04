# T10 — Evolutionary Scale and Experiment Infrastructure

**Status**: In Progress
**Last updated**: 2026-09-04
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Run, resume, compare, and audit long evolutionary campaigns with enough
throughput and retained evidence to support causal ecological and cognitive
claims without placing multi-hour or multi-day experiments in the normal build
gate.

## Track Success Criteria

- [ ] A versioned manifest identifies code, configuration, seeds, treatments, exposure targets, sampling, stopping rules, and artifact schemas for every campaign.
- [ ] Exact checkpoints resume world, organisms, genomes, learned state, random state, counters, and experiment identity without changing the continued trajectory.
- [ ] Batch treatments run independent replicates, recover from interruption, and report partial or failed runs truthfully.
- [ ] Compact longitudinal artifacts retain sufficient ancestry, specimen, ecology, behavior, and cognition evidence without recording every tick trace.
- [ ] A throughput baseline and per-core budget at the standard replicate world are recorded before campaign orchestration is designed.
- [ ] Every feature closed after the harness exists carries a stored benchmark report whose deterministic block is byte-identical on re-run, and a severe compute regression is detected and surfaced at that feature's closure rather than later.
- [ ] Throughput budgets are expressed in evolutionary opportunity as well as wall-clock cost, and long campaigns remain outside `make check`.
- [ ] Large evidence survives feature-worktree cleanup in an explicit repository-external artifact root with verified identity, atomic publication, retention, and safe garbage collection.

## Executable Features

- [ ] **T10.F01 — Experiment Manifests, Artifact Custody, and Identity** — Depends on: None
- [ ] **T10.F02 — Exact Simulation Checkpoint and Resume** — Depends on: T10.F01
- [ ] **T10.F03 — Typed Artifact Capture and Serialization** — Depends on: T10.F01
- [ ] **T10.F04 — Treatment Matrix and Replicate Orchestration** — Depends on: T10.F02, T10.F03
- [ ] **T10.F05 — Counterfactual Replay Mechanics and Intervention Hooks** — Depends on: T01.F02, T10.F02, T10.F03
- [ ] **T10.F06 — Campaign Monitoring, Stopping, and Recovery** — Depends on: T10.F04
- [ ] **T10.F07 — Evolutionary Throughput Benchmark and Budget** — Depends on: T10.F03, T10.F06, T10.F09
- [ ] **T10.F08 — Provenance and Immutable Result-Bundle Packaging** — Depends on: T10.F05, T10.F06, T10.F07
- [ ] **T10.F09 — Throughput Baseline and Profiling Budget** — Depends on: T01.F11
- [x] **T10.F10 — Deterministic Benchmark Harness and Per-Feature Report** — Depends on: None

## Notes for AI Agents

- Extend the headless CLI and core simulation interfaces rather than driving experiments through the interactive server or frontend.
- The current CLI supports fixed-seed runs and sparse NDJSON tick samples but lacks checkpointing, treatment orchestration, specimen retention, and causal replay.
- T10.F09 measures ticks per second and per-tick cost at every T01.F11 sweep point through the T10.F10 sweep profile, profiles the tick hot paths, and records a per-core budget expressed in births and ticks per wall-clock hour. It decides, with numbers, whether core parallelism is required before any campaign; if so it adds one bounded dependency-linked feature rather than implementing parallelism itself. On 2026-09-03 the default world ran at about 3 ticks per second and a 512-by-512 world at about 35 ticks per second, single-threaded; the core crate does not use the `rayon` workspace dependency. The scale reference in the master roadmap is roughly 190 steps per second with 60,000 agents on a GPU.
- T10.F10 is the first feature of the program and has no dependencies, because a deterministic benchmark needs a fixed seed and horizon, not a persisting population. It delivers one checked-in `v3-cli bench` subcommand and one `make` target with two profiles: a gate profile with a fixed small world, predeclared seed set, and horizon sized in seconds that every later feature runs, and a sweep profile that takes world size, founder count, seeds, and horizon as parameters for characterization runs such as T01.F11. Both emit the same JSON schema: one compact report per feature under `docs/progress/features/tNN-fNN-<slug>.json`, compared against two stored references, the previous closed feature's report for per-feature regression and a pinned epoch baseline for cumulative drift. T10.F10 creates that directory; T01.F10 separately creates `docs/progress.md`.
- Determinism contract: the same commit and the same inputs must produce a byte-identical deterministic block. Keys are sorted, floats use a fixed format, and no timestamp, hostname, or duration appears inside that block; host identity, wall-clock, and the report date live in one separate clearly labeled block. A fast test runs the gate profile twice and diffs the deterministic block, and it is part of `make check`.
- The report schema defines both halves on day one. The performance half carries the work counters below. The goal half carries the complete indicator schema for ecosystem diversity and cognitive dependence, with every indicator present and recorded as `Undefined`, never zero, until an owning measure exists. At the time this feature is written the only populated indicators are population persistence, births per 100 ticks, and the reachable-structure size distribution from the existing functional-complexity analysis. Any later feature that introduces a diversity or cognition measure wires its indicator into the report as part of that feature; the indicator otherwise stays `Undefined` and that feature's spec must say so. The T01.F13 activity statistics are populated only when the profile's horizon meets a minimum T01.F13 declares, because the shadow model needs a parallel neutral run and activity over a few hundred ticks is not interpretable.
- Goal-half readings are fixture-class evidence and a trend series and tripwire only. A short benchmark run cannot establish diversity or cognition, so a reading must never advance a track or final success criterion, and no feature may optimize against one. Their purpose is to make an unnoticed drift or collapse visible at the feature that caused it. When an indicator definition changes, or when T01.F12 switches the gate profile to the standard replicate world, start a new labeled series and re-pin the epoch baseline in that same commit, exactly as the ledger requires.
- Report deterministic work counters as the primary signal, because they are exactly reproducible for a seed and are independent of the host machine: ticks, creature-ticks, mesh hops, VM steps, graph relaxation iterations, plasticity updates, actions applied, and births. Most of these do not exist yet: `SimStats` carries only reproduction, mutation, and predation counters, and the per-creature cost report carries two floating-point costs rather than counts. T10.F10 therefore adds integer work counters to `SimStats`, derived from applied execution in keeping with the runtime truthfulness invariant, before it can report them. Report wall-clock as a secondary signal tagged with host identity, since it is noisy and not comparable across machines; do not measure peak memory in this feature. Normalize both by creature-tick, never by tick alone, because population varies between runs and a smaller population would otherwise read as a speedup.
- Default regression thresholds, revisable by this feature with evidence, applied against both references: flag when deterministic work per creature-tick rises more than 10 percent, or when wall-clock per creature-tick rises more than 25 percent on the same host. Treat more than 50 percent work growth or a doubling of wall-clock time as severe. A severe regression is a P1 finding for the owning feature and blocks its closure unless that feature's spec predeclared the cost and justified it; a justified cost re-pins the epoch baseline in the same commit that closes the feature, so no later feature inherits the block. Compute cost is not the same thing as the in-simulation energy tradeoff every new capacity owes; a feature may owe both.
- Only deterministic counters may fail a test. The counter comparison runs inside the fast test set and fails on a severe unjustified regression. Wall-clock is recorded and reported in the JSON but is never asserted, because stored baselines come from one machine and any other host or CI runner would flake.
- Keep the harness outside long-campaign infrastructure. Reports are small enough for Git; nothing here belongs in the artifact root, and no dashboard, database, service, or composite score is permitted.
- The harness applies to every feature closed after it, which is every other feature in the program. The 2026-09-03 measurements recorded in this roadmap are superseded by the first sweep-profile output and must not be cited once it exists.
- T10.F01 and T10.F03 are part of the first slice. Their first-slice scope is the minimum a T09.F08 specimen capture needs: a manifest identifying code revision, config, seed, and horizon, plus genome and creature-state serialization. Checkpoint and resume (T10.F02) may wait for the first campaign that resumes, because seeded runs are already deterministic.
- Separate fast correctness and viability checks, bounded feature characterization, and confirmatory long campaigns. Only the first category belongs in `make check`.
- Define exposure using births, applied mutations, lineage or generation depth, and ecological turnover where relevant; ticks and wall time are safety bounds, not universal scientific units.
- T10 owns generic capture and serialization schemas, replay mechanics, typed intervention extension points, scheduling, provenance, and packaging. T01 owns sampling/statistical policy and domain-neutral measures; each domain track owns the meaning and analysis of its interventions. T10.F03 and T10.F05 must expose typed extension seams without embedding ecology or cognition conclusions.
- T10.F01 requires an explicit absolute artifact root outside every Git checkout or worktree. Git stores schemas, manifests, protocol hashes, content hashes, compact summaries, and claim-to-artifact indexes; the artifact root stores checkpoints, specimens, longitudinal samples, and raw replay outputs. Feature-worktree cleanup must never delete the artifact root.
- Publish an artifact bundle by writing and verifying it in a temporary sibling directory, recording content hashes and the producing manifest, then atomically renaming it to its immutable content-addressed identity. Retention pins every artifact reachable from a frozen protocol or result bundle; garbage collection is dry-run by default and may remove only unreferenced, unpinned content within the declared storage cap.
- Every campaign manifest declares per-run and aggregate wall-clock limits, concurrency or CPU limits, storage limits, interruption behavior, and exposure/stopping targets. Crossing a limit stops truthfully and requires renewed user authorization rather than silently expanding resource use.
- Research basis reviewed 2026-09-02: [Microbial Experimental Evolution](https://doi.org/10.1128/AEM.01992-18), [MODES Toolbox](https://doi.org/10.1162/artl_a_00280), [Nextflow execution and provenance documentation](https://docs.seqera.io/nextflow/cli), and [Snakemake execution documentation](https://snakemake.readthedocs.io/en/stable/executing/cli.html).

| Option | Petri fit and evidence | Decision / exit criterion |
| --- | --- | --- |
| POSIX shell loops around `v3-cli` | Dependency-free and transparent, but weak for structured recovery, identity, and custody. | Use only for fixtures; reject for long campaigns. |
| Nextflow | Strong resume, executors, logs, and evolving lineage support; adds a JVM/Groovy workflow layer and its data-lineage interface is still marked experimental. | Scale-out candidate if non-local executors or multi-host scheduling become requirements. |
| Snakemake | Strong DAG, resource, executor-plugin, cache, and provenance features; adds Python and a second workflow model. | Strongest alternative for a bounded orchestration PoC. |
| Thin Petri-native experiment layer | Reuses Rust/CLI contracts and can own exact simulator checkpoint and intervention semantics with the least initial integration surface. | Adopt for local campaigns. In T10.F04, run the same restartable 2-by-2 treatment fixture through the native design and Snakemake; adopt an engine instead if native recovery/provenance requires recreating a general DAG, remote executor, or storage-plugin system. |

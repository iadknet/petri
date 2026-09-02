# T10 — Evolutionary Scale and Experiment Infrastructure

**Status**: Planned
**Last updated**: 2026-09-02
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
- [ ] Throughput budgets are expressed in evolutionary opportunity as well as wall-clock cost, and long campaigns remain outside `make check`.
- [ ] Large evidence survives feature-worktree cleanup in an explicit repository-external artifact root with verified identity, atomic publication, retention, and safe garbage collection.

## Executable Features

- [ ] **T10.F01 — Experiment Manifests, Artifact Custody, and Identity** — Depends on: None
- [ ] **T10.F02 — Exact Simulation Checkpoint and Resume** — Depends on: T10.F01
- [ ] **T10.F03 — Typed Artifact Capture and Serialization** — Depends on: T10.F01
- [ ] **T10.F04 — Treatment Matrix and Replicate Orchestration** — Depends on: T10.F02, T10.F03
- [ ] **T10.F05 — Counterfactual Replay Mechanics and Intervention Hooks** — Depends on: T01.F02, T10.F02, T10.F03
- [ ] **T10.F06 — Campaign Monitoring, Stopping, and Recovery** — Depends on: T10.F04
- [ ] **T10.F07 — Evolutionary Throughput Benchmark and Budget** — Depends on: T10.F03, T10.F06
- [ ] **T10.F08 — Provenance and Immutable Result-Bundle Packaging** — Depends on: T10.F05, T10.F06, T10.F07

## Notes for AI Agents

- Extend the headless CLI and core simulation interfaces rather than driving experiments through the interactive server or frontend.
- The current CLI supports fixed-seed runs and sparse NDJSON tick samples but lacks checkpointing, treatment orchestration, specimen retention, and causal replay.
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

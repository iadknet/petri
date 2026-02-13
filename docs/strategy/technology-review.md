# Petri — Technology Stack Review (Rebaselined)

## Scope

This document reflects the **actual** repository state and near-term planned direction.
It replaces older assumptions that are no longer valid.

## Current Workspace Dependencies

Source of truth: workspace `Cargo.toml` and crate `Cargo.toml` files.

### Rust Workspace Dependencies (Current)

- `serde`
- `rand` (with `small_rng`)
- `slotmap`
- `anyhow`
- `axum`
- `tokio`
- `rmp-serde`
- `tracing`, `tracing-subscriber`
- `clap`
- `futures-util`
- `tower`, `tower-http`
- `serde_json`

### Crate-Level Reality (Current)

- `petri-graph`: `serde`, `rand`
- `petri-core`: `rand`, `serde`, `slotmap`, `petri-graph`
- `petri-server`: `axum`/`tokio` stack plus transport/logging utilities and `petri-core`
- `petri-cli`: `clap`, `petri-core`

## Removed/Rejected Historical Assumptions

The following assumptions should no longer appear in active architecture decisions unless intentionally reintroduced:

- `petri-genome` crate exists in workspace.
- `petgraph` is currently used as graph storage.
- `rayon` is currently active for simulation parallelism.

None of the above are true in the current repository state.

## Frontend Stack (Current)

- React + TypeScript + Vite
- MessagePack decode path in client protocol layer
- Canvas-based world rendering and inspector-centric workflows

## Cognition-First Refactor: Technology Impact

### Implemented Impact

- No mandatory new third-party dependencies are required for the refactor design itself.
- Primary changes are semantic and structural in existing crates (`petri-core`, `petri-graph`, server/web contract types).

### Implemented Runtime Semantics

- Energy-bounded internal think loop per tick.
- `halt` and `no_op` controller outputs.
- Single world interaction max per tick.
- Confidence introspection channels in controller inputs.

## Metrics and Performance Posture During Refactor

- Keep benchmark tooling (`stage1_benchmark`) active.
- Treat throughput as informational while post-refactor stabilization and profiling continue.
- Prefer correctness/observability gates first, then tighten performance thresholds afterward.

## Selection Criteria Going Forward

When adding/changing dependencies, require all of:
- direct need not solvable cleanly in current stack
- maintained crate/project with active releases
- clear boundary ownership (core vs transport vs tooling)
- testability and deterministic replay impact documented

## Short-Term Recommendations

1. Maintain dependency minimization during docs/rebaseline and semantics refactor phases.
2. Delay new infrastructure dependencies until current cognition behavior is profiled and bottlenecks are clear.
3. Keep architecture docs and roadmap synchronized to prevent future drift.

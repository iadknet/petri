# Architecture Lint Policy (v1)

This document defines the mechanical architecture checks enforced by:

- `scripts/check-architecture-harness.sh --mode warn`
- `scripts/check-architecture-harness.sh --mode strict`

## Scope

This policy covers core architecture guardrails only:

- crate dependency direction
- forbidden runtime deps/imports in pure crates
- `src/lib.rs` export-focus constraints
- production Rust file size thresholds
- test placement guardrails tied to `src/lib.rs`

Protocol-coupling heuristics are explicitly out of scope for v1.

## Rules

### 1. Crate dependency direction

Allowed workspace path dependencies:

- `petri-core` -> `petri-graph`
- `petri-server` -> `petri-core`
- `petri-cli` -> `petri-core`

Disallowed examples:

- `petri-graph` -> `petri-core`
- `petri-core` -> `petri-server`
- `petri-cli` -> `petri-server`

### 2. Forbidden runtime deps/imports in pure crates

In `crates/petri-core` and `crates/petri-graph`, these runtime/transport dependencies are forbidden:

- `axum`
- `tokio`
- `tower-http`
- `hyper`

The harness checks both manifest dependencies and source imports.

### 3. `src/lib.rs` export-focus guard

In `crates/*/src/lib.rs`:

- no `#[cfg(test)]`
- no inline implementation items (`fn`, `struct`, `enum`, `impl`)

`lib.rs` should remain an export-focused module surface (`mod` + `pub use`).

### 4. Production file size thresholds

Target set: `crates/*/src/**/*.rs`, excluding test-only files:

- `*/tests.rs`
- `*/test*.rs`

Thresholds:

- `> 400` lines: warning
- `> 600` lines: strict violation unless allowlisted in baseline TSV

### 5. Baseline semantics (v1)

Baseline file: `docs/standards/architecture-size-baseline.tsv`

For allowlisted oversized files:

- growth beyond baseline line count: warning in both modes
- strict mode does not fail solely due to allowlisted file growth in v1

## Rollout (completion-gate only)

- Phase 1: through February 27, 2026
  - run warn mode and report warnings
- Phase 2: starting February 28, 2026
  - run strict mode; strict violations block completion claims

No CI gate changes in v1.

## Out of Scope

- `cargo-deny` integration (explicitly deferred)
- protocol/API coupling checks between server and web
- auto-remediation

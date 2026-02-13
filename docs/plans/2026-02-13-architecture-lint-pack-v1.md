# Architecture Lint Pack v1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a shell-based architecture harness that mechanically enforces core structural constraints and integrate it into completion-gate docs.

**Scope:** Repository docs and harness scripts only. No runtime behavior, API, or wire-format changes.

**Docs Impact:**
- Add architecture lint standards docs and baseline TSV.
- Add architecture harness command to completion gate and operations docs.
- Add standards index link in docs index.

**Supersedes:** none

**Superseded-By:** none

## Tasks

1. Add `docs/standards/architecture-lint-policy.md` and `docs/standards/architecture-size-baseline.tsv`.
2. Implement `scripts/check-architecture-harness.sh` with `--mode warn|strict`.
3. Update `AGENTS.md`, `docs/operations/doc-hygiene.md`, and `docs/README.md`.
4. Execute positive and negative verification scenarios for the architecture harness.
5. Run completion-gate checks.

## Verification

- `scripts/check-architecture-harness.sh --mode warn`
- `scripts/check-architecture-harness.sh --mode strict`
- Negative scenario checks (dependency direction, forbidden deps/imports, lib.rs focus, oversize file, baseline growth)
- `scripts/check-doc-harness.sh --mode strict`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

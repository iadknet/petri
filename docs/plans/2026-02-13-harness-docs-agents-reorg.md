# Harness-Informed Docs + AGENTS Reorganization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reorganize docs and instruction surfaces so AGENTS files become map-like, canonical docs move under `docs/`, and mechanical harness checks enforce drift control.

**Scope:** Documentation and instruction-system reorganization only. No Rust/TypeScript behavior changes.

**Docs Impact:**
- Canonical strategy docs moved to `docs/strategy/*`
- Root compatibility stubs retained for roadmap/architecture/technology review
- Root + crate + web AGENTS surfaces refactored
- Added standards/operations docs and docs harness script

**Supersedes:** none

**Superseded-By:** none

## Tasks

1. Move canonical strategy docs under `docs/strategy/`.
2. Create root compatibility stubs for moved strategy docs.
3. Add `docs/README.md` index and standards/operations docs.
4. Refactor root `AGENTS.md` into a map-like contract.
5. Add crate-local AGENTS files and update `web/AGENTS.md` links.
6. Replace `CLAUDE.md` with strict AGENTS pointer adapter.
7. Add `docs/plans/README.md` metadata requirements.
8. Implement `scripts/check-doc-harness.sh` and verify warn/strict modes.

## Verification

- `scripts/check-doc-harness.sh --mode warn`
- `scripts/check-doc-harness.sh --mode strict`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

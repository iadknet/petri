# Doc Hygiene and Harness Checks

This repository uses local harness scripts to keep docs and architecture constraints coherent.

## Command

Run from repository root:

```bash
scripts/check-doc-harness.sh --mode warn
scripts/check-doc-harness.sh --mode strict
scripts/check-architecture-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode strict
```

Modes:

- `warn`: report findings, always exits `0`.
- `strict`: report findings and exit non-zero on violations.

## What the doc harness validates

- Required instruction files exist (root, `web/`, and each crate `AGENTS.md`).
- `CLAUDE.md` is a minimal adapter and contains required pointer text.
- Root strategy files are compatibility stubs that link to `docs/strategy/*` canonical documents.
- Local markdown links resolve.
- Root `AGENTS.md` includes required headings and local AGENTS references.

## What the architecture harness validates

- Workspace crate dependency direction for path dependencies.
- Forbidden runtime deps/imports in pure crates (`petri-core`, `petri-graph`).
- `src/lib.rs` export-focus constraints (no test module content or inline implementation items).
- Production Rust file size thresholds with baseline semantics.
- Baseline file consistency (`docs/standards/architecture-size-baseline.tsv`).

## Rollout (completion-gate only)

No CI gate is added in this phase; this is enforced by agent completion workflow.

- Phase 1: February 13, 2026 through February 27, 2026
  - Run `scripts/check-doc-harness.sh --mode warn`
  - Run `scripts/check-architecture-harness.sh --mode warn`
  - Include findings summary in handoff
- Phase 2: February 28, 2026 onward
  - Run `scripts/check-doc-harness.sh --mode strict`
  - Run `scripts/check-architecture-harness.sh --mode strict`
  - Non-zero exit blocks completion claims

## Findings Summary Format

Use this compact format in handoff for each harness:

- Harness mode used
- Violation count
- Warning count (architecture harness only)
- Any follow-up actions taken

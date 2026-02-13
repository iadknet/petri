# Doc Hygiene and Harness Checks

This repository uses a local harness script to keep docs and agent instruction surfaces coherent.

## Command

Run from repository root:

```bash
scripts/check-doc-harness.sh --mode warn
scripts/check-doc-harness.sh --mode strict
```

Modes:

- `warn`: report findings, always exits `0`.
- `strict`: report findings and exit non-zero on violations.

## What the harness validates

- Required instruction files exist (root, `web/`, and each crate `AGENTS.md`).
- `CLAUDE.md` is a minimal adapter and contains required pointer text.
- Root strategy files are compatibility stubs that link to `docs/strategy/*` canonical documents.
- Local markdown links resolve.
- Root `AGENTS.md` includes required headings and local AGENTS references.

## Rollout (completion-gate only)

No CI gate is added in this phase; this is enforced by agent completion workflow.

- Phase 1: February 13, 2026 through February 27, 2026
  - Run `scripts/check-doc-harness.sh --mode warn`
  - Include findings summary in handoff
- Phase 2: February 28, 2026 onward
  - Run `scripts/check-doc-harness.sh --mode strict`
  - Non-zero exit blocks completion claims

## Findings Summary Format

Use this compact format in handoff:

- Harness mode used
- Violation count
- Any follow-up actions taken

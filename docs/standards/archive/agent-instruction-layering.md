# Agent Instruction Layering

This repository uses explicit instruction layering so agents can reconcile superpowers skills with project docs.

## ELI5 Model

- Skills tell agents **HOW** to execute.
- `AGENTS.md` and docs tell agents **WHAT** this project requires.
- `README.md` is human quickstart context.

## Responsibility Matrix

| Surface | Owns | Does Not Own |
| --- | --- | --- |
| Superpowers skills | Execution method, workflow mechanics, process discipline | Project-specific architecture and invariants |
| `AGENTS.md` + local `*/AGENTS.md` | Boundaries, invariants, completion gates, test expectations | Re-teaching generic skill workflows |
| `docs/` canonical references | Stable source-of-truth project policy and architecture docs | Step-by-step skill mechanics |
| `README.md` | Human quickstart and run/build commands | Full agent policy |

## Conflict Resolution

Resolve conflicts in this order:

1. System/developer/user instructions from the active session.
2. Repository `AGENTS.md` and canonical standards docs.
3. Superpowers skill workflow guidance.
4. Secondary convenience docs.

If a skill conflicts with project invariants in repo docs, follow repo invariants unless a higher-priority session instruction explicitly overrides them.

## No-Duplication Rule

- Root `AGENTS.md` should stay map-like and project-specific.
- Generic workflow mechanics already enforced by superpowers skills should not be duplicated in full inside root `AGENTS.md`.
- Local crate/frontend `AGENTS.md` files should focus on local boundaries and invariants.

---
name: agents-md
description: Keep AGENTS.md (and the CLAUDE.md that imports it) bare bones. Use before adding, editing, reviewing or auditing any line of AGENTS.md or frontend/AGENTS.md, including when a task tempts you to record a new rule, decision, model setting or track detail there.
---

# AGENTS.md stays bare bones

`AGENTS.md` loads into every agent session, so every line costs context on every turn.
It holds only:

- repository-wide rules that apply to most tasks;
- the repository layout;
- pointers to the documents that hold detail.

## Limits

- Under 300 lines. `scripts/policy-check` (run by `make check-docs` and `make check`)
  fails at 300 lines or more, and on any line over 120 characters, so a rule cannot hide as
  one long paragraph. Aim far below the limit; the limit is a backstop, not a budget.
- One rule per bullet, one or two sentences. A pointer beats a paragraph.

## What never goes in

- Per-track, per-feature or per-task instructions, exemptions and contracts.
- Model, effort or role tables and delegation details for the roadmap workflow.
- Dated decisions, rationale, history or "why" explanations.
- Anything another document already states. Point to it instead of restating it.

## Where it goes instead

| Content | Home |
| --- | --- |
| Roadmap execution, roles, gates, goal command | `docs/workflow.md` |
| Codex model, tool and worktree substitutions | `docs/workflow-codex.md` |
| Rationale and dated workflow changes | `docs/workflow-history.md` |
| Track or feature rules | the track file in `docs/roadmaps/` or the feature spec |
| Research, diagnosis, decisions with evidence | a dated note in `docs/strategy/` |
| Benchmark and telemetry storage | `docs/benchmark-artifacts.md` |
| Frontend-only rules | `frontend/AGENTS.md` |
| Contribution, commit and security process | `CONTRIBUTING.md`, `SECURITY.md` |

## Editing checklist

1. Can an existing line or a pointer cover it? Then do that and stop.
2. Is it repo-wide and needed by most sessions? If not, put it in its home above.
3. When adding a line, look for one to shorten or remove.
4. Run `make check-docs`.

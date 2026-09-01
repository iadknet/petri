---
description: Route repository work through PRD creation, readiness review, ordered implementation, final review, and archival based on the current PRD state.
license: MIT
metadata:
    github-path: skills/agent-project-scaffold/assets/project-skills/project-workflow
    github-ref: 83ef0fb281a17c28c584a410b371d1afe7c6b37b
    github-repo: https://github.com/iadknet/new_app_scaffolding_skills
    github-tree-sha: d272325df083fc2f9390c4b78284b34d32f9ed2e
name: project-workflow
---
# Project Workflow

Inspect `AGENTS.md`, `docs/prds/active/`, and the user's requested outcome, then
select the narrowest applicable workflow and execution mode.

## Execution modes

Use **Lean** mode unless the user explicitly authorizes **Deep** mode. Record the
mode and any Deep-mode authorization in the master PRD. If a Lean limit is
exceeded, stop and narrow the work or request authorization; never silently
expand into Deep mode.

Lean mode is the ordinary feature path:

1. One bounded planning pass creates a compact PRD with at most two stages and an
   affected-file budget no greater than 25. When agents are requested, use one
   Sol x-high planner.
2. Perform one compact readiness review and the mechanical PRD checks. Do not
   repeat repository discovery or produce a separate narrative review artifact.
3. Use one persistent implementer for all stages and later fixes. When agents are
   requested, use Luna high. Run focused checks while working and `make check`
   once when implementation is ready for final review.
4. Use one independent, diff-scoped final review. When agents are requested, use
   Sol high. Give it the PRD paths, changed-file list, diff, and verification
   evidence rather than full conversation history.
5. Return P1/P2 findings to the same implementer for one bounded fix pass. Rerun
   affected checks and one final `make check`. If P1/P2 findings remain, stop for
   user direction instead of starting another autonomous cycle.
6. On a pass, synchronize status, validate, and archive.

Use compact handoffs with `fork_turns: none` or the smallest useful recent-turn
window. Include only the assignment, PRD paths, affected paths, decisions,
verification results, and blockers. Summarize successful command output; retain
diagnostic detail only for failures.

Deep mode is for user-authorized work that cannot fit Lean limits, such as broad
migrations or cross-cutting architecture. Its PRD must state why Lean mode is
insufficient and define explicit scope, review, and stopping budgets. Deep mode
does not authorize implementation or external side effects by itself.

## State routing

- No applicable PRD, or a material scope/design change: use `$prd-create`.
- `Draft` PRD with `Review Status: DRAFT`: use `$prd-review` for readiness,
  respecting the mode's review budget.
- A PRD at its readiness-review limit with unresolved P1/P2 findings: stop for
  human intervention.
- `Ready` or `In Progress` PRD with `Review Status: APPROVED`: use
  `$prd-implement`.
- Implemented stages awaiting final review: use `$prd-review` for final code.
- `Complete` PRD: validate and archive through `scripts/prd-archive`.

Run only phases authorized by the user. Creating or reviewing a plan does not by
itself authorize implementation, commits, remotes, or external changes. When a
phase reveals a material mismatch, move back to the appropriate earlier skill and
preserve truthful statuses and checkboxes.

Execution mode, Lean scope and document budgets, readiness status, review caps,
evidence structure, documentation synchronization, and the general 750-line
ceiling are hard gates enforced by `scripts/prd-check`; do not bypass them.

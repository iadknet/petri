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

Inspect `AGENTS.md`, `docs/prds/active/`, and the user's requested outcome.
Lean mode is the default; select Deep mode only when the user explicitly opts in
and that authorization is recorded in the PRD. Use the narrowest applicable
workflow:

- No applicable PRD, or a material scope/design change: use `$prd-create`.
- `Draft` PRD with `Review Status: DRAFT` and fewer than three reviews: use
  `$prd-review` for readiness.
- `Draft` PRD with three reviews and unresolved P1/P2 findings: stop for human
  intervention.
- `Ready` or `In Progress` PRD with `Review Status: APPROVED`: use
  `$prd-implement`.
- Implemented stages awaiting final review: use `$prd-review` for final code.
- `Complete` PRD: validate and archive through `scripts/prd-archive`.

Run only phases authorized by the user. Creating or reviewing a plan does not by
itself authorize implementation, commits, remotes, or external changes. When a
phase reveals a material mismatch or exceeds the declared scope budget, stop for
direction rather than recursing or expanding scope.

## Lean orchestration

Use compact path/artifact handoffs; never create full-history forks. For an
approved Lean PRD, use this bounded sequence:

1. One Sol x-high planning pass, followed by compact, mechanical readiness
   validation.
2. One persistent Luna high implementer, working stages in order and running
   focused checks.
3. One `make check` before review.
4. One diff-scoped Sol high final review.
5. The same Luna performs one P1/P2 fix pass only, then runs affected checks and
   a final `make check`.
6. Archive only after the final gate passes.

If review or remediation needs another pass, changes the plan, or exceeds the
scope budget, stop for user direction. Deep orchestration may add work only when
the recorded authorization calls for it.

The execution metadata, stage budget, documentation gate, and document limits
are hard gates enforced by `scripts/prd-check`; do not bypass them.

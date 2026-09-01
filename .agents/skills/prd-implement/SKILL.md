---
description: Implement an active PRD that has passed readiness review, preserving stage order, truthful state, verification, final review, and archival gates.
license: MIT
metadata:
    github-path: skills/agent-project-scaffold/assets/project-skills/prd-implement
    github-ref: 83ef0fb281a17c28c584a410b371d1afe7c6b37b
    github-repo: https://github.com/iadknet/new_app_scaffolding_skills
    github-tree-sha: 67d18918502517ddb4d8b693410a9de919ddedc7
name: prd-implement
---
# Implement a Reviewed PRD

Accept only a PRD with `Review Status: APPROVED` whose master is `Ready` or
`In Progress` and whose applicable stage is `Ready` or `In Progress`. If not,
route to `$prd-review` or `$prd-create` instead of starting code changes.

## Workflow

1. Read the master, every stage, their references, and affected code. Confirm the
   dependency order and current repository state still match the plan.
2. Implement stages in order. Keep changes within the stated boundaries; revise
   and re-review the PRD if discovered work materially changes scope, architecture,
   dependencies, or non-goals.
3. Mark a task complete only after its result exists. Run each declared check and
   mark verification complete only after observing success. Record useful evidence
   in the stage rather than relying on intent.
4. Create, update, or synchronize the durable documentation identified by each
   stage so it matches implemented behavior. If implementation evidence shows no
   documentation is affected, record the concrete rationale before completing the
   documentation gate.
5. Set a stage `Complete` only when its dependencies, tasks, documentation gate,
   and verification are complete. Keep the master `In Progress` until every stage
   and implementation task is complete.
6. Use `$prd-review` for the final-code gate. Address P1/P2 findings and rerun
   affected verification after review-driven changes. This review does not change
   the PRD readiness `Review Count`.
7. Run `make check`. When final review has passed and validation confirms complete
   state, run `scripts/prd-archive <slug>`.

Do not archive manually, skip dependencies, or conceal follow-up debt in order to
declare completion.

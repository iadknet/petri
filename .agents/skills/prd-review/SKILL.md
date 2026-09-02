---
description: Review a PRD for readiness or review its completed implementation, with severity-ranked findings and enforced revision recursion.
license: MIT
metadata:
    github-path: skills/agent-project-scaffold/assets/project-skills/prd-review
    github-ref: 83ef0fb281a17c28c584a410b371d1afe7c6b37b
    github-repo: https://github.com/iadknet/new_app_scaffolding_skills
    github-tree-sha: ecf39e66d0f8b40275df4a69c492f28899627e9c
name: prd-review
---
# Review a PRD or Its Implementation

Review final code independently. For Lean readiness, the coordinator performs a
compact audit in its existing context and must not spawn another agent. Deep
readiness remains independent. The master's `Review Count` is the number of
readiness audits attempted. It starts at 0 and never exceeds 1 in Lean mode or 3
in authorized Deep mode. `Review Status` is `DRAFT` until readiness is approved,
then `APPROVED`. Final-code review does not change either readiness field.

## Review criteria

Inspect repository evidence, not only the prose. For Lean readiness, the
coordinator uses the PRD, directly affected code, and the planner's compact
handoff; do not delegate, repeat broad repository exploration, or reload planning
history unless a concrete claim cannot otherwise be verified.
Check consistency, dependency
order, component boundaries, abstraction levels, separation of concerns,
testability, existing-code impact, technical debt, security implications, and
accidental complexity. Confirm each stage identifies affected durable
documentation or gives a supported no-change rationale. For final-code review,
inspect the actual diff rather than the whole repository, verify affected
documentation matches implemented behavior, and use the recorded verification
evidence. Rerun a check only when the evidence is missing, stale, or contradicted
by the diff.

Classify findings:

- `P1`: unsafe or fundamentally incorrect; blocks the gate.
- `P2`: material gap, inconsistency, boundary failure, or missing verification;
  blocks the gate.
- `P3`: useful improvement that does not block the stated outcome.

## PRD readiness workflow

1. Require `Review Status: DRAFT`. Determine the review limit from Execution Mode:
   1 for Lean and 3 for authorized Deep. At the limit, stop for human intervention
   instead of beginning another automated review.
2. Increment `Review Count` by one for this attempt, review the current PRD, and
   report severity-ranked findings directly to the user.
3. P1/P2 findings block automated approval. In Lean mode, return a compact finding
   list and stop; the planner may revise the PRD, but another autonomous readiness
   review requires user direction or a newly authorized planning cycle. In Deep
   mode, make or request authorized in-scope revisions and repeat while below its
   review limit. P3 findings do not block approval.
4. If an attempt finds no P1/P2 findings, set `Review Status: APPROVED`, set the
   master and stages to `Ready`, update the index, and run `scripts/prd-check`.
5. If P1/P2 findings remain at the mode's review limit, keep `Review Status: DRAFT`
   and stop for a human reviewer. If that reviewer explicitly approves, set `Review Status`
   to `APPROVED`, set the master and stages to `Ready`, update the index, and run
   `scripts/prd-check`. If the reviewer instead requests material revisions and
   authorizes a new automated cycle, apply them and reset the count to 0 before
   reviewing.
6. Any later material PRD revision resets the lifecycle to `Draft`, the review
   status to `DRAFT`, and the count to 0.

Do not create separate review-record files.

## Final-code workflow

1. Inspect the actual implementation diff and recorded verification for affected
   stages. Rerun only checks whose evidence is missing, stale, or contradicted.
2. P1/P2 findings block completion. In Lean mode, send one compact finding list
   to the existing implementer for one fix pass, then perform one closure review.
   If blockers remain, stop for user direction. Deep mode follows the explicit
   remediation budget in its master PRD. P3 findings do not block the outcome.
3. On a pass, mark documentation synchronization and final-code review checkboxes
   complete and set the master `Complete` only when every stage and checkbox is
   complete. Update the index, run `scripts/prd-check`, and leave `Review Status`
   and `Review Count` unchanged.

Never increment a count, reduce severity, or mark a checkbox complete merely to
pass validation.

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

Review independently. The master's `Review Count` is the number of automated PRD
readiness reviews attempted. It starts at 0 and never exceeds 3. `Review Status`
is `DRAFT` until readiness is approved, then `APPROVED`. Final-code review does
not change either readiness field.

## Review criteria

Inspect repository evidence, not only the prose. Check consistency, dependency
order, component boundaries, abstraction levels, separation of concerns,
testability, existing-code impact, technical debt, security implications, and
accidental complexity. Confirm each stage identifies affected durable
documentation or gives a supported no-change rationale. For final-code review,
inspect the actual diff, verify affected documentation matches implemented
behavior, and rerun the verification declared by affected stages.

Classify findings:

- `P1`: unsafe or fundamentally incorrect; blocks the gate.
- `P2`: material gap, inconsistency, boundary failure, or missing verification;
  blocks the gate.
- `P3`: useful improvement that does not block the stated outcome.

## PRD readiness workflow

1. Require `Review Status: DRAFT`. If `Review Count` is already 3, stop for human
   intervention instead of beginning another automated review.
2. Increment `Review Count` by one for this attempt, review the current PRD, and
   report severity-ranked findings directly to the user.
3. P1/P2 findings block automated approval. Make or request authorized in-scope
   PRD revisions, then repeat from step 1 while the count is below 3. P3 findings
   do not block approval.
4. If an attempt finds no P1/P2 findings, set `Review Status: APPROVED`, set the
   master and stages to `Ready`, update the index, and run `scripts/prd-check`.
5. If P1/P2 findings remain after attempt 3, keep `Review Status: DRAFT` and stop
   for a human reviewer. If that reviewer explicitly approves, set `Review Status`
   to `APPROVED`, set the master and stages to `Ready`, update the index, and run
   `scripts/prd-check`. If the reviewer instead requests material revisions and
   authorizes a new automated cycle, apply them and reset the count to 0 before
   reviewing.
6. Any later material PRD revision resets the lifecycle to `Draft`, the review
   status to `DRAFT`, and the count to 0.

Do not create separate review-record files.

## Final-code workflow

1. Inspect the actual implementation diff and rerun the verification declared by
   affected stages.
2. P1/P2 findings block completion. Make or request authorized fixes and review
   again; P3 findings do not block the stated outcome.
3. On a pass, mark documentation synchronization and final-code review checkboxes
   complete and set the master `Complete` only when every stage and checkbox is
   complete. Update the index, run `scripts/prd-check`, and leave `Review Status`
   and `Review Count` unchanged.

Never increment a count, reduce severity, or mark a checkbox complete merely to
pass validation.

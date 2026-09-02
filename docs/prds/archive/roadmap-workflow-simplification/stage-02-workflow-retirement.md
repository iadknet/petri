# Stage 02 — Workflow Retirement

- Status: Complete
- Depends on: stage-01-roadmap-contract.md
- Master: [Master PRD](master-prd.md)

## Goal

Make the roadmap contract the only live feature-program workflow while retaining
completed PRDs and the former V3 roadmap as clearly non-executable history.

## Scope

- Simplify durable repository instructions and contribution surfaces.
- Remove the four active PRD skills, Claude links, templates, and scripts.
- Replace PRD gates in Makefile, CI, policy, and pre-commit wiring.
- Regenerate curated-skill provenance after removal.
- Archive the V3 roadmap and this migration record.

## Non-Goals

- Rewriting historical completed PRDs to the new format.
- Adding a general roadmap execution skill.
- Authoring the ecosystem-and-cognition roadmap or starting its goal.

## Inputs and Existing-Code Interactions

Stage 01 must pass before retirement. Live references currently occur in
`AGENTS.md`, `CONTRIBUTING.md`, the PR template, strategy and documentation
indexes, Makefile/CI/pre-commit, skill provenance, and `prd-*` scripts. Completed
records under `docs/prds/archive/` are inputs to preserve, not instructions to
rewrite or validate.

## Boundaries and Abstraction Layers

Root guidance retains only stable product and engineering invariants. Roadmap
execution details remain in the opt-in prompt template. Policy checks prevent
live PRD machinery from returning but exclude designated archive directories.
No application code or public API changes.

The final root invariants, create/modify/delete/move manifest, staged-index
semantics, residual denylist/exclusions, and one-time policy handoff are
normative in the master PRD.

## Separation of Concerns and Decomposition

All enforcement surfaces must switch together or a stale instruction can still
override the goal prompt. The history move and compatibility pointers belong in
the same cutover because they prevent two documents from claiming roadmap
authority.

## Tech Debt and Spaghetti-Code Implications

Removing the duplicated workflow reduces policy drift and context cost. The
remaining risk is over-broad residual scanning; exclusions are limited to the
two explicit historical directories and regression coverage protects live docs.

## Documentation Impact and Synchronization

Update `AGENTS.md`, `CONTRIBUTING.md`, the pull-request template, docs indexes,
strategy goals and architecture, archive notices, and root compatibility stubs.
Move the former V3 program roadmap to `docs/archive/` and leave a pointer at its
old path.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | No live instruction or automated gate requires PRD status, review counts, Lean/Deep modes, or the retired workflow skills; the older Superpowers/feature-lifecycle denylist remains active while both archive roots may retain history. | `scripts/residual-cruft-check` and `scripts/residual-cruft-check-test` | Passed: both commands pass; the live denylist audit is clean; fixtures cover both archive-root exclusions, named self-file content exclusions, whitespace-safe filenames, symlinks, and every retired path class. |
| AC-2 | Archived PRDs and the V3 roadmap remain accessible and explicitly historical. | Documentation-link audit | Passed: `docs/prds/archive/README.md` marks the archive static and non-executable; the migration record and `docs/archive/v3-program-roadmap.md` are present; docs, strategy, and root compatibility pointers resolve to the replacement contract/history. |
| AC-3 | Makefile, CI, policy, and pre-commit run the roadmap checker and no deleted command. | `make roadmap-check roadmap-check-test policy-check quality-check` | Passed: roadmap checker and 25-test suite pass; policy and quality checks pass; CI invokes only `make policy-check quality-check`; project-precommit starts with staged roadmap validation. |
| AC-4 | Skill mirrors and lock metadata contain only the retained curated skills. | `scripts/skill-provenance-check` | Passed: provenance check passes after lock regeneration; the four retired workflow skill directories and Claude links are removed and the retained curated skills remain represented. |

## Implementation or Decision Tasks

- [x] Replace root and contributor workflow prose with stable invariants and roadmap-applicable guidance.
- [x] Remove active PRD skills, links, templates, lifecycle scripts, and their tests.
- [x] Update Makefile, CI, pre-commit, policy, residual checks, provenance generator, and lock.
- [x] Archive the prior roadmap and migration PRD; synchronize all documentation pointers.
- [x] Complete the first full-diff cutover review and resolve its five P2 findings
  and trivial P3 in one authorized remediation pass.
- [x] Complete the closure review and final policy handoff. The user explicitly
  accepted the remaining non-runtime P2/P3 findings before merge.

## Verification and Observable Success Criteria

- [x] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete by explicit user acceptance. The first full-diff review identified five
P2 findings and one trivial P3, and the authorized remediation resolved them.
The elevated full `make check` passed. The closure review found a residual
live-symlink hardening P2 and dangling-symlink diagnostic P3; the user
explicitly accepted both before merge.

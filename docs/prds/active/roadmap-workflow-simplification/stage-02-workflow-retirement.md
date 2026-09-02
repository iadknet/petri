# Stage 02 — Workflow Retirement

- Status: Ready
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
| AC-1 | No live instruction or automated gate requires PRD status, review counts, Lean/Deep modes, or the retired workflow skills; the older Superpowers/feature-lifecycle denylist remains active while both archive roots may retain history. | `scripts/residual-cruft-check` and `scripts/residual-cruft-check-test` | Pending |
| AC-2 | Archived PRDs and the V3 roadmap remain accessible and explicitly historical. | Documentation-link audit | Pending |
| AC-3 | Makefile, CI, policy, and pre-commit run the roadmap checker and no deleted command. | `make roadmap-check roadmap-check-test policy-check quality-check` | Pending |
| AC-4 | Skill mirrors and lock metadata contain only the retained curated skills. | `scripts/skill-provenance-check` | Pending |

## Implementation or Decision Tasks

- [ ] Replace root and contributor workflow prose with stable invariants and roadmap-applicable guidance.
- [ ] Remove active PRD skills, links, templates, lifecycle scripts, and their tests.
- [ ] Update Makefile, CI, pre-commit, policy, residual checks, provenance generator, and lock.
- [ ] Archive the prior roadmap and migration PRD; synchronize all documentation pointers.
- [ ] Run both declared implementation reviews and resolve blocking findings.
- [ ] Follow the policy handoff exactly; do not mark the archived record Complete
  until the full cutover review has passed and closure evidence exists.

## Verification and Observable Success Criteria

- [ ] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [ ] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Ready. Starts only after Stage 01 is complete and the replacement checker passes.

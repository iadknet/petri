# Plans (Moved)

The plans system has moved to the unified feature lifecycle hierarchy.

**New locations:**
- Feature ideas: [`docs/features/brainstorms/ideas.md`](../features/brainstorms/ideas.md)
- Features in refinement: [`docs/features/needs_refinement/`](../features/needs_refinement/)
- Features ready to implement: [`docs/features/ready_to_implement/`](../features/ready_to_implement/)
- Features in progress: [`docs/features/in_progress/`](../features/in_progress/)
- Completed features: [`docs/features/completed/`](../features/completed/)
- Cancelled features: [`docs/features/cancelled/`](../features/cancelled/)

## Skills

Use the `features:` skills to move features through the workflow:
- `features:capture-idea` — add an idea to brainstorms
- `features:promote-to-refinement` — create a structured refinement doc
- `features:promote-to-ready` — create a full master_plan.md
- `features:start-implementation` — move to in_progress and begin
- `features:resume-feature` — resume an in-progress feature
- `features:complete-feature` — finalize and move to completed
- `features:cancel-feature` — cancel from any stage

## Plan Harness

`scripts/check-plan-harness.sh` now scans `docs/features/ready_to_implement/` and
`docs/features/in_progress/` for `master_plan.md` files.

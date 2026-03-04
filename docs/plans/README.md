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

Use the `feature-*` skills to move features through the workflow:
- `feature-1-capture` — add an idea to brainstorms
- `feature-2-refine` — create a structured refinement doc
- `feature-3-plan` — create a full master_plan.md
- `feature-4-implement` — move to in_progress and begin
- `feature-5-resume` — resume an in-progress feature
- `feature-6-complete` — finalize and move to completed
- `feature-7-cancel` — cancel from any stage

## Plan Harness

`scripts/check-plan-harness.sh` now scans `docs/features/ready_to_implement/` and
`docs/features/in_progress/` for `master_plan.md` files.

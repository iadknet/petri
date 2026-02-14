# Plans Metadata and Template Rules

Use `docs/plans/YYYY-MM-DD-<topic>.md` for non-trivial, multi-step work.

Archived historical plans live under `docs/plans/archive/` and are excluded from plan-quality harness enforcement.

## Required metadata

Each active non-trivial plan must include:

- `Goal`: one-sentence outcome statement
- `Goal IDs`: one or more IDs from `docs/strategy/goals.md`
- `Scope`: what is included and excluded
- `Docs Impact`: canonical docs touched and stale docs retired/superseded
- `Supersedes`: older plan IDs/files being replaced (or `none`)
- `Superseded-By`: newer plan file when this plan is replaced (or `none`)

## Required structure

At minimum include:

1. `## Goal Alignment`
2. `## Boundary Impact`
3. `## Existing Boundary Recheck`
4. `## Open Questions`
5. Task list with explicit file targets
6. Verification commands
7. Risks/rollback notes when applicable

## Existing Boundary Recheck requirements

Include at least two reviewed existing areas (crate/module/file), each with:

- decision (`keep` or `change`)
- rationale

## Open Questions table format

`## Open Questions` must use a Markdown table with columns:

- `question`
- `decision`
- `owner`
- `status` (use `resolved` or `unresolved`)

## Docs Impact section

Use a compact table or bullet list with:

- updated canonical docs
- compatibility stubs (if any)
- retired or superseded docs/plans

This keeps plan output aligned with docs and plan harness checks and avoids silent drift.

## Plan Lifecycle

- Active plans in `docs/plans/` should reflect the current product direction.
- When strategy changes (for example, a greenfield reset), move stale plans to `docs/plans/archive/`.
- Prefer replacing one large plan with stage/checkpoint plans so ownership and go/stop boundaries are explicit.
- For multi-checkpoint programs, maintain one shared verification matrix plan and reference it from each stage.

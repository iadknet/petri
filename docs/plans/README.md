# Plans Metadata and Template Rules

Use `docs/plans/YYYY-MM-DD-<topic>.md` for non-trivial, multi-step work.

## Required metadata

Each new non-trivial plan must include:

- `Goal`: one-sentence outcome statement
- `Scope`: what is included and excluded
- `Docs Impact`: canonical docs touched and stale docs retired/superseded
- `Supersedes`: older plan IDs/files being replaced (or `none`)
- `Superseded-By`: newer plan file when this plan is replaced (or `none`)

## Required structure

At minimum include:

1. Context
2. Task list with explicit file targets
3. Verification commands
4. Risks/rollback notes when applicable

## Docs Impact section

Use a compact table or bullet list with:

- updated canonical docs
- compatibility stubs (if any)
- retired or superseded docs/plans

This keeps plan output aligned with the docs harness checks and avoids silent drift.

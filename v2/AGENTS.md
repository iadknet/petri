# v2 AGENTS.md

Local instructions for all work under `v2/`.

## Scope

- `v2` is a greenfield rewrite boundary.
- Keep all new implementation inside `v2/`.
- Treat legacy crates/web as reference-only.
- Backward compatibility is not required in this stage; restart-first behavior is acceptable.

## Boundary Rules

- No dependency from `v2/*` into legacy `petri-*` crates or root `web/`.
- Reuse via copy/adapt/test only (see `v2/docs/COPY_POLICY.md`).
- Keep module ownership aligned with `v2/docs/BOUNDARIES.md`.

## Planning and Gates

- For multi-step work, update the active plan in `docs/plans/`.
- Keep checkpoint verification aligned with active gate docs in `docs/plans/` (use `docs/plans/archive/` only for historical comparison).
- Prefer small, checkpoint-scoped commits.
- Golden rule for completion: checkpoint intent must be functionally implemented/integrated and test-proven, not contract-only.
- Treat `docs/standards/runtime-behavior-realism-policy.md` as a standing requirement for any runtime-facing change.
- Treat `docs/standards/intent-verification-policy.md` as a standing requirement for all feature areas.

## Test Execution Rule

- During `v2` rewrite work, never run legacy test suites.
- Run only the appropriate `v2` tests for the area being changed.

## Local Map

- `v2/crates/AGENTS.md`
- `v2/crates/v2-core/AGENTS.md`
- `v2/crates/v2-server/AGENTS.md`
- `v2/crates/v2-cli/AGENTS.md`
- `v2/web/AGENTS.md`

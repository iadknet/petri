# v2 crates AGENTS.md

Shared instructions for `v2/crates/*`.

## Dependency Direction

- `v2-core` is the policy/runtime base crate.
- `v2-server` and `v2-cli` depend on `v2-core`.
- `v2-core` must not depend on `v2-server` or `v2-cli`.

## Contract Discipline

- Keep public interfaces explicit and minimal.
- When wire-facing contracts change, update tests and CP docs in the same slice.
- Prefer reproducible, fixture-friendly behavior in tests over implicit defaults.
- No backward-compatibility layer is required in this phase; breaking changes may assume restart.

## Testing

- Add/adjust failing tests first for behavior changes.
- Keep crate-local tests focused; use checkpoint gates for cross-crate validation.

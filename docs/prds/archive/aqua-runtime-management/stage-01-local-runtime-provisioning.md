# Stage 01 — Local Runtime Provisioning

- Status: Complete
- Depends on: None
- Master: [Master PRD](master-prd.md)

## Goal

Provision the exact Node/npm and uv toolchain from Aqua for every local Petri
command while preserving rustup-based Rust selection.

## Scope

- Add exact Node 24.20.0 and uv 0.12.1 package entries to `aqua.yaml`.
- Regenerate and commit Aqua checksum metadata for every configured supported
  platform.
- Extend `scripts/aqua` with a POSIX-safe command environment mode and route
  local Make, development, bootstrap, and uv installer calls through it.
- Remove `.nvmrc`; retain `frontend/package.json` Node/npm declarations as
  package-policy checks.
- Update `README.md` and `SECURITY.md`.

## Non-Goals

- GitHub Actions job setup changes, which belong to Stage 02.
- Changing Rust installation or the `rust-toolchain.toml` format.
- Adding globally installed npm tools or an npm prefix.

## Inputs and Existing-Code Interactions

`scripts/aqua` currently supports only `install` and `exec`; its local root and
config environment must be shared with shell scripts that call Node/npm/uv.
`scripts/bootstrap-check` validates versions before setup, so it must install
Aqua packages before checking those commands. `Makefile`, `scripts/dev.sh`,
`scripts/install-pre-commit`, and `scripts/install-skill-scanner` invoke host
Node, npm, or uv today. The four existing Aqua CLIs and their callers must
continue working unchanged.

## Boundaries and Abstraction Layers

The process boundary is `scripts/aqua`: its new environment-preserving mode may
be consumed by project scripts, while Aqua configuration stays declarative in
`aqua.yaml`. Make targets expose the same public commands (`make setup`,
`make run`, `make check`); callers do not need a shell activation hook. Rust
commands remain direct so rustup honors `rust-toolchain.toml`.

## Separation of Concerns and Decomposition

Tool declarations, execution adapter, consumers, and documentation form one
observable local bootstrap contract. Splitting them would produce an
intermediate state where Aqua contains Node/uv but repository commands still
use host binaries.

## Tech Debt and Spaghetti-Code Implications

Centralizing environment setup in `scripts/aqua` removes duplicated PATH and
version-check logic. No new tool-specific wrapper is introduced.

## Documentation Impact and Synchronization

Update `README.md` to require Aqua and rustup rather than preinstalled Node/npm/
uv, and state that `make setup` provisions the pinned non-Rust tools. Update
`SECURITY.md` to include Node and uv in the Aqua-managed checksum-enforced
tooling description. No architecture, API, user-interface, or generated
reference documentation changes are required because this stage changes only
developer provisioning.

## Implementation or Decision Tasks

- [x] Add exact Node and uv Aqua packages and regenerate `aqua-checksums.json`.
- [x] Add a documented POSIX-safe `scripts/aqua` mode that executes a command
  with the project-local Aqua bin directory first in `PATH`.
- [x] Make bootstrap install Aqua tools before version checks, then check
  Node 24.20.0, bundled npm 11.19.0, uv 0.12.1, and Rust 1.93.0.
- [x] Route local npm and uv consumers through the adapter; keep Rust commands
  direct.
- [x] Remove `.nvmrc` and synchronize `README.md` and `SECURITY.md`.

## Verification and Observable Success Criteria

- [x] `make setup` installed the Aqua-pinned Node v24.20.0, npm 11.19.0, uv
  0.12.1, hook tools, and frontend dependencies in this worktree.
- [x] `scripts/aqua env` placed `.tools/aqua/bin` first in `PATH`; shell syntax
  validation passed for every changed POSIX script.
- [x] Run `make policy-check`, `make quality-check`, and `make frontend-check`.
- [x] Documentation is synchronized as specified above.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [x] The full `make check` gate passed outside the sandbox so WebSocket tests
  could bind their local listener (the sandbox-only run failed exclusively with
  `Operation not permitted`).
- [x] On the verified `.tools` directory, `make setup` observed the exact
  Node, npm, uv, Rust, and Aqua CLI versions.

## Current Status

Complete. Aqua now supplies Node v24.20.0/npm 11.19.0 and uv 0.12.1 through
checksum-enforced local paths; rustup remains the Rust authority. `make setup`
and the complete validation gate passed.

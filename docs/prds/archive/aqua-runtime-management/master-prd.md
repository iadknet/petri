# Aqua Runtime Management — Master PRD

- Status: Complete
- Owner: Unassigned
- Created: 2026-09-01
- Review Status: APPROVED
- Review Count: 1

## Goal

Make `make setup` provision Petri's Node, npm, uv, and auxiliary CLI versions
from the repository while retaining rustup as the Rust toolchain authority.

## Scope

- Extend the existing checksum-enforced Aqua configuration to include Node 24.20.0
  and uv 0.12.1.
- Route local bootstrap and developer commands through the project-local Aqua
  environment so host Node, npm, and uv versions cannot satisfy Petri by accident.
- Remove the redundant Node version file and synchronize CI with the local
  provisioning contract.
- Update user and security documentation to name the resulting bootstrap
  contract.

## Non-Goals

- Replacing rustup or `rust-toolchain.toml`.
- Changing frontend dependencies, Node package-management policy, or Rust code.
- Adding a new version manager such as mise, asdf, or Volta.
- Changing the supported OS matrix beyond the existing Darwin and Linux Aqua
  checksum coverage.

## Inputs and Existing-Code Interactions

`aqua.yaml` already pins actionlint, ShellCheck, Gitleaks, and OSV-Scanner, and
`scripts/aqua` isolates them under `.tools/aqua` with committed checksums.
`rust-toolchain.toml` already pins Rust 1.93.0, while `.nvmrc`,
`frontend/package.json`, `scripts/bootstrap-check`, and CI separately name the
Node/npm/uv versions. Node 24.20.0 bundles the required npm 11.19.0.

Stage 01 changes Aqua configuration, the local command boundary, bootstrap,
Make targets, installer scripts, developer documentation, and generated
checksums. Stage 02 removes redundant CI setup actions and makes CI use the
same Aqua-defined Node and uv tools. The pre-existing Rust CI action remains
the Rust installation boundary.

## Boundaries and Abstraction Layers

`aqua.yaml` is the declarative source of truth for tools that Aqua can manage.
`scripts/aqua` is the sole process-execution adapter that establishes the
project-local Aqua environment. Make, development, and installer scripts invoke
that adapter rather than duplicate PATH or version logic. `rust-toolchain.toml`
remains the source of truth for Rust.

Stage 01 establishes the local contract; Stage 02 consumes that contract in CI
without changing its semantics.

## Separation of Concerns and Decomposition

Local provisioning must exist and be verified before CI can consume it. The
first stage owns repository configuration and local execution paths; the second
owns GitHub Actions convergence. Neither stage introduces runtime-facing product
behavior.

## Tech Debt and Spaghetti-Code Implications

The change removes split-brain Node and uv setup logic and centralizes the
process environment in the existing wrapper. Rust's separate native manager is
deliberate rather than debt.

## Documentation Impact and Synchronization

Update `README.md` for the one-command setup contract and `SECURITY.md` for the
expanded Aqua-managed tool set. No architecture, API, user-interface, or
generated-reference documentation changes are required because this affects
developer tooling only.

## Stage Order and Links

1. [Stage 01 — Local Runtime Provisioning](stage-01-local-runtime-provisioning.md) — `Complete`
2. [Stage 02 — CI Convergence](stage-02-ci-convergence.md) — `Complete`

Stage 01 establishes the Aqua-managed local tool contract. Stage 02 removes CI
copies of Node and uv setup and verifies each workflow job receives that same
contract.

## Cross-Stage Decisions

- Aqua, already used with checksum enforcement and a pinned registry, is the
  selected manager for Node, uv, and auxiliary CLIs.
- Node 24.20.0 is an exact Aqua pin; npm remains exact in `package.json` and is
  provided by that Node release.
- `rust-toolchain.toml` and rustup remain authoritative for Rust.
- Tool installers must remain POSIX `sh` compatible.

## Implementation or Decision Tasks

- [x] Complete Stage 01 before modifying CI in Stage 02.
- [x] Keep stage links, task checkboxes, and status summaries current.

## Verification and Observable Success Criteria

- [x] Every stage's declared verification has passed.
- [x] `README.md` and `SECURITY.md` match the implemented bootstrap contract.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [x] The final-code review gate has passed: implementation review found no
  P1/P2 findings, and `make check` passed outside the sandbox.

## Current Status

Complete. Readiness review 1 and final implementation review found no P1/P2
findings. The unavailable uv 0.12.3 pin was replaced with the nearest available
official release, uv 0.12.1. The checksum-enforced Aqua implementation and
full `make check` gate passed.

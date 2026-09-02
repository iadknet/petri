# Stage 02 — CI Convergence

- Status: Complete
- Depends on: stage-01-local-runtime-provisioning.md
- Master: [Master PRD](master-prd.md)

## Goal

Make GitHub Actions provision Node and uv through the same Aqua configuration
used locally, while retaining the existing Rust toolchain action.

## Scope

- Remove `actions/setup-node` and `astral-sh/setup-uv` from jobs whose only need
  is the project-pinned tool.
- Install Aqua and invoke the Stage 01 adapter before policy, frontend, and
  skill-security commands.
- Preserve action SHA pinning, read-only permissions, job fan-out, and Rust
  setup behavior.

## Non-Goals

- Reorganizing CI jobs, adding caches, changing timeouts, or changing the
  supported runner matrix.
- Replacing the Rust action with a new mechanism.

## Inputs and Existing-Code Interactions

The `policy` job currently installs uv, Node, and Aqua separately. Frontend jobs
install Node separately. The `skill-security` job installs uv separately. Stage
01 gives each job a project-local Aqua command adapter that resolves Node and
uv after the pinned Aqua installer action runs. Rust format/test/clippy jobs
continue using `dtolnay/rust-toolchain` at the exact current action commit.

## Boundaries and Abstraction Layers

The workflow retains job names and Make-target entry points. Its setup boundary
changes only from vendor setup actions to the repository's Aqua installation and
adapter. CI never relies on a developer's shell PATH.

## Separation of Concerns and Decomposition

All changes are consumers of the completed Stage 01 local contract. Splitting
by job would create temporary inconsistent CI provisioning with no independent
long-term abstraction.

## Tech Debt and Spaghetti-Code Implications

This removes duplicated version literals and setup mechanisms from CI while
leaving the established job topology intact.

## Documentation Impact and Synchronization

No documentation changes required: Stage 01 documents the user-visible setup
contract, and this stage only aligns internal GitHub Actions implementation with
that already documented contract.

## Implementation or Decision Tasks

- [x] Replace Node and uv setup actions with the pinned Aqua installer where
  needed, preserving all remaining pinned actions and permissions.
- [x] Invoke the Stage 01 adapter for policy, frontend, and skill-security
  commands that require Node or uv.
- [x] Confirm workflow definitions stay valid under existing repository policy.

## Verification and Observable Success Criteria

- [x] `make policy-check` and `make quality-check` passed, including
  ShellCheck and actionlint.
- [x] CI contains no `actions/setup-node` or `astral-sh/setup-uv`; its Node and
  uv consumers invoke the existing pinned Aqua installer and `scripts/aqua`.
- [x] Inspect CI jobs to confirm Node 24.20.0 and uv 0.12.1 appear only in the
  Aqua source of truth, while Rust remains in `rust-toolchain.toml` and its
  existing Rust setup action.
- [x] No documentation changes required: Stage 01 owns the durable setup
  documentation and this stage does not change its externally visible contract.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. CI provisions Node and uv only through the checksum-enforced Aqua
contract, while Rust retains its existing rustup action.

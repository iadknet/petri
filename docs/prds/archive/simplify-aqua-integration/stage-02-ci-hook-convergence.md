# Stage 02 — Ci Hook Convergence

- Status: Complete
- Depends on: stage-01-proxy-path-boundary.md
- Master: [Master PRD](master-prd.md)

## Goal

Make Git hooks and CI consume the Stage 01 proxy/PATH boundary without custom
Aqua invocation layers.

## Scope

- Add an internal Make target for project pre-commit validation and route the
  pre-commit configuration through it.
- Give CI project-local Aqua root and checksum environment values.
- Rely on aqua-installer's default proxy-link step; remove redundant install
  steps and invoke npm/uv directly.

## Non-Goals

- Changing CI topology, action versions, caching, permissions, or Rust setup.

## Inputs and Existing-Code Interactions

Git pre-commit runs outside a developer's Make command, while Aqua's GitHub
Action adds its root bin directory and runs `aqua install -l` by default. CI
currently runs the action and then repeats adapter-managed installation.

## Boundaries and Abstraction Layers

The project hook calls Make rather than inheriting shell state. CI workflow
environment owns its Aqua root and checksum enforcement, allowing direct proxy
commands in each job.

## Separation of Concerns and Decomposition

Both consumers run outside normal local recipes and must converge on the same
environment contract after Stage 01.

## Tech Debt and Spaghetti-Code Implications

The stage removes redundant CI downloads and an adapter boundary from hooks.

## Documentation Impact and Synchronization

No documentation changes required: Stage 01 documents the user-visible setup
contract; this stage changes its consumers only.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | Hook validation inherits Make's proxy-first environment. | Installed hook run. | `./.tools/bin/pre-commit run project-validation --all-files` passed with the hook entry `make project-precommit`. |
| AC-2 | CI has no adapter calls or redundant Aqua install steps. | Workflow search and actionlint. | Workflow-scoped `AQUA_ROOT_DIR` and checksum enforcement are present; static search found no adapter, duplicate install, setup-node, or setup-uv reference; actionlint passed. |

## Implementation or Decision Tasks

- [x] Route project pre-commit validation through a Make target.
- [x] Remove CI adapter and duplicate-install calls while preserving pins and permissions.
- [x] Confirm workflow and hook behavior with focused checks.

## Verification and Observable Success Criteria

- [x] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. No user-facing documentation change was needed beyond the Stage 01
proxy setup wording.

# Stage 01 — Proxy Path Boundary

- Status: Complete
- Depends on: None
- Master: [Master PRD](master-prd.md)

## Goal

Make normal tool invocations resolve Aqua's project-local proxies.

## Scope

- Export project-local root, proxy-first PATH, and checksum enforcement in Make.
- Use `aqua install -l` in setup; remove `scripts/aqua` and all adapter calls.
- Update local bootstrap, development, quality, security, dependency, and uv
  installer scripts to use normal proxied executables.
- Synchronize README and security setup documentation.

## Non-Goals

- Git hook entry and GitHub Actions changes, which belong to Stage 02.

## Inputs and Existing-Code Interactions

Make already owns all stable developer targets. `scripts/aqua` duplicates PATH
handling, while quality, audit, and installer scripts currently invoke it. The
existing `aqua.yaml` checksum block is the durable enforcement configuration.

## Boundaries and Abstraction Layers

Make exports the environment to its recipes. Scripts execute command names such
as `node`, `npm`, `uv`, ShellCheck, actionlint, Gitleaks, and OSV-Scanner
directly; they do not choose or configure Aqua execution modes.

## Separation of Concerns and Decomposition

These changes all remove the same local abstraction. Splitting individual tool
callers would leave an inconsistent command boundary.

## Tech Debt and Spaghetti-Code Implications

The task removes accidental complexity without introducing new dependencies.

## Documentation Impact and Synchronization

Update README and SECURITY wording from eager installation through a wrapper to
proxy-link setup with checksum-verified first-use downloads.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | Make routes Node/npm/uv through local Aqua proxies. | Clean-root `make setup` plus path/version probe. | `make setup` completed from an absent `.tools/aqua`; proxy probe resolved Node v24.20.0, npm 11.19.0, uv 0.12.1, and npm subprocess Node v24.20.0. |
| AC-2 | No local executable caller invokes `scripts/aqua` or `aqua exec`. | Static search and focused checks. | `rg` over Make, scripts, workflows, and pre-commit config returned no adapter or `aqua exec` reference; `make policy-check quality-check` passed. |

## Implementation or Decision Tasks

- [x] Export the project-local Aqua environment from Make and preserve override support.
- [x] Replace adapter calls with normal proxied commands and delete `scripts/aqua`.
- [x] Make setup create proxy links before version validation and synchronize docs.

## Verification and Observable Success Criteria

- [x] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. README and SECURITY now document proxy links and checksum-verified
first-use installation.

# Simplify Aqua Integration — Master PRD

- Status: Complete
- Owner: Unassigned
- Created: 2026-09-01
- Review Status: APPROVED
- Review Count: 1
- Execution Mode: Lean
- Deep Mode Authorization: Not required
- Affected-File Budget: 25
- Actual Affected Files: 18

## Goal

Use Aqua's supported proxy/PATH integration without a custom command adapter,
while retaining project-local isolation and checksum enforcement.

## Scope

- Export the Aqua tool environment once through GNU Make.
- Remove the `scripts/aqua` interface and make all consumers invoke proxied tools
  normally.
- Align the pre-commit hook and CI with that same boundary.

## Non-Goals

- Changing Aqua, Node, npm, uv, registry, or checksum versions.
- Replacing rustup, adding a version manager, or changing product behavior.

## Inputs and Existing-Code Interactions

`scripts/aqua` currently has `install`, `exec`, and `env` modes. The prior
`aqua exec` layer caused npm to resolve a host Node interpreter before a PATH
patch was added. Aqua documents proxy links on PATH as the normal execution
model; its installer action defaults to `aqua install -l`.

## Boundaries and Abstraction Layers

Make owns local environment exports. Aqua proxies own executable resolution.
Stage 01 owns local command migration; Stage 02 consumes that boundary in hooks
and CI. `aqua.yaml` remains the declarative source of tool pins and checksums.

## Separation of Concerns and Decomposition

The local proxy boundary must exist before hooks and CI can consume it. The
stages separate local command behavior from external execution environments.

## Tech Debt and Spaghetti-Code Implications

Removing the custom adapter eliminates duplicated environment logic and reliance
on an Aqua command intended for proxy internals.

## Documentation Impact and Synchronization

Update `README.md` and `SECURITY.md` to describe proxy links and first-use
downloads. No product, API, or architecture documentation changes are needed.

## Stage Order and Links

1. [Stage 01 — Proxy Path Boundary](stage-01-proxy-path-boundary.md) — `Complete`
2. [Stage 02 — Ci Hook Convergence](stage-02-ci-hook-convergence.md) — `Complete`

Stage 01 establishes the local interface. Stage 02 removes adapter use from
non-Make entry points and CI without changing public Make targets.

## Cross-Stage Decisions

- `.tools/aqua` remains the default root, with `AQUA_ROOT_DIR` override support.
- Lazy installation remains enabled; `aqua install -l` creates proxy links.
- `AQUA_CONFIG` is omitted because Aqua discovers the repository config upward.
- Make and CI enforce checksums with `AQUA_ENFORCE_*`; Rust remains rustup-owned.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | Every managed executable uses Aqua proxies without `aqua exec`. | Clean-root setup and static search. | Node v24.20.0, npm 11.19.0, uv 0.12.1, and `npm exec -- node --version` all resolved through `.tools/aqua/bin`; search was empty. |
| AC-2 | Hooks and CI use the same project-local proxy boundary. | Pre-commit and workflow inspection. | Installed `project-validation` hook passed through `make project-precommit`; CI has workflow-scoped project-local settings, no adapter calls, and actionlint passed. |

## Implementation or Decision Tasks

- [x] Keep stage links and status summaries current.

## Verification and Observable Success Criteria

- [x] Run focused checks during implementation and record their results in the evidence table.
- [x] Run `make check` once before final review and once after fixes only when fixes were required.
- [x] Every stage's declared verification has passed.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [x] The final-code review gate has passed.

## Current Status

Complete. Final review found no blocking issues: the adapter is removed, the
Make and CI proxy boundaries are explicit, and `make check` passed once.

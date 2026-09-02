# Aqua Execution Path Correction — Master PRD

- Status: Complete
- Owner: Unassigned
- Created: 2026-09-01
- Review Status: APPROVED
- Review Count: 1
- Execution Mode: Lean
- Deep Mode Authorization: Not required
- Affected-File Budget: 25
- Actual Affected Files: 5

## Goal

Ensure every command invoked through `scripts/aqua exec` resolves every
Aqua-managed executable—including subprocess interpreters—from the
project-local Aqua bin directory.

## Scope

- Add the Aqua PATH boundary to `exec` mode as well as `env` mode.
- Prove an Aqua-launched shell resolves Node and npm from `.tools/aqua/bin`.

## Non-Goals

- Changing tool versions, checksum policy, CI topology, or Rust toolchain
  management.

## Inputs and Existing-Code Interactions

Merge validation showed `scripts/aqua exec npm ci` launched the correct npm
binary but its `#!/usr/bin/env node` interpreter resolved host Node v26.5.1.
`scripts/aqua env` already prepends `.tools/aqua/bin`, but `exec` did not.

## Boundaries and Abstraction Layers

`scripts/aqua` owns the process environment. Consumers retain their existing
`scripts/aqua exec` calls and do not duplicate PATH handling.

## Separation of Concerns and Decomposition

One cohesive wrapper correction fixes every existing consumer; splitting it
would add no independent contract.

## Tech Debt and Spaghetti-Code Implications

This removes an interpreter-resolution leak between the project tool boundary
and host PATH.

## Documentation Impact and Synchronization

No documentation changes required: the documented Aqua setup contract remains
unchanged; this corrects its implementation.

## Stage Order and Links

1. [Stage 01 — Local Execution Path](stage-01-local-execution-path.md) — `Complete`

The single stage corrects the shared execution boundary and verifies it.

## Cross-Stage Decisions

- The wrapper, not individual callers, prepends `.tools/aqua/bin` to PATH.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | `scripts/aqua exec` uses Aqua Node and npm. | Run path/version probe. | Both commands resolved from `.tools/aqua/bin` at the pinned versions. |

## Implementation or Decision Tasks

- [x] Keep stage links and status summaries current.

## Verification and Observable Success Criteria

- [x] Run focused checks during implementation and record their results in the evidence table.
- [x] Run `make check` once before final review and once after fixes only when fixes were required.
- [x] Every stage's declared verification has passed.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [x] The final-code review gate has passed: no P1/P2 findings in implementation review.

## Current Status

Complete. The merged tool boundary now prevents host Node lookup by npm and
the full validation gate passed.

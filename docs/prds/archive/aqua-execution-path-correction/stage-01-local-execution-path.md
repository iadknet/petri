# Stage 01 — Local Execution Path

- Status: Complete
- Depends on: None
- Master: [Master PRD](master-prd.md)

## Goal

Prevent host PATH lookup when Aqua-launched tools spawn sibling tools.

## Scope

- Prepend the Aqua bin directory in `scripts/aqua exec` mode.
- Verify `node` and `npm` resolve to their Aqua-pinned versions through exec.

## Non-Goals

- Changing the existing `env` mode or any pinned package version.

## Inputs and Existing-Code Interactions

The Node-provided `npm` entry point uses an env-based Node interpreter. Without
the Aqua bin directory on PATH, it ran with host Node during `make setup`.

## Boundaries and Abstraction Layers

`scripts/aqua exec` remains the caller interface; it becomes responsible for
setting the same PATH boundary as `env` mode.

## Separation of Concerns and Decomposition

This is a one-line shared-boundary fix plus a focused path regression check.

## Tech Debt and Spaghetti-Code Implications

The correction removes an otherwise hidden dependency on the host Node
interpreter.

## Documentation Impact and Synchronization

No documentation changes required: the public setup instructions already state
that Aqua provisions Node and npm; this fixes implementation conformance.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | Aqua exec resolves Node/npm from the project tool root. | Run path/version probe. | `node`/`npm` resolved from `.tools/aqua/bin` at v24.20.0/11.19.0. |

## Implementation or Decision Tasks

- [x] Prepend `.tools/aqua/bin` to PATH in `scripts/aqua exec`.

## Verification and Observable Success Criteria

- [x] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. The focused path/version probe, `make policy-check`,
`make quality-check`, and the full `make check` gate passed.

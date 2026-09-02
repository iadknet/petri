# <Title> — Master PRD

- Status: Draft
- Owner: Unassigned
- Created: <YYYY-MM-DD>
- Review Status: DRAFT
- Review Count: 0
- Execution Mode: Lean
- Deep Mode Authorization: Not required
- Affected-File Budget: 25
- Actual Affected Files: Pending

Use `scripts/prd-new` to create a conforming PRD set. If adapting this template
manually, replace every placeholder and keep metadata, links, evidence, and
checkboxes truthful.

## Goal

State the user or project outcome.

## Scope

- Define what this PRD set owns.

## Non-Goals

- Define nearby work this PRD set does not own.

## Inputs and Existing-Code Interactions

Describe repository evidence, dependencies, and affected existing code.

## Boundaries and Abstraction Layers

Define cross-stage boundaries and which abstraction level owns each decision.

## Separation of Concerns and Decomposition

Explain why these stages are independently coherent and dependency ordered.

## Tech Debt and Spaghetti-Code Implications

None identified. Revise this after concrete design analysis.

## Documentation Impact and Synchronization

Summarize durable documentation affected across stages. If none is affected,
record a concrete no-change rationale.

## Stage Order and Links

1. [Stage 01 — <Title>](stage-01-<stage-slug>.md) — `Draft`

Summarize stage ordering and contracts without duplicating stage tasks.

## Cross-Stage Decisions

- Record decisions shared by multiple stages.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | State an observable cross-stage outcome. | Name the focused check. | Pending |

## Implementation or Decision Tasks

- [ ] Keep stage links and status summaries current.

## Verification and Observable Success Criteria

- [ ] Run focused checks during implementation and record their results in the evidence table.
- [ ] Run `make check` once before final review and once after fixes only when fixes were required.
- [ ] Every stage's declared verification has passed.
- [ ] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [ ] The final-code review gate has passed.

## Current Status

Draft. Complete the content, then request the mode-appropriate readiness audit.

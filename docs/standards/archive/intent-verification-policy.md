# Intent Verification Policy

This policy defines a generalized, open-ended requirement for implementation completeness: project work must be verified against functional intent and integration behavior, not only contract shape.

## Golden Rule

- Completion requires verified functional intent and integration behavior; contract/shape checks alone are never sufficient.

## Scope

Applies to all current and future features, checkpoints, and surfaces (core runtime, transport, CLI, web, tooling, and docs-backed requirements).

## Core Rules

1. Contract checks are necessary but not sufficient.
   - Schema/envelope/serialization tests alone do not establish feature completion.
2. Functional intent must be explicitly tested.
   - Tests must verify the behavior the feature is intended to deliver.
3. Integration behavior must be explicitly tested.
   - Tests must verify the feature is fully wired in the owning runtime flow (not only unit-isolated behavior).
4. Negative/depletion/error paths must be covered where applicable.
   - If a feature has failure, depletion, invalid-state, or no-op paths, at least one regression must cover them.
5. Completion claims require intent evidence.
   - A checkpoint/feature cannot be marked complete when intent-level behavior remains unimplemented or unverified, even if contract tests pass.

## Generalized Verification Pattern

For each feature area, required evidence should include:

- one contract/shape assertion (request/response/schema/fixture)
- one intent-behavior assertion (what should actually happen)
- one integration assertion (behavior appears in the real execution path)
- one negative-path assertion (where relevant)

Equivalent domain-specific tests are allowed; exact suite names may evolve.

## Planning and Matrix Alignment

- Plans should encode intent-level acceptance criteria, not only interface checklists.
- Shared checkpoint matrices should include explicit intent/integration gates for each checkpoint area.
- Exceptions must be explicitly documented in plan `Open Questions` with owner and status.

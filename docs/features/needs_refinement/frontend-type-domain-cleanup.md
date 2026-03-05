---
title: Frontend Type File Domain Cleanup
tags: [frontend, architecture]
size: S
depends-on: []
status: needs-review
---

## Problem Statement

The frontend type files under `frontend/src/types/` have domain boundary violations where HTTP response envelope types live alongside the domain model types they happen to reference.

The primary offender is `CreatureDetail` (the HTTP response type for `GET /creature/:id`), which is defined in `src/types/genome.ts` (lines 116-129) even though it is not a genome type. It is an HTTP response envelope that aggregates fields from multiple domains: position, energy, age, generation, phenotype, genome, memory, and action log. It belongs in `src/types/http.ts` alongside the other HTTP response/request types (`LifecycleResponse`, `StartupResponse`, `StepResponse`, `StatusResponse`, `SnapshotResponse`, `ConfigResponse`, `StartupRequest`, `StepRequest`).

A secondary instance of the same pattern exists in `src/types/trace.ts`, which defines `SampleResponse` and `StartSampleResponse` (lines 106-120). These are HTTP response envelopes for `GET/POST /creature/:id/sample` but live in the trace domain file. Similarly, `PaintRequest`/`PaintResponse` in `src/types/paint.ts` mix HTTP envelope concerns with paint domain types, though that file is small and self-contained.

Fixing the primary case (`CreatureDetail`) is straightforward but touches the internal wiring of the `types/` directory. All consumers import through the barrel re-export in `src/types/api.ts`, so consumer import paths will not change.

### Current import chain

1. `CreatureDetail` is defined in `src/types/genome.ts` (line 116).
2. `src/types/api.ts` re-exports everything from `genome.ts` via `export * from "./genome.ts"`.
3. Consumers import from `../types/api.ts` (e.g., `src/api/rest.ts` line 4).

### Affected files (blast radius)

Direct definition site:
- `frontend/src/types/genome.ts` -- remove `CreatureDetail` interface (lines 116-129)

Destination:
- `frontend/src/types/http.ts` -- add `CreatureDetail` interface, add imports for `CreaturePhenotype` and `CreatureGenome` from `./genome.ts` and `ActionLogEntry` from `./action-log.ts`

No consumer-side import changes needed because all consumers import through the `api.ts` barrel, which already re-exports both `genome.ts` and `http.ts`.

## User Stories / Acceptance Criteria

- As a frontend developer, I want HTTP response types grouped in `http.ts` so that when I need to find or modify a response shape I look in one predictable location.
- As a codebase maintainer, I want type files to have clear domain boundaries so that `genome.ts` contains only genome/phenotype model types and `http.ts` contains only HTTP request/response envelope types.

### Acceptance Criteria

1. `CreatureDetail` interface is defined in `src/types/http.ts`, not in `src/types/genome.ts`.
2. `src/types/http.ts` imports `CreaturePhenotype` and `CreatureGenome` from `./genome.ts` and `ActionLogEntry` from `./action-log.ts` (the domain types that `CreatureDetail` references).
3. The barrel re-export in `src/types/api.ts` continues to work unchanged -- all existing consumer imports compile without modification.
4. `npm run build` passes with zero errors.
5. `npx tsc --noEmit` passes with zero errors.
6. No runtime behavior changes.

## Out of Scope

- Moving `SampleResponse`/`StartSampleResponse` out of `trace.ts` into `http.ts`. This is the same pattern but is a separate decision because those types have tighter coupling to the trace domain types in the same file. Can be addressed as a follow-up if desired.
- Moving `PaintRequest`/`PaintResponse` out of `paint.ts`. The file is small and cohesive enough that splitting it would add complexity without clarity.
- Renaming types (e.g., `CreatureDetail` to `CreatureDetailResponse`). That increases blast radius and is a separate concern.
- Refactoring the barrel re-export pattern itself (`api.ts` re-exporting all type files).

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `SampleResponse`/`StartSampleResponse` also move to `http.ts` in this task? | Excluded for now -- can be a follow-up | - | open |
| Should `PaintRequest`/`PaintResponse` also move to `http.ts`? | Excluded -- `paint.ts` is small and self-contained | - | open |
| Should `CreatureDetail` be renamed to `CreatureDetailResponse` for consistency with `LifecycleResponse`, `ConfigResponse`, etc.? | Deferred -- would increase blast radius since `rest.ts` references the type by name | - | open |

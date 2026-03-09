---
title: Split CreatureInspector Into Data and Sampler Sections
tags: [frontend, architecture]
size: S
depends-on: []
status: cancelled
---

## Cancellation Note

Cancelled after the unified inspector workspace landed and the legacy stacked inspector was removed. The original split proposed here targeted a pre-workspace `CreatureInspector.tsx` shape that no longer exists.

## Problem Statement

`CreatureInspector.tsx` (229 lines) mixes two distinct concerns in a single component:

1. **Data display orchestration** — subscribes to `useCreatureInspectorStore` for stats, genome, memory, and action log data. Renders `InspectorHeader`, `StatsSection`, `ActionTimeline`, `PhenotypeDetail`, `NodeGraph`, and `MemoryHexView`.

2. **Sampler orchestration** — subscribes to `useExecutionSamplerStore` for sample, playback state, position, and playback speed. Manages the `useExecutionSampler` hook, builds ~8 callbacks, derives tick/hop/detail totals and `activeNodeId`, handles creature-change cleanup via effect+ref, and renders `SamplerControls` and `SamplerPlaybackPanel`.

The single coupling point between the two concerns is `activeNodeId`, which is derived from sampler state and passed as a prop to `NodeGraph`. This narrow interface makes splitting straightforward.

Sub-components already live in `frontend/src/components/inspector/`. The parent orchestrator at `frontend/src/components/CreatureInspector.tsx` would become a thin shell delegating to `InspectorDataSections` and `SamplerSection` sub-orchestrators.

As the inspector grows (genome behavioral summary, annotated mesh, genome diff — all planned features), keeping both concerns in one file will make it increasingly difficult to reason about data flow and state ownership.

## User Stories / Acceptance Criteria

- As a frontend developer, I want the data display and sampler concerns separated so that I can modify one without understanding the other.
- As a contributor adding new inspector sections (genome summary, mesh annotations), I want a clear place to add data display components without touching sampler logic.

### Acceptance Criteria

1. `CreatureInspector.tsx` is a thin shell (~30-50 lines) that composes `InspectorDataSections` and `SamplerSection`.
2. `InspectorDataSections` owns all `useCreatureInspectorStore` subscriptions and renders data display components.
3. `SamplerSection` owns all `useExecutionSamplerStore` subscriptions, the `useExecutionSampler` hook, all sampler callbacks, and renders `SamplerControls` and `SamplerPlaybackPanel`.
4. `activeNodeId` is passed from `SamplerSection` to `InspectorDataSections` (or lifted to the parent) as the single coupling point.
5. No behavioral changes — the inspector looks and works identically.
6. `npm run build` passes with zero errors.

## Out of Scope

- Refactoring the inspector store itself (`useCreatureInspectorStore`).
- Refactoring the sampler store (`useExecutionSamplerStore`).
- Adding new inspector sections or tabs.
- Reorganizing the `inspector/` sub-component directory structure.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `activeNodeId` be lifted to the parent shell or passed via callback from SamplerSection? | Pending — depends on whether other future sections need it | - | open |
| Should the new sub-orchestrators live in `components/inspector/` or alongside `CreatureInspector.tsx`? | Pending | - | open |

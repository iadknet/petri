---
title: Fix Creature Inspector Crash
tags: [frontend, ui]
size: S
depends-on: []
status: needs-review
---

## Problem Statement

The creature inspector crashes with `TypeError: Cannot use 'in' operator to search for 'World' in ActionQueue` when selecting creatures whose genomes contain the `ActionQueue` input reference variant. This was introduced when `InputReference::ActionQueue` was added to the backend — a unit enum variant that serde serializes as the bare string `"ActionQueue"` rather than an object. The frontend's `inputRefUtils.ts` uses `"World" in ref` which requires an object operand and throws on strings.

**Root cause**: `frontend/src/types/genome.ts` defines `InputReference` with only 4 of the 5 backend variants (missing `ActionQueue`), and `frontend/src/components/inspector/inputRefUtils.ts` doesn't guard against string-typed refs before using the `in` operator.

## User Stories / Acceptance Criteria

- As a user, I want to click any creature on the map and see its inspector panel without crashes, regardless of which `InputReference` variants its genome uses.
- `InputReference` type in `frontend/src/types/genome.ts` must include `"ActionQueue"` as a valid variant.
- `formatInputRef()` and `inputRefColor()` in `inputRefUtils.ts` must handle string-typed refs (unit variants) before attempting `"key" in ref` checks.
- The `ActionQueue` ref should display a readable label (e.g. `"ActionQueue"`) and have a distinct color in the node graph tooltip.
- No error boundary should trigger when inspecting any creature in a running simulation.
- Add defensive `typeof` guards so that future unit-variant additions to backend enums (`InputReference`, `BackendDef`, `WorldAction`, etc.) degrade gracefully (show "unknown" label) instead of crashing. Audit all `"key" in value` patterns in inspector code and add string guards where the backend type is a serde-tagged enum that could gain unit variants.

## Validation

- Use `agent-browser` to validate the fix end-to-end in a running simulation:
  1. Start a simulation and seed/find a creature whose genome includes an `ActionQueue` input reference.
  2. Click the creature to open the inspector panel.
  3. Verify the inspector renders without errors, the `ActionQueue` input ref displays correctly in the node graph tooltip, and no error boundary is triggered.
  4. Verify the browser console has no `TypeError` related to the `in` operator.

## Out of Scope

- Refactoring the creature inspector into tabs (separate feature: "Refactor creature inspector" in ideas.md).
- Changes to the backend `InputReference` enum or serialization format.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `ActionQueue` get its own distinct color in the node graph, or reuse the gray "upstream" fallback? | Use a distinct color (e.g. orange/amber) since it's a meaningful input category | — | open |

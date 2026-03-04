**Goal:** Reconcile the frontend config panel with all backend SimulationConfig options so every tunable parameter has a UI control.
**Goal IDs:** GP-04 (Keep Behavior Observable)
**Scope:** Frontend TypeScript types, frontend config panel components, and the runtime patch field list. No backend changes.
**Docs Impact:** None — no canonical docs touched; only frontend code.
**Supersedes:** "Expose predation config in UI" idea in brainstorms/ideas.md
**Superseded-By:** none

## Goal Alignment

| Goal ID | Work Items |
|---------|-----------|
| GP-04 | Every backend config field gets a UI control, making all simulation parameters observable and tunable without code changes. Fixes stale field naming that obscures what's actually being configured. |

## Boundary Impact

Frontend-only changes. No crate boundaries, public APIs, or wire formats change. The server already serializes all fields — the frontend just doesn't consume them all.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| Server API wire format | Keep | All fields already serialized via serde; no server changes needed |
| Frontend config type ↔ server JSON | Change | TypeScript types must add missing fields and fix `hebbian_update_cost` → `plasticity_update_cost` |
| Config panel component architecture | Keep | Existing `FieldDef[]` + `RuntimeFieldGroup` pattern is clean and scalable — new sections follow the same pattern |
| Startup vs runtime panel split | Keep | `edge_mode` goes in startup panel (server enforces topology lock during non-Idle state) |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `edge_mode` be runtime-patchable or startup-only? | Startup-only — server already rejects topology patches during non-Idle state | agent | resolved |
| Is `hebbian_update_cost` → `plasticity_update_cost` a type issue or also API? | Both — server serializes `plasticity_update_cost`, frontend type says `hebbian_update_cost`. Works via serde alias but is a live mismatch. Fix frontend type and label. | agent | resolved |
| Should `max_actions_per_turn` be exposed? | Yes — standard runtime limit, bounded 1–20 is safe | agent | resolved |
| Are any fields intentionally hidden? | No — all gaps are oversights; `#[serde(default)]` on newer fields confirms they were added without frontend updates | agent | resolved |

## Required Skills

- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns`
  BEFORE writing any frontend code and before each review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE writing any UI
  code; use `agent-browser` for screenshot-based design validation after each step

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines` review after each step.
Repeat screenshot + review until clean (recursive).

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Implementation Steps

### Phase 1: Type alignment

- [x] **Fix TypeScript config types** (`frontend/src/types/config.ts`):
  - Rename `hebbian_update_cost` → `plasticity_update_cost` in `RuntimeConfig`
  - Add `reward_learning_cost: number` to `RuntimeConfig`
  - Add `max_actions_per_turn: number` to `RuntimeConfig`
  - Add `perception: PerceptionRuntimeConfig` to `RuntimeConfig` (new interface: `{ vision_radius: number }`)
  - Add `action_queue_cap: number` to `MutationConfig`
  - Add `age_cost: AgeEnergyCostConfig` to `EnergyConfig` (new interface: `{ enabled: boolean; age_cap: number; max_multiplier: number }`)
  - Add `predation: PredationConfig` to `SimulationConfig` (new interface: `{ steal_cost_rate: number; kill_complexity_bonus_multiplier: number }`)
  - Update any imports/references to `hebbian_update_cost` across the frontend

### Phase 2: Fix existing stale control

- [x] **Fix Runtime section stale field** (`frontend/src/components/config-panel/runtime/RuntimeSection.tsx`):
  - Change path from `runtime.hebbian_update_cost` → `runtime.plasticity_update_cost`
  - Change label from "Hebbian Update Cost" → "Plasticity Update Cost"
  - Update tooltip text accordingly

### Phase 3: Add missing runtime controls

- [x] [parallel] **Add Age Cost section** — new `AgeEnergyCostSection.tsx` following `ComplexityEnergyCostSection.tsx` pattern:
  - Toggle: `energy.age_cost.enabled` (default: true, tooltip: "When enabled, older creatures pay higher energy costs for all actions")
  - Field: `energy.age_cost.age_cap` (min: 1, max: 5000, step: 10, default: 500, tooltip: "Age in ticks at which the maximum cost multiplier applies")
  - Field: `energy.age_cost.max_multiplier` (min: 1, max: 50, step: 0.5, default: 10, tooltip: "Maximum energy cost multiplier applied to creatures at or beyond age cap")
  - Wire into `RuntimeConfigPanel.tsx` after Complexity Cost section
  - Add fields to `RUNTIME_PATCH_FIELDS`

- [x] [parallel] **Add Predation section** — new `PredationSection.tsx` following `RuntimeFieldGroup` pattern:
  - Field: `predation.steal_cost_rate` (min: 0, max: 1, step: 0.01, default: 0.2, tooltip: "Fraction of attempted steal amount paid as attacker energy cost")
  - Field: `predation.kill_complexity_bonus_multiplier` (min: 0, max: 1, step: 0.01, default: 0.05, tooltip: "Energy bonus per unit of victim genome complexity on kill")
  - Wire into `RuntimeConfigPanel.tsx` after Mutation section
  - Add fields to `RUNTIME_PATCH_FIELDS`

- [x] [parallel] **Add missing Runtime fields** to `RuntimeSection.tsx`:
  - Field: `runtime.reward_learning_cost` (min: 0, max: 10, step: 0.001, default: 0, tooltip: "Energy cost per reward-modulated weight update during learning")
  - Field: `runtime.max_actions_per_turn` (min: 1, max: 20, step: 1, default: 10, tooltip: "Maximum number of actions a creature can queue per turn")
  - Field: `runtime.perception.vision_radius` (min: 1, max: 8, step: 1, default: 5, tooltip: "Vision radius for extended perception area summaries")

- [x] [parallel] **Add missing Mutation field** to `MutationSection.tsx`:
  - Field: `mutation.action_queue_cap` (min: 1, max: 16, step: 1, default: 4, tooltip: "Maximum number of actions in the creature action queue")

### Phase 4: Add startup control

- [x] **Add edge_mode to startup panel** (`WorldTopologySection.tsx`):
  - Add a select/dropdown for `world.edge_mode` with options "Wrap" (default) and "Bounded"
  - This needs a new `SelectFieldDef` type or inline implementation since existing `FieldDef` only supports numeric inputs
  - Wire the value through the startup preset → config path

### Phase 5: Verification

- [x] **Verify all controls work end-to-end**: start the app, confirm each new control appears, modify values, apply, and verify the server receives correct patches

**Review cycles:** 1

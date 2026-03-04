---
title: Reconcile Config Panel with Backend Options
tags: [frontend, ui]
size: M
depends-on: []
status: needs-review
---

## Problem Statement

The frontend config panel has drifted out of sync with the backend `SimulationConfig`. Several backend config fields have no corresponding UI controls, at least one field name is stale (`hebbian_update_cost` vs `plasticity_update_cost`), and the logical grouping of controls could be improved as the config surface has grown.

This creates a poor operator experience — users can't tune all simulation parameters without editing code or sending raw API calls.

## User Stories / Acceptance Criteria

- As a simulation operator, I want every runtime-tunable config field to have a UI control so that I can experiment without editing code.
- As a simulation operator, I want config controls grouped logically so that related parameters are easy to find.
- As a simulation operator, I want startup-only fields clearly separated from runtime-patchable fields so that I understand when changes take effect.
- As a developer, I want the frontend TypeScript config types to match the backend Rust config structs exactly so that there are no stale or missing fields.

## Gap Inventory

| Backend Field | Config Section | Status |
|---------------|---------------|--------|
| `world.edge_mode` | WorldConfig | Missing from frontend |
| `energy.age_cost.enabled` | AgeEnergyCostConfig | Entire section missing |
| `energy.age_cost.age_cap` | AgeEnergyCostConfig | Entire section missing |
| `energy.age_cost.max_multiplier` | AgeEnergyCostConfig | Entire section missing |
| `runtime.plasticity_update_cost` | RuntimeConfig | Frontend has stale name `hebbian_update_cost` |
| `runtime.reward_learning_cost` | RuntimeConfig | Missing from frontend |
| `runtime.max_actions_per_turn` | RuntimeConfig | Missing from frontend |
| `runtime.perception.vision_radius` | PerceptionRuntimeConfig | Missing from frontend (entire sub-config) |
| `mutation.action_queue_cap` | MutationConfig | Missing from frontend |
| `predation.steal_cost_rate` | PredationConfig | Entire section missing |
| `predation.kill_complexity_bonus_multiplier` | PredationConfig | Entire section missing |

## Proposed Grouping Reassessment

Current groups and suggested changes (to be confirmed during planning):

- **Food Parameters** — keep as-is
- **Population** — keep as-is
- **Energy > Lifecycle** — keep as-is
- **Energy > Costs** — keep as-is
- **Energy > Complexity Cost** — keep as-is
- **Energy > Age Cost** — NEW section for `AgeEnergyCostConfig`
- **Predation** — NEW section for `PredationConfig`
- **Runtime > Execution Limits** — consider splitting current Runtime into sub-groups (execution limits vs graph convergence vs perception)
- **Runtime > Perception** — NEW sub-section for `PerceptionRuntimeConfig`
- **Mutation** — keep as-is, add `action_queue_cap`
- **World** — add `edge_mode` (startup-only)

## Out of Scope

- Adding new backend config fields that don't yet exist
- Redesigning the config panel layout/styling (only logical grouping changes)
- Config validation UI (showing warnings for dangerous parameter combinations)
- Save/load config presets (separate feature idea)

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `edge_mode` be runtime-patchable or startup-only? | Likely startup-only (changing wrap mode mid-sim could cause issues) | — | open |
| Is the `hebbian_update_cost` → `plasticity_update_cost` rename just a frontend label issue, or is the API field name also wrong? | Need to check server API wire format | — | open |
| Should `max_actions_per_turn` be exposed or is it too dangerous for casual tuning? | — | — | open |
| Are there any fields intentionally hidden from UI (design decisions vs oversights)? | — | — | open |

---
title: Storage Slots
tags: [simulation, core, frontend]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

Creatures currently have no way to interact with the environment beyond eating food on their current cell and moving. Adding an inventory system with pickup/place actions gives creatures the ability to manipulate barriers and food spatially, enabling emergent tool-use behaviors like barrier construction, food hoarding, and environment shaping.

## User Stories / Acceptance Criteria

- As a simulation observer, I want creatures to be able to pick up barriers and food from adjacent cells and store them in inventory slots, so that creatures can develop environment-manipulation behaviors.
- As a simulation observer, I want creatures to be able to place stored items onto adjacent cells, so that creatures can build structures and redistribute food.
- As a simulation observer, I want to see a creature's inventory state in the inspector, so that I can understand what a creature is carrying.
- As a simulation observer, I want creatures to have introspection inputs for their inventory (slot occupancy and item kind), so that creature cognition can reason about what they're carrying.

## Detailed Design

### Inventory State
- Fixed slot count controlled by `inventory.slot_count` config (default `5`)
- Each slot stores: `Empty`, `Barrier`, or `Food { density }`
- Creature-local runtime state; offspring spawn with empty inventory
- Slot count comes from config, not inheritance

### Actions
- `Pickup { direction, slot }` — pick up from adjacent cell into a slot
- `Place { direction, slot }` — place from a slot onto adjacent cell
- Both use existing 8-neighbor `Direction` model only (no self-cell)
- Current-cell food remains `Eat`-only

### Pickup Semantics
- Resolve target neighbor using existing world edge rules
- Fail if: target unresolved, occupied by creature, slot invalid/full
- Barrier on target: remove barrier, store `Barrier` in slot
- Food on target (density > 0.0): clear food, store `Food { density }`
- No barrier or food: action fails
- Barrier takes priority over food if both somehow present

### Place Semantics
- Resolve target neighbor using existing world edge rules
- Fail if: target unresolved, slot invalid/empty
- `Barrier`: succeeds only if target unoccupied and barrier-free; clears any food on placement
- `Food { density }`: succeeds only if target is not a barrier and adding density would not exceed `world.food.max_density`; fails on overflow (no clamping, conserves density)

### Introspection
- Initial: expose slot occupancy and item kind only (not stored food density)
- Stored food density preserved internally and visible in inspector/debug surfaces

## Out of Scope

- Self-cell pickup/place
- Inheriting inventory to offspring
- Exposing stored food density to creature cognition (future enhancement)
- Inventory mutations during reproduction
- Dropping all items on death (could be a follow-up)

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should items drop on creature death? | Defer to follow-up feature | User | resolved |
| Should pickup/place have energy costs? | Yes, use existing action energy cost model | User | resolved |
| Should slot count be evolvable? | No — config-only for now | User | resolved |

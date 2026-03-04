---
title: Refactor Movement to Forward/Turn Model
tags: [simulation, core, frontend]
size: XL
depends-on: []
status: needs-review
---

## Problem Statement

The current movement system uses 8 cardinal/diagonal direction actions (MoveN, MoveNE, MoveE, etc.). This gives creatures omnidirectional movement with no concept of facing or orientation. A forward/turn model — where creatures face a direction, can turn left/right, and move forward — creates more realistic movement patterns, enables directional behaviors (forward-facing predation, field-of-view sensing), and reduces the action space from 8 movement actions to 3 (turn left, turn right, move forward).

This is an XL feature because it touches the action system, creature state, sensor inputs, predation mechanics, the frontend renderer, and potentially the genome/mesh structure. It is kept unified because orientation is a single coherent concept that must be consistently applied across all these systems — splitting it would create intermediate states where some systems know about orientation and others don't.

## User Stories / Acceptance Criteria

- As a simulation observer, I want creatures to have a facing direction, so that movement and predation feel more natural and directional.
- As a simulation observer, I want to see which direction a creature is facing in the zoomed-in view (e.g., a pointy tip), so that I can understand creature behavior visually.
- As a simulation designer, I want turn actions to have a small but nonzero energy cost, so that turning is not free but also not prohibitive.
- As a simulation designer, I want predation to be forward-facing only, so that creatures must orient toward prey.

## Detailed Design

- Add `facing: Direction` to creature state (one of 8 cardinal/diagonal directions)
- Replace 8 MoveX actions with: `TurnLeft`, `TurnRight`, `MoveForward`
- Turn actions rotate facing by 45 degrees (one step in the 8-direction system)
- MoveForward moves the creature one cell in its facing direction
- Turn energy cost: small, configurable (default: low but nonzero)
- Predation: only targets the cell the creature is facing
- Offspring inherit parent's facing direction (or random — TBD)
- Zoomed-in creature rendering: add a directional indicator (pointed tip)

## Out of Scope

- Field-of-view / cone-based sensing (follow-up feature)
- Backward movement
- Strafing (sideways movement)
- Multi-step turns (e.g., 180-degree turn as single action)

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| What should the default turn energy cost be? | Start low (~10% of move cost), tune later | User | resolved |
| Should offspring inherit parent facing or get random? | Needs discussion | User | open |
| Should existing sensor inputs change to be relative to facing? | Yes for neighbor sensing, deferred for others | User | resolved |
| How to handle genome migration for existing saves? | Map old MoveN/S/E/W to TurnTo+MoveForward sequences | User | open |

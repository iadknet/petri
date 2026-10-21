# T28 — Niche Construction

**Status**: Planned
**Last updated**: 2026-10-20
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Creatures carry food and barriers in four slots, set them down elsewhere and
pay to haul them, so the world they leave behind is partly one they built.
Design, options and user decisions:
[interaction features design note](../strategy/interaction-features-design-2026-10-20.md);
review: [T26–T28 design review](../progress/readings/t26-t28-design-review.md).

## Track Success Criteria

- [ ] A creature has four slots, picks up into and places from a named slot in a chosen direction, senses each slot's content, and pays a movement surcharge per occupied slot.
- [ ] Food can be picked up, carried and placed, and barriers dug out and built; transfers conserve material, and every loss (overflow, failed death drop, food under a built barrier) is a recorded sink, separate from growth and consumption.
- [ ] Both run in all three goal worlds, and the goal readings report pick-ups, placements, Canyon's remaining wall count, and the final barrier mask's largest connected open component and disconnected occupied regions, including zero.

## Executable Features

- [ ] **T28.F01 — Carrying and Food Caching** — Depends on: T26.F02, T27.F03
  - Goal: Caching and seed dispersal: a creature picks food into one of four slots, carries it at a cost and sets it down elsewhere, where it can be eaten later or grow.
- [ ] **T28.F02 — Digging and Building** — Depends on: T28.F01
  - Goal: Burrows and walls: a creature digs a barrier out of the ground into a slot and builds it elsewhere, so walls outlast their makers and shape who comes after.

## Notes for AI Agents

- User decisions (2026-10-20): four slots with a slot-index parameter per
  action (`PickUpSlot`, `PlaceSlot`, rounded to 0..3) and per-slot self
  inputs reading 0 empty, 1 food, 2 barrier; directions use the drive plus
  eight direction sinks; every barrier is movable, nothing decays, and
  Canyon's wall drift is a reported finding, not a blocker. The research
  objections to index parameters and ordinal codes are recorded in the
  design note §3 E; the specs report per-slot use.
- F01 lands `WorldState::place_barrier/remove_barrier`, which hold the
  paint tool's rules except that a creature's placement rejects an occupied
  cell rather than evicting. Creature-moved barriers reach clients through
  the existing projection publication, which already compares the barrier
  mask (`crates/v3-server/src/query/projection.rs`); F01 adds a
  creature-edit → publication → client-update fixture and no second
  revision counter (review finding 3). F01 extends T27.F01's input-bucket
  table with its slot inputs.
- Carried food cannot be eaten from a slot. It is placed on the ground and
  eaten there (user ruling, 2026-10-20, review round 5 finding 6).
- F01 owns the live food contract; the archived T06.F01 contract covers
  barriers only:
  - pick-up takes one bite of food (the spec justifies a densest-type rule
    as body mechanics or lets the type be chosen) as
    `(type, amount)` within the configured density bounds;
  - placement into a partly full cell either keeps the remainder in the slot
    or rejects atomically (the spec chooses);
  - food death-drop validity, and exactly-once biological drops distinct
    from administrative removal (paint eviction), which discards;
  - carried stock, overflow and destruction as named ledger buckets;
  - successful and failed handling charges, and the load surcharge through
    T26.F01's move-effort function (archived start 0.05 per slot);
  - inspection, frontend types and trace surfaces for slots.
  Open for the spec: placed food against grazing and fertility state.
- F02 owns food displaced or destroyed by a built barrier (growth zeroes
  food on barrier cells) and barrier death drops per the archived T06.F01
  contract. It adds no sinks or parameters; its expected pin changes come
  from applied barrier behavior.
- F01 depends on T26.F02 because both change the move-effort function, and
  on T27.F03 because both add directed kinds to the same catalog.
- Both are creature-authored pressures under the
  [shared baseline contract](../workflow.md#environmental-pressures-in-the-standard-baseline).
- Verification: F01 viability first, gate + goal; F02 viability first,
  gate + goal + goal-long (the track's one goal-long carrier). Each spec
  predeclares its expected changes by artifact.

# T06.F02 — Food Transport and Caching

**Status**: Planned
**Last updated**: 2026-09-23
**Feature**: T06.F02
**Track**: [T06 — Niche Construction and Ecological Inheritance](../../roadmaps/t06-niche-construction-and-ecological-inheritance.md)

## Goal

Creatures carry uneaten food in the same four slots used for barriers, eat it
later, or deposit it where any creature can discover and use it. Food retains
its properties and continuing decay clock through transfers; caching competes
with construction for capacity and incurs loaded movement cost.

## Non-Goals

- Additional inventory capacity, evolving capacity, containers, private caches,
  ownership enforcement, direct gifts, or automatic cache-seeking behavior.
- Internal fat/reserve traits, digestion specialization, new food types, or
  byproduct conversion. Existing ordinary typed food and per-type nutrition remain authoritative.
- Merging conflicting deposited food, resetting freshness on pickup, or growing
  harvested food while it is carried or cached.
- Confirmatory evidence that hoarding or cognition evolves from this mechanism.

## Inputs and Invariants

The owning roadmap row is authoritative for dependencies.
[T06.F01](t06-f01-material-carrying-and-barrier-construction.md) supplies slots,
loaded movement cost, action failure conventions, and ordered death drops.

Use the [world food contract](../../reference/v3-world-grid-spec.md),
[tick order](../../reference/v3-tick-orchestration-spec.md), and existing ordinary
food implementation in `crates/v3-core/src/kernel/ordinary_food/` and typed
sensors in `crates/v3-core/src/sensors/typed_food.rs`. Current food is stored in
per-type density planes, with per-run stable type IDs. Feeding uses each type's
effective `energy_per_unit`, falling back to `energy.costs.eat_reward_per_food`
when no per-type value is configured. Storage preserves that food-type identity
and uses the same effective nutrition as ground consumption.
Internal energy storage in T03.F05 is not a prerequisite for carrying uneaten food.

Research considered contextual pickup/drop and separate operations (the
[CoGrid actions](https://cogrid.readthedocs.io/en/latest/concepts/actions/) and
[MiniGrid actions](https://github.com/Farama-Foundation/Minigrid/blob/main/minigrid/core/actions.py),
reviewed 2026-09-04). Retain separate intent, extend the existing typed food
substrate with bounded harvested-food state, and reuse F01 inventory. A separate
cache ownership system or external inventory dependency would add semantics the
requested public caches do not need.

Food and slot contract:

- Each slot is empty, one barrier, or one food parcel containing one food type,
  a positive bounded quantity, and all intrinsic food properties supported by the
  simulation at implementation time. Do not invent traits to fill the parcel.
- A parcel's capacity equals the world's ordinary-food per-type maximum at run
  initialization; it is not an independently evolving trait. A pickup moves up
  to that capacity into a selected empty slot and leaves any remainder at source.
  Partially filled slots cannot be topped up or merged in this version.
- Every nonempty food slot incurs exactly the F01 per-slot surcharge regardless
  of quantity or type. Four shared slots allow mixed food/barrier loads.
- Pickup and placement transfer actual food, never energy. Type, quantity,
  intrinsic properties, and any existing
  age/expiry survive transfer unchanged except for elapsed-time decay. Splitting
  a source preserves its properties on both portions. Cell fertility and position
  are environmental properties and do not travel with food.
- Preserve property-bearing parcels in the applied world representation rather
  than flattening them into a density value that loses identity or freshness.
  World sensors and existing Eat actions see the actual available quantity.

Actions and feeding:

| Action | Parameters | Successful effect |
| --- | --- | --- |
| Pick up food | Food type, empty destination slot | Transfer up to one parcel capacity from the actor's current cell. |
| Place food | Food source slot | Transfer the whole parcel to the actor's current cell, replacing conflicting food. |
| Eat from storage | Food source slot | Consume that parcel using its type's effective energy per unit and the existing energy cap, then empty the slot. |

- Retain current ground Eat behavior. Make stored-food consumption an explicit
  controller choice, not an automatic fallback when ground food is absent.
- Pickup/placement use F01 handling cost conventions; their initial base costs
  equal barrier pickup/placement respectively. Eating from storage uses existing
  Eat cost and the selected type's effective energy per unit, including its
  shared-default fallback. Handling yields no energy. New actions use F01's
  extension of the existing vote interface.
- Deliberate placement permits the acting creature's own occupancy. It still
  rejects barriers and other actual occupation blockers; no adjacent target is
  chosen. On failure the parcel stays in its slot and destination is unchanged.
- Invalid slot/type, barrier contents for food actions, empty source, or occupied
  pickup slot fail through normal outcome/energy accounting. Transfers are atomic.
- At cognition snapshot time, controllers can read each slot's kind, food type,
  quantity, and remaining lifetime (when applicable), as well as occupied count.
  Existing food perception exposes public cache food without owner filtering.

Overwrite and ordering:

- A conflict means existing food of the same type in the destination cell,
  whether naturally growing or previously deposited. Other food types coexist
  in their existing planes and remain unchanged.
- A successful placement replaces the entire conflicting amount and its
  metadata with the incoming parcel; it does not add, average, or merge them.
  Replaced food is destroyed and that quantity is accounted for explicitly.
- Deliberate actions apply in normal sequential order against current world
  state. Among successful conflicting placements, the last applied parcel wins.
  This is not a new arbitration pass or a change to movement's occupancy rules.
- Death drops use F01's fixed slot-to-cardinal-neighbor mapping and order. Any
  creature occupancy blocks a death food drop; existing food does not. Successful
  food drops use the same overwrite rule; failed drops destroy the carried parcel
  and leave the destination unchanged. No retry or search occurs.
- The same rules cover aliased wrapped neighbors and different creatures' drops.
  An earlier barrier blocks later food; earlier food does not block later barrier
  placement. Food underneath a barrier remains subject to its clock.

Clock and world ecology:

- Reuse any intrinsic food aging/decay policy already supplied by prerequisites,
  preserving its state across all transfers. For ordinary food that has no such
  clock, first harvest creates an absolute expiry at `pickup_tick + lifetime`.
  Initial harvested-food lifetime is 256 ticks, configurable as a positive integer
  at run initialization. This is a balancing default, not a measured optimum.
- For this fallback policy quantity stays unchanged until expiry;
  at tick `>= expires_at` the parcel disappears. Re-pickup, placement, death drops,
  or changing carrier never restart the clock. Food with an existing intrinsic
  policy continues that policy rather than gaining a second freshness reset.
- Process carried and cached expiry once in Phase 0 before cognition, growth,
  and death drops. Expired slots become empty and immediately stop adding carrying
  cost. Preserve the order of the other existing Phase 0 operations.
- A cached parcel is harvested food, not a new growing food substrate. While
  it occupies a cell/type, exclude it from natural growth and spread as a source,
  and prevent natural growth/spread/recovery from adding to that same cell/type.
  Other food types continue normally. Consumption, pickup that empties it, expiry,
  or explicit replacement removes its metadata; ordinary ecology can resume.
- No new timing-dependent loss is allowed through serialization or config changes.
  Preserve clocks and properties in existing state persistence paths. Reject
  mid-run catalog/capacity changes that would invalidate inventory or caches;
  do not silently retype or truncate them. Do not add a save format if none exists.

## Implementation Tasks

- [ ] Extend F01's four slots with bounded typed food parcels and continuing clock
      state; integrate inventory sensors and existing inspection/serialization.
- [ ] Add pickup, placement, and stored-food consumption through supported
      controller backends, mutation reachability, action logging, and accounting.
- [ ] Implement applied-world parcel preservation and same-type replacement for
      deliberate placement and death drops, preserving other food planes.
- [ ] Integrate carried/cache expiry and cache growth exclusions into ordinary
      food ecology with no freshness reset or duplicate expiry processing.
- [ ] Expose quantities picked up, cached, consumed from storage, expired,
      overwritten, and destroyed on death from actual outcomes, distinguishing
      transfers from energy gains and world growth.
- [ ] Update canonical food, action, sensor, lifecycle, config, and transport
      references for implemented behavior.

## Verification

- [ ] Use TDD for food actions, overwrite, expiry, and death behavior. Property-test
      quantity accounting, type/property preservation, slot capacity, and atomic
      failure, with explicit consumption/expiry/overwrite/destruction sinks.
- [ ] Verify mixed four-slot loads, partially filled occupied slots, no top-up,
      bounded pickup and remainder, insufficient storage, and invalid selections.
- [ ] Verify pickup yields no energy; stored eating uses the same effective
      per-type nutrition as ground eating, including explicit overrides and the
      shared fallback, with existing caps and costs; placing then eating cannot
      multiply food or energy.
- [ ] Verify pickup/place/re-pickup and death drops preserve food properties and
      expiry; test just before, at, and after expiry while carried and cached.
- [ ] Verify 0.25 units replacing 0.80 same-type units leaves 0.25, not 1.05 or
      0.80; another type in that cell remains unchanged. Verify the replacement
      metadata belongs to the incoming parcel.
- [ ] Verify acting-creature occupancy permits deliberate placement; any creature
      occupancy blocks a death drop; blocked deliberate placement retains its
      parcel while a blocked death drop destroys it without altering destination.
- [ ] Verify sequential same-type death-drop conflicts, tiny wrapped aliases,
      mixed barrier/food order, and all F01 biological death paths with food.
- [ ] Verify public sensing and consumption by another creature, no cache growth
      or incoming same-type spread/recovery, normal other-type growth, ecology
      resumption after removal, and expiry underneath barriers.
- [ ] Verify controller decoding, applied telemetry, persistence round trips where
      supported, and rejection of invalidating config changes.
- [ ] Run `cargo test -p v3-core --test viability` first when changing defaults or
      tick-loop mechanics, focused tests, `make roadmap-check`, and `make check`.
- [ ] Benchmark summary stored at `docs/progress/features/t06-f02.json`, with raw
      reports and provenance under the [artifact contract](../../benchmark-artifacts.md).

## Performance and Goal Impact

Natural analog: transporting food to survive lean periods or depositing caches
that competitors can discover. Effects reach creatures through actual food,
shared carrying capacity, handling cost, and loaded movement. Caches have no
owner preference or direct reward for their creator.

Expected incremental cost is bounded food data in four slots, metadata for
occupied cached cell/type pairs, constant work per transfer, and expiry work
proportional to carried/cached parcels. Avoid full-world scans per action and
avoid storing large metadata records for every empty cell/type. This forecast
does not authorize a severe regression or a baseline change.

At closure record deterministic work and wall-clock deltas per creature-tick
against the previous closed feature and pinned epoch baseline, threshold results,
dated goal-profile indicator readings, and the verification and benchmark records
required by `docs/workflow.md`. Include
applied food transfers and losses in existing reporting. This feature introduces
no new diversity or cognition measure; a scripted cache demonstration proves the
mechanism, not that a caching strategy evolves.

## Success Criteria

- [ ] A creature can harvest, carry, and later consume food or leave it for another
      creature while sharing four cost-bearing slots with barriers.
- [ ] All transfers preserve intrinsic food properties and continuing clocks;
      successful placement overwrites only conflicting same-type food.
- [ ] Death drops obey the agreed adjacency, blocking, destruction, and ordering
      rules for mixed inventories without creating food or barriers.
- [ ] Caches are public, decay independently of their maker, and cannot grow or
      refresh through pickup/placement cycles.
- [ ] Required verification and performance evidence pass and are recorded.

## Notes for AI Agents

This is a requested advance design spec; execution follows `docs/workflow.md`
when requested. The same-type meaning of conflicting food follows the existing
multi-type world. Current-cell deliberate placement intentionally permits the
actor, whereas death placement does not permit any creature occupant.

The 256-tick fallback lifetime and F01 carrying cost are initial design choices
for an executable specification. Validate their ecological effects on the goal
profile during implementation; do not describe these values as user-selected or
experimentally established. Preserve intrinsic property/clock continuity if
prerequisite features introduce a richer food model before this feature runs.

Implementation verification, mutation survivor record, review findings, and cost
record: pending execution.

# T06.F01 — Material Carrying and Barrier Construction

**Status**: Planned
**Last updated**: 2026-09-04
**Feature**: T06.F01
**Track**: [T06 — Niche Construction and Ecological Inheritance](../../roadmaps/t06-niche-construction-and-ecological-inheritance.md)

## Goal

Creatures carry and relocate barriers using four storage slots, paying for
handling and loaded movement. The resulting walls, openings, and potential
burrows persist after their makers leave or die and affect movement and sight.

## Non-Goals

- Food transport, eating from storage, and food decay belong to T06.F02.
- Evolving slot capacity, different item weights, crafting, construction rewards,
  ownership rights, and automatic building plans.
- Separate burrow or trail layers, fertility changes, or barrier weathering.
- Confirmatory ecological-inheritance campaigns or claims that construction
  strategies necessarily evolve because the actions exist.

## Inputs and Invariants

The owning roadmap row is authoritative for dependencies. T02.F02 supplies
seeded barrier terrain and its visibility behavior. Use the existing applied
world, action, energy, and lifecycle machinery:

- [World geometry and occupancy](../../reference/v3-world-grid-spec.md).
- [Tick ordering](../../reference/v3-tick-orchestration-spec.md).
- [Sensors](../../reference/v3-sensor-spec.md).
- `crates/v3-core/src/contracts/actions.rs`, `runtime/action_decode.rs`, and
  `runtime/cgp/effects.rs` for controller-accessible actions.
- `crates/v3-core/src/simulation/actions/`, `simulation/tick.rs`, and
  `simulation/simulation.rs` for application and removal.

Research considered separate pickup/place operations, as in
[MiniGrid](https://github.com/Farama-Foundation/Minigrid/blob/main/minigrid/core/actions.py),
and a contextual combined operation, as in
[CoGrid](https://cogrid.readthedocs.io/en/latest/concepts/actions/)
(reviewed 2026-09-04). Separate operations match the requested controller
intent and allow unambiguous failure outcomes. Extend Petri's existing grid
and action machinery; neither an external environment dependency nor a generic
inventory framework is needed.

Storage contract:

- Every creature has exactly four indexed slots, `0..3`. Founders and offspring
  start empty; reproduction does not copy or transfer inventory.
- F01 supports empty slots and one barrier unit per occupied slot. F02 adds
  food to these same slots, not four additional slots.
- No stacking, automatic rearrangement, or implicit slot switching. Pickup
  names an empty destination slot; placement names an occupied source slot.
  Invalid indices or contents fail without changing inventory or terrain.
- Slot kind and occupied-slot count are available to the controller through
  normal sensor inputs and to the existing creature inspection surface.
  Sensors follow the existing cognition snapshot timing; action validity uses
  current applied state.

Action contract (names describe semantics; encodings follow existing conventions):

| Action | Parameters | Successful effect |
| --- | --- | --- |
| Pick up barrier | Adjacent direction, destination slot | Remove one barrier from the resolved neighbor and put it in the empty slot. |
| Place barrier | Adjacent direction, source slot | Remove the slot's barrier and install a barrier in a valid neighbor. |

- Directions use the existing eight-neighbor and wrap/bounded geometry.
  A neighbor resolving to the actor's own cell is invalid.
- Pickup requires an existing barrier and an empty destination slot. Existing
  seeded and creature-placed barriers are both movable; no new material source
  is added. Removing a barrier does not remove underlying food.
- Placement requires a cell free of barriers, creatures, and any other actual
  occupation blockers. Food is not an occupation blocker and remains underneath
  a placed barrier, subject to its own environmental rules.
- Each operation is atomic. A blocked deliberate placement retains the item;
  failed pickup retains both source material and inventory. No fallback target
  or slot is selected.
- Changes apply immediately in normal sequential action order. A barrier can
  block a later move or placement in that tick; removing one can enable a later
  action. Frozen cognition inputs are not recomputed mid-tick.
- Charge a handling action cost on every attempt, plus the existing failed-action
  penalty when appropriate. Use the existing energy adjustment and affordability
  rules, without double-charging. Initial handling base costs equal the existing
  base move cost; expose finite nonnegative values through the existing config.
- Loaded movement has base cost `move_cost + occupied_slots * carry_cost` before
  the existing age/complexity adjustments. The surcharge applies to attempted
  moves, including blocked moves, just as the existing move cost does. Initial
  `carry_cost` is `0.05` energy per occupied slot per attempt. Empty slots add
  nothing. Costs and losses are recorded from applied outcomes.

Persistence and death contract:

- Relocated barriers have no autonomous decay in this feature. They remain until
  a normal world mutation removes them, independent of constructor survival.
  Record which current barriers were creature-placed and which seeded positions
  were cleared by creatures so inspection can distinguish construction from
  initial terrain; metadata does not grant ownership or alter collision rules.
- At each actual death removal, clear the dying creature's occupancy and attempt
  one drop for each occupied slot, in index order: `0=N, 1=E, 2=S, 3=W`, relative
  to its last position. Empty slots do nothing. No retry or search occurs.
- Resolve each destination with existing edge rules. An unresolved bounded
  neighbor, the origin itself after wrapping, or a currently blocked destination
  destroys that item without mutating the destination. Successful barrier drops
  use the same placement effect as deliberate placement, at no extra energy cost.
- In tiny wrapped worlds, two directions may alias the same non-origin cell.
  Process them in slot order against current state; the first barrier can block
  the second. Do not silently relocate or deduplicate attempts.
- Preserve existing death timing. For a Phase 0 batch, process dead creatures in
  stable CreatureId order, removing and dropping each before the next; a creature
  awaiting removal still occupies its cell. During sequential actions, resolve
  death drops at the actual removal point. Repeated removal cannot duplicate drops.
- Cover starvation, action-cost death, and lethal predation, including paths
  that currently remove victims directly. Administrative reset/paint eviction
  discards inventory without ecological death drops; it is not creature death.
- Material is conserved by successful pickup/place. Failed death drops are an
  explicit destruction sink, not a conservation failure. Track this sink.

## Implementation Tasks

- [ ] Add four-slot creature state, empty initialization, inventory sensors, and
      inspection/serialization support through existing surfaces.
- [ ] Add barrier pickup/place contracts, decoding and execution for supported
      controller backends, with mutation reachability and accurate action logs.
- [ ] Apply handling costs and occupied-slot movement surcharge through shared
      energy accounting and validated configuration.
- [ ] Implement immediate terrain transfers, construction provenance, and
      consistent movement/visibility projections from applied world state.
- [ ] Integrate exactly-once ordered death drops into every biological death
      path while distinguishing administrative removal.
- [ ] Update affected canonical action, sensor, lifecycle, world, and transport
      references to the implemented behavior; keep this spec's scope intact.

## Verification

- [ ] Use TDD for action and death behavior; property-test slot bounds, atomic
      transfers, and barrier conservation with explicit destruction accounting.
- [ ] Verify four occupied slots prevent additional pickup; founders and children
      start empty; invalid slots, empty sources, and occupied destinations fail.
- [ ] Verify pickup opens a route and sight line, placement closes them, and
      effects survive constructor movement and death across subsequent ticks.
- [ ] Verify costs at zero through four occupied slots, blocked moves, failed
      handling, and energy exhaustion after an applied action.
- [ ] Verify cardinal death mapping, all blocked destinations, bounded edges,
      wrapped aliases/origin, sequential batch deaths, predation, repeated
      removal, and administrative removal; no fallback or destination mutation
      occurs on a failed drop.
- [ ] Verify current-state contention, frozen sensor timing, supported controller
      decoding, and inspection/serialization round trips for inventory/provenance.
- [ ] Run `cargo test -p v3-core --test viability` first when changing defaults or
      tick-loop mechanics, focused tests, `make roadmap-check`, and `make check`.
- [ ] Benchmark report stored at `docs/progress/features/t06-f01.json`.

## Performance and Goal Impact

Natural analog: carrying building material to open passages or construct walls
and burrows. Effects reach creatures through passability, occlusion, handling
costs, and loaded movement, without direct rewards for construction.

Expected cost is four fixed inventory entries per creature, constant work per
handling/move action, at most four attempts per death, and provenance for edited
terrain. Avoid scanning the full grid to find death-drop locations or rebuilding
visibility structures globally after each edit. This forecast does not authorize
a severe regression or a baseline change.

At implementation closure record deterministic work and wall-clock deltas per
creature-tick against the previous closed feature and pinned epoch baseline,
threshold results, dated goal-profile indicators, and a second-run determinism
check. Report applied construction, loaded movement cost, and destroyed drops
through existing reporting. No new diversity or cognition metric is introduced;
observing a construction does not prove adaptive ecological inheritance.

## Success Criteria

- [ ] Creatures can choose and execute barrier relocation using four cost-bearing
      slots, and other creatures encounter the resulting applied terrain.
- [ ] Construction persists independently of its maker and is distinguishable
      from initial terrain in inspection.
- [ ] Every biological death drops or destroys each carried barrier exactly once
      under the fixed adjacency/conflict rules.
- [ ] Required verification and performance evidence pass and are recorded.

## Notes for AI Agents

This is a requested advance design spec, not authorization to execute or commit
feature implementation. Follow `docs/workflow.md` when execution is requested.
The initial carrying cost is a design default, not an empirically established
balance value; any tuning must retain a positive per-slot tradeoff.

The tick reference currently says post-action deaths wait until Phase 0, while
`simulation/tick.rs` removes dead actors after actions and predation directly
removes victims. Preserve the current applied timing and correct the affected
reference during implementation; do not delay drops to match stale prose.

T06.F02 extends this inventory and death-placement contract with food. It owns
food-specific overwrite, nutrition, and clock rules. T06.F04 owns later habitat
property changes rather than reimplementing barrier relocation.

Implementation verification, mutation survivor record, review findings, and cost
record: pending execution.

# T18 — Founder Architecture

**Status**: Planned
**Last updated**: 2026-09-18
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The founder is born with a nervous system divided the way evolution is
asked to extend it: a sensory node that reads the world and writes what it
saw to memory, a router whose route is a live decision, one motor node per
action kind (eat, move, reproduce) with its own gate, and a terminal node
that commits the queue. Every one of those parts is an existing graph
primitive (`WriteSlot`, `RouterGate`, the per-kind action bank with its
direction bank, the execute gate); the founder today wires none of them and
runs its decisions through a 70-instruction VM program with 26 relative
jumps in which a tick is reproduce *or* eat-and-move. Afterward the
population's decision core starts as five small graph nodes on the executed
chain, so routing, memory, and per-action gates receive mutation supply
from tick zero and a lesion in one motor path no longer takes the others
with it. The natural analog is the functional segregation of a nervous
system into afferent, central, and efferent divisions with one motor pool
per effector. Evidence and the measured probe are the
[founder refactor research note](../strategy/founder-refactor-research-2026-09-18.md).

## Track Success Criteria

- [ ] A specialized-node founder profile exists, every node a graph
  backend, and on the founder tests it moves exactly as V3Alpha1 does
  (first-argmax over the cardinal ring, measured at 0 mismatches in 2,000
  random rings) while eating, moving, and reproducing in one tick.
- [ ] The T11.F14 battery and births probe carry founder rows for both
  profiles, so changed, silent, dead, and sterile fractions per birth are
  attributable to the layout (probe: sterile 2.7% against 4.6% per birth;
  `genome_size` 55 against 111).
- [ ] The specialized profile is the default founder, the gate and goal
  epochs are re-pinned on it, and V3Alpha1 remains selectable as the
  control.

## Executable Features

- [ ] **T18.F01 — Specialized-Node Founder Profile** — Depends on: T11.F21, T11.F22
  - Goal: A creature is born with a body plan for its brain: one node senses and remembers, one node per act moves the body, one node decides whether to breed, one node commits. New `FounderProfile` (wire name decided in the spec) built in `creature/founder.rs` from existing graph primitives only, chain `sense → eat → move → router → { reproduce → execute | execute }`: a sense node reading the V3Alpha1 inputs and writing food-here, the energy-and-age gate, and the cardinal primary-food ring to shared memory; eat gated on food-here; move always, with four direction-bid edges from the ring; a router node whose `RouterGate` reads the gate from memory and whose two targets are the reproduce node (bias 0, scored by the gate) and the executor (bias 0.5), so the route *is* the reproduce decision and the reproduce node's own gate is a constant (a decision lives in exactly one place; a router whose choice a downstream gate repeats is a knockout node, the live survey's non-contributing detour); reproduce placing the child behind the move (bids reversed, scalar direction S) with a constant transfer; a terminal node whose execute gate is a constant. Chain order is action order: Eat → Move → Reproduce, so the vacated cell is free for the child, and the router sits after the motor nodes so a route broken by mutation still leaves a queued action. Analog: Calabretta, Nolfi, Parisi and Wagner 2000 start with one gated module per effector and let duplication specialize them; Nolfi 1997's hand-partitioned state router (its architecture D) reached the plateau later than the per-effector form, so the spec reads this routed layout beside the serial-gated variant the research note probed (reproduce gated in its own node, no router) on the births probe and Orchards, and records which ships. Per-profile `genome_size` anchor for the replication cost. Predeclared, from the probe (both layouts read in vitro and on Orchards seed 11): founder rows of the battery and births probe for the new profile beside V3Alpha1 (changed, silent, dead, sterile; probe: serial-gated 0.059 changed per birth, 0.8% dead per mutated birth, 2.7% sterile; routed 0.083 changed, 0.13% dead, 4.3% sterile, the extra sterility being `SwapRouteTargets` and `RemoveRouteTarget` on the router, the measured price of a decision that lives in a route); viability on the gate profile; Orchards seeds 11 and 12 to 2,000 ticks read for trough fertile share and rebuild (probe, seed 11: serial-gated 70% fertile at the first trough against the control's 10%, rebuild to 956, second famine fatal; routed 4% at the trough, rebuild to 2,123, no second collapse, 5,110 alive and 60% fertile at tick 2,000 while both others are under 10 creatures; one seed, so the direction is read, not predeclared; the breed-at-15%-condition rule stays T17's). Gate and goal trajectories unchanged: the default profile does not move here.
- [ ] **T18.F02 — Specialized Founder as the Default** — Depends on: T18.F01, T17.F02
  - Goal: Every world starts from the specialized founder. `FounderProfile::default()` becomes the T18.F01 profile; `FOUNDER_GENOME_SIZE_UNITS` is re-anchored to it; the seeding spec's Section 5.1 describes it as canonical; the founder rows of every indicator (battery, births, drift walk, steering, mesh execution) are re-baselined and V3Alpha1 stays selectable as the control. Placed after T17.F02 so the default founder is born with its energy gate on the unit scale (`Threshold(0.15)`) and is never re-expressed. Predeclared: the gate and goal trajectories move and both epochs are re-pinned in the closing commit (the roadmap's epoch rule); births per creature-tick on the gate profile read before and after; T17's carried hazard ("the founder must queue reproduce and forage in one tick, or keep its gate at or above acceptance") is discharged for the default founder and T17.F01's founder constraint is rewritten to say so.

## Notes for AI Agents

- Origin, 2026-09-18: created at the user's direction after the
  [founder refactor research note](../strategy/founder-refactor-research-2026-09-18.md),
  which read the engine's primitives, the prior art (Nolfi 1997 and
  Calabretta et al. 2000 in full), and a scratch-worktree probe of a
  five-node serial founder beside V3Alpha1. The user's decisions: a new
  profile first, V3Alpha1 kept as the control; the specialized profile
  becomes the default when done; the founder should carry a router that
  makes a real decision. The owned-route shape (the router's route is the
  reproduce decision, the reproduce gate a constant) is the note's design
  for that, unmeasured at creation; F01's spec reads it beside the
  serial-gated variant.
- A decision lives in exactly one place. A router whose choice a downstream
  action gate repeats contributes nothing and reads as a knockout node.
  Every router path ends in a motor node, and routers sit after the eat node
  where the layout allows, so a route broken by mutation still leaves a
  queued action (the executor returns the queue accumulated before a broken
  hop).
- What the track is not: it does not change the reproduce transfer, the
  energy scale, or the famine rule (T17 and the deferred collapse
  discussion); it does not add fan-out to the mesh executor (one successor
  per hop stays; a router that picks *the action* re-creates the
  reproduce-or-forage tick); it adds no sensor, operator, or assay. The
  topology weight rescale that made `ChangeEntryNode` 1 draw in 211 was made
  on 2026-09-18 ahead of this track (in the working tree, gate decision
  pending at creation) and is not part of it.
- Whether routing becomes load-bearing in the population is not this
  track's claim. The live survey found nothing in the current worlds pays
  for a second decision; the founder seeds the pattern, an ecology that
  needs memory (T02 seasons, T11.F10's temporal tasks) creates the demand.
  Read the survey's contributing-node and load-bearing-route counters at
  each closure, without a predeclared direction.
- Priority: T18.F01 is the next new start (placed 2026-09-18 ahead of
  T11.F20); T18.F02 is unplaced until T17.F02 is placed.

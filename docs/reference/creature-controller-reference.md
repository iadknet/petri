# Creature Controller Reference

This document intentionally separates:
- **Current implementation** (aligned to code today)
- **Planned cognition-first model** (not implemented yet)

## Current implementation (source-of-truth)

Primary files:
- `crates/petri-core/src/world/mod.rs`
- `crates/petri-core/src/world/tick.rs`
- `crates/petri-core/src/types.rs`
- `crates/petri-core/src/config.rs`
- `crates/petri-graph/src/types.rs`
- `crates/petri-graph/src/eval/evaluate.rs`
- `crates/petri-graph/src/eval/mod.rs`

### Runtime creature structure (current)

- Identity/state: `id`, `lineage_id`, `parent_id`, `age`, `generation`, `energy`, position.
- Controller: `ComputationGraph { palette, nodes, edges }`.
- Memory: byte `memory_register` and `last_memory_head`.
- Inventory: slot-capacity + slot contents.
- Inspector/debug caches: `last_inputs`, `last_outputs`, `last_move_blocked`, event logs, illegal attempts.
- RNG: per-creature `SmallRng`.

### Controller inputs and outputs (current)

Current `SensorInputs` include:
- world/sensing: food, nearest-food direction/distance, creature direction/distance, local density, barrier direction/distance
- state: energy, random, move-blocked-last-tick
- memory: `memory_read`, `memory_address_norm`
- touch/slot arrays for inventory context

Current `ActionOutputs` include:
- movement: `move_x`, `move_y`
- core actions: `eat`, `reproduce`
- memory ops: `memory_address_select`, `memory_write_value`, `memory_write_enable`
- inventory ops: pickup/put + slot/direction selectors

### Tick behavior (current)

Current per-creature tick behavior in `World::tick`:
1. apply tick and compute energy charges
2. evaluate controller stage A to choose memory address
3. read memory byte and evaluate stage B
4. apply memory write if enabled
5. resolve action attempts by output thresholds and validity gates

Important current semantic:
- More than one world interaction may occur in the same tick when multiple outputs are active.

### Current action interpretation highlights

- `move_x`/`move_y` map through axis-step thresholds to discrete movement deltas.
- `eat > 0.5` triggers eat attempt.
- `reproduce > 0.5` triggers reproduction attempt (subject to energy/capacity checks).
- inventory pickup/put intent is selector-based and direction-targeted.

## Planned cognition-first model (not implemented)

The project is rebaselining toward this model in upcoming refactor work.

### Planned controller additions

Planned new outputs:
- `halt`
- `no_op`

Planned new introspection inputs:
- previous-step confidence per action candidate
- running-max confidence per action candidate in current tick
- `energy_start_tick` (normalized energy at tick start, constant during tick)
- `energy_spent_tick` (normalized cumulative energy spent so far in current tick)
- `energy_remaining` (normalized current energy, refreshed each think step)

Action candidates in planned arbitration:
- `move`, `eat`, `reproduce`, `inventory_pickup`, `inventory_put`, `no_op`

Planned movement confidence:
- derived from `sqrt(move_x^2 + move_y^2)`

### Planned tick semantics

Per creature, per tick:
1. apply passive tick costs
2. run internal think loop (energy-bounded)
3. each think step receives refreshed `energy_spent_tick`/`energy_remaining` and pays `energy_per_think_step`
4. stop thinking on `halt` or energy exhaustion
5. select final action from final-thought confidences
6. execute at most one world interaction (or `no_op`)

Planned tie-break behavior:
- equal final confidences break randomly via per-creature seeded RNG (deterministic under fixed seeds/snapshots)

### Planned compatibility posture

- This is expected to be a semantic breaking change.
- Older snapshot/config assumptions are not guaranteed to remain load-compatible.

## Notes for Readers

- Treat this document as operational reference for both implementation and planning phases.
- If a section conflicts with code behavior, code wins for **Current implementation** sections.
- Planned sections are intent-only until merged implementation changes land.

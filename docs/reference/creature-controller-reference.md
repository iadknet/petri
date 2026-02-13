# Creature Controller Reference

This document describes the implemented controller and tick semantics.

Primary files:
- `crates/petri-core/src/world/mod.rs`
- `crates/petri-core/src/world/tick.rs`
- `crates/petri-core/src/world/snapshot.rs`
- `crates/petri-core/src/types.rs`
- `crates/petri-core/src/config.rs`
- `crates/petri-graph/src/types.rs`
- `crates/petri-graph/src/eval/evaluate.rs`
- `crates/petri-graph/src/eval/mod.rs`

## Runtime Creature Structure

Creature runtime state includes:
- identity/state: `id`, `lineage_id`, `parent_id`, `age`, `generation`, `energy`, position
- controller: `ComputationGraph { palette, nodes, edges }`
- byte memory: `memory_register` and `last_memory_head`
- inventory: slot capacity and slot contents
- inspector/debug caches: `last_inputs`, `last_outputs`, `last_move_blocked`, event logs, illegal attempts
- cognition diagnostics: `think_steps`, `halted`, `selected_action`, `selected_confidence`
- RNG: per-creature `SmallRng`

## Controller I/O Surface

### Sensor Inputs

`SensorInputs` includes:
- world/sensing: food, nearest-food direction/distance, creature direction/distance, local density, barrier direction/distance
- state: `energy`, `random`, `move_blocked_last_tick`
- memory: `memory_read`, `memory_address_norm`
- introspection:
  - `prev_action_confidence[6]`
  - `max_action_confidence[6]`
  - `energy_start_tick`
  - `energy_spent_tick`
  - `energy_remaining`
- touch/slot arrays for inventory context

Action-confidence slots are ordered:
1. `move` (derived from movement vector magnitude)
2. `eat`
3. `reproduce`
4. `inventory_pickup`
5. `inventory_put`
6. `no_op`

### Action Outputs

`ActionOutputs` includes:
- movement: `move_x`, `move_y`
- core actions: `eat`, `reproduce`
- inventory actions: `inventory_pickup`, `inventory_put`, selector outputs
- memory ops: `memory_address_select`, `memory_write_value`, `memory_write_enable`
- cognition controls: `halt`, `no_op`

Movement confidence is derived as `sqrt(move_x^2 + move_y^2)`.

## Tick Semantics (Cognition-First)

Per creature, per tick:
1. Apply passive costs (`energy_per_tick_decay`).
2. Enter think loop while energy remains.
3. Each think step executes:
   - stage A eval to select memory address
   - memory read
   - stage B eval for final outputs
   - optional memory write
   - introspection refresh (`prev_action_confidence`, `max_action_confidence`, energy-awareness fields)
   - cognition cost charge (`energy_per_think_step`)
4. Stop thinking when:
   - `halt > 0.5`, or
   - energy is exhausted, or
   - controller lacks `OutputHalt` (single-step fallback)
5. Run final-thought arbitration once.
6. Execute at most one world interaction path:
   - `move`, `eat`, `reproduce`, `inventory_pickup`, `inventory_put`, or `no_op`.

Tie break semantics:
- Exact final-confidence ties are resolved with per-creature seeded RNG to preserve deterministic replay from fixed seeds/snapshots.

## Config and Contract Notes

- Cognition cost control is `energy_per_think_step` (replacing `energy_per_compute_node`).
- Runtime config patch and startup draft contracts expose `energy_per_think_step`.
- Creature detail and snapshot payloads include cognition diagnostics.
- Snapshot compatibility is intentionally strict for cognition diagnostics: payloads missing `cognition` are rejected.

# Petri V3 Genome and Sensor Contract Reference

Reference specification for v3 genome schema, typed I/O contracts, and sensor
system. Adapted from v2 design. This is a reference document, not an
implementation plan.

> **V3 architecture context**: Module structure uses `creature/genome.rs` for
> genome ownership, `contracts/` for I/O types, and `sensors/` for input
> assembly. Energy is `u32`. Ticks are phase-based.

---

## 1. Genome Schema

### CreatureGenome

| field             | type                      | description                          |
| ----------------- | ------------------------- | ------------------------------------ |
| entry_node_id     | NodeId                    | First node evaluated each tick       |
| nodes             | Vec\<NodeGenome\>         | All decision/processing nodes        |
| evolution_params  | Option\<EvolutionParams\> | Optional per-creature mutation knobs (future; fields and inheritance TBD) |

### Validation rules

- `nodes` must be non-empty.
- All `node_id` values within `nodes` must be unique.
- `entry_node_id` must match an existing `node_id` in `nodes`.
- Every output target referenced by any node must resolve to a valid `node_id`
  or a world action.

### NodeGenome

| field              | type                        | description                              |
| ------------------ | --------------------------- | ---------------------------------------- |
| node_id            | NodeId                      | Unique identifier within the genome      |
| node_type          | NodeType (Graph \| Vm)      | Which backend evaluates this node        |
| backend_def        | BackendDef                  | Backend-specific definition              |
| output_definitions | Vec\<OutputDefinition\>     | Where this node sends its results        |
| local_state_init   | Option\<LocalStateInit\>    | Initial local state for the node         |

### BackendDef

```
enum BackendDef {
    Graph(GraphBackendDef),
    Vm(VmBackendDef),
}
```

### VmBackendDef

| field           | type                | constraints    | description                          |
| --------------- | ------------------- | -------------- | ------------------------------------ |
| register_count  | u8                  | 1..=64         | Number of VM registers               |
| program         | Vec\<VmInstruction\>| 0..=512        | Instruction sequence (empty = immediate halt) |
| constants       | Vec\<f32\>          | 0..=64         | Constant pool                        |
| max_input_slots | u8                  | 1..=64         | Maximum number of input slot indices |

---

## 2. Creature Memory Contract

- Each creature has **1 KiB (1024 bytes)** of persistent memory.
- Memory is **runtime state**, not part of the genome.
- The VM reads and writes memory via dedicated memory opcodes.
- Memory **persists across ticks** for the lifetime of the creature.
- On reproduction, memory is **copied byte-for-byte** from parent to offspring.

---

## 3. Typed Input References

The `InputReference` enum defines every way a node can request a value from the
world, from the creature's own state, or from another node.

```
enum InputReference {
    World(WorldInputKey),
    Introspection(IntrospectionInputKey),
    Packet(PacketFieldKey),
    SensorCell { dx: i32, dy: i32, field: SensorCellField },
    SensorCreature { dx: i32, dy: i32, field: SensorCreatureField },
    SensorSummary(SensorSummaryField),
    NeighborCell { direction: NeighborDirection, field: NeighborCellField },
    NeighborCreature { direction: NeighborDirection, field: NeighborCreatureField },
}
```

### WorldInputKey

| key                          | description                                        |
| ---------------------------- | -------------------------------------------------- |
| food_here                    | Food density at the creature's cell                |
| nearest_food_distance        | Distance to nearest food                           |
| nearest_food_direction       | Direction to nearest food                          |
| nearest_creature_distance    | Distance to nearest other creature                 |
| nearest_creature_direction   | Direction to nearest other creature                |
| occupied_here                | Whether the creature's cell is occupied by another |

### IntrospectionInputKey

| key                        | description                                  |
| -------------------------- | -------------------------------------------- |
| energy_current             | Creature's current energy                    |
| energy_spent_this_tick     | Energy spent so far in the current tick      |
| energy_remaining_this_tick | Energy budget remaining for the current tick |
| age_ticks                  | Creature's age in ticks                      |
| generation                 | Creature's generation (0 for seed creatures) |
| phenotype_r                | Red component of creature's own phenotype    |
| phenotype_g                | Green component of creature's own phenotype  |
| phenotype_b                | Blue component of creature's own phenotype   |
| inventory_slot_count       | Number of inventory slots (genome-determined) |
| inventory_slots_used       | Number of occupied inventory slots            |
| memory_bytes_total         | Total memory size (always 1024)              |

### PacketFieldKey

- Stable string key identifying a field within an inter-node packet.
- Regex: `^[a-z][a-z0-9_]{0,31}$`
- Maximum length: 32 characters.

---

## 4. Sensor Field Enums

### SensorCellField

| variant          | description                                    |
| ---------------- | ---------------------------------------------- |
| FoodDensityNorm  | Normalized food density at (dx, dy)            |
| BarrierFlag      | Whether the cell is a barrier                  |
| OccupiedFlag     | Whether the cell is occupied by any creature   |
| IsSelfFlag       | Whether the cell is the creature's own cell    |

### SensorCreatureField

| variant          | description                                    |
| ---------------- | ---------------------------------------------- |
| PresentFlag      | Whether a creature is present at (dx, dy)      |
| PhenotypeRNorm   | Normalized red component of phenotype          |
| PhenotypeGNorm   | Normalized green component of phenotype        |
| PhenotypeBNorm   | Normalized blue component of phenotype         |
| EnergyNorm       | Normalized energy of creature at (dx, dy)      |
| AgeNorm          | Normalized age of creature at (dx, dy)         |
| GenerationNorm   | Normalized generation of creature at (dx, dy)  |

### SensorSummaryField

| variant                   | description                                |
| ------------------------- | ------------------------------------------ |
| VisibleCreatureCountNorm  | Fraction of visible cells with creatures   |
| VisibleFoodMeanNorm       | Mean food density across visible cells     |
| VisibleFoodTotalNorm      | Total food density normalized by cell count|

### NeighborDirection

| variant    | offset (dx, dy) |
| ---------- | --------------- |
| North      | (0, -1)         |
| NorthEast  | (1, -1)         |
| East       | (1, 0)          |
| SouthEast  | (1, 1)          |
| South      | (0, 1)          |
| SouthWest  | (-1, 1)         |
| West       | (-1, 0)         |
| NorthWest  | (-1, -1)        |

### NeighborCellField

| variant          | description                              |
| ---------------- | ---------------------------------------- |
| FoodDensityNorm  | Normalized food density at neighbor cell |
| BarrierFlag      | Whether the neighbor cell is a barrier   |
| OccupiedFlag     | Whether the neighbor cell is occupied    |

### NeighborCreatureField

| variant          | description                                          |
| ---------------- | ---------------------------------------------------- |
| PresentFlag      | Whether a creature is present at the neighbor cell   |
| PhenotypeRNorm   | Normalized red component of neighbor's phenotype     |
| PhenotypeGNorm   | Normalized green component of neighbor's phenotype   |
| PhenotypeBNorm   | Normalized blue component of neighbor's phenotype    |
| EnergyNorm       | Normalized energy of creature at neighbor cell       |
| AgeNorm          | Normalized age of creature at neighbor cell          |
| GenerationNorm   | Normalized generation of creature at neighbor cell   |

---

## 5. SensorFrame Contract

The `SensorFrame` is built once per acting creature per tick, centered on the
creature's current position.

### Coverage

All cells within **Chebyshev radius** `sensor_radius`:

```
max(|dx|, |dy|) <= sensor_radius
```

`sensor_radius` is a global simulation config value. Minimum value: **1**.

### Per-cell data

| field            | type                | description                        |
| ---------------- | ------------------- | ---------------------------------- |
| food_density_u8  | u8                  | Raw food density (0..255)          |
| barrier_flag     | bool                | Whether the cell is a barrier      |
| occupied_flag    | bool                | Whether the cell is occupied       |
| creature_meta    | Option\<CreatureMeta\> | Metadata if a creature is present |

### Creature metadata (CreatureMeta)

| field      | type   | description                |
| ---------- | ------ | -------------------------- |
| phenotype  | (u8, u8, u8) | RGB phenotype color  |
| energy     | u32    | Creature's energy          |
| age_ticks  | u64    | Creature's age in ticks    |
| generation | u32    | Creature's generation      |

### Coordinate rules

- **dx > 0** is east; **dy > 0** is south.
- If `world_wrap = true`, coordinates wrap around world boundaries.
- If `world_wrap = false`, out-of-bounds cells return zero/default values.

---

## 6. Neighbor Input Contract

Neighbor inputs provide a convenient direction-based interface to the **Moore
neighborhood** at radius 1 (the 8 immediately adjacent cells).

### Direction-to-offset mapping

| direction  | (dx, dy)  |
| ---------- | --------- |
| North      | (0, -1)   |
| NorthEast  | (1, -1)   |
| East       | (1, 0)    |
| SouthEast  | (1, 1)    |
| South      | (0, 1)    |
| SouthWest  | (-1, 1)   |
| West       | (-1, 0)   |
| NorthWest  | (-1, -1)  |

### Behavior

- Neighbor inputs are **aliases** over the SensorFrame at the mapped offset.
- Neighbor inputs are only valid when `sensor_radius >= 1`.

---

## 7. Normalization Tables

### Built-in inputs

| input key                    | mapped value                                                        |
| ---------------------------- | ------------------------------------------------------------------- |
| food_here                    | food_density / 255.0                                                |
| nearest_food_distance        | clamp(distance / sensor_radius, 0.0, 1.0), default 1.0             |
| nearest_food_direction       | atan2(dy, dx) / PI in \[-1.0, 1.0\], default 0.0                   |
| nearest_creature_distance    | clamp(distance / sensor_radius, 0.0, 1.0), default 1.0             |
| nearest_creature_direction   | atan2(dy, dx) / PI in \[-1.0, 1.0\], default 0.0                   |
| occupied_here                | 1.0 if occupied else 0.0                                           |
| energy_current               | raw energy as f32                                                   |
| energy_spent_this_tick       | raw energy as f32                                                   |
| energy_remaining_this_tick   | raw energy as f32                                                   |
| age_ticks                    | raw ticks as f32                                                    |
| generation                   | raw generation as f32                                               |
| phenotype_r                  | r / 255.0                                                           |
| phenotype_g                  | g / 255.0                                                           |
| phenotype_b                  | b / 255.0                                                           |
| inventory_slot_count         | raw count as f32                                                    |
| inventory_slots_used         | raw count as f32                                                    |
| memory_bytes_total           | 1024.0                                                              |

### Sensor fields

| sensor field              | mapped value                              |
| ------------------------- | ----------------------------------------- |
| FoodDensityNorm           | food_density_u8 / 255.0                   |
| BarrierFlag               | 1.0 if barrier else 0.0                   |
| OccupiedFlag              | 1.0 if occupied else 0.0                  |
| IsSelfFlag                | 1.0 at (0, 0) else 0.0                    |
| PresentFlag               | 1.0 if creature else 0.0                  |
| PhenotypeRNorm            | r / 255.0                                 |
| PhenotypeGNorm            | g / 255.0                                 |
| PhenotypeBNorm            | b / 255.0                                 |
| EnergyNorm                | clamp(energy / 10.0, 0.0, 1.0)            |
| AgeNorm                   | clamp(age / 10000.0, 0.0, 1.0)            |
| GenerationNorm            | clamp(generation / 256.0, 0.0, 1.0)       |
| VisibleCreatureCountNorm  | visible_creatures / max_visible_cells      |
| VisibleFoodMeanNorm       | mean(food_density_norm)                    |
| VisibleFoodTotalNorm      | sum(food_density_norm) / max_visible_cells |

### Normalization defaults

| parameter                  | default value |
| -------------------------- | ------------- |
| sensor_energy_norm_scale   | 10.0          |
| sensor_age_norm_ticks      | 10000.0       |
| sensor_generation_norm     | 256.0         |

### V3 adaptation note

Energy values in input normalization require integer-to-f32 conversion since v3
uses `u32` energy internally. The normalization formulas remain the same -- the
conversion happens at the sensor assembly boundary.

---

## 8. Output Definitions

### OutputDefinition

```
enum OutputDefinition {
    InternalTarget {
        target_node_id: NodeId,
        input_refs: Vec<InputReference>,
        payload_fields: Vec<PacketFieldKey>,
    },
    WorldAction {
        action_kind: ActionKind,
        action_metadata_fields: Vec<ActionMetadataField>,
    },
}
```

### ActionMetadataField

```
enum ActionMetadataField {
    Direction(i32),
    Amount(u8),
    Slot(u8),
}
```

### World action metadata requirements

| action_kind    | required metadata          | optional metadata |
| -------------- | -------------------------- | ----------------- |
| Move           | Direction                  |                   |
| Eat            | (none)                     |                   |
| Reproduce      | Direction, Amount          |                   |
| PickupFood     | Direction, Slot            |                   |
| PickupBarrier  | Direction, Slot            |                   |
| PlaceFood      | Direction, Slot            |                   |
| PlaceBarrier   | Direction, Slot            |                   |
| NoOp           | (none)                     |                   |

### Inventory action semantics

- **PickupFood / PickupBarrier**: Remove the item from the target cell (by Direction)
  and store it in the creature's inventory at the given Slot index.
- **PlaceFood / PlaceBarrier**: Remove the item from the creature's inventory at the
  given Slot index and place it in the target cell (by Direction).
- **Failed actions** (full slot, empty slot, invalid target cell, occupied cell) still
  cost energy. The action is consumed (no fallback to NoOp) and the world state is
  unchanged.

### PacketValue types

```
enum PacketValue {
    Bool(bool),
    I32(i32),
    F32(f32),
    U8(u8),
}
```

### Output field uniqueness rules

Within a single node's `output_definitions`, all payload field keys and action
metadata field discriminants must be unique. Duplicate field keys within the same
output definition are a validation error.

---

## 9. ReadInput Slot Mapping

### Slot resolution

The `resolved_input_slots` array for a node is built by iterating `input_refs`
in declaration order. Each `InputReference` in the list corresponds to a
zero-indexed slot.

### ReadInput behavior

- `ReadInput(index)` reads the value at the given slot index.
- If `index` is out of range (>= number of resolved slots), the instruction
  returns **0.0**.

### Validation

- `max_input_slots` is enforced at schema validation time.
- The number of `input_refs` on any incoming edge must not exceed the target
  node's `max_input_slots`.

---

## 10. Founder Genomes

Founder genomes are named starting genome templates used to seed creatures when
initializing a new world. Defined in `creature/founders.rs` as a registry of
named genomes. Additional founders can be added over time.

### Registry

| name       | purpose                                                      |
| ---------- | ------------------------------------------------------------ |
| `test`     | Exercises all genome features. Used by viability tests and   |
|            | the non-collapse contract. Must be updated when new creature |
|            | features are added.                                          |
| `simple`   | Minimal viable creature: move toward food, eat, reproduce.   |
|            | Default founder for normal world initialization.             |

### Contract

- Every founder genome must pass `CreatureGenome::validate()`.
- The `test` founder must exercise: VM nodes, graph nodes, inter-node routing,
  world action outputs, memory read/write, inventory actions, and sensor inputs.
- The `simple` founder must sustain a viable population under default config
  (pass the non-collapse contract in the architecture design doc).
- Founder genomes are defined in code (`creature/founders.rs`), not in config
  files. Their exact structure is an implementation detail, but the behavioral
  contracts above must hold.

### World Initialization

`seed.rs` accepts a founder genome name (defaulting to `simple`) and passes it
to `creature/founders::get()` to retrieve the genome template. All seed
creatures in a given world receive a clone of the same genome.
`CreatureState::new_founder()` constructs each creature with the fixed
phenotype baseline (`[204, 61, 61]`), zeroed memory, and empty inventory. No
RNG is used during creature construction — seed creatures are deterministically
identical except for position.

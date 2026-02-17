# Petri V3 Creature Lifecycle Reference

Reference specification for v3 mutation, reproduction, and phenotype evolution.
This is a reference document, not an implementation plan.

> **V3 architecture context**: Mutation lives in `creature/mutation.rs`,
> reproduction in `creature/reproduction.rs`, phenotype in
> `creature/phenotype.rs`, genome validation in `creature/genome.rs`.

---

## 1. Mutation Config Defaults

### MutationConfig

| Parameter                                    | Default |
|----------------------------------------------|---------|
| `mutation_probability`                       | 0.01    |
| `per_birth_mutation_events_min`              | 0       |
| `per_birth_mutation_events_max`              | 8       |
| `max_nodes`                                  | 64      |
| `min_nodes`                                  | 1       |
| `max_outputs_per_node`                       | 8       |
| `max_subgraph_duplication_nodes`             | 6       |
| `max_vm_program_len`                         | 512     |
| `node_duplication_clone_outputs_probability` | 1.0     |
| `node_duplication_target_remap_probability`  | 0.35    |
| `subgraph_external_edge_retarget_probability`| 0.0     |
| `min_inventory_slots`                        | 1       |
| `max_inventory_slots`                        | 12      |
| `default_inventory_slots`                    | 1       |

### MutationWeights (must sum to 1.0)

| Operator                  | Weight |
|---------------------------|--------|
| `add_node`                | 0.10   |
| `remove_node`             | 0.06   |
| `retarget_node`           | 0.06   |
| `add_output`              | 0.10   |
| `remove_output`           | 0.08   |
| `retarget_output`         | 0.12   |
| `graph_local_mutation`    | 0.11   |
| `vm_instruction_mutation` | 0.19   |
| `node_duplication`        | 0.10   |
| `subgraph_duplication`    | 0.04   |
| `genome_param_mutation`   | 0.04   |

---

## 2. Mutation Pipeline

On reproduction, the offspring genome is copied from the parent. A random roll
against `mutation_probability` (default 1%) determines whether any mutation
occurs. If the roll fails, the offspring is an **exact genome copy** (no
mutation operators run, phenotype is also unchanged). If the roll succeeds,
`per_birth_mutation_events_min..=per_birth_mutation_events_max` mutation events
are applied sequentially, each chosen by weighted selection from the 11
operators below.

### Mutation Operators

All 11 operators must preserve structural invariants (Section 3) or defer to
the repair policy (Section 4).

1. **add_node** -- Creates a unique `node_id`, assigns a random type,
   initializes backend and state within configured bounds.

2. **remove_node** -- Never removes the final node below `min_nodes`. If the
   removed node is the entry node, selects a replacement entry. Rewires all
   targets that pointed to the removed node.

3. **retarget_node** -- Reassigns the entry pointer or a routing target to an
   existing node.

4. **add_output** -- Adds an output edge to a node, respecting
   `max_outputs_per_node`. Enforces metadata validity for the output's action
   kind.

5. **remove_output** -- Removes an output edge from a node. Preserves
   `max_outputs_per_node` constraint and enforces metadata validity.

6. **retarget_output** -- Changes the target of an existing output edge.
   Preserves `max_outputs_per_node` and enforces metadata validity.

7. **graph_local_mutation** -- Mutates graph-local parameters only (e.g.,
   thresholds, weights on a single node). No cross-node side effects.

8. **vm_instruction_mutation** -- Mutates opcode, operands, inserts, or deletes
   VM instructions. Respects `max_vm_program_len`.

9. **node_duplication** -- Clones a node with a new unique ID. Copies
   state and backend. Clones outputs with
   `node_duplication_clone_outputs_probability`. Remaps targets with
   `node_duplication_target_remap_probability`.

10. **subgraph_duplication** -- BFS from a seed node up to
    `max_subgraph_duplication_nodes`. Clones internal edges and remaps internal
    targets. External edges are retargeted with
    `subgraph_external_edge_retarget_probability`.

11. **genome_param_mutation** -- Mutates a genome-level parameter (not tied to
    any specific node). Selects one parameter uniformly at random and applies a
    small perturbation. Current genome-level parameters:
    - `inventory_slot_count`: increment or decrement by 1, clamped to
      `[min_inventory_slots, max_inventory_slots]`.

    This operator is the extension point for future genome-level evolvable
    parameters.

---

## 3. Structural Invariants

The following invariants **must** hold after each birth (post-mutation,
post-repair):

- `CreatureGenome::validate()` passes.
- All node IDs are unique.
- The entry node exists and is present in the node set.
- All internal targets resolve to existing nodes.
- All world action metadata is valid for its action kind.
- `nodes.len()` is within `[min_nodes, max_nodes]`.
- Output counts on every node are `<= max_outputs_per_node`.
- VM program lengths are within `[0, max_vm_program_len]`.
- Inventory slot count is within `[min_inventory_slots, max_inventory_slots]`.
- Offspring memory is copied byte-for-byte from parent.
- Offspring inventory is empty (items are not inherited).

---

## 4. Repair Policy

After applying the full mutation sequence for a birth:

1. Validate the offspring genome.
2. If invalid, attempt bounded repair (`<= 3` passes).
3. If still invalid after repair attempts, **discard the offspring** and keep the
   parent unchanged.

---

## 5. Reproduction Memory Inheritance

- Offspring memory is **1024 bytes**.
- Copied **byte-for-byte** from parent at reproduction commit.
- The snapshot is taken at commit time (post-cognition).
- No random perturbation is applied to the copied memory.

---

## 6. Phenotype Evolution

### Founder Baseline

| Field                | Default               |
|----------------------|-----------------------|
| `rgb`                | `[204, 61, 61]`      |
| `channel_weights`    | `[1.0, 1.0, 1.0]`    |
| `positive_increment` | `[true, true, true]`  |

All seed creatures start with this identical baseline. Phenotype then evolves
via the inheritance rules below (same algorithm as v1). Note: `seed.rs` must
use this fixed baseline rather than random colors.

### Inheritance Rules

- **No mutation event:** offspring matches parent exactly.
- **Mutation triggers:**
  1. Select channel by weighted selection.
  2. Optionally flip polarity (probability `0.002`).
  3. Mutate selected channel value by step `2`.
  4. Re-randomize the selected channel's weight in `[0.05, 1.0]`.
- Must be **deterministic** for a fixed seed.

---

## V3 Architecture Note

In v3's phase-based tick, offspring are deferred to a spawn queue during Phase 2
and inserted after all actions complete. This ensures `SlotMap` keys remain valid
during action execution. The mutation/repair cycle runs during
`create_offspring()` in `creature/reproduction.rs`.

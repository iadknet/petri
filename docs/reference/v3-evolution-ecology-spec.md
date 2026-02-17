# Petri V3 Evolution and Ecology Reference

Reference specification for v3 mutation, reproduction, ecology, and telemetry.
Adapted from v2 design. This is a reference document, not an implementation plan.

---

## 1. Mutation Config Defaults

### MutationConfig

| Parameter                                    | Default |
|----------------------------------------------|---------|
| `per_birth_mutation_events_min`              | 2       |
| `per_birth_mutation_events_max`              | 8       |
| `max_nodes`                                  | 64      |
| `min_nodes`                                  | 1       |
| `max_outputs_per_node`                       | 8       |
| `max_subgraph_duplication_nodes`             | 6       |
| `max_vm_program_len`                         | 128     |
| `node_duplication_clone_outputs_probability` | 1.0     |
| `node_duplication_target_remap_probability`  | 0.35    |
| `subgraph_external_edge_retarget_probability`| 0.0     |

### MutationWeights (must sum to 1.0)

| Operator                  | Weight |
|---------------------------|--------|
| `add_node`                | 0.10   |
| `remove_node`             | 0.06   |
| `retarget_node`           | 0.06   |
| `add_output`              | 0.10   |
| `remove_output`           | 0.08   |
| `retarget_output`         | 0.12   |
| `graph_local_mutation`    | 0.12   |
| `vm_instruction_mutation` | 0.20   |
| `node_duplication`        | 0.10   |
| `subgraph_duplication`    | 0.06   |

---

## 2. Mutation Operators

All 10 operators are listed below. Each operator must preserve structural
invariants (Section 3) or defer to the repair policy (Section 4).

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
- Offspring memory is copied byte-for-byte from parent.

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

## 7. Ecology Config Defaults

### EcologyConfig

| Parameter                      | Default |
|--------------------------------|---------|
| `resource_gradient_bands`      | 4       |
| `base_food_spawn_rate`         | 0.010   |
| `scarcity_multiplier_min`      | 0.25    |
| `scarcity_multiplier_max`      | 1.75    |
| `crowding_radius`              | 3       |
| `crowding_penalty_per_neighbor`| 0.005   |
| `season_length_ticks`          | 500     |
| `season_transition_ticks`      | 50      |
| `barrier_density`              | 0.06    |
| `health_window_ticks`          | 100     |

---

## 8. Ecology Mechanisms

### Resource Heterogeneity

The world is partitioned into bands (count = `resource_gradient_bands`) with
gradient multipliers controlling per-band food spawn rates.

### Scarcity Dynamics

Local food attenuates with overconsumption and recovers when underused.
Multipliers are clamped to `[scarcity_multiplier_min, scarcity_multiplier_max]`.

### Regime Shifts (Seasons)

Periodic band remapping occurs every `season_length_ticks`. During the
`season_transition_ticks` window, smoothing is applied to avoid abrupt resource
discontinuities.

### Crowding Pressure

- Distance metric: **Chebyshev distance** (L-infinity).
- Radius: `crowding_radius`.
- Excludes self.
- Formula:

  ```
  crowding_multiplier = clamp(1.0 - crowding_penalty_per_neighbor * neighbor_count, 0.0, 1.0)
  ```

- Applies to **food-energy gain** and **passive recovery** only (not action
  costs).
- The crowding multiplier is **monotonic non-increasing** with respect to
  neighbor count.

---

## 9. Telemetry Proxies

### RunHealthSnapshot

| Field                     | Description                                 |
|---------------------------|---------------------------------------------|
| `tick`                    | Current simulation tick                     |
| `population`              | Current live creature count                 |
| `births_last_window`      | Births in trailing window                   |
| `deaths_last_window`      | Deaths in trailing window                   |
| `mean_energy`             | Mean energy across live creatures            |
| `genome_node_count_p50`   | Median genome node count                    |
| `genome_node_count_p90`   | 90th percentile genome node count           |

The window is trailing `health_window_ticks`, right-aligned at the current tick.

---

## 10. Non-Collapse Contract

- Deterministic seed: `42`.
- Duration: `2000` ticks.
- Config: default.
- `baseline_population` = actual seeded population at tick 0.

### Pass Criteria

1. No crash.
2. At least one birth.
3. Mean population in the final 400 ticks >= `ceil(baseline_population * 0.10)`.

---

## 11. Viability Gate

Reusable helper for validating simulation viability.

### Parameters

| Parameter                  | Default | Description                            |
|----------------------------|---------|----------------------------------------|
| `probe_ticks`              | 100     | Number of ticks to probe               |
| `min_final_window_ratio`   | 0.10    | Minimum ratio of baseline population   |
| `require_births`           | toggle  | Whether births are required to pass    |

### Output

| Field                          | Description                                |
|--------------------------------|--------------------------------------------|
| `viable`                       | Boolean pass/fail                          |
| `baseline_population`          | Population at tick 0                       |
| `births_total`                 | Total births during probe                  |
| `final_window_mean_population` | Mean population in final window            |
| `threshold_population`         | Minimum population required to pass        |

---

## V3 Architecture Note

In v3's phase-based tick, offspring are deferred to a spawn queue during Phase 2
and inserted after all actions complete. This ensures `SlotMap` keys remain valid
during action execution. The mutation/repair cycle runs during
`create_offspring()` in `creature/reproduction.rs`.

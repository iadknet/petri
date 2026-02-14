# Petri V2 CP-2 Evolution and Ecology Spec

**Goal:** Define implementation-complete mutation, reproduction, and ecology pressure semantics for checkpoint `CP-2` so emergent behavior can scale without novelty scoring.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2/crates/v2-core` evolution and ecology configs, operators, invariants, and integration test expectations; excludes `v2-server/cli/web` contracts.
**Docs Impact:** Adds CP-2 implementation spec consumed by Stage 3 plan.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: expands behavioral search via structural and program-level mutation.
- `GP-03`: defines strict validity invariants to keep random mutation safe.
- `GP-04`: introduces lightweight run-health telemetry for collapse detection.

## Boundary Impact

- Implementation remains fully inside `v2/crates/v2-core`.
- No mutation/ecology code shared with legacy runtime.
- Transport layers consume outputs later without influencing operator semantics.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/evolution/*` | change | Owns operator sampling, mutation application, and repair validation. |
| `v2/crates/v2-core/src/ecology/*` | change | Owns resource regimes, scarcity gradients, and crowding pressure. |
| `v2/crates/v2-core/src/mesh.rs` | keep | Structural validity contracts should build on existing mesh schema. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should CP-2 include sexual recombination? | No, asexual only. | user+agent | resolved |
| Should novelty archives be introduced in CP-2? | No, ecology pressure is the novelty driver. | user+agent | resolved |
| Should invalid mutations be discarded or repaired? | Repair first; discard only if repair fails invariants. | user+agent | resolved |
| Should creature runtime memory be heritable at birth? | Yes in v1; offspring memory is copied byte-for-byte from parent. | user+agent | resolved |
| What explicitly counts as a CP-2 non-collapse pass? | Fixed-seed baseline run must keep final-window mean population at or above `10%` of initial and produce at least one birth event. | user+agent | resolved |
| Should duplication/remap behavior be probabilistic but explicitly defaulted? | Yes; defaults are explicit in `MutationConfig` and must be test-covered. | user+agent | resolved |

## Evolution Contract

### Mutation config defaults (normative v1 defaults)

`MutationConfig`:
- `per_birth_mutation_events_min: 2`
- `per_birth_mutation_events_max: 8`
- `max_nodes: 64`
- `min_nodes: 1`
- `max_outputs_per_node: 8`
- `max_subgraph_duplication_nodes: 6`
- `max_vm_program_len: 128`
- `node_duplication_clone_outputs_probability: 1.0`
- `node_duplication_target_remap_probability: 0.35`
- `subgraph_external_edge_retarget_probability: 0.0`

`MutationWeights` (must sum to `1.0`):
- `add_node: 0.10`
- `remove_node: 0.06`
- `retarget_node: 0.06`
- `add_output: 0.10`
- `remove_output: 0.08`
- `retarget_output: 0.12`
- `graph_local_mutation: 0.12`
- `vm_instruction_mutation: 0.20`
- `node_duplication: 0.10`
- `subgraph_duplication: 0.06`

### Required mutation operators

1. `add_node`
- creates unique `node_id`
- random `node_type` (`graph` or `vm`)
- initializes backend/local state within bounds

2. `remove_node`
- never removes final node below `min_nodes`
- if removing entry node, selects replacement existing node as new entry
- rewires or removes targets referencing deleted node

3. `retarget_node`
- reassigns entry or selected routing target to existing node ID

4. `add_output`, `remove_output`, `retarget_output`
- preserve `max_outputs_per_node`
- enforce output metadata validity

5. `graph_local_mutation`
- mutates graph-local parameters/state only, no cross-node side effects

6. `vm_instruction_mutation`
- mutate opcode, operands, insertion, deletion under `max_vm_program_len`
- maintain decodable instruction stream

7. `node_duplication`
- clone source node (with new node ID)
- copy local state/backend def
- clone outputs with probability `node_duplication_clone_outputs_probability` (default `1.0`)
- for each cloned internal-target output field, apply target remap to duplicated IDs with probability `node_duplication_target_remap_probability` (default `0.35`)

8. `subgraph_duplication`
- BFS from seed node with upper bound `max_subgraph_duplication_nodes`
- clone internal edges among duplicated nodes
- remap internal targets to cloned IDs
- external edges are retargeted with probability `subgraph_external_edge_retarget_probability` (default `0.0`, preserve external edges)

### Reproduction memory inheritance contract

1. Offspring memory arena size is fixed `1024` bytes.
2. At successful reproduction commit, offspring memory is copied byte-for-byte from parent memory.
3. Inheritance snapshot is taken at reproduction commit time (post parent cognition for the tick).
4. No memory randomization or reset is applied in v1 reproduction path.

### Structural invariants (must hold after each birth)

1. `CreatureGenome::validate()` passes.
2. Node IDs are unique.
3. Entry node exists.
4. Every internal target resolves to existing node.
5. Every world action metadata payload is valid for action kind.
6. `nodes.len()` within `[min_nodes, max_nodes]`.
7. Node output counts are `<= max_outputs_per_node`.
8. VM program lengths are within bounds.
9. Offspring runtime memory arena is copied byte-for-byte from parent at reproduction commit.

Repair policy:
1. Apply mutation sequence.
2. Attempt bounded repair (`<= 3` repair passes).
3. If still invalid, discard offspring and keep parent unchanged.

## Ecology Contract

### Ecology config defaults

`EcologyConfig`:
- `resource_gradient_bands: 4`
- `base_food_spawn_rate: 0.010`
- `scarcity_multiplier_min: 0.25`
- `scarcity_multiplier_max: 1.75`
- `crowding_radius: 3`
- `crowding_penalty_per_neighbor: 0.005`
- `season_length_ticks: 500`
- `season_transition_ticks: 50`
- `barrier_density: 0.06`
- `health_window_ticks: 100`

### Required ecology mechanisms

1. Resource heterogeneity
- world partitioned into bands with deterministic gradient multipliers
- spawn/growth rates vary per band each tick

2. Scarcity dynamics
- local food generation attenuates with regional overconsumption
- scarcity recovers gradually if region is underused

3. Regime shifts (seasons)
- periodic band multiplier remapping every `season_length_ticks`
- transition smoothing over `season_transition_ticks`

4. Crowding pressure
- local creature density adds energy pressure / reduced gain
- pressure is monotonic with neighbor count in `crowding_radius`

### Crowding pressure formula (normative)

1. Neighbor count uses Chebyshev distance (`max(|dx|, |dy|) <= crowding_radius`) and excludes self.
2. `crowding_multiplier = clamp(1.0 - crowding_penalty_per_neighbor * neighbor_count, 0.0, 1.0)`.
3. `crowding_multiplier` applies to food-energy gain and passive recovery terms only (not direct movement/action costs).
4. Higher neighbor count must never increase effective gain (monotonic non-increasing).

### Telemetry proxies (minimal)

`RunHealthSnapshot` fields:
- `tick`
- `population`
- `births_last_window`
- `deaths_last_window`
- `mean_energy`
- `genome_node_count_p50`
- `genome_node_count_p90`

No lineage explanation is required in v1.

Telemetry window contract:
1. `births_last_window` and `deaths_last_window` are computed over trailing `health_window_ticks`.
2. Window is tick-based and right-aligned at current `tick`.
3. When `tick < health_window_ticks`, window starts at `tick=0`.

### Baseline non-collapse contract (`ecology_noncollapse`)

1. Test run is deterministic with fixed seed `42`.
2. Run length is `2000` ticks with CP-2 default config.
3. `baseline_population` is the actual seeded population at `tick=0` after occupancy constraints (not requested `initial_creatures`).
4. Pass criteria:
- no crash/panic
- at least one successful birth occurs during run
- mean population across final `400` ticks is `>= ceil(baseline_population * 0.10)`

## Task List

### Task 1: Add failing mutation invariant and repair tests

Files:
- Create: `v2/crates/v2-core/tests/mutation_invariants.rs`
- Create: `v2/crates/v2-core/tests/mutation_repair.rs`
- Create: `v2/crates/v2-core/tests/reproduction_memory_inheritance.rs`

Steps:
1. Add failing tests for each required operator.
2. Add failing tests for invariants and repair policy behavior.
3. Add deterministic seed fixtures for reproducible failures.
4. Add failing tests proving offspring memory equals parent memory at reproduction commit.
5. Add failing tests for duplication/remap default probabilities and external-edge-preservation default.

### Task 2: Implement mutation engine with defaults

Files:
- Create: `v2/crates/v2-core/src/evolution/mod.rs`
- Create: `v2/crates/v2-core/src/evolution/config.rs`
- Create: `v2/crates/v2-core/src/evolution/operators.rs`
- Create: `v2/crates/v2-core/src/evolution/validation.rs`

Steps:
1. Implement config/weights and operator sampling.
2. Implement mutation operators and bounded repair loop.
3. Integrate with asexual reproduction path.

### Task 3: Add failing ecology pressure tests

Files:
- Create: `v2/crates/v2-core/tests/ecology_pressures.rs`
- Create: `v2/crates/v2-core/tests/ecology_noncollapse.rs`

Steps:
1. Add failing tests for gradients, seasons, and crowding monotonicity.
2. Add baseline non-collapse smoke test with bounded runtime and explicit pass thresholds using `baseline_population` definition from this spec.

### Task 4: Implement ecology systems and minimal telemetry

Files:
- Create: `v2/crates/v2-core/src/ecology/mod.rs`
- Create: `v2/crates/v2-core/src/ecology/config.rs`
- Create: `v2/crates/v2-core/src/ecology/regimes.rs`
- Create: `v2/crates/v2-core/src/telemetry.rs`

Steps:
1. Implement ecology dynamics from this spec.
2. Emit `RunHealthSnapshot` at configurable windows.
3. Keep telemetry lightweight and non-blocking.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test mutation_invariants`
3. `cd v2 && cargo test -p v2-core --test mutation_repair`
4. `cd v2 && cargo test -p v2-core --test reproduction_memory_inheritance`
5. `cd v2 && cargo test -p v2-core --test ecology_pressures`
6. `cd v2 && cargo test -p v2-core --test ecology_noncollapse`
7. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: aggressive operator weights can produce unstable baseline populations.
- Risk: ecology defaults can mask behavior diversity if gradients are too weak.
- Rollback:
1. Revert CP-2 implementation commits.
2. Re-run with tuned defaults while preserving invariant test suite.

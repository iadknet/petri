# Creature Controller Reference (Stage 1)

This document is the implementation-aligned reference for creature/controller structure and mutation behavior in Stage 1.

Source of truth files:
- `crates/petri-core/src/world.rs`
- `crates/petri-core/src/types.rs`
- `crates/petri-core/src/config.rs`
- `crates/petri-graph/src/types.rs`
- `crates/petri-graph/src/eval.rs`

## Creature and Controller Structure

| Element | Kind | Defined in | Purpose | Key fields |
| --- | --- | --- | --- | --- |
| `Creature` | Runtime internal struct | `crates/petri-core/src/world.rs` | Per-creature mutable simulation state stored in `World` | `x`, `y`, `energy`, `age`, `generation`, `lineage_id`, `parent_id`, `controller`, `rng`, `events` |
| `CreatureView` | Lightweight view struct | `crates/petri-core/src/world.rs` | Public read-only projection used by API helpers | `id`, `x`, `y`, `energy`, `age`, `generation` |
| `CreatureSnapshot` | Frame payload struct | `crates/petri-core/src/types.rs` | Per-tick transport snapshot for clients | `id`, lineage/parent, position, `energy`, `age`, `generation`, `node_count` |
| `CreatureStateSnapshot` | Full snapshot struct | `crates/petri-core/src/types.rs` | Save/load representation of full creature state | Includes full `controller: ComputationGraph` |
| `SensorInputs` | Controller input struct | `crates/petri-graph/src/types.rs` | Normalized sensory values fed into graph evaluation | `food_here`, `energy`, `random`, `food_direction`, `food_distance` |
| `ActionOutputs` | Controller output struct | `crates/petri-graph/src/types.rs` | Intent values emitted by graph before world-level action gates | `move_x`, `move_y`, `eat`, `reproduce` |
| `ComputationGraph` | Controller struct | `crates/petri-graph/src/eval.rs` | Evolvable DAG-like controller graph | `palette`, `nodes: Vec<NodeKind>`, `edges: Vec<Edge>` |
| `Edge` | Graph edge struct | `crates/petri-graph/src/types.rs` | Weighted directional connection between node indices | `from`, `to`, `weight` |
| `ControllerPalette` | Enum | `crates/petri-graph/src/types.rs` | Chooses starter topology (`NeuralOnly`, `LogicOnly`, `Hybrid`) | Used by `from_palette` and `founder` |
| `MutationConfig` | Struct | `crates/petri-graph/src/eval.rs` | Parameter bundle controlling mutation operators | `weight_mutation_rate`, `weight_mutation_magnitude`, `logic_node_mutation_rate`, `structural_mutation_rate` |

## Node Types and Functions (`NodeKind`)

| Node kind | Category | Evaluation behavior | Output range / notes |
| --- | --- | --- | --- |
| `InputFoodHere` | Input | Uses `inputs.food_here.clamp(0.0, 1.0)` | `0.0..=1.0` |
| `InputEnergy` | Input | Uses `inputs.energy.clamp(0.0, 1.0)` | `0.0..=1.0` |
| `InputRandom` | Input | Uses `inputs.random.clamp(-1.0, 1.0)` | `-1.0..=1.0` |
| `InputFoodDirection` | Input | Uses `inputs.food_direction.clamp(-1.0, 1.0)` | `-1.0..=1.0` |
| `InputFoodDistance` | Input | Uses `inputs.food_distance.clamp(0.0, 1.0)` | `0.0..=1.0` |
| `Constant(f32)` | Hidden/function | Returns stored constant | Value set by graph/mutation |
| `Add` | Hidden/function | Sum of weighted inputs | `sum(weighted_inputs)` |
| `Multiply` | Hidden/function | Product of weighted inputs | Returns `0.0` if no inputs |
| `Threshold(f32)` | Hidden/function | `1.0` if sum of weighted inputs `>= threshold`, else `0.0` | Logic-like gate |
| `GreaterThan` | Hidden/function | Compares first two weighted inputs (`a > b`) | Returns `1.0` or `0.0` |
| `Sigmoid` | Hidden/function | Logistic activation over sum of weighted inputs | `0.0..=1.0` |
| `Tanh` | Hidden/function | Hyperbolic tangent over sum of weighted inputs | `-1.0..=1.0` |
| `Relu` | Hidden/function | `max(sum(weighted_inputs), 0.0)` | `>= 0.0` |
| `Select` | Hidden/function | Control = first input; returns third input if control `> 0`, else second input | Ternary-style switch |
| `OutputMoveX` | Output | Sum of weighted inputs | Clamped to `-1.0..=1.0` in `ActionOutputs` |
| `OutputMoveY` | Output | Sum of weighted inputs | Clamped to `-1.0..=1.0` in `ActionOutputs` |
| `OutputEat` | Output | Sum of weighted inputs | Clamped to `0.0..=1.0`; world attempts eat when `> 0.5` |
| `OutputReproduce` | Output | Sum of weighted inputs | Clamped to `0.0..=1.0`; world attempts reproduction when `> 0.5` and energy/cap checks pass |

## World-Level Interpretation of Controller Outputs

| Output | World function | Behavior |
| --- | --- | --- |
| `move_x`, `move_y` | `axis_step` in `crates/petri-core/src/world.rs` | Each axis maps to `-1`, `0`, `1` via thresholds (`> 0.25`, `< -0.25`); movement also requires destination vacancy |
| `eat` | Tick action logic in `World::tick` | If `> 0.5`, creature consumes all available food in its current cell and gains energy scaled by `food_energy_value` |
| `reproduce` | Tick action logic in `World::tick` | If `> 0.5`, reproduction still requires empty neighbor, `min_reproduce_energy`, and `max_creatures` capacity |

## Mutation Functions

| Function | Location | What it changes | Constraints / guardrails | Return |
| --- | --- | --- | --- | --- |
| `mutate_weights(rng, rate, magnitude)` | `ComputationGraph` (`crates/petri-graph/src/eval.rs`) | Edge weights, `Constant` node values, `Threshold` node values | `rate` clamped `0..1`; `magnitude` absolute; edge/constant clamp `-8..8`; threshold clamp `0..1` | `bool` changed |
| `mutate_with_config(rng, cfg)` | `ComputationGraph` | Orchestrates all configured mutation operators | Applies weight mutation first; each structural operator sampled independently using `cfg.structural_mutation_rate`; hidden type mutation sampled by `cfg.logic_node_mutation_rate` | `bool` changed |
| `add_hidden_node_by_splicing_edge(rng)` | `ComputationGraph` | Removes one edge and inserts one new hidden node plus two replacement edges | Only edges with `from < to` are candidates; updates node indices to keep graph consistent | `bool` changed |
| `add_edge_mutation(rng)` | `ComputationGraph` | Adds one new edge | Never from output nodes; never to input nodes; only forward edges (`from < to`); skips duplicates | `bool` changed |
| `remove_edge_mutation(rng)` | `ComputationGraph` | Removes one random edge | No-op if graph has zero edges | `bool` changed |
| `remove_disconnected_hidden_nodes()` | `ComputationGraph` | Removes hidden nodes with zero incoming and zero outgoing edges | Input/output nodes are never removed by this operator | `usize` removed count |
| `change_hidden_node_type(rng)` | `ComputationGraph` | Replaces one hidden node with another random hidden node kind | No-op if no hidden nodes; retries up to 8 draws to avoid same type | `bool` changed |
| `initial_mutation_config()` | `World` (`crates/petri-core/src/world.rs`) | Produces startup mutation config for founder diversification | Scales weight rate by `INITIAL_WEIGHT_MUTATION_SCALE` (`0.7`); scales structural+logic rates by `INITIAL_STRUCTURAL_MUTATION_SCALE` (`0.35`) | `MutationConfig` |
| `offspring_mutation_config()` | `World` | Produces per-birth mutation config | Uses world config directly, with clamping/non-negative guards | `MutationConfig` |

### Mutation Config Fields

| Field | Default (`WorldConfig`) | Meaning |
| --- | --- | --- |
| `weight_mutation_rate` | `0.26` | Per-edge/per-parameter probability for value perturbation |
| `weight_mutation_magnitude` | `0.18` | Max absolute perturbation magnitude for values |
| `logic_node_mutation_rate` | `0.04` | Chance to run hidden node type-change operator |
| `structural_mutation_rate` | `0.08` | Independent chance for each structural operator (add node, add edge, remove edge, prune disconnected) |

## Other Similar Docs Worth Adding

- Founder palette topology snapshots (`from_palette` and `founder`) with node/edge counts and intent.
- Sensor normalization and world sensing rules (especially nearest-food direction/distance semantics).
- Reproduction lifecycle and lineage bookkeeping reference (energy transfer, parent linkage, event log behavior).
- A mutation tuning guide with expected behavioral effects of each config knob.

# Petri — Architecture Design Document

## Overview

Petri is an artificial life simulator inspired by classic PowerPC-era "Creatures" programs. It simulates a 2D world populated by digital organisms whose behavior is governed by evolvable hybrid computation graphs — a blend of neural network nodes and discrete logic primitives. Creatures consume food, compete, cooperate, reproduce, and die under configurable rules. The architecture focuses on correctness, observability, and experimentation; interesting emergent patterns are possible but not a delivery requirement.

The project is built on a Rust simulation backend with a web-based frontend, connected via WebSocket and REST APIs.

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────┐
│                 Web Frontend                     │
│  (React + TypeScript + Vite)                     │
│                                                  │
│  - World visualization (creatures, food, walls)  │
│  - Creature inspector (graph view, stats)        │
│  - Evolutionary tree browser                     │
│  - Configuration panel                           │
│  - World painting tools (barriers, food)         │
└────────────┬──────────────┬──────────────────────┘
             │ WebSocket    │ REST API
             │ (world       │ (config, commands,
             │  stream)     │  queries, painting)
┌────────────▼──────────────▼──────────────────────┐
│                 Rust Server                       │
│  (API layer, session management)                 │
├──────────────────────────────────────────────────┤
│               Simulation Engine                  │
│                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │  World   │ │ Creature │ │    Genome /       │ │
│  │  Grid    │ │ Evaluator│ │    Lineage        │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
└──────────────────────────────────────────────────┘
```

### Deployment Model

**Rust server + web frontend (only supported mode):** The Rust binary runs the simulation and serves REST/WebSocket APIs. The web frontend connects over localhost or network and can be hosted separately or served alongside the backend.

This supports both direct host execution and containerized deployment (for example, Docker). Save/load files should be written to a mounted persistent volume in containerized environments.

---

## Crate Structure

```
petri/
├── crates/
│   ├── petri-core/          # World, creatures, grid, energy, tick loop
│   ├── petri-graph/         # Computation graph: node types, evaluation, I/O
│   ├── petri-genome/        # Mutation, crossover, lineage tracking, kin tags
│   ├── petri-server/        # WebSocket + REST API, session management
│   └── petri-cli/           # Headless runner, batch experiments, analysis tools
├── web/                     # Frontend (React + TypeScript + Vite)
│   ├── src/
│   ├── index.html
│   └── package.json
├── Cargo.toml               # Workspace root
└── README.md
```

### Crate Responsibilities

**Core/runtime boundary constraint:** `petri-core`, `petri-graph`, and `petri-genome` should have zero dependencies on `tokio`, `axum`, or other server/async runtime components. They remain pure computation libraries reusable by both `petri-server` and `petri-cli`, while networking and API concerns stay in `petri-server`.

**petri-core:** Owns the world grid, creature storage (via `slotmap::HopSlotMap`), food/barrier placement, the main simulation tick loop, and action resolution. This is the integration point — it pulls in `petri-graph` for creature evaluation and `petri-genome` for reproduction. Uses `rustc-hash::FxHashMap` for spatial hashing of creature positions.

**petri-graph:** Defines the computation graph structure using `petgraph::Graph` as the underlying storage. All node types (neural and discrete), graph evaluation logic, and the input/output interface. This crate knows nothing about the world — it takes a vector of sensor inputs and produces a vector of action outputs.

**petri-genome:** Handles everything related to heredity. Mutation operators (add/remove node, add/remove edge, change node type, perturb parameters, duplicate subgraph), crossover for sexual reproduction, lineage tree maintenance, and kin tag mechanics. Implements NEAT-style innovation numbering for crossover alignment — a global counter assigns unique IDs to structural mutations, allowing genomes with different topologies to be meaningfully combined during sexual reproduction.

**petri-server:** Wraps the simulation in network APIs using `axum` and `tokio`. Adds `rayon`-based parallelism for creature evaluation (Phase 2) and sensing (Phase 1). Manages simulation lifecycle (create, pause, resume, save, load). Serializes world state to MessagePack (`rmp-serde`) for WebSocket streaming and JSON for REST responses. Streams frames directly to the active local web client.

**petri-cli:** Command-line tool for running headless simulations, dumping statistics, exporting evolutionary trees, and batch parameter sweeps. Useful for overnight runs and automated experiments.

---

## Core Data Structures

### World

```rust
struct World {
    width: u32,
    height: u32,
    grid: Grid,                          // spatial data (food, barriers)
    creatures: HopSlotMap<CreatureId, Creature>,
    lineage: LineageTree,
    config: WorldConfig,
    tick: u64,
    rng: SmallRng,                       // Xoshiro-based, fast and seedable
}

struct Grid {
    cells: Vec<Cell>,                    // width * height, row-major
}

struct Cell {
    food: f32,                           // 0.0 = none, 1.0 = full
    barrier: bool,
    creature: Option<CreatureId>,        // at most one creature per cell
}

struct WorldConfig {
    // Food
    food_spawn_rate: f32,                // probability per tick of new food source
    food_growth_rate: f32,               // rate food spreads to adjacent cells
    food_max_density: f32,               // cap per cell
    food_energy_value: f32,              // energy gained per unit consumed

    // Energy
    energy_per_tick_decay: f32,          // passive energy loss per tick
    energy_per_move: f32,               // cost to move one cell
    energy_per_compute_node: f32,        // cost per graph node evaluated per tick
    energy_per_signal: f32,              // cost to emit a signal
    energy_per_reproduce: f32,           // base cost of reproduction
    energy_initial: f32,                 // energy given to newborns
    energy_max: f32,                     // cap (excess is wasted)

    // Reproduction
    weight_mutation_rate: f32,           // probability of perturbing continuous parameters
    weight_mutation_magnitude: f32,      // scale of weight/parameter perturbations
    logic_node_mutation_rate: f32,       // probability of discrete logic-node type changes
    structural_mutation_rate: f32,       // probability of add/remove node/edge
    crossover_rate: f32,                 // for sexual reproduction
    min_reproduce_energy: f32,           // threshold to attempt reproduction
    offspring_energy_fraction: f32,      // fraction of parent energy given to child

    // Predation
    predation_enabled: bool,
    predation_energy_transfer: f32,      // fraction of prey energy gained
    predation_size_advantage: f32,       // how much size/energy matters
    energy_per_attack: f32,              // energy cost of attempting predation
    predation_min_population: u32,       // disable predation below this population (0 = no floor)

    // Social
    signaling_enabled: bool,
    signal_channels: u8,                 // number of distinct signal channels
    signal_range: u8,                    // how far signals propagate (cells)
    kin_tag_length: u8,                  // length of kin recognition tag
    energy_sharing_enabled: bool,
    energy_share_transfer_rate: f32,     // fraction of energy actually transferred
    energy_share_cost: f32,              // energy lost in transit (inefficiency)
    memory_slots: u8,                    // number of memory registers per creature

    // World
    world_wrap: bool,                    // toroidal world vs hard boundaries
    max_creatures: u32,                  // population cap
    ticks_per_second: u32,               // target simulation speed
}
```

### Creature

```rust
struct Creature {
    id: CreatureId,
    position: (u32, u32),
    energy: f32,
    age: u64,                            // ticks alive
    graph: ComputationGraph,
    kin_tag: Vec<u8>,                    // heritable identity marker
    lineage_id: LineageId,
    parent_id: Option<CreatureId>,
    generation: u32,
    phenotype: Phenotype,
    memory: Vec<f32>,                    // persistent registers (length = config.memory_slots)
    signals_received: Vec<Signal>,       // incoming signals this tick
    rng: SmallRng,                       // per-creature RNG, seeded from parent at birth
    event_log: RingBuffer<CreatureEvent, 8>, // recent events for debugging/inspection
}

struct Phenotype {
    color: (u8, u8, u8),                // derived from genome, used for display
    size: f32,                           // could affect predation, energy costs
}
```

### Computation Graph

```rust
struct ComputationGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<Edge>,
    input_node_ids: Vec<NodeId>,         // fixed: one per sensor input
    output_node_ids: Vec<NodeId>,        // fixed: one per action output
    activation_buffer: Vec<f32>,         // current tick values
    prev_activation_buffer: Vec<f32>,    // previous tick (for recurrence)
    // Cached topology (recomputed when graph structure is mutated)
    topo_order: Vec<NodeId>,             // topological sort ignoring back edges
    back_edges: Vec<EdgeId>,             // edges that create cycles (read from prev buffer)
}

struct GraphNode {
    id: NodeId,
    node_type: NodeType,
    parameters: Vec<f32>,                // weights, thresholds, etc.
}

struct Edge {
    from: NodeId,
    to: NodeId,
    weight: f32,
}

enum NodeType {
    // --- Input nodes (fixed, not evolvable) ---
    SensorFoodDirection,                 // angle to nearest food
    SensorFoodDistance,                   // distance to nearest food
    SensorFoodHere,                      // food level at current cell
    SensorCreatureDirection,             // angle to nearest creature
    SensorCreatureDistance,               // distance to nearest creature
    SensorCreatureEnergy,                // energy of nearest creature (relative to self)
    SensorCreatureKinSimilarity,         // kin tag match with nearest creature
    SensorEnergy,                        // own energy level (normalized)
    SensorAge,                           // own age (normalized)
    SensorSignal(u8),                    // incoming signal on channel N
    SensorPopulationDensity,             // creatures in local area
    SensorRandom,                        // random value each tick (for stochasticity)
    SensorMemory(u8),                    // read from memory slot N
    SensorBarrierDirection,              // angle to nearest barrier
    SensorBarrierDistance,               // distance to nearest barrier

    // --- Hidden / processing nodes (evolvable) ---
    NeuralSigmoid,                       // weighted sum → sigmoid activation
    NeuralRelu,                          // weighted sum → ReLU activation
    NeuralTanh,                          // weighted sum → tanh activation
    Threshold,                           // neural->logic converter: float to 0.0/1.0 by threshold
    Select,                              // logic->neural converter: route one of two float values
    Add,                                 // sum of inputs (no weights)
    Multiply,                            // product of inputs
    Negate,                              // -input
    Abs,                                 // |input|
    GreaterThan,                         // 1.0 if input_a > input_b, else 0.0
    Min,                                 // min of inputs
    Max,                                 // max of inputs
    Constant(f32),                       // outputs a fixed value
    Gate,                                // if control > 0, pass signal; else 0
    Accumulator,                         // running sum across ticks (recurrent)
    Delay,                               // output = previous tick's input

    // --- Output nodes (fixed, not evolvable) ---
    OutputMoveX,                         // movement intent X component
    OutputMoveY,                         // movement intent Y component
    OutputEat,                           // eat intent (> 0.5 = try to eat)
    OutputReproduce,                     // reproduce intent (> threshold)
    OutputReproduceMode,                 // < 0.5 = asexual, > 0.5 = sexual
    OutputSignal(u8),                    // emit signal value on channel N
    OutputShareEnergy,                   // energy sharing intent
    OutputMemoryWrite(u8),               // write to memory slot N
    OutputAttack,                        // predation intent
}
```

### Hybrid Graph Design Rules

The controller is intentionally **neurosymbolic**: neural nodes and discrete logic/arithmetic nodes coexist in one graph. To keep this evolvable and stable:

1. **Use explicit interface nodes between styles:** `Threshold` converts continuous signals into logic-like 0.0/1.0 values; `Select` routes continuous signals based on a logic-like control input.
2. **Use a default topology bias (soft prior):** graph initialization and topology mutations should *prefer* `sensors -> neural/perception -> logic/decision -> outputs`. This is a starting bias, not a hard constraint; evolution may discover profitable cross-links later.
3. **Separate mutation timescales:** weight/parameter perturbations happen frequently; logic-node type changes and structural edits happen less frequently. This reduces catastrophic behavior resets from discrete mutations.
4. **Keep numeric semantics uniform:** all node I/O remains `f32`; logic-like truth values are represented as 0.0/1.0.

### Graph Evaluation

Each tick, every creature's graph is evaluated using the following process:

1. **Populate inputs:** Sensor nodes are filled with values derived from the creature's current state and surroundings. These are computed by the world simulation (nearest food scan, creature proximity, etc.).

2. **Topological evaluation with recurrence:** Edges are classified as *forward edges* and *back edges* (those that create cycles). Back edges are identified once when the graph is mutated and cached. During evaluation, nodes are topologically sorted considering only forward edges. Each node computes its output using current-tick values for forward-edge inputs and previous-tick values (from `prev_activation_buffer`) for back-edge inputs. This allows recurrent connections to carry signals across ticks without breaking the evaluation order.

3. **Read outputs:** Output node activations are read and translated into creature actions. Movement outputs are interpreted as a direction vector (normalized, then the creature moves one cell in the closest cardinal/diagonal direction). Binary actions (eat, reproduce, attack) use threshold activation.

4. **Energy cost:** The creature is charged `energy_per_compute_node * node_count` for graph evaluation.

5. **Buffer swap:** The current activation buffer becomes the previous buffer for the next tick.

---

## Simulation Tick Loop

Each world tick proceeds in the following phases:

### Phase 1: Sense
For each creature, compute all sensor input values based on current world state. This includes spatial queries (nearest food, nearest creature) which should use efficient spatial indexing.

### Phase 2: Think
Evaluate every creature's computation graph. This is the most CPU-intensive phase and the best candidate for parallelization (each creature's evaluation is independent).

### Phase 3: Act
Collect all creature action intents. Resolve conflicts (two creatures trying to move to the same cell, simultaneous predation, etc.) using a deterministic priority system. Execute valid actions:
- **Move:** Update creature position, deduct energy cost.
- **Eat food:** Remove food from cell, add energy to creature.
- **Eat creature (predation):** If attacker has sufficient energy advantage, prey dies, attacker gains fraction of prey's energy.
- **Reproduce:** If sufficient energy, create child creature with mutated genome. Split energy between parent and child.
- **Signal:** Place signal in the world signal layer, deduct energy.
- **Share energy:** Transfer energy to adjacent creature, deduct transfer cost.
- **Memory write:** Update creature's memory registers.

### Phase 4: World Update
- Decay all creature energy by `energy_per_tick_decay`.
- Remove dead creatures (energy ≤ 0).
- Grow existing food (spread to adjacent cells at `food_growth_rate`).
- Randomly spawn new food sources at `food_spawn_rate`.
- Propagate/decay signals.
- Update lineage tree with births and deaths.

### Phase 5: Emit
Package world state delta and send to connected clients via WebSocket.

---

## Spatial Indexing

Sensor computation (nearest food, nearest creature) is potentially expensive at O(n) per creature if done naively. The world grid itself acts as a natural spatial index for food and barriers (direct cell lookup). Libraries like `rstar` (R*-trees) and `kiddo` (k-d trees) are unnecessary for a fixed discrete grid.

For creature-to-creature queries, a grid-based spatial hash using `FxHashMap<(i32, i32), Vec<CreatureId>>` from the `rustc-hash` crate maps chunk coordinates to creature lists. Sensor queries then only scan nearby chunks within sensing radius. The hash map is rebuilt each tick during Phase 4 (World Update).

For signal propagation, a simple distance-based falloff from the emitting cell is sufficient, evaluated lazily when a creature's signal sensors are computed.

---

## API Design

### WebSocket: World State Stream

The primary real-time channel. In the current single-user local model, one frontend client connects and receives world state at a configurable frame rate.

**Bandwidth budget:** At 50,000 creatures × ~12 bytes each (ID + position + color + state) = 600KB per frame. At 30fps, that's 18MB/s — far too much for WebSocket. Two mitigations are required:

1. **Strict delta compression:** `WorldDelta` messages contain only entities that moved or changed state since the last frame. In a typical tick, perhaps 30–60% of creatures move, and food changes are sparse. This alone roughly halves bandwidth.
2. **Decoupled update rate:** The server emits frames at a configurable display rate (default 10Hz, not 30Hz). The frontend interpolates creature positions between updates for smooth visual animation at 30–60fps. Simulation ticks continue at full speed regardless.

Combined, these keep bandwidth manageable for local deployment even at high population.

**Message types (server → client):**
- `WorldSnapshot`: Full world state. Sent on initial connection and periodically for sync.
- `WorldDelta`: Incremental update — only creatures that moved/changed, born, died; food cells that changed; signals emitted. This is the common per-frame message.
- `SimulationStatus`: Tick count, population stats, simulation speed, paused/running state.

**Message types (client → server):**
- `SetSpeed`: Change simulation tick rate.
- `Pause` / `Resume`: Control simulation execution.

**Wire format:** MessagePack (`rmp-serde` on the server, `@msgpack/msgpack` in the browser) for world snapshots/deltas due to volume. JSON for control messages.

### REST API: Commands and Queries

For non-streaming interactions that don't need real-time delivery.

**Configuration:**
- `GET /config` — current WorldConfig
- `PATCH /config` — update configuration values (applied at next tick)

**World interaction:**
- `POST /world/paint` — place food, barriers, or remove them at specified coordinates
- `POST /world/spawn-creature` — manually place a creature (with optional genome)
- `POST /world/clear-region` — remove all entities in a rectangular region

**Creature inspection:**
- `GET /creatures/{id}` — full creature data including graph, energy, lineage
- `GET /creatures/{id}/graph` — computation graph in a visualization-friendly format
- `GET /creatures/{id}/lineage` — ancestry chain

**Evolutionary tree:**
- `GET /lineage/tree` — full or filtered evolutionary tree
- `GET /lineage/stats` — diversity metrics, dominant lineages, extinction events

**Simulation management:**
- `POST /simulation/save` — serialize full simulation state to file
- `POST /simulation/load` — restore from saved state
- `GET /simulation/stats` — population over time, energy distribution, species count

---

## Frontend Architecture

The web frontend is a standalone React + TypeScript project built with Vite. React handles UI controls, panels, and layout. Performance-critical rendering runs on its own animation loop outside React's reactivity system.

### Technology Choices

**Framework:** React 18 + TypeScript + Vite. React manages the UI shell; Canvas handles the pixels. This is a well-documented pattern with a large ecosystem.

**World rendering:** Direct `ImageData` buffer writes, blitted to a `<canvas>` via `putImageData()`. For a 1000×1000 grid, this is a 4MB RGBA buffer updated per frame. Each cell maps to one pixel (or a small square at zoom levels). This is faster than any sprite-based or WebGL approach for 1:1 pixel mapping with no transforms. If smooth zoom, camera effects, or multi-pixel creature rendering are needed later, PixiJS (WebGL-based 2D renderer) is a clean upgrade path.

**Creature graph inspector:** ReactFlow — a React-native node graph library. Handles custom node rendering (color-coded by type, showing activation values), zoom/pan, and interaction. For graphs up to 200 nodes, performance is not a concern.

**Evolutionary tree:** `d3-hierarchy` for layout computation (tree node positions, parent-child links), with a custom Canvas renderer for drawing. SVG-based rendering breaks down at scale; Canvas with level-of-detail filtering (progressively hiding nodes with few descendants as the user zooms out) handles trees with tens of thousands of nodes.

**Live charts:** uPlot — purpose-built for real-time time series. Benchmarks at 10% CPU and 12MB RAM for 3,600 points at 60fps, compared to 40% CPU and 77MB for Chart.js. Handles incremental data appending without re-rendering full history.

**WebSocket decoding:** `@msgpack/msgpack` for deserializing binary world state frames from the Rust backend.

### Key Views

**World view (primary):** Renders the 2D grid with creatures as colored dots/pixels, food as green patches, barriers as solid regions. Supports pan and zoom. Painting tools for placing food and barriers. Click a creature to select it.

**Creature inspector (panel):** Shows selected creature's stats (energy, age, generation), computation graph rendered via ReactFlow as a node diagram (color-coded by node type, animated activation flow), genome summary, and lineage path.

**Evolutionary tree (panel/fullscreen):** Zoomable Canvas-rendered tree visualization showing lineage divergence over time. Layout computed by d3-hierarchy. Nodes represent species or significant genome branches. Color-coded by survival success or phenotype. Level-of-detail filtering for large trees.

**Configuration panel:** All WorldConfig knobs exposed as sliders and toggles, adjustable at runtime. Presets for common scenarios (abundant world, harsh world, predator-prey, social).

**Statistics dashboard:** Live uPlot time-series charts for population over time, average energy, species diversity, food availability, and average genome complexity.

---

## Serialization and Persistence

The full simulation state (world grid, all creatures with genomes, lineage tree, config, and RNG runtime state) must be serializable for save/load. Two formats serve two purposes:

**Save/load (Rust-to-Rust):** `postcard` — a compact, fast binary format with serde support. ~30% smaller than bincode, actively maintained. (`bincode` has a RUSTSEC-2025 advisory marking it unmaintained and should not be used.)

**WebSocket streaming (Rust-to-JavaScript):** MessagePack via `rmp-serde` — a cross-language binary format with mature JavaScript decoders (`@msgpack/msgpack`).

Save files should include a format version number so saves from earlier stages remain loadable as the simulation evolves. Lineage data may also be exported separately in JSON for external analysis or visualization tools. Deterministic replay is not a current requirement.

---

## Performance Considerations

**Target scale:** 10,000–50,000 creatures in a 1000×1000 world running at 30+ ticks per second on modern hardware.

**Parallelism:** Phase 2 (graph evaluation) is embarrassingly parallel — `petri-server` uses `rayon` for data-parallel creature evaluation (`.par_iter()` over the creature SlotMap). Phase 1 (sensing) can also be parallelized per creature given read-only world access. Phase 3 (action resolution) is sequential due to conflict resolution — this is intentional, as it avoids contention on the innovation number counter (used during reproduction/mutation). Note: parallelism is injected by `petri-server`, while core crates stay runtime-agnostic.

**RNG strategy:** Each creature owns its own `SmallRng`, seeded from the parent's RNG at birth. This avoids shared mutable RNG state during parallel work and keeps stochastic behavior localized to each creature. The world-level `SmallRng` is used only for world events (food spawning, catastrophes). Exact deterministic replay across runs is out of scope for the current roadmap.

**Rayon + Tokio integration:** The simulation tick loop uses `rayon` internally, but the server's async runtime is `tokio`. To avoid blocking tokio worker threads (which would starve WebSocket heartbeats), the tick loop must be wrapped in `tokio::task::spawn_blocking`:
```rust
let next_state = tokio::task::spawn_blocking(move || {
    world.tick() // rayon parallelism runs safely inside here
}).await?;
```

**Memory layout:** Creature graphs use `petgraph::Graph` (adjacency list) for cache-friendly node/edge storage. The `HopSlotMap` for creature storage avoids indirection, supports stable IDs, and provides fast iteration that skips vacant slots.

**Frontend throttling:** The backend simulation may run faster than the frontend can render. The WebSocket stream should be decoupled from the tick rate — emit frames at a configurable display rate (e.g., 30fps) regardless of simulation speed. When running at high speed, the stream sends sampled snapshots rather than every tick.

---

## Known Tradeoffs and Future Optimizations

**Floating-point variability across platforms:** `f32` math can differ across CPU architectures and compiler optimization settings (for example, x86_64 vs ARM64, FMA behavior). Small cross-platform numeric divergence is acceptable in the current project goals. Moving to fixed-point math (`fixed` crate) is a future option only if strict numerical reproducibility becomes necessary.

**petgraph memory at extreme scale:** Each creature's `petgraph::Graph` involves 2–3 heap allocations for its internal vectors. At 50,000 creatures this means ~150,000 allocations. Modern allocators (jemalloc, mimalloc) handle this well, and the graphs are long-lived. If profiling reveals cache pressure at scale, a future optimization path is a `SmallGraph` struct using inline storage (`SmallVec` or `TinyVec`) for creatures below a node count threshold, falling back to heap allocation for larger graphs.

## Future Considerations

These are not in the initial architecture but are anticipated for later stages:

- **Multi-resource economy:** Multiple food types with different energy values, possibly requiring different evolutionary adaptations to consume.
- **Environmental hazards:** Regions with energy drain, periodic catastrophes (floods, famines), day/night cycles affecting food growth.
- **Creature morphology:** Evolvable body plans beyond a single pixel — multi-cell creatures, sensory appendages, size as a continuous trait.
- **Distributed simulation:** Partitioning the world across multiple cores or machines for very large worlds.
- **Optional replay tooling:** If needed later, record full tick history or event streams for offline replay and analysis.

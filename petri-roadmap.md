# Petri — Implementation Roadmap

## Guiding Principles

Each stage produces a working, inspectable simulation. No stage is a "setup-only" milestone — every stage should be something you can run and verify. Later stages build on earlier ones without requiring rewrites, because the core architecture (graph-based creatures, energy economics, tick loop) is designed to accommodate all planned features from the start.

## Rebaseline (2026-02-11)

- Stage 1 is complete and remains the baseline for all future stage planning.
- Already-landed baseline primitives include creature-direction/distance/local-density sensing, move-blocked feedback, memory read/write baseline, and arithmetic helpers (`Negate`, `Abs`, `Min`, `Max`).
- Stage 2 remains full scope and will be delivered as vertical slices.

---

## Stage 1: Minimal Viable Life

**Goal:** A working simulation where creatures can sense, act, reproduce, and die. Validates the entire pipeline from Rust simulation to web visualization.

### Internal Milestone: Walking Skeleton (Stage 1a)
Before building the full graph engine, get visual feedback by building the minimum end-to-end pipeline: grid world, creatures with hardcoded random-walk behavior (no graph evaluation yet), food, energy, death, WebSocket streaming, and a basic Canvas renderer. This validates the architecture (Rust tick loop → axum WebSocket → React canvas) with the smallest possible scope. Once pixels are moving on screen, replace the hardcoded behavior with the graph evaluation engine.

### Internal Milestone: Hybrid Stability Check (Stage 1b)
After graph evaluation is running, add a small ablation harness that runs three controller palettes under identical world settings:
- Neural-only palette
- Logic/arithmetic-only palette
- Hybrid palette (neural + logic + interface nodes)

The purpose is not to prove superior behavior. It is to verify that all three modes run stably, produce valid actions, and expose useful diagnostics for tuning mutation and energy settings.

### Simulation Features
- 2D grid world with configurable dimensions (start with 200×200)
- Food cells with energy value; food regrows at configurable rate; food randomly spawns
- Creatures as single-cell entities with position and energy
- Passive energy decay per tick
- Movement in 8 directions (cardinal + diagonal), costing energy
- Eating: creature on a food cell can consume it, gaining energy
- Death when energy reaches zero
- Asexual reproduction: creature splits energy with offspring, offspring gets mutated copy of parent's graph
- Computation graph evaluation with energy cost per node
- Per-creature RNG (seeded from parent at birth) to keep stochastic behavior local and avoid shared RNG contention under parallel execution
- Per-creature event log (small ring buffer of recent events: "ate food", "reproduced", "starved") for debugging and inspection

### Creature Computation (Stage 1 Subset)
Input nodes available:
- `SensorFoodDirection` — angle to nearest food within a limited sensing radius
- `SensorFoodDistance` — distance to nearest food
- `SensorFoodHere` — food value at current cell
- `SensorEnergy` — own energy level (0.0–1.0 normalized)
- `SensorRandom` — random float each tick

Hidden node types available:
- `NeuralSigmoid`, `NeuralTanh`, `NeuralRelu` — basic neural processing
- `Threshold` — converts float inputs to logic-like 0.0/1.0 outputs
- `Select` — routes one of two float inputs based on a control input
- `Add`, `Multiply` — arithmetic
- `GreaterThan` — comparison
- `Constant` — fixed value output
- Graph initialization and topology mutation bias: prefer `sensor -> neural/perception -> logic/decision -> outputs` as a default prior (not a hard constraint)

Output nodes available:
- `OutputMoveX`, `OutputMoveY` — movement direction
- `OutputEat` — eat intent (threshold-based)
- `OutputReproduce` — reproduction intent (threshold-based)

### Mutation Operators (Stage 1 Subset)
- Perturb edge weight/parameter (most common mutation, high rate)
- Add a new hidden node (splicing an existing edge)
- Add a new edge between existing nodes
- Remove an edge
- Remove a disconnected hidden node
- Change a hidden node's type (within available types, low rate for logic/converter nodes)

### Genome and Lineage
- Each creature gets a unique lineage ID at creation; offspring inherit the parent's lineage but get their own creature ID
- Track parent-child relationships from the start — the lineage tree data structure should be built in stage 1 even if the visualization comes later
- Generation counter per creature

### Backend Deliverables
- `petri-core` crate: world grid (`Vec<Cell>`), creature storage (`HopSlotMap`), tick loop, action resolution. No dependency on server/runtime frameworks (`tokio`, `axum`) so logic is reusable in server and CLI.
- `petri-graph` crate: graph structure (backed by `petgraph::Graph`), stage 1 node types, evaluation engine. Keep it as a pure computation crate with no server/runtime dependencies.
- `petri-genome` crate: mutation operators, innovation numbering for future crossover, lineage tracking. Keep it as a pure computation crate with no server/runtime dependencies.
- `petri-server` crate: `axum` + `tokio` WebSocket world stream (MessagePack frames via `rmp-serde`), REST config/command API (JSON). `rayon` for parallel creature evaluation.
- `petri-cli` crate: headless runner with basic stats output (population count, avg energy, avg genome size per tick)

### Frontend Deliverables
- React + TypeScript + Vite project scaffold
- World renderer using direct `ImageData` buffer writes to `<canvas>`: grid with food (green intensity), creatures (white or colored dots), empty space (black). WebSocket connection receiving MessagePack frames via `@msgpack/msgpack`.
- Pan and zoom
- Config panel: React-based sliders for all WorldConfig values, pause/resume, speed control. REST API calls via `PATCH /config`.
- Basic creature click-to-inspect: show energy, age, generation, node count
- Population and average energy time-series charts (live, rendered with uPlot)

### Configuration Knobs Active
- `food_spawn_rate`, `food_growth_rate`, `food_max_density`, `food_energy_value`
- `energy_per_tick_decay`, `energy_per_move`, `energy_per_compute_node`
- `energy_per_reproduce`, `energy_initial`, `energy_max`
- `weight_mutation_rate`, `weight_mutation_magnitude`, `logic_node_mutation_rate`, `structural_mutation_rate`
- `min_reproduce_energy`, `offspring_energy_fraction`
- `world_wrap`, `max_creatures`

### What You'll See
Creatures moving around a food field, consuming energy, reproducing, and dying according to configured rules. Changing configuration values should produce predictable mechanical changes (for example, higher movement cost reduces mobility and population).

### Definition of Done
- Simulation runs at 30+ ticks/sec with 5,000 creatures on a 200×200 grid
- Creatures execute graph-driven actions and lifecycle transitions as expected (move/eat/reproduce/die)
- Neural-only, logic-only, and hybrid controller palettes complete fixed-length test runs and emit comparable diagnostics (population, lifetime, action counts)
- All config knobs adjustable at runtime via frontend
- Full simulation state serializable and loadable (save/load)

### Stage 1 Status (2026-02-11)
- [x] Core simulation loop and graph-driven lifecycle are implemented
- [x] Stage 1 sensors/nodes and mutation operators are implemented
- [x] Lineage tracking, world-wrap toggle, and full config surfaces are implemented
- [x] Inspector, live metrics chart, and snapshot save/load are implemented
- [x] 5k-creature throughput benchmark binary is available (`petri-cli --bin stage1_benchmark`)

### Baseline Extensions Already Landed
- `SensorCreatureDirection`, `SensorCreatureDistance`, `SensorLocalDensity`, `SensorMoveBlockedLastTick`
- `InputMemoryRead` + `OutputMemoryWrite` baseline single-register read/write loop
- `Negate`, `Abs`, `Min`, `Max`

---

## Stage 2: Rich Environment

**Goal:** Add environment editing and richer inspection tools while preserving stable simulation behavior, delivered as full-scope vertical slices.

### Planned Interfaces (Stage 2)
- `POST /simulation/world/paint`
- `GET /simulation/lineage/tree`
- `GET /simulation/creature/{id}/graph`
- Frame payload additions: barriers, phenotype color, expanded metrics
- Config addition: `sensor_radius`

### Vertical Slices

**Slice 1: Barrier substrate + frame/render**
- Add barrier cells as impassable world state.
- Extend frame payload and frontend renderer to display barriers.

**Slice 2: Paint API + paint UI**
- Add world painting API for food/barrier placement and erase behavior.
- Add paint tools in frontend (food, barrier, eraser with brush support).

**Slice 3: Barrier-aware food rules + configurable sensing radius**
- Ensure food spawn/spread respects barriers and occupied cells.
- Add `sensor_radius` configuration with frontend controls.

**Slice 4: Barrier sensors + inspector exposure**
- Add `SensorBarrierDirection` and `SensorBarrierDistance`.
- Expose barrier sensing values in creature inspector payload/views.
- 
**Slice 4.1: Creature container slots (this feature needs more refinement before implementatino)**
- Add "container slots" as a feature to creatures that can be used to store barriers or food.
- Number of container slots is evolvable
- Add ability for creatures to pick up food or barriers and insert them into container slots
- Add ability for creatures to deposit items from container slots
- Full container slots multiply movement cost
  
**Slice 5: Phenotype color pipeline**
- Add deterministic phenotype color derived from genome structure.
- Render phenotype colors in world view and related inspection surfaces.

**Slice 6: Evolutionary tree API + canvas visualization**
- Add lineage tree query/filter API.
- Add scalable tree visualization (`d3-hierarchy` layout + canvas render).

**Slice 7: Graph inspector + expanded metrics**
- Add interactive computation-graph inspector view (ReactFlow-based).
- Add expanded metrics: species diversity, food availability, average genome complexity.

### What You'll See
With barriers, you can create mazes, islands, and corridors. Painting food and barrier regions immediately affects movement and resource access. The tree and graph-inspector views remain navigable at target event counts, and expanded metrics make population dynamics easier to interpret.

### Definition of Done
- Barriers fully functional; creature movement correctly respects blocked cells
- Painting tools work smoothly in the frontend
- Barrier-aware food rules and `sensor_radius` controls are fully wired and tested
- Evolutionary tree renders and is navigable for simulations with 10,000+ birth events
- Creature graph inspector is interactive and shows current node/edge activation context
- Expanded statistics (species diversity, food availability, average genome complexity) stream and render correctly
- Phenotype colors are stable and derived consistently from genome data

---

## Stage 3: Predation

**Goal:** Introduce creature-eats-creature dynamics with correct conflict resolution and tunable safety controls.

### New Simulation Features
- Predation action: a creature adjacent to another creature can attempt to consume it
- Predation resolution: based on relative energy levels (and optionally size). The attacker must have significantly more energy than the defender to succeed. If successful, the prey dies and the predator gains a fraction of its energy. If unsuccessful, both lose some energy from the struggle.
- Defense: no explicit defense action — energy level acts as implicit armor. Creatures that are well-fed are harder to kill.
- **Population safety mechanisms:** Predation often causes extinction spirals in early evolutionary sims (predators eat everyone, then starve). Two mitigations: (1) a configurable minimum population floor below which predation is disabled, and (2) optional "nursery" zones where predation is disabled and food is abundant, allowing populations to recover from crashes.

### New Creature Computation
Input nodes added:
- `SensorCreatureEnergy` — energy level of nearest creature (relative to self)

Output nodes added:
- `OutputAttack` — predation intent

Hidden nodes added:
- `Gate` — conditional signal pass-through (useful for "if big enough to eat, approach; else flee")

Baseline note:
- `SensorCreatureDirection`, `SensorCreatureDistance`, `Negate`, `Abs`, `Min`, and `Max` are already part of the pre-Stage-3 baseline and are not introduced as new Stage 3 primitives.

### New Configuration Knobs
- `predation_enabled` — toggle predation globally
- `predation_energy_transfer` — fraction of prey energy gained (e.g., 0.5)
- `predation_size_advantage` — how much relative energy matters (threshold multiplier)
- `energy_per_attack` — energy cost of attempting predation (even if unsuccessful)
- `predation_min_population` — population floor below which predation is disabled (default 0 = no floor)

### What You'll See
Predation attempts occurring in-world with clear success/failure outcomes, energy transfer, and expected death handling. Population safety controls should prevent runaway collapse during aggressive tuning.

### Definition of Done
- Predation mechanics work correctly (energy transfer, death, conflict resolution)
- Predation configuration knobs produce expected mechanical effects in tests and runtime
- Creature inspector exposes inputs/outputs relevant to predation debugging (`SensorCreature*`, `OutputAttack`)

---

## Stage 4: Social Layer

**Goal:** Add communication, kin recognition, sharing, memory, and sexual reproduction mechanics with clear observability.

### New Simulation Features

**Signaling:**
- Creatures can emit a signal value (float) on one of N configurable channels
- Signals propagate to cells within `signal_range` with distance falloff
- Signals last one tick (or configurable decay) then dissipate
- Signal emission costs energy

**Kin Recognition:**
- Each creature has a kin tag (short sequence of values, inherited with slight mutation)
- Creatures can sense the kin similarity of the nearest creature (cosine similarity or hamming distance of tags)
- This doesn't directly affect anything — it just provides information that the creature's graph can use to make decisions

**Resource Sharing:**
- New output: `OutputShareEnergy` — when activated and adjacent to another creature, transfer a portion of own energy to that creature
- Energy transfer has a cost (some energy lost in transit to prevent perpetual motion)

**Sexual Reproduction:**
- New output: `OutputReproduceMode` — controls whether reproduction is asexual or sexual
- Sexual reproduction requires adjacency with another creature that is also signaling reproductive intent
- Offspring genome is crossover of both parents' graphs (aligned by node historical markers, similar to NEAT), then mutated
- Offspring kin tag is blend of parents' tags

**Memory (upgrade from baseline):**
- Baseline (already landed): single-register read/write loop via `InputMemoryRead` and `OutputMemoryWrite`.
- Stage 4 target: indexed multi-slot memory (`SensorMemory(N)`, `OutputMemoryWrite(N)`) with richer semantics and observability.
- Enables creatures to maintain structured state across ticks without relying solely on recurrent graph connections.

### New Creature Computation
Input nodes added:
- `SensorCreatureKinSimilarity` — how similar nearest creature's kin tag is
- `SensorSignal(N)` — incoming signal value on channel N
- `SensorPopulationDensity` — creature count in local area
- `SensorMemory(N)` — indexed read from memory register slot `N` (beyond baseline single-register behavior)

Output nodes added:
- `OutputSignal(N)` — emit signal on channel N
- `OutputShareEnergy` — energy transfer intent
- `OutputReproduceMode` — asexual/sexual toggle
- `OutputMemoryWrite(N)` — indexed write to memory register slot `N` (beyond baseline single-register behavior)

Hidden nodes added:
- `Accumulator` — running sum across ticks (useful for integrating signals over time)
- `Delay` — output previous tick's input (explicit single-tick memory)

### New Configuration Knobs
- `signaling_enabled`, `signal_channels`, `signal_range`
- `kin_tag_length`
- `energy_sharing_enabled`, `energy_share_transfer_rate`, `energy_share_cost`
- `crossover_rate` — probability of gene crossover during sexual reproduction
- `memory_slots` — number of memory registers per creature

### What You'll See
Signals appearing in-world, kin similarity values available to creature logic, energy-sharing transfers between neighbors, memory reads/writes affecting outputs, and sexual reproduction producing crossover offspring structures.

### Definition of Done
- Signaling works: signals visible in the world view as colored overlays, creatures respond to signals
- Kin tags are generated, inherited, and exposed via sensors/inspector
- Energy sharing transfers energy with configured transfer rate and cost
- Sexual reproduction produces offspring with combined parent traits
- Memory registers are readable/writable through graph nodes and visible in inspector/debug views

---

## Stage 5: Group Mechanics and Analysis

**Goal:** Add group-level analysis tooling and environmental stressors to evaluate social mechanics.

### This Stage Is Primarily About Tuning and Environment Design

By stage 4, all the mechanical ingredients exist. Stage 5 focuses on stress-testing and measuring those mechanics at group scale.

### Environmental Additions
- **Hazard zones:** Regions that drain energy at an accelerated rate, used to stress-test sharing and adaptation mechanics.
- **Periodic catastrophes:** Events that kill creatures below an energy threshold, used to test recovery and resilience.
- **Rich/barren regions:** Uneven food distribution, used to test migration and information-sharing mechanisms.
- **Scaling predation:** Tunable predator pressure (higher energy transfer, faster) to test social responses under threat.

### Analysis and Visualization Additions
- **Group detection:** Algorithm to identify clusters of kin-similar creatures. Display group boundaries in the world view.
- **Group statistics:** Track group size, group average energy, inter-group vs intra-group interactions over time.
- **Behavioral profiling:** Classify creatures by behavior pattern (forager, predator, sentinel, sharer) based on action frequency distribution. Show population breakdown by behavioral type.
- **Signal analysis:** Visualize signal patterns — which signals correlate with which events? Are different groups using signals differently?

### What You'll See
Group overlays, metrics, and signal analysis views updating as simulations run under hazard/catastrophe configurations. The focus is on instrumentation quality and interpretability rather than guaranteeing specific emergent outcomes.

### Definition of Done
- Group detection runs on live data and renders boundaries correctly
- Group statistics and interaction metrics update correctly over time
- Signal analysis views correlate emitted signals with configured world events
- Visualization clearly shows group dynamics (boundaries, signals, resource flow)

---

## Cross-Cutting Concerns (All Stages)

### Testing Strategy
- **Unit tests:** Per-crate tests for graph evaluation, mutation correctness, energy accounting, action resolution.
- **Integration tests:** Full tick-loop tests with known initial conditions asserting expected outcomes.
- **Ablation tests:** Neural-only vs logic-only vs hybrid palette runs with fixed configs to validate controller stability and diagnostic outputs.
- **Statistical tests:** Run headless simulations for N ticks and verify properties (population doesn't immediately go extinct, energy is conserved, lineage tree is consistent).
- **Benchmark tests:** Measure tick rate at target creature count to catch performance regressions.

### Save/Load
- Available from stage 1. Full simulation state serialized via `postcard` (compact binary serde format), including RNG runtime state needed for correct continuation after load. (Not `bincode` — it has a RUSTSEC-2025 unmaintained advisory.)
- Versioned format so saves from earlier stages remain loadable.

### Configuration Presets
- Build up a library of configuration presets that produce interesting dynamics: "Abundant World", "Famine", "Predator Arena", "Island Archipelago", "Cooperative Pressure".

### Documentation
- Each stage should include updated API documentation for new endpoints and WebSocket message types.
- A living "field guide" document that catalogs notable simulation states, parameter effects, and debugging notes observed during development.

---

## Rough Timeline Guidance

These aren't hard deadlines — they're rough estimates of relative complexity to help with planning.

| Stage | Relative Effort | Key Risk |
|-------|----------------|----------|
| Stage 1 | Large (40%) | This is the foundation — world sim, graph engine, API, frontend. Most of the infrastructure is built here. |
| Stage 2 | Medium-Large (20%) | Full-scope vertical slices span simulation, API, and frontend; sequencing and payload boundaries are the main risk. |
| Stage 3 | Medium (12%) | Predation conflict resolution needs careful design. Balancing predation parameters is iterative. |
| Stage 4 | Medium-Large (18%) | Sexual reproduction crossover is algorithmically complex. Multiple new systems (signals, kin, indexed multi-slot memory, sharing). |
| Stage 5 | Small-Medium (10%) | Mostly tuning, environmental additions, and analysis tooling. Risk is metric complexity and interpretation overhead. |

Stage 1 is the critical path. Once it's working, the remaining stages are incremental additions to a proven foundation.

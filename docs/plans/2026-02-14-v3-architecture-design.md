# Petri V3 Architecture Design

**Goal:** Design a clean, maintainable architecture for v3 that achieves the v2 vision (complex mesh creatures with VM+Graph nodes) while avoiding v2's leaky abstraction failures.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Complete v3 backend architecture including module boundaries, data flow, testing strategy, and walking skeleton stages. Includes core simulation (v3-core) and transport layer (v3-server). Excludes specific VM instruction set details and graph operator implementations (deferred to implementation). Frontend (v3/web) is excluded — it will be designed in a separate frontend architecture plan.

**See also:**
- `docs/reference/v3-vm-isa-spec.md` — VM instruction set, opcodes, execution rules, numeric determinism
- `docs/reference/v3-graph-operator-spec.md` — Graph operators, cost model, local state contract
- `docs/reference/v3-genome-sensor-spec.md` — Genome schema, typed I/O, sensor system, normalization
- `docs/reference/v3-creature-lifecycle-spec.md` — Mutation, reproduction, phenotype evolution

**Docs Impact:**
- Creates v3 architecture design document
- Will supersede `docs/strategy/architecture.md` when v3 becomes primary implementation
- v2 remains as reference for "what not to do"

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

- **GP-01**: Provides expressive substrate for richer emergent behavior through VM+Graph mesh architecture, rich sensors, energy-bound processing, and phenotype evolution
- **GP-02**: Establishes explicit ownership boundaries across kernel, contracts, config, creature, runtime, sensors, and tick modules to prevent leaky abstractions
- **GP-03**: Enforces testable contracts through dedicated contracts/ module and intent-verification testing strategy
- **GP-04**: Tick returns TickStats (births, deaths, action counts) for runtime observability; transport layer surfaces these to product UIs

---

## Boundary Impact

- New `v3/` root directory owns all v3 implementation
- v1 (`crates/petri-*`, `web/`) remains working reference
- v2 (`v2/`) remains as reference for architectural failures
- Independent toolchain manifests in `v3/` (no coupling to v1 or v2)
- Clear dependency direction enforced by convention and code review

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| v1 `crates/petri-*` | keep | Working reference for proven patterns (world/food, world/tick modular structure) |
| v2 `v2/` directory | keep | Reference for what NOT to do (leaky abstractions, poor integration) |
| v3 `v3/` directory (new) | change | Clean slate required to avoid v2's architectural mistakes |
| `docs/strategy/architecture.md` | keep | Will be updated when v3 becomes primary |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should v3 use randomized turn order or two-phase commit? | Randomized turn order (simpler, solves bias, can upgrade later). | user+agent | resolved |
| Where do input/output contracts live? | Dedicated `contracts/` module (interface segregation, modules as peers). | user+agent | resolved |
| Where does energy logic live? | Energy type in `creature/`, energy config in `config/`, operations distributed. | user+agent | resolved |
| Should reproduction spawn immediately or defer? | Defer: collect offspring during action execution, append after all actions complete. Keeps SlotMap keys valid during Phase 2. | user+agent | resolved |
| What happens if reproduce has no spawn location? | Reproduction fails, parent loses energy (consistent with other failed actions). | user+agent | resolved |
| Should we use granular config submodules? | Yes, split by concern (world/food, energy/costs, runtime/vm, etc.). | user+agent | resolved |

---

## Repository Architecture

```
petri/
├── v3/                          # New v3 implementation
│   ├── Cargo.toml              # Workspace manifest
│   ├── rust-toolchain.toml     # Pin Rust version
│   ├── README.md               # v3 purpose & commands
│   ├── crates/
│   │   ├── v3-core/            # Simulation kernel + capabilities
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── kernel/     # Core primitives (world state, types)
│   │   │   │   ├── contracts/  # Interface contracts (inputs, outputs)
│   │   │   │   ├── config/     # Simulation configuration (granular)
│   │   │   │   ├── creature/   # Genome, phenotype, mutation, founders, state
│   │   │   │   ├── runtime/    # VM + Graph execution
│   │   │   │   ├── sensors/    # Input assembly
│   │   │   │   ├── tick/       # Tick orchestration
│   │   │   │   └── seed.rs     # Initial creature placement
│   │   │   └── tests/          # Integration tests
│   │   ├── v3-server/          # HTTP + WebSocket transport
│   │   └── v3-cli/             # Headless runner
├── v2/                         # Reference: what NOT to do
├── crates/                     # v1 working reference
├── web/                        # v1 web client
└── docs/                       # Canonical strategy/plans
```

---

## Dependency Flow

```
v3-core internal:
  contracts/   → kernel/ (references primitives like Position)
  config/      → (no dependencies, pure data)
  creature/    → kernel/ + config/ + contracts/
  runtime/     → kernel/ + config/ + contracts/ + creature/
  sensors/     → kernel/ + config/ + contracts/ + creature/
  tick/        → kernel/ + config/ + contracts/ + creature/ + runtime/ + sensors/
  seed         → kernel/ + creature/ (founders, state) + config/ + SimulationState
  SimulationState (lib.rs) → tick/ + kernel/ + creature/ + config/

v3 workspace:
  v3-server → v3-core (owns a SimulationState)
  v3-cli → v3-core (owns a SimulationState)
```

**Key insight:** contracts/ is the "clipboard" modules share. Kernel owns primitives, capabilities transform and consume via contracts. `SimulationState` is the thin coordination shell that bundles mutable simulation state — callers (server, cli) own one and call `state.tick(config, rng)`.

**Error philosophy:** v3-core panics on invariant violations (`debug_assert` for internal invariants, `assert` for preconditions that indicate programming errors). Invalid input from within the crate is a programming error, not an expected runtime condition. The `Result` type is reserved for fallible operations at crate boundaries (e.g., config parsing, file I/O in server/cli). Within v3-core, functions either succeed or panic — there are no "soft failures" for impossible states.

---

## Module Design

### Kernel (Foundational Primitives)

**Purpose:** World reality - food, barriers, positions, occupancy. No business logic.

**Files:**
- `kernel/types.rs` - Position, Direction (with `delta() -> (i32, i32)`), CreatureId (slotmap key type)
- `kernel/world_state.rs` - Food grid, barriers, occupancy tracking, position resolution

**Key methods:**
```rust
// kernel/world_state.rs
pub fn get_food_density(&self, pos: Position) -> u8;
pub fn set_food_density(&mut self, pos: Position, density: u8);
pub fn consume_food(&mut self, pos: Position) -> u8;
pub fn is_barrier(&self, pos: Position) -> bool;
pub fn is_occupied(&self, pos: Position) -> bool;      // convenience: creature_at(pos).is_some()
pub fn creature_at(&self, pos: Position) -> Option<CreatureId>;
pub fn place_creature(&mut self, pos: Position, id: CreatureId);
pub fn remove_creature(&mut self, pos: Position);
pub fn grow_food(&mut self, config: &FoodConfig, rng: &mut impl Rng);
```

**Storage:** All per-cell data uses flat `Vec` arrays indexed by `y * width + x` for cache-friendly O(1) access. Avoid HashMap/HashSet for grid data — a 400x400 world is only 160K cells. The `creature_at` spatial index (`Vec<Option<CreatureId>>`) replaces boolean occupancy — it provides O(1) position→creature lookup and occupancy checking in a single array.

**Position resolution:** Kernel owns world geometry. All position arithmetic (neighbor calculation, wrapping, bounds checking) lives here — not in sensors or tick.
```rust
// kernel/world_state.rs

/// Resolve a neighbor position, respecting wrap/clamp. Returns None if
/// the target is out-of-bounds in a non-wrapping world.
pub fn resolve_neighbor(&self, pos: Position, dir: Direction) -> Option<Position> {
    let (dx, dy) = dir.delta();
    let nx = pos.x as i32 + dx;
    let ny = pos.y as i32 + dy;

    if self.wrap {
        Some(Position {
            x: ((nx % self.width as i32 + self.width as i32) % self.width as i32) as u16,
            y: ((ny % self.height as i32 + self.height as i32) % self.height as i32) as u16,
        })
    } else if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
        Some(Position { x: nx as u16, y: ny as u16 })
    } else {
        None
    }
}
```

**Bounds safety:** `cell_index` is private and assumes valid positions. All public methods that accept `Position` must validate bounds (debug_assert or early return). `resolve_neighbor` is the canonical way to produce valid positions from user-supplied directions.

**Principle:** Kernel has NO knowledge of creature behavior, actions, VM, or ticks. Pure world primitives. The kernel does store `CreatureId` in the spatial index (it's a primitive key type, not creature logic).

---

### Contracts (Interface Agreements)

**Purpose:** Shared language between capabilities. Data transfer objects, no behavior.

**Files:**
- `contracts/inputs.rs` - EnvironmentalInputs, IntrospectionInputs, CreatureInputs
- `contracts/outputs.rs` - WorldAction, InternalOutputs, CreatureOutputs

**Why separate from kernel:** Contracts are "how we see reality" (perspective), kernel is "what reality is" (objective truth). Keeping them separate prevents coupling runtime/sensors to kernel implementation details.

**Example:**
```rust
// contracts/inputs.rs
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
    pub neighbor_n: bool,
    pub barrier_n: bool,
    // ... sensor data
}

// contracts/outputs.rs
pub enum WorldAction {
    Move { direction: Direction },
    Eat,
    Reproduce { direction: Direction, energy_amount: u32 },
    PickupFood { direction: Direction, slot: u8 },
    PickupBarrier { direction: Direction, slot: u8 },
    PlaceFood { direction: Direction, slot: u8 },
    PlaceBarrier { direction: Direction, slot: u8 },
    NoOp,
}
```

---

### Config (Simulation Parameters)

**Purpose:** All tunable simulation parameters, organized by concern.

**Structure (Granular):**
```
config/
├── mod.rs
├── world/
│   ├── mod.rs
│   ├── dimensions.rs    # WorldDimensions: width, height, wrap
│   └── food.rs          # FoodConfig: growth, spread, spawn
├── energy/
│   ├── mod.rs
│   ├── lifecycle.rs     # EnergyLifecycle: initial, max, decay
│   └── costs.rs         # EnergyCosts: action costs, rewards
└── runtime/
    ├── mod.rs
    ├── vm.rs            # VmConfig: memory size, registers
    ├── sensors.rs       # SensorConfig: radius
    ├── execution.rs     # ExecutionConfig: max think steps
    └── mutation.rs      # MutationConfig: mutation rates
```

**Principle:** Config is cross-cutting concern that affects multiple modules. Centralized, organized by domain, easy to serialize.

---

### Creature (Genome, Phenotype, State)

**Purpose:** What creatures ARE (data, identity, heritable traits). Not what they DO (runtime).

**Files:**
- `creature/state.rs` - CreatureState, Energy type, `new_founder()` constructor
- `creature/genome.rs` - CreatureGenome, NodeGenome, validation
- `creature/founders.rs` - Named founder genome registry (`test`, `simple`)
- `creature/phenotype.rs` - Phenotype, color evolution, founder baseline
- `creature/mutation.rs` - mutate_genome()
- `creature/reproduction.rs` - create_offspring()

**Energy design:**
```rust
// creature/state.rs
pub struct Energy(u32);  // Integer energy avoids floating-point flakiness in tests

impl Energy {
    pub fn drain(&mut self, amount: u32) -> bool;  // Returns success (false if insufficient)
    pub fn charge(&mut self, amount: u32, max: u32);
    pub fn is_alive(&self) -> bool;  // self.0 > 0
    pub fn value(&self) -> u32;
}
```

Energy uses `u32` (not `f32`) to ensure deterministic equality comparisons and avoid floating-point edge cases in tests. All config values (costs, rewards, thresholds) are also integers. Energy is a creature property. Config defines energy policy (costs, gains). Runtime and tick modify energy through the Energy type's safe operations.

**Offspring creation:**
```rust
// creature/reproduction.rs
pub fn create_offspring(
    parent: &CreatureState,
    spawn_position: Position,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> CreatureState {
    // Mutate genome, derive phenotype, calculate energy, create state
}
```

**Per-creature memory:** CreatureState will gain a `memory: Vec<u8>` field in Stage 3 (VM brain). This is the persistent scratch space the VM reads/writes across ticks. Deferred from Stage 1 because the stub runtime doesn't use it.

**CreatureId:** `CreatureId` is a `slotmap` key type. The SlotMap generates IDs automatically on insert — no manual ID allocator needed.

**Principle:** Creature owns data about what creatures are. Reproduction logic lives here (how to make offspring), but tick decides when reproduction happens.

---

### Runtime (VM + Graph Execution)

**Purpose:** Execute creature brains with energy-bound processing. Produces outputs, doesn't modify world.

**Files:**
- `runtime/executor.rs` - Energy-bound think loop orchestration
- `runtime/vm.rs` - VM instruction execution
- `runtime/graph.rs` - Graph node operator execution
- `runtime/memory.rs` - Memory read/write operations

**Energy-bound execution:**
```rust
// runtime/executor.rs
pub fn execute_creature(
    genome: &CreatureGenome,
    inputs: CreatureInputs,
    memory: &mut [u8],
    energy: &mut Energy,
    config: &SimulationConfig,
) -> CreatureOutputs {
    let max_steps = config.runtime.execution.max_think_steps_per_tick;
    let step_cost: u32 = config.energy.costs.energy_per_think_step;

    let mut step_count = 0;
    while step_count < max_steps {
        if !energy.drain(step_cost) {
            break;  // Out of energy
        }

        step_count += 1;
        let outputs = execute_node(genome, &inputs, memory, config);

        if outputs.halted || outputs.action_decided {
            return outputs;
        }
    }

    // Default to NoOp if no decision made
    CreatureOutputs::noop()
}
```

**Principle:** Runtime executes brains, doesn't modify world. Returns outputs (world action + internal outputs + energy consumed). Tick uses these outputs to modify world/creatures.

---

### Sensors (Input Assembly)

**Purpose:** Assemble inputs from various sources into contract format. Doesn't own data, just provides views.

**Files:**
- `sensors/environmental.rs` - Gather environmental inputs from world
- `sensors/introspection.rs` - Gather introspection inputs from creature
- `sensors/mod.rs` - Unified gather_inputs() function

**Input assembly:**
```rust
// sensors/mod.rs
pub fn gather_inputs(
    creature: &CreatureState,
    world: &WorldState,
    config: &SimulationConfig,
) -> CreatureInputs {
    CreatureInputs {
        environmental: gather_environmental_inputs(creature.position, world, config),
        introspection: gather_introspection_inputs(creature, config),
    }
}

// sensors/environmental.rs
pub fn gather_environmental_inputs(
    position: Position,
    world: &WorldState,
    config: &SimulationConfig,
) -> EnvironmentalInputs {
    // Use kernel's resolve_neighbor for position arithmetic
    let neighbor_n = world.resolve_neighbor(position, Direction::N);

    EnvironmentalInputs {
        food_density_self: world.get_food_density(position),
        neighbor_n: neighbor_n.map_or(false, |p| world.is_occupied(p)),
        barrier_n: neighbor_n.map_or(true, |p| world.is_barrier(p)),  // OOB = barrier
        // ... assemble all fields
    }
}
```

**Principle:** Sensors translate reality (kernel) to perspective (contracts). Sensors are the "lens" through which creatures perceive.

---

### SimulationState (Coordination Shell)

**Purpose:** Bundle all mutable simulation state into a single owner. Thin coordination struct — no business logic. Lives at crate root (`lib.rs`).

```rust
// lib.rs
use slotmap::SlotMap;

pub struct SimulationState {
    pub creatures: SlotMap<CreatureId, CreatureState>,
    pub world: WorldState,
    pub tick_number: u64,
}

impl SimulationState {
    pub fn new(world: WorldState) -> Self {
        Self {
            creatures: SlotMap::with_key(),
            world,
            tick_number: 0,
        }
    }

    /// Insert a creature and register it in the spatial index.
    pub fn spawn_creature(&mut self, creature: CreatureState) -> CreatureId {
        let pos = creature.position;
        let id = self.creatures.insert(creature);
        self.world.place_creature(pos, id);
        id
    }

    pub fn tick(&mut self, config: &SimulationConfig, rng: &mut impl Rng) -> TickStats {
        tick::orchestrator::tick(self, config, rng)
    }
}
```

**Why a struct:** The tick function needs mutable access to creatures, world, and tick counter. Bundling them eliminates a multi-parameter function and makes it impossible for callers to forget state. Server and CLI each own a `SimulationState` and call `state.tick(config, rng)`.

**Not a god object:** `SimulationState` has no business logic. Modules (kernel, creature, tick, runtime, sensors) own behavior. `SimulationState` is just "the mutable things the simulation needs."

**Why pub fields (borrow splitting):** The tick orchestrator needs simultaneous mutable borrows of different fields — e.g. `&mut state.creatures` and `&mut state.world` in the same function. Rust's borrow checker allows this through direct field access (`state.creatures` and `state.world` are disjoint borrows) but NOT through accessor methods (which take `&mut self`, borrowing the whole struct). This is a structural requirement, not convenience.

---

### Seed (Initial Creature Placement)

**Purpose:** Populate a SimulationState with identical seed creatures from a named founder genome. Cross-cutting placement logic used by server, CLI, and tests. Thin orchestrator — delegates genome knowledge to `creature/founders.rs` and creature construction to `CreatureState::new_founder()`.

**File:** `seed.rs` (crate root)

```rust
// seed.rs

/// Seed `count` identical creatures from the named founder genome
/// at random empty positions. Best-effort: if the world is too full,
/// fewer creatures are placed.
pub fn seed_creatures(
    state: &mut SimulationState,
    founder_name: &str,
    count: usize,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) {
    let genome = founders::get(founder_name);
    for _ in 0..count {
        if let Some(pos) = find_empty_position(&state.world, rng) {
            let creature = CreatureState::new_founder(pos, genome.clone(), config);
            state.spawn_creature(creature);
        }
    }
}

fn find_empty_position(world: &WorldState, rng: &mut impl Rng) -> Option<Position> {
    // Random probe with bounded retries — O(1) expected for sparse worlds.
    // Falls back to None if world is full (callers handle best-effort).
    let max_attempts = 100;
    for _ in 0..max_attempts {
        let pos = Position {
            x: rng.gen_range(0..world.width),
            y: rng.gen_range(0..world.height),
        };
        if !world.is_occupied(pos) && !world.is_barrier(pos) {
            return Some(pos);
        }
    }
    None
}
```

**Separation of concerns:**
- `creature/founders.rs` — owns genome definitions (the "what"). Knows about node types, VM instructions, graph operators. Updated when creature features are added.
- `CreatureState::new_founder()` — owns creature construction from a genome. Applies fixed phenotype baseline `[204, 61, 61]`, zeroes memory, empties inventory. No `rng` parameter — seed creatures are deterministically identical except for position.
- `seed.rs` — owns placement (the "where"). Finds empty positions, clones genomes, spawns into SimulationState. Does not know genome internals.

**Why a separate module:** Seeding touches world (find empty positions), creature (create state), and SimulationState (register in spatial index). It doesn't belong inside any single module. Server, CLI, and viability tests all call the same function — only the founder name and count differ.

---

### Tick (Orchestration)

**Purpose:** Integrate all capabilities to execute world ticks. Enforces phase-based execution for fairness.

**Files:**
- `tick/orchestrator.rs` - Main tick loop with phases
- `tick/actions.rs` - Action validation and execution

**Phase-based tick:**
```rust
// tick/orchestrator.rs

/// Per-tick observability counters (GP-04).
pub struct TickStats {
    pub births: u32,
    pub deaths: u32,
    pub actions_attempted: u32,
    pub actions_succeeded: u32,
}

pub fn tick(
    state: &mut SimulationState,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> TickStats {
    // Phase 0: World mechanics
    state.world.grow_food(&config.world.food, rng);
    apply_energy_decay(&mut state.creatures, config.energy.lifecycle.energy_decay_per_tick);

    // Phase 1: Cognition — all creatures think, actions collected.
    // Collect (CreatureId, WorldAction) pairs — SlotMap keys are stable handles.
    let mut action_queue: Vec<(CreatureId, WorldAction)> = Vec::new();
    for (id, creature) in state.creatures.iter_mut() {
        let inputs = sensors::gather_inputs(creature, &state.world, config);
        let outputs = runtime::execute_creature(
            &creature.genome,
            inputs,
            &mut creature.memory,
            &mut creature.energy,
            config,
        );
        action_queue.push((id, outputs.world_action));
    }

    // Phase 2: Action execution — randomize and execute.
    // INVARIANT: SlotMap keys remain valid during this phase because
    // (a) no creatures are removed (cleanup is Phase 3), and
    // (b) offspring are deferred to a separate spawn queue.
    action_queue.shuffle(rng);
    let mut spawns: Vec<CreatureState> = Vec::new();
    let mut actions_attempted: u32 = 0;
    let mut actions_succeeded: u32 = 0;

    for (id, action) in &action_queue {
        let creature = &mut state.creatures[*id];
        if !creature.energy.is_alive() {
            continue;  // Creature died during action phase
        }

        actions_attempted += 1;
        // Actions apply energy changes directly (drain/charge) on creature.
        // ActionResult.status is informational for stats tracking.
        let result = actions::execute_action(
            action, creature, &mut state.world, config, rng,
        );

        if matches!(result.status, ActionStatus::Success) {
            actions_succeeded += 1;
        }
        if let Some(offspring) = result.spawn {
            spawns.push(offspring);
        }
    }

    let births = spawns.len() as u32;

    // Append deferred offspring — SlotMap assigns IDs on insert
    for offspring in spawns {
        let pos = offspring.position;
        let id = state.creatures.insert(offspring);
        state.world.place_creature(pos, id);
    }

    // Phase 3: Cleanup — remove dead creatures and update spatial index
    let dead_ids: Vec<CreatureId> = state.creatures.iter()
        .filter(|(_, c)| !c.energy.is_alive())
        .map(|(id, _)| id)
        .collect();
    for id in &dead_ids {
        if let Some(creature) = state.creatures.remove(*id) {
            state.world.remove_creature(creature.position);
        }
    }
    let deaths = dead_ids.len() as u32;

    state.tick_number += 1;

    TickStats { births, deaths, actions_attempted, actions_succeeded }
}
```

**Why phases:**
1. **All creatures see same world during cognition** - no information advantage
2. **Randomized action execution** - solves turn order bias
3. **Separates thinking from acting** - cleaner conceptually

**Action execution:**
```rust
// tick/actions.rs
pub enum ActionStatus {
    Success,
    Failed(FailureReason),
}

pub enum FailureReason {
    InsufficientEnergy,
    TargetOccupied,
    TargetOutOfBounds,
    NoFood,
    NoSpawnLocation,
}

/// Actions apply energy changes (drain/charge) directly on the creature
/// during execution. ActionResult reports what happened for stats/logging
/// purposes only — the tick orchestrator does NOT re-apply energy changes.
pub struct ActionResult {
    pub status: ActionStatus,
    pub spawn: Option<CreatureState>,
}

pub fn execute_action(
    action: &WorldAction,
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> ActionResult {
    match action {
        WorldAction::Eat => execute_eat(creature, world, config),
        WorldAction::Move { direction } => execute_move(creature, world, *direction, config),
        WorldAction::Reproduce { direction, energy_amount } => execute_reproduce(creature, world, *direction, *energy_amount, config, rng),
        WorldAction::PickupFood { direction, slot } => execute_pickup_food(creature, world, *direction, *slot, config),
        WorldAction::PickupBarrier { direction, slot } => execute_pickup_barrier(creature, world, *direction, *slot, config),
        WorldAction::PlaceFood { direction, slot } => execute_place_food(creature, world, *direction, *slot, config),
        WorldAction::PlaceBarrier { direction, slot } => execute_place_barrier(creature, world, *direction, *slot, config),
        WorldAction::NoOp => ActionResult::success(),
    }
}

fn execute_eat(...) -> ActionResult {
    // 1. Validate: check food exists via kernel.get_food_density()
    // 2. Execute: consume via kernel.consume_food()
    // 3. Apply: gain energy via creature.energy.charge()
    // 4. Return: ActionResult with status and costs
}

fn execute_reproduce(...) -> ActionResult {
    // 1. Validate: check energy sufficient
    // 2. Find spawn location: find_empty_neighbor()
    // 3. If no location: fail with NoSpawnLocation, still drain energy
    // 4. Create offspring: creature::create_offspring()
    // 5. Drain parent energy
    // 6. Return: ActionResult with offspring
}
```

**Failed action policy:** All world actions cost energy regardless of success or failure. If an action fails validation (e.g., pickup into a full inventory slot, place from an empty slot, place into an occupied or out-of-bounds cell, move into a barrier), the energy cost is still deducted but the world state is unchanged. Failed actions do **not** fall back to NoOp. The creature's turn is consumed.

**Principle:** Tick orchestrates but delegates. Kernel provides world modification methods, creature provides offspring creation, tick coordinates timing and fairness.

---

## Data Flow (End-to-End Example)

```
Server owns SimulationState, calls state.tick(config, rng)
  → v3_core::tick::orchestrator::tick(&mut state, config, rng)

Phase 0: World Mechanics
  state.world.grow_food(config, rng)
  creature::Energy.drain() for all (decay)

Phase 1: Cognition (collect keyed actions)
  For each (id, creature) in state.creatures (SlotMap iteration):
    sensors::gather_inputs()
      ├─ sensors::environmental() → state.world methods
      └─ sensors::introspection() → creature.energy, creature.phenotype

    runtime::executor::execute_creature()
      └─ runtime::vm::execute_vm_node() OR runtime::graph::execute_graph_node()

    Add (id, outputs.world_action) to action_queue

Phase 2: Action Execution (randomized, deferred spawns)
  Shuffle action_queue
  [SlotMap keys stable — no removals, spawns deferred]
  For each (id, action):
    tick::actions::execute_action()
      ├─ Validate via state.world methods
      ├─ Execute via state.world methods (consume_food, place_creature)
      ├─ Apply energy directly on creature (drain/charge)
      ├─ Or create offspring via creature::create_offspring()
      └─ Return ActionResult (status + optional offspring)

    Track stats (attempted/succeeded/births)
    If reproduce succeeded: spawns.push(offspring)

  Insert spawns into state.creatures (SlotMap assigns IDs)
  Update world.place_creature() for each spawn

Phase 3: Cleanup
  Collect dead CreatureIds, remove from SlotMap + world.remove_creature()
  Count deaths

state.tick_number += 1
Return TickStats { births, deaths, actions_attempted, actions_succeeded }
→ Server stores/exposes stats for transport layer
```

**Key observation:** Data flows through clean module boundaries. No module directly manipulates another module's internal state. Actions own energy mutation; tick owns orchestration and stats. SimulationState is the coordination shell, not a behavior owner. SlotMap provides stable creature handles without manual ID allocation.

---

## Testing Strategy

### The v2 Problem

v2 tests passed but nothing worked because:
- Tests only validated contracts (type shapes), not intent (behavior)
- Mocks everywhere prevented integration testing
- No end-to-end verification

### The v3 Solution

**Three layers of testing:**

#### 1. Contract Tests (Type/Shape validation)
```rust
// v3-core/tests/contracts_test.rs
#[test]
fn environmental_inputs_has_required_fields() {
    let inputs = EnvironmentalInputs::default();
    assert_eq!(inputs.neighbor_n, false);  // Field exists
}
```
**Purpose:** Validate struct shapes match contracts.

#### 2. Intent Verification Tests (Functional behavior)
```rust
// v3-core/tests/eat_action_intent.rs
#[test]
fn eat_action_removes_food_and_increases_energy() {
    // Setup
    let mut world = WorldState::new(10, 10, true);
    world.set_food_density(Position { x: 5, y: 5 }, 200);
    let mut creature = CreatureState::new(...);
    creature.position = Position { x: 5, y: 5 };
    creature.energy = Energy::new(10);

    // Execute
    let result = execute_action(&WorldAction::Eat, &mut creature, &mut world, &config, &mut rng);

    // Verify INTENT: food gone, energy increased
    assert_eq!(world.get_food_density(Position { x: 5, y: 5 }), 0);
    assert!(creature.energy.value() > 10);
    assert!(matches!(result.status, ActionStatus::Success));
}
```
**Purpose:** Validate functional behavior, not just type contracts.

#### 3. Integration Tests (End-to-end)
```rust
// v3-core/tests/tick_integration.rs
#[test]
fn full_tick_produces_valid_stats() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    // ... seed::seed_creatures(&mut state, ...), add food, config ...

    let stats = state.tick(&config, &mut rng);

    assert_eq!(state.tick_number, 1);
    assert!(!state.creatures.is_empty());
    // Verify stats reflect actual behavior
    assert_eq!(stats.deaths, 0);
}
```
**Purpose:** Validate full-stack integration.

### Testing Principles

1. **Both contracts AND intent** - validate shape and behavior
2. **Integration tests required** - each feature needs end-to-end test
3. **No mocks in core** - v3-core tests use real implementations
4. **Mocks only at boundaries** - mock HTTP/WebSocket for server tests
5. **Golden rule:** Completion requires verified functional intent and integration behavior

---

## Walking Skeleton Strategy

Build v3 incrementally in 3 stages. Each stage delivers a qualitatively different simulation. The seed creature viability test (introduced in Stage 2) is the primary forcing function — it won't pass unless the whole ecology works together end-to-end.

### Stage 1: Foundation + Working Tick
**Goal:** Creatures exist, tick advances, server responds to REST queries.

**Scope:**
- kernel/ - Position, Direction (with delta), CreatureId (slotmap), WorldState (grid storage, creature_at spatial index)
- creature/ - CreatureState, Energy type (u32) with safe operations
- contracts/ - stub inputs/outputs (NoOp only)
- runtime/ - stub executor (returns NoOp)
- sensors/ - stub inputs (zeros)
- tick/ - Phase 0 only (age increment, dead creature cleanup, no cognition)
- seed - seed_creatures() for initial placement (shared by server, CLI, tests)
- v3-server/ - start/status/pause REST endpoints, tick loop
- SimulationState at crate root (SlotMap creatures, WorldState, tick counter)

**Gate:** Server starts, tick advances, creatures persist across ticks.
**Note:** Frontend (WebSocket, web client) deferred to separate frontend architecture plan.
**Plan:** `docs/plans/2026-02-14-v3-phase1-walking-skeleton.md`

---

### Stage 2: Viable Ecology
**Goal:** Creatures survive by eating, die without food, move toward resources. The simulation is alive.

**Introduces:** config/ module, food growth, energy decay, eat/move actions, heuristic brain

**Scope:**
- config/ - FoodConfig, EnergyConfig (lifecycle + costs), SimulationConfig top-level
- kernel/ - food grid operations: set_food_density(), consume_food(), grow_food()
- contracts/ - food_density_self input, Eat/Move action variants
- sensors/ - sense food at current position and neighbors
- tick/actions/ - execute_eat(), execute_move()
- tick/orchestrator - full Phase 0-3 with cognition and action execution
- runtime/ - simple heuristic brain (eat if food present, move toward food, else random)
- config threading through tick function signature

**Gate:** Seed creature viability test passes (see Viability Test Design below).
**Plan:** Separate plan document (created when Stage 1 completes).

---

### Stage 3: Evolving Ecology
**Goal:** Population grows through reproduction, offspring vary through mutation. Creatures make decisions via VM brain.

**Introduces:** reproduction action, genome mutation, VM brain

**Scope:**
- contracts/ - Reproduce action variant
- creature/reproduction/ - create_offspring()
- creature/mutation/ - mutate_genome()
- creature/genome/ - minimal VM genome
- runtime/vm/ - basic VM execution (replacing heuristic brain)
- runtime/executor/ - energy-bound think loop
- tick/actions/ - execute_reproduce()

**Gate:** Extended viability test — population sustains via reproduction, offspring differ from parents.
**Plan:** Separate plan document (created when Stage 2 completes).

---

### Stage 4+: Expand Complexity
- Graph nodes
- Rich sensors (8-directional, neighbor detection)
- Inventory system (pickup/place with slot-addressed storage; see genome-sensor spec)
- Phenotype evolution
- Advanced mutation operators

**Each stage maintains a working system. Each new creature capability adds assertions to the viability test suite.**

---

## Telemetry

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

The window is trailing `health_window_ticks` (default `100`), right-aligned at the current tick.

---

## Non-Collapse Contract

The non-collapse contract is the minimum bar for a working simulation.

- Founder genome: `test`.
- Deterministic seed: `42`.
- Duration: `2000` ticks.
- Config: default.
- `baseline_population` = actual seeded population at tick 0.

### Pass Criteria

1. No crash.
2. At least one birth.
3. Mean population in the final 400 ticks >= `ceil(baseline_population * 0.10)`.

---

## Viability Gate

Reusable helper for validating simulation viability in tests.

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

## Viability Test Design

The seed creature viability test is the primary integration gate from Stage 2 onward. It verifies that the ecology works end-to-end — not just that individual modules pass contract tests. This is the "intent verification" that AGENTS.md requires.

### Speed Requirement

**Target: < 2 seconds for the core viability test.** In v1, the viability test became a development bottleneck because it ran at production-scale configs. v3 avoids this by using a deliberately small, fast test config separate from the default server config:

- Small world: 32x32 (1,024 cells) — not 400x400
- Few creatures: 10-20 — not hundreds
- Short run: ~100 ticks — enough to prove viability, not simulate evolution
- Tuned config: high food density, moderate decay — fast convergence to viability signal

The viability test proves "the ecology mechanics work," not "the default config is interesting at scale."

### Two Levels

1. **Core viability test** (`v3-core/tests/seed_viability.rs`): Pure SimulationState, no server overhead. This is the fast gate that runs on every change.
2. **Server viability test** (`v3-server/tests/seed_viability.rs`): Exercises HTTP API and tick loop using Axum test utilities (no real port binding). Slightly slower but validates transport integration.

### Assertions (grow with each stage)

**Stage 2 (viable ecology):**
- Population > 0 after N ticks (creatures didn't all starve)
- At least some creatures have eaten (energy economy works)
- Food has been consumed and regrown (food cycle works)
- Creature positions have changed (movement works)
- TickStats counters are non-zero and consistent (GP-04 observability)

**Stage 3 adds:**
- Population has grown via reproduction (births > 0)
- Offspring positions differ from parents (spawning works)
- Offspring genomes differ from parents (mutation works)

**Stage 4+ adds:** assertions for each new capability.

### Test Config

The viability test uses a shared `viability_config()` function — not the default server config. This config is tuned for fast, reliable viability signal and lives in a test utility module.

---

## Verification Commands

Each stage must pass:

1. `cd v3 && cargo fmt --all --check`
2. `cd v3 && cargo test --workspace`
3. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
4. Manual: Start server, verify stage goals via curl/REST

---

## Risks and Mitigation

**Risk:** Module boundaries become porous during implementation
**Mitigation:** Code reviews focus on boundary violations. Use `pub(crate)` aggressively. Gemini reviews.

**Risk:** Walking skeleton stages grow too large
**Mitigation:** Each stage has single, testable goal. If stage feels large, split it.

**Risk:** Performance issues with phase-based tick
**Mitigation:** Profile early. If Phase 1 (collect actions) + Phase 2 (execute) is too slow, optimize or reconsider two-phase.

**Risk:** Contract/intent testing discipline slips
**Mitigation:** CI requires both test types. Code review checklist includes intent verification.

---

## Success Criteria

v3 is successful when:

1. **Feature parity with v1** - food, barriers, movement, eating, reproduction work
2. **Mesh architecture works** - creatures with VM+Graph nodes execute correctly
3. **Clean boundaries** - no leaky abstractions, modules testable in isolation
4. **Walking skeleton proven** - each stage delivered working vertical slice
5. **Tests verify intent** - not just contracts, actual behavior validated

When these criteria are met, v3 can replace v1 as the primary implementation.

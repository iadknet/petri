# Petri V3 Architecture Design

**Goal:** Design a clean, maintainable architecture for v3 that achieves the v2 vision (complex mesh creatures with VM+Graph nodes) while avoiding v2's leaky abstraction failures.

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** Complete v3 architecture including module boundaries, data flow, testing strategy, and walking skeleton phases. Includes core simulation (v3-core), transport layer (v3-server), and web client (v3/web). Excludes specific VM instruction set details and graph operator implementations (deferred to implementation).

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

---

## Boundary Impact

- New `v3/` root directory owns all v3 implementation
- v1 (`crates/petri-*`, `web/`) remains working reference
- v2 (`v2/`) remains as reference for architectural failures
- Independent toolchain manifests in `v3/` (no coupling to v1 or v2)
- Clear dependency direction enforced by module structure

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| v1 `crates/petri-*` | keep | Working reference for proven patterns (world/food, world/tick modular structure) |
| v2 `v2/` directory | keep | Reference for what NOT to do (leaky abstractions, poor integration) |
| v3 `v3/` directory (new) | change | Clean slate required to avoid v2's architectural mistakes |
| `docs/strategy/architecture.md` | keep for now | Will be updated when v3 becomes primary |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should v3 use randomized turn order or two-phase commit? | Randomized turn order (simpler, solves bias, can upgrade later). | user+agent | resolved |
| Where do input/output contracts live? | Dedicated `contracts/` module (interface segregation, modules as peers). | user+agent | resolved |
| Where does energy logic live? | Energy type in `creature/`, energy config in `config/`, operations distributed. | user+agent | resolved |
| Should reproduction spawn immediately or defer? | Collect during action execution, add to creatures list in same phase (safe with randomized indices). | user+agent | resolved |
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
│   │   │   │   ├── creature/   # Genome, phenotype, mutation, state
│   │   │   │   ├── runtime/    # VM + Graph execution
│   │   │   │   ├── sensors/    # Input assembly
│   │   │   │   └── tick/       # Tick orchestration
│   │   │   └── tests/          # Integration tests
│   │   ├── v3-server/          # HTTP + WebSocket transport
│   │   └── v3-cli/             # Headless runner
│   ├── web/                    # React + TypeScript client
│   └── docs/
│       ├── BOUNDARIES.md       # Module boundary rules
│       └── WALKING_SKELETON.md # Incremental build phases
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

v3 workspace:
  v3-server → v3-core
  v3-cli → v3-core
  v3/web → v3-server (protocol only)
```

**Key insight:** contracts/ is the "clipboard" modules share. Kernel owns primitives, capabilities transform and consume via contracts.

---

## Module Design

### Kernel (Foundational Primitives)

**Purpose:** World reality - food, barriers, positions, occupancy. No business logic.

**Files:**
- `kernel/types.rs` - Position, Direction, CreatureId, WorldCell
- `kernel/world_state.rs` - Food grid, barriers, occupancy tracking

**Key methods:**
```rust
// kernel/world_state.rs
pub fn get_food_density(&self, pos: WorldCell) -> u8;
pub fn consume_food(&mut self, pos: WorldCell) -> u8;
pub fn is_barrier(&self, pos: WorldCell) -> bool;
pub fn is_occupied(&self, pos: WorldCell) -> bool;
pub fn mark_occupied(&mut self, pos: WorldCell);
pub fn mark_unoccupied(&mut self, pos: WorldCell);
pub fn grow_food(&mut self, tick: u64, seed: u64, config: &FoodConfig);
```

**Principle:** Kernel has NO knowledge of creatures, actions, VM, or ticks. Pure world primitives.

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
    Reproduce { direction: Direction },
    PickupFood { direction: Direction },
    PickupBarrier { direction: Direction },
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
- `creature/state.rs` - CreatureState, Energy type
- `creature/genome.rs` - CreatureGenome, NodeGenome, validation
- `creature/phenotype.rs` - Phenotype, color evolution
- `creature/mutation.rs` - mutate_genome()
- `creature/reproduction.rs` - create_offspring()

**Energy design:**
```rust
// creature/state.rs
pub struct Energy(f32);  // Newtype ensures non-negative

impl Energy {
    pub fn drain(&mut self, amount: f32) -> bool;  // Returns success
    pub fn charge(&mut self, amount: f32, max: f32);
    pub fn is_alive(&self) -> bool;
}
```

Energy is a creature property. Config defines energy policy (costs, gains). Runtime and tick modify energy through the Energy type's safe operations.

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
    let step_cost = config.energy.costs.energy_per_think_step;

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
    EnvironmentalInputs {
        food_density_self: world.get_food_density(position),
        neighbor_n: world.is_occupied(calculate_neighbor(position, Direction::N, config)),
        barrier_n: world.is_barrier(calculate_neighbor(position, Direction::N, config)),
        // ... assemble all fields
    }
}
```

**Principle:** Sensors translate reality (kernel) to perspective (contracts). Sensors are the "lens" through which creatures perceive.

---

### Tick (Orchestration)

**Purpose:** Integrate all capabilities to execute world ticks. Enforces phase-based execution for fairness.

**Files:**
- `tick/orchestrator.rs` - Main tick loop with phases
- `tick/actions.rs` - Action validation and execution

**Phase-based tick:**
```rust
// tick/orchestrator.rs
pub fn tick(
    creatures: &mut Vec<CreatureState>,
    world: &mut WorldState,
    config: &SimulationConfig,
    tick_number: u64,
    rng: &mut impl Rng,
) {
    // Phase 0: World mechanics
    world.grow_food(tick_number, config.world.food, rng);
    apply_energy_decay(creatures, config.energy.lifecycle.energy_decay_per_tick);

    // Phase 1: Cognition - all creatures think, actions collected
    let mut action_queue = Vec::new();
    for creature in creatures.iter_mut() {
        let inputs = sensors::gather_inputs(creature, world, config);
        let outputs = runtime::execute_creature(
            &creature.genome,
            inputs,
            &mut creature.memory,
            &mut creature.energy,
            config,
        );
        action_queue.push((creature.id, outputs.world_action));
    }

    // Phase 2: Action execution - randomize and execute
    action_queue.shuffle(rng);
    for (creature_id, action) in action_queue {
        let Some(creature) = creatures.iter_mut().find(|c| c.id == creature_id) else {
            continue;  // Creature died during action phase
        };

        let result = actions::execute_action(&action, creature, world, config, rng);

        // Spawn offspring immediately if reproduce succeeded
        if let Some(offspring) = result.spawn {
            creatures.push(offspring);
        }
    }

    // Phase 3: Cleanup - remove dead creatures
    creatures.retain(|c| c.energy.is_alive());
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

pub struct ActionResult {
    pub status: ActionStatus,
    pub energy_cost: f32,
    pub energy_gain: f32,
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
        WorldAction::Reproduce { direction } => execute_reproduce(creature, world, *direction, config, rng),
        WorldAction::NoOp => ActionResult::success(),
        // ...
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

**Principle:** Tick orchestrates but delegates. Kernel provides world modification methods, creature provides offspring creation, tick coordinates timing and fairness.

---

## Data Flow (End-to-End Example)

```
Server tick loop → v3_core::tick::orchestrator::tick()

Phase 0: World Mechanics
  kernel::world_state.grow_food()
  creature::Energy.drain() for all (decay)

Phase 1: Cognition (collect actions)
  For each creature:
    sensors::gather_inputs()
      ├─ sensors::environmental() → kernel::world_state methods
      └─ sensors::introspection() → creature.energy, creature.phenotype

    runtime::executor::execute_creature()
      └─ runtime::vm::execute_vm_node() OR runtime::graph::execute_graph_node()

    Add (creature.id, outputs.world_action) to action_queue

Phase 2: Action Execution (randomized)
  Shuffle action_queue
  For each (creature_id, action):
    tick::actions::execute_action()
      ├─ Validate via kernel methods
      ├─ Execute via kernel methods (world.consume_food(), world.mark_occupied())
      ├─ Or create offspring via creature::create_offspring()
      └─ Modify creature.energy

    If reproduce succeeded: creatures.push(offspring)

Phase 3: Cleanup
  creatures.retain(|c| c.energy.is_alive())

Server serializes WorldFrame → WebSocket → Web client renders
```

**Key observation:** Data flows through clean module boundaries. No module directly manipulates another module's internal state.

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
    let mut world = WorldState::new(10, 10);
    world.set_food(Position { x: 5, y: 5 }, 200);
    let mut creature = CreatureState::new(...);
    creature.position = Position { x: 5, y: 5 };
    creature.energy = Energy::new(10.0);

    // Execute
    let result = execute_action(&WorldAction::Eat, &mut creature, &mut world, &config, &mut rng);

    // Verify INTENT: food gone, energy increased
    assert_eq!(world.get_food_density(Position { x: 5, y: 5 }), 0);
    assert!(creature.energy.value() > 10.0);
    assert!(matches!(result.status, ActionStatus::Success));
}
```
**Purpose:** Validate functional behavior, not just type contracts.

#### 3. Integration Tests (End-to-end)
```rust
// v3-server/tests/tick_integration.rs
#[test]
fn full_tick_produces_valid_world_frame() {
    let mut state = ServerState::new();
    state.tick();

    assert_eq!(state.tick, 1);
    assert!(state.creatures.len() > 0);
    // Verify world changed (food grew, creatures acted)
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

Build v3 incrementally with always-working vertical slices:

### Phase 1: Minimal Walking Skeleton
**Goal:** Creatures exist and can be rendered

**Scope:**
- kernel/ - positions only (no food yet)
- creature/ - minimal state (id, position, energy)
- contracts/ - stub inputs/outputs (NoOp only)
- runtime/ - stub executor (returns NoOp)
- sensors/ - stub inputs (zeros)
- tick/ - Phase 0 only (no cognition)
- v3-server/ - start/status endpoints, WebSocket streaming
- v3/web/ - canvas rendering colored dots

**Verification:** Creatures render, tick advances

---

### Phase 2: Add Food and Eat Action
**Goal:** Creatures can eat and gain energy

**Scope:**
- kernel/ - food grid, grow_food(), consume_food()
- contracts/ - add food_density_self, Eat action
- sensors/ - sense food at position
- tick/actions/ - execute_eat()
- v3/web/ - render food grid

**Verification:** Creatures eat, energy increases, food disappears

---

### Phase 3: Add Movement
**Goal:** Creatures can move around

**Scope:**
- kernel/ - occupancy tracking
- contracts/ - Move action
- tick/actions/ - execute_move()
- v3/web/ - animate movement

**Verification:** Creatures move, don't overlap

---

### Phase 4: Add Energy Decay and Death
**Goal:** Creatures must eat or die

**Scope:**
- config/energy/ - decay_per_tick
- tick/ - apply decay, remove dead
- creature/ - Energy::is_alive()

**Verification:** Creatures without food die

---

### Phase 5: Add Simple VM Brain
**Goal:** Creatures make decisions

**Scope:**
- creature/genome/ - minimal VM genome
- runtime/vm/ - basic VM execution
- runtime/executor/ - energy-bound think loop
- tick/ - Phase 1 cognition, Phase 2 execution

**Verification:** Different genomes produce different behaviors

---

### Phase 6: Add Reproduction
**Goal:** Population grows and evolves

**Scope:**
- contracts/ - Reproduce action
- creature/reproduction/ - create_offspring()
- creature/mutation/ - mutate_genome()
- tick/actions/ - execute_reproduce()

**Verification:** Population grows, offspring differ from parents

---

### Phase 7+: Expand Complexity
- Graph nodes
- Rich sensors (8-directional, neighbor detection)
- Inventory system (pickup/put)
- Phenotype evolution
- Advanced mutation operators

**Each phase maintains a working system.**

---

## Verification Commands

Each phase must pass:

1. `cd v3 && cargo fmt --all --check`
2. `cd v3 && cargo test --workspace`
3. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
4. `cd v3/web && npm run build`
5. Manual: Start server, open web UI, verify phase goals work

---

## Risks and Mitigation

**Risk:** Module boundaries become porous during implementation
**Mitigation:** Code reviews focus on boundary violations. Use `pub(crate)` aggressively. Gemini reviews.

**Risk:** Walking skeleton phases grow too large
**Mitigation:** Each phase has single, testable goal. If phase feels large, split it.

**Risk:** Performance issues with phase-based tick
**Mitigation:** Profile early. If Phase 1 (collect actions) + Phase 2 (execute) is too slow, optimize or reconsider two-phase.

**Risk:** Randomized turn order breaks deterministic replay
**Mitigation:** Seed the shuffle RNG with tick number + global seed. Runs remain reproducible.

**Risk:** Contract/intent testing discipline slips
**Mitigation:** CI requires both test types. Code review checklist includes intent verification.

---

## Success Criteria

v3 is successful when:

1. **Feature parity with v1** - food, barriers, movement, eating, reproduction work
2. **Mesh architecture works** - creatures with VM+Graph nodes execute correctly
3. **Clean boundaries** - no leaky abstractions, modules testable in isolation
4. **Walking skeleton proven** - each phase delivered working vertical slice
5. **Tests verify intent** - not just contracts, actual behavior validated
6. **Web UI functional** - creatures render, controls work, simulation observable

When these criteria are met, v3 can replace v1 as the primary implementation.

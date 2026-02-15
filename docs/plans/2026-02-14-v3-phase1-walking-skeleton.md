# V3 Phase 1: Minimal Walking Skeleton Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build minimal end-to-end vertical slice where creatures exist, tick advances, and creatures render in web UI.

**Architecture:** Phase-based tick (Phase 0: world mechanics only, no cognition yet), kernel owns world primitives, creature owns state, contracts define interfaces, stub runtime/sensors return NoOp, server streams frames via WebSocket, web client renders colored dots.

**Tech Stack:** Rust 1.93.0, Axum (server), Tokio (async), React + TypeScript + Vite (web), MessagePack (serialization)

---

## Task 1: Create v3 Workspace Structure

**Files:**
- Create: `v3/Cargo.toml`
- Create: `v3/rust-toolchain.toml`
- Create: `v3/README.md`
- Create: `v3/crates/v3-core/Cargo.toml`
- Create: `v3/crates/v3-core/src/lib.rs`

**Step 1: Create v3 root directory and workspace manifest**

```bash
mkdir -p v3/crates
```

Create `v3/Cargo.toml`:
```toml
[workspace]
members = [
    "crates/v3-core",
]
resolver = "2"
```

**Step 2: Pin Rust toolchain**

Create `v3/rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.93.0"
components = ["rustfmt", "clippy"]
```

**Step 3: Create README**

Create `v3/README.md`:
```markdown
# Petri V3

Clean architecture rewrite with mesh creatures (VM + Graph nodes).

## Build

```bash
cd v3
cargo build
```

## Test

```bash
cargo test --workspace
```

## Run

```bash
cargo run -p v3-server
```
```

**Step 4: Create v3-core crate**

```bash
mkdir -p v3/crates/v3-core/src
```

Create `v3/crates/v3-core/Cargo.toml`:
```toml
[package]
name = "v3-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

Create `v3/crates/v3-core/src/lib.rs`:
```rust
// Placeholder - modules will be added incrementally
```

**Step 5: Verify workspace builds**

Run: `cd v3 && cargo check`
Expected: Success (empty workspace compiles)

**Step 6: Commit**

```bash
git add v3/
git commit -m "feat(v3): initialize workspace structure

- Create v3/ workspace root
- Add v3-core crate skeleton
- Pin Rust 1.93.0 toolchain

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Implement Kernel Module (World Primitives)

**Files:**
- Create: `v3/crates/v3-core/src/kernel/mod.rs`
- Create: `v3/crates/v3-core/src/kernel/types.rs`
- Create: `v3/crates/v3-core/src/kernel/world_state.rs`
- Create: `v3/crates/v3-core/tests/kernel_world_state_test.rs`

**Step 1: Write failing test for Position type**

Create `v3/crates/v3-core/tests/kernel_world_state_test.rs`:
```rust
#[test]
fn position_equality_works() {
    use v3_core::kernel::types::Position;

    let pos1 = Position { x: 5, y: 10 };
    let pos2 = Position { x: 5, y: 10 };
    let pos3 = Position { x: 6, y: 10 };

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test kernel_world_state_test`
Expected: FAIL with "use of undeclared crate or module `v3_core::kernel`"

**Step 3: Implement kernel types**

Create `v3/crates/v3-core/src/kernel/mod.rs`:
```rust
pub mod types;
pub mod world_state;
```

Create `v3/crates/v3-core/src/kernel/types.rs`:
```rust
/// Position in world grid
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Direction (cardinal and intercardinal)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

/// Unique creature identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CreatureId(pub u64);

/// World cell (alias for Position, semantic clarity)
pub type WorldCell = Position;
```

Update `v3/crates/v3-core/src/lib.rs`:
```rust
pub mod kernel;
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test kernel_world_state_test::position_equality_works`
Expected: PASS

**Step 5: Write failing test for WorldState creation**

Add to `v3/crates/v3-core/tests/kernel_world_state_test.rs`:
```rust
#[test]
fn world_state_new_creates_empty_world() {
    use v3_core::kernel::world_state::WorldState;
    use v3_core::kernel::types::Position;

    let world = WorldState::new(10, 10, true);

    assert_eq!(world.width, 10);
    assert_eq!(world.height, 10);
    assert_eq!(world.wrap, true);
    assert_eq!(world.get_food_density(Position { x: 5, y: 5 }), 0);
    assert_eq!(world.is_occupied(Position { x: 5, y: 5 }), false);
}
```

**Step 6: Run test to verify it fails**

Run: `cd v3 && cargo test world_state_new_creates_empty_world`
Expected: FAIL with "no method named `new`"

**Step 7: Implement WorldState**

Create `v3/crates/v3-core/src/kernel/world_state.rs`:
```rust
use std::collections::{HashMap, HashSet};
use super::types::{Position, WorldCell};

pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,

    // Food density per cell (0-255 quantized)
    food: HashMap<WorldCell, u8>,

    // Barrier locations
    barriers: HashSet<WorldCell>,

    // Occupied cells (creature positions)
    occupied: HashSet<WorldCell>,
}

impl WorldState {
    pub fn new(width: u16, height: u16, wrap: bool) -> Self {
        Self {
            width,
            height,
            wrap,
            food: HashMap::new(),
            barriers: HashSet::new(),
            occupied: HashSet::new(),
        }
    }

    pub fn get_food_density(&self, pos: WorldCell) -> u8 {
        self.food.get(&pos).copied().unwrap_or(0)
    }

    pub fn is_occupied(&self, pos: WorldCell) -> bool {
        self.occupied.contains(&pos)
    }

    pub fn is_barrier(&self, pos: WorldCell) -> bool {
        self.barriers.contains(&pos)
    }

    pub fn mark_occupied(&mut self, pos: WorldCell) {
        self.occupied.insert(pos);
    }

    pub fn mark_unoccupied(&mut self, pos: WorldCell) {
        self.occupied.remove(&pos);
    }
}
```

**Step 8: Run tests to verify they pass**

Run: `cd v3 && cargo test kernel_world_state_test`
Expected: All tests PASS

**Step 9: Commit**

```bash
git add v3/crates/v3-core/src/kernel/ v3/crates/v3-core/tests/kernel_world_state_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): implement kernel module with world primitives

- Add Position, Direction, CreatureId types
- Implement WorldState with food/barrier/occupancy tracking
- Add kernel module tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Implement Creature Module (Minimal State)

**Files:**
- Create: `v3/crates/v3-core/src/creature/mod.rs`
- Create: `v3/crates/v3-core/src/creature/state.rs`
- Create: `v3/crates/v3-core/tests/creature_state_test.rs`

**Step 1: Write failing test for Energy type**

Create `v3/crates/v3-core/tests/creature_state_test.rs`:
```rust
#[test]
fn energy_drain_returns_success_when_sufficient() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(10.0);

    assert_eq!(energy.drain(5.0), true);
    assert_eq!(energy.value(), 5.0);

    assert_eq!(energy.drain(10.0), false);  // Insufficient
    assert_eq!(energy.value(), 5.0);  // Unchanged
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test energy_drain_returns_success`
Expected: FAIL with "use of undeclared crate or module"

**Step 3: Implement Energy type**

Create `v3/crates/v3-core/src/creature/mod.rs`:
```rust
pub mod state;
```

Create `v3/crates/v3-core/src/creature/state.rs`:
```rust
use crate::kernel::types::{CreatureId, Position};

/// Energy value with safe operations (non-negative, clamped)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Energy(f32);

impl Energy {
    pub fn new(value: f32) -> Self {
        Energy(value.max(0.0))
    }

    pub fn drain(&mut self, amount: f32) -> bool {
        if self.0 >= amount {
            self.0 -= amount;
            true
        } else {
            false
        }
    }

    pub fn charge(&mut self, amount: f32, max: f32) {
        self.0 = (self.0 + amount).min(max).max(0.0);
    }

    pub fn is_alive(&self) -> bool {
        self.0 > 0.0
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Minimal creature state for Phase 1
#[derive(Clone, Debug)]
pub struct CreatureState {
    pub id: CreatureId,
    pub position: Position,
    pub energy: Energy,
    pub age: u64,
    pub generation: u32,
    pub phenotype_r: u8,
    pub phenotype_g: u8,
    pub phenotype_b: u8,
}

impl CreatureState {
    pub fn new(
        id: CreatureId,
        position: Position,
        initial_energy: f32,
        generation: u32,
        phenotype_rgb: [u8; 3],
    ) -> Self {
        Self {
            id,
            position,
            energy: Energy::new(initial_energy),
            age: 0,
            generation,
            phenotype_r: phenotype_rgb[0],
            phenotype_g: phenotype_rgb[1],
            phenotype_b: phenotype_rgb[2],
        }
    }
}
```

Update `v3/crates/v3-core/src/lib.rs`:
```rust
pub mod kernel;
pub mod creature;
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test energy_drain_returns_success`
Expected: PASS

**Step 5: Write test for CreatureState creation**

Add to `v3/crates/v3-core/tests/creature_state_test.rs`:
```rust
#[test]
fn creature_state_new_initializes_correctly() {
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::{CreatureId, Position};

    let creature = CreatureState::new(
        CreatureId(42),
        Position { x: 10, y: 20 },
        15.0,
        0,
        [255, 128, 64],
    );

    assert_eq!(creature.id, CreatureId(42));
    assert_eq!(creature.position, Position { x: 10, y: 20 });
    assert_eq!(creature.energy.value(), 15.0);
    assert_eq!(creature.generation, 0);
    assert_eq!(creature.phenotype_r, 255);
    assert_eq!(creature.phenotype_g, 128);
    assert_eq!(creature.phenotype_b, 64);
}
```

**Step 6: Run test to verify it passes**

Run: `cd v3 && cargo test creature_state_new_initializes_correctly`
Expected: PASS

**Step 7: Commit**

```bash
git add v3/crates/v3-core/src/creature/ v3/crates/v3-core/tests/creature_state_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): implement creature module with minimal state

- Add Energy type with safe operations (drain, charge, is_alive)
- Add CreatureState with id, position, energy, phenotype
- Add creature module tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Implement Contracts Module (Stub Types)

**Files:**
- Create: `v3/crates/v3-core/src/contracts/mod.rs`
- Create: `v3/crates/v3-core/src/contracts/inputs.rs`
- Create: `v3/crates/v3-core/src/contracts/outputs.rs`
- Create: `v3/crates/v3-core/tests/contracts_test.rs`

**Step 1: Write contract shape test**

Create `v3/crates/v3-core/tests/contracts_test.rs`:
```rust
#[test]
fn creature_inputs_has_required_fields() {
    use v3_core::contracts::inputs::CreatureInputs;

    let inputs = CreatureInputs::default();

    // Verify fields exist (contract shape test)
    let _env = inputs.environmental;
    let _intro = inputs.introspection;
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test creature_inputs_has_required_fields`
Expected: FAIL with "use of undeclared module"

**Step 3: Implement stub contracts**

Create `v3/crates/v3-core/src/contracts/mod.rs`:
```rust
pub mod inputs;
pub mod outputs;
```

Create `v3/crates/v3-core/src/contracts/inputs.rs`:
```rust
use crate::kernel::types::Position;

/// Environmental perception inputs (stub for Phase 1)
#[derive(Clone, Debug, Default)]
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
}

/// Introspection inputs (stub for Phase 1)
#[derive(Clone, Debug, Default)]
pub struct IntrospectionInputs {
    pub energy: f32,
    pub position: Position,
}

impl Default for Position {
    fn default() -> Self {
        Position { x: 0, y: 0 }
    }
}

/// Combined inputs for creature execution
#[derive(Clone, Debug, Default)]
pub struct CreatureInputs {
    pub environmental: EnvironmentalInputs,
    pub introspection: IntrospectionInputs,
}
```

Create `v3/crates/v3-core/src/contracts/outputs.rs`:
```rust
use crate::kernel::types::Direction;

/// World actions a creature can attempt
#[derive(Clone, Debug, PartialEq)]
pub enum WorldAction {
    NoOp,
    // More actions added in later phases
}

/// Internal outputs (stub for Phase 1)
#[derive(Clone, Debug, Default)]
pub struct InternalOutputs {
    // Memory writes will be added later
}

/// Complete creature execution output
#[derive(Clone, Debug)]
pub struct CreatureOutputs {
    pub world_action: WorldAction,
    pub internal: InternalOutputs,
    pub energy_consumed: f32,
}

impl CreatureOutputs {
    pub fn noop() -> Self {
        Self {
            world_action: WorldAction::NoOp,
            internal: InternalOutputs::default(),
            energy_consumed: 0.0,
        }
    }
}
```

Update `v3/crates/v3-core/src/lib.rs`:
```rust
pub mod kernel;
pub mod creature;
pub mod contracts;
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test creature_inputs_has_required_fields`
Expected: PASS

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/contracts/ v3/crates/v3-core/tests/contracts_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): implement contracts module with stub types

- Add EnvironmentalInputs and IntrospectionInputs (minimal)
- Add WorldAction enum (NoOp only for Phase 1)
- Add CreatureOutputs structure
- Add contract shape tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Implement Stub Runtime and Sensors

**Files:**
- Create: `v3/crates/v3-core/src/runtime/mod.rs`
- Create: `v3/crates/v3-core/src/runtime/executor.rs`
- Create: `v3/crates/v3-core/src/sensors/mod.rs`
- Create: `v3/crates/v3-core/tests/runtime_executor_test.rs`

**Step 1: Write test for stub executor**

Create `v3/crates/v3-core/tests/runtime_executor_test.rs`:
```rust
#[test]
fn execute_creature_returns_noop_for_phase1() {
    use v3_core::runtime::executor::execute_creature_stub;
    use v3_core::contracts::inputs::CreatureInputs;
    use v3_core::contracts::outputs::WorldAction;

    let inputs = CreatureInputs::default();
    let outputs = execute_creature_stub(&inputs);

    assert!(matches!(outputs.world_action, WorldAction::NoOp));
    assert_eq!(outputs.energy_consumed, 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test execute_creature_returns_noop_for_phase1`
Expected: FAIL

**Step 3: Implement stub runtime**

Create `v3/crates/v3-core/src/runtime/mod.rs`:
```rust
pub mod executor;
```

Create `v3/crates/v3-core/src/runtime/executor.rs`:
```rust
use crate::contracts::inputs::CreatureInputs;
use crate::contracts::outputs::CreatureOutputs;

/// Stub executor for Phase 1 (always returns NoOp)
pub fn execute_creature_stub(_inputs: &CreatureInputs) -> CreatureOutputs {
    CreatureOutputs::noop()
}
```

Create `v3/crates/v3-core/src/sensors/mod.rs`:
```rust
use crate::creature::state::CreatureState;
use crate::kernel::world_state::WorldState;
use crate::contracts::inputs::{CreatureInputs, EnvironmentalInputs, IntrospectionInputs};

/// Stub sensors for Phase 1 (returns minimal inputs)
pub fn gather_inputs_stub(
    creature: &CreatureState,
    _world: &WorldState,
) -> CreatureInputs {
    CreatureInputs {
        environmental: EnvironmentalInputs {
            food_density_self: 0,
        },
        introspection: IntrospectionInputs {
            energy: creature.energy.value(),
            position: creature.position,
        },
    }
}
```

Update `v3/crates/v3-core/src/lib.rs`:
```rust
pub mod kernel;
pub mod creature;
pub mod contracts;
pub mod runtime;
pub mod sensors;
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test execute_creature_returns_noop_for_phase1`
Expected: PASS

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/runtime/ v3/crates/v3-core/src/sensors/ v3/crates/v3-core/tests/runtime_executor_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): add stub runtime and sensors for Phase 1

- Stub executor returns NoOp (no brain execution yet)
- Stub sensors return minimal inputs
- Runtime executor test

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement Tick Orchestrator (Phase 0 Only)

**Files:**
- Create: `v3/crates/v3-core/src/tick/mod.rs`
- Create: `v3/crates/v3-core/src/tick/orchestrator.rs`
- Create: `v3/crates/v3-core/tests/tick_integration_test.rs`

**Step 1: Write failing integration test**

Create `v3/crates/v3-core/tests/tick_integration_test.rs`:
```rust
#[test]
fn tick_advances_and_creatures_persist() {
    use v3_core::tick::orchestrator::tick;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::kernel::types::{CreatureId, Position};
    use v3_core::creature::state::CreatureState;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut world = WorldState::new(10, 10, true);
    let mut creatures = vec![
        CreatureState::new(
            CreatureId(1),
            Position { x: 5, y: 5 },
            10.0,
            0,
            [255, 0, 0],
        ),
    ];

    let mut rng = SmallRng::seed_from_u64(42);

    tick(&mut creatures, &mut world, 0, &mut rng);

    assert_eq!(creatures.len(), 1);
    assert_eq!(creatures[0].age, 1);  // Age should increment
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test tick_advances_and_creatures_persist`
Expected: FAIL

**Step 3: Implement tick orchestrator (Phase 0 only)**

Create `v3/crates/v3-core/src/tick/mod.rs`:
```rust
pub mod orchestrator;
```

Create `v3/crates/v3-core/src/tick/orchestrator.rs`:
```rust
use crate::kernel::world_state::WorldState;
use crate::creature::state::CreatureState;
use rand::Rng;

/// Phase 1 tick: Only world mechanics and age increment (no cognition yet)
pub fn tick(
    creatures: &mut Vec<CreatureState>,
    _world: &mut WorldState,
    _tick_number: u64,
    _rng: &mut impl Rng,
) {
    // Phase 0: World mechanics (stub for now - no food growth yet)

    // Increment creature age
    for creature in creatures.iter_mut() {
        creature.age += 1;
    }

    // Phase 1: Cognition (stub - not implemented in Phase 1)
    // Phase 2: Action execution (stub - not implemented in Phase 1)

    // Phase 3: Cleanup (remove dead creatures)
    creatures.retain(|c| c.energy.is_alive());
}
```

Update `v3/crates/v3-core/src/lib.rs`:
```rust
pub mod kernel;
pub mod creature;
pub mod contracts;
pub mod runtime;
pub mod sensors;
pub mod tick;
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test tick_advances_and_creatures_persist`
Expected: PASS

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/tick/ v3/crates/v3-core/tests/tick_integration_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): implement tick orchestrator with Phase 0 only

- Tick increments creature age
- Removes dead creatures
- Integration test verifies tick advances
- Cognition and actions stubbed for later phases

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add v3-server Crate with Basic Endpoints

**Files:**
- Create: `v3/crates/v3-server/Cargo.toml`
- Create: `v3/crates/v3-server/src/main.rs`
- Create: `v3/crates/v3-server/src/state.rs`
- Create: `v3/crates/v3-server/src/api.rs`
- Modify: `v3/Cargo.toml`

**Step 1: Add v3-server to workspace**

Update `v3/Cargo.toml`:
```toml
[workspace]
members = [
    "crates/v3-core",
    "crates/v3-server",
]
resolver = "2"
```

**Step 2: Create v3-server crate**

```bash
mkdir -p v3/crates/v3-server/src
```

Create `v3/crates/v3-server/Cargo.toml`:
```toml
[package]
name = "v3-server"
version = "0.1.0"
edition = "2021"

[dependencies]
v3-core = { path = "../v3-core" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = { version = "0.8", features = ["small_rng"] }
```

**Step 3: Implement server state**

Create `v3/crates/v3-server/src/state.rs`:
```rust
use v3_core::{
    kernel::{world_state::WorldState, types::{CreatureId, Position}},
    creature::state::CreatureState,
};
use rand::{SeedableRng, rngs::SmallRng};

pub struct ServerState {
    pub creatures: Vec<CreatureState>,
    pub world: WorldState,
    pub tick: u64,
    pub running: bool,
    pub rng: SmallRng,
}

impl ServerState {
    pub fn new() -> Self {
        let mut world = WorldState::new(400, 400, true);

        // Spawn initial creatures
        let creatures = vec![
            CreatureState::new(
                CreatureId(1),
                Position { x: 200, y: 200 },
                20.0,
                0,
                [255, 100, 50],
            ),
            CreatureState::new(
                CreatureId(2),
                Position { x: 150, y: 150 },
                20.0,
                0,
                [100, 200, 255],
            ),
        ];

        // Mark creatures as occupying cells
        for creature in &creatures {
            world.mark_occupied(creature.position);
        }

        Self {
            creatures,
            world,
            tick: 0,
            running: false,
            rng: SmallRng::seed_from_u64(42),
        }
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }

        v3_core::tick::orchestrator::tick(
            &mut self.creatures,
            &mut self.world,
            self.tick,
            &mut self.rng,
        );

        self.tick += 1;
    }
}
```

**Step 4: Implement API endpoints**

Create `v3/crates/v3-server/src/api.rs`:
```rust
use axum::{Json, extract::State, http::StatusCode};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::ServerState;

#[derive(Serialize)]
pub struct StatusResponse {
    pub tick: u64,
    pub running: bool,
    pub creature_count: usize,
}

pub async fn get_status(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> Json<StatusResponse> {
    let state = state.read().await;
    Json(StatusResponse {
        tick: state.tick,
        running: state.running,
        creature_count: state.creatures.len(),
    })
}

pub async fn start_simulation(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> StatusCode {
    let mut state = state.write().await;
    state.running = true;
    StatusCode::OK
}

pub async fn pause_simulation(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> StatusCode {
    let mut state = state.write().await;
    state.running = false;
    StatusCode::OK
}
```

**Step 5: Implement main server**

Create `v3/crates/v3-server/src/main.rs`:
```rust
mod state;
mod api;

use axum::{Router, routing::{get, post}};
use std::sync::Arc;
use tokio::sync::RwLock;
use state::ServerState;

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(ServerState::new()));

    // Spawn tick loop
    let tick_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(100)
        );
        loop {
            interval.tick().await;
            tick_state.write().await.tick();
        }
    });

    let app = Router::new()
        .route("/v3/simulation/status", get(api::get_status))
        .route("/v3/simulation/start", post(api::start_simulation))
        .route("/v3/simulation/pause", post(api::pause_simulation))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:4000");
    axum::serve(listener, app).await.unwrap();
}
```

**Step 6: Test server builds and runs**

Run: `cd v3 && cargo run -p v3-server`
Expected: Server starts on port 4000

Stop server with Ctrl+C

**Step 7: Test endpoints**

In another terminal:
```bash
curl http://127.0.0.1:4000/v3/simulation/status
# Expected: {"tick":0,"running":false,"creature_count":2}

curl -X POST http://127.0.0.1:4000/v3/simulation/start

curl http://127.0.0.1:4000/v3/simulation/status
# Expected: {"tick":<some number>,"running":true,"creature_count":2}
```

**Step 8: Commit**

```bash
git add v3/Cargo.toml v3/crates/v3-server/
git commit -m "feat(v3): add server with basic endpoints

- Create v3-server crate
- Implement ServerState with tick loop
- Add /status, /start, /pause endpoints
- Server listens on 127.0.0.1:4000

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Verify Phase 1 Complete

**Step 1: Run all tests**

Run: `cd v3 && cargo test --workspace`
Expected: All tests PASS

**Step 2: Run formatting check**

Run: `cd v3 && cargo fmt --all --check`
Expected: No formatting issues

**Step 3: Run clippy**

Run: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
Expected: No clippy warnings

**Step 4: Manual verification**

Run server:
```bash
cd v3 && cargo run -p v3-server
```

Test in browser/curl:
- Status endpoint returns valid JSON
- Start makes tick advance
- Pause stops tick advancement

**Step 5: Final commit**

```bash
git add -A
git commit -m "chore(v3): Phase 1 walking skeleton complete

Phase 1 delivers:
- Minimal kernel (world state primitives)
- Minimal creature (state + energy)
- Stub contracts (inputs/outputs)
- Stub runtime/sensors (NoOp)
- Tick orchestrator (Phase 0 only)
- Server with basic endpoints
- All tests passing

Next: Phase 2 will add food and eat action

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Next Steps

**Phase 1 is complete when:**
- All tests pass
- Server starts and responds to endpoints
- Tick advances when simulation is running
- Creatures persist across ticks

**Phase 2 will add:**
- Food grid in world state
- Food sensing in sensors
- Eat action in contracts
- Action execution in tick

**Phase 3+ continue building out the walking skeleton incrementally.**

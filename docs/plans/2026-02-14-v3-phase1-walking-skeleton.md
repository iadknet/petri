# V3 Stage 1: Foundation + Working Tick Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build minimal backend vertical slice where creatures exist, tick advances, and server responds to REST queries. This is Stage 1 of the walking skeleton (see architecture design doc for the full 3-stage strategy). Frontend (WebSocket streaming, web client rendering) is deferred to a separate frontend architecture plan.

**Architecture:** Phase-based tick (Phase 0: world mechanics only, no cognition yet), kernel owns world primitives, creature owns state, contracts define interfaces, stub runtime/sensors return NoOp, server exposes REST lifecycle endpoints.

**Tech Stack:** Rust 1.93.0, Axum (server), Tokio (async), serde_json (serialization), slotmap (creature storage)

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
slotmap = "1.0"
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

#[test]
fn direction_delta_returns_correct_offsets() {
    use v3_core::kernel::types::Direction;

    assert_eq!(Direction::N.delta(),  ( 0, -1));
    assert_eq!(Direction::NE.delta(), ( 1, -1));
    assert_eq!(Direction::E.delta(),  ( 1,  0));
    assert_eq!(Direction::SE.delta(), ( 1,  1));
    assert_eq!(Direction::S.delta(),  ( 0,  1));
    assert_eq!(Direction::SW.delta(), (-1,  1));
    assert_eq!(Direction::W.delta(),  (-1,  0));
    assert_eq!(Direction::NW.delta(), (-1, -1));
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
use slotmap::new_key_type;

/// Position in world grid
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Direction (cardinal and intercardinal)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::N  => ( 0, -1),
            Direction::NE => ( 1, -1),
            Direction::E  => ( 1,  0),
            Direction::SE => ( 1,  1),
            Direction::S  => ( 0,  1),
            Direction::SW => (-1,  1),
            Direction::W  => (-1,  0),
            Direction::NW => (-1, -1),
        }
    }
}

/// Unique creature identifier — slotmap key for O(1) lookup with stable handles.
new_key_type! { pub struct CreatureId; }
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
    assert!(!world.is_occupied(Position { x: 5, y: 5 }));
    assert!(world.creature_at(Position { x: 5, y: 5 }).is_none());
}
```

**Step 6: Run test to verify it fails**

Run: `cd v3 && cargo test world_state_new_creates_empty_world`
Expected: FAIL with "no method named `new`"

**Step 7: Implement WorldState**

Create `v3/crates/v3-core/src/kernel/world_state.rs`:
```rust
use super::types::{CreatureId, Position};

pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,

    // Flat arrays indexed by (y * width + x) for cache-friendly O(1) access.
    // A 400x400 world is only 160K entries — flat Vec beats HashMap here.

    /// Food density per cell (0-255 quantized)
    food: Vec<u8>,

    /// Barrier locations
    barriers: Vec<bool>,

    /// Spatial index: which creature (if any) occupies each cell.
    /// Provides O(1) position→creature lookup and occupancy checking.
    creature_at: Vec<Option<CreatureId>>,
}

impl WorldState {
    pub fn new(width: u16, height: u16, wrap: bool) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            wrap,
            food: vec![0u8; len],
            barriers: vec![false; len],
            creature_at: vec![None; len],
        }
    }

    #[inline]
    fn cell_index(&self, pos: Position) -> usize {
        (pos.y as usize) * (self.width as usize) + (pos.x as usize)
    }

    pub fn get_food_density(&self, pos: Position) -> u8 {
        self.food[self.cell_index(pos)]
    }

    pub fn is_occupied(&self, pos: Position) -> bool {
        self.creature_at[self.cell_index(pos)].is_some()
    }

    pub fn creature_at(&self, pos: Position) -> Option<CreatureId> {
        self.creature_at[self.cell_index(pos)]
    }

    pub fn is_barrier(&self, pos: Position) -> bool {
        self.barriers[self.cell_index(pos)]
    }

    pub fn place_creature(&mut self, pos: Position, id: CreatureId) {
        let idx = self.cell_index(pos);
        debug_assert!(self.creature_at[idx].is_none(), "cell already occupied");
        self.creature_at[idx] = Some(id);
    }

    pub fn remove_creature(&mut self, pos: Position) {
        let idx = self.cell_index(pos);
        debug_assert!(self.creature_at[idx].is_some(), "cell not occupied");
        self.creature_at[idx] = None;
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

    let mut energy = Energy::new(10);

    assert_eq!(energy.drain(5), true);
    assert_eq!(energy.value(), 5);

    assert_eq!(energy.drain(10), false);  // Insufficient
    assert_eq!(energy.value(), 5);  // Unchanged
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
use crate::kernel::types::Position;

/// Energy value with safe operations (non-negative, integer).
/// Uses u32 to avoid floating-point flakiness in tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Energy(u32);

impl Energy {
    pub fn new(value: u32) -> Self {
        Energy(value)
    }

    pub fn drain(&mut self, amount: u32) -> bool {
        if self.0 >= amount {
            self.0 -= amount;
            true
        } else {
            false
        }
    }

    pub fn charge(&mut self, amount: u32, max: u32) {
        self.0 = (self.0 + amount).min(max);
    }

    pub fn is_alive(&self) -> bool {
        self.0 > 0
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Minimal creature state for Stage 1.
/// Note: no `id` field — the SlotMap key IS the creature's identity.
#[derive(Clone, Debug)]
pub struct CreatureState {
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
        position: Position,
        initial_energy: u32,
        generation: u32,
        phenotype_rgb: [u8; 3],
    ) -> Self {
        Self {
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
    use v3_core::kernel::types::Position;

    let creature = CreatureState::new(
        Position { x: 10, y: 20 },
        15,
        0,
        [255, 128, 64],
    );

    assert_eq!(creature.position, Position { x: 10, y: 20 });
    assert_eq!(creature.energy.value(), 15);
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
- Add CreatureState with position, energy, phenotype (SlotMap key is identity)
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

/// Environmental perception inputs (stub for Stage 1)
#[derive(Clone, Debug, Default)]
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
}

/// Introspection inputs (stub for Stage 1)
#[derive(Clone, Debug, Default)]
pub struct IntrospectionInputs {
    pub energy: u32,
    pub position: Position,
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
/// World actions a creature can attempt
#[derive(Clone, Debug, PartialEq)]
pub enum WorldAction {
    NoOp,
    // More actions added in later stages (Move, Eat, Reproduce, etc.)
}

/// Internal outputs (stub for Stage 1)
#[derive(Clone, Debug, Default)]
pub struct InternalOutputs {
    // Memory writes will be added later
}

/// Complete creature execution output
#[derive(Clone, Debug)]
pub struct CreatureOutputs {
    pub world_action: WorldAction,
    pub internal: InternalOutputs,
    pub energy_consumed: u32,
}

impl CreatureOutputs {
    pub fn noop() -> Self {
        Self {
            world_action: WorldAction::NoOp,
            internal: InternalOutputs::default(),
            energy_consumed: 0,
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
- Add WorldAction enum (NoOp only for Stage 1)
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
    assert_eq!(outputs.energy_consumed, 0);
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

/// Stub executor for Stage 1 (always returns NoOp)
pub fn execute_creature_stub(_inputs: &CreatureInputs) -> CreatureOutputs {
    CreatureOutputs::noop()
}
```

Create `v3/crates/v3-core/src/sensors/mod.rs`:
```rust
use crate::creature::state::CreatureState;
use crate::kernel::world_state::WorldState;
use crate::contracts::inputs::{CreatureInputs, EnvironmentalInputs, IntrospectionInputs};

/// Stub sensors for Stage 1 (returns minimal inputs)
pub fn gather_inputs_stub(
    creature: &CreatureState,
    _world: &WorldState,
) -> CreatureInputs {
    CreatureInputs {
        environmental: EnvironmentalInputs::default(),
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
git commit -m "feat(v3): add stub runtime and sensors for Stage 1

- Stub executor returns NoOp (no brain execution yet)
- Stub sensors return minimal inputs
- Runtime executor test

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement SimulationState and Tick Orchestrator (Phase 0 Only)

**Files:**
- Modify: `v3/crates/v3-core/src/lib.rs` (add SimulationState)
- Create: `v3/crates/v3-core/src/tick/mod.rs`
- Create: `v3/crates/v3-core/src/tick/orchestrator.rs`
- Create: `v3/crates/v3-core/tests/tick_integration_test.rs`

**Step 1: Write failing integration test**

Create `v3/crates/v3-core/tests/tick_integration_test.rs`:
```rust
#[test]
fn tick_advances_and_creatures_persist() {
    use v3_core::SimulationState;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    // Use spawn_creature directly (seed module is Task 7)
    state.spawn_creature(CreatureState::new(Position { x: 1, y: 1 }, 10, 0, [255, 0, 0]));
    state.spawn_creature(CreatureState::new(Position { x: 2, y: 2 }, 10, 0, [0, 255, 0]));
    state.spawn_creature(CreatureState::new(Position { x: 3, y: 3 }, 10, 0, [0, 0, 255]));
    assert_eq!(state.creatures.len(), 3);

    state.tick(&mut rng);

    assert_eq!(state.tick_number, 1);
    assert_eq!(state.creatures.len(), 3);
    // All creatures should have aged
    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.age, 1);
        assert!(state.world.creature_at(creature.position).is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test tick_advances_and_creatures_persist`
Expected: FAIL

**Step 3: Implement SimulationState and tick orchestrator (Phase 0 only)**

Create `v3/crates/v3-core/src/tick/mod.rs`:
```rust
pub mod orchestrator;
```

Create `v3/crates/v3-core/src/tick/orchestrator.rs`:
```rust
use crate::SimulationState;
use crate::kernel::types::CreatureId;
use rand::Rng;

/// Per-tick observability counters (GP-04). Stub for Stage 1.
pub struct TickStats {
    pub births: u32,
    pub deaths: u32,
}

/// Stage 1 tick: Only world mechanics and age increment (no cognition yet)
pub fn tick(
    state: &mut SimulationState,
    _rng: &mut impl Rng,
) -> TickStats {
    // Phase 0: World mechanics (stub for now - no food growth yet)

    // Increment creature age
    for (_, creature) in state.creatures.iter_mut() {
        creature.age += 1;
    }

    // Phase 1: Cognition (stub - not implemented in Stage 1)
    // Phase 2: Action execution (stub - not implemented in Stage 1)

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

    TickStats { births: 0, deaths }
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

use slotmap::SlotMap;
use kernel::types::CreatureId;
use kernel::world_state::WorldState;
use creature::state::CreatureState;

/// Coordination shell for all mutable simulation state.
/// No business logic — modules own behavior, this owns the data bundle.
///
/// Fields are `pub` for borrow splitting: the tick orchestrator needs
/// simultaneous `&mut creatures` and `&mut world`. Rust allows this via
/// direct field access but not through `&mut self` accessor methods.
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

    pub fn tick(&mut self, rng: &mut impl rand::Rng) -> tick::orchestrator::TickStats {
        tick::orchestrator::tick(self, rng)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd v3 && cargo test tick_advances_and_creatures_persist`
Expected: PASS

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/tick/ v3/crates/v3-core/tests/tick_integration_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): add SimulationState and tick orchestrator (Phase 0)

- SimulationState bundles SlotMap creatures/world/tick counter
- Tick increments creature age, removes dead creatures + spatial index cleanup
- TickStats returned for GP-04 observability
- Cognition and actions stubbed for later phases

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement Seed Module

**Files:**
- Create: `v3/crates/v3-core/src/seed.rs`
- Modify: `v3/crates/v3-core/src/lib.rs` (add `pub mod seed;`)
- Create: `v3/crates/v3-core/tests/seed_test.rs`

**Step 1: Write failing test for seed_creatures**

Create `v3/crates/v3-core/tests/seed_test.rs`:
```rust
#[test]
fn seed_creatures_places_creatures_at_unique_positions() {
    use v3_core::SimulationState;
    use v3_core::seed::seed_creatures;
    use v3_core::kernel::world_state::WorldState;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 5, 20, &mut rng);

    assert_eq!(state.creatures.len(), 5);

    // All creatures should be at unique, occupied positions
    for (id, creature) in state.creatures.iter() {
        assert!(state.world.creature_at(creature.position).is_some());
        assert_eq!(state.world.creature_at(creature.position), Some(id));
    }
}

#[test]
fn seed_creatures_respects_world_capacity() {
    use v3_core::SimulationState;
    use v3_core::seed::seed_creatures;
    use v3_core::kernel::world_state::WorldState;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    // 2x2 world = 4 cells, request 10 creatures
    let mut state = SimulationState::new(WorldState::new(2, 2, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 10, 20, &mut rng);

    assert!(state.creatures.len() <= 4);  // Best-effort, bounded by capacity
}
```

**Step 2: Run test to verify it fails**

Run: `cd v3 && cargo test seed_test`
Expected: FAIL with "use of undeclared module"

**Step 3: Implement seed module**

Create `v3/crates/v3-core/src/seed.rs`:
```rust
use crate::SimulationState;
use crate::kernel::types::Position;
use crate::kernel::world_state::WorldState;
use crate::creature::state::CreatureState;
use rand::Rng;

/// Seed creatures into the simulation at random empty positions.
/// Best-effort: if the world is too full, fewer creatures are placed.
pub fn seed_creatures(
    state: &mut SimulationState,
    count: usize,
    initial_energy: u32,
    rng: &mut impl Rng,
) {
    for _ in 0..count {
        if let Some(pos) = find_empty_position(&state.world, rng) {
            let creature = CreatureState::new(
                pos,
                initial_energy,
                0,
                random_phenotype(rng),
            );
            state.spawn_creature(creature);
        }
    }
}

fn find_empty_position(world: &WorldState, rng: &mut impl Rng) -> Option<Position> {
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

fn random_phenotype(rng: &mut impl Rng) -> [u8; 3] {
    [rng.gen(), rng.gen(), rng.gen()]
}
```

Update `v3/crates/v3-core/src/lib.rs` — add after `pub mod tick;`:
```rust
pub mod seed;
```

**Step 4: Run tests to verify they pass**

Run: `cd v3 && cargo test seed_test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/seed.rs v3/crates/v3-core/tests/seed_test.rs v3/crates/v3-core/src/lib.rs
git commit -m "feat(v3): add seed module for initial creature placement

- seed_creatures() places N creatures at random empty positions
- Best-effort placement bounded by world capacity
- Shared by server, CLI, and viability tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add v3-server Crate with Basic Endpoints

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
    SimulationState,
    seed::seed_creatures,
    kernel::world_state::WorldState,
};
use rand::{SeedableRng, rngs::SmallRng};

pub struct ServerState {
    pub sim: SimulationState,
    pub running: bool,
    pub rng: SmallRng,
}

impl ServerState {
    pub fn new() -> Self {
        let world = WorldState::new(400, 400, true);
        let mut sim = SimulationState::new(world);
        let mut rng = SmallRng::seed_from_u64(42);

        seed_creatures(&mut sim, 50, 20, &mut rng);

        Self {
            sim,
            running: false,
            rng,
        }
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }

        let _stats = self.sim.tick(&mut self.rng);
    }
}
```

**Step 4: Implement API endpoints**

Create `v3/crates/v3-server/src/api.rs`:
```rust
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
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
        tick: state.sim.tick_number,
        running: state.running,
        creature_count: state.sim.creatures.len(),
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
# Expected: {"tick":0,"running":false,"creature_count":50}

curl -X POST http://127.0.0.1:4000/v3/simulation/start
# Wait a moment...

curl http://127.0.0.1:4000/v3/simulation/status
# Expected: {"tick":<nonzero>,"running":true,"creature_count":50}
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

## Task 9: Verify Stage 1 Complete

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
git commit -m "chore(v3): Stage 1 walking skeleton complete

Stage 1 delivers:
- Minimal kernel (world state primitives)
- Minimal creature (state + energy)
- Stub contracts (inputs/outputs)
- Stub runtime/sensors (NoOp)
- Tick orchestrator (Phase 0 only)
- Server with basic endpoints
- All tests passing

Next: Stage 2 will add viable ecology (food, eat, move, decay, viability test)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Next Steps

**Stage 1 is complete when:**
- All tests pass (`cargo test --workspace`, `cargo clippy -- -D warnings`, `cargo fmt --check`)
- Server starts and responds to REST endpoints
- Tick advances when simulation is running
- Creatures persist across ticks

**Stage 2 (Viable Ecology) will add:**
- `config/` module (FoodConfig, EnergyConfig, SimulationConfig)
- Food grid operations in kernel (set_food_density, consume_food, grow_food)
- Energy decay, Eat action, Move action
- Heuristic brain (eat if food, move toward food, else random)
- **Seed creature viability test** (< 2 second target, 32x32 world, ~100 ticks)
- Config parameter threaded through tick function signature

**Frontend architecture** (WebSocket streaming, web client, frame serialization) will be designed in a separate plan.

**Stage 3 (Evolving Ecology)** adds reproduction, mutation, and VM brain.

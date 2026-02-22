# V3 Stage 1: Scaffolding + Contracts + Kernel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Invoke `rust-skills` before writing any Rust code.

**Goal:** Create the v3-core crate scaffold, all shared contract types, the simulation config struct, and the kernel world-state module — giving all later stages a stable foundation.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`

**Scope:** Everything in `v3/crates/v3-core/` for contracts, config, and kernel modules. AGENTS.md restoration. v3-server shell (empty, just a valid Cargo manifest). No runtime, sensors, creature, or tick modules yet.

**Docs Impact:**
- No canonical docs changed.
- AGENTS.md restored from git history; no spec changes.
- `docs/standards/architecture-size-baseline.tsv` may need new entries if any file exceeds 400 lines (unlikely at this stage).

**Supersedes:** none

**Superseded-By:** none

**Architecture:** The v3 workspace lives at `v3/` (a separate nested Cargo workspace, not part of the root workspace). `v3-core` is a pure simulation library — no tokio, axum, or transport deps. `v3-server` is an empty shell until Stage 6. All modules in v3-core follow strict dependency direction per architecture.md: `contracts` has no internal deps; `kernel` depends only on `contracts`.

**Tech Stack:** Rust 1.93.0, serde (derive), rand (SmallRng/Bernoulli), slotmap (for CreatureId), no async runtime in v3-core.

---

## Goal Alignment

- **GP-01**: Contracts establish the `WorldAction`, `Direction`, `InputReference`, and genome type boundaries needed for richer creature decision-making.
- **GP-02**: Clean module split — `contracts/` < `kernel/` — with no circular deps; `lib.rs` is export-only.
- **GP-03**: Every type is covered by unit tests; food growth uses seeded RNG for reproducible tests; config defaults are explicitly tested against spec values.

## Boundary Impact

- New crate `v3/crates/v3-core` created. No changes to legacy `crates/petri-*`.
- New crate `v3/crates/v3-server` created as empty shell.
- `AGENTS.md` restored at repo root.
- Architecture harness (`scripts/check-architecture-harness.sh`) only scans `crates/*/`, not `v3/crates/*/`. No violations introduced.

## Existing Boundary Recheck

| area | decision | rationale |
|------|----------|-----------|
| `crates/petri-core/src/` | keep | Legacy crate; not touched in this stage |
| `crates/petri-graph/src/` | keep | Legacy crate; not touched in this stage |
| `v3/Cargo.toml` workspace manifest | keep | Already exists; only members we add are v3-core/v3-server which already declared |
| `docs/strategy/architecture.md` | keep | Already correctly describes target V3 layout |

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| Does the v3 workspace need its own rustfmt/clippy config separate from the root? | Yes — create `v3/rustfmt.toml` and `v3/.clippy.toml` per the architecture lint policy requiring clippy lints and rustfmt config. | agent | resolved |
| Is `CreatureId` a slotmap key or a newtype u64? | Use `slotmap::DefaultKey` wrapped in a newtype `CreatureId(DefaultKey)` so future stages can swap to `SecondaryMap` indexing. | agent | resolved |
| Where does `NodeId` live? | In `contracts/` as `pub struct NodeId(pub u32)` — simple u32 newtype for genome node references. | agent | resolved |
| Does `SimulationConfig` own world config or is it a separate struct? | Single `SimulationConfig` containing nested sub-structs (`WorldConfig`, `EnergyConfig`, `RuntimeConfig`, `MutationConfig`, `PopulationConfig`) — not split into separate files until needed. | agent | resolved |

---

## Tasks

### Task 1: Restore AGENTS.md

**Files:**
- Create: `AGENTS.md`

**Step 1: Restore from git history and add rust-skills imperative**

Recover the last known content from `6adee98^` (already retrieved above). Add the rust-skills imperative to the "Required Skills" section.

The full content is captured via `git show 6adee98^:AGENTS.md`. Write it to `AGENTS.md` with one addition in the "Required Skills" section under "Rust crates (`crates/`)" — expand the first bullet to also cover `v3/` code:

```text
- **`rust-skills`**: Invoke when writing, reviewing, or refactoring ANY Rust code (both `crates/` and `v3/`). ALWAYS invoke this skill before writing or reviewing Rust code. Covers ownership, error handling, async patterns, API design, memory optimization, performance, and testing.
```

**Step 2: Verify file exists**

```bash
test -f AGENTS.md && echo "exists" || echo "MISSING"
```

**Step 3: Commit**

```bash
git add AGENTS.md
git commit -m "docs: restore AGENTS.md with rust-skills imperative for v3"
```

---

### Task 2: v3-core crate scaffold

**Files:**
- Modify: `v3/Cargo.toml` (add workspace.package and workspace.dependencies)
- Create: `v3/crates/v3-core/Cargo.toml`
- Create: `v3/crates/v3-server/Cargo.toml`
- Create: `v3/crates/v3-core/src/lib.rs`
- Create: `v3/crates/v3-server/src/lib.rs`
- Create: `v3/rustfmt.toml`

**Step 1: Update `v3/Cargo.toml` with workspace deps**

```toml
[workspace]
members = [
    "crates/v3-core",
    "crates/v3-server",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"
authors = ["Petri Developers"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
rand = { version = "0.8", features = ["small_rng"] }
slotmap = "1.0"
serde_json = "1.0"
```

**Step 2: Create `v3/crates/v3-core/Cargo.toml`**

```toml
[package]
name = "v3-core"
edition.workspace = true
version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
serde = { workspace = true }
rand = { workspace = true }
slotmap = { workspace = true }

[dev-dependencies]
serde_json = { workspace = true }
```

**Step 3: Create `v3/crates/v3-server/Cargo.toml`**

```toml
[package]
name = "v3-server"
edition.workspace = true
version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
v3-core = { path = "../v3-core" }
```

**Step 4: Create `v3/crates/v3-core/src/lib.rs`** (export-only, starts minimal)

```rust
// v3-core: simulation engine
// This file is export-only. No impl items here.
pub mod contracts;
pub mod config;
pub mod kernel;
```

**Step 5: Create module skeleton directories**

```bash
mkdir -p v3/crates/v3-core/src/contracts
mkdir -p v3/crates/v3-core/src/config
mkdir -p v3/crates/v3-core/src/kernel
mkdir -p v3/crates/v3-server/src
touch v3/crates/v3-server/src/lib.rs
```

**Step 6: Create skeleton `mod.rs` files** (empty, will be filled in subsequent tasks)

`v3/crates/v3-core/src/contracts/mod.rs`:
```rust
mod direction;
mod position;
mod ids;
mod actions;
mod inputs;

pub use direction::Direction;
pub use position::Position;
pub use ids::{CreatureId, NodeId};
pub use actions::WorldAction;
pub use inputs::{InputReference, WorldInputKey, StaticIntrospectionKey, DynamicIntrospectionKey};
```

`v3/crates/v3-core/src/config/mod.rs`:
```rust
mod simulation;

pub use simulation::SimulationConfig;
```

`v3/crates/v3-core/src/kernel/mod.rs`:
```rust
mod grid;
mod world;

pub use grid::Grid;
pub use world::WorldState;
```

**Step 7: Create `v3/rustfmt.toml`**

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
```

**Step 8: Verify workspace compiles (once all stubs are added)**

```bash
cd v3 && cargo check --workspace 2>&1 | head -20
```

Expected: errors only about missing modules/types (not yet written).

---

### Task 3: Direction type

**Files:**
- Create: `v3/crates/v3-core/src/contracts/direction.rs`

**Spec reference:** `docs/reference/v3-world-grid-spec.md` Section 3.

**Step 1: Write failing test**

Add to `v3/crates/v3-core/src/contracts/direction.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N, Direction::NE, Direction::E, Direction::SE,
        Direction::S, Direction::SW, Direction::W, Direction::NW,
    ];

    /// Returns (dx, dy) delta. x increases right, y increases down.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_all_has_eight_entries() {
        assert_eq!(Direction::ALL.len(), 8);
    }

    #[test]
    fn direction_deltas_canonical() {
        assert_eq!(Direction::N.delta(),  ( 0, -1));
        assert_eq!(Direction::NE.delta(), ( 1, -1));
        assert_eq!(Direction::E.delta(),  ( 1,  0));
        assert_eq!(Direction::SE.delta(), ( 1,  1));
        assert_eq!(Direction::S.delta(),  ( 0,  1));
        assert_eq!(Direction::SW.delta(), (-1,  1));
        assert_eq!(Direction::W.delta(),  (-1,  0));
        assert_eq!(Direction::NW.delta(), (-1, -1));
    }

    #[test]
    fn direction_order_matches_spec() {
        // Canonical order per v3-world-grid-spec: N, NE, E, SE, S, SW, W, NW
        let expected = [
            Direction::N, Direction::NE, Direction::E, Direction::SE,
            Direction::S, Direction::SW, Direction::W, Direction::NW,
        ];
        assert_eq!(Direction::ALL, expected);
    }

    #[test]
    fn direction_serde_roundtrip() {
        let d = Direction::SE;
        let json = serde_json::to_string(&d).unwrap();
        let d2: Direction = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
    }
}
```

**Step 2: Run test**

```bash
cd v3 && cargo test contracts::direction -- --nocapture 2>&1 | tail -10
```

Expected: PASS (implementation is inline with test).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/contracts/direction.rs
git commit -m "feat(v3-core): add Direction type with canonical ALL and delta mapping"
```

---

### Task 4: Position type

**Files:**
- Create: `v3/crates/v3-core/src/contracts/position.rs`

**Spec reference:** `docs/reference/v3-world-grid-spec.md` Sections 3–4.

**Step 1: Write and implement Position**

```rust
/// A 2D grid position with (x, y) coordinates.
/// x in [0, width-1], y in [0, height-1]. Origin (0,0) is top-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Compute neighbor in wrap mode (toroidal).
    /// Returns None only if width or height is 0 (invalid grid).
    pub fn neighbor_wrap(self, dx: i32, dy: i32, width: u16, height: u16) -> Option<Position> {
        if width == 0 || height == 0 {
            return None;
        }
        let nx = (self.x as i32 + dx).rem_euclid(width as i32) as u16;
        let ny = (self.y as i32 + dy).rem_euclid(height as i32) as u16;
        Some(Position::new(nx, ny))
    }

    /// Compute neighbor in bounded mode.
    /// Returns None if the neighbor is outside [0, width-1] x [0, height-1].
    pub fn neighbor_bounded(self, dx: i32, dy: i32, width: u16, height: u16) -> Option<Position> {
        let nx = self.x as i32 + dx;
        let ny = self.y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            return None;
        }
        Some(Position::new(nx as u16, ny as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;

    #[test]
    fn position_wrap_north_from_top_edge() {
        let pos = Position::new(5, 0);
        let (dx, dy) = Direction::N.delta();
        let neighbor = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(5, 9), "wrap north from row 0 should reach row 9");
    }

    #[test]
    fn position_wrap_east_from_right_edge() {
        let pos = Position::new(9, 5);
        let (dx, dy) = Direction::E.delta();
        let neighbor = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(0, 5), "wrap east from col 9 should reach col 0");
    }

    #[test]
    fn position_bounded_off_edge_returns_none() {
        let pos = Position::new(0, 0);
        let (dx, dy) = Direction::N.delta();
        assert!(pos.neighbor_bounded(dx, dy, 10, 10).is_none());
    }

    #[test]
    fn position_bounded_valid_returns_some() {
        let pos = Position::new(3, 3);
        let (dx, dy) = Direction::SE.delta();
        let neighbor = pos.neighbor_bounded(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(4, 4));
    }

    #[test]
    fn position_wrap_all_directions_stay_in_bounds() {
        let pos = Position::new(5, 5);
        for dir in Direction::ALL {
            let (dx, dy) = dir.delta();
            let n = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
            assert!(n.x < 10 && n.y < 10, "wrap must stay in bounds");
        }
    }
}
```

**Step 2: Run tests**

```bash
cd v3 && cargo test contracts::position -- --nocapture 2>&1 | tail -15
```

Expected: PASS (5 tests).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/contracts/position.rs
git commit -m "feat(v3-core): add Position type with toroidal wrap and bounded neighbor resolution"
```

---

### Task 5: Identity types (CreatureId, NodeId)

**Files:**
- Create: `v3/crates/v3-core/src/contracts/ids.rs`

**Step 1: Implement**

```rust
use slotmap::new_key_type;

new_key_type! {
    /// Opaque identifier for a living creature in the simulation.
    pub struct CreatureId;
}

/// Opaque identifier for a genome node. Simple u32 newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
         serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    #[test]
    fn creature_id_slotmap_roundtrip() {
        let mut map: SlotMap<CreatureId, u32> = SlotMap::with_key();
        let id = map.insert(42);
        assert_eq!(map[id], 42);
        map.remove(id);
        assert!(!map.contains_key(id));
    }

    #[test]
    fn node_id_ordering() {
        assert!(NodeId(0) < NodeId(1));
        assert_eq!(NodeId(5), NodeId(5));
    }

    #[test]
    fn node_id_serde_roundtrip() {
        let id = NodeId(7);
        let json = serde_json::to_string(&id).unwrap();
        let id2: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, id2);
    }
}
```

**Step 2: Run tests**

```bash
cd v3 && cargo test contracts::ids -- --nocapture 2>&1 | tail -10
```

Expected: PASS (3 tests).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/contracts/ids.rs
git commit -m "feat(v3-core): add CreatureId (slotmap key) and NodeId types"
```

---

### Task 6: WorldAction type

**Files:**
- Create: `v3/crates/v3-core/src/contracts/actions.rs`

**Spec reference:** `docs/reference/v3-mesh-execution-spec.md` Section 2 (WorldAction).

**Step 1: Implement**

```rust
use crate::contracts::Direction;

/// The action a creature emits at the end of mesh execution for one tick.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldAction {
    /// Do nothing this tick.
    NoOp,
    /// Consume food on current cell.
    Eat,
    /// Move one step in the given direction.
    Move(Direction),
    /// Attempt to spawn offspring in the given direction, transferring energy.
    Reproduce {
        direction: Direction,
        energy_transfer: f32,
    },
}

impl WorldAction {
    /// Returns true if the action is a NoOp.
    pub fn is_noop(&self) -> bool {
        matches!(self, WorldAction::NoOp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_noop() {
        assert!(WorldAction::NoOp.is_noop());
    }

    #[test]
    fn move_not_noop() {
        assert!(!WorldAction::Move(Direction::N).is_noop());
    }

    #[test]
    fn reproduce_fields_accessible() {
        let action = WorldAction::Reproduce {
            direction: Direction::SE,
            energy_transfer: 10.0,
        };
        if let WorldAction::Reproduce { direction, energy_transfer } = action {
            assert_eq!(direction, Direction::SE);
            assert!((energy_transfer - 10.0).abs() < f32::EPSILON);
        } else {
            panic!("expected Reproduce variant");
        }
    }

    #[test]
    fn world_action_serde_roundtrip() {
        let action = WorldAction::Move(Direction::W);
        let json = serde_json::to_string(&action).unwrap();
        let a2: WorldAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, a2);
    }

    #[test]
    fn all_variants_constructible() {
        let _noop = WorldAction::NoOp;
        let _eat = WorldAction::Eat;
        let _mv = WorldAction::Move(Direction::N);
        let _rep = WorldAction::Reproduce { direction: Direction::S, energy_transfer: 5.0 };
    }
}
```

**Step 2: Run tests**

```bash
cd v3 && cargo test contracts::actions -- --nocapture 2>&1 | tail -10
```

Expected: PASS (5 tests).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/contracts/actions.rs
git commit -m "feat(v3-core): add WorldAction enum (NoOp, Eat, Move, Reproduce)"
```

---

### Task 7: InputReference types

**Files:**
- Create: `v3/crates/v3-core/src/contracts/inputs.rs`

**Spec reference:** `docs/reference/v3-sensor-spec.md`, `docs/reference/v3-mesh-execution-spec.md`.

**Step 1: Read the sensor spec first**

```bash
cat docs/reference/v3-sensor-spec.md | head -120
```

**Step 2: Implement**

```rust
use crate::contracts::Direction;

/// Which world input value to read for a given input slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorldInputKey {
    /// Food density on current cell, normalized to [0.0, 1.0].
    FoodHere,
    /// Food density on neighbor cell in given direction, normalized to [0.0, 1.0].
    NeighborCellFood(Direction),
    /// Whether neighbor cell has a barrier (1.0) or not (0.0).
    NeighborCellBarrier(Direction),
    /// Whether neighbor cell is occupied by another creature (1.0) or not (0.0).
    NeighborCellOccupied(Direction),
}

/// Static (snapshot-assembled) introspection values for the creature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StaticIntrospectionKey {
    /// Number of generations from the founder.
    Generation,
    /// Age in ticks.
    AgeTicks,
}

/// Dynamic (live) introspection values — resolved at input read time, not snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicIntrospectionKey {
    /// Current energy level.
    EnergyCurrent,
    /// Total energy consumed by Eat actions this tick.
    EnergyConsumedThisTick,
}

/// A reference to a specific input source for a node input slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InputReference {
    /// World state input (spatial sensor).
    World(WorldInputKey),
    /// Static introspection (assembled at tick start).
    StaticIntrospection(StaticIntrospectionKey),
    /// Dynamic introspection (live at read time).
    DynamicIntrospection(DynamicIntrospectionKey),
    /// Output slot from the upstream node in the mesh chain.
    UpstreamSlot(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_input_keys_constructible() {
        let _food = WorldInputKey::FoodHere;
        let _nfood = WorldInputKey::NeighborCellFood(Direction::N);
        let _barrier = WorldInputKey::NeighborCellBarrier(Direction::SE);
        let _occ = WorldInputKey::NeighborCellOccupied(Direction::W);
    }

    #[test]
    fn input_reference_upstream_slot() {
        let r = InputReference::UpstreamSlot(3);
        if let InputReference::UpstreamSlot(idx) = r {
            assert_eq!(idx, 3);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn input_reference_serde_roundtrip() {
        let refs = vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::UpstreamSlot(0),
        ];
        for r in refs {
            let json = serde_json::to_string(&r).unwrap();
            let r2: InputReference = serde_json::from_str(&json).unwrap();
            assert_eq!(r, r2);
        }
    }

    #[test]
    fn all_eight_directions_covered_for_neighbor_inputs() {
        for dir in Direction::ALL {
            let _food = InputReference::World(WorldInputKey::NeighborCellFood(dir));
            let _barrier = InputReference::World(WorldInputKey::NeighborCellBarrier(dir));
            let _occ = InputReference::World(WorldInputKey::NeighborCellOccupied(dir));
        }
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test contracts::inputs -- --nocapture 2>&1 | tail -10
```

Expected: PASS (4 tests).

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/contracts/inputs.rs
git commit -m "feat(v3-core): add InputReference, WorldInputKey, and introspection key types"
```

---

### Task 8: SimulationConfig with defaults

**Files:**
- Create: `v3/crates/v3-core/src/config/simulation.rs`

**Spec references:** `docs/reference/v3-runtime-config-spec.md`, `docs/reference/v3-world-grid-spec.md`.

**Step 1: Read the config spec values** (already done above; key defaults below)

World: width=400, height=400, edge_mode=wrap, food.growth_rate=0.02, food.initial_density=80, food.initial_coverage=0.3
Energy lifecycle: initial_energy=20.0, max_energy=100.0, energy_decay_per_tick=0.2, min_reproduce_energy=24.0, default_offspring_energy=20.0
Energy costs: move_cost=0.2, eat_cost=0.0, noop_cost=0.0, reproduce_cost=2.0, eat_reward_per_food=1.0
Runtime: max_mesh_hops=128, max_vm_steps=1024, max_graph_relax_iters=4, graph_convergence_epsilon=1e-3, graph_convergence_stable_passes=1, graph_node_base_cost=1.0, vm.opcode_cost_multiplier=1.0
Mutation: mutation_probability=0.01, per_birth_mutation_events_min=1, per_birth_mutation_events_max=4
Population: initial_creatures=50, max_creatures=1000

**Step 2: Implement**

```rust
/// Edge mode for the world grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorldEdgeMode {
    Wrap,
    Bounded,
}

impl Default for WorldEdgeMode {
    fn default() -> Self {
        WorldEdgeMode::Wrap
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldFoodConfig {
    pub growth_rate: f32,
    pub initial_density: u8,
    pub initial_coverage: f32,
}

impl Default for WorldFoodConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.02,
            initial_density: 80,
            initial_coverage: 0.3,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    pub food: WorldFoodConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 400,
            edge_mode: WorldEdgeMode::default(),
            food: WorldFoodConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyLifecycleConfig {
    pub initial_energy: f32,
    pub max_energy: f32,
    pub energy_decay_per_tick: f32,
    pub min_reproduce_energy: f32,
    pub default_offspring_energy: f32,
}

impl Default for EnergyLifecycleConfig {
    fn default() -> Self {
        Self {
            initial_energy: 20.0,
            max_energy: 100.0,
            energy_decay_per_tick: 0.2,
            min_reproduce_energy: 24.0,
            default_offspring_energy: 20.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyCostsConfig {
    pub move_cost: f32,
    pub eat_cost: f32,
    pub noop_cost: f32,
    pub reproduce_cost: f32,
    pub eat_reward_per_food: f32,
}

impl Default for EnergyCostsConfig {
    fn default() -> Self {
        Self {
            move_cost: 0.2,
            eat_cost: 0.0,
            noop_cost: 0.0,
            reproduce_cost: 2.0,
            eat_reward_per_food: 1.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycleConfig,
    pub costs: EnergyCostsConfig,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            lifecycle: EnergyLifecycleConfig::default(),
            costs: EnergyCostsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmRuntimeConfig {
    pub opcode_cost_multiplier: f32,
}

impl Default for VmRuntimeConfig {
    fn default() -> Self {
        Self {
            opcode_cost_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    pub max_mesh_hops: u32,
    pub max_vm_steps: u32,
    pub max_graph_relax_iters: u32,
    pub graph_convergence_epsilon: f32,
    pub graph_convergence_stable_passes: u32,
    pub graph_node_base_cost: f32,
    pub vm: VmRuntimeConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_mesh_hops: 128,
            max_vm_steps: 1024,
            max_graph_relax_iters: 4,
            graph_convergence_epsilon: 1e-3,
            graph_convergence_stable_passes: 1,
            graph_node_base_cost: 1.0,
            vm: VmRuntimeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutationConfig {
    pub mutation_probability: f64,
    pub per_birth_mutation_events_min: u32,
    pub per_birth_mutation_events_max: u32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_probability: 0.01,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 4,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PopulationConfig {
    pub initial_creatures: u32,
    pub max_creatures: u32,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            initial_creatures: 50,
            max_creatures: 1000,
        }
    }
}

/// Full simulation configuration.
/// All fields have spec-defined defaults.
/// Use `SimulationConfig::default()` to get canonical defaults.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SimulationConfig {
    pub world: WorldConfig,
    pub energy: EnergyConfig,
    pub runtime: RuntimeConfig,
    pub mutation: MutationConfig,
    pub population: PopulationConfig,
}

impl SimulationConfig {
    /// Apply normalization/fallback for out-of-range values.
    /// Called after deserialization or programmatic construction.
    pub fn normalize(&mut self) {
        let w = &mut self.world;
        if w.width == 0 { w.width = 400; }
        if w.height == 0 { w.height = 400; }
        w.food.growth_rate = normalize_f32_clamp(w.food.growth_rate, 0.0, 1.0, 0.02);
        w.food.initial_coverage = normalize_f32_clamp(w.food.initial_coverage, 0.0, 1.0, 0.3);

        let el = &mut self.energy.lifecycle;
        el.initial_energy = normalize_f32_finite_nonneg(el.initial_energy, 20.0);
        el.max_energy = normalize_f32_finite_min(el.max_energy, 1.0, 100.0);
        el.energy_decay_per_tick = normalize_f32_finite_nonneg(el.energy_decay_per_tick, 0.2);
        el.min_reproduce_energy = normalize_f32_finite_nonneg(el.min_reproduce_energy, 24.0);
        el.default_offspring_energy = normalize_f32_finite_nonneg(el.default_offspring_energy, 20.0);

        let ec = &mut self.energy.costs;
        ec.move_cost = normalize_f32_finite_nonneg(ec.move_cost, 0.2);
        ec.eat_cost = normalize_f32_finite_nonneg(ec.eat_cost, 0.0);
        ec.noop_cost = normalize_f32_finite_nonneg(ec.noop_cost, 0.0);
        ec.reproduce_cost = normalize_f32_finite_nonneg(ec.reproduce_cost, 2.0);
        ec.eat_reward_per_food = normalize_f32_finite_nonneg(ec.eat_reward_per_food, 1.0);

        let rt = &mut self.runtime;
        if rt.max_mesh_hops < 1 { rt.max_mesh_hops = 128; }
        if rt.max_vm_steps < 1 { rt.max_vm_steps = 1024; }
        if rt.max_graph_relax_iters < 1 { rt.max_graph_relax_iters = 4; }
        rt.graph_convergence_epsilon = normalize_f32_nonneg(rt.graph_convergence_epsilon, 1e-3);
        if rt.graph_convergence_stable_passes < 1 { rt.graph_convergence_stable_passes = 1; }
        rt.graph_node_base_cost = normalize_f32_nonneg(rt.graph_node_base_cost, 1.0);
        rt.vm.opcode_cost_multiplier = normalize_f32_finite_nonneg(rt.vm.opcode_cost_multiplier, 1.0);

        let m = &mut self.mutation;
        m.mutation_probability = m.mutation_probability.clamp(0.0, 1.0);
        if m.per_birth_mutation_events_min < 1 { m.per_birth_mutation_events_min = 1; }
        if m.per_birth_mutation_events_max < m.per_birth_mutation_events_min {
            m.per_birth_mutation_events_max = m.per_birth_mutation_events_min;
        }

        let p = &mut self.population;
        if p.initial_creatures < 1 { p.initial_creatures = 50; }
        if p.max_creatures < p.initial_creatures { p.max_creatures = 1000; }
    }
}

fn normalize_f32_clamp(v: f32, lo: f32, hi: f32, fallback: f32) -> f32 {
    if v.is_finite() { v.clamp(lo, hi) } else { fallback }
}

fn normalize_f32_finite_nonneg(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v >= 0.0 { v } else { fallback }
}

fn normalize_f32_finite_min(v: f32, min: f32, fallback: f32) -> f32 {
    if v.is_finite() && v >= min { v } else { fallback }
}

fn normalize_f32_nonneg(v: f32, fallback: f32) -> f32 {
    if v >= 0.0 { v } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_spec() {
        let cfg = SimulationConfig::default();
        assert_eq!(cfg.world.width, 400);
        assert_eq!(cfg.world.height, 400);
        assert!(matches!(cfg.world.edge_mode, WorldEdgeMode::Wrap));
        assert!((cfg.world.food.growth_rate - 0.02).abs() < 1e-6);
        assert_eq!(cfg.world.food.initial_density, 80);
        assert!((cfg.world.food.initial_coverage - 0.3).abs() < 1e-6);

        assert!((cfg.energy.lifecycle.initial_energy - 20.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.max_energy - 100.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.energy_decay_per_tick - 0.2).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.min_reproduce_energy - 24.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.default_offspring_energy - 20.0).abs() < 1e-6);

        assert!((cfg.energy.costs.move_cost - 0.2).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.noop_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.reproduce_cost - 2.0).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_reward_per_food - 1.0).abs() < 1e-6);

        assert_eq!(cfg.runtime.max_mesh_hops, 128);
        assert_eq!(cfg.runtime.max_vm_steps, 1024);
        assert_eq!(cfg.runtime.max_graph_relax_iters, 4);
        assert!((cfg.runtime.graph_convergence_epsilon - 1e-3).abs() < 1e-6);
        assert_eq!(cfg.runtime.graph_convergence_stable_passes, 1);
        assert!((cfg.runtime.graph_node_base_cost - 1.0).abs() < 1e-6);
        assert!((cfg.runtime.vm.opcode_cost_multiplier - 1.0).abs() < 1e-6);

        assert!((cfg.mutation.mutation_probability - 0.01).abs() < 1e-9);
        assert_eq!(cfg.mutation.per_birth_mutation_events_min, 1);
        assert_eq!(cfg.mutation.per_birth_mutation_events_max, 4);

        assert_eq!(cfg.population.initial_creatures, 50);
        assert_eq!(cfg.population.max_creatures, 1000);
    }

    #[test]
    fn normalize_clamps_invalid_f32_to_fallback() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = f32::NAN;
        cfg.energy.lifecycle.max_energy = -1.0;
        cfg.runtime.max_mesh_hops = 0;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 0.02).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.max_energy - 100.0).abs() < 1e-6);
        assert_eq!(cfg.runtime.max_mesh_hops, 128);
    }

    #[test]
    fn normalize_clamps_growth_rate_to_range() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn config_serde_roundtrip() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.world.width, cfg2.world.width);
        assert_eq!(cfg.runtime.max_mesh_hops, cfg2.runtime.max_mesh_hops);
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test config::simulation -- --nocapture 2>&1 | tail -15
```

Expected: PASS (4 tests).

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/config/simulation.rs
git commit -m "feat(v3-core): add SimulationConfig with all spec-defined defaults and normalization"
```

---

### Task 9: Grid<T> generic grid

**Files:**
- Create: `v3/crates/v3-core/src/kernel/grid.rs`

**Step 1: Implement**

```rust
/// A flat row-major 2D grid of values.
/// Width and height are fixed at construction time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Grid<T> {
    width: u16,
    height: u16,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    pub fn new(width: u16, height: u16, fill: T) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![fill; size],
        }
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    fn idx(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn get(&self, x: u16, y: u16) -> &T {
        &self.cells[self.idx(x, y)]
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut T {
        let idx = self.idx(x, y);
        &mut self.cells[idx]
    }

    pub fn set(&mut self, x: u16, y: u16, value: T) {
        let idx = self.idx(x, y);
        self.cells[idx] = value;
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, u16, &T)> {
        self.cells.iter().enumerate().map(move |(i, v)| {
            let x = (i % self.width as usize) as u16;
            let y = (i / self.width as usize) as u16;
            (x, y, v)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_get_set_roundtrip() {
        let mut g: Grid<u8> = Grid::new(4, 3, 0);
        g.set(2, 1, 42);
        assert_eq!(*g.get(2, 1), 42);
        assert_eq!(*g.get(0, 0), 0);
    }

    #[test]
    fn grid_dimensions() {
        let g: Grid<bool> = Grid::new(10, 20, false);
        assert_eq!(g.width(), 10);
        assert_eq!(g.height(), 20);
    }

    #[test]
    fn grid_size_matches_width_height() {
        let g: Grid<u8> = Grid::new(5, 3, 0);
        let count = g.iter().count();
        assert_eq!(count, 15);
    }

    #[test]
    fn grid_iter_visits_all_positions() {
        let w = 3u16;
        let h = 2u16;
        let g: Grid<u8> = Grid::new(w, h, 0);
        let positions: Vec<(u16, u16)> = g.iter().map(|(x, y, _)| (x, y)).collect();
        assert_eq!(positions.len(), (w * h) as usize);
        // Row 0: (0,0),(1,0),(2,0); Row 1: (0,1),(1,1),(2,1)
        assert!(positions.contains(&(0, 0)));
        assert!(positions.contains(&(2, 1)));
    }

    #[test]
    fn grid_fill_initializes_all_cells() {
        let g: Grid<i32> = Grid::new(3, 3, 99);
        for (_, _, v) in g.iter() {
            assert_eq!(*v, 99);
        }
    }
}
```

**Step 2: Run tests**

```bash
cd v3 && cargo test kernel::grid -- --nocapture 2>&1 | tail -10
```

Expected: PASS (5 tests).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/kernel/grid.rs
git commit -m "feat(v3-core): add generic Grid<T> with row-major storage"
```

---

### Task 10: WorldState with food and occupancy

**Files:**
- Create: `v3/crates/v3-core/src/kernel/world.rs`

**Spec reference:** `docs/reference/v3-world-grid-spec.md` Sections 2, 5–7.

**Step 1: Implement**

```rust
use rand::Rng;
use crate::contracts::{CreatureId, Direction, Position};
use crate::config::{SimulationConfig, WorldEdgeMode};
use crate::kernel::Grid;

/// Central world state: food, barriers, and creature occupancy.
pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    food_density: Grid<u8>,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
}

impl WorldState {
    /// Create a new empty world (no food, no barriers, no creatures).
    pub fn new(width: u16, height: u16, edge_mode: WorldEdgeMode) -> Self {
        Self {
            width,
            height,
            edge_mode,
            food_density: Grid::new(width, height, 0),
            barriers: Grid::new(width, height, false),
            creature_at: Grid::new(width, height, None),
        }
    }

    /// Seed initial food distribution.
    /// Each non-barrier cell independently rolls `initial_coverage`;
    /// on success, food is set to `initial_density`.
    pub fn seed_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        let coverage = config.world.food.initial_coverage;
        let density = config.world.food.initial_density;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.barriers.get(x, y) {
                    continue;
                }
                if rng.gen::<f32>() < coverage {
                    self.food_density.set(x, y, density);
                }
            }
        }
    }

    /// Grow food: each non-barrier cell with food < 255 independently rolls
    /// `growth_rate`; on success, food increases by 1 (saturating).
    pub fn grow_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        let rate = config.world.food.growth_rate;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.barriers.get(x, y) {
                    continue;
                }
                if rng.gen::<f32>() < rate {
                    let current = *self.food_density.get(x, y);
                    self.food_density.set(x, y, current.saturating_add(1));
                }
            }
        }
    }

    /// Consume all food on a cell. Returns the amount consumed.
    pub fn consume_food(&mut self, pos: Position) -> u8 {
        let amount = *self.food_density.get(pos.x, pos.y);
        self.food_density.set(pos.x, pos.y, 0);
        amount
    }

    /// Get food density at a position.
    pub fn food_at(&self, pos: Position) -> u8 {
        *self.food_density.get(pos.x, pos.y)
    }

    /// Get whether a cell has a barrier.
    pub fn is_barrier(&self, pos: Position) -> bool {
        *self.barriers.get(pos.x, pos.y)
    }

    /// Set a barrier cell.
    pub fn set_barrier(&mut self, pos: Position, value: bool) {
        self.barriers.set(pos.x, pos.y, value);
    }

    /// Get creature occupying a cell.
    pub fn creature_at(&self, pos: Position) -> Option<CreatureId> {
        *self.creature_at.get(pos.x, pos.y)
    }

    /// Place a creature on a cell. Caller must ensure cell is valid.
    pub fn place_creature(&mut self, pos: Position, id: CreatureId) {
        self.creature_at.set(pos.x, pos.y, Some(id));
    }

    /// Remove creature from a cell.
    pub fn remove_creature(&mut self, pos: Position) {
        self.creature_at.set(pos.x, pos.y, None);
    }

    /// Resolve a neighbor position applying edge mode.
    /// Returns None for bounded mode out-of-bounds.
    pub fn resolve_neighbor(&self, pos: Position, dir: Direction) -> Option<Position> {
        let (dx, dy) = dir.delta();
        match self.edge_mode {
            WorldEdgeMode::Wrap => pos.neighbor_wrap(dx, dy, self.width, self.height),
            WorldEdgeMode::Bounded => pos.neighbor_bounded(dx, dy, self.width, self.height),
        }
    }

    /// Whether a cell is a valid move/spawn target.
    /// Position must already be resolved (in bounds).
    pub fn is_valid_target_cell(&self, pos: Position) -> bool {
        !self.is_barrier(pos) && self.creature_at(pos).is_none()
    }

    /// Total food across all cells (for testing).
    pub fn total_food(&self) -> u64 {
        let mut sum = 0u64;
        for y in 0..self.height {
            for x in 0..self.width {
                sum += *self.food_density.get(x, y) as u64;
            }
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn make_config() -> SimulationConfig {
        SimulationConfig::default()
    }

    fn small_world() -> WorldState {
        WorldState::new(10, 10, WorldEdgeMode::Wrap)
    }

    #[test]
    fn new_world_has_no_food_no_barriers_no_creatures() {
        let w = small_world();
        assert_eq!(w.total_food(), 0);
        for y in 0..10u16 {
            for x in 0..10u16 {
                let pos = Position::new(x, y);
                assert!(!w.is_barrier(pos));
                assert!(w.creature_at(pos).is_none());
            }
        }
    }

    #[test]
    fn seed_food_places_food_on_non_barrier_cells() {
        let mut w = small_world();
        let mut rng = SmallRng::seed_from_u64(42);
        let cfg = make_config();
        w.seed_food(&mut rng, &cfg);
        // With coverage=0.3 and 100 cells, expect some food but not all
        assert!(w.total_food() > 0, "expect some food after seeding");
    }

    #[test]
    fn seed_food_respects_initial_density() {
        let mut w = small_world();
        let mut rng = SmallRng::seed_from_u64(42);
        let cfg = make_config();
        w.seed_food(&mut rng, &cfg);
        // Every seeded cell should have exactly initial_density (80) or 0
        for y in 0..10u16 {
            for x in 0..10u16 {
                let f = w.food_at(Position::new(x, y));
                assert!(f == 0 || f == cfg.world.food.initial_density);
            }
        }
    }

    #[test]
    fn seed_food_skips_barrier_cells() {
        let mut w = small_world();
        let barrier_pos = Position::new(5, 5);
        w.set_barrier(barrier_pos, true);
        let mut rng = SmallRng::seed_from_u64(0);
        // Force all cells to seed with coverage = 1.0
        let mut cfg = make_config();
        cfg.world.food.initial_coverage = 1.0;
        w.seed_food(&mut rng, &cfg);
        assert_eq!(w.food_at(barrier_pos), 0, "barrier cell must not have food");
    }

    #[test]
    fn grow_food_increases_food_over_ticks() {
        let mut w = WorldState::new(4, 4, WorldEdgeMode::Wrap);
        // Pre-fill some food (not at max) to measure growth
        for y in 0..4u16 {
            for x in 0..4u16 {
                w.food_density.set(x, y, 10);
            }
        }
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut cfg = make_config();
        cfg.world.food.growth_rate = 1.0; // guaranteed growth
        w.grow_food(&mut rng, &cfg);
        let after = w.total_food();
        assert!(after > before, "food must grow with rate=1.0");
    }

    #[test]
    fn grow_food_saturates_at_255() {
        let mut w = WorldState::new(1, 1, WorldEdgeMode::Wrap);
        w.food_density.set(0, 0, 255);
        let mut rng = SmallRng::seed_from_u64(0);
        let mut cfg = make_config();
        cfg.world.food.growth_rate = 1.0;
        w.grow_food(&mut rng, &cfg);
        assert_eq!(w.food_at(Position::new(0, 0)), 255, "must saturate at 255");
    }

    #[test]
    fn consume_food_returns_amount_and_clears_cell() {
        let mut w = small_world();
        let pos = Position::new(3, 3);
        w.food_density.set(3, 3, 100);
        let consumed = w.consume_food(pos);
        assert_eq!(consumed, 100);
        assert_eq!(w.food_at(pos), 0);
    }

    #[test]
    fn resolve_neighbor_wrap_crosses_edge() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Wrap);
        let pos = Position::new(0, 0);
        let neighbor = w.resolve_neighbor(pos, Direction::NW).unwrap();
        assert_eq!(neighbor, Position::new(9, 9));
    }

    #[test]
    fn resolve_neighbor_bounded_returns_none_off_edge() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Bounded);
        let pos = Position::new(0, 0);
        assert!(w.resolve_neighbor(pos, Direction::N).is_none());
        assert!(w.resolve_neighbor(pos, Direction::W).is_none());
    }

    #[test]
    fn single_occupancy_invariant() {
        use slotmap::SlotMap;
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id1 = sm.insert(());
        let id2 = sm.insert(());
        let mut w = small_world();
        let pos = Position::new(2, 2);
        w.place_creature(pos, id1);
        assert_eq!(w.creature_at(pos), Some(id1));
        // A second placement overwrites (caller ensures validity)
        w.place_creature(pos, id2);
        assert_eq!(w.creature_at(pos), Some(id2));
        w.remove_creature(pos);
        assert_eq!(w.creature_at(pos), None);
    }

    #[test]
    fn is_valid_target_cell_requires_no_barrier_and_no_creature() {
        use slotmap::SlotMap;
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = small_world();
        let pos = Position::new(4, 4);
        // Empty cell is valid
        assert!(w.is_valid_target_cell(pos));
        // Barrier makes it invalid
        w.set_barrier(pos, true);
        assert!(!w.is_valid_target_cell(pos));
        w.set_barrier(pos, false);
        // Occupied makes it invalid
        w.place_creature(pos, id);
        assert!(!w.is_valid_target_cell(pos));
    }

    #[test]
    fn seed_food_deterministic_with_same_seed() {
        let mut w1 = small_world();
        let mut w2 = small_world();
        let cfg = make_config();
        w1.seed_food(&mut SmallRng::seed_from_u64(99), &cfg);
        w2.seed_food(&mut SmallRng::seed_from_u64(99), &cfg);
        assert_eq!(w1.total_food(), w2.total_food());
        for y in 0..10u16 {
            for x in 0..10u16 {
                assert_eq!(
                    w1.food_at(Position::new(x, y)),
                    w2.food_at(Position::new(x, y))
                );
            }
        }
    }
}
```

**Step 2: Run tests**

```bash
cd v3 && cargo test kernel::world -- --nocapture 2>&1 | tail -20
```

Expected: PASS (12 tests).

**Step 3: Commit**

```bash
git add v3/crates/v3-core/src/kernel/world.rs
git commit -m "feat(v3-core): add WorldState with food seeding, growth, consumption, and occupancy"
```

---

### Task 11: Wire all modules together and run full quality checks

**Files:**
- Verify: `v3/crates/v3-core/src/lib.rs`
- Verify: `v3/crates/v3-core/src/contracts/mod.rs`
- Verify: `v3/crates/v3-core/src/config/mod.rs`
- Verify: `v3/crates/v3-core/src/kernel/mod.rs`

**Step 1: Ensure all stub placeholder files are created (for modules not yet implemented)**

Tasks 3–10 create these files in each module. Make sure `mod.rs` files reference them correctly.

For `contracts/mod.rs` — re-export all types:
```rust
mod direction;
mod position;
mod ids;
mod actions;
mod inputs;

pub use direction::Direction;
pub use position::Position;
pub use ids::{CreatureId, NodeId};
pub use actions::WorldAction;
pub use inputs::{InputReference, WorldInputKey, StaticIntrospectionKey, DynamicIntrospectionKey};
```

For `kernel/mod.rs`:
```rust
mod grid;
mod world;

pub use grid::Grid;
pub use world::WorldState;
```

**Step 2: Cargo fmt**

```bash
cd v3 && cargo fmt --all
```

**Step 3: Cargo clippy — no warnings**

```bash
cd v3 && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```

Expected: 0 warnings, 0 errors.

**Step 4: Run all tests**

```bash
cd v3 && cargo test --workspace 2>&1 | tail -20
```

Expected: all tests pass.

**Step 5: Run plan harness (from repo root)**

```bash
scripts/check-plan-harness.sh --mode strict 2>&1 | tail -5
```

Expected: `violations=0`.

**Step 6: Run architecture harness (from repo root)**

```bash
scripts/check-architecture-harness.sh --mode warn 2>&1 | tail -5
```

Expected: 0 violations (harness only checks `crates/`, not `v3/crates/`).

**Step 7: Final commit**

```bash
git add -A v3/
git commit -m "feat(v3-core): wire all stage-1 modules; contracts, config, kernel complete"
```

---

## Verification Checklist

Before claiming Stage 1 complete:

- [x] `AGENTS.md` exists at repo root and includes rust-skills imperative for v3
- [x] `v3/crates/v3-core/src/contracts/` has: `direction.rs`, `position.rs`, `ids.rs`, `actions.rs`, `inputs.rs`, `mod.rs`
- [x] `v3/crates/v3-core/src/config/` has: `simulation.rs`, `mod.rs`
- [x] `v3/crates/v3-core/src/kernel/` has: `grid.rs`, `world.rs`, `mod.rs`
- [x] `cd v3 && cargo test --workspace` → all green
- [x] `cd v3 && cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings
- [x] `cd v3 && cargo fmt --all --check` → no diffs
- [x] `scripts/check-plan-harness.sh --mode strict` → violations=0
- [x] `scripts/check-architecture-harness.sh --mode warn` → no new violations
- [x] All `SimulationConfig::default()` fields match spec exactly (covered by `default_config_matches_spec` test)
- [x] `WorldState` food seeding is deterministic (covered by `seed_food_deterministic_with_same_seed` test)

## Reconciliation Snapshot (2026-02-22)

- verified: 11
- partial: 0
- missing: 0
- conflict: 0
- evidence matrix: `docs/plans/archive/reconciliation/2026-02-22-v3-stage-1-evidence-matrix.md`

---

## Risks and Rollback

- **Risk:** `v3/` is a nested workspace and root `cargo test` won't pick it up. Mitigation: all v3 test commands run as `cd v3 && cargo ...`.
- **Risk:** The architecture harness checks `crates/*/Cargo.toml` — v3 crates won't be checked. Mitigation: Rely on clippy/fmt for v3 crate quality, plan to extend harness in Stage 7 if needed.
- **Risk:** The `slotmap` crate version in `v3/Cargo.toml` may conflict with root workspace. Mitigation: v3 workspace is independent, no shared lockfile with root.

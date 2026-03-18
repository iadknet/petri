# Stage 1: Backend Core — Companion Plan

**Parent plan:** `2026-03-18-food-fertility-layer.md`
**Spec:** `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md`

> **For agentic workers:** Use `rust-skills` for all Rust code in this stage. TDD: write failing test first, then implement.

---

## Task 1: Add `noise` Crate Dependency

**Files:**
- Modify: `v3/crates/v3-core/Cargo.toml`

- [ ] **Step 1:** Add `noise` to `[dependencies]` in `v3/crates/v3-core/Cargo.toml`:

```toml
noise = "0.9"
```

- [ ] **Step 2:** Verify it compiles

Run: `cd v3 && cargo check -p v3-core`
Expected: compiles without errors

- [ ] **Step 3:** Commit

```bash
git add v3/crates/v3-core/Cargo.toml
git commit -m "chore: add noise crate dependency for Fbm fertility generation"
```

---

## Task 2: Config Structs — FoodResourceConfig, FertilityConfig, AnnealingConfig

**Files:**
- Modify: `v3/crates/v3-core/src/config/simulation.rs`

This task renames `WorldFoodConfig` to `FoodResourceConfig` and adds the new fertility and annealing config structs. All new structs follow `#[serde(deny_unknown_fields)]` convention.

- [ ] **Step 1:** Write failing test for new config defaults

Add to the `#[cfg(test)] mod tests` at the bottom of `simulation.rs`:

```rust
#[test]
fn food_resource_config_default_has_fertility_disabled() {
    let config = FoodResourceConfig::default();
    assert!(!config.fertility.enabled);
    assert_eq!(config.fertility.min_fertility, 0.0);
    assert_eq!(config.fertility.max_fertility, 2.0);
    assert!(!config.annealing.enabled);
    assert_eq!(config.annealing.ramp_ticks, 5000);
}

#[test]
fn fertility_config_deserializes_with_defaults_when_omitted() {
    let json = r#"{"growth_rate":0.05,"initial_density":1.0,"initial_coverage":0.15,"spread_threshold_ratio":0.8,"spread_density_ratio":0.25,"recovery_spawn_rate":0.01,"recovery_floor_ratio":0.01,"max_density":1.0}"#;
    let config: FoodResourceConfig = serde_json::from_str(json).unwrap();
    assert!(!config.fertility.enabled);
    assert!(!config.annealing.enabled);
}
```

- [ ] **Step 2:** Run test to verify it fails

Run: `cd v3 && cargo test -p v3-core -- food_resource_config_default --no-capture`
Expected: FAIL — `FoodResourceConfig` not found

- [ ] **Step 3:** Implement config structs

In `v3/crates/v3-core/src/config/simulation.rs`:

1. Rename `WorldFoodConfig` to `FoodResourceConfig` (update all references)
2. Add `FertilityConfig`, `FertilityLayer`, `FertilityAlgorithm`, `AnnealingConfig` structs
3. Add `#[serde(default)] pub fertility: FertilityConfig` and `#[serde(default)] pub annealing: AnnealingConfig` to `FoodResourceConfig`

```rust
/// Food resource config. Replaces WorldFoodConfig.
/// Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodResourceConfig {
    pub growth_rate: f32,
    pub initial_density: f32,
    pub initial_coverage: f32,
    pub spread_threshold_ratio: f32,
    pub spread_density_ratio: f32,
    pub recovery_spawn_rate: f32,
    pub recovery_floor_ratio: f32,
    pub max_density: f32,
    #[serde(default)]
    pub fertility: FertilityConfig,
    #[serde(default)]
    pub annealing: AnnealingConfig,
}

// Keep the same Default impl values as the old WorldFoodConfig

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityConfig {
    pub enabled: bool,
    pub min_fertility: f32,
    pub max_fertility: f32,
    pub layers: Vec<FertilityLayer>,
}

impl Default for FertilityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_fertility: 0.0,
            max_fertility: 2.0,
            layers: vec![FertilityLayer::default()],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityLayer {
    pub algorithm: FertilityAlgorithm,
    pub weight: f32,
}

impl Default for FertilityLayer {
    fn default() -> Self {
        Self {
            algorithm: FertilityAlgorithm::PoissonBlobs {
                blob_count: 8,
                min_radius: 5.0,
                max_radius: 15.0,
                falloff: 0.5,
                seed: None,
            },
            weight: 1.0,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FertilityAlgorithm {
    Uniform { value: f32 },
    Fbm {
        octaves: u32,
        frequency: f32,
        lacunarity: f32,
        persistence: f32,
        seed: Option<u64>,
    },
    PoissonBlobs {
        blob_count: u32,
        min_radius: f32,
        max_radius: f32,
        falloff: f32,
        seed: Option<u64>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnealingConfig {
    pub enabled: bool,
    pub ramp_ticks: u64,
    pub initial_min_fertility: f32,
    pub initial_max_fertility: f32,
}

impl Default for AnnealingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ramp_ticks: 5000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        }
    }
}
```

4. Update `WorldConfig` to use `FoodResourceConfig` instead of `WorldFoodConfig`
5. Fix all compile errors from the rename (search for `WorldFoodConfig` across the workspace)

- [ ] **Step 4:** Run tests to verify they pass

Run: `cd v3 && cargo test -p v3-core -- food_resource_config --no-capture`
Expected: PASS

- [ ] **Step 5:** Run full test suite to check rename didn't break anything

Run: `cd v3 && cargo test --workspace`
Expected: all pass

- [ ] **Step 6:** Commit

```bash
git add -A
git commit -m "refactor: rename WorldFoodConfig to FoodResourceConfig, add fertility and annealing config structs"
```

---

## Task 3: FoodResource Struct and Basic Methods

**Files:**
- Create: `v3/crates/v3-core/src/kernel/food_resource.rs`
- Modify: `v3/crates/v3-core/src/kernel/mod.rs`

- [ ] **Step 1:** Write failing test for FoodResource basic operations

At the bottom of `food_resource.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::simulation::FoodResourceConfig;

    fn test_config() -> FoodResourceConfig {
        FoodResourceConfig::default()
    }

    #[test]
    fn food_at_returns_density() {
        let mut res = FoodResource::new(4, 4, test_config());
        res.set_food(Position { x: 1, y: 2 }, 0.5);
        assert_eq!(res.food_at(Position { x: 1, y: 2 }), 0.5);
        assert_eq!(res.food_at(Position { x: 0, y: 0 }), 0.0);
    }

    #[test]
    fn consume_clears_cell_and_returns_amount() {
        let mut res = FoodResource::new(4, 4, test_config());
        res.set_food(Position { x: 1, y: 1 }, 0.8);
        let consumed = res.consume(Position { x: 1, y: 1 });
        assert_eq!(consumed, 0.8);
        assert_eq!(res.food_at(Position { x: 1, y: 1 }), 0.0);
    }

    #[test]
    fn fertility_accessor_returns_grid() {
        let res = FoodResource::new(4, 4, test_config());
        assert_eq!(res.fertility().width(), 4);
        assert_eq!(res.fertility().height(), 4);
    }
}
```

- [ ] **Step 2:** Run test to verify it fails

Run: `cd v3 && cargo test -p v3-core -- food_resource::tests --no-capture`
Expected: FAIL — module not found

- [ ] **Step 3:** Implement FoodResource

Create `v3/crates/v3-core/src/kernel/food_resource.rs`:

```rust
use crate::config::simulation::FoodResourceConfig;
use crate::kernel::grid::Grid;
use crate::kernel::Position;

/// Public type with private fields. Encapsulates food density, fertility,
/// config, and growth logic. Transport layer accesses fertility data via
/// the `fertility()` and `config()` accessors.
pub struct FoodResource {
    density: Grid<f32>,
    fertility: Grid<f32>,
    config: FoodResourceConfig,
    growth_scratch: Vec<f32>,
}

impl FoodResource {
    pub fn new(width: u16, height: u16, config: FoodResourceConfig) -> Self {
        Self {
            density: Grid::new(width, height, 0.0),
            fertility: Grid::new(width, height, 0.0),
            config,
            growth_scratch: Vec::new(),
        }
    }

    pub fn food_at(&self, pos: Position) -> f32 {
        *self.density.get(pos.x, pos.y)
    }

    pub fn consume(&mut self, pos: Position) -> f32 {
        let amount = *self.density.get(pos.x, pos.y);
        self.density.set(pos.x, pos.y, 0.0);
        amount
    }

    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.density.set(pos.x, pos.y, value);
    }

    pub fn fertility(&self) -> &Grid<f32> {
        &self.fertility
    }

    pub fn config(&self) -> &FoodResourceConfig {
        &self.config
    }

    pub fn density_grid(&self) -> &Grid<f32> {
        &self.density
    }

    pub fn update_config(&mut self, config: FoodResourceConfig) {
        self.config = config;
    }

    pub fn width(&self) -> u16 {
        self.density.width()
    }

    pub fn height(&self) -> u16 {
        self.density.height()
    }
}
```

Add to `v3/crates/v3-core/src/kernel/mod.rs`:

```rust
pub mod food_resource;
```

- [ ] **Step 4:** Run tests to verify they pass

Run: `cd v3 && cargo test -p v3-core -- food_resource::tests --no-capture`
Expected: PASS

- [ ] **Step 5:** Commit

```bash
git add v3/crates/v3-core/src/kernel/food_resource.rs v3/crates/v3-core/src/kernel/mod.rs
git commit -m "feat: add FoodResource struct with basic food accessors"
```

---

## Task 4: Move Growth Logic into FoodResource

**Files:**
- Modify: `v3/crates/v3-core/src/kernel/food_resource.rs`
- Modify: `v3/crates/v3-core/src/kernel/world.rs`

Move `grow_food()` and `seed_food()` from `WorldState` into `FoodResource`. The FoodResource version takes `barriers` and `tick` as parameters. WorldState delegates.

- [ ] **Step 1:** Write failing test — FoodResource::grow produces same output as old WorldState::grow_food

```rust
#[test]
fn grow_matches_old_behavior_when_fertility_disabled() {
    use crate::config::simulation::SimulationConfig;
    let config = SimulationConfig::default();
    let mut res = FoodResource::new(4, 4, config.world.food.clone());
    let barriers = Grid::new(4, 4, false);
    // Seed some food
    res.set_food(Position { x: 1, y: 1 }, 0.5);
    res.set_food(Position { x: 2, y: 2 }, 0.3);

    // Grow with fertility disabled (default) — should behave identically to old code
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    res.grow(&barriers, 0, &mut rng);

    // Proportional growth: 0.5 + 0.5 * 0.05 = 0.525
    let food_1_1 = res.food_at(Position { x: 1, y: 1 });
    assert!((food_1_1 - 0.525).abs() < 0.001, "expected ~0.525, got {food_1_1}");
}
```

- [ ] **Step 2:** Run test to verify it fails

Run: `cd v3 && cargo test -p v3-core -- grow_matches_old --no-capture`
Expected: FAIL — `grow` method not found

- [ ] **Step 3:** Implement `FoodResource::grow()` and `FoodResource::seed_density()`

Move the logic from `WorldState::grow_food()` (world.rs lines 91-176) into `FoodResource::grow()`. Key changes:
- Read config from `self.config` instead of `&SimulationConfig`
- Accept `barriers: &Grid<bool>` and `tick: u64` as parameters
- When `self.config.fertility.enabled` is false, treat all cells as fertility 1.0 (identity multiplier)
- When enabled, compute effective fertility per cell at growth time

Move `WorldState::seed_food()` (world.rs lines 42-78) into `FoodResource::seed_density()`.

Move `WorldState::total_food()` into `FoodResource::total_food()`.

- [ ] **Step 4:** Run test to verify it passes

Run: `cd v3 && cargo test -p v3-core -- grow_matches_old --no-capture`
Expected: PASS

- [ ] **Step 5:** Commit

```bash
git add v3/crates/v3-core/src/kernel/food_resource.rs
git commit -m "feat: move grow_food and seed_food logic into FoodResource"
```

---

## Task 5: WorldState Integration — Delegates and Call Sites

**Files:**
- Modify: `v3/crates/v3-core/src/kernel/world.rs`
- Modify: `v3/crates/v3-core/src/simulation/tick.rs:96`
- Modify: various call sites

Replace `WorldState` food fields with `food: FoodResource`. Add delegate methods. Update all call sites.

- [ ] **Step 1:** Replace `WorldState` food fields with `FoodResource`

In `world.rs`:
- Remove `food_density: Grid<f32>` and `food_snapshot: Vec<f32>` fields
- Add `food: FoodResource` field (private)
- Add `pub fn food(&self) -> &FoodResource` accessor
- Add delegate methods: `grow_food`, `food_at`, `consume_food`, `set_food`, `total_food`, `seed_food`
- Add `apply_food_config` method
- Update `WorldState::new()` to create `FoodResource`

- [ ] **Step 2:** Update `grow_food` delegate signature

```rust
/// Convenience delegate for single-food model.
/// Will be revised or removed when multi-food is implemented.
pub fn grow_food(&mut self, tick: u64, rng: &mut impl Rng) {
    self.food.grow(&self.barriers, tick, rng);
}
```

- [ ] **Step 3:** Update `run_phase_0` in `tick.rs` line 96

```rust
// before: sim.world.grow_food(&mut sim.rng, &sim.config);
// after:
sim.world.grow_food(sim.tick, &mut sim.rng);
```

- [ ] **Step 4:** Fix all remaining compile errors across the workspace

Search for all references to `food_density`, `food_snapshot`, `WorldFoodConfig`, and old `grow_food` signatures. Update each call site.

Key call sites to check:
- `v3/crates/v3-core/src/simulation/tick.rs` — `run_phase_0`
- `v3/crates/v3-core/src/sensors/static_inputs.rs` — food reads
- `v3/crates/v3-core/src/simulation/actions/mod.rs` — eat action
- `v3/crates/v3-server/src/handlers/lifecycle.rs` — `build_ws_frame` food reads
- `v3/crates/v3-server/src/handlers/status.rs` — config patch handler
- `v3/crates/v3-core/tests/viability.rs` — test fixtures

- [ ] **Step 5:** Add `apply_food_config` for runtime config propagation

```rust
pub fn apply_food_config(&mut self, config: FoodResourceConfig) {
    self.food.update_config(config);
}
```

Update `patch_config()` in `v3/crates/v3-server/src/handlers/status.rs` to call `apply_food_config` after config merge.

- [ ] **Step 6:** Run full workspace tests

Run: `cd v3 && cargo test --workspace`
Expected: all pass — this is a pure refactor with no behavior change

- [ ] **Step 7:** Run viability tests specifically

Run: `cd v3 && cargo test -p v3-core --test viability`
Expected: all pass

- [ ] **Step 8:** Write test for runtime config propagation

```rust
#[test]
fn apply_food_config_updates_food_resource() {
    let mut world = WorldState::new(4, 4, WorldEdgeMode::Wrap);
    let mut config = FoodResourceConfig::default();
    config.growth_rate = 0.99;
    world.apply_food_config(config);
    assert_eq!(world.food().config().growth_rate, 0.99);
}
```

- [ ] **Step 9:** Run and verify

Run: `cd v3 && cargo test -p v3-core -- apply_food_config --no-capture`
Expected: PASS

- [ ] **Step 10:** Commit

```bash
git add -A
git commit -m "refactor: integrate FoodResource into WorldState with delegate methods

Move food density, growth logic, and seeding into FoodResource.
WorldState delegates food_at, consume_food, set_food, grow_food.
Update all call sites across v3-core and v3-server.
Add apply_food_config for runtime config patch propagation."
```

---

## Task 6: Fertility Seeding — Uniform Algorithm

**Files:**
- Create: `v3/crates/v3-core/src/kernel/fertility/mod.rs`
- Create: `v3/crates/v3-core/src/kernel/fertility/uniform.rs`
- Modify: `v3/crates/v3-core/src/kernel/mod.rs`

- [ ] **Step 1:** Write failing test

In `uniform.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_fills_grid_with_clamped_value() {
        let grid = generate_uniform(4, 4, 0.5);
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(*grid.get(x, y), 0.5);
            }
        }
    }

    #[test]
    fn uniform_clamps_to_negative_one_one() {
        let grid = generate_uniform(2, 2, 2.0);
        assert_eq!(*grid.get(0, 0), 1.0);

        let grid = generate_uniform(2, 2, -3.0);
        assert_eq!(*grid.get(0, 0), -1.0);
    }
}
```

- [ ] **Step 2:** Run test to verify it fails

- [ ] **Step 3:** Implement

```rust
use crate::kernel::grid::Grid;

pub fn generate_uniform(width: u16, height: u16, value: f32) -> Grid<f32> {
    let clamped = value.clamp(-1.0, 1.0);
    Grid::new(width, height, clamped)
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add Uniform fertility seeding algorithm"
```

---

## Task 7: Fertility Seeding — Fbm Algorithm

**Files:**
- Create: `v3/crates/v3-core/src/kernel/fertility/fbm.rs`

- [ ] **Step 1:** Write failing tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fbm_output_in_range() {
        let grid = generate_fbm(32, 32, 4, 0.02, 2.0, 0.5, 42);
        for y in 0..32 {
            for x in 0..32 {
                let v = *grid.get(x, y);
                assert!(v >= -1.0 && v <= 1.0, "value {v} out of range at ({x},{y})");
            }
        }
    }

    #[test]
    fn fbm_is_deterministic() {
        let a = generate_fbm(16, 16, 4, 0.02, 2.0, 0.5, 123);
        let b = generate_fbm(16, 16, 4, 0.02, 2.0, 0.5, 123);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(*a.get(x, y), *b.get(x, y));
            }
        }
    }

    #[test]
    fn fbm_different_seeds_differ() {
        let a = generate_fbm(16, 16, 4, 0.02, 2.0, 0.5, 1);
        let b = generate_fbm(16, 16, 4, 0.02, 2.0, 0.5, 2);
        let mut differs = false;
        for y in 0..16 {
            for x in 0..16 {
                if *a.get(x, y) != *b.get(x, y) {
                    differs = true;
                    break;
                }
            }
        }
        assert!(differs, "different seeds should produce different grids");
    }
}
```

- [ ] **Step 2:** Run test to verify it fails
- [ ] **Step 3:** Implement using `noise` crate

```rust
use crate::kernel::grid::Grid;
use noise::{NoiseFn, Fbm, Perlin};

pub fn generate_fbm(
    width: u16, height: u16,
    octaves: u32, frequency: f32,
    lacunarity: f32, persistence: f32,
    seed: u64,
) -> Grid<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed as u32);
    fbm.octaves = octaves as usize;
    fbm.frequency = frequency as f64;
    fbm.lacunarity = lacunarity as f64;
    fbm.persistence = persistence as f64;

    let mut grid = Grid::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            let raw = fbm.get([x as f64, y as f64]) as f32;
            grid.set(x, y, raw.clamp(-1.0, 1.0));
        }
    }
    grid
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add Fbm fertility seeding algorithm using noise crate"
```

---

## Task 8: Fertility Seeding — PoissonBlobs Algorithm

**Files:**
- Create: `v3/crates/v3-core/src/kernel/fertility/poisson_blobs.rs`

- [ ] **Step 1:** Write failing tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn poisson_blobs_output_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let grid = generate_poisson_blobs(32, 32, 5, 3.0, 8.0, 0.5, &mut rng);
        for y in 0..32 {
            for x in 0..32 {
                let v = *grid.get(x, y);
                assert!(v >= -1.0 && v <= 1.0, "value {v} out of range at ({x},{y})");
            }
        }
    }

    #[test]
    fn poisson_blobs_is_deterministic() {
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);
        let a = generate_poisson_blobs(16, 16, 5, 3.0, 8.0, 0.5, &mut rng1);
        let b = generate_poisson_blobs(16, 16, 5, 3.0, 8.0, 0.5, &mut rng2);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(*a.get(x, y), *b.get(x, y));
            }
        }
    }

    #[test]
    fn poisson_blobs_creates_variation() {
        let mut rng = StdRng::seed_from_u64(42);
        let grid = generate_poisson_blobs(32, 32, 5, 3.0, 8.0, 0.5, &mut rng);
        let mut has_high = false;
        let mut has_low = false;
        for y in 0..32 {
            for x in 0..32 {
                let v = *grid.get(x, y);
                if v > 0.5 { has_high = true; }
                if v < -0.5 { has_low = true; }
            }
        }
        assert!(has_high, "should have fertile zones");
        assert!(has_low, "should have barren zones");
    }
}
```

- [ ] **Step 2:** Run test to verify it fails
- [ ] **Step 3:** Implement

Algorithm: random placement with minimum-distance rejection + Gaussian falloff.
- Background: -1.0 (barren)
- Blob centers: placed randomly, rejected if too close to existing center
- Each blob: Gaussian falloff from center, mapping to +1.0 at center → -1.0 at edge

```rust
use crate::kernel::grid::Grid;
use rand::Rng;

pub fn generate_poisson_blobs(
    width: u16, height: u16,
    blob_count: u32,
    min_radius: f32, max_radius: f32,
    falloff: f32,
    rng: &mut impl Rng,
) -> Grid<f32> {
    let mut grid = Grid::new(width, height, -1.0); // barren background
    let mut centers: Vec<(f32, f32, f32)> = Vec::new(); // (cx, cy, radius)
    let min_distance = min_radius * 2.0;

    for _ in 0..blob_count * 3 {
        // Try up to 3x attempts to place blob_count blobs
        if centers.len() >= blob_count as usize {
            break;
        }
        let cx = rng.gen_range(0.0..width as f32);
        let cy = rng.gen_range(0.0..height as f32);
        let radius = rng.gen_range(min_radius..max_radius);

        // Minimum-distance rejection
        let too_close = centers.iter().any(|(ox, oy, _)| {
            let dx = cx - ox;
            let dy = cy - oy;
            (dx * dx + dy * dy).sqrt() < min_distance
        });
        if too_close {
            continue;
        }
        centers.push((cx, cy, radius));
    }

    // Apply Gaussian falloff for each blob
    for (cx, cy, radius) in &centers {
        let sigma = radius * falloff;
        let r_max = radius * 2.0; // extent of influence
        let x_min = ((*cx - r_max) as i32).max(0) as u16;
        let x_max = ((*cx + r_max) as i32).min(width as i32 - 1) as u16;
        let y_min = ((*cy - r_max) as i32).max(0) as u16;
        let y_max = ((*cy + r_max) as i32).min(height as i32 - 1) as u16;

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = dx * dx + dy * dy;
                let gauss = (-(dist_sq) / (2.0 * sigma * sigma)).exp();
                // Map: center of blob = 1.0, far = -1.0
                let value = gauss * 2.0 - 1.0;
                let current = *grid.get(x, y);
                grid.set(x, y, current.max(value).clamp(-1.0, 1.0));
            }
        }
    }
    grid
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add PoissonBlobs fertility seeding algorithm"
```

---

## Task 9: Mixing Pipeline

**Files:**
- Create: `v3/crates/v3-core/src/kernel/fertility/mixing.rs`
- Modify: `v3/crates/v3-core/src/kernel/fertility/mod.rs`

- [ ] **Step 1:** Write failing tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_layer_passthrough() {
        let grid = Grid::new(2, 2, 0.5);
        let result = mix_layers(&[(grid, 1.0)]);
        assert_eq!(*result.get(0, 0), 0.5);
    }

    #[test]
    fn two_layers_weighted_average() {
        let a = Grid::new(2, 2, 1.0);
        let b = Grid::new(2, 2, -1.0);
        let result = mix_layers(&[(a, 1.0), (b, 1.0)]);
        assert_eq!(*result.get(0, 0), 0.0); // average of 1.0 and -1.0
    }

    #[test]
    fn unequal_weights() {
        let a = Grid::new(2, 2, 1.0);
        let b = Grid::new(2, 2, -1.0);
        let result = mix_layers(&[(a, 3.0), (b, 1.0)]);
        assert!((result.get(0, 0) - 0.5).abs() < 0.001); // (3*1 + 1*-1)/4 = 0.5
    }

    #[test]
    fn zero_weight_fallback() {
        let a = Grid::new(2, 2, 0.8);
        let result = mix_layers(&[(a, 0.0)]);
        assert_eq!(*result.get(0, 0), 0.0); // zero total weight → uniform 0.0
    }

    #[test]
    fn output_clamped_to_range() {
        let a = Grid::new(2, 2, 1.0);
        let b = Grid::new(2, 2, 1.0);
        let result = mix_layers(&[(a, 2.0), (b, 2.0)]);
        assert!(*result.get(0, 0) <= 1.0);
        assert!(*result.get(0, 0) >= -1.0);
    }
}
```

- [ ] **Step 2:** Run test to verify it fails
- [ ] **Step 3:** Implement

```rust
use crate::kernel::grid::Grid;

pub fn mix_layers(layers: &[(Grid<f32>, f32)]) -> Grid<f32> {
    if layers.is_empty() {
        return Grid::new(0, 0, 0.0);
    }
    let width = layers[0].0.width();
    let height = layers[0].0.height();
    let total_weight: f32 = layers.iter().map(|(_, w)| w).sum();

    if total_weight <= 0.0 {
        return Grid::new(width, height, 0.0);
    }

    let mut result = Grid::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (grid, weight) in layers {
                sum += *grid.get(x, y) * weight;
            }
            result.set(x, y, (sum / total_weight).clamp(-1.0, 1.0));
        }
    }
    result
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add fertility layer mixing pipeline"
```

---

## Task 10: Annealing Computation

**Files:**
- Create: `v3/crates/v3-core/src/kernel/fertility/annealing.rs`

- [ ] **Step 1:** Write failing tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::simulation::AnnealingConfig;

    #[test]
    fn annealing_at_tick_zero_uses_initial_range() {
        let annealing = AnnealingConfig {
            enabled: true,
            ramp_ticks: 1000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        };
        let (min, max) = effective_fertility_range(&annealing, 0.0, 2.0, 0);
        assert!((min - 0.3).abs() < 0.001);
        assert!((max - 1.5).abs() < 0.001);
    }

    #[test]
    fn annealing_at_midpoint_interpolates() {
        let annealing = AnnealingConfig {
            enabled: true,
            ramp_ticks: 1000,
            initial_min_fertility: 0.0,
            initial_max_fertility: 2.0,
        };
        // target min=0.5, max=1.5 — at tick 500: lerp(0.0, 0.5, 0.5)=0.25, lerp(2.0, 1.5, 0.5)=1.75
        let (min, max) = effective_fertility_range(&annealing, 0.5, 1.5, 500);
        assert!((min - 0.25).abs() < 0.001);
        assert!((max - 1.75).abs() < 0.001);
    }

    #[test]
    fn annealing_at_ramp_end_uses_target() {
        let annealing = AnnealingConfig {
            enabled: true,
            ramp_ticks: 1000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        };
        let (min, max) = effective_fertility_range(&annealing, 0.0, 2.0, 1000);
        assert!((min - 0.0).abs() < 0.001);
        assert!((max - 2.0).abs() < 0.001);
    }

    #[test]
    fn annealing_beyond_ramp_uses_target() {
        let annealing = AnnealingConfig {
            enabled: true,
            ramp_ticks: 1000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        };
        let (min, max) = effective_fertility_range(&annealing, 0.0, 2.0, 9999);
        assert!((min - 0.0).abs() < 0.001);
        assert!((max - 2.0).abs() < 0.001);
    }

    #[test]
    fn annealing_disabled_uses_target_always() {
        let annealing = AnnealingConfig {
            enabled: false,
            ramp_ticks: 1000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        };
        let (min, max) = effective_fertility_range(&annealing, 0.0, 2.0, 0);
        assert!((min - 0.0).abs() < 0.001);
        assert!((max - 2.0).abs() < 0.001);
    }
}
```

- [ ] **Step 2:** Run test to verify it fails
- [ ] **Step 3:** Implement

```rust
use crate::config::simulation::AnnealingConfig;

/// Compute the effective fertility range at the given tick.
/// When annealing is disabled, returns the target range directly.
pub fn effective_fertility_range(
    annealing: &AnnealingConfig,
    target_min: f32,
    target_max: f32,
    tick: u64,
) -> (f32, f32) {
    if !annealing.enabled || annealing.ramp_ticks == 0 {
        return (target_min, target_max);
    }
    let progress = (tick as f32 / annealing.ramp_ticks as f32).clamp(0.0, 1.0);
    let effective_min = lerp(annealing.initial_min_fertility, target_min, progress);
    let effective_max = lerp(annealing.initial_max_fertility, target_max, progress);
    (effective_min, effective_max)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Map a raw [-1, 1] value to the effective [min, max] range.
pub fn map_fertility(raw: f32, effective_min: f32, effective_max: f32) -> f32 {
    let t = (raw + 1.0) / 2.0; // [-1, 1] → [0, 1]
    effective_min + t * (effective_max - effective_min)
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add fertility annealing computation with lerp mapping"
```

---

## Task 11: seed_fertility and Growth Integration

**Files:**
- Modify: `v3/crates/v3-core/src/kernel/food_resource.rs`
- Modify: `v3/crates/v3-core/src/kernel/fertility/mod.rs`

Wire everything together: `seed_fertility()` generates and stores the fertility grid, and `grow()` applies effective fertility to proportional growth, spread, and recovery spawn.

- [ ] **Step 1:** Write failing tests for fertility-aware growth

```rust
#[test]
fn fertility_zero_stops_proportional_growth() {
    let mut config = FoodResourceConfig::default();
    config.fertility.enabled = true;
    config.fertility.min_fertility = 0.0;
    config.fertility.max_fertility = 0.0;
    let mut res = FoodResource::new(4, 4, config);
    res.set_food(Position { x: 1, y: 1 }, 0.5);
    // All fertility is 0.0 → no growth
    let barriers = Grid::new(4, 4, false);
    let mut rng = StdRng::seed_from_u64(42);
    res.grow(&barriers, 1000, &mut rng);
    assert_eq!(res.food_at(Position { x: 1, y: 1 }), 0.5);
}

#[test]
fn fertility_doubles_growth_rate() {
    let mut config = FoodResourceConfig::default();
    config.fertility.enabled = true;
    config.fertility.min_fertility = 2.0;
    config.fertility.max_fertility = 2.0;
    config.fertility.layers = vec![]; // will default to PoissonBlobs but range forces 2.0 everywhere
    let mut res = FoodResource::new(4, 4, config);
    res.set_food(Position { x: 1, y: 1 }, 0.5);
    let barriers = Grid::new(4, 4, false);
    let mut rng = StdRng::seed_from_u64(42);
    res.grow(&barriers, 1000, &mut rng);
    // With fertility 2.0: 0.5 + 0.5 * 0.05 * 2.0 = 0.55
    let food = res.food_at(Position { x: 1, y: 1 });
    assert!((food - 0.55).abs() < 0.001, "expected ~0.55, got {food}");
}

#[test]
fn fertility_disabled_matches_baseline() {
    let config = FoodResourceConfig::default(); // fertility disabled
    let mut res = FoodResource::new(4, 4, config.clone());
    res.set_food(Position { x: 1, y: 1 }, 0.5);
    let barriers = Grid::new(4, 4, false);
    let mut rng = StdRng::seed_from_u64(42);
    res.grow(&barriers, 0, &mut rng);
    let food_disabled = res.food_at(Position { x: 1, y: 1 });

    // Same but explicit fertility = 1.0 everywhere
    let mut config2 = config;
    config2.fertility.enabled = true;
    config2.fertility.min_fertility = 1.0;
    config2.fertility.max_fertility = 1.0;
    let mut res2 = FoodResource::new(4, 4, config2);
    res2.set_food(Position { x: 1, y: 1 }, 0.5);
    let mut rng2 = StdRng::seed_from_u64(42);
    res2.grow(&barriers, 0, &mut rng2);
    let food_enabled = res2.food_at(Position { x: 1, y: 1 });

    assert!((food_disabled - food_enabled).abs() < 0.001);
}

#[test]
fn recovery_spawn_respects_fertility() {
    // Set up: all food consumed, trigger recovery spawn in a world where
    // half the cells have 0.0 fertility and half have 1.0
    let mut config = FoodResourceConfig::default();
    config.fertility.enabled = true;
    config.fertility.min_fertility = 0.0;
    config.fertility.max_fertility = 1.0;
    config.recovery_floor_ratio = 1.0; // always trigger recovery
    config.recovery_spawn_rate = 1.0;  // attempt every cell
    let mut res = FoodResource::new(4, 4, config);
    // Set fertility: left half barren (-1.0 → 0.0), right half fertile (1.0 → 1.0)
    for y in 0..4 {
        for x in 0..2 {
            res.fertility.set(x, y, -1.0);
        }
        for x in 2..4 {
            res.fertility.set(x, y, 1.0);
        }
    }
    let barriers = Grid::new(4, 4, false);
    let mut rng = StdRng::seed_from_u64(42);
    res.grow(&barriers, 1000, &mut rng);

    // Barren cells should have no recovery food
    for y in 0..4u16 {
        for x in 0..2u16 {
            assert_eq!(res.food_at(Position { x, y }), 0.0,
                "barren cell ({x},{y}) should have no recovery food");
        }
    }
}
```

- [ ] **Step 2:** Run tests to verify they fail
- [ ] **Step 3:** Implement `seed_fertility` on `FoodResource`

In `food_resource.rs`, add the `seed_fertility` method that:
1. Checks if `self.config.fertility.enabled` — if not, fill fertility grid with 0.0
2. Resolves empty layers to default PoissonBlobs
3. Generates each layer's grid using the appropriate algorithm
4. Passes grids to mixing pipeline
5. Stores result in `self.fertility`

- [ ] **Step 4:** Integrate fertility into `grow()` — proportional growth

In the proportional growth loop, compute effective fertility for each cell:
```rust
let effective_fertility = if self.config.fertility.enabled {
    let (eff_min, eff_max) = annealing::effective_fertility_range(
        &self.config.annealing,
        self.config.fertility.min_fertility,
        self.config.fertility.max_fertility,
        tick,
    );
    annealing::map_fertility(*self.fertility.get(x, y), eff_min, eff_max)
} else {
    1.0 // identity when disabled
};
let delta = source * self.config.growth_rate * effective_fertility;
```

- [ ] **Step 5:** Integrate fertility into `grow()` — spread deposit

Scale spread deposit by neighbor's effective fertility.

- [ ] **Step 6:** Integrate fertility into `grow()` — recovery spawn

Scale recovery deposit by cell's effective fertility.

- [ ] **Step 7:** Wire `seed_fertility()` at world startup

In `WorldState::new()` or the seeding path, call `self.food.seed_fertility(&mut rng)` after `seed_density()`.

- [ ] **Step 8:** Run tests — verify pass

Run: `cd v3 && cargo test -p v3-core -- food_resource --no-capture`
Expected: all pass

- [ ] **Step 9:** Run viability tests

Run: `cd v3 && cargo test -p v3-core --test viability`
Expected: all pass (fertility disabled by default)

- [ ] **Step 10:** Commit

```bash
git add -A
git commit -m "feat: wire fertility seeding and growth integration

seed_fertility generates raw [-1,1] grid from configured layers.
grow() applies effective fertility to proportional growth, spread,
and recovery spawn. All growth operations respect fertility invariant.
Fertility disabled matches baseline behavior exactly."
```

---

## Stage 1 Gates

- [ ] Run: `cd v3 && cargo test -p v3-core` — all pass
- [ ] Run: `cd v3 && cargo test --workspace` — all pass
- [ ] Run: `cd v3 && cargo fmt --all -- --check` — clean
- [ ] Run: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings` — clean
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `rust-skills`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run all Stage 1 tests after review-introduced changes
- [ ] Review Gate (Architecture): Dispatch `superpowers:code-reviewer` for architecture and decomposition review — verify boundary compliance with `docs/strategy/`, dependency directions, separation of concerns. Fix all findings. Re-dispatch until zero new findings.
- [ ] Re-run all Stage 1 tests after architecture review changes

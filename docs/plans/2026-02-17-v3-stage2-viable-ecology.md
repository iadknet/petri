# V3 Stage 2: Viable Ecology

**Goal:** Creatures survive by eating, die without food, move toward resources. The simulation is alive.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** config/ module, food growth, energy decay, eat/move actions, heuristic brain, sensors, full Phase 0-3 tick, seed creature viability test. Excludes genome, VM brain, reproduction, mutation, inventory, and frontend.

**Docs Impact:**
- Architecture design doc: no changes needed (Stage 2 scope already defined)
- Existing Stage 1 tests updated for new signatures

**Supersedes:** `docs/plans/2026-02-14-v3-phase1-walking-skeleton.md` (Stage 1 complete)

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-14-v3-architecture-design.md`

**Architecture:** Phase-based tick (Phase 0: world mechanics + energy decay, Phase 1: cognition via heuristic brain, Phase 2: randomized action execution, Phase 3: dead creature cleanup). Config threaded through all function signatures.

---

## Goal Alignment

- **GP-01**: Food growth + energy economy creates first emergent survival pressure
- **GP-02**: config/ module establishes clean configuration boundaries; tick/actions/ separates action execution from orchestration
- **GP-03**: Viability test verifies intent (creatures eat, move, survive) not just contracts
- **GP-04**: TickStats gains `actions_attempted`/`actions_succeeded` for runtime observability

---

## Boundary Impact

- New `config/` module added to v3-core (pure data, no dependencies)
- `tick/actions.rs` added (depends on kernel + config + creature + contracts)
- `sensors/` upgraded from stub to real implementation
- `runtime/` upgraded from stub to heuristic brain
- `seed.rs` signature changes (takes `&SimulationConfig` instead of `initial_energy`)
- `SimulationState::tick()` signature changes (takes `&SimulationConfig`)
- v3-server `ServerState` gains `SimulationConfig` field
- No changes to v1, v2, or docs/strategy

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| v1 `crates/petri-*` | keep | Untouched |
| kernel/ | change | Add `grow_food()`, `seed_food()`, `set_barrier()`, `Direction::ALL`, `Energy::drain_saturating()` |
| contracts/ | change | Add `Eat`, `Move` to WorldAction; extend EnvironmentalInputs with neighbors |
| creature/ | keep | Energy type already sufficient |
| seed.rs | change | Signature change for config threading |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `execute_action` take `CreatureId`? | Yes — `execute_move` needs it for `place_creature()` spatial index update | agent | resolved |
| Should `Energy::drain` always deduct (saturating)? | Add `drain_saturating()` for action costs that always deduct. Keep existing `drain()` for optional checks. | agent | resolved |
| Should EnvironmentalInputs use flat fields or arrays? | Use `[NeighborSense; 8]` array indexed by direction — cleaner for iteration, forward-compatible | agent | resolved |
| Should Phase 1 need `iter_mut()` for Stage 2? | No — heuristic brain only reads `CreatureInputs`, doesn't modify creature state. Use `iter()` in Phase 1, collect actions, then `iter_mut()` in Phase 2 when executing. | agent | resolved |
| What food growth algorithm? | Simple probabilistic: each non-barrier cell gains +1 food with probability `growth_rate`, capped at 255. Spread not needed for Stage 2. | agent | resolved |
| Should `execute_action` include `rng`? | Yes — forward compatibility with Stage 3 `execute_reproduce`. Stage 2 actions do not use it. | agent | resolved |
| Where do `ActionResult`/`ActionStatus`/`FailureReason` live? | In `tick/actions.rs`, not `contracts/outputs.rs`. These are execution artifacts, not cross-module contracts. Placing them in contracts would invert the dependency direction. | agent | resolved |
| Should config include `WorldDimensions`? | Deferred to Stage 3. World dimensions are currently passed to `WorldState::new()` directly and are not tunable during Stage 2 viability testing. Architecture doc lists them under config but they are not needed for Stage 2's goals. | agent | resolved |
| Should `seed_creatures` take `founder_name`? | Deferred to Stage 3. Stage 2 has no genome/founders system. The parameter will be added when `creature/founders.rs` is implemented. | agent | resolved |

---

## Architectural Issue: `execute_action` needs `CreatureId`

The architecture doc's `execute_action` signature takes `&mut CreatureState` but `execute_move` needs to call `world.place_creature(new_pos, creature_id)` which requires the `CreatureId`.

**Resolution:** Add `creature_id: CreatureId` as a parameter to `execute_action`. This is a minor signature addition that doesn't violate any boundary — the orchestrator already has the id from the action queue.

```rust
pub fn execute_action(
    action: &WorldAction,
    creature_id: CreatureId,
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> ActionResult
```

Note: `rng` is included for forward compatibility with Stage 3 `execute_reproduce`. Stage 2 actions (Eat, Move, NoOp) do not use it.

---

## Implementation Tasks

### Task 1: Config Module (pure data, no dependencies)
- [ ] Create `config/mod.rs` with `SimulationConfig` top-level struct
- [ ] Create `config/world/mod.rs` + `config/world/food.rs` with `FoodConfig`
- [ ] Create `config/energy/mod.rs` + `config/energy/lifecycle.rs` with `EnergyLifecycle`
- [ ] Create `config/energy/costs.rs` with `EnergyCosts`
- [ ] All structs derive `Clone, Debug` and implement `Default` with sensible values
- [ ] Register `pub mod config` in `lib.rs`
- [ ] `cargo test --workspace` passes (no tests broken)

**Config defaults:**

```rust
// FoodConfig
pub struct FoodConfig {
    pub growth_rate: f32,         // 0.02 — probability per cell per tick
    pub initial_density: u8,      // 80 — food density when seeding world
    pub initial_coverage: f32,    // 0.3 — fraction of cells with initial food
}

// EnergyLifecycle
pub struct EnergyLifecycle {
    pub initial_energy: u32,      // 20
    pub max_energy: u32,          // 100
    pub energy_decay_per_tick: u32, // 1
}

// EnergyCosts
pub struct EnergyCosts {
    pub move_cost: u32,           // 1
    pub eat_cost: u32,            // 0
    pub noop_cost: u32,           // 0
    pub eat_reward_per_food: u32, // 1 — energy gained per food unit consumed
}

// SimulationConfig
pub struct SimulationConfig {
    pub world: WorldConfig,
    pub energy: EnergyConfig,
}
pub struct WorldConfig {
    pub food: FoodConfig,
}
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycle,
    pub costs: EnergyCosts,
}
```

### Task 2: Contracts Extension + Kernel Helpers
- [ ] Add `Eat` and `Move { direction: Direction }` variants to `WorldAction`
- [ ] Add `Direction::ALL: [Direction; 8]` constant for iteration
- [ ] Add `NeighborSense` struct: `{ food_density: u8, passable: bool }`
- [ ] Extend `EnvironmentalInputs` with `neighbors: [NeighborSense; 8]`
- [ ] Add `actions_attempted: u32` and `actions_succeeded: u32` to `TickStats`
- [ ] Add `set_barrier(&mut self, pos, val)` method to `WorldState` (needed for sensor/action tests)
- [ ] Update existing contracts tests for new variants
- [ ] `cargo test --workspace` passes

Note: `ActionResult`/`ActionStatus`/`FailureReason` live in `tick/actions.rs` (Task 6), not in contracts. They are execution artifacts.

### Task 3: Kernel — Food Growth
- [ ] Add `grow_food(&mut self, config: &FoodConfig, rng: &mut impl Rng)` to `WorldState`
- [ ] Add `seed_food(&mut self, config: &FoodConfig, rng: &mut impl Rng)` for initial world setup
- [ ] Add `Energy::drain_saturating(&mut self, amount: u32)` — always deducts, clamps to 0
- [ ] Write unit test: food grows probabilistically, capped at 255
- [ ] Write unit test: `drain_saturating` always reduces energy
- [ ] `cargo test --workspace` passes

### Task 4: Sensors — Real Implementation
- [ ] Replace `gather_inputs_stub` with `gather_inputs(creature, world)` in `sensors/mod.rs`
- [ ] Implement neighbor sensing: for each of 8 directions, resolve neighbor position and fill `NeighborSense`
- [ ] Passable = not barrier AND not occupied AND not OOB
- [ ] Write test: sensor correctly reads food density from world
- [ ] Write test: sensor correctly detects barriers and occupied cells in neighbors
- [ ] `cargo test --workspace` passes

### Task 5: Runtime — Heuristic Brain
- [ ] Replace `execute_creature_stub` with `execute_heuristic(inputs, rng)` in `runtime/executor.rs`
- [ ] Heuristic logic: (1) if food_density_self > 0 → Eat, (2) else find neighbor with highest food that is passable → Move toward it (break ties randomly via `rng`), (3) else → Move in random passable direction, (4) if no passable direction → NoOp
- [ ] Write test: returns Eat when food present
- [ ] Write test: moves toward food when neighbor has food
- [ ] Write test: moves randomly when no food visible
- [ ] Write test: returns NoOp when surrounded by barriers
- [ ] `cargo test --workspace` passes

### Task 6: Tick Actions — Eat and Move
- [ ] Create `tick/actions.rs` with `ActionResult`, `ActionStatus`, `FailureReason` types and `execute_action()` dispatcher
- [ ] Implement `execute_eat`: drain eat_cost, consume food, charge energy, return result
- [ ] Implement `execute_move`: drain move_cost, validate target, update position + spatial index
- [ ] `execute_action` signature includes `rng: &mut impl Rng` for forward compatibility (unused in Stage 2)
- [ ] `NoOp` returns `ActionResult::success()` with no side effects
- [ ] All actions drain energy cost first (via `drain_saturating`), regardless of outcome
- [ ] Register `pub mod actions` in `tick/mod.rs`
- [ ] Write intent test: creature eats food, gains energy, food density decreases
- [ ] Write intent test: creature moves to empty cell, position updates, spatial index updates
- [ ] Write intent test: move into barrier fails but still costs energy
- [ ] Write intent test: eat with no food fails but still costs energy (if eat_cost > 0)
- [ ] `cargo test --workspace` passes

### Task 7: Tick Orchestrator — Full Phase 0-3
- [ ] Update `tick()` signature: `tick(state, config, rng) -> TickStats`
- [ ] Phase 0: call `world.grow_food(config)` + `apply_energy_decay()` for all creatures
- [ ] Phase 1: iterate creatures with `iter()` (not `iter_mut()` — heuristic brain is pure/read-only for Stage 2), gather inputs, run heuristic brain, collect `(CreatureId, WorldAction)` queue
- [ ] Phase 2: shuffle queue, execute actions, track attempted/succeeded counts
- [ ] Phase 3: remove dead creatures (already exists, verify still works)
- [ ] Update `SimulationState::tick()` to accept `&SimulationConfig`
- [ ] Write integration test: creatures eat food and gain energy over multiple ticks
- [ ] Write integration test: creatures with no food eventually die (energy decay)
- [ ] Write integration test: TickStats shows non-zero actions_attempted
- [ ] `cargo test --workspace` passes

### Task 8: Seed Module Update
- [ ] Change `seed_creatures` signature: replace `initial_energy: u32` with `config: &SimulationConfig`
- [ ] Use `config.energy.lifecycle.initial_energy` for creature energy
- [ ] Use fixed phenotype `[204, 61, 61]` instead of `random_phenotype(rng)`
- [ ] Add `seed_food()` call to seed initial food on the world
- [ ] Update all existing seed tests
- [ ] `cargo test --workspace` passes

### Task 9: Server Update
- [ ] Add `config: SimulationConfig` field to `ServerState`
- [ ] Update `ServerState::new()` to create default config and pass to `seed_creatures`
- [ ] Update `ServerState::tick()` to pass `&self.config` to simulation tick
- [ ] Update all existing server tests
- [ ] `cargo test --workspace` passes

### Task 10: Viability Test (Stage Gate)
- [ ] Create `v3-core/tests/seed_viability.rs`
- [ ] Create `viability_config()` helper: 32x32 world, 15 creatures, high food density, moderate decay
- [ ] Assert: population > 0 after 100 ticks
- [ ] Assert: at least some creatures have energy > initial (they ate)
- [ ] Assert: food has been consumed (total food < initial)
- [ ] Assert: creature positions have changed from initial
- [ ] Assert: TickStats actions_attempted > 0
- [ ] Test runs in < 2 seconds
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all --check` passes

---

## Review cycles: 1

Cycle 1: Architecture review identified C-03 (missing `rng` in `execute_action`), W-05 (`ActionResult` placement), W-02 (`Direction::ALL`), W-04 (`WorldDimensions` deferred), W-06 (tie-breaking), W-07 (`founder_name` deferred), N-04 (`set_barrier` for tests). All resolved in plan revision.

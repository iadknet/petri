# Food Fertility Layer v1 — Design Spec

**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** v3-core (kernel, config, simulation), v3-server (projection, transport), frontend (overlay)
**Supersedes:** None
**Superseded-By:** None
**Docs Impact:** `v3-world-grid-spec.md` (fertility layer, fertility/annealing config, config defaults reconciliation), startup seeding spec (fertility generation), server projection and API protocol specs (`food_fertility_u8`)

---

## Problem

Creatures initially evolve increasing complexity (barrier awareness, multi-sensor
integration) but then gradually simplify to basic random-walk programs. The root
cause: the environment doesn't demand complexity. Random walkers find enough food
to reproduce, so there's no sustained fitness gap between smart and simple
creatures. Complexity is mutation-fragile overhead that erodes over time.

## Solution

Add a static **fertility layer** to the food system that creates persistent
spatial structure — fertile zones where food regrows quickly and barren zones
where it doesn't. This forces creatures to navigate between productive areas
rather than wandering randomly. Creatures that sense food gradients and move
toward fertile zones gain a real survival advantage over random walkers.

## Goal

Create environmental conditions that reward navigation and sensing over random
walks, addressing the observed complexity collapse. Build the `FoodResource`
abstraction as the foundation for future multi-food-type ecology.

## Scope

**In scope:**

- `FoodResource` abstraction (single instance in v1)
- Static fertility grid with 3 seeding algorithms + mixed-mode layering
- Warmup annealing (structured from tick 0, severity parameters anneal over time)
- Integration into food growth (proportional, spread, and recovery spawn)
- Frontend fertility overlay toggle
- Server transport for fertility data

**Out of scope (future features):**

- Multiple food types
- Occupancy depletion
- Barrier-attached food / barrier affinity seeding
- Temporal drift
- Food system performance optimization
- Fertility painting tools

## Goal Alignment

| Goal ID | Alignment |
|---------|-----------|
| GP-01   | Primary driver. Fertility creates spatial structure that rewards sensing and navigation over random walks, directly addressing the observed complexity collapse. |
| GP-02   | `FoodResource` abstraction cleanly encapsulates food state and logic. Fertility seeding is isolated from growth. Transport uses quantized static payload, not per-tick data. |
| GP-03   | Regression tests at every stage, intermediate viability checks, TDD for all behavior changes, and mandatory review gates ensure high-confidence iteration. |
| GP-04   | Frontend fertility overlay makes the spatial structure visible. Profiling comparison (fertility off vs on) ensures observability of performance impact. |

## Boundary Impact

| Area | Impact |
|------|--------|
| **v3-core / kernel** | `FoodResource` struct added to `kernel/` or `kernel/ecology/`. `pub` type with private fields. `WorldState` delegates food methods to `FoodResource`. Read APIs (`food_at`) stable; growth call site (`run_phase_0`) signature changes minimally. |
| **v3-core / config** | New `FertilityConfig`, `AnnealingConfig`, `FoodResourceConfig` structs. `WorldFoodConfig` renamed to `FoodResourceConfig`. New fields use `#[serde(default)]` so configs that don't need fertility can omit those sections. |
| **v3-core / simulation** | `seed_fertility()` called at world startup. `grow()` takes `tick` as parameter (passed from `Simulation.tick` in `run_phase_0`). |
| **v3-server / projection** | `food_fertility_u8` added to world-static projection. `same_world_static()` in `query/projection.rs` updated to include fertility mask in structural diff. |
| **v3-server / transport** | Protocol payloads gain `food_fertility_u8: Vec<u8>` field. |
| **frontend** | New protocol field, store slice, overlay rendering layer, toggle control. |
| **Dependency direction** | No new cross-crate dependencies. `noise` crate added to `v3-core` for Fbm generation. |

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `WorldState` as read surface | Preserved | `WorldState` delegates remain the primary API for simulation code. `FoodResource` is `pub` with private fields — transport layer accesses it via `WorldState::food()` read-only accessor to read the fertility grid. Sensor and runtime read APIs remain stable; growth call sites change minimally (signature update in `run_phase_0`). |
| Config `deny_unknown_fields` | Follows convention | New config structs use `#[serde(deny_unknown_fields)]` per codebase convention. `#[serde(default)]` on fertility/annealing fields for ergonomics (optional when not needed). |
| World-static projection | Extended | `food_fertility_u8` is a static field (sent once, not per tick). Fits existing revision-tracking pattern. |
| Simulation tick ownership | Preserved | `tick` counter stays on `Simulation`. Passed as parameter to `FoodResource::grow()` — no ownership change. |

## Research Context

Kashtan & Alon (2005) showed that environments with modularly varying goals
produce dramatically faster evolution of modularity and complexity. While a
static fertility layer provides spatial challenge, the strongest complexity
driver would be temporal variation — periodic shifts in fertility patterns.
Temporal drift is deferred from this scope but the architecture should support
it. See Future Considerations.

Sources:

- [Kashtan & Alon 2005 — Spontaneous evolution of modularity and network motifs (PNAS)](https://www.pnas.org/doi/10.1073/pnas.0503610102)
- [Varying environments can speed up evolution (PNAS 2007)](https://www.pnas.org/doi/10.1073/pnas.0611630104)
- [Environmental complexity favors the evolution of learning (Behavioral Ecology 2016)](https://academic.oup.com/beheco/article/27/3/842/2364810)

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `FertilityConfig.layers` default to empty vec or a single algorithm? | Default to `vec![FertilityLayer { algorithm: PoissonBlobs::default(), weight: 1.0 }]`. PoissonBlobs creates the clearest patch structure (discrete fertile zones with barren gaps), which most directly creates "hunt for food patches" pressure. Fbm creates smoother gradients that a random walker can still stumble through. When `enabled: true`, an empty layers vec is treated as this default. | Design | Resolved |
| Which noise crate for Fbm? | Use the `noise` crate. Lightweight, pure Rust, well-maintained. | Design | Resolved |
| Poisson disk sampling: true Bridson's algorithm or simpler random-with-separation? | Simple random placement with minimum-distance rejection. Sufficient for v1; upgrade to Bridson's later if blob distribution quality matters. | Design | Resolved |
| Config default discrepancies between code and `v3-world-grid-spec.md` | Code is canonical. Spec will be updated to match code defaults as part of Stage 4 docs work. Discrepancies: `growth_rate` (code: 0.05, spec: 0.25), `spread_threshold_ratio` (code: 0.8, spec: 0.75), `recovery_spawn_rate` (code: 0.01, spec: 0.02), `recovery_floor_ratio` (code: 0.01, spec: 0.03). | Stage 4 | Resolved — code is source of truth |
| Transport quantization: raw `[-1, 1]` or mapped `[min, max]` values? | Two-step pipeline: (1) map raw `[-1, 1]` → effective `[min, max]` using target config range, (2) quantize effective → `u8` via `((effective - min) / (max - min) * 255.0).round() as u8`. This gives the frontend the actual growth multiplier, not the raw intermediate. | Design | Resolved |

---

## Design

### 1. FoodResource Abstraction

Extract all food-related state and logic into a self-contained unit. This is the
foundation for future multi-food-type ecology — each food type will be an
instance of `FoodResource`.

```rust
/// Public type with private fields. Transport layer accesses fertility data
/// via the `fertility()` accessor for serialization.
pub struct FoodResource {
    density: Grid<f32>,
    fertility: Grid<f32>,       // raw [-1, 1] values from seeding
    config: FoodResourceConfig,
    growth_scratch: Vec<f32>,   // reusable scratch buffer for growth snapshot
}

impl FoodResource {
    /// Read-only access to the raw fertility grid (for transport serialization).
    pub fn fertility(&self) -> &Grid<f32> { &self.fertility }
    /// Read-only access to the config (for transport quantization range).
    pub fn config(&self) -> &FoodResourceConfig { &self.config }
}

pub struct FoodResourceConfig {
    // moved from WorldFoodConfig:
    pub growth_rate: f32,            // default: 0.05
    pub max_density: f32,            // default: 1.0
    pub initial_density: f32,        // default: 1.0
    pub initial_coverage: f32,       // default: 0.15
    pub spread_threshold_ratio: f32, // default: 0.8
    pub spread_density_ratio: f32,   // default: 0.25
    pub recovery_spawn_rate: f32,    // default: 0.01
    pub recovery_floor_ratio: f32,   // default: 0.01
    // new:
    #[serde(default)]
    pub fertility: FertilityConfig,
    #[serde(default)]
    pub annealing: AnnealingConfig,
}

impl FoodResource {
    pub fn grow(&mut self, barriers: &Grid<bool>, tick: u64, rng: &mut impl Rng) { ... }
    pub fn consume(&mut self, pos: Position) -> f32 { ... }
    pub fn food_at(&self, pos: Position) -> f32 { ... }
    pub fn set_food(&mut self, pos: Position, value: f32) { ... }
    pub fn seed_density(&mut self, barriers: &Grid<bool>, rng: &mut impl Rng) { ... }
    pub fn seed_fertility(&mut self, rng: &mut impl Rng) { ... }
}
```

`WorldState` delegates to `FoodResource`. Note: `WorldState` does not own the
tick counter — `Simulation` does. The `grow_food` delegate accepts `tick` as a
parameter, passed from `run_phase_0`.

```rust
pub struct WorldState {
    food: FoodResource,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
    // ...
}

impl WorldState {
    /// Read-only access to the food resource (e.g., for transport layer to
    /// read the fertility grid for serialization).
    pub fn food(&self) -> &FoodResource { &self.food }
}

impl WorldState {
    /// Convenience delegate for single-food model.
    /// Will be revised or removed when multi-food is implemented.
    pub fn grow_food(&mut self, tick: u64, rng: &mut impl Rng) {
        self.food.grow(&self.barriers, tick, rng);
    }

    /// Convenience delegate for single-food model.
    /// Will be revised or removed when multi-food is implemented.
    pub fn food_at(&self, pos: Position) -> f32 {
        self.food.food_at(pos)
    }

    /// Convenience delegate for single-food model.
    /// Will be revised or removed when multi-food is implemented.
    pub fn consume_food(&mut self, pos: Position) -> f32 {
        self.food.consume(pos)
    }

    /// Convenience delegate for single-food model.
    /// Will be revised or removed when multi-food is implemented.
    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.food.set_food(pos, value);
    }
}
```

Call site change in `run_phase_0` (tick.rs):
```rust
// before: sim.world.grow_food(&mut sim.rng, &sim.config);
// after:  sim.world.grow_food(sim.tick, &mut sim.rng);
```

Config is absorbed into `FoodResource` — `grow()` reads from `self.config`
instead of accepting `&SimulationConfig`. `WorldFoodConfig` is renamed to
`FoodResourceConfig` and `#[serde(default)]` fields for `fertility` and
`annealing` are added. Breaking config changes are acceptable.

**Runtime config updates:** When `PATCH /config` modifies `world.food.*` fields,
the updated config must be propagated into `FoodResource.config`. Add a
`WorldState::apply_food_config(&mut self, config: &FoodResourceConfig)` method
that updates `self.food.config`. This is called from the config patch handler.
Note: fertility and annealing config changes take effect on the next `grow()`
call — fertility topology is NOT re-seeded (it's startup-only in v1).

### 2. Fertility Config

```rust
/// All new config structs follow the existing `#[serde(deny_unknown_fields)]`
/// convention used throughout the codebase.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityConfig {
    pub enabled: bool,               // default: false
    pub min_fertility: f32,          // default: 0.0
    pub max_fertility: f32,          // default: 2.0
    pub layers: Vec<FertilityLayer>, // default: vec![FertilityLayer::default()]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityLayer {
    pub algorithm: FertilityAlgorithm,
    pub weight: f32,                 // default: 1.0
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FertilityAlgorithm {
    Uniform {
        value: f32,                  // [-1, 1], default: 1.0
    },
    Fbm {
        octaves: u32,                // default: 4
        frequency: f32,              // default: 0.02 (large patches)
        lacunarity: f32,             // default: 2.0
        persistence: f32,            // default: 0.5
        seed: Option<u64>,           // None = derive from world seed
    },
    PoissonBlobs {
        blob_count: u32,             // default: 8
        min_radius: f32,             // default: 5.0
        max_radius: f32,             // default: 15.0
        falloff: f32,                // gaussian sigma multiplier, default: 0.5
        seed: Option<u64>,           // None = derive from world seed
    },
}
```

**Note on `#[non_exhaustive]`:** All internal `match` arms on `FertilityAlgorithm`
will need a wildcard/unreachable arm. This is an intentional ergonomic cost to
support future variants (e.g., `BarrierDistanceBias`) without breaking changes.

**Mixing pipeline:**

1. Each algorithm generates a `Grid<f32>` with values in `[-1, 1]`, using
   `self.fertility.width()` / `self.fertility.height()` for world dimensions
2. Per-cell weighted sum: `value = sum(layer_weight * layer_output[cell]) / total_weight`
3. Clamp result to `[-1, 1]`
4. Store raw clamped values in `self.fertility`

**Edge case — zero total weight:** If all layer weights are zero (or layers is
empty after default resolution), treat as uniform fertility at 0.0 (all cells
get raw value 0.0, which maps to the midpoint of `[min, max]`).

The mapping from raw `[-1, 1]` to effective fertility `[min, max]` happens at
growth time (see Section 4), not at seeding time. This enables annealing to
change the effective range without re-seeding.

When `enabled: true` with an empty `layers` vec, a single `PoissonBlobs` layer
with defaults is used. PoissonBlobs is the default because it creates discrete
fertile zones with clear barren gaps, which most directly creates the "hunt for
food patches" pressure this feature targets.

### 3. Warmup Annealing

Fertility topology is fixed at tick 0 — the spatial structure never changes in
v1. Annealing adjusts the severity parameters over time so founders see
a friendlier world initially while learning the correct spatial structure from
the start.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnealingConfig {
    pub enabled: bool,               // default: false
    pub ramp_ticks: u64,             // default: 5000
    pub initial_min_fertility: f32,  // default: 0.3 (friendlier floor)
    pub initial_max_fertility: f32,  // default: 1.5 (less extreme ceiling)
}
```

At tick `t`, the effective fertility range is:

```
progress = clamp(t / ramp_ticks, 0.0, 1.0)
effective_min = lerp(initial_min_fertility, min_fertility, progress)
effective_max = lerp(initial_max_fertility, max_fertility, progress)
```

This means:

- Tick 0: spatial structure exists but barren zones still have 0.3 fertility
  (founders can survive)
- Over `ramp_ticks`: barren zones approach 0.0, contrast increases
- After ramp completes: full target ecology is active

### 4. Growth Integration

`FoodResource::grow()` applies fertility at two points in the existing growth
algorithm:

**Proportional growth:**

```
effective_fertility = map(raw_fertility[cell], [-1, 1] -> [effective_min, effective_max])
new = current + (current * growth_rate * effective_fertility)
clamped to max_density
```

**Spread deposit:**

```
neighbor_fertility = map(raw_fertility[neighbor], [-1, 1] -> [effective_min, effective_max])
deposit = delta * spread_density_ratio * neighbor_fertility
clamped to max_density
```

**Recovery spawn:** Fertility-aware. Recovery spawn deposits are scaled by the
cell's effective fertility:

```
deposit = max_density * growth_rate * effective_fertility
```

Barren cells (effective fertility 0.0) receive no recovery food. Fertile cells
receive the full deposit. This establishes the invariant that **all food growth
operations respect the fertility layer**, which ensures clean composition with
future ecology layers (occupancy depletion, multi-food). Recovery spawn remains
an emergency mechanism (triggers when average density < recovery floor), but it
respects the world's spatial structure.

### 5. Server Transport

Add `food_fertility_u8` to the world-static projection payload:

- Quantization is a two-step pipeline:
  1. Map raw `[-1, 1]` → effective `[min_fertility, max_fertility]` using the
     **target** (post-annealing) config range, not the current annealed range
  2. Quantize effective → `u8`: `u8 = ((effective - min) / (max - min) * 255.0).round() as u8`
- **Edge case — `min == max`:** When the fertility range is a single value (no
  variation), all cells quantize to `128` (midpoint). Skip the division.
- This means the frontend overlay shows the steady-state fertility landscape,
  not the currently-annealed state. This is intentional — the overlay represents
  the world's permanent structure, and annealing is a transient startup ramp.
- Included in world-static payload (sent once on connect + when world resets,
  not every tick since fertility is static)
- Update `same_world_static()` in `v3-server/src/query/projection.rs` to include
  fertility mask in structural diff
- Static revision tracking: structural diff over dimensions + barrier mask +
  fertility mask

Future multi-food will migrate `food_fertility_u8` to an indexed
`Vec<FoodLayerTransport>` structure.

### 6. Frontend Overlay

- Ingest `food_fertility_u8` in frontend protocol types and world-view state
- Add read-only fertility overlay toggle in the control bar
- Render as a color layer (e.g., green = fertile, red/brown = barren) with
  configurable opacity
- No editing or painting in v1

---

## Implementation Steps

### Checkmark Protocol

Every step in this plan is a checkmark item. Implementing agents MUST update
this file to check off each item (`- [x]`) as it is completed. Do not proceed
past a Review Gate until it is checked off. Progress is tracked by the state of
these checkmarks — unchecked items are incomplete work.

### Review Gate Protocol

Review gates are mandatory recursive review loops. The protocol for every review
gate is:

1. Dispatch `superpowers:code-reviewer` subagent with the required domain skill
   (`rust-skills` for backend, `vercel-react-best-practices` and
   `vercel-composition-patterns` for frontend)
2. Fix all findings
3. Re-dispatch `superpowers:code-reviewer` on the changed code
4. Repeat until clean pass (zero new findings)
5. Re-run all tests that cover code changed during the review cycle
6. Only then check off the review gate and proceed

### Stage 0: Setup

- [ ] Create worktree and branch (`codex/food-fertility-v1`)
- [ ] Capture baseline test status (`cargo test --workspace`)
- [ ] Document frozen v1 scope decisions in plan

### Stage 1: Backend Core

**A1 — FoodResource abstraction:**

- [ ] Create `FoodResource` struct with `density`, `fertility`, `config`, `growth_scratch` fields
- [ ] Rename `WorldFoodConfig` to `FoodResourceConfig`, add `#[serde(default)]` `FertilityConfig` and `AnnealingConfig` fields
- [ ] Implement `FoodResource::food_at()`, `FoodResource::consume()`, `FoodResource::set_food()`, `FoodResource::grow()`
- [ ] Implement `FoodResource::seed_density()` (move existing food seeding logic)
- [ ] Add `food: FoodResource` to `WorldState`
- [ ] Implement `WorldState` delegate methods (`grow_food`, `food_at`, `consume_food`, `set_food`) with transitional doc comments
- [ ] Update `run_phase_0` call site: pass `sim.tick` to `grow_food()` delegate
- [ ] Update all other existing call sites to work with new structure
- [ ] Implement `WorldState::apply_food_config()` for runtime config patch propagation
- [ ] Tests: `FoodResource` produces identical behavior to existing food system (regression)
- [ ] Tests: runtime config patch propagates to `FoodResource.config` and takes effect on next `grow()`
- [ ] Intermediate verification: `cargo test -p v3-core --test viability` passes after A1

**A2 — Fertility seeding algorithms:**

- [ ] Add `noise` crate dependency to `v3-core/Cargo.toml`
- [ ] Implement `Uniform` fertility seeding algorithm (output clamped to `[-1, 1]`)
- [ ] Implement `Fbm` fertility seeding algorithm (deterministic from seed, output `[-1, 1]`)
- [ ] Implement `PoissonBlobs` fertility seeding algorithm (random placement with minimum-distance rejection + Gaussian falloff, deterministic from seed, output `[-1, 1]`)
- [ ] Implement mixing pipeline: per-layer generation, weighted sum, clamp to `[-1, 1]`
- [ ] Implement `FoodResource::seed_fertility()` (reads dimensions from `self.fertility` grid)
- [ ] Tests: each algorithm outputs values in `[-1, 1]` range
- [ ] Tests: each algorithm is deterministic given same seed
- [ ] Tests: mixing pipeline correctly weights and maps multiple layers
- [ ] Tests: mixing pipeline with all-zero weights falls back to uniform 0.0
- [ ] Tests: empty layers vec defaults to single PoissonBlobs

**A3 — Annealing:**

- [ ] Implement `AnnealingConfig` struct with defaults
- [ ] Implement effective fertility range computation (`lerp` based on tick / ramp_ticks progress)
- [ ] Tests: annealing at tick 0 uses initial range
- [ ] Tests: annealing at midpoint interpolates correctly
- [ ] Tests: annealing at/beyond `ramp_ticks` uses target range
- [ ] Tests: annealing disabled uses target range at all ticks

**A4 — Growth integration:**

- [ ] Wire `seed_fertility()` call at world startup
- [ ] Integrate fertility into `FoodResource::grow()` proportional growth step (map raw `[-1, 1]` → effective range at growth time)
- [ ] Integrate fertility into `FoodResource::grow()` spread deposit step
- [ ] Integrate fertility into `FoodResource::grow()` recovery spawn step (deposit scaled by effective fertility)
- [ ] Tests: fertility = 0.0 cell produces no proportional growth
- [ ] Tests: fertility = 2.0 cell produces doubled growth rate
- [ ] Tests: spread deposit scales by neighbor fertility
- [ ] Tests: recovery spawn scales deposit by cell fertility (barren cell receives no recovery food)
- [ ] Tests: fertility disabled produces identical behavior to baseline (regression)

**Stage 1 Gates:**

- [ ] All backend tests pass (`cargo test -p v3-core`)
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `rust-skills`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run all Stage 1 tests after review-introduced changes
- [ ] Review Gate (Architecture): Dispatch `superpowers:code-reviewer` for architecture and decomposition review — verify boundary compliance with `docs/strategy/`, dependency directions, and separation of concerns. Fix all findings. Re-dispatch until zero new findings.

### Stage 2: Server / Projection

**B1 — Projection and transport types:**

- [ ] Add `food_fertility_u8` field to world-static projection struct
- [ ] Implement quantization: (1) map raw `[-1, 1]` → effective `[min, max]` using target config range, (2) quantize `u8` via `((effective - min) / (max - min) * 255.0).round() as u8`
- [ ] Add `food_fertility_u8` to protocol payload types
- [ ] Tests: quantization round-trips within acceptable precision
- [ ] Tests: quantization with `min == max` produces uniform 128

**B2 — Revision semantics:**

- [ ] Update `same_world_static()` in `query/projection.rs` to include fertility mask in diff
- [ ] Verify static-change hint fires on world reset
- [ ] Tests: revision changes when fertility grid changes
- [ ] Tests: revision stable when fertility grid unchanged

**B3 — Endpoint coverage:**

- [ ] Snapshot path includes `food_fertility_u8`
- [ ] WebSocket path includes `food_fertility_u8` in initial payload
- [ ] HTTP path includes `food_fertility_u8`
- [ ] Tests: all transport paths include consistent fertility data

**Stage 2 Gates:**

- [ ] All server tests pass (`cargo test -p v3-server`)
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `rust-skills`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run all Stage 2 tests after review-introduced changes

### Stage 3: Frontend

**C1 — Protocol and store wiring:**

- [ ] Add `food_fertility_u8` to frontend protocol types
- [ ] Ingest fertility data into world-view state/store
- [ ] Tests: protocol parsing handles fertility field correctly
- [ ] Tests: store updates when fertility data received

**C2 — Overlay rendering and toggle:**

- [ ] Add fertility overlay toggle to control bar
- [ ] Implement fertility overlay rendering layer (color mapping: fertile = green, barren = brown/red)
- [ ] Implement configurable opacity for overlay
- [ ] Tests: overlay visibility toggles correctly
- [ ] Tests: overlay renders without errors when fertility data present
- [ ] Tests: overlay handles missing fertility data gracefully (disabled state)

**Stage 3 Gates:**

- [ ] All frontend tests pass
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `vercel-react-best-practices` and `vercel-composition-patterns`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run all Stage 3 tests after review-introduced changes

### Stage 4: Docs and Final Verification

- [ ] Update `v3-world-grid-spec.md`: add fertility layer section, reconcile config defaults to match code (growth_rate: 0.05, spread_threshold_ratio: 0.8, recovery_spawn_rate: 0.01, recovery_floor_ratio: 0.01)
- [ ] Update startup seeding spec for fertility generation
- [ ] Update server projection and API protocol specs for `food_fertility_u8`
- [ ] Final: `cargo test -p v3-core --test viability` (fertility disabled — production defaults)
- [ ] Final: `cargo test -p v3-core --test viability` (fertility + annealing enabled — uses `SimulationConfig::default()` with only `fertility.enabled = true` and `annealing.enabled = true` toggled, per Viability Test Policy)
- [ ] Final: `cargo test --workspace`
- [ ] Final: server test suite passes
- [ ] Final: frontend test suite passes
- [ ] Final: profiling comparison — `profile_ticks` with fertility off vs on
- [ ] Review Gate: Cross-cutting dispatch of `superpowers:code-reviewer` with `rust-skills` (backend) and Vercel skills (frontend). Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Review Gate (Architecture): Final architecture and decomposition review — verify all boundaries, dependency directions, separation of concerns consistent with `docs/strategy/`. Fix all findings. Re-dispatch until zero new findings.
- [ ] Re-run all workspace tests after review-introduced changes

---

## Acceptance Criteria

1. Fertility disabled path matches current behavior exactly (regression-safe)
2. Fertility generation is deterministic given the same seed
3. All algorithm outputs normalized to `[-1, 1]` before mapping
4. Fertility affects all food growth operations: proportional growth, spread
   deposit, and recovery spawn
5. Annealing correctly lerps fertility range over configured ticks
6. `FoodResource` abstraction contains all food state and logic; `WorldState`
   delegates
7. `food_fertility_u8` present in world-static transport
8. Frontend overlay toggleable and renders fertility grid
9. Viability tests pass with fertility disabled (backwards compatibility)
10. Viability tests pass with a representative fertility + annealing config
    (founders survive the annealing period)

---

## Future Considerations

These are documented design decisions deferred from v1. The current architecture
is designed to support each of these as additive extensions.

### Multi-food types (3–6 with exclusive spatial occupancy)

- `WorldState.food: FoodResource` becomes `food_resources: Vec<FoodResource>`
  or `[FoodResource; N]`
- WorldState delegate methods (`grow_food`, `food_at`, `consume_food`,
  `set_food`) are marked transitional and will be revised or removed
- Sensors (`FoodHere`, `NeighborFoodRing`, `AreaFoodSummary`) need per-type
  variants — sensor system extension is a separate design task
- Eat action dispatch needs food type targeting
- Exclusive spatial occupancy is a cross-resource concern requiring a coordinator
  or shared occupancy mask passed to `grow()`
- Transport migrates from `food_fertility_u8` to indexed
  `Vec<FoodLayerTransport>`

### Occupancy depletion

- Adds a `depletion: Grid<f32>` to `FoodResource`
- `grow()` incorporates depletion as another multiplicative factor alongside
  fertility
- `consume()` updates depletion when food is eaten
- Per-tick recovery decreases depletion values
- Prerequisite: extend profiling harness to benchmark with depletion on/off
  before shipping, per performance decision
- **Composition note:** Because recovery spawn already respects fertility (v1
  invariant), adding depletion as another multiplicative factor composes cleanly
  — recovery spawn will respect both fertility and depletion without special-casing

### Barrier-attached food

- Re-add `BarrierDistanceBias` variant to `FertilityAlgorithm`
  (`#[non_exhaustive]` makes this backwards-compatible)
- Re-add `barriers: &Grid<bool>` parameter to `seed_fertility()`
- Can be implemented as a food type whose fertility heavily weights barrier
  proximity, or as a configurable option on any food type

### Temporal drift

- Periodic shifts in fertility patterns create the "modularly varying
  environment" that Kashtan & Alon (2005) showed drives modularity and
  complexity most effectively
- Raw `[-1, 1]` fertility storage + runtime mapping provides a clean foundation:
  drift can modify raw values or add a time-varying overlay
- Annealing already demonstrates time-varying parameters on the fertility system
- `seed_fertility()` may need to be callable at runtime (not just startup) for
  periodic regeneration

### Performance optimization

- Ship static fertility first, measure actual cost
- Extend profiling harness (`profile_ticks`) to benchmark: world size, creature
  count, fertility on/off
- `FoodResource::grow()` internals can be optimized (SIMD, parallelization)
  without changing the API surface
- Critical before scaling to 1600×1600+ default world sizes
- For multi-food, independent `FoodResource` instances can be grown in parallel

**Review cycles:** 4

# V3 High-Level Implementation Master Plan

> **For Claude:** Each stage below begins with "Create detailed implementation plan." Use `superpowers:writing-plans` for that task, then `superpowers:executing-plans` or `superpowers:subagent-driven-development` to implement.

**Goal:** Build the complete Petri V3 simulation from greenfield, staged bottom-up by dependency, with a viability E2E test by Stage 4.

**Architecture:** V3 is a Rust workspace with two crates: `v3-core` (simulation engine with 6 modules: contracts, kernel, sensors, creature, runtime, tick) and `v3-server` (HTTP/WS transport). A `v3-cli` binary provides headless NDJSON output. All modules follow strict dependency direction: contracts <- kernel <- sensors <- runtime <- creature <- tick <- server/cli.

**Tech Stack:** Rust, Tokio, Axum, Serde, SlotMap, IEEE-754 f32 arithmetic.

**Plan Layers:** Two layers only. This master plan defines stages and boundaries. Each stage's first task creates a detailed implementation plan in `docs/plans/YYYY-MM-DD-v3-stage-N-<name>.md` with bite-sized TDD tasks, exact file paths, and code.

---

## Principles

1. **TDD throughout** — failing test first, minimal implementation, green, commit.
2. **Spec contradictions** — when implementation reveals a spec conflict, stop and surface it before proceeding. Highest-risk areas identified below.
3. **Two plan layers** — this master plan + per-stage detailed plans. No third layer. If a stage is too large during detailed planning, split into sub-stages at that level.
4. **Viability E2E by Stage 4** — 32x32 grid, 20 founders, 100 ticks, under 5 seconds. Validates the full simulation loop before building mutation/server/CLI.
5. **Consult v1/v2 as reference** — working implementations exist in `crates/` and `v2/`. Greenfield, but informed by prior art.
6. **Checkmark tracking** — detailed stage plans use `- [ ]` / `- [x]` checkmarks for every task. Update checkmarks as tasks are completed. This master plan's stage table also tracks completion status.

---

## Stage Overview

| Status | Stage | Name | Key Deliverables | Est. Complexity |
|--------|-------|------|-----------------|-----------------|
| `[ ]` | **1** | Scaffolding + Contracts + Kernel | Workspace, AGENTS.md, all shared types, grid, food, barriers, occupancy, config | Medium |
| `[ ]` | **2** | Creature Schema | Genome types, creature state, founder genome constant, parseability gate | Low-Medium |
| `[ ]` | **3a** | Sensors + VM Backend | StaticInputs assembly, input resolution, VM execution (33 opcodes), energy metering | High |
| `[ ]` | **3b** | Graph Backend + Mesh Chain | Graph relaxation, 21 operators, stateful operators, mesh chain algorithm, soft defaults | High |
| `[ ]` | **4** | Tick + Seeding + Viability E2E | Phase 0, turn queue, action application, startup seeding, minimal reproduce, E2E test | Medium-High |
| `[ ]` | **5** | Mutation + Phenotype + Evolution | MutationEngine, 3 mutation domains, phenotype mutation, full reproduction with mutation | High |
| `[ ]` | **6** | CLI + Server | v3-cli NDJSON output, v3-server HTTP/WS endpoints, lifecycle state machine | Medium |
| `[ ]` | **7** | Observability + Hardening | Evolution counters, config digest, deterministic fixtures, performance, edge cases | Low-Medium |

---

## Stage 1: Scaffolding + Contracts + Kernel

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-1-scaffolding-contracts-kernel.md`

### Deliverables

1. **Restore `AGENTS.md`** from git history (`git show 6adee98^:AGENTS.md`), add imperative to always invoke `rust-skills` skill when writing/reviewing/refactoring Rust code.

2. **Workspace scaffolding:**
   - `v3/Cargo.toml` workspace with `v3-core` and `v3-server` members
   - `v3/crates/v3-core/` with module structure: `contracts/`, `kernel/`, `sensors/`, `creature/`, `runtime/`, `tick/`
   - `v3/crates/v3-server/` (empty shell, populated in Stage 6)
   - Clippy lints, rustfmt config, `lib.rs` as export-only (per architecture lint policy)

3. **Contracts module (`contracts/`):**
   - `WorldAction` enum: `NoOp`, `Eat`, `Move(Direction)`, `Reproduce { direction: Direction, energy_transfer: f32 }`
   - `Direction` enum (8 directions + `Direction::ALL`, delta vectors)
   - `Position` struct with toroidal/bounded arithmetic
   - `CreatureId` type (slotmap key or u64)
   - `NodeId` type
   - `InputReference` enum (`World(WorldInputKey)`, `StaticIntrospection(StaticIntrospectionKey)`, `DynamicIntrospection(DynamicIntrospectionKey)`, `UpstreamSlot(usize)`)
   - All input key enums (`WorldInputKey`, `StaticIntrospectionKey`, `DynamicIntrospectionKey`)

4. **Config module (in `contracts/` or standalone):**
   - Full `SimulationConfig` struct with all ~25 fields across 5 domains (runtime, mutation, energy, population, world)
   - All defaults from `v3-runtime-config-spec.md`
   - Validation and normalization logic
   - `serde::Deserialize` for partial config overrides

5. **Kernel module (`kernel/`):**
   - `Grid<T>` generic grid with width/height
   - `WorldState` struct: food density grid (`Grid<u8>`), barrier grid (`Grid<bool>`), creature occupancy (`Grid<Option<CreatureId>>`)
   - Food growth (Bernoulli per empty cell per tick)
   - Food consumption (decrement cell, return consumed amount)
   - Food seeding (initial distribution from config + seed)
   - Coordinate system (top-left origin, 8-direction adjacency)
   - Edge mode (`Wrap` / `Bounded`) with neighbor resolution
   - Validity primitives (position bounds, occupancy checks)

### Canonical Specs
- `docs/reference/v3-world-grid-spec.md`
- `docs/reference/v3-runtime-config-spec.md`
- `docs/reference/v3-mesh-execution-spec.md` (for `WorldAction` and `InputReference` definitions)

### Tests
- Direction delta arithmetic, toroidal wrapping, bounded clamping
- Food growth probability distribution (seeded RNG)
- Grid occupancy invariants (single-occupancy)
- Config default assembly and validation
- `WorldAction` construction and field access

---

## Stage 2: Creature Schema

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-2-creature-schema.md`

### Deliverables

1. **Genome types (`creature/genome.rs`):**
   - `CreatureGenome { entry_node_id: NodeId, nodes: Vec<NodeGenome> }`
   - `NodeGenome { id: NodeId, backend: BackendDef, input_map: Vec<InputReference>, route_targets: Vec<NodeId> }`
   - `BackendDef` enum: `Vm(VmBackendDef)`, `Graph(GraphBackendDef)`
   - `VmBackendDef { program: Vec<VmInstruction>, constants: Vec<f32>, register_count: usize }`
   - `GraphBackendDef { internal_nodes: Vec<GraphInternalNode>, ... }`
   - `GraphInternalNode`, `GraphInput`, `GraphNodeKind` (all 21 operator variants)
   - `VmInstruction` enum (33 opcodes)

2. **Creature state (`creature/state.rs`):**
   - `CreatureState { id: CreatureId, genome: CreatureGenome, position: Position, energy: f32, age: u64, generation: u64, memory: [u8; 1024], graph_state: HashMap<NodeId, Vec<f32>>, phenotype_rgb: [u8; 3] }`

3. **Founder genome constant (`creature/founder.rs`):**
   - Canonical `v3alpha1` 2-node founder genome from `v3-startup-seeding-spec.md` Section 5.1
   - Node 0: Graph sensor aggregator (10 inputs, 18 internal nodes, 6 output slots)
   - Node 1: VM decision emitter (6 upstream slot inputs, 4-priority action program)

4. **Parseability gate (`creature/parseability.rs`):**
   - `ParseabilityGate::validate(genome) -> Result<(), ParseabilityError>`
   - Structural checks: entry node exists, all route targets reference existing nodes, node IDs unique
   - No behavioral/viability checks (runtime handles via soft defaults)

5. **CreatureId allocator** (simple atomic counter or slotmap-based)

### Canonical Specs
- `docs/reference/v3-genome-spec.md`
- `docs/reference/v3-startup-seeding-spec.md` (Section 5.1 for founder)
- `docs/reference/v3-vm-isa-spec.md` (for `VmInstruction` enum)
- `docs/reference/v3-graph-backend-spec.md` (for `GraphInternalNode`, `GraphNodeKind`)

### Tests
- Founder genome passes parseability gate
- Invalid genomes (missing entry, broken route targets, duplicate IDs) fail parseability
- Genome serialization/deserialization roundtrip
- CreatureState construction with defaults

---

## Stage 3a: Sensors + VM Backend

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-3a-sensors-vm.md`

### Deliverables

1. **Sensors module (`sensors/`):**
   - `StaticInputs` struct: resolved world inputs + static introspection values
   - `assemble_static_inputs(world: &WorldState, creature: &CreatureState, config: &SimulationConfig) -> StaticInputs`
   - World input resolution: `FoodHere`, `NeighborCellFood(dir)`, `NeighborCellBarrier(dir)`, `NeighborCellOccupied(dir)`
   - Static introspection: `Generation`, `AgeTicks`
   - Sensor output normalization (food density -> `[0.0, 1.0]`, booleans -> 0.0/1.0)

2. **Input resolution (`runtime/inputs.rs`):**
   - `resolve_input(reference: &InputReference, static_inputs: &StaticInputs, upstream_slots: &[f32; 12], energy: f32, energy_consumed: f32) -> f32`
   - Dynamic introspection resolved live: `EnergyCurrent`, `EnergyConsumedThisTick`

3. **VM backend (`runtime/vm.rs`):**
   - `execute_vm_node(def: &VmBackendDef, inputs: &[f32], memory: &mut [u8; 1024], energy_budget: &mut f32, config: &RuntimeConfig) -> NodeResult`
   - 33-opcode execution loop with program counter
   - IEEE-754 f32 registers, `sanitize_f32()` after every operation
   - Operand normalization: `rem_euclid` wrapping for register indices, memory addresses, jump targets
   - Output lifecycle: `SetOutput(slot, reg)` -> payload buffer, `SetMeta(slot, reg)` -> metadata buffer
   - `EmitWorldAction(action_type_reg)` -> immediate halt, decode action from payload+metadata
   - `SetRouteTarget(reg)` -> route target override
   - Energy metering: per-opcode cost from energy cost table, halt on exhaustion
   - Memory operations: `LoadMem8`, `StoreMem8`, `LoadMem16`, `StoreMem16`
   - `max_vm_steps` enforcement (default 256)

4. **WorldAction decode (`runtime/action_decode.rs`):**
   - Decode `WorldAction` from action type + payload buffer + metadata buffer
   - Direction decode: `meta[0].round().clamp(0.0, 7.0) as usize`
   - Energy transfer for Reproduce: `payload[0]`

5. **NodeResult type (`runtime/types.rs`):**
   - `NodeResult { output_slots: [f32; 12], route_target_idx: f32, world_action: Option<WorldAction> }`

### Canonical Specs
- `docs/reference/v3-sensor-spec.md`
- `docs/reference/v3-vm-isa-spec.md`
- `docs/reference/v3-mesh-execution-spec.md` (Sections 2-3 for NodeResult, output lifecycle)

### Risk Area
- **Sensor normalization vs raw values**: food density normalized to `[0.0, 1.0]` in sensors, but eat reward uses raw consumed amount. Don't normalize in the wrong place.
- **VM direction encoding**: founder genome's direction selection is intentionally imperfect. Don't expect optimal behavior in tests.

### Tests
- Each of the 33 opcodes individually
- `sanitize_f32` (NaN, Inf, -Inf, subnormals)
- Operand wrapping (`rem_euclid` for all index types)
- Output lifecycle (SetOutput -> EmitWorldAction -> correct WorldAction)
- Energy exhaustion halts VM
- `max_vm_steps` enforcement
- Memory load/store roundtrip
- Single-node VM genome producing correct WorldAction for known inputs
- StaticInputs assembly from known world state

---

## Stage 3b: Graph Backend + Mesh Chain

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-3b-graph-mesh.md`

### Deliverables

1. **Graph backend (`runtime/graph.rs`):**
   - `execute_graph_node(def: &GraphBackendDef, inputs: &[f32], graph_state: &mut HashMap<NodeId, Vec<f32>>, energy_budget: &mut f32, config: &RuntimeConfig) -> NodeResult`
   - Relaxation loop with `max_graph_relax_iters` (default 4) passes
   - Gauss-Seidel evaluation order (forward nodes use current-pass values)
   - Convergence detection (delta < epsilon, default 0.001)
   - All 21 `GraphNodeKind` operators:
     - Stateless: `Sum`, `Product`, `Max`, `Min`, `Mean`, `Abs`, `Negate`, `Sigmoid`, `Tanh`, `ReLU`, `Step`, `Sine`, `Clamp`, `Gate`, `Interpolate`
     - Stateful: `DecayIntegrator`, `Momentum`, `Oscillator`, `AdaptiveGain`
     - Outputs: `CustomOutput(slot_index)`, `RouterOutput`
   - `graph_state` persistence: stateful operator state stored per-node across ticks
   - Energy metering: per-internal-node-per-pass cost

2. **Mesh chain algorithm (`runtime/mesh.rs`):**
   - `execute_creature_mesh(genome: &CreatureGenome, static_inputs: &StaticInputs, creature_state: &mut CreatureState, config: &SimulationConfig) -> WorldAction`
   - Chain algorithm: entry_node -> evaluate -> check action -> route to next -> repeat
   - Routing: `route_targets[route_target_idx.round() as usize % route_targets.len()]`
   - `max_mesh_hops` enforcement (default 128)
   - Hop-visited tracking (optional cycle detection)

3. **Soft-default matrix (`runtime/mesh.rs`):**
   - All 11 conditions from `v3-mesh-execution-spec.md` Section 4:
     - Missing entry node -> NoOp
     - Empty node list -> NoOp
     - Node evaluation crash -> NoOp for that node
     - Route target not found -> NoOp (terminate chain)
     - Empty route targets -> NoOp (terminate)
     - Max hops exceeded -> NoOp
     - Energy exhausted mid-chain -> NoOp
     - etc.

### Canonical Specs
- `docs/reference/v3-graph-backend-spec.md`
- `docs/reference/v3-mesh-execution-spec.md`

### Risk Areas
- **Graph evaluation order**: Gauss-Seidel is order-dependent. Mutation can reorder internal nodes, changing outputs for same weights.
- **Stateful operator convergence**: `DecayIntegrator` and `Momentum` modify state each pass. Convergence detection must account for stateful updates.

### Tests
- Each of the 21 GraphNodeKind operators individually
- Relaxation convergence for simple known graphs
- Stateful operator persistence across ticks (graph_state)
- Full mesh chain with 2-node founder genome (Graph -> VM)
- Soft-default matrix: all 11 degraded conditions produce NoOp without panic
- Topology patterns: single VM node, single Graph node, Graph->VM chain, multi-node dispatcher, junk DNA (dangling nodes)
- Energy exhaustion mid-chain
- Max hops exceeded

---

## Stage 4: Tick Orchestration + Startup Seeding + Viability E2E

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-4-tick-seeding-viability.md`

### Deliverables

1. **Phase 0 (`tick/phase0.rs`):**
   - Food growth (Bernoulli per empty cell)
   - Creature aging (increment age)
   - Energy decay (subtract `energy_decay_per_tick`)
   - Death removal (energy <= 0.0 -> remove creature, free grid cell)

2. **Turn queue (`tick/queue.rs`):**
   - Collect surviving creatures
   - Stable sort by CreatureId
   - Shuffle with tick-seeded RNG

3. **Action application (`tick/apply.rs`):**
   - `NoOp`: nothing
   - `Eat`: decrement food cell, add `eat_reward_per_food * consumed` to energy, cap at `max_energy`
   - `Move(dir)`: resolve target cell, check occupancy + barriers, update position + occupancy grid
   - `Reproduce(dir, energy)`: resolve target cell, check occupancy, enforce `min_reproduce_energy` gate, pay `reproduce_cost`, transfer energy, clone genome (no mutation yet), spawn offspring
   - First-processed-wins arbitration (sequential application)

4. **Tick orchestration (`tick/mod.rs`):**
   - `run_tick(world: &mut WorldState, creatures: &mut CreatureStore, config: &SimulationConfig, tick: u64, rng: &mut impl Rng)`
   - Phase 0 -> build queue -> for each creature: assemble inputs -> execute mesh -> apply action
   - Newborns not eligible until next tick

5. **Startup seeding (`tick/startup.rs` or `kernel/startup.rs`):**
   - `seed_simulation(config: &SimulationConfig, seed: u64) -> (WorldState, CreatureStore)`
   - Food seeding from config + seed
   - Founder placement: N founders with canonical `v3alpha1` genome, placed at random non-barrier positions
   - Initial energy, age=0, generation=0, memory zeroed, graph_state empty
   - Determinism contract: identical seed + config -> identical initial state

6. **Viability E2E test (`tests/viability.rs` or `v3-core/tests/`):**
   - 32x32 grid, 20 founders, seed=42, 100 ticks
   - **Asserts:**
     - Simulation runs without panic
     - Population changes (some creatures die from energy decay)
     - Creatures move (positions change between ticks)
     - Creatures eat (energy increases for creatures on food cells)
     - Food is consumed and regrows
     - Creatures reproduce (population can increase)
     - Tick count advances correctly
     - Phase 0 runs correctly (aging, energy decay, death)
   - **Performance:** completes in under 5 seconds
   - **Determinism:** same seed produces same outcome

### Canonical Specs
- `docs/reference/v3-tick-orchestration-spec.md`
- `docs/reference/v3-startup-seeding-spec.md`
- `docs/reference/v3-reproduction-spec.md` (for minimal reproduce without mutation)

### Risk Areas
- **Energy transfer sequencing**: reproduce_cost is paid *before* min_reproduce_energy check. Effective threshold is `min_reproduce_energy + reproduce_cost`. The founder's Graph threshold (24.0) may not align perfectly with config defaults.
- **Reproduce without mutation**: Stage 4 reproduce clones genome directly. Stage 5 adds mutation. Make sure the reproduce action handler is designed to accept an optional mutation step.

---

## Stage 5: Mutation + Phenotype + Full Evolution Loop

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-5-mutation-evolution.md`

### Deliverables

1. **Mutation engine (`creature/mutation.rs`):**
   - `MutationEngine::mutate(genome: &CreatureGenome, rng: &mut impl Rng, config: &MutationConfig) -> (CreatureGenome, MutationSummary)`
   - Trigger: per-reproduction roll against `mutation_rate`
   - Event count: geometric distribution from `mutation_event_rate`
   - Domain-operator sampling: weighted selection across 3 domains
   - `MutationSummary { events_applied: Vec<MutationEvent>, domains_touched: HashSet<MutationDomain> }`

2. **Topology mutations (`creature/mutation/topology.rs`):**
   - `AddNode`, `RemoveNode`, `RetargetRoute`, `ChangeEntryNode`
   - Post-mutation parseability gate enforcement

3. **VM mutations (`creature/mutation/vm.rs`):**
   - `MutateInstruction`, `InsertInstruction`, `RemoveInstruction`, `MutateConstant`, `MutateRegisterCount`

4. **Graph mutations (`creature/mutation/graph.rs`):**
   - `AddInternalNode`, `RemoveInternalNode`, `MutateEdgeWeight`, `MutateOperator`, `AddEdge`, `RemoveEdge`

5. **Phenotype mutation (`creature/phenotype.rs`):**
   - Triggered when any genome mutation events applied
   - Weighted-channel random walk on RGB values
   - Heritable channel weights and polarity direction

6. **Full reproduction integration:**
   - Update Stage 4's reproduce action to call mutation engine
   - Offspring draft: genome = mutated parent genome, memory = parent memory copy, graph_state = reset, energy = transferred amount, phenotype = mutated if genome events applied

7. **Evolution E2E test:**
   - Extend viability test to 500+ ticks
   - Assert: offspring genomes differ from parents (mutation applied)
   - Assert: phenotype variation emerges in population
   - Assert: population sustains across generations (not all die)

### Canonical Specs
- `docs/reference/v3-mutation-spec.md`
- `docs/reference/v3-reproduction-spec.md`
- `docs/reference/v3-phenotype-spec.md`

---

## Stage 6: CLI + Server

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-6-cli-server.md`

**Build CLI first** — it's simpler and exercises `v3-core` end-to-end without HTTP complexity.

### Deliverables

1. **v3-cli (`v3/crates/v3-cli/`):**
   - `v3-cli run --ticks N --sample-every M --seed S [--config path]`
   - NDJSON output: `run_started`, `tick_sample`, `run_completed`
   - Per `v3-cli-contract-spec.md`

2. **v3-server (`v3/crates/v3-server/`):**
   - HTTP endpoints: `POST /v3/simulation/startup`, `POST /v3/simulation/start|pause|step`, `GET /v3/simulation/status|frame|config`, `PATCH /v3/simulation/config`
   - WebSocket: `GET /v3/ws` with `status`, `frame`, `health` event types
   - Lifecycle state machine: `uninitialized -> idle -> running -> paused`
   - Error envelope with `validation_rejected`, `conflict`, etc.
   - Axum + Tokio runtime

### Canonical Specs
- `docs/reference/v3-cli-contract-spec.md`
- `docs/reference/v3-server-api-protocol-spec.md`

---

## Stage 7: Observability + Hardening + Polish

**First task:** Create detailed plan at `docs/plans/YYYY-MM-DD-v3-stage-7-observability-hardening.md`

### Deliverables

1. **Evolution observability counters** (per `v3-evolution-observability-spec.md`):
   - Mutation event counters by domain and operator
   - Death reason counters (energy, age, etc.)
   - Action counters by type
   - Population/energy/age statistics per tick

2. **Config digest**: SHA-256 of effective config for reproducibility verification

3. **Deterministic test fixtures**: seed-based regression tests with golden output

4. **Performance profiling**: identify hot paths, optimize if needed

5. **Edge case hardening**: fuzz testing for genome parsing, VM execution, graph evaluation

6. **Documentation sync**: verify all specs match implemented behavior, update any contradictions found during implementation

---

## Spec Contradiction Risk Areas (Watch List)

These are the areas most likely to surface spec conflicts during implementation. When hit, stop and discuss before proceeding.

| Risk | Area | Details |
|------|------|---------|
| HIGH | Energy transfer in reproduce | `reproduce_cost` paid before `min_reproduce_energy` check. Effective threshold = `min_reproduce_energy + reproduce_cost`. Founder threshold (24.0) vs config defaults may not align. |
| HIGH | Sensor normalization | Food density normalized to `[0.0, 1.0]` in sensors, but eat reward uses raw consumed amount. Don't normalize in wrong place. |
| MEDIUM | Graph evaluation order | Gauss-Seidel is order-dependent. Mutation reordering internal nodes changes outputs. By design, but surprising. |
| MEDIUM | VM direction encoding | Founder VM's direction selection is intentionally imperfect. Don't expect optimal behavior. |
| MEDIUM | WorldAction direction rounding | `.round()` in Rust is ties-away-from-zero. Verify spec consistency. |
| LOW | Single-node graph convergence | Graph with 1 internal node converges immediately (delta check over empty range). Correct but subtle. |

---

## Quality Gates (Applied to Every Stage)

Per `AGENTS.md` and `docs/standards/`:

1. `scripts/check-plan-harness.sh --mode strict` — all plans
2. `scripts/check-architecture-harness.sh --mode strict` — crate structure (strict from Feb 28)
3. `scripts/check-doc-harness.sh --mode strict` — doc consistency (strict from Feb 28)
4. `cargo fmt --all --check`
5. `cargo test --workspace`
6. `cargo clippy --workspace --all-targets -- -D warnings`
7. Intent verification: every feature has contract + intent + integration + negative-path tests
8. Runtime behavior realism: no synthetic counters or placeholder values

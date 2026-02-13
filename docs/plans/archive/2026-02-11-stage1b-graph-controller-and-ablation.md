# Stage 1b Graph Controller and Ablation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace random-walk behavior with graph-evaluated actions and add a stability ablation harness for neural-only, logic-only, and hybrid palettes.

**Architecture:** Add a new pure crate `petri-graph` that owns graph nodes/edges, sensor inputs, and action outputs. Integrate one graph per creature in `petri-core` so `tick()` uses evaluated outputs for move/eat/reproduce. Extend `petri-cli` with an `ablation` subcommand that runs each palette under identical config and prints comparable diagnostics.

**Tech Stack:** Rust workspace crates (`petri-core`, `petri-graph`, `petri-cli`), `serde`, `rand`, deterministic seeded simulation.

### Task 1: Add `petri-graph` crate and failing evaluator tests

**Files:**
- Create: `crates/petri-graph/Cargo.toml`
- Create: `crates/petri-graph/src/lib.rs`
- Create: `crates/petri-graph/src/types.rs`
- Create: `crates/petri-graph/src/eval.rs`
- Modify: `Cargo.toml`

**Step 1: Write failing tests**

Add tests for:
- neural palette produces bounded outputs in `[-1.0, 1.0]`
- logic palette outputs deterministic eat/reproduce gating from food/energy sensors
- hybrid palette yields non-zero motion intent with random sensor changes

**Step 2: Run test to verify failure**

Run: `cargo test -p petri-graph`
Expected: FAIL (crate / evaluator not implemented)

**Step 3: Implement minimal graph engine**

- Node kinds: `Input`, `Constant`, `Add`, `Multiply`, `Threshold`, `GreaterThan`, `Sigmoid`, `Tanh`, `Select`, `Output`.
- Edge list with weighted directed connections.
- Acyclic evaluation order with per-node activation array.
- `ControllerPalette` factory methods:
  - `neural_only()`
  - `logic_only()`
  - `hybrid()`
- `evaluate(inputs) -> ActionOutputs`.

**Step 4: Re-run tests**

Run: `cargo test -p petri-graph`
Expected: PASS

### Task 2: Integrate graph controllers into `petri-core`

**Files:**
- Modify: `crates/petri-core/Cargo.toml`
- Modify: `crates/petri-core/src/lib.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-core/src/types.rs`

**Step 1: Write failing core tests**

Add/adjust tests to prove:
- creatures move because controller outputs move intents (not hardcoded random walk)
- eat happens only when eat intent is asserted and food exists
- reproduce happens only when reproduce intent is asserted and energy threshold is met
- world exposes per-tick diagnostics counters (`moves`, `eats`, `reproductions`, `deaths`)

**Step 2: Run test to verify failure**

Run: `cargo test -p petri-core`
Expected: FAIL (controller integration + diagnostics missing)

**Step 3: Implement core integration**

- Add controller field per creature (`ComputationGraph`) and palette-aware spawn path.
- Add `World::new_with_palette(config, seed, palette)` while keeping `World::new` default.
- Build sensor vector each tick from local world state (`food_here`, normalized energy, random).
- Replace hardcoded random actions with graph outputs:
  - `OutputMoveX`, `OutputMoveY` -> direction step
  - `OutputEat` -> consume food gate
  - `OutputReproduce` -> reproduction gate
- Charge compute energy proportional to evaluated node count.
- Record counters into a world diagnostics struct.

**Step 4: Re-run tests**

Run: `cargo test -p petri-core`
Expected: PASS

### Task 3: Add CLI ablation harness

**Files:**
- Modify: `crates/petri-cli/src/main.rs`

**Step 1: Write failing CLI tests**

Add tests for:
- ablation runner returns three palette results
- each result includes palette, final population, avg energy, and action counters
- output formatting includes all palette names

**Step 2: Run test to verify failure**

Run: `cargo test -p petri-cli`
Expected: FAIL (`ablation` mode missing)

**Step 3: Implement ablation mode**

- Add CLI subcommands:
  - `run` (existing behavior)
  - `ablation` (new)
- Run three simulations with identical config/seed:
  - `NeuralOnly`
  - `LogicOnly`
  - `Hybrid`
- Print diagnostics table with:
  - population
  - avg energy
  - moves / eats / reproductions / deaths

**Step 4: Re-run tests**

Run: `cargo test -p petri-cli`
Expected: PASS

### Task 4: End-to-end verification and docs update

**Files:**
- Modify: `README.md`

**Step 1: Add Stage 1b run commands**

- `cargo run -p petri-cli -- run --ticks 200`
- `cargo run -p petri-cli -- ablation --ticks 500`

**Step 2: Run verification**

Run:
- `cargo test --workspace`
- `cargo run -p petri-cli -- ablation --ticks 100 --sample-every 25`
- `cd web && npm run build`

Expected:
- all tests green
- ablation prints three palette diagnostics
- web build remains green

**Step 3: Commit**

```bash
git add Cargo.toml crates README.md docs/plans/2026-02-11-stage1b-graph-controller-and-ablation.md
git commit -m "feat: add stage1b graph controllers and ablation harness"
```

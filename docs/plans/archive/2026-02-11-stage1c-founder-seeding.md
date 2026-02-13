# Stage 1c Founder Seeding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Seed new worlds with a viable, basic foraging program and controlled variation so initial populations can survive long enough for evolution dynamics to matter.

**Architecture:** Add an explicit founder-controller factory in `petri-graph`, plus a small structural-preserving mutation routine that perturbs edge weights and numeric node parameters. Integrate this into `petri-core` creature creation for initial spawn and reproduction, and add deterministic survival-oriented regression checks in core/CLI to ensure behavior is mechanically viable.

**Tech Stack:** Rust workspace crates (`petri-graph`, `petri-core`, `petri-cli`), deterministic RNG seeding, existing TDD workflow.

### Task 1: Founder controller API in `petri-graph`

**Files:**
- Modify: `crates/petri-graph/src/eval.rs`
- Modify: `crates/petri-graph/src/lib.rs`

**Step 1: Write failing tests**

Add tests proving:
- founder controller eats when food is present and suppresses eating when absent
- founder reproduction gate depends on high energy threshold
- founder motion is bounded and low-cost by default (reduced move impulses)

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-graph`
Expected: FAIL (founder API missing)

**Step 3: Implement minimal founder API**

- Add `ComputationGraph::founder(palette)` returning tuned baseline graphs per palette.
- Tune thresholds/weights to prefer foraging and delayed reproduction.
- Keep graph acyclic and output bounded as existing evaluator expects.

**Step 4: Re-run tests**

Run: `cargo test -p petri-graph`
Expected: PASS

### Task 2: Controlled mutation API in `petri-graph`

**Files:**
- Modify: `crates/petri-graph/src/eval.rs`
- Modify: `crates/petri-graph/src/lib.rs`

**Step 1: Write failing tests**

Add tests proving:
- mutation changes some numeric parameters with non-zero magnitude
- mutation does not alter node/edge topology counts
- mutated graph still evaluates to bounded outputs

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-graph`
Expected: FAIL (mutation API missing)

**Step 3: Implement minimal mutation**

- Add `mutate_weights(&mut self, rng, rate, magnitude)` to perturb:
  - edge weights
  - `Constant(v)` and `Threshold(t)` values
- Clamp to safe ranges to avoid runaway activations.

**Step 4: Re-run tests**

Run: `cargo test -p petri-graph`
Expected: PASS

### Task 3: Integrate founder seeding + mutation in `petri-core`

**Files:**
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-core/src/lib.rs` (if additional exports needed)

**Step 1: Write failing core tests**

Add tests proving:
- initial creatures are seeded from founder template with variation (not all identical numerics)
- offspring controller differs from parent due small mutation
- deterministic survival regression (`seed`, config, ticks) keeps population above zero

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-core`
Expected: FAIL (founder+mutation integration absent)

**Step 3: Implement core integration**

- Replace `ComputationGraph::from_palette(...)` seeding with `ComputationGraph::founder(...)`.
- Apply small mutation at:
  - initial world seeding (low magnitude)
  - offspring creation (slightly higher magnitude)
- Keep deterministic behavior using per-creature RNG seeds.

**Step 4: Re-run tests**

Run: `cargo test -p petri-core`
Expected: PASS

### Task 4: CLI regression visibility

**Files:**
- Modify: `crates/petri-cli/src/main.rs`
- Modify: `README.md`

**Step 1: Write failing CLI test**

Add a test for ablation output that checks non-zero survival for at least the default hybrid run under deterministic short-horizon config.

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-cli`
Expected: FAIL (survival expectation not met)

**Step 3: Implement minimal update**

- Add deterministic short-horizon survival assertion in tests (not runtime hard-fail path).
- Keep runtime output format stable.
- Update README with founder-seeding note.

**Step 4: Re-run tests**

Run: `cargo test -p petri-cli`
Expected: PASS

### Task 5: End-to-end verification and commit

**Files:**
- Modify: `docs/plans/2026-02-11-stage1c-founder-seeding.md` (if adjustments needed)

**Step 1: Run full verification**

Run:
- `cargo test --workspace`
- `cargo run -p petri-cli -- ablation --ticks 100 --sample-every 25`
- `cd web && npm run build`

Expected:
- tests green
- ablation shows improved baseline viability for tuned founder behavior
- web build green

**Step 2: Commit**

```bash
git add Cargo.toml crates README.md docs/plans/2026-02-11-stage1c-founder-seeding.md
git commit -m "feat: add viable founder seeding and controller mutation"
```

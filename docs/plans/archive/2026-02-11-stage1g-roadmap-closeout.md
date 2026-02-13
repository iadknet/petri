# Stage 1g Roadmap Closeout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Close all remaining Stage 1 roadmap gaps so Stage 1 can be declared complete against `petri-roadmap.md`.

**Architecture:** Keep `petri-core` as simulation policy, `petri-graph` as graph/mutation mechanics, `petri-server` as transport/API, and `web/` as consumer UI. Add missing Stage 1 capabilities incrementally with TDD: first lock regressions, then implement the smallest behavior needed, then verify full gates. Preserve current startup/runtime control flow from Stage 1e while extending contracts to full Stage 1 knobs.

**Tech Stack:** Rust workspace crates (`petri-core`, `petri-graph`, `petri-server`, `petri-cli`), React + TypeScript + Vite frontend, Vitest + Testing Library + MSW + Playwright smoke.

## Scope Lock (What This Plan Completes)

1. Fix Stage 1f e2e verification instability (`npm run test:e2e`).
2. Add missing Stage 1 sensor/node types (`food direction`, `food distance`, `relu`).
3. Add Stage 1 mutation operators and make mutation knobs configurable.
4. Add lineage identifiers and parent-child tracking.
5. Add `world_wrap` toggle and bounded-edge behavior.
6. Make full Stage 1 config knobs editable from frontend/server contracts.
7. Add creature click-to-inspect panel and live population/energy charts.
8. Add full simulation save/load support.
9. Add measurable 5k-creature performance evidence and run completion gates.

## Assumptions and Defaults

- `petri-roadmap.md` is the source of truth for Stage 1 completion.
- Keep the current startup draft model; knobs that cannot be safely hot-swapped are marked restart-required.
- Snapshot save/load uses API payloads (`GET/POST /simulation/snapshot`) rather than server-local file paths.
- `world_wrap` default remains `true` to preserve current behavior.
- Mutation settings live in `WorldConfig` and are applied by `petri-core` when mutating founders/offspring.

## Execution Status (2026-02-11)

- [x] Task 1: Stabilize Stage 1f verification baseline
- [x] Task 2: Add missing Stage 1 sensors and node types
- [x] Task 3: Implement Stage 1 mutation operators and knobs
- [x] Task 4: Add lineage and parent-child tracking
- [x] Task 5: Add `world_wrap` runtime behavior
- [x] Task 6: Complete full Stage 1 config control surface
- [x] Task 7: Add creature inspector + live charts
- [x] Task 8: Add simulation snapshot save/load
- [x] Task 9: Performance evidence and completion gate

Benchmark evidence:
- `cargo run -p petri-cli --bin stage1_benchmark -- --ticks 200 --assert-min --min-ticks-per-second 30`
- Result: `ticks_per_second=106.38` on 200x200 with 5,000 creatures (seed 42)

Completion gate:
- `cargo fmt --all --check` ✅
- `cargo test --workspace` ✅
- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `cd web && npm run test` ✅
- `cd web && npm run test:e2e` ✅
- `cd web && npm run build` ✅

### Task 1: Stabilize Stage 1f Verification Baseline

**Files:**
- Modify: `web/e2e/smoke.spec.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `crates/petri-server/src/app_state.rs` (only if startup viability default is actually non-viable)
- Test: `web/e2e/smoke.spec.ts`

**Step 1: Reproduce failing smoke test**

Run: `cd web && npm run test:e2e`
Expected: currently FAILS with disabled `Start Simulation` button.

**Step 2: Add deterministic wait/assertions for startup-ready state in e2e**

- Update smoke flow to wait for status hydration (`phase === idle` + startup viability known) before asserting start button enabled.
- Keep assertion strict: `Start Simulation` must become enabled for viable default draft.

**Step 3: If needed, fix startup viability default draft**

- Only if server reports `startup_viable=false` for default draft, tune `StartupDraft::viable_default()` to deterministic viable parameters.
- Add/adjust server unit test proving default startup draft is viable.

**Step 4: Verify smoke and web unit tests**

Run:
- `cd web && npm run test`
- `cd web && npm run test:e2e`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/e2e/smoke.spec.ts web/src/features/simulation/store/simulationStore.ts crates/petri-server/src/app_state.rs
git commit -m "test: stabilize stage1 smoke lifecycle verification"
```

### Task 2: Add Missing Stage 1 Sensors and Node Types

**Files:**
- Modify: `crates/petri-graph/src/types.rs`
- Modify: `crates/petri-graph/src/eval.rs`
- Modify: `crates/petri-graph/src/lib.rs`
- Modify: `crates/petri-core/src/world.rs`
- Test: `crates/petri-graph/src/lib.rs` (sensor/node evaluation tests)
- Test: `crates/petri-core/src/world.rs` (sensor wiring regression tests)

**Step 1: Write failing graph tests**

- Add tests that fail until:
  - `SensorFoodDirection` and `SensorFoodDistance` are represented and consumed.
  - `Relu` node type evaluates correctly.

**Step 2: Extend graph contracts**

- Add `food_direction` and `food_distance` fields to `SensorInputs`.
- Add `InputFoodDirection`, `InputFoodDistance`, and `Relu` to `NodeKind`.
- Implement eval semantics:
  - direction in `[-1.0, 1.0]` normalized angle.
  - distance in `[0.0, 1.0]` normalized radius.
  - relu as `max(0, x)`.

**Step 3: Wire sensors in world tick**

- Compute nearest-food direction/distance per creature before controller evaluate.
- Feed new values into `SensorInputs`.

**Step 4: Update founder palettes**

- Ensure founder graph templates include/route new inputs in meaningful paths.

**Step 5: Verify**

Run: `cargo test -p petri-graph && cargo test -p petri-core`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/petri-graph/src/types.rs crates/petri-graph/src/eval.rs crates/petri-graph/src/lib.rs crates/petri-core/src/world.rs
git commit -m "feat: add stage1 food direction distance sensors and relu node"
```

### Task 3: Implement Stage 1 Mutation Operators and Knobs

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-graph/src/eval.rs`
- Modify: `crates/petri-graph/src/lib.rs`
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/api/simulationApiClient.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Modify: `web/src/features/simulation/components/RuntimeTuningPanel.tsx`
- Test: `crates/petri-graph/src/lib.rs`
- Test: `crates/petri-core/src/world.rs`
- Test: `crates/petri-server/src/lib.rs`
- Test: `web/src/App.test.tsx`

**Step 1: Add failing mutation operator tests**

- Add one failing regression test per operator:
  - perturb weights/params (existing)
  - add hidden node via edge splice
  - add edge
  - remove edge
  - remove disconnected hidden node
  - change hidden node type

**Step 2: Add mutation config knobs to `WorldConfig`**

- Add:
  - `weight_mutation_rate`
  - `weight_mutation_magnitude`
  - `logic_node_mutation_rate`
  - `structural_mutation_rate`
- Set conservative defaults to preserve current founder viability.

**Step 3: Implement configurable mutation pipeline**

- Add a single `mutate_with_config` entrypoint in `ComputationGraph`.
- Use `structural_mutation_rate` to gate topology operations.
- Use `logic_node_mutation_rate` for type-change operations on logic nodes.

**Step 4: Replace hardcoded mutation constants in `World`**

- Use config-driven mutation for startup founder variation and offspring mutation.

**Step 5: Expose knobs server + frontend**

- Extend startup draft/runtime patch contracts as appropriate.
- Add validation ranges in server state.
- Add frontend sliders/inputs and tests.

**Step 6: Verify**

Run:
- `cargo test -p petri-graph`
- `cargo test -p petri-core`
- `cargo test -p petri-server`
- `cd web && npm run test`
Expected: PASS.

**Step 7: Commit**

```bash
git add crates/petri-core/src/config.rs crates/petri-core/src/world.rs crates/petri-graph/src/eval.rs crates/petri-graph/src/lib.rs crates/petri-server/src/app_state.rs web/src/protocol.ts web/src/features/simulation/api/simulationApiClient.ts web/src/features/simulation/store/simulationStore.ts web/src/features/simulation/components/StartupDraftPanel.tsx web/src/features/simulation/components/RuntimeTuningPanel.tsx web/src/App.test.tsx
git commit -m "feat: implement stage1 structural mutation operators and tuning knobs"
```

### Task 4: Add Lineage and Parent-Child Tracking

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-core/src/lib.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/protocol.test.ts`
- Test: `crates/petri-core/src/world.rs`

**Step 1: Add failing lineage tests**

- Test offspring inherits lineage id, has unique creature id, and records parent id.
- Test generation increments remain correct.

**Step 2: Extend core creature/frame model**

- Add `lineage_id` and `parent_id` to creature internal state and `CreatureSnapshot`.
- Ensure founder creatures get unique lineage ids and no parent.

**Step 3: Build in-memory lineage tree structure**

- Add a world-owned lineage map storing parent-child links for future Stage 2 visualization.

**Step 4: Expose lineage fields over wire**

- Update frame serialization and TS types.

**Step 5: Verify**

Run:
- `cargo test -p petri-core`
- `cd web && npm run test`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/petri-core/src/types.rs crates/petri-core/src/world.rs crates/petri-core/src/lib.rs web/src/protocol.ts web/src/protocol.test.ts
git commit -m "feat: add stage1 lineage ids and parent child tracking"
```

### Task 5: Add `world_wrap` Runtime Behavior

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Test: `crates/petri-core/src/world.rs`
- Test: `crates/petri-server/src/lib.rs`

**Step 1: Add failing movement-edge tests**

- Test with `world_wrap=true`: creature crossing edge wraps.
- Test with `world_wrap=false`: creature clamps/stays in-bounds without wrapping.

**Step 2: Extend config and world movement helpers**

- Add `world_wrap: bool` to config with default `true`.
- Replace unconditional `wrap_axis` with wrap-or-clamp logic for movement and neighbor selection.

**Step 3: Expose in startup config surface**

- Add startup draft patch handling and UI control for `world_wrap`.

**Step 4: Verify**

Run: `cargo test -p petri-core && cargo test -p petri-server && cd web && npm run test`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/petri-core/src/config.rs crates/petri-core/src/world.rs crates/petri-server/src/app_state.rs web/src/protocol.ts web/src/features/simulation/components/StartupDraftPanel.tsx
git commit -m "feat: add world wrap toggle and bounded world behavior"
```

### Task 6: Complete Full Stage 1 Config Control Surface

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/api.rs`
- Modify: `crates/petri-server/src/lib.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/api/simulationApiClient.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Modify: `web/src/features/simulation/components/RuntimeTuningPanel.tsx`
- Add: `web/src/features/simulation/components/AdvancedConfigPanel.tsx`
- Test: `crates/petri-server/src/lib.rs`
- Test: `web/src/App.test.tsx`

**Step 1: Add failing contract tests**

- Server tests for patching all Stage 1 knobs with correct validation and restart semantics.
- Frontend tests for rendering/editing all knobs.

**Step 2: Define full knob ownership**

- Immediate runtime knobs: values safe to apply to active world each tick.
- Restart-required knobs: values that change world shape/initialization behavior.
- Surface restart-required edits via `pending_restart=true`.

**Step 3: Expand API and frontend contracts**

- Include all roadmap Stage 1 knobs in startup/runtime payloads.
- Add typed helpers for scalar, boolean, and range controls.

**Step 4: Verify**

Run:
- `cargo test -p petri-server`
- `cd web && npm run test`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/petri-server/src/app_state.rs crates/petri-server/src/api.rs crates/petri-server/src/lib.rs web/src/protocol.ts web/src/features/simulation/api/simulationApiClient.ts web/src/features/simulation/store/simulationStore.ts web/src/features/simulation/components/StartupDraftPanel.tsx web/src/features/simulation/components/RuntimeTuningPanel.tsx web/src/features/simulation/components/AdvancedConfigPanel.tsx web/src/App.test.tsx
git commit -m "feat: expose complete stage1 config knob set in server and frontend"
```

### Task 7: Add Creature Inspector + Live Charts

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `web/package.json`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/app/App.tsx`
- Modify: `web/src/features/simulation/components/ViewportCanvas.tsx`
- Add: `web/src/features/simulation/components/CreatureInspectorPanel.tsx`
- Add: `web/src/features/simulation/components/PopulationEnergyChart.tsx`
- Test: `web/src/App.test.tsx`
- Add: `web/src/features/simulation/components/PopulationEnergyChart.test.tsx`

**Step 1: Add failing UI tests**

- Click creature in viewport shows inspector with energy, age, generation, node count.
- Live chart renders population + avg energy series updates over frames.

**Step 2: Extend frame payload for inspector**

- Add `node_count` to `CreatureSnapshot`.
- Keep payload compact (u16/u32-friendly values where possible).

**Step 3: Implement click-to-inspect**

- Map canvas click to world coordinates using current pan/zoom transform.
- Resolve creature in current frame at clicked cell.

**Step 4: Add uPlot chart module**

- Add `uplot` dependency.
- Implement rolling time-series chart for population and average energy.

**Step 5: Verify**

Run:
- `cd web && npm run test`
- `cd web && npm run build`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/petri-core/src/types.rs web/package.json web/src/protocol.ts web/src/app/App.tsx web/src/features/simulation/components/ViewportCanvas.tsx web/src/features/simulation/components/CreatureInspectorPanel.tsx web/src/features/simulation/components/PopulationEnergyChart.tsx web/src/features/simulation/components/PopulationEnergyChart.test.tsx web/src/App.test.tsx
git commit -m "feat: add creature inspector and live stage1 metrics charts"
```

### Task 8: Add Simulation Snapshot Save/Load

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-core/src/lib.rs`
- Modify: `crates/petri-server/src/api.rs`
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/lib.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/api/simulationApiClient.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Add: `web/src/features/simulation/components/SnapshotPanel.tsx`
- Test: `crates/petri-core/src/world.rs`
- Test: `crates/petri-server/src/lib.rs`
- Test: `web/src/App.test.tsx`

**Step 1: Add failing round-trip tests in core**

- Serialize world snapshot -> deserialize -> assert equivalent core state (tick, creatures, food, diagnostics, config, lineage).

**Step 2: Implement core snapshot model**

- Add `WorldSnapshot` with serde support.
- Add `World::snapshot()` and `World::from_snapshot(...)`.

**Step 3: Add server API**

- `GET /simulation/snapshot` returns current snapshot (or startup template in idle).
- `POST /simulation/snapshot` loads snapshot and updates simulation state/run metadata.

**Step 4: Add minimal frontend controls**

- Expose copy/paste JSON snapshot import/export in `SnapshotPanel`.
- Show parse and server validation errors clearly.

**Step 5: Verify**

Run:
- `cargo test -p petri-core`
- `cargo test -p petri-server`
- `cd web && npm run test`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/petri-core/src/types.rs crates/petri-core/src/world.rs crates/petri-core/src/lib.rs crates/petri-server/src/api.rs crates/petri-server/src/app_state.rs crates/petri-server/src/lib.rs web/src/protocol.ts web/src/features/simulation/api/simulationApiClient.ts web/src/features/simulation/store/simulationStore.ts web/src/features/simulation/components/SnapshotPanel.tsx web/src/App.test.tsx
git commit -m "feat: add stage1 simulation snapshot save load support"
```

### Task 9: Performance Evidence, Docs, and Final Gate

**Files:**
- Add: `crates/petri-cli/src/bin/stage1_benchmark.rs`
- Modify: `README.md`
- Modify: `petri-roadmap.md` (checklist annotations only)
- Modify: `docs/plans/2026-02-11-stage1g-roadmap-closeout.md` (mark completed items)

**Step 1: Add benchmark harness**

- Implement deterministic benchmark run for 5,000 creatures on 200x200.
- Output measured ticks/sec and run metadata.

**Step 2: Add regression threshold check**

- Add test/assert mode requiring `>=30 ticks/sec` on local baseline profile.
- Document that CI may run with relaxed/noise-tolerant threshold if required.

**Step 3: Update docs**

- Update README commands and feature list for full Stage 1 scope.
- Mark Stage 1 done criteria status in roadmap notes.

**Step 4: Run completion gate**

Run and confirm all pass:
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run test`
5. `cd web && npm run test:e2e`
6. `cd web && npm run build`

**Step 5: Final commit**

```bash
git add crates/petri-cli/src/bin/stage1_benchmark.rs README.md petri-roadmap.md docs/plans/2026-02-11-stage1g-roadmap-closeout.md
git commit -m "docs: close stage1 roadmap and verification evidence"
```

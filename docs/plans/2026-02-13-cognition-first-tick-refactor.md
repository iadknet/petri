# Cognition-First Tick Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a cognition-first creature lifecycle where per-tick internal deliberation is energy-bounded, haltable, and followed by at most one world interaction (including explicit no-op).

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`, `GP-04`

**Scope:** Tick semantics, controller I/O, diagnostics/protocol wiring, config/runtime/startup draft migration to `energy_per_think_step`, snapshot persistence updates, and validation for cognition-first behavior. Out of scope: unrelated feature slices and CI/pipeline policy changes.

**Docs Impact:**
- Update `README.md`, `docs/reference/creature-controller-reference.md`, `docs/strategy/architecture.md`, and `docs/strategy/roadmap.md` once implementation lands.
- Update `docs/strategy/technology-review.md` if benchmark posture wording changes during stabilization.
- Keep root compatibility stubs (`petri-roadmap.md`, `petri-architecture.md`, `petri-technology-review.md`) short and canonical-link only.
- No additional compatibility stubs are introduced and no docs/plans are retired by this plan refinement.

**Supersedes:** `none`

**Superseded-By:** `none`

**Architecture:** Extend controller I/O in `petri-graph` for halt/no-op, confidence introspection, and explicit energy-awareness inputs (`energy_start_tick`, `energy_spent_tick`, `energy_remaining`), then refactor `petri-core` tick semantics from multi-action-per-tick to think-loop plus one-action arbitration with per-think-step memory read/write. Replace compute-node cognition charging with `energy_per_think_step`, migrate config/runtime/startup/protocol contracts accordingly, and preserve deterministic replay using per-creature seeded RNG tie-breaks.

**Tech Stack:** Rust (`petri-core`, `petri-graph`, `petri-server`, `petri-cli`), TypeScript (`web` protocol/tests), Cargo, npm.

## Goal Alignment

- `GP-01`: Prioritizes high-confidence testing of arbitration and think-loop behavior; deterministic tie-break semantics are used where they improve test reliability.
- `GP-02`: Keeps crate direction and ownership boundaries explicit while refactoring cross-crate interfaces.
- `GP-03`: Maintains reliable, regression-resistant iteration through explicit contract tests for cognition semantics.
- `GP-04`: Requires inspector/protocol diagnostics for think-loop and selected-action visibility.

## Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire-format changes are expected for controller I/O, config patch/startup draft fields, and creature-inspector diagnostics fields.
- Snapshot format changes are expected for new cognition diagnostics and are intentionally backward-incompatible for older snapshot assumptions.
- Test migration approach:
  - graph contract tests in `crates/petri-graph/tests/*`
  - world lifecycle/arbitration tests in `crates/petri-core/src/world/tests.rs`
  - snapshot serialization tests in `crates/petri-core/src/world/tests.rs`
  - transport/protocol tests in `crates/petri-server/tests/*` and `web/src/protocol.test.ts`

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/src/world/tick.rs` action resolution path | `change` | Must move from multi-action execution to one-action arbitration while preserving deterministic ordering and penalties. |
| `crates/petri-server` protocol/detail payload boundary | `change` | Inspector and payload contract must expose cognition diagnostics without leaking server transport logic into `petri-core`. |
| `crates/petri-graph` node surface (`types.rs`, evaluator) | `change` | New halt/no-op and introspection channels belong in graph representation/eval, not in server or web layers. |
| `crates/petri-core/src/world/snapshot.rs` serialization boundary | `change` | Snapshot must persist new cognition diagnostics used by inspector/replay, and this slice intentionally does not guarantee legacy compatibility. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| How are equal final action confidences resolved? | Use per-creature seeded RNG tie-break to preserve deterministic replay. | `petri-core` maintainers | `resolved` |
| Should throughput assertions block this refactor while semantics stabilize? | Keep benchmark informational during stabilization; re-tighten thresholds after profiling. | `petri-cli` maintainers | `resolved` |
| Where should cognition diagnostics be surfaced for debugging? | Surface in server creature-detail payload and web protocol/tests together. | `petri-server` + `web` maintainers | `resolved` |
| Must old snapshots remain compatible after cognition diagnostics are added? | No; this semantic refactor is allowed to break old snapshot assumptions, and docs must state the break explicitly. | Project maintainers | `resolved` |

## Locked Implementation Decisions

- Energy cost model: `energy_per_think_step` is the single cognition cost knob and replaces `energy_per_compute_node` behavior in tick cognition semantics.
- Config exposure surfaces: `energy_per_think_step` must be exposed in both runtime config patch and startup draft/startup patch.
- Contract migration: remove `energy_per_compute_node` from refactor-target runtime/startup/web protocol contracts in this slice.
- Think-loop memory rule: every think step performs memory address selection, memory read, action evaluation, and conditional memory write.
- Snapshot stance: persist new cognition diagnostics in snapshot and inspector structures; do not provide backward compatibility guarantees for legacy snapshots missing new fields.
- Tie-break behavior: exact final-confidence ties remain deterministic via per-creature seeded RNG.

## Assumptions and Defaults

- Snapshot backward compatibility is intentionally not guaranteed for this semantic refactor.
- Tie-break determinism remains per-creature seeded RNG.
- `energy_per_think_step` is the single cognition compute-cost knob post-refactor.
- Frontend completion includes unit tests and e2e, not just build.
- Canonical roadmap updates happen in `docs/strategy/roadmap.md`; root stubs remain minimal compatibility pointers.

## Test Scenarios

1. Plan-harness structural validation: `scripts/check-plan-harness.sh --mode strict`.
2. Docs/architecture harness verification according to date-gated policy in root `AGENTS.md`.
3. Runtime patch/startup draft contract coverage for payloads with `energy_per_think_step` and without `energy_per_compute_node`.
4. Snapshot round-trip coverage for new cognition diagnostics serialization/deserialization in the new format.
5. Frontend protocol typing and behavior coverage ensuring `npm run test`, `npm run test:e2e`, and `npm run build` pass with updated contracts.

### Task 1: Add graph contract tests for new cognition I/O (RED)

**Files:**
- Modify: `crates/petri-graph/tests/node_support.rs`
- Modify: `crates/petri-graph/tests/mutation_behavior.rs`

**Step 1: Add failing tests for new outputs and inputs**

- Add tests for `halt` and `no_op` outputs.
- Add tests for introspection input channels (previous-step and running-max confidence arrays).
- Add tests for new energy inputs (`energy_start_tick`, `energy_spent_tick`, `energy_remaining`) and their expected per-step update rules.

**Step 2: Add failing tests for clamping and movement-confidence derivation helper compatibility**

- Ensure movement confidence consumers can derive from `sqrt(move_x^2 + move_y^2)` with clamped axis outputs.

**Step 3: Run targeted tests to confirm RED**

Run:
- `cargo test -p petri-graph node_support -- --nocapture`
- `cargo test -p petri-graph mutation_behavior -- --nocapture`

Expected: FAIL on missing node kinds/fields/eval support.

### Task 2: Implement graph I/O extensions (GREEN)

**Files:**
- Modify: `crates/petri-graph/src/types.rs`
- Modify: `crates/petri-graph/src/eval/evaluate.rs`
- Modify: `crates/petri-graph/src/eval/mod.rs`
- Modify: `crates/petri-graph/src/eval/node_utils.rs`
- Modify: `crates/petri-graph/src/eval/presets.rs`

**Step 1: Extend types**

- Add `NodeKind` variants for new introspection inputs.
- Add `NodeKind` variants for `OutputHalt` and `OutputNoOp`.
- Add corresponding fields to `SensorInputs` and `ActionOutputs`, including the three energy-awareness inputs.

**Step 2: Extend evaluator behavior**

- Read introspection input values from `SensorInputs`.
- Populate `halt` and `no_op` output fields with clamping.

**Step 3: Update node classification and preset/founder graphs**

- Ensure new nodes are correctly treated as input/output, not hidden.
- Include new outputs in founder/preset output surfaces.

**Step 4: Re-run targeted graph tests to confirm GREEN**

Run:
- `cargo test -p petri-graph node_support -- --nocapture`
- `cargo test -p petri-graph mutation_behavior -- --nocapture`

Expected: PASS.

### Task 3: Add world-level cognition semantics tests (RED)

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Add failing tests for think-loop termination rules**

- `halt` terminates loop.
- Energy exhaustion terminates loop and can cause death.

**Step 2: Add failing tests for one-action arbitration**

- Exactly one world interaction max per tick.
- Explicit `no_op` can win and result in no world interaction.

**Step 3: Add failing tests for introspection channels**

- previous-step confidence inputs update correctly.
- running-max confidence inputs update correctly.
- energy-awareness inputs update correctly (`energy_start_tick` stable per tick; spent/remaining refreshed per think step).
- memory read/write state updates correctly on each think step.

**Step 4: Add failing tests for tie-break determinism**

- Equal final confidences resolved via per-creature seeded RNG deterministically across fixed seeds/snapshots.

**Step 5: Run targeted RED checks**

Run: `cargo test -p petri-core world::tests:: -- --nocapture`
Expected: FAIL for missing semantics.

### Task 4: Implement config/runtime fields for cognition cost

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/app_state/types.rs`
- Modify: `crates/petri-server/src/app_state/runtime_patch.rs`
- Modify: `crates/petri-server/src/app_state/startup_draft.rs`
- Modify: `crates/petri-server/tests/simulation_lifecycle.rs`

**Step 1: Add `energy_per_think_step` config field and defaults across core/runtime/startup surfaces**

- Add `energy_per_think_step` to world config, runtime patch, startup draft, and startup patch.
- Use strictly positive defaults and validation/clamp behavior consistent across runtime and startup entry points.

**Step 2: Migrate away from `energy_per_compute_node` in refactor-target contracts**

- Remove `energy_per_compute_node` from runtime patch/startup/web-facing refactor contracts in this slice.
- Ensure tests reflect `energy_per_think_step` presence and `energy_per_compute_node` removal.

**Step 3: Add/adjust tests for config and runtime patch behavior**

- Verify `energy_per_think_step` round-trip and clamp/validation semantics in both runtime patch and startup draft flows.
- Verify removed field behavior is reflected in server contract tests.

### Task 5: Refactor world tick lifecycle to cognition-first model

**Files:**
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/types.rs`

**Step 1: Introduce think-loop state per creature per tick**

- Maintain previous-step and running-max confidence trackers.
- Capture normalized tick-start energy once per tick for `energy_start_tick`.
- Recompute normalized `energy_spent_tick` and `energy_remaining` at the start of every think step.
- Feed these trackers back into controller inputs each step.

**Step 2: Apply per-think-step memory lifecycle**

- For each think step, perform memory address selection, memory read, action evaluation, and conditional memory write.
- Persist per-step memory-head outputs needed for diagnostics and final inspector state.

**Step 3: Implement halt/energy termination logic**

- Stop think loop on `halt` or insufficient energy for next think step.
- Charge cognition energy with `energy_per_think_step` each think step.

**Step 4: Implement final-thought arbitration**

- Compute candidate confidences from final outputs.
- Derive move confidence from vector magnitude.
- Allow `no_op` as explicit candidate.
- Break exact ties using per-creature RNG.

**Step 5: Enforce one-world-action max**

- Execute only selected action path.
- Preserve existing legality checks/penalties per selected action type.

**Step 6: Extend diagnostics and inspector state**

- Add think-step counters and arbitration outcome visibility.

### Task 6: Persist cognition diagnostics in snapshots

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world/snapshot.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Extend snapshot/detail structures**

- Add cognition diagnostic fields needed for think-loop and final-action visibility.
- Keep runtime and snapshot structs synchronized for these fields.

**Step 2: Persist and restore cognition diagnostics through snapshot flow**

- Serialize from live world state into snapshot.
- Deserialize from snapshot into live world state using the new schema.

**Step 3: Encode compatibility stance in tests**

- Add snapshot tests that validate new cognition fields round-trip.
- Treat missing new cognition fields in old snapshots as incompatible for this slice.

**Step 4: Run targeted snapshot tests**

Run: `cargo test -p petri-core world::tests:: -- --nocapture`
Expected: PASS for new snapshot round-trip expectations.

### Task 7: Update transport contracts and web protocol

**Files:**
- Modify: `crates/petri-server/tests/snapshot_and_creature.rs`
- Modify: `crates/petri-server/tests/simulation_lifecycle.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/protocol.test.ts`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/features/simulation/components/CreatureInspectorPanel.tsx` (if new fields displayed)

**Step 1: Add failing protocol/server tests for new fields and config contracts**

- Ensure `CreatureDetail` payload supports new cognition diagnostics/state.
- Ensure server runtime/startup payloads expose `energy_per_think_step` and no longer expose `energy_per_compute_node`.

**Step 2: Implement protocol type updates**

- Keep type names and field contracts synchronized across Rust/TS.
- Update TypeScript config patch/startup draft types to match server contract changes.

**Step 3: Re-run targeted tests**

Run:
- `cargo test -p petri-server snapshot_and_creature -- --nocapture`
- `cargo test -p petri-server simulation_lifecycle -- --nocapture`
- `cd web && npm test -- --run protocol`

Expected: PASS.

### Task 8: Regression and verification gates

**Files:**
- Verify-only across workspace

**Step 1: Run touched-crate test sweeps**

Run:
- `cargo test -p petri-graph`
- `cargo test -p petri-core`
- `cargo test -p petri-server`

**Step 2: Run full completion gate**

Run:
- `scripts/check-doc-harness.sh --mode warn` (through February 27, 2026)
- `scripts/check-architecture-harness.sh --mode warn` (through February 27, 2026)
- `scripts/check-doc-harness.sh --mode strict` (starting February 28, 2026)
- `scripts/check-architecture-harness.sh --mode strict` (starting February 28, 2026)
- `scripts/check-plan-harness.sh --mode strict`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run test`
- `cd web && npm run test:e2e`
- `cd web && npm run build`

Expected: all PASS.

**Step 3: Run informational benchmark**

Run: `cargo run -p petri-cli --bin stage1_benchmark -- --ticks 200`
Expected: benchmark prints values; no pass/fail threshold assertions required during stabilization.

### Task 9: Final docs sync after implementation

**Files:**
- Modify: `README.md`
- Modify: `docs/reference/creature-controller-reference.md`
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/strategy/roadmap.md`
- Modify: `docs/strategy/technology-review.md` (if benchmark posture wording changes)

**Step 1: Move planned semantics to current sections where implementation landed**

- Update wording from "planned" to "implemented" only for completed behaviors.

**Step 2: Verify no stale planned/current contradictions**

Run: `rg -n "planned|not implemented|current" README.md docs/reference/creature-controller-reference.md docs/strategy/architecture.md docs/strategy/roadmap.md docs/strategy/technology-review.md`
Expected: wording consistent with merged code state.

**Step 3: Verify root compatibility stubs remain minimal pointers**

Run: `rg -n "Canonical document|docs/strategy" petri-roadmap.md petri-architecture.md petri-technology-review.md`
Expected: stubs remain canonical-link only; no duplicated roadmap content.

## Risks and Rollback

- Throughput risk: cognition-first looping may reduce ticks-per-second temporarily while semantics stabilize.

Rollback approach:

- There are no plans to roll back this change.
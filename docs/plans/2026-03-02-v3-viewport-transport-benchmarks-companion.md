# V3 Viewport Transport Benchmarks and Core-Perf Companion

**Parent plan:** `docs/plans/2026-03-02-v3-viewport-transport-plan.md`

**Goal:** Establish benchmark gates for every refactor phase and land the core runtime optimizations after transport waste is removed.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Included: benchmark harnesses, profiling checkpoints, server projection/view/protocol measurement, frontend render benchmarks, and the core metadata-cache and allocation/perception optimization phases. Excluded: protocol design itself and non-performance feature work.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` | Shared command source of truth |
| `docs/reference/v3-evolution-observability-spec.md` | Clarify wall-clock perf metric semantics |

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `docs/plans/2026-03-02-v3-viewport-transport-plan.md`
- `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md`

## Goal Alignment

- **GP-01:** Measure and improve the costs that actually dominate large-world interaction.
- **GP-02:** Keep transport and runtime optimization work separated so improvements are attributable.
- **GP-03:** No phase closes without a before/after benchmark record.
- **GP-04:** Perf telemetry remains behavior-backed and clearly separated from energy-cost metrics.

## Boundary Impact

- Benchmark fixtures live outside production transport code paths wherever possible.
- Core runtime optimization remains inside `v3-core`.
- Server projection/view/protocol measurement remains inside `v3-server`.
- Frontend render measurement remains inside `frontend`.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/benches/tick.rs` | change | Existing benches are not enough to isolate projection and viewport transport improvements. |
| `v3/crates/v3-server` benches | change | No server benchmark suite exists yet for projection/view/protocol work. |
| frontend render path | change | Current tests cover correctness, not the new viewport/render workload. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should benchmark artifacts be committed? | No. Store them under `/tmp/v3-perf/<phase>/`. | user+agent | resolved |
| When should core micro-optimizations start? | After transport/projection refactor phases land and are re-profiled. | user+agent | resolved |

## Execution Requirements

- Any Rust benchmark or runtime optimization work must explicitly load and apply `rust-skills` before implementation.
- Any frontend benchmark/render work must explicitly load and apply `vercel-react-best-practices`; use `frontend-design` only when UI behavior or presentation changes.
- Every behavior change still follows TDD before the benchmark-driven implementation step.
- After every task in this companion, run the recursive review gate that matches the code touched by that task: `rust-skills` for Rust work, `vercel-react-best-practices` plus `vercel-composition-patterns` for frontend work, and `frontend-design` when UI behavior or presentation changes.
- If a task touches both Rust and frontend code, both recursive review gates must pass.
- Do not close a benchmarking or optimization task while any required recursive review still reports new findings.

## Benchmark Families

- `core_fast`
- `core_stress`
- `server_projection`
- `server_view_assembly`
- `server_protocol`
- `frontend_render`
- `stack_profile`

## Phase Gates

For every implementation phase:
1. Capture primary benchmark before the code change.
2. Capture at least one guardrail benchmark before the code change.
3. Implement the change.
4. Rerun the same benchmarks.
5. Stop if the primary benchmark fails to improve or the guardrail regresses by more than `5%`.

## Task List

### Task 1: Add benchmark fixtures and harnesses

**Files:**
- Create: `v3/crates/v3-perf-support/Cargo.toml`
- Create: `v3/crates/v3-perf-support/src/lib.rs`
- Modify: `v3/Cargo.toml`
- Modify: `v3/crates/v3-core/Cargo.toml`
- Modify: `v3/crates/v3-core/benches/tick.rs`
- Modify: `v3/crates/v3-server/Cargo.toml`
- Create: `v3/crates/v3-server/benches/transport.rs`
- Create or modify frontend render benchmark harnesses as needed

### Task 2: Measure projection and transport phases

**Files:**
- Modify: `v3/crates/v3-server/benches/transport.rs`
- Modify: `v3/crates/v3-server/src/query/*`
- Modify: `v3/crates/v3-server/src/transport/*`

### Task 3: Land core metadata-cache optimizations

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/mod.rs`
- Modify: `v3/crates/v3-core/src/creature/state.rs`
- Modify: `v3/crates/v3-core/src/creature/founder.rs`
- Modify: `v3/crates/v3-core/src/simulation/seeding.rs`
- Modify: `v3/crates/v3-core/src/mutation/engine/mod.rs`
- Modify: `v3/crates/v3-core/src/runtime/vm.rs`
- Modify: `v3/crates/v3-core/src/runtime/graph.rs`
- Modify: `v3/crates/v3-core/src/runtime/mesh.rs`
- Modify: `v3/crates/v3-core/src/runtime/traced_mesh.rs`
- Modify: `v3/crates/v3-core/src/sensors/perception.rs`

### Task 4: Land allocation/perception optimizations

**Files:**
- Modify: `v3/crates/v3-core/Cargo.toml`
- Modify: `v3/crates/v3-core/src/contracts/action_queue.rs`
- Modify: `v3/crates/v3-core/src/runtime/types.rs`
- Modify: `v3/crates/v3-core/src/sensors/visibility.rs`
- Modify: `v3/crates/v3-core/src/sensors/reducers.rs`
- Modify: `v3/crates/v3-core/src/simulation/tick.rs`

### Task 5: Conditional final runtime cleanup

**Files:**
- Modify: `v3/crates/v3-core/src/simulation/simulation.rs`
- Modify: `v3/crates/v3-core/src/simulation/tick.rs`

## Acceptance Criteria

- Every phase has recorded before/after artifacts.
- Server projection and view assembly can be benchmarked independently of the frontend.
- Core hot-path improvements are attributed after transport waste is removed.
- Final profile data confirms whether the conditional cleanup phase is still needed.
- Every benchmarking or optimization task has a recorded recursive review pass with no new findings before task completion.

## Verification Commands

- Use `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` as the command source of truth.

## Risks and Rollback

- Benchmark drift can hide regressions if fixtures are not deterministic.
- Early micro-optimizations can mask transport wins; keep runtime optimization phases after transport cutover.
- If a targeted optimization regresses behavior, keep the benchmark harness and revert only the failing optimization slice.

**Review cycles:** 1

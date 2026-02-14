# Petri V2 Stage 4: Product Surface and Stabilization

> **Stage Type:** This stage is part of a **MAJOR GREENFIELD REWRITE** program.

**Goal:** Deliver stable `v2-server`, `v2-cli`, and `v2-web` surfaces over the new `v2-core` runtime and promote `v2` docs as the active architecture target.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2` API/protocol/UI/CLI integration, docs rebaseline toward `v2`; excludes changes to legacy runtime behavior.
**Docs Impact:** Update canonical docs to describe `v2` target architecture and keep legacy root documented as historical baseline; consume CP-3 API/protocol spec and shared test matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: exposes new cognition runtime via usable app surfaces.
- `GP-02`: keeps clean boundaries between `v2-core`, `v2-server`, `v2-cli`, and `v2-web`.
- `GP-03`: integration and protocol tests gate release readiness.
- `GP-04`: keeps runtime behavior inspectable through CLI/server/web outputs.

## Boundary Impact

- `v2-server` is the only network boundary for `v2-web`.
- `v2-cli` depends on `v2-core` only.
- Legacy server/web contracts remain unchanged; no cross-wire integration with `v2`.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-server/*` | change | New protocol endpoints and contract serialization live here. |
| `v2/crates/v2-cli/*` | change | New run/ablation workflows and reporting surfaces live here. |
| `v2/web/*` | change | New UI protocol models and visualization paths live here. |
| legacy `crates/petri-server/*` and `web/*` | keep | Legacy app remains isolated from `v2` release path. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should snapshot import/export endpoints be required in initial `v2`? | No; snapshotting is out of scope for initial release. | user+agent | resolved |
| Should `v2` and legacy share API schema files? | No; `v2` owns independent contracts. | user+agent | resolved |
| Should docs present legacy or `v2` as active architecture target? | `v2` once stage 4 gates pass. | user+agent | resolved |

## Specification Dependencies

- API/protocol contract source of truth:
  - `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- Gate consistency source of truth:
  - `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

### Task 1: Add failing `v2` integration and protocol tests

Files:
- Create: `v2/crates/v2-server/tests/lifecycle.rs`
- Create: `v2/crates/v2-server/tests/payloads.rs`
- Create: `v2/crates/v2-server/tests/ws_stream.rs`
- Create: `v2/crates/v2-cli/tests/ablation_output.rs`
- Create: `v2/crates/v2-cli/tests/run_output.rs`
- Create: `v2/web/src/protocol.test.ts`

Steps:
1. Add failing tests for server lifecycle endpoints and status payloads.
2. Add failing tests for core frame/detail contract serialization.
3. Add failing tests for CLI run/ablation output format.
4. Add failing tests for web protocol parsing/render assumptions.

### Task 2: Implement `v2-server` API and runtime loop wiring

Files:
- Create: `v2/crates/v2-server/src/main.rs`
- Create: `v2/crates/v2-server/src/api.rs`
- Create: `v2/crates/v2-server/src/state.rs`
- Create: `v2/crates/v2-server/src/ws.rs`

Steps:
1. Implement startup/start/pause/step/status endpoints.
2. Wire runtime tick loop to `v2-core`.
3. Ensure endpoint payloads match protocol tests.

### Task 3: Implement `v2-cli` run and ablation flows

Files:
- Create: `v2/crates/v2-cli/src/main.rs`
- Create: `v2/crates/v2-cli/src/ablation.rs`
- Create: `v2/crates/v2-cli/src/output.rs`

Steps:
1. Implement run loop and periodic stats reporting.
2. Implement ablation command over controller palettes/config presets.
3. Keep CLI outputs deterministic in shape for downstream analysis.

### Task 4: Implement `v2-web` protocol/client baseline and rebaseline docs

Files:
- Create: `v2/web/src/protocol.ts`
- Create: `v2/web/src/App.tsx`
- Modify: `README.md`
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/strategy/roadmap.md`

Steps:
1. Implement minimal client protocol layer and runtime status/frame UI.
2. Wire web client to `v2-server` endpoints.
3. Update canonical docs to mark `v2` as target architecture path.

## Checkpoint Boundaries

### Entry Checkpoint (`S4-ENTRY`)

Required before starting:
1. Stage-3 `S3-EXIT` is `go`.
2. `v2-core` runtime contracts are frozen for integration window.

### Midpoint Checkpoint (`S4-MID`)

Required before task 4:
1. Server and CLI integration tests pass.
2. Web protocol tests pass against fixture payloads.
3. Protocol version is stable across server/cli/web fixtures.

Stop conditions:
1. Repeated contract drift between server and web.
2. Cross-root legacy coupling introduced.

### Exit Checkpoint (`S4-EXIT`)

Required to close stage:
1. `v2` integration tests are green.
2. `v2` web build passes.
3. Canonical docs are updated for `v2` target state.

Go / stop rule:
1. `go` (program complete) only when stage verification gates pass in one full run.
2. `stop` and open follow-up stage if protocol or stability gaps remain.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo fmt --all --check`
3. `cd v2 && cargo clippy --workspace --all-targets -- -D warnings`
4. `cd v2 && cargo test --workspace`
5. `cd v2/web && npm run test`
6. `cd v2/web && npm run build`

## Risks and Rollback

- Risk: early API shape churn can destabilize web/client wiring.
- Risk: docs can drift if canonical updates lag implementation.
- Rollback: revert stage-4 `v2` integration commits while preserving stage-1..3 runtime foundation.

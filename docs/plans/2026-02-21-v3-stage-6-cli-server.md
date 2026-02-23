# Stage 6: CLI + Server — Implementation Plan

**Goal:** Expose the v3-core simulation engine through a `v3-cli` NDJSON local runner and a
fully-implemented `v3-server` HTTP/WebSocket service, with observability stats wired end-to-end.

**Goal IDs:** GP-02, GP-03, GP-04

**Scope:** New `v3/crates/v3-cli` binary crate; flesh out `v3/crates/v3-server`; add `SimStats`
observability to `v3/crates/v3-core`. Excludes snapshot import/export (out of scope per spec
Section 7) and Stage 7 hardening.

**Docs Impact:**
- `docs/plans/2026-02-21-v3-stage-6-cli-server.md` — created on completion
- archived reconciliation evidence file removed
- `docs/plans/2026-02-21-v3-implementation-master-plan.md` — Stage 6 row `[x]` on completion

**Supersedes:** none
**Superseded-By:** none
**Parent plan:** `docs/plans/2026-02-21-v3-implementation-master-plan.md`

---

## Context

Stages 1-5 built a fully-runnable simulation with real mutation and phenotype evolution.
Stage 6 exposes this engine through two transport surfaces: a `v3-cli` binary that runs the
simulation locally and emits NDJSON events, and a `v3-server` HTTP/WebSocket server that provides
state-machine-controlled simulation with live frame streaming. Both surfaces require observability
counters (mutation/reproduction stats) that don't yet exist in `v3-core`.

---

## Goal Alignment

- **GP-02:** `v3-cli` depends on `v3-core` only (no server dependency). `v3-server` depends on
  `v3-core` only. Transport concerns stay entirely outside `v3-core`.
- **GP-03:** Every new public behavior (CLI event ordering, HTTP state transitions, WebSocket
  envelope, stats counting semantics) gets unit or integration test coverage.
- **GP-04:** `SimStats` in `v3-core` provides a direct bridge between applied simulation behavior
  and the observable API surfaces. The Runtime Truthfulness Invariant is enforced; no synthetic values.

---

## Boundary Impact

| area | decision | rationale |
| --- | --- | --- |
| `v3-core/src/simulation/stats.rs` | NEW | `SimStats` struct, plain data, no new external deps |
| `v3-core/src/simulation/simulation.rs` | change | gains `pub stats: SimStats`, `mean_energy()` |
| `v3-core/src/simulation/actions.rs` | change | `apply_reproduce` updates `sim.stats` cumulative counts |
| `v3-core/src/simulation/tick.rs` | change | resets `last_tick_*` per tick; increments per action branch |
| `v3-core/src/simulation/seeding.rs` | change | add `stats: SimStats::default()` to returned `Simulation` |
| `v3-core/src/simulation/mod.rs` | change | export `SimStats` |
| `v3-core/src/config/simulation.rs` | change | add `#[serde(deny_unknown_fields)]` to all config structs |
| `v3/crates/v3-cli/` | NEW binary crate | depends on `v3-core`, `clap 4`, `serde_json` |
| `v3/crates/v3-server/` | fleshed out | depends on `v3-core`, `axum 0.7`, `tokio 1`, `sha2` |
| `v3/Cargo.toml` | change | add `"crates/v3-cli"` to workspace members; add clap/axum/tokio/sha2 workspace deps |

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `creature/`, `kernel/`, `sensors/`, `runtime/`, `mutation/` | keep | no changes needed for Stage 6 |
| `simulation/actions.rs` | change | stats accumulation; no signature change |
| `simulation/tick.rs` | change | counter reset/increment only; no logic changes |
| `v3-server/src/lib.rs` | change | full replacement of stub comment |
| `contracts/ids.rs` | keep | use `id.data().as_ffi()` in server frame handler for numeric creature ID |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| `SimStats` derives | `Debug`, `Clone`, `Default` — zero-initializes all counters | agent | resolved |
| `mean_energy` on empty population | returns `0.0` to avoid divide-by-zero | agent | resolved |
| `last_tick_reproduce` increment location | top of `apply_reproduce`, before any early return (spec: "regardless of outcome") | agent | resolved |
| Double-increment risk for `last_tick_reproduce` | `tick.rs` must NOT increment in the `Reproduce` arm (already done in `apply_reproduce`) | agent | resolved |
| Per-reason rejection breakdown | add `pub reproduction_actions_rejected_by_reason: HashMap<String, u64>` to `SimStats`; populated in `apply_reproduce` at each rejection point | agent | resolved |
| `deny_unknown_fields` for server/CLI config validation | add `#[serde(deny_unknown_fields)]` to all `SimulationConfig` sub-structs in `v3-core`; enables spec-compliant rejection of unknown fields in both server startup and CLI config files | agent | resolved |
| `MutationConfig.domain_selection_weights` / `operator_selection_weights` (in spec request body) | these keys do NOT exist in `SimulationConfig`; submitting them returns 422 per `deny_unknown_fields`; out of scope for Stage 6 | agent | resolved |
| `POST /startup` while running | spec allows from any state; implementation sets `status = Idle` before re-seeding; running loop exits on next check | agent | resolved |
| Server binary entrypoint | add `src/bin/server.rs` to `v3-server` for standalone server launch | agent | resolved |
| Run loop double-spawn prevention | check `status == Running` under mutex before spawning; idempotent return if already running | agent | resolved |

---

## Context and File Layout

- Working directory: `v3/` (all cargo commands run from here)
- Key specs: `docs/reference/v3-cli-contract-spec.md`, `docs/reference/v3-server-api-protocol-spec.md`,
  `docs/reference/v3-evolution-observability-spec.md`

Expected new/modified files after this stage:

```
v3/Cargo.toml                                          <- add v3-cli to members; add clap/axum/tokio/sha2 workspace deps
v3/crates/v3-core/src/
  config/simulation.rs                                 <- add #[serde(deny_unknown_fields)] to all config structs
  simulation/
    mod.rs                                             <- export stats::SimStats
    simulation.rs                                      <- add pub stats: SimStats; add mean_energy()
    stats.rs                                           <- NEW: SimStats struct
    actions.rs                                         <- apply_reproduce: stats accumulation
    seeding.rs                                         <- add stats: SimStats::default()
    tick.rs                                            <- reset last_tick_*; increment per branch
v3/crates/v3-cli/
  Cargo.toml                                           <- NEW
  src/
    lib.rs                                             <- NEW: run_simulation(), event types, RunError
    main.rs                                            <- NEW: clap CLI binary
  tests/
    cli.rs                                             <- NEW: integration tests
v3/crates/v3-server/
  Cargo.toml                                           <- add axum/tokio/sha2/tower deps
  src/
    lib.rs                                             <- replace stub: router() function + pub mods
    state.rs                                           <- NEW: AppState, SimHandle, SimulationStatus, WsEvent
    error.rs                                           <- NEW: AppError enum, FieldError, IntoResponse impl
    types.rs                                           <- NEW: StepRequest, deep_merge, config_digest, sort_json_keys
    ws.rs                                              <- NEW: ws_handler, handle_socket
    handlers/
      mod.rs                                           <- NEW
      lifecycle.rs                                     <- NEW: startup, start, pause_sim, step, run_loop
      status.rs                                        <- NEW: get_status, get_frame, get_config, patch_config
    bin/
      server.rs                                        <- NEW: tokio main, bind and serve
  tests/
    server.rs                                          <- NEW: integration tests (tower::ServiceExt::oneshot)
docs/plans/2026-02-21-v3-stage-6-cli-server.md         <- NEW (written at Task 8)
archived reconciliation evidence file removed
```

---

## Tasks

- [x] **Task 0: Write subplan document to `docs/plans/`**

  Write this plan's content to `docs/plans/2026-02-21-v3-stage-6-cli-server.md` before any code
  changes. The file should contain the full plan with all tasks marked `[ ]` (pending).
  This ensures the plan is in the repo and visible before implementation begins.

  **Acceptance:** file exists at correct path; `scripts/check-plan-harness.sh --mode strict` passes

- [x] **Task 1: Add `SimStats` to v3-core — struct + Simulation integration**

  **Files:** `simulation/stats.rs` (NEW), `simulation/simulation.rs`, `simulation/seeding.rs`,
  `simulation/mod.rs`

- [x] **Task 2: Wire stats accumulation in `apply_reproduce` and `run_tick`**

  **Files:** `simulation/actions.rs`, `simulation/tick.rs`

- [x] **Task 3: Add `#[serde(deny_unknown_fields)]` to all `SimulationConfig` structs**

  **File:** `v3/crates/v3-core/src/config/simulation.rs`

- [x] **Task 4: Add `v3-cli` binary crate**

  **Files:** `v3/Cargo.toml`, `v3/crates/v3-cli/Cargo.toml`, `v3/crates/v3-cli/src/lib.rs`,
  `v3/crates/v3-cli/src/main.rs`, `v3/crates/v3-cli/tests/cli.rs`

- [x] **Task 5: Implement `v3-server` — state, error, types, config helpers**

  **Files:** `v3-server/Cargo.toml`, `src/state.rs` (NEW), `src/error.rs` (NEW), `src/types.rs` (NEW)

- [x] **Task 6: Implement `v3-server` HTTP handlers and run loop**

  **Files:** `src/lib.rs`, `src/handlers/mod.rs`, `src/handlers/lifecycle.rs`,
  `src/handlers/status.rs`, `src/bin/server.rs` (NEW)

- [x] **Task 7: Implement `v3-server` WebSocket handler + integration tests**

  **Files:** `src/ws.rs`, `tests/server.rs`

- [x] **Task 8: Quality gates, evidence matrix, master plan update**

---

## Verification Commands

```bash
# Run from v3/ directory:
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Run from repo root:
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode warn
```

---

## Key Reference Files

- `v3/crates/v3-core/src/simulation/simulation.rs` — add `pub stats: SimStats`, `mean_energy()`
- `v3/crates/v3-core/src/simulation/actions.rs` — `apply_reproduce` stats wiring (precise order matters)
- `v3/crates/v3-core/src/simulation/tick.rs` — per-tick reset + per-branch increments
- `v3/crates/v3-core/src/config/simulation.rs` — add `deny_unknown_fields` to all config structs
- `v3/crates/v3-server/src/lib.rs` — router entry point; replaces stub
- `docs/reference/v3-cli-contract-spec.md` — authoritative CLI spec
- `docs/reference/v3-server-api-protocol-spec.md` — authoritative server spec
- `docs/reference/v3-evolution-observability-spec.md` — authoritative observability spec

---

**Review cycles:** 1

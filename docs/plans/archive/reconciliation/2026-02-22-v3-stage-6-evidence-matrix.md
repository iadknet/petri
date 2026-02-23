# V3 Stage 6 Evidence Matrix

**Stage:** 6 — CLI + Server
**Date verified:** 2026-02-22
**Subplan:** `docs/plans/2026-02-21-v3-stage-6-cli-server.md`
**Test count:** 286 (245 v3-core unit + 17 v3-core integration + 6 v3-cli + 5 v3-server unit + 13 v3-server integration)
**Verification gate:** cargo test + clippy + fmt + harness scripts all pass

---

## Task 0: Write subplan document

| Item | Code Ref | Test |
| --- | --- | --- |
| Subplan written before code changes | `docs/plans/2026-02-21-v3-stage-6-cli-server.md` | `scripts/check-plan-harness.sh --mode strict` → violations=0 |

## Task 1: Add `SimStats` to v3-core

| Item | Code Ref | Test |
| --- | --- | --- |
| `SimStats` struct with all 11 fields | `v3/crates/v3-core/src/simulation/stats.rs:6` | (compile-time + downstream accumulation tests) |
| `Simulation.stats: SimStats` field | `v3/crates/v3-core/src/simulation/simulation.rs:20` | (compile-time) |
| `mean_energy()` on non-empty population | `v3/crates/v3-core/src/simulation/simulation.rs:59` | `simulation::simulation::tests::mean_energy_returns_correct_mean_for_multiple_creatures` |
| `mean_energy()` returns 0.0 for empty | `v3/crates/v3-core/src/simulation/simulation.rs:59` | `simulation::simulation::tests::mean_energy_returns_zero_for_empty_simulation` |
| `SimStats` exported from `mod.rs` | `v3/crates/v3-core/src/simulation/mod.rs` | (compile-time) |
| `stats: SimStats::default()` in seeding | `v3/crates/v3-core/src/simulation/seeding.rs` | `tests/viability.rs::mean_energy_is_positive_in_viable_sim` |

## Task 2: Wire stats accumulation in `apply_reproduce` and `run_tick`

| Item | Code Ref | Test |
| --- | --- | --- |
| `attempted_total` incremented at top of `apply_reproduce` | `v3/crates/v3-core/src/simulation/actions.rs:69` | `tests/viability.rs::stats_accounting_invariant_holds_over_50_ticks` |
| `last_tick_reproduce` incremented at top (before any return) | `v3/crates/v3-core/src/simulation/actions.rs:70` | `tests/viability.rs::stats_last_tick_counters_reset_each_tick` |
| `rejected_total` + `rejected_by_reason` at each rejection site | `v3/crates/v3-core/src/simulation/actions.rs` | `tests/viability.rs::stats_accounting_invariant_holds_over_50_ticks` |
| Mutation stats accumulated from `MutationSummary` | `v3/crates/v3-core/src/simulation/actions.rs` | `tests/viability.rs::stats_accounting_invariant_holds_over_50_ticks` |
| `spawned_total` incremented before `Spawned` return | `v3/crates/v3-core/src/simulation/actions.rs:187` | `tests/viability.rs::stats_accounting_invariant_holds_over_50_ticks` |
| Per-tick counters reset at start of `run_tick` | `v3/crates/v3-core/src/simulation/tick.rs:55` | `tests/viability.rs::stats_last_tick_counters_reset_each_tick` |
| `last_tick_noop/eat/move` incremented per action branch | `v3/crates/v3-core/src/simulation/tick.rs:101–113` | (integration viability runs) |
| Accounting invariant: `attempted == spawned + rejected` | `v3/crates/v3-core/src/simulation/actions.rs` | `tests/viability.rs::stats_accounting_invariant_holds_over_50_ticks` |
| `mean_energy` positive in viable sim | `v3/crates/v3-core/src/simulation/simulation.rs:59` | `tests/viability.rs::mean_energy_is_positive_in_viable_sim` |

## Task 3: Add `#[serde(deny_unknown_fields)]` to all `SimulationConfig` structs

| Item | Code Ref | Test |
| --- | --- | --- |
| `deny_unknown_fields` on `SimulationConfig` | `v3/crates/v3-core/src/config/simulation.rs:215` | `config::simulation::tests::deny_unknown_fields_rejects_extra_key` |
| `deny_unknown_fields` on all 11 sub-structs | `v3/crates/v3-core/src/config/simulation.rs:4–195` | `config::simulation::tests::deny_unknown_fields_rejects_extra_key` |
| Unknown config keys rejected at deserialization | `v3/crates/v3-core/src/config/simulation.rs:477` | `config::simulation::tests::deny_unknown_fields_rejects_extra_key` |

## Task 4: Add `v3-cli` binary crate

| Item | Code Ref | Test |
| --- | --- | --- |
| `RunStartedEvent` struct | `v3/crates/v3-cli/src/lib.rs:26` | `cli::tests::run_started_is_first_event` |
| `TickSampleEvent` struct (all stats fields) | `v3/crates/v3-cli/src/lib.rs:35` | `cli::tests::stats_accounting_invariant_in_cli_output` |
| `RunCompletedEvent` struct | `v3/crates/v3-cli/src/lib.rs:50` | `cli::tests::run_completed_is_last_event` |
| `RunError` enum (ValidationError / RuntimeError) | `v3/crates/v3-cli/src/lib.rs:9` | `cli::tests::unknown_config_field_returns_validation_error` |
| `run_simulation()` — run_started first event | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::run_started_is_first_event` |
| `run_simulation()` — tick_sample alignment | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::tick_sample_alignment_sample_every_3` |
| `run_simulation()` — final-tick emission rule | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::tick_sample_alignment_sample_every_3` |
| `run_simulation()` — run_completed last event | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::run_completed_is_last_event` |
| Stats accounting invariant in CLI output | `v3/crates/v3-cli/src/lib.rs:117` | `cli::tests::stats_accounting_invariant_in_cli_output` |
| Unknown config field returns `ValidationError` | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::unknown_config_field_returns_validation_error` |
| `sample_every=1` emits one sample per tick | `v3/crates/v3-cli/src/lib.rs:66` | `cli::tests::sample_every_1_emits_one_sample_per_tick` |

## Task 5: Implement `v3-server` — state, error, types

| Item | Code Ref | Test |
| --- | --- | --- |
| `SimulationStatus` enum (Idle/Running/Paused) | `v3/crates/v3-server/src/state.rs:9` | `server::tests::start_transitions_to_running` |
| `SimHandle` struct | `v3/crates/v3-server/src/state.rs:15` | (compile-time) |
| `WsEvent` struct (3 payloads + tick) | `v3/crates/v3-server/src/state.rs:32` | (compile-time) |
| `AppState` (Arc<Mutex<SimHandle>> + broadcast) | `v3/crates/v3-server/src/state.rs:40` | (compile-time) |
| `AppError::InvalidRequest` → 400 | `v3/crates/v3-server/src/error.rs:14` | `server::tests::startup_missing_seed_returns_400` |
| `AppError::InvalidStateTransition` → 409 | `v3/crates/v3-server/src/error.rs:15` | `server::tests::pause_on_idle_returns_409` |
| `AppError::ValidationRejected` → 422 | `v3/crates/v3-server/src/error.rs:19` | `server::tests::startup_unknown_field_returns_422` |
| Error envelope includes `protocol_version` | `v3/crates/v3-server/src/error.rs` | `server::tests::error_envelope_has_protocol_version` |
| `deep_merge()` recursive merge | `v3/crates/v3-server/src/types.rs` | `types::tests::deep_merge_preserves_unpatched_keys` |
| `sort_json_keys_recursive()` alphabetical | `v3/crates/v3-server/src/types.rs` | `types::tests::sort_json_keys_recursive_sorts_all_levels` |
| `config_digest()` SHA-256 of sorted JSON | `v3/crates/v3-server/src/types.rs` | `types::tests::config_digest_is_deterministic` |
| `config_digest()` changes on config change | `v3/crates/v3-server/src/types.rs` | `types::tests::config_digest_changes_on_config_change` |
| `StepRequest` with default `steps=1` | `v3/crates/v3-server/src/types.rs` | (compile-time) |

## Task 6: Implement `v3-server` HTTP handlers and run loop

| Item | Code Ref | Test |
| --- | --- | --- |
| Router with 9 routes | `v3/crates/v3-server/src/lib.rs:11` | `server::tests::startup_returns_200_with_required_fields` |
| `startup` — extract seed (400 if missing) | `v3/crates/v3-server/src/handlers/lifecycle.rs:12` | `server::tests::startup_missing_seed_returns_400` |
| `startup` — deep_merge config (422 on unknown) | `v3/crates/v3-server/src/handlers/lifecycle.rs:12` | `server::tests::startup_unknown_field_returns_422` |
| `startup` — returns `config_digest`, `seeded_creatures` | `v3/crates/v3-server/src/handlers/lifecycle.rs:12` | `server::tests::startup_returns_200_with_required_fields` |
| `startup` — `config_digest` starts with `sha256:` | `v3/crates/v3-server/src/handlers/lifecycle.rs:12` | `server::tests::config_digest_present_in_startup_response` |
| `start` — idempotent when Running | `v3/crates/v3-server/src/handlers/lifecycle.rs:68` | `server::tests::start_transitions_to_running` |
| `start` — spawns `run_loop` | `v3/crates/v3-server/src/handlers/lifecycle.rs:68` | `server::tests::start_transitions_to_running` |
| `pause_sim` — 409 if Idle | `v3/crates/v3-server/src/handlers/lifecycle.rs:94` | `server::tests::pause_on_idle_returns_409` |
| `step` — 409 if not Paused | `v3/crates/v3-server/src/handlers/lifecycle.rs:111` | `server::tests::step_on_non_paused_returns_409` |
| `step` — 422 if steps=0 | `v3/crates/v3-server/src/handlers/lifecycle.rs:111` | `server::tests::step_with_zero_steps_returns_422` |
| `step` — 422 if steps>1000 | `v3/crates/v3-server/src/handlers/lifecycle.rs:111` | `server::tests::step_with_too_many_steps_returns_422` |
| `get_status` — all required fields | `v3/crates/v3-server/src/handlers/status.rs:11` | `server::tests::get_status_has_all_required_fields` |
| `get_frame` — creatures/food/barriers arrays | `v3/crates/v3-server/src/handlers/status.rs:37` | `server::tests::get_frame_returns_creature_food_barrier_arrays` |
| `patch_config` — 409 on topology while Running | `v3/crates/v3-server/src/handlers/status.rs:90` | `server::tests::patch_config_world_field_while_running_returns_409` |
| Server binary entrypoint at `:3000` | `v3/crates/v3-server/src/bin/server.rs` | (compile-time) |

## Task 7: Implement WebSocket handler + integration tests

| Item | Code Ref | Test |
| --- | --- | --- |
| `ws_handler` — upgrades HTTP to WS | `v3/crates/v3-server/src/ws.rs:8` | (compile-time) |
| `handle_socket` — sends 3 events per tick | `v3/crates/v3-server/src/ws.rs:15` | (compile-time; event ordering via broadcast) |
| WS envelope includes `protocol_version`, `event`, `tick`, `payload` | `v3/crates/v3-server/src/ws.rs:15` | (compile-time) |
| `Lagged` error continues without crash | `v3/crates/v3-server/src/ws.rs:15` | (compile-time) |
| 13 server integration tests pass | `v3/crates/v3-server/tests/server.rs` | all 13 tests pass |

## Task 8: Quality gates

| Gate | Result |
| --- | --- |
| `cargo test --workspace` | 286 tests pass; 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (0 warnings) |
| `cargo fmt --all -- --check` | clean |
| `scripts/check-plan-harness.sh --mode strict` | violations=0, warnings=0, files_checked=14 |
| `scripts/check-doc-harness.sh --mode warn` | violations=2 (pre-existing CLAUDE.md warnings, not Stage 6) |
| `scripts/check-architecture-harness.sh --mode warn` | violations=0 (5 pre-existing petri-core size warnings) |

### Pre-existing test bug fixes (root-cause investigation)

Four pre-existing v3-core test failures were investigated and fixed in this stage:

| Test | Root cause | Fix |
| --- | --- | --- |
| `normalize_nan_vm_opcode_cost_multiplier_falls_back` | `normalize()` called fallback `0.25` but default and test expected `0.5` | Changed normalize fallback to `0.5` |
| `energy_exhaustion_returns_exhausted` (graph) | Test assumed `graph_node_base_cost=1.0` but `default_config()` returns `0.05` | Added explicit `config.graph_node_base_cost = 1.0` |
| `state_not_mutated_on_energy_exhaustion` (graph) | Same as above | Same fix |
| `energy_exhaustion_does_not_commit_memory_writes` (vm) | Test assumed `opcode_cost_multiplier=1.0` but `RuntimeConfig::default()` returns `0.5` | Added explicit `cfg.vm.opcode_cost_multiplier = 1.0` |

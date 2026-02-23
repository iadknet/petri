# V3 Stage 7 Evidence Matrix

**Stage:** 7 — Observability + Hardening
**Date verified:** 2026-02-23
**Subplan:** `docs/plans/2026-02-23-v3-stage-7-observability-hardening.md`
**Test count:** 39 (18 v3-core integration + 7 v3-cli integration + 14 v3-server integration)
**Verification gate:** cargo test + clippy + fmt + harness scripts all pass

---

## Task 0: Write subplan document

| Item | Code Ref | Test |
| --- | --- | --- |
| Subplan written before code changes | `docs/plans/2026-02-23-v3-stage-7-observability-hardening.md` | `scripts/check-plan-harness.sh --mode strict` → violations=0 |

## Task 1: Add `mutation_events_skipped_by_reason` to `SimStats` + wire in `actions.rs`

| Item | Code Ref | Test |
| --- | --- | --- |
| `mutation_events_skipped_by_reason: HashMap<String, u64>` field added to `SimStats` | `v3/crates/v3-core/src/simulation/stats.rs:15` | (compile-time; `Default` auto-inits to empty map) |
| Per-reason rollup loop in `apply_reproduce` after mutation stats lines | `v3/crates/v3-core/src/simulation/actions.rs:156–162` | `tests/viability.rs::mutation_skip_reason_tracking_accumulates_correctly` |
| All skip reason keys are valid `MutationSkipReason` Debug names | `v3/crates/v3-core/src/simulation/actions.rs:157` | `tests/viability.rs::mutation_skip_reason_tracking_accumulates_correctly` |
| Sum of per-reason counts equals `mutation_events_skipped_total` | `v3/crates/v3-core/src/simulation/actions.rs:156–162` | `tests/viability.rs::mutation_skip_reason_tracking_accumulates_correctly` |

## Task 2: Fix CLI event schema compliance

| Item | Code Ref | Test |
| --- | --- | --- |
| `RunStartedEvent` field `sample_every: u16` (replaces `initial_population`) | `v3/crates/v3-cli/src/lib.rs:26` | `cli::tests::run_started_is_first_event` |
| `run_started` emits correct `sample_every` value | `v3/crates/v3-cli/src/lib.rs:75` | `cli::tests::run_started_is_first_event` (asserts `sample_every == 1`) |
| `RunCompletedEvent` field `final_mean_energy: f32` added | `v3/crates/v3-cli/src/lib.rs:50` | `cli::tests::run_completed_has_final_mean_energy` |
| `run_completed` emits non-negative `final_mean_energy` | `v3/crates/v3-cli/src/lib.rs:106` | `cli::tests::run_completed_has_final_mean_energy` |

## Task 3: Add `mutation_events_skipped_total_by_reason` to server health payload

| Item | Code Ref | Test |
| --- | --- | --- |
| `build_ws_event` promoted from `pub(crate)` to `pub` | `v3/crates/v3-server/src/handlers/lifecycle.rs:167` | (compile-time; accessible in integration test) |
| `mutation_events_skipped_total_by_reason` field added to `health_payload` | `v3/crates/v3-server/src/handlers/lifecycle.rs:232` | `server::tests::health_payload_contains_mutation_skip_by_reason` |
| Health payload key present after 20 ticks with mutation_probability=1.0 | `v3/crates/v3-server/src/handlers/lifecycle.rs:222` | `server::tests::health_payload_contains_mutation_skip_by_reason` |

## Task 4: Quality gates

| Gate | Result |
| --- | --- |
| `cargo test --workspace` | 39 tests pass; 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (0 warnings) |
| `cargo fmt --all -- --check` | clean |
| `scripts/check-plan-harness.sh --mode strict` | violations=0, warnings=0, files_checked=14 |
| `scripts/check-doc-harness.sh --mode warn` | violations=2 (pre-existing: CLAUDE.md pointer + legacy crate size warnings, not Stage 7) |
| `scripts/check-architecture-harness.sh --mode warn` | violations=0 (5 pre-existing legacy crate size warnings) |

# Stage 7: Observability + Hardening — Implementation Plan

**Goal:** Close the remaining observability contract gap (`mutation_events_skipped_total_by_reason`),
fix CLI event spec compliance gaps (`run_started`/`run_completed` schema mismatches), and deliver
the Stage 7 evidence matrix.

**Goal IDs:** GP-03, GP-04

**Scope:** Add mutation skip reason breakdown to `SimStats`; fix CLI event schemas to match spec;
expose mutation skip by reason in server health payload; add targeted tests; run quality gates.

**Docs Impact:**
- `docs/plans/2026-02-23-v3-stage-7-observability-hardening.md` — created at Task 0
- `docs/plans/archive/reconciliation/2026-02-23-v3-stage-7-evidence-matrix.md` — created at Task 4
- `docs/plans/2026-02-21-v3-implementation-master-plan.md` — Stage 7 row `[x]` at Task 4

**Supersedes:** none
**Superseded-By:** none
**Parent plan:** `docs/plans/2026-02-21-v3-implementation-master-plan.md`

---

## Context

Stages 1–6 built and exposed the full simulation engine. Stage 6 explicitly deferred Stage 7
hardening. Two spec compliance gaps remain:

1. **Observability gap:** `v3-evolution-observability-spec.md` Section 2 requires
   `mutation_events_skipped_total_by_reason` as a counter. `SimStats` has `mutation_events_skipped_total`
   but no per-reason breakdown. `MutationSummary.skip_reasons: HashMap<MutationSkipReason, u32>`
   already tracks the reasons internally; it is just not rolled up into `SimStats`.

2. **CLI event schema gaps:** `v3-cli-contract-spec.md` Section 5 defines:
   - `run_started` must include `sample_every` field — current code emits `initial_population` instead.
   - `run_completed` must include `final_mean_energy` — current code omits this field.

Both are spec compliance bugs introduced in Stage 6 that Stage 7 must fix.

The `domain_selection_weights`, `operator_selection_weights`, and `operator_modifier_scale` config
fields (in `v3-runtime-config-spec.md` Section 3) are not yet implemented and remain deferred;
the runtime config spec notes "Some fields may be staged and not yet implemented in current code."
These are separate mutation-engine features outside Stage 7 scope.

---

## Goal Alignment

- **GP-03:** New tests verify mutation skip reason tracking and CLI event schema compliance.
  All existing tests continue to pass.
- **GP-04:** Closing the `mutation_events_skipped_total_by_reason` gap directly satisfies the
  observability contract. CLI schema fixes ensure events are truthfully spec-compliant.

---

## Boundary Impact

| area | decision | rationale |
| --- | --- | --- |
| `v3-core/src/simulation/stats.rs` | change | add `mutation_events_skipped_by_reason: HashMap<String,u64>` |
| `v3-core/src/simulation/actions.rs` | change | accumulate from `MutationSummary.skip_reasons` |
| `v3-core/tests/viability.rs` | change | add mutation skip reason invariant test |
| `v3-cli/src/lib.rs` | change | fix `RunStartedEvent` and `RunCompletedEvent` schemas |
| `v3-cli/tests/cli.rs` | change | update/add CLI event schema tests |
| `v3-server/src/handlers/lifecycle.rs` | change | add `mutation_events_skipped_total_by_reason` to health payload; promote `build_ws_event` to `pub` |
| `v3-server/tests/server.rs` | change | add health payload mutation skip reason test |

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `mutation/types.rs` (`MutationSkipReason`, `MutationSummary`) | keep | already has `skip_reasons: HashMap<MutationSkipReason,u32>`; no changes needed |
| `mutation/engine.rs` | keep | already populates `skip_reasons` per event; no changes needed |
| `simulation/tick.rs` | keep | no changes needed |
| `simulation/seeding.rs` | keep | `SimStats::default()` zero-inits `HashMap` to empty — no change needed |
| `v3-server/src/handlers/status.rs` | keep | `get_status` does not include mutation stats per spec 4.5 |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| String key naming for mutation skip reasons | Use `format!("{reason:?}")` → `"ParseabilityViolation"`, `"NoApplicableTarget"`, `"BudgetExhausted"` | agent | resolved |
| Health payload JSON key name | `mutation_events_skipped_total_by_reason` (matching `reproduction_actions_rejected_total_by_reason` pattern) | agent | resolved |
| Does `run_started` `initial_population` break existing tests? | No — `run_started_is_first_event` only checks `event_type`; safe to remove `initial_population` | agent | resolved |
| `build_ws_event` visibility for test access | Promote to `pub fn build_ws_event` in `lifecycle.rs`; function already in server crate, no boundary violation | agent | resolved |

---

## Tasks

- [x] **Task 0: Write subplan document to `docs/plans/`**

  Write this plan's full content to
  `docs/plans/2026-02-23-v3-stage-7-observability-hardening.md` before any code changes.
  All tasks marked `[ ]`.

  **Acceptance:** file exists at correct path; `scripts/check-plan-harness.sh --mode strict` passes

- [x] **Task 1: Add `mutation_events_skipped_by_reason` to `SimStats` + wire in `actions.rs`**

  **Files:** `simulation/stats.rs`, `simulation/actions.rs`, `v3-core/tests/viability.rs`

  Add `pub mutation_events_skipped_by_reason: std::collections::HashMap<String, u64>` to `SimStats`
  after `mutation_events_skipped_total`. Wire rollup from `MutationSummary.skip_reasons` in `actions.rs`.
  Add `mutation_skip_reason_tracking_accumulates_correctly` test in `viability.rs`.

  **Acceptance:** `cargo test -p v3-core` green; `SimStats::default()` still compiles everywhere

- [x] **Task 2: Fix CLI event schema compliance**

  **Files:** `v3/crates/v3-cli/src/lib.rs`, `v3/crates/v3-cli/tests/cli.rs`

  Replace `initial_population: usize` with `sample_every: u16` in `RunStartedEvent`.
  Add `final_mean_energy: f32` to `RunCompletedEvent`. Update `run_simulation` construction.
  Update/add CLI tests.

  **Acceptance:** `cargo test -p v3-cli` green; 7 tests pass; `run_started` emits `sample_every`

- [x] **Task 3: Add `mutation_events_skipped_total_by_reason` to server health payload**

  **Files:** `v3-server/src/handlers/lifecycle.rs`, `v3-server/tests/server.rs`

  Promote `build_ws_event` to `pub`. Add `mutation_events_skipped_total_by_reason` to health payload.
  Add `health_payload_contains_mutation_skip_by_reason` test.

  **Acceptance:** `cargo test -p v3-server` green; 14 tests pass

- [x] **Task 4: Quality gates, evidence matrix, master plan update**

  Run full quality gate from `v3/`: `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`. Run harness scripts from repo root.
  Create evidence matrix. Update master plan Stage 7 row to `[x]`.

  **Acceptance:** all gates pass; violations=0 (plan-harness); master plan and subplan updated

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

- `v3/crates/v3-core/src/simulation/stats.rs` — add `mutation_events_skipped_by_reason`
- `v3/crates/v3-core/src/simulation/actions.rs` — wire rollup from `MutationSummary.skip_reasons`
- `v3/crates/v3-core/src/mutation/types.rs` — `MutationSkipReason` enum (Debug names), `MutationSummary.skip_reasons`
- `v3/crates/v3-cli/src/lib.rs` — `RunStartedEvent`, `RunCompletedEvent`, `run_simulation`
- `v3/crates/v3-server/src/handlers/lifecycle.rs` — `build_ws_event`, `health_payload` construction
- `docs/reference/v3-evolution-observability-spec.md` — Section 2 (required counters), Section 3 (required reasons)
- `docs/reference/v3-cli-contract-spec.md` — Section 5.1 (`run_started`), Section 5.3 (`run_completed`)
- `docs/reference/v3-server-api-protocol-spec.md` — Section 5 (WebSocket health payload)

---

**Review cycles:** 1

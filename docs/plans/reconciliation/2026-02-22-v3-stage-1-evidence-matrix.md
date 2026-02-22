# V3 Stage 1 Evidence Matrix (2026-02-22)

Checklist source: `docs/plans/2026-02-21-v3-stage-1-scaffolding-contracts-kernel.md` (`Verification Checklist`, 11 items).

Status taxonomy: `VERIFIED`, `PARTIAL`, `MISSING`, `CONFLICT`, `N/A`.

| item_id | stage | status | code_refs | tests_refs | harness_refs | commit_refs | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S1-C01 | 1 | VERIFIED | `AGENTS.md:93` `AGENTS.md:104` | N/A | N/A | `6deabfb` `aef21de` | Root policy contains explicit rust-skills imperative for v3 Rust work. |
| S1-C02 | 1 | VERIFIED | `v3/crates/v3-core/src/contracts/mod.rs` `v3/crates/v3-core/src/contracts/direction.rs` `v3/crates/v3-core/src/contracts/position.rs` `v3/crates/v3-core/src/contracts/ids.rs` `v3/crates/v3-core/src/contracts/actions.rs` `v3/crates/v3-core/src/contracts/inputs.rs` | `v3/crates/v3-core/src/contracts/direction.rs` tests | N/A | `f5c8877` | Required contracts files exist and are wired. |
| S1-C03 | 1 | VERIFIED | `v3/crates/v3-core/src/config/mod.rs` `v3/crates/v3-core/src/config/simulation.rs` | `v3/crates/v3-core/src/config/simulation.rs:288` (`default_config_matches_spec`) | N/A | `f5c8877` | Config files exist with defaults/normalization and test coverage. |
| S1-C04 | 1 | VERIFIED | `v3/crates/v3-core/src/kernel/mod.rs` `v3/crates/v3-core/src/kernel/grid.rs` `v3/crates/v3-core/src/kernel/world.rs` | `v3/crates/v3-core/src/kernel/world.rs` tests | N/A | `f5c8877` | Required kernel files exist and are wired. |
| S1-C05 | 1 | VERIFIED | N/A | `cargo test --workspace` (`180 passed`) | N/A | N/A | Workspace tests green on reconciliation run (2026-02-22). |
| S1-C06 | 1 | VERIFIED | N/A | N/A | N/A | N/A | `cargo clippy --workspace --all-targets -- -D warnings` passes on reconciliation run. |
| S1-C07 | 1 | VERIFIED | N/A | N/A | N/A | N/A | `cargo fmt --all -- --check` passes on reconciliation run. |
| S1-C08 | 1 | VERIFIED | N/A | N/A | `scripts/check-plan-harness.sh --mode strict` | N/A | Strict plan harness passes after archival of recovered historical master file. |
| S1-C09 | 1 | VERIFIED | N/A | N/A | `scripts/check-architecture-harness.sh --mode warn` | N/A | Warn-only baseline warnings, `violations=0`. |
| S1-C10 | 1 | VERIFIED | `v3/crates/v3-core/src/config/simulation.rs` | `v3/crates/v3-core/src/config/simulation.rs:288` (`default_config_matches_spec`) | N/A | `f5c8877` | Checklist refers to exact default alignment; covered by explicit test. |
| S1-C11 | 1 | VERIFIED | `v3/crates/v3-core/src/kernel/world.rs` | `v3/crates/v3-core/src/kernel/world.rs:310` (`seed_food_deterministic`) | N/A | `f5c8877` | Deterministic seeding test name differs slightly from checklist wording, behavior verified. |

## Snapshot

- verified: 11
- partial: 0
- missing: 0
- conflict: 0

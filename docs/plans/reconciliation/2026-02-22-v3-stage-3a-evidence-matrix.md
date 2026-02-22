# V3 Stage 3a Evidence Matrix (2026-02-22)

Checklist source: `docs/plans/2026-02-21-v3-stage-3a-sensors-vm.md` (`Quality Checklist`, 9 items).

Status taxonomy: `VERIFIED`, `PARTIAL`, `MISSING`, `CONFLICT`, `N/A`.

| item_id | stage | status | code_refs | tests_refs | harness_refs | commit_refs | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S3A-C01 | 3a | VERIFIED | N/A | N/A | N/A | N/A | `cargo fmt --all -- --check` passes on reconciliation run (2026-02-22). |
| S3A-C02 | 3a | VERIFIED | N/A | N/A | N/A | N/A | `cargo clippy --workspace --all-targets -- -D warnings` passes on reconciliation run. |
| S3A-C03 | 3a | VERIFIED | N/A | `cargo test --workspace` (`180 passed`) | N/A | N/A | Workspace tests green on reconciliation run. |
| S3A-C04 | 3a | VERIFIED | `v3/crates/v3-core/src/runtime/types.rs` | `v3/crates/v3-core/src/runtime/types.rs:73` ... `v3/crates/v3-core/src/runtime/types.rs:121` (9 tests total) | N/A | `8066855` | `sanitize_f32` and `NodeResult` are implemented with 9 direct unit tests. |
| S3A-C05 | 3a | VERIFIED | `v3/crates/v3-core/src/sensors/static_inputs.rs` | static input tests in `v3/crates/v3-core/src/sensors/static_inputs.rs` | N/A | `acc235c` | `StaticInputs`, `assemble_static_inputs`, and static reference resolution exist and are tested. |
| S3A-C06 | 3a | VERIFIED | `v3/crates/v3-core/src/runtime/inputs.rs` | tests in `v3/crates/v3-core/src/runtime/inputs.rs` cover `World`, `StaticIntrospection`, `DynamicIntrospection`, `UpstreamSlot` | N/A | `ea0b44c` | `resolve_input` supports all 4 `InputReference` variants with coverage. |
| S3A-C07 | 3a | VERIFIED | `v3/crates/v3-core/src/runtime/action_decode.rs` | tests in `v3/crates/v3-core/src/runtime/action_decode.rs` cover action types 0/1/2/3 plus unknown/edge cases | N/A | `92b9e3b` | Decode semantics implemented and exercised on edge values. |
| S3A-C08 | 3a | VERIFIED | `v3/crates/v3-core/src/runtime/vm.rs` | opcode-family tests plus `max_vm_steps_enforced`, memory/energy tests in `v3/crates/v3-core/src/runtime/vm.rs` | N/A | `34473e5` | `execute_vm_node` covers 33-opcode surface, metering, step cap, wrapping, and memory contract. |
| S3A-C09 | 3a | VERIFIED | `v3/crates/v3-core/src/runtime/vm.rs` | `v3/crates/v3-core/src/runtime/vm.rs:1388` (`vm_eats_when_food_here`) | N/A | `da7405d` | Integration behavior is present; checklist wording "food-present" maps to `vm_eats_when_food_here`. |

## Snapshot

- verified: 9
- partial: 0
- missing: 0
- conflict: 0

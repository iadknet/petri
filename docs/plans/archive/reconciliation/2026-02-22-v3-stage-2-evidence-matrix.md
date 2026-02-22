# V3 Stage 2 Evidence Matrix (2026-02-22)

Checklist source: `docs/plans/2026-02-21-v3-stage-2-creature-schema.md` (`Verification Checklist`, 11 items).

Status taxonomy: `VERIFIED`, `PARTIAL`, `MISSING`, `CONFLICT`, `N/A`.

| item_id | stage | status | code_refs | tests_refs | harness_refs | commit_refs | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S2-C01 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/genome.rs` `v3/crates/v3-core/src/creature/state.rs` `v3/crates/v3-core/src/creature/founder.rs` `v3/crates/v3-core/src/creature/parseability.rs` `v3/crates/v3-core/src/creature/mod.rs` | N/A | N/A | `163d9f4` | Required Stage 2 creature files exist and are wired. |
| S2-C02 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/genome.rs` | `v3/crates/v3-core/src/creature/genome.rs:198` (`vm_instruction_all_33_variants_constructible`) | N/A | `163d9f4` | Checklist intent (33 variants) is satisfied; test name differs from checklist wording. |
| S2-C03 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/genome.rs` | `v3/crates/v3-core/src/creature/genome.rs:280` (`graph_node_kinds_all_22_constructible`) | N/A | `163d9f4` | 22 graph node kinds are constructible; aligns with implemented enum. |
| S2-C04 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/founder.rs:11` | `v3/crates/v3-core/src/creature/founder.rs:246` (`founder_genome_structure`) | N/A | `163d9f4` | `v3alpha1_founder_genome()` returns expected 2-node founder shape. |
| S2-C05 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/parseability.rs` | `v3/crates/v3-core/src/creature/parseability.rs:56` (`founder_genome_passes_parseability`) | N/A | `163d9f4` | Founder genome passes parseability gate. |
| S2-C06 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/parseability.rs` | `v3/crates/v3-core/src/creature/parseability.rs:62` (`empty_node_list_fails`) | N/A | `163d9f4` | Empty-node parseability failure verified. |
| S2-C07 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/parseability.rs` | `v3/crates/v3-core/src/creature/parseability.rs:74` (`duplicate_node_ids_fail`) | N/A | `163d9f4` | Duplicate node ID parseability failure verified. |
| S2-C08 | 2 | VERIFIED | `v3/crates/v3-core/src/creature/parseability.rs` | `v3/crates/v3-core/src/creature/parseability.rs:86` (`unresolved_entry_node_passes_parseability`) | N/A | `163d9f4` | Unresolved entry node passes parseability by design. |
| S2-C09 | 2 | VERIFIED | N/A | `cargo test --workspace` (`180 passed`) | N/A | N/A | Workspace tests green on reconciliation run (2026-02-22). |
| S2-C10 | 2 | VERIFIED | N/A | N/A | N/A | N/A | `cargo clippy --workspace --all-targets -- -D warnings` passes on reconciliation run. |
| S2-C11 | 2 | VERIFIED | N/A | N/A | `scripts/check-plan-harness.sh --mode strict` | N/A | Strict plan harness passes with archived historical master. |

## Snapshot

- verified: 11
- partial: 0
- missing: 0
- conflict: 0

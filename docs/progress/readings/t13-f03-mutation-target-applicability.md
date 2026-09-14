# T13.F03 — Mutation Target Applicability readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f03-mutation-target-applicability.md`](../../specs/roadmap/t13-f03-mutation-target-applicability.md).

## Build-pass verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first, before any edit) | ok. 24 passed; 0 failed; 2 ignored=0; 0.42 s |
| `cargo test -p v3-core --test viability` (after the repair) | ok. 24 passed; 0 failed; 0.29 s |
| `cargo test -p v3-core` | ok. 1412 lib + all integration targets passed; 0 failed; 2 ignored |
| `cargo test -p v3-core --test reproducibility` | ok. 3 passed; 0 failed; 8.95 s |
| `cargo test -p v3-cli` | ok. 101 + 11 + 20 + 20 + 11 passed; 0 failed; 1 ignored |
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets` | clean (no warnings) |
| `cargo fmt --all` | applied |
| `make roadmap-check` | validation passed |

Property tests (`crates/v3-core/src/mutation/applicability_tests.rs`) cover
invariants 1–5. Red-then-green was checked by forcing every predicate to
`true` (the pre-repair select-then-fail behavior): all six tests fail; with
the predicates in place all six pass. No `proptest-regressions` file was
produced by a genuine failure.

## Re-pinned expectations

| Test | Why it moved |
| --- | --- |
| `mutation::engine::tests::a_module_with_no_applicable_site_is_never_selected` (was `a_selected_module_with_no_applicable_site_is_recorded_with_its_node_id`) | The edgeless Graph module is no longer selected; the event is a no-eligible-node skip with `target: None` and every discard carrying `None`. |
| `mutation::engine::tests::an_edge_operator_selects_the_module_that_has_an_edge` (new) | Companion genome with one edgeless and one edged Graph module: edge operators apply to the edged one. |
| `mutation::engine::tests::a_discarded_operator_stays_visible_when_a_later_operator_applied` | Same discard survives the retry, now with `pick == None`. |
| `mutation::vm::tests::register_count_shrink_canonicalizes_every_register_field_or_grows_instead` | A blocked shrink now grows instead of skipping; canonicalization unchanged. |
| `mutation::vm::tests::register_count_grows_when_shrink_is_blocked_and_shrinks_otherwise` | Same, at the operator level. |
| `mutation::vm::tests::register_count_grows_when_shrink_is_blocked_and_preserves_register_identity` | Same, through `VmMutator::apply`. |
| `neighborhood::drift::tests::the_walk_dates_modules_from_their_birth_and_from_every_refresh` | Changed operator mix and RNG stream move this 8-lineage walk: created 14 → 8, dispatched 4 → 2, dispatch TTF `reached` 4 → 3 (median 4 unchanged), internal-change TTF `reached` 6 → 2 (median 6 → 14). |
| `neighborhood::recruitment_paths::experiment::tests::production_prepared_lineages_match_the_recorded_baseline_and_metadata` | New numerators and zero discards, table below. |
| `tests/applied_trajectory.rs::accounting_preserves_pre_feature_sampled_trajectories_and_actions` | Whole-trajectory digest: `5898914f…` → `86eee62c…`. |
| `tests/baseline_worlds.rs::legacy_default_short_run_identity` | 30-tick identity hash: `13138541837675773035` → `12330723111342885916`. |

No assertion was weakened: every re-pin asserts the same property on the
repaired behavior.

## T13.F02 in-report experiment, before and after

`selected_inapplicable_by_backend_operator` summed per backend across all
production lineages of each arm.

| Arm | Numerators before | Numerators after | Graph/VM discards before | after |
| --- | --- | --- | --- | --- |
| graph_prepared / Drift | 19, 14, 12, 1 | 17, 13, 12, 2 | 372 / 332 | 0 / 0 |
| graph_prepared / Selection | 19, 19, 19, 16 | 17, 17, 17, 16 | 331 / 350 | 0 / 0 |
| vm_prepared / Drift | 19, 15, 15, 2 | 18, 14, 14, 1 | 355 / 228 | 0 / 0 |
| vm_prepared / Selection | 19, 19, 19, 15 | 18, 18, 18, 15 | 297 / 228 | 0 / 0 |

Numerators are proposal discovery, retained discovery, viable retained
discovery, retained useful. They move as consequences; the zero discard
columns are the repair.

## Graph applicability audit

Predicate per operator, in `mutation/graph/operators.rs` and
`mutation/graph/hebbian.rs`, shared with the site enumeration application
draws from.

| Operator | Applicable when | Post-draw failure removed |
| --- | --- | --- |
| `AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge` | `has_edge_site` — any edge on any of the 5 surfaces | edgeless module selected |
| `SwapGraphOperator`, `RemoveInternalGraphNode` | `has_compute_node` | module with no compute node selected |
| `MutateGraphOperatorParam` | `has_parameterized_compute_node` | module with no parameterized kind selected |
| `MutateActionSlotBehavior` | `has_action_slot` | empty action bank selected |
| `AddInternalGraphNode` | always (disconnected form appends to any def); the split form is offered only when a splittable edge and capacity exist | split drawn on a module with no splittable edge |
| `AddGraphEdge` | always (the execute gate is always a surface) | none (was already infallible) |
| `GraphRawFieldMutation` | `has_raw_field_site` — a parameterized node, or an edge with a valid unit move | edge with no valid move drawn |
| `CopyInternalNode` | `can_copy_compute_node` — a compute node and room for one copy | capacity or empty-node skip |
| `CopySubgraph` | `can_copy_subgraph` — ≥2 compute nodes and room for `min(4, n)` copies | capacity skip after the walk |
| `CopyEdgeBundle` | `can_copy_edge_bundle` — ≥2 compute nodes and one with edges | edgeless source drawn |
| `EnableHebbian` | `any_node(can_enable_hebbian)` — a node with inputs and no plasticity | no eligible compute node |
| `DisableHebbian`, `MutateHebbianRule`, `MutateHebbianRate`, `ToggleHebbianLamarckian` | `any_node(is_plastic)` | no plastic node |
| `EnableRewardModulation` | `any_node(can_enable_reward_modulation)` | no unmodulated plastic node |
| `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay` | `any_node(is_reward_modulated)` | no modulated node |

`has_raw_field_site` is the one predicate that is not constant time: it tests
the parameterized nodes first and only then walks edges, worst case `O(edges)`
on a module with no parameterized compute node. Every other predicate is a
constant-time check or a short-circuiting scan of compute nodes.

## VM applicability audit

| Operator | Post-draw failure condition (before) | Disposition |
| --- | --- | --- |
| `VmConstantMutation` | none — an empty pool is seeded | predicate `true` |
| `VmInstructionMutation` | none — an empty program takes an insert; replace and delete splices always repair | predicate `true` |
| `VmDeleteInstruction` | program of at most one instruction | `program.len() > 1` |
| `VmRegisterCountMutation` | width outside 1..=32; drawn direction out of bounds; shrink with the removed register in use | `register_count_moves` → direction drawn among feasible moves only |
| `VmInstructionRawFieldMutation` | program of only fieldless instructions (`Noop`, `Halt`, `ExecuteActionQueue`, `PopAction`) | `any(has_mutable_field)` |
| `VmCopyInstructionBlock` | empty program | `!program.is_empty()` |
| `VmCopyInstructionBlockRemapped` | empty program (insert-only splice cannot fail) | `!program.is_empty()` |
| `VmCopyConstantBlock` | empty constant pool | `!constants.is_empty()` |
| `VmCopyGeneBackwardSlice` | no output instruction to anchor the slice | `any(vm_is_output_instruction)` |
| `VmCopyGeneForwardSlice` | no register-writing instruction to seed the slice | `any(vm_register_write(..).is_some())` |
| `VmInsertReadStoreMotif`, `VmInsertReadBidMotif` | no input reference to read | `!input_refs.is_empty()` |
| `VmInsertLoadCompareMotif` | none — inserts into any program | predicate `true` |
| `VmMutateSlotAddress` | no slot instruction | `any(is_slot_instruction)` |
| `VmMutatePairedSlotAddress` | no slot carrying both a load and a store | `!paired_slot_groups(..).is_empty()` |

## InputRef audit — no change

| Operator | Post-draw failure condition | Disposition |
| --- | --- | --- |
| `Add` | none — draws over every node and pushes a reference | unchanged |
| `Remove` | none — the eligible set is already filtered to nodes with a non-empty `input_refs` before `TargetSelector::select` | unchanged |
| `Swap` | none — same pre-filtered eligible set | unchanged |
| `RawFieldMutation` | none after a draw — it does not select a mesh node at all; it counts `UpstreamSlot` and typed-food references genome-wide and skips only when that total is zero | unchanged (exempt from both bias layers, `NotApplicable`) |

The domain shows none of the select-then-fail defect, so it is untouched by
this feature.

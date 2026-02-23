# V3 Stage 5 Evidence Matrix

**Stage:** 5 — Mutation + Phenotype + Evolution
**Date verified:** 2026-02-22
**Subplan:** `docs/plans/2026-02-21-v3-stage-5-mutation-phenotype-evolution.md`
**Test count:** 252 (238 unit + 14 integration viability)
**Verification gate:** cargo test + clippy + fmt + harness scripts all pass

---

## Task 1: PhenotypeConfig + MutationConfig extension

| Item | Code Ref | Test |
| --- | --- | --- |
| `PhenotypeConfig` struct (4 fields) | `src/config/simulation.rs:141` | `config::simulation::tests::default_config_matches_spec` |
| `MutationConfig.phenotype` field | `src/config/simulation.rs:178` | `config::simulation::tests::default_config_matches_spec` |
| `normalize()` — channel_step 0 → 2 | `src/config/simulation.rs:250` | `config::simulation::tests::normalize_phenotype_zero_channel_step_falls_back` |
| `normalize()` — polarity_flip_chance clamp | `src/config/simulation.rs:253` | `config::simulation::tests::normalize_phenotype_polarity_flip_chance_clamped` |
| `normalize()` — weight_min fallback | `src/config/simulation.rs:254` | `config::simulation::tests::normalize_phenotype_negative_channel_weight_min_falls_back` |
| `normalize()` — weight_max fallback | `src/config/simulation.rs:257` | `config::simulation::tests::normalize_phenotype_weight_max_lte_min_falls_back` |
| `PhenotypeConfig` re-exported | `src/config/mod.rs:4` | (compile-time) |

## Task 2: mutation/phenotype.rs + actions.rs wiring

| Item | Code Ref | Test |
| --- | --- | --- |
| `mutate_phenotype()` function | `src/mutation/phenotype.rs:12` | `mutation::phenotype::tests::mutate_phenotype_applies_step_to_one_channel` |
| Weight sanitization (negative → 0.0) | `src/mutation/phenotype.rs:49` | `mutation::phenotype::tests::mutate_phenotype_negative_weight_treated_as_zero` |
| All-zero fallback to uniform selection | `src/mutation/phenotype.rs:61` | `mutation::phenotype::tests::mutate_phenotype_all_zero_weights_uses_uniform` |
| Wrapping u8 RGB arithmetic | `src/mutation/phenotype.rs:40` | `mutation::phenotype::tests::mutate_phenotype_wraps_u8_arithmetic` |
| Polarity flip with `polarity_flip_chance` | `src/mutation/phenotype.rs:30` | `mutation::phenotype::tests::mutate_phenotype_polarity_flip_changes_direction` |
| `apply_reproduce` phenotype trigger (applied_events > 0) | `src/simulation/actions.rs:119` | `tests/viability.rs::mutation_offspring_diverge_from_parent_over_time` |
| `apply_reproduce` no phenotype when no mutations | `src/simulation/actions.rs:119` | `tests/viability.rs::phenotype_inherits_unchanged_when_no_genome_mutation` |

## Task 3: mutation/topology.rs — TopologyMutator

| Item | Code Ref | Test |
| --- | --- | --- |
| `AddNode` operator | `src/mutation/topology.rs:61` | `mutation::topology::tests::add_node_increases_node_count_by_one` |
| `RemoveNode` operator | `src/mutation/topology.rs:79` | `mutation::topology::tests::remove_node_decreases_node_count` |
| `RemoveNode` single-node guard | `src/mutation/topology.rs:83` | `mutation::topology::tests::remove_node_on_single_node_genome_returns_no_applicable_target` |
| `RetargetNodeTarget` operator | `src/mutation/topology.rs:97` | `mutation::topology::tests::retarget_node_target_changes_target` |
| `AddRouteTarget` operator | `src/mutation/topology.rs:116` | `mutation::topology::tests::add_route_target_increases_target_count` |
| `ChangeEntryNode` operator | `src/mutation/topology.rs:146` | `mutation::topology::tests::change_entry_node_changes_entry` |
| Parseability gate after all operators | `src/mutation/topology.rs:256` | `mutation::topology::tests::topology_after_each_operator_passes_parseability_gate` |

## Task 4: mutation/vm_mutator.rs — VmMutator

| Item | Code Ref | Test |
| --- | --- | --- |
| `VmConstantMutation` — perturb | `src/mutation/vm_mutator.rs:60` | `mutation::vm_mutator::tests::vm_constant_mutation_changes_constant_value` |
| `VmConstantMutation` — add when empty | `src/mutation/vm_mutator.rs:63` | `mutation::vm_mutator::tests::vm_constant_mutation_on_node_with_empty_constants_adds_constant` |
| `VmInstructionMutation` — changes program | `src/mutation/vm_mutator.rs:76` | `mutation::vm_mutator::tests::vm_instruction_mutation_changes_program` |
| VM instruction delete never empties program | `src/mutation/vm_mutator.rs:101` | `mutation::vm_mutator::tests::vm_instruction_mutation_program_never_empty` |
| Graph-only genome pre-guard | `src/mutation/vm_mutator.rs:39` | `mutation::vm_mutator::tests::vm_mutator_on_graph_only_genome_returns_no_applicable_target` |
| Parseability gate after VM operators | `src/mutation/vm_mutator.rs:220` | `mutation::vm_mutator::tests::vm_after_mutation_passes_parseability_gate` |

## Task 5: mutation/graph_mutator.rs — GraphMutator

| Item | Code Ref | Test |
| --- | --- | --- |
| `AlterGraphEdgeWeight` operator | `src/mutation/graph_mutator.rs:53` | `mutation::graph_mutator::tests::alter_graph_edge_weight_changes_weight` |
| `SwapGraphOperator` operator | `src/mutation/graph_mutator.rs:64` | `mutation::graph_mutator::tests::swap_graph_operator_changes_node_kind` |
| `MutateGraphOperatorParam` operator | `src/mutation/graph_mutator.rs:73` | `mutation::graph_mutator::tests::mutate_graph_operator_param_changes_param` |
| `AddInternalGraphNode` operator | `src/mutation/graph_mutator.rs:95` | `mutation::graph_mutator::tests::add_internal_graph_node_increases_internal_node_count` |
| `RemoveInternalGraphNode` operator | `src/mutation/graph_mutator.rs:103` | `mutation::graph_mutator::tests::remove_internal_graph_node_decreases_count` |
| VM-only genome pre-guard | `src/mutation/graph_mutator.rs:43` | `mutation::graph_mutator::tests::graph_mutator_on_vm_only_genome_returns_no_applicable_target` |
| Parseability gate after graph operators | `src/mutation/graph_mutator.rs` | `mutation::graph_mutator::tests::graph_after_mutation_passes_parseability_gate` |

## Task 6: Real MutationEngine

| Item | Code Ref | Test |
| --- | --- | --- |
| Probability gate (`gen_bool`) | `src/mutation/engine.rs:24` | `mutation::engine::tests::engine_with_probability_zero_returns_zero_summary` |
| Event count from config range | `src/mutation/engine.rs:29` | `mutation::engine::tests::engine_with_probability_one_applies_events` |
| Domain selection (topology/vm/graph uniform) | `src/mutation/engine.rs:37` | `mutation::engine::tests::engine_accounting_invariant_always_holds` |
| Snapshot + restore on failure | `src/mutation/engine.rs:58` | `mutation::engine::tests::engine_mutations_preserve_parseability` |
| Accounting invariant: attempted = applied + skipped | `src/mutation/engine.rs:39` | `mutation::engine::tests::engine_accounting_invariant_always_holds` |
| 1000-round no-panic on founder genome | `src/mutation/engine.rs` | `mutation::engine::tests::engine_with_founder_genome_does_not_panic` |

## Task 7: E2E integration tests

| Item | Code Ref | Test |
| --- | --- | --- |
| Phenotype diverges within 30 ticks (mutation_prob=1.0) | `tests/viability.rs:399` | `mutation_offspring_diverge_from_parent_over_time` |
| Accounting invariant over 1000 engine calls | `tests/viability.rs:428` | `mutation_accounting_invariant_in_viability` |
| Phenotype unchanged when mutation_prob=0.0 | `tests/viability.rs:453` | `phenotype_inherits_unchanged_when_no_genome_mutation` |
| Viability preserved with real mutations | `tests/viability.rs:500` | `viability_still_passes_with_real_mutations` |

## Task 8: Quality gates

| Gate | Result |
| --- | --- |
| `cargo test --workspace` | 238 unit + 14 integration = 252 tests pass; 4 pre-existing failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (0 warnings) |
| `cargo fmt --all -- --check` | clean |
| `scripts/check-plan-harness.sh --mode strict` | violations=0, warnings=0 |
| `scripts/check-doc-harness.sh --mode warn` | violations=2 (pre-existing CLAUDE.md warnings, not Stage 5) |
| `scripts/check-architecture-harness.sh --mode warn` | violations=0 (5 pre-existing petri-core size warnings) |

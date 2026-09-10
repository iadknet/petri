use super::*;
use crate::config::MutationConfig;
use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{NodeId, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{
    ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate,
    GraphEdge, GraphSource, OutputSink, OutputSinkKind, WorldActionKind,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::compound::sub_value_count;
use crate::mutation::sampling::{random_input_reference, random_input_reference_for_food_types};
use crate::runtime::OUTPUT_SLOT_COUNT;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn default_config() -> MutationConfig {
    MutationConfig::default()
}

fn single_node_genome_with_input_ref(input_ref: InputReference) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![input_ref],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn add_input_ref_increases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    let mut r = rng(0);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert_eq!(after, before + 1);
}

#[test]
fn remove_input_ref_decreases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert!(before > 0, "founder must have input_refs");
    let mut r = rng(0);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Remove,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert_eq!(after, before - 1);
}

#[test]
fn remove_input_ref_on_empty_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    for node in &mut genome.nodes {
        node.input_refs.clear();
    }
    let mut r = rng(0);
    let result = InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Remove,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn swap_input_ref_changes_value() {
    let genome = v3alpha1_founder_genome();
    let original_refs: Vec<InputReference> = genome
        .nodes
        .iter()
        .flat_map(|n| n.input_refs.iter().cloned())
        .collect();
    let mut changed = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut g,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .is_ok()
        {
            let new_refs: Vec<InputReference> = g
                .nodes
                .iter()
                .flat_map(|n| n.input_refs.iter().cloned())
                .collect();
            if new_refs != original_refs {
                changed = true;
                break;
            }
        }
    }
    assert!(changed, "swap must change at least one input ref");
}

#[test]
fn swap_graph_input_ref_clamps_out_of_range_sub_indices() {
    let config = default_config();
    let def = CgpGraphBackendDef {
        birth_weights: None,
        compute_nodes: vec![ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 3,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                },
            ],
            plasticity: None,
        }],
        output_sinks: vec![OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 2,
                },
                weight: 1.0,
            }],
        }],
        action_bank: vec![ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 4,
                },
                weight: 1.0,
            }],
            param_inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 1.0,
            }],
        }],
        execute_gate: ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 5,
                },
                weight: 1.0,
            }],
        },
    };
    let base_genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::ActionQueue],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    };

    for seed in 0u64..5_000 {
        let mut genome = base_genome.clone();
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &config,
        )
        .unwrap();

        let new_ref = genome.nodes[0].input_refs[0].clone();
        if sub_value_count(&new_ref, &config) != 1 {
            continue;
        }

        let BackendDef::Graph(graph) = &genome.nodes[0].backend_def else {
            panic!("expected graph backend");
        };

        // Compute input with sub_idx 3 should be removed, sub_idx 0 should remain.
        assert_eq!(graph.compute_nodes[0].inputs.len(), 1);
        assert!(matches!(
            graph.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0
            }
        ));

        // Wired surface edges with out-of-range sub_idx should be removed.
        assert!(graph.output_sinks[0].inputs.is_empty());
        assert!(graph.action_bank[0].gate_inputs.is_empty());
        assert!(graph.execute_gate.inputs.is_empty());

        // Valid sub_idx 0 action param edge should remain.
        assert_eq!(graph.action_bank[0].param_inputs.len(), 1);
        assert!(matches!(
            graph.action_bank[0].param_inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0
            }
        ));
        return;
    }

    panic!("did not hit scalar swap target in tested seeds");
}

#[test]
fn random_input_reference_covers_all_categories() {
    use std::collections::HashSet;
    let mut categories: HashSet<String> = HashSet::new();
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        let ir = random_input_reference(&mut r);
        let cat = match ir {
            InputReference::World(WorldInputKey::FoodHere { .. }) => "FoodHere".to_string(),
            InputReference::World(WorldInputKey::NeighborFoodRing { .. }) => {
                "NeighborFoodRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborBarrierRing) => {
                "NeighborBarrierRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborOccupiedRing) => {
                "NeighborOccupiedRing".to_string()
            }
            InputReference::World(WorldInputKey::AreaFoodSummary { .. }) => {
                "AreaFoodSummary".to_string()
            }
            InputReference::World(WorldInputKey::AreaBarrierSummary) => {
                "AreaBarrierSummary".to_string()
            }
            InputReference::World(WorldInputKey::AreaOccupancySummary) => {
                "AreaOccupancySummary".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureCore) => {
                "NearbyCreatureCore".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureVitals) => {
                "NearbyCreatureVitals".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureIdentity) => {
                "NearbyCreatureIdentity".to_string()
            }
            InputReference::StaticIntrospection(_) => "StaticIntrospection".to_string(),
            InputReference::DynamicIntrospection(_) => "DynamicIntrospection".to_string(),
            InputReference::UpstreamSlot(_) => "UpstreamSlot".to_string(),
            InputReference::ActionQueue => "ActionQueue".to_string(),
        };
        categories.insert(cat);
    }
    // 14 categories: 8 original + 6 extended perception families
    assert_eq!(
        categories.len(),
        14,
        "all 14 input reference categories must be reachable; got {:?}",
        categories
    );
}

#[test]
fn random_input_reference_upstream_slot_bounded() {
    for seed in 0u64..20_000 {
        let mut r = rng(seed);
        if let InputReference::UpstreamSlot(slot) = random_input_reference(&mut r) {
            assert!(
                slot < OUTPUT_SLOT_COUNT,
                "upstream slot must be bounded < {OUTPUT_SLOT_COUNT}, got {slot}"
            );
        }
    }
}

#[test]
fn raw_field_mutation_upstream_slot_bounded() {
    for seed in 0u64..512 {
        let mut genome = single_node_genome_with_input_ref(InputReference::UpstreamSlot(0));
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::RawFieldMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .unwrap();
        if let InputReference::UpstreamSlot(slot) = genome.nodes[0].input_refs[0] {
            assert!(
                slot < OUTPUT_SLOT_COUNT,
                "mutated upstream slot must be bounded < {OUTPUT_SLOT_COUNT}, got {slot}"
            );
        }
    }
}

#[test]
fn input_ref_after_mutation_passes_parseability_gate() {
    let operators = [
        InputRefOperator::Add,
        InputRefOperator::Remove,
        InputRefOperator::Swap,
        InputRefOperator::RawFieldMutation,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 400);
        let _ = InputRefMutator::apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        );
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability failed after {:?}",
            op
        );
    }
}

#[test]
fn input_ref_weighted_random_favors_refinement() {
    let mut counts = std::collections::HashMap::new();
    let mut r = rng(42);
    for _ in 0..10_000 {
        let op = InputRefOperator::random(&mut r);
        *counts.entry(op).or_insert(0u32) += 1;
    }
    let swap = counts.get(&InputRefOperator::Swap).copied().unwrap_or(0);
    let add = counts.get(&InputRefOperator::Add).copied().unwrap_or(0);
    assert!(
        swap > add + (add / 2),
        "Swap (weight 4) must appear >1.5x Add (weight 2); got {} vs {}",
        swap,
        add,
    );
}

#[test]
fn input_ref_operator_weights_are_positive() {
    let all = InputRefOperator::ALL;
    assert_eq!(
        all.len(),
        4,
        "ALL must cover every InputRefOperator variant"
    );
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &InputRefOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

// ── InputRef::Add wires nothing (T11.F03) ───────────────────────────────────

fn graph_node_genome_zero_refs() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![],
                    plasticity: None,
                }],
                output_sinks: vec![OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: vec![],
                }],
                action_bank: vec![ActionSlot {
                    behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                    gate_inputs: vec![],
                    param_inputs: vec![],
                }],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
            targets: vec![],
        }],
    }
}

fn count_input_leaf_edges_for_ref(genome: &CreatureGenome, node_idx: usize, ref_idx: u16) -> usize {
    let BackendDef::Graph(ref def) = genome.nodes[node_idx].backend_def else {
        return 0;
    };
    let mut count = 0;
    for node in &def.compute_nodes {
        count += node
            .inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
    }
    for sink in &def.output_sinks {
        count += sink
            .inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
    }
    for slot in &def.action_bank {
        count += slot
            .gate_inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
        count += slot
            .param_inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
    }
    count += def
        .execute_gate
        .inputs
        .iter()
        .filter(|e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx))
        .count();
    count
}

/// `InputRef::Add` is a growth operator: it must be neutral at fire time
/// (`docs/reference/v3-mutation-spec.md`'s node-type contract), so it wires
/// the new reference into nothing on the graph backend, no matter how many
/// sub-values the sampled reference has.
#[test]
fn add_input_ref_to_graph_node_wires_nothing() {
    for seed in 0u64..200 {
        let mut genome = graph_node_genome_zero_refs();
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .unwrap();
        assert_eq!(genome.nodes[0].input_refs.len(), 1, "seed {seed}");
        assert_eq!(
            count_input_leaf_edges_for_ref(&genome, 0, 0),
            0,
            "seed {seed}: InputRef::Add must not wire the new reference into any edge"
        );
    }
}

#[test]
fn add_input_ref_to_graph_node_leaves_existing_edges_untouched() {
    let mut genome = graph_node_genome_zero_refs();
    genome.nodes[0]
        .input_refs
        .push(InputReference::World(WorldInputKey::food_here(
            OrdinaryFoodTypeId::default(),
        )));
    let before = genome.clone();
    let mut r = rng(99);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    assert_eq!(genome.nodes[0].input_refs.len(), 2);
    let BackendDef::Graph(ref before_def) = before.nodes[0].backend_def else {
        panic!("expected graph backend");
    };
    let BackendDef::Graph(ref after_def) = genome.nodes[0].backend_def else {
        panic!("expected graph backend");
    };
    assert_eq!(
        before_def, after_def,
        "InputRef::Add must not touch any existing edge"
    );
}

/// The VM `ReadInput` auto-insertion is removed: `InputRef::Add` on a VM
/// node leaves the program byte-identical.
#[test]
fn add_input_ref_to_vm_node_leaves_program_unchanged() {
    let mut genome = single_node_genome_with_input_ref(InputReference::World(
        WorldInputKey::food_here(OrdinaryFoodTypeId::default()),
    ));
    genome.nodes[0].input_refs.clear(); // start with 0 refs
    let program_before = match &genome.nodes[0].backend_def {
        BackendDef::Vm(vm) => vm.program.clone(),
        _ => panic!("expected VM backend"),
    };
    let mut r = rng(42);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    assert_eq!(genome.nodes[0].input_refs.len(), 1);
    let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    assert_eq!(
        vm.program, program_before,
        "InputRef::Add must not insert or otherwise change any VM instruction"
    );
}

// Legacy flat-graph lifecycle tests were removed during the CGP migration.
// The remaining Graph coverage lives in genome/mod.rs tests.

#[test]
fn reindex_vm_decrements_and_noops_orphans() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
                InputReference::UpstreamSlot(0),
                InputReference::UpstreamSlot(1),
            ],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::ReadInput {
                        dst: 1,
                        ref_idx: 1,
                        sub_idx: 0,
                    },
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 2,
                        sub_idx: 5,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    // Remove input ref at index 1
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(1);
    if let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def {
        // ref_idx 0 < 1 -> unchanged
        assert!(matches!(
            vm.program[0],
            VmInstruction::ReadInput {
                ref_idx: 0,
                sub_idx: 0,
                ..
            }
        ));
        // ref_idx 1 == removed -> converted to Noop (not u16::MAX zombie)
        assert!(
            matches!(vm.program[1], VmInstruction::Noop),
            "orphaned ReadInput must become Noop, got {:?}",
            vm.program[1]
        );
        // ref_idx 2 > 1 -> decremented to 1
        assert!(matches!(
            vm.program[2],
            VmInstruction::ReadInput {
                ref_idx: 1,
                sub_idx: 5,
                ..
            }
        ));
    } else {
        panic!("expected VM backend");
    }
}

#[test]
fn reindex_noop_on_empty_vm_program() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::food_here(
                OrdinaryFoodTypeId::default(),
            ))],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![], // empty program
            }),
            targets: vec![],
        }],
    };
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(0); // should not panic
}

#[test]
fn extended_perception_families_reachable_in_pool() {
    let mut found = [false; 6]; // Food, Barrier, Occupancy, Core, Vitals, Identity
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        match random_input_reference(&mut r) {
            InputReference::World(WorldInputKey::AreaFoodSummary { .. }) => found[0] = true,
            InputReference::World(WorldInputKey::AreaBarrierSummary) => found[1] = true,
            InputReference::World(WorldInputKey::AreaOccupancySummary) => found[2] = true,
            InputReference::World(WorldInputKey::NearbyCreatureCore) => found[3] = true,
            InputReference::World(WorldInputKey::NearbyCreatureVitals) => found[4] = true,
            InputReference::World(WorldInputKey::NearbyCreatureIdentity) => found[5] = true,
            _ => {}
        }
        if found.iter().all(|&f| f) {
            break;
        }
    }
    let names = [
        "AreaFoodSummary",
        "AreaBarrierSummary",
        "AreaOccupancySummary",
        "NearbyCreatureCore",
        "NearbyCreatureVitals",
        "NearbyCreatureIdentity",
    ];
    for (i, &f) in found.iter().enumerate() {
        assert!(
            f,
            "{} must be reachable from random_input_reference",
            names[i]
        );
    }
}

#[test]
fn action_queue_appears_in_random_input_reference_pool() {
    let mut found_aq = false;
    for seed in 0u64..500 {
        let mut r = rng(seed);
        if random_input_reference(&mut r) == InputReference::ActionQueue {
            found_aq = true;
            break;
        }
    }
    assert!(
        found_aq,
        "ActionQueue must be reachable from random_input_reference"
    );
}

fn any_non_default_food_type(input_refs: &[InputReference]) -> bool {
    input_refs.iter().any(|input_ref| match input_ref {
        InputReference::World(WorldInputKey::FoodHere { type_idx })
        | InputReference::World(WorldInputKey::NeighborFoodRing { type_idx })
        | InputReference::World(WorldInputKey::AreaFoodSummary { type_idx }) => {
            *type_idx != OrdinaryFoodTypeId::default()
        }
        _ => false,
    })
}

#[test]
fn random_input_reference_for_food_types_can_sample_non_default_food_type() {
    let mut r = rng(0xC0FFEE);
    let saw_non_default = (0..2048)
        .map(|_| random_input_reference_for_food_types(&mut r, 3))
        .any(|input_ref| any_non_default_food_type(&[input_ref]));

    assert!(
        saw_non_default,
        "typed sampler should produce food refs with non-default type_idx when 3 food types are available"
    );
}

#[test]
fn input_ref_add_can_introduce_non_default_food_type() {
    let mut r = rng(0xA11D);
    let mut saw_non_default = false;
    for _ in 0..1024 {
        let mut genome = v3alpha1_founder_genome();
        let _ = InputRefMutator::apply_with_food_type_count(
            &mut genome,
            InputRefOperator::Add,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            3,
        );
        if genome
            .nodes
            .iter()
            .any(|node| any_non_default_food_type(&node.input_refs))
        {
            saw_non_default = true;
            break;
        }
    }

    assert!(
        saw_non_default,
        "InputRef.Add should be able to introduce food sensors for non-default food types"
    );
}

#[test]
fn input_ref_swap_can_introduce_non_default_food_type() {
    let mut r = rng(0x5A9);
    let mut saw_non_default = false;
    for _ in 0..1024 {
        let mut genome = v3alpha1_founder_genome();
        let _ = InputRefMutator::apply_with_food_type_count(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            4,
        );
        if genome
            .nodes
            .iter()
            .any(|node| any_non_default_food_type(&node.input_refs))
        {
            saw_non_default = true;
            break;
        }
    }

    assert!(
        saw_non_default,
        "InputRef.Swap should be able to produce food refs with non-default type_idx"
    );
}

#[test]
fn input_ref_raw_field_mutation_can_mutate_food_type_idx() {
    let mut genome = single_node_genome_with_input_ref(InputReference::World(
        WorldInputKey::food_here(OrdinaryFoodTypeId::default()),
    ));
    let mut r = rng(0xFACE);
    let result = InputRefMutator::apply_with_food_type_count(
        &mut genome,
        InputRefOperator::RawFieldMutation,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
        3,
    );

    assert!(
        result.is_ok() && any_non_default_food_type(&genome.nodes[0].input_refs),
        "InputRef.RawFieldMutation should be able to mutate food sensor type_idx when multiple food types exist"
    );
}

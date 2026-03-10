use super::*;
use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInternalNode, GraphNodeKind, NodeGenome,
    VmBackendDef, VmInstruction,
};
use crate::creature::parseability::ParseabilityGate;
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
        &[],
        0.0,
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
        &[],
        0.0,
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
        &[],
        0.0,
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
            &[],
            0.0,
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
fn random_input_reference_covers_all_categories() {
    use std::collections::HashSet;
    let mut categories: HashSet<String> = HashSet::new();
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        let ir = random_input_reference(&mut r);
        let cat = match ir {
            InputReference::World(WorldInputKey::FoodHere) => "FoodHere".to_string(),
            InputReference::World(WorldInputKey::NeighborFoodRing) => {
                "NeighborFoodRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborBarrierRing) => {
                "NeighborBarrierRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborOccupiedRing) => {
                "NeighborOccupiedRing".to_string()
            }
            InputReference::World(WorldInputKey::AreaFoodSummary) => "AreaFoodSummary".to_string(),
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
            assert!(slot < 12, "upstream slot must be bounded < 12, got {slot}");
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
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .unwrap();
        if let InputReference::UpstreamSlot(slot) = genome.nodes[0].input_refs[0] {
            assert!(
                slot < 12,
                "mutated upstream slot must be bounded < 12, got {slot}"
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
        let _ = InputRefMutator::apply(&mut genome, op, &[], 0.0, &mut r, &default_config());
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

#[test]
fn random_non_increasing_never_returns_increasing() {
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = InputRefOperator::random_non_increasing(&mut r) {
            assert_ne!(
                op.complexity_effect(),
                ComplexityEffect::Increasing,
                "random_non_increasing returned Increasing operator {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_non_increasing_covers_neutral_and_decreasing() {
    use crate::mutation::types::ComplexityEffect;
    let mut saw_neutral = false;
    let mut saw_decreasing = false;
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        if let Some(op) = InputRefOperator::random_non_increasing(&mut r) {
            match op.complexity_effect() {
                ComplexityEffect::Neutral => saw_neutral = true,
                ComplexityEffect::Decreasing => saw_decreasing = true,
                ComplexityEffect::Increasing => unreachable!(),
            }
        }
        if saw_neutral && saw_decreasing {
            break;
        }
    }
    assert!(saw_neutral, "must produce at least one neutral operator");
    assert!(
        saw_decreasing,
        "must produce at least one decreasing operator"
    );
}

#[test]
fn random_decreasing_never_returns_non_decreasing() {
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = InputRefOperator::random_decreasing(&mut r) {
            assert_eq!(
                op.complexity_effect(),
                ComplexityEffect::Decreasing,
                "random_decreasing returned non-Decreasing operator {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_decreasing_covers_all_decreasing_operators() {
    use std::collections::HashSet;
    let expected: HashSet<InputRefOperator> = InputRefOperator::ALL
        .iter()
        .copied()
        .filter(|op| op.complexity_effect().is_decreasing())
        .collect();
    let mut seen = HashSet::new();
    for seed in 0u64..2000 {
        let mut r = rng(seed);
        if let Some(op) = InputRefOperator::random_decreasing(&mut r) {
            seen.insert(op);
        }
        if seen == expected {
            break;
        }
    }
    assert_eq!(
        seen, expected,
        "random_decreasing must cover all Decreasing operators"
    );
}

// -- reindex_input_refs_after_removal tests --------------------------------

fn graph_genome_with_input_refs(
    input_refs: Vec<InputReference>,
    nodes: Vec<GraphInternalNode>,
) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: nodes,
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn reindex_graph_decrements_refs_above_removed() {
    let genome = graph_genome_with_input_refs(
        vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::UpstreamSlot(0),
            InputReference::UpstreamSlot(1),
        ],
        vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 2,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
        ],
    );
    let mut g = genome;
    // Remove input ref at index 1 -> ref_idx 0 stays, ref_idx 2 -> 1
    g.nodes[0].backend_def.reindex_input_refs_after_removal(1);
    if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
        // ref_idx 0 < removed(1) -> unchanged
        assert_eq!(
            gd.internal_nodes[0].kind,
            GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0
            }
        );
        // ref_idx 2 > removed(1) -> decremented to 1
        assert_eq!(
            gd.internal_nodes[1].kind,
            GraphNodeKind::InputRef {
                ref_idx: 1,
                sub_idx: 0
            }
        );
    } else {
        panic!("expected Graph backend");
    }
}

#[test]
fn reindex_graph_invalidates_removed_ref() {
    let genome = graph_genome_with_input_refs(
        vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::UpstreamSlot(0),
        ],
        vec![GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: 1,
                sub_idx: 3,
            },
            inputs: vec![],
            plasticity: None,
        }],
    );
    let mut g = genome;
    // Remove the ref at index 1 -> ref_idx 1 == removed -> invalidate
    g.nodes[0].backend_def.reindex_input_refs_after_removal(1);
    if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
        assert_eq!(
            gd.internal_nodes[0].kind,
            GraphNodeKind::InputRef {
                ref_idx: u16::MAX,
                sub_idx: 3
            }
        );
    } else {
        panic!("expected Graph backend");
    }
}

#[test]
fn reindex_vm_decrements_and_invalidates() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::FoodHere),
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
        // ref_idx 1 == removed -> invalidated to u16::MAX
        assert!(matches!(
            vm.program[1],
            VmInstruction::ReadInput {
                ref_idx: u16::MAX,
                sub_idx: 0,
                ..
            }
        ));
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
fn remove_operator_calls_reindex() {
    // Build a genome with 2 input refs and a graph node referencing ref_idx 1.
    // After Remove removes ref at index 0, the graph node's ref_idx should be 0 (decremented).
    let genome = graph_genome_with_input_refs(
        vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::UpstreamSlot(0),
        ],
        vec![GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: 1,
                sub_idx: 0,
            },
            inputs: vec![],
            plasticity: None,
        }],
    );
    // Force rng to pick node 0 and ref_idx 0 for removal.
    // With seed search, find one that removes index 0.
    let mut found = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut g,
            InputRefOperator::Remove,
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .is_ok()
            && g.nodes[0].input_refs.len() == 1
        {
            // Removed one ref. Check if the remaining is UpstreamSlot(0)
            // meaning we removed index 0 (FoodHere).
            if g.nodes[0].input_refs[0] == InputReference::UpstreamSlot(0) {
                // Ref at index 0 was removed -> old ref_idx 1 should be 0 now.
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    assert_eq!(
                        gd.internal_nodes[0].kind,
                        GraphNodeKind::InputRef {
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        "Remove must call reindex to decrement ref_idx above removed"
                    );
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(found, "must find a seed that removes index 0");
}

#[test]
fn reindex_noop_on_empty_graph_backend() {
    let mut genome = graph_genome_with_input_refs(
        vec![InputReference::World(WorldInputKey::FoodHere)],
        vec![], // no internal nodes
    );
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(0); // should not panic
}

#[test]
fn reindex_noop_on_empty_vm_program() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
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
fn reindex_graph_invalidates_all_matching_refs() {
    let mut genome = graph_genome_with_input_refs(
        vec![InputReference::World(WorldInputKey::FoodHere)],
        vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 1,
                },
                inputs: vec![],
                plasticity: None,
            },
        ],
    );
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(0);
    if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
        for (i, node) in gd.internal_nodes.iter().enumerate() {
            if let GraphNodeKind::InputRef { ref_idx, .. } = node.kind {
                assert_eq!(
                    ref_idx,
                    u16::MAX,
                    "internal node {} should be invalidated",
                    i
                );
            }
        }
    } else {
        panic!("expected Graph backend");
    }
}

// -- Phase 2b compound-aware tests ----------------------------------------

#[test]
fn add_compound_input_creates_fan_out_nodes() {
    // Build a graph genome with no input refs. Force Add to pick ActionQueue.
    // Since random_input_reference picks ActionQueue at idx 29, we search seeds.
    let mut found = false;
    for seed in 0u64..2000 {
        let mut genome = graph_genome_with_input_refs(vec![], vec![]);
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .unwrap();
        if genome.nodes[0].input_refs.last() == Some(&InputReference::ActionQueue) {
            // ActionQueue was added -- should have fan-out nodes
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                // default config: action_queue_cap=4 -> 12 fan-out nodes
                assert_eq!(
                    gd.internal_nodes.len(),
                    12,
                    "compound Add must create 4*3=12 fan-out InputRef nodes"
                );
                for (i, n) in gd.internal_nodes.iter().enumerate() {
                    assert_eq!(
                        n.kind,
                        GraphNodeKind::InputRef {
                            ref_idx: 0,
                            sub_idx: i as u16,
                        }
                    );
                }
                found = true;
                break;
            }
        }
    }
    assert!(found, "must find a seed producing ActionQueue input ref");
}

#[test]
fn swap_to_compound_creates_fan_out_nodes() {
    // Start with a scalar input ref, swap to ActionQueue.
    let mut found = false;
    for seed in 0u64..2000 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)],
            vec![],
        );
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .unwrap();
        if genome.nodes[0].input_refs[0] == InputReference::ActionQueue {
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                assert_eq!(
                    gd.internal_nodes.len(),
                    12,
                    "compound Swap must create fan-out nodes"
                );
                found = true;
                break;
            }
        }
    }
    assert!(found, "must find a seed swapping to ActionQueue");
}

#[test]
fn raw_field_mutation_mutates_sub_idx_on_graph_input_ref() {
    // Create a genome with a graph InputRef node -- RawFieldMutation should
    // be able to mutate its sub_idx.
    let mut found_changed = false;
    for seed in 0u64..500 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::ActionQueue],
            vec![GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 5,
                },
                inputs: vec![],
                plasticity: None,
            }],
        );
        let mut r = rng(seed);
        let _ = InputRefMutator::apply(
            &mut genome,
            InputRefOperator::RawFieldMutation,
            &[],
            0.0,
            &mut r,
            &default_config(),
        );
        if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
            if let GraphNodeKind::InputRef { sub_idx, .. } = gd.internal_nodes[0].kind {
                if sub_idx != 5 {
                    found_changed = true;
                    break;
                }
            }
        }
    }
    assert!(
        found_changed,
        "RawFieldMutation must be able to mutate sub_idx on graph InputRef nodes"
    );
}

#[test]
fn extended_perception_families_reachable_in_pool() {
    let mut found = [false; 6]; // Food, Barrier, Occupancy, Core, Vitals, Identity
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        match random_input_reference(&mut r) {
            InputReference::World(WorldInputKey::AreaFoodSummary) => found[0] = true,
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
fn add_extended_perception_compound_creates_correct_fan_out() {
    // Verify that adding a NearbyCreatureCore input creates 16 fan-out nodes
    let mut found = false;
    for seed in 0u64..5000 {
        let mut genome = graph_genome_with_input_refs(vec![], vec![]);
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .unwrap();
        if genome.nodes[0].input_refs.last()
            == Some(&InputReference::World(WorldInputKey::NearbyCreatureCore))
        {
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                assert_eq!(
                    gd.internal_nodes.len(),
                    16,
                    "NearbyCreatureCore compound Add must create 16 fan-out nodes"
                );
                found = true;
                break;
            }
        }
    }
    assert!(found, "must find a seed producing NearbyCreatureCore");
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

// -- apply_add auto-connect integration tests --

#[test]
fn add_scalar_input_to_graph_creates_internal_node() {
    // With chance=0.0, adding any scalar input should create exactly one
    // InputRef internal node (previously only compound inputs created leaves).
    let config = MutationConfig {
        input_auto_connect_chance: 0.0,
        ..MutationConfig::default()
    };
    let mut found = false;
    for seed in 0u64..500 {
        let mut genome = graph_genome_with_input_refs(vec![], vec![]);
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .unwrap();
        let ref_added = &genome.nodes[0].input_refs[0];
        let count = compound::sub_value_count(ref_added, &config);
        if count == 1 {
            // Scalar input was added — should now have exactly 1 internal node
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                assert_eq!(
                    gd.internal_nodes.len(),
                    1,
                    "scalar Add must create 1 InputRef internal node"
                );
                assert!(matches!(
                    gd.internal_nodes[0].kind,
                    GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    }
                ));
                found = true;
                break;
            }
        }
    }
    assert!(found, "must find a seed producing a scalar input ref");
}

#[test]
fn add_input_auto_connects_with_chance_one() {
    // With chance=1.0 and pre-existing internal nodes, every new leaf should
    // get a bootstrap edge.
    let config = MutationConfig {
        input_auto_connect_chance: 1.0,
        ..MutationConfig::default()
    };
    let preexisting = vec![GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![],
        plasticity: None,
    }];
    let mut found = false;
    for seed in 0u64..500 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)],
            preexisting.clone(),
        );
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .unwrap();
        if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
            let new_leaves = gd.internal_nodes.len() - 1; // subtract pre-existing
            if new_leaves > 0 {
                // Check that each new leaf has a bootstrap edge from preexisting[0]
                let edges_from_new: usize = gd.internal_nodes[0]
                    .inputs
                    .iter()
                    .filter(|inp| inp.source_idx as usize >= 1)
                    .count();
                assert_eq!(
                    edges_from_new, new_leaves,
                    "every new leaf should have a bootstrap edge (chance=1.0)"
                );
                found = true;
                break;
            }
        }
    }
    assert!(found, "must find a seed that adds input and connects");
}

#[test]
fn add_scalar_input_increases_genome_size() {
    let config = MutationConfig {
        input_auto_connect_chance: 0.0,
        ..MutationConfig::default()
    };
    let mut found = false;
    for seed in 0u64..500 {
        let mut genome = graph_genome_with_input_refs(vec![], vec![]);
        let size_before = genome.genome_size();
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .unwrap();
        let ref_added = &genome.nodes[0].input_refs[0];
        let count = compound::sub_value_count(ref_added, &config);
        if count == 1 {
            let size_after = genome.genome_size();
            assert!(
                size_after > size_before,
                "genome_size must increase after scalar Add: before={}, after={}",
                size_before,
                size_after
            );
            found = true;
            break;
        }
    }
    assert!(found, "must find a seed producing a scalar input ref");
}

// -- Lifecycle cleanup tests ------------------------------------------------

#[test]
fn gc_orphaned_removes_u16max_nodes() {
    // Graph with 2 valid InputRef nodes + 1 orphaned (ref_idx = u16::MAX).
    // gc_orphaned_input_ref_nodes should remove only the orphaned one.
    let mut backend = BackendDef::Graph(GraphBackendDef {
        internal_nodes: vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: u16::MAX,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![],
                plasticity: None,
            },
        ],
    });
    let removed = gc_orphaned_input_ref_nodes(&mut backend);
    assert_eq!(removed, 1);
    if let BackendDef::Graph(ref g) = backend {
        assert_eq!(g.internal_nodes.len(), 2);
        // Valid InputRef still there
        assert!(matches!(
            g.internal_nodes[0].kind,
            GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0,
            }
        ));
        // Add node still there
        assert_eq!(g.internal_nodes[1].kind, GraphNodeKind::Add);
    }
}

#[test]
fn remove_input_ref_leaves_for_removes_matching_ref_idx() {
    // Graph with InputRef nodes for ref_idx 0 and ref_idx 1.
    // remove_input_ref_leaves_for(1) should only remove ref_idx=1 leaves.
    let mut backend = BackendDef::Graph(GraphBackendDef {
        internal_nodes: vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 1,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![],
                plasticity: None,
            },
        ],
    });
    let removed = remove_input_ref_leaves_for(&mut backend, 1);
    assert_eq!(removed, 2);
    if let BackendDef::Graph(ref g) = backend {
        assert_eq!(g.internal_nodes.len(), 2);
        assert!(matches!(
            g.internal_nodes[0].kind,
            GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0,
            }
        ));
        assert_eq!(g.internal_nodes[1].kind, GraphNodeKind::Add);
    }
}

#[test]
fn remove_input_ref_deletes_orphaned_leaves() {
    // After apply_remove, no InputRef nodes with ref_idx == u16::MAX should remain.
    let genome = graph_genome_with_input_refs(
        vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::UpstreamSlot(0),
        ],
        vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![],
                plasticity: None,
            },
        ],
    );
    // Run Remove over many seeds until we find one that removes an input_ref
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut g,
            InputRefOperator::Remove,
            &[],
            0.0,
            &mut r,
            &default_config(),
        )
        .is_ok()
        {
            // Check: no orphaned InputRef nodes remain
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                for node in &gd.internal_nodes {
                    if let GraphNodeKind::InputRef { ref_idx, .. } = node.kind {
                        assert_ne!(
                            ref_idx,
                            u16::MAX,
                            "orphaned InputRef node found after Remove (seed {})",
                            seed
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn swap_cleans_up_old_compound_leaves() {
    // Start with an 8-wide compound input, swap to scalar.
    // Old 8 leaves should be removed, 1 new leaf created.
    let config = MutationConfig {
        input_auto_connect_chance: 0.0,
        ..MutationConfig::default()
    };
    let mut found = false;
    for seed in 0u64..5000 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::NearbyCreatureVitals)], // 8-wide
            // Create 8 InputRef leaves for ref_idx 0
            (0..8)
                .map(|sub| GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: sub,
                    },
                    inputs: vec![],
                    plasticity: None,
                })
                .collect(),
        );
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .is_ok()
        {
            let new_ref = &genome.nodes[0].input_refs[0];
            let new_count = compound::sub_value_count(new_ref, &config);
            if new_count == 1 {
                // Swapped to a scalar: old 8 leaves removed, 1 new created
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    let input_ref_count = gd
                        .internal_nodes
                        .iter()
                        .filter(|n| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                        .count();
                    assert_eq!(
                        input_ref_count, 1,
                        "swap 8-wide→scalar: expected 1 leaf, got {} (seed {})",
                        input_ref_count, seed
                    );
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(found, "must find a seed swapping compound to scalar");
}

#[test]
fn swap_compound_to_narrower_removes_excess() {
    // Start with 8-wide, swap to 7-wide. Should have exactly 7 leaves.
    let config = MutationConfig {
        input_auto_connect_chance: 0.0,
        ..MutationConfig::default()
    };
    let mut found = false;
    for seed in 0u64..5000 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::NearbyCreatureVitals)], // 8-wide
            (0..8)
                .map(|sub| GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: sub,
                    },
                    inputs: vec![],
                    plasticity: None,
                })
                .collect(),
        );
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .is_ok()
        {
            let new_ref = &genome.nodes[0].input_refs[0];
            let new_count = compound::sub_value_count(new_ref, &config);
            if new_count == 7 {
                // Swapped to 7-wide (e.g., AreaFoodSummary)
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    let input_ref_count = gd
                        .internal_nodes
                        .iter()
                        .filter(|n| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                        .count();
                    assert_eq!(
                        input_ref_count, 7,
                        "swap 8-wide→7-wide: expected 7 leaves, got {} (seed {})",
                        input_ref_count, seed
                    );
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(found, "must find a seed swapping 8-wide to 7-wide");
}

#[test]
fn swap_creates_bootstrap_edges_with_chance_one() {
    // With input_auto_connect_chance = 1.0 and pre-existing nodes,
    // new leaves from swap should have bootstrap edges.
    let config = MutationConfig {
        input_auto_connect_chance: 1.0,
        ..MutationConfig::default()
    };
    let mut found = false;
    for seed in 0u64..5000 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)], // scalar
            vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        );
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .is_ok()
        {
            let new_ref = &genome.nodes[0].input_refs[0];
            let new_count = compound::sub_value_count(new_ref, &config);
            if new_count == 1 {
                // Scalar swap: old leaf removed, 1 new leaf created.
                // With chance=1.0, the new leaf should have a bootstrap edge
                // from a pre-existing node (the Add node).
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    let new_leaves: Vec<usize> = gd
                        .internal_nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                        .map(|(i, _)| i)
                        .collect();
                    if new_leaves.len() == 1 {
                        let leaf_idx = new_leaves[0] as u16;
                        // Check if any pre-existing node has an edge pointing to the new leaf
                        let has_bootstrap = gd
                            .internal_nodes
                            .iter()
                            .any(|n| n.inputs.iter().any(|e| e.source_idx == leaf_idx));
                        if has_bootstrap {
                            found = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    assert!(
        found,
        "swap with chance=1.0 must create bootstrap edges to new leaves"
    );
}

#[test]
fn swap_no_bootstrap_edges_with_chance_zero() {
    // With input_auto_connect_chance = 0.0, new leaves from swap should
    // NOT have bootstrap edges from pre-existing nodes.
    let config = MutationConfig {
        input_auto_connect_chance: 0.0,
        ..MutationConfig::default()
    };
    for seed in 0u64..500 {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)], // scalar
            vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        );
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Swap,
            &[],
            0.0,
            &mut r,
            &config,
        )
        .is_ok()
        {
            // With chance=0.0, no pre-existing node should gain new edges
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                // Add node (which is non-InputRef) should still have 0 inputs
                for n in &gd.internal_nodes {
                    if n.kind == GraphNodeKind::Add {
                        assert!(
                            n.inputs.is_empty(),
                            "Add node should have no edges with chance=0.0 (seed {})",
                            seed
                        );
                    }
                }
            }
        }
    }
}

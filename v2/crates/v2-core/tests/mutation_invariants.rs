use v2_core::evolution::{
    MutationConfig, MutationOperatorKind, apply_operator, validate_mutation_invariants,
};
use v2_core::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    InputReference, InternalTargetDef, NodeGenome, NodeType, OutputDefinition, PayloadField,
    VmBackendDef, VmInstruction, WorldActionDef, WorldActionKind,
};

fn baseline_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: 1,
        nodes: vec![
            NodeGenome {
                node_id: 1,
                node_type: NodeType::Graph,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    operator: GraphOperator::WeightedSum,
                    inputs: vec![InputReference::World(
                        v2_core::mesh::WorldInputKey::FoodHere,
                    )],
                    coefficients: vec![1.0],
                    bias: 0.0,
                    state_slot_count: 0,
                }),
                output_definitions: vec![
                    OutputDefinition::InternalTarget(InternalTargetDef {
                        target_node_id: 2,
                        input_refs: vec![InputReference::Packet("signal".to_string())],
                        payload_fields: vec![PayloadField::Scalar {
                            key: "signal".to_string(),
                            value: 1,
                        }],
                    }),
                    OutputDefinition::WorldAction(WorldActionDef {
                        action_kind: WorldActionKind::Move,
                        action_metadata_fields: vec![ActionMetadataField::Direction(2)],
                    }),
                ],
                local_state_init: Vec::new(),
            },
            NodeGenome {
                node_id: 2,
                node_type: NodeType::Vm,
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                    constants: vec![1.0],
                    max_input_slots: 8,
                }),
                output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                    target_node_id: 1,
                    input_refs: vec![InputReference::Packet("feedback".to_string())],
                    payload_fields: vec![PayloadField::Scalar {
                        key: "feedback".to_string(),
                        value: 1,
                    }],
                })],
                local_state_init: vec![0, 1],
            },
            NodeGenome {
                node_id: 3,
                node_type: NodeType::Graph,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    operator: GraphOperator::Passthrough,
                    inputs: Vec::new(),
                    coefficients: Vec::new(),
                    bias: 0.0,
                    state_slot_count: 0,
                }),
                output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                    target_node_id: 1,
                    input_refs: Vec::new(),
                    payload_fields: Vec::new(),
                })],
                local_state_init: Vec::new(),
            },
        ],
        evolution_params: None,
    }
}

#[test]
fn mutation_defaults_match_cp2_spec() {
    let config = MutationConfig::default();

    assert_eq!(config.per_birth_mutation_events_min, 2);
    assert_eq!(config.per_birth_mutation_events_max, 8);
    assert_eq!(config.max_nodes, 64);
    assert_eq!(config.min_nodes, 1);
    assert_eq!(config.max_outputs_per_node, 8);
    assert_eq!(config.max_subgraph_duplication_nodes, 6);
    assert_eq!(config.max_vm_program_len, 128);

    assert!((config.node_duplication_clone_outputs_probability - 1.0).abs() < f32::EPSILON);
    assert!((config.node_duplication_target_remap_probability - 0.35).abs() < f32::EPSILON);
    assert!((config.subgraph_external_edge_retarget_probability - 0.0).abs() < f32::EPSILON);

    assert!((config.weights.add_node - 0.10).abs() < f32::EPSILON);
    assert!((config.weights.remove_node - 0.06).abs() < f32::EPSILON);
    assert!((config.weights.retarget_node - 0.06).abs() < f32::EPSILON);
    assert!((config.weights.add_output - 0.10).abs() < f32::EPSILON);
    assert!((config.weights.remove_output - 0.08).abs() < f32::EPSILON);
    assert!((config.weights.retarget_output - 0.12).abs() < f32::EPSILON);
    assert!((config.weights.graph_local_mutation - 0.12).abs() < f32::EPSILON);
    assert!((config.weights.vm_instruction_mutation - 0.20).abs() < f32::EPSILON);
    assert!((config.weights.node_duplication - 0.10).abs() < f32::EPSILON);
    assert!((config.weights.subgraph_duplication - 0.06).abs() < f32::EPSILON);
    assert!((config.weights.total() - 1.0).abs() < 1e-6);
    assert!(config.validate().is_ok());
}

#[test]
fn add_node_operator_adds_unique_node_and_preserves_invariants() {
    let mut genome = baseline_genome();
    let config = MutationConfig::default();
    let existing_ids = genome.nodes.iter().map(|node| node.node_id).collect::<Vec<_>>();

    let changed = apply_operator(&mut genome, &config, MutationOperatorKind::AddNode, 17);
    assert!(changed);
    assert_eq!(genome.nodes.len(), existing_ids.len() + 1);

    let new_id = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .find(|id| !existing_ids.contains(id))
        .expect("new node id should exist");
    assert!(!existing_ids.contains(&new_id));

    validate_mutation_invariants(&genome, &config).expect("add node mutation should stay valid");
}

#[test]
fn remove_node_operator_retargets_removed_internal_targets() {
    let mut genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![
            NodeGenome {
                node_id: 1,
                node_type: NodeType::Graph,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    operator: GraphOperator::Passthrough,
                    inputs: Vec::new(),
                    coefficients: Vec::new(),
                    bias: 0.0,
                    state_slot_count: 0,
                }),
                output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                    target_node_id: 2,
                    input_refs: Vec::new(),
                    payload_fields: Vec::new(),
                })],
                local_state_init: Vec::new(),
            },
            NodeGenome {
                node_id: 2,
                node_type: NodeType::Graph,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    operator: GraphOperator::Passthrough,
                    inputs: Vec::new(),
                    coefficients: Vec::new(),
                    bias: 0.0,
                    state_slot_count: 0,
                }),
                output_definitions: Vec::new(),
                local_state_init: Vec::new(),
            },
        ],
        evolution_params: None,
    };
    let config = MutationConfig::default();

    let changed = apply_operator(&mut genome, &config, MutationOperatorKind::RemoveNode, 3);
    assert!(changed);
    assert_eq!(genome.nodes.len(), 1);
    assert_eq!(genome.entry_node_id, 1);

    let node = genome.nodes.first().expect("remaining node");
    let OutputDefinition::InternalTarget(target) = node
        .output_definitions
        .first()
        .expect("retargeted internal output")
    else {
        panic!("expected internal target output");
    };
    assert_eq!(target.target_node_id, 1);

    validate_mutation_invariants(&genome, &config)
        .expect("remove node mutation should stay valid");
}

#[test]
fn retarget_node_operator_retargets_to_existing_node() {
    let mut genome = baseline_genome();
    let config = MutationConfig::default();
    let before_entry = genome.entry_node_id;
    let before_targets = genome
        .nodes
        .iter()
        .flat_map(|node| node.output_definitions.iter())
        .filter_map(|output| match output {
            OutputDefinition::InternalTarget(target) => Some(target.target_node_id),
            OutputDefinition::WorldAction(_) => None,
        })
        .collect::<Vec<_>>();

    let changed = apply_operator(&mut genome, &config, MutationOperatorKind::RetargetNode, 29);
    assert!(changed);

    let node_ids = genome.nodes.iter().map(|node| node.node_id).collect::<Vec<_>>();
    assert!(node_ids.contains(&genome.entry_node_id));
    let after_targets = genome
        .nodes
        .iter()
        .flat_map(|node| node.output_definitions.iter())
        .filter_map(|output| match output {
            OutputDefinition::InternalTarget(target) => Some(target.target_node_id),
            OutputDefinition::WorldAction(_) => None,
        })
        .collect::<Vec<_>>();
    assert!(after_targets.iter().all(|target| node_ids.contains(target)));
    assert!(
        genome.entry_node_id != before_entry || after_targets != before_targets,
        "retarget node should change entry or at least one internal target"
    );

    validate_mutation_invariants(&genome, &config)
        .expect("retarget node mutation should stay valid");
}

#[test]
fn required_operators_preserve_structural_invariants() {
    let mut genome = baseline_genome();
    let config = MutationConfig::default();

    for (index, operator) in MutationOperatorKind::all().into_iter().enumerate() {
        let changed = apply_operator(
            &mut genome,
            &config,
            operator,
            u64::try_from(index).expect("index fits in u64") + 13,
        );
        assert!(changed, "operator {:?} made no mutation", operator);
        validate_mutation_invariants(&genome, &config).expect("mutated genome should stay valid");
    }
}

#[test]
fn node_duplication_clones_outputs_by_default() {
    let mut genome = baseline_genome();
    let config = MutationConfig::default();

    let source_output_count = genome
        .nodes
        .iter()
        .find(|node| node.node_id == 1)
        .expect("source node")
        .output_definitions
        .len();

    let before = genome.nodes.len();
    let changed = apply_operator(
        &mut genome,
        &config,
        MutationOperatorKind::NodeDuplication,
        1,
    );
    assert!(changed);
    assert!(genome.nodes.len() > before);

    let duplicated = genome
        .nodes
        .iter()
        .find(|node| node.node_id > 3)
        .expect("duplicated node exists");
    assert_eq!(duplicated.output_definitions.len(), source_output_count);

    validate_mutation_invariants(&genome, &config).expect("duplicated genome remains valid");
}

#[test]
fn node_duplication_can_remap_self_target_to_duplicate() {
    let mut genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Graph,
            backend_def: BackendDef::Graph(GraphBackendDef {
                operator: GraphOperator::Passthrough,
                inputs: Vec::new(),
                coefficients: Vec::new(),
                bias: 0.0,
                state_slot_count: 0,
            }),
            output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                target_node_id: 1,
                input_refs: Vec::new(),
                payload_fields: Vec::new(),
            })],
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };
    let mut config = MutationConfig::default();
    config.node_duplication_clone_outputs_probability = 1.0;
    config.node_duplication_target_remap_probability = 1.0;

    let changed = apply_operator(
        &mut genome,
        &config,
        MutationOperatorKind::NodeDuplication,
        11,
    );
    assert!(changed);
    assert_eq!(genome.nodes.len(), 2);

    let duplicated = genome
        .nodes
        .iter()
        .find(|node| node.node_id != 1)
        .expect("duplicated node should exist");
    let OutputDefinition::InternalTarget(target) = duplicated
        .output_definitions
        .first()
        .expect("duplicated output")
    else {
        panic!("expected duplicated internal target");
    };
    assert_eq!(target.target_node_id, duplicated.node_id);

    validate_mutation_invariants(&genome, &config).expect("duplicated genome remains valid");
}

#[test]
fn subgraph_duplication_preserves_external_edges_by_default() {
    let mut genome = baseline_genome();
    let config = MutationConfig::default();

    let external_before = genome
        .nodes
        .iter()
        .find(|node| node.node_id == 3)
        .and_then(|node| node.output_definitions.first())
        .expect("external edge")
        .clone();

    let changed = apply_operator(
        &mut genome,
        &config,
        MutationOperatorKind::SubgraphDuplication,
        2,
    );
    assert!(changed);

    let external_after = genome
        .nodes
        .iter()
        .find(|node| node.node_id == 3)
        .and_then(|node| node.output_definitions.first())
        .expect("external edge after")
        .clone();

    assert_eq!(external_before, external_after);
    validate_mutation_invariants(&genome, &config).expect("subgraph duplication remains valid");
}

#[test]
fn subgraph_duplication_respects_budget_and_remaps_internal_edges() {
    let mut genome = baseline_genome();
    let mut config = MutationConfig::default();
    config.max_subgraph_duplication_nodes = 2;

    let before_len = genome.nodes.len();
    let changed = apply_operator(
        &mut genome,
        &config,
        MutationOperatorKind::SubgraphDuplication,
        7,
    );
    assert!(changed);
    assert_eq!(genome.nodes.len(), before_len + 2);

    let clone_entry = genome
        .nodes
        .iter()
        .find(|node| node.node_id == 4)
        .expect("entry clone should exist");
    let clone_second = genome
        .nodes
        .iter()
        .find(|node| node.node_id == 5)
        .expect("second clone should exist");

    let entry_has_remapped_target = clone_entry.output_definitions.iter().any(|output| {
        matches!(
            output,
            OutputDefinition::InternalTarget(InternalTargetDef { target_node_id: 5, .. })
        )
    });
    let second_has_remapped_target = clone_second.output_definitions.iter().any(|output| {
        matches!(
            output,
            OutputDefinition::InternalTarget(InternalTargetDef { target_node_id: 4, .. })
        )
    });
    assert!(entry_has_remapped_target);
    assert!(second_has_remapped_target);

    validate_mutation_invariants(&genome, &config)
        .expect("bounded subgraph duplication remains valid");
}

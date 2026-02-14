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

    assert!((config.weights.total() - 1.0).abs() < 1e-6);
    assert!(config.validate().is_ok());
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

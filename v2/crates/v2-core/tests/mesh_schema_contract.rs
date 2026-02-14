use v2_core::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    InputReference, InternalTargetDef, NodeGenome, NodeType, OutputDefinition, VmBackendDef,
    VmInstruction, WorldActionDef, WorldActionKind,
};

fn graph_node(id: u32) -> NodeGenome {
    NodeGenome {
        node_id: id,
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
    }
}

#[test]
fn backend_definition_must_match_node_type() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Graph,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![VmInstruction::Noop],
                constants: Vec::new(),
                max_input_slots: 4,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    assert!(genome.validate().is_err());
}

#[test]
fn duplicate_payload_keys_are_rejected() {
    let mut node = graph_node(1);
    node.output_definitions = vec![OutputDefinition::InternalTarget(InternalTargetDef {
        target_node_id: 1,
        input_refs: Vec::new(),
        payload_fields: vec![
            v2_core::mesh::PayloadField::Scalar {
                key: "x".to_string(),
                value: 1,
            },
            v2_core::mesh::PayloadField::Scalar {
                key: "x".to_string(),
                value: 2,
            },
        ],
    })];

    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![node],
        evolution_params: None,
    };
    assert!(genome.validate().is_err());
}

#[test]
fn world_action_direction_range_is_validated() {
    let mut node = graph_node(1);
    node.output_definitions = vec![OutputDefinition::WorldAction(WorldActionDef {
        action_kind: WorldActionKind::Move,
        action_metadata_fields: vec![ActionMetadataField::Direction(8)],
    })];
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![node],
        evolution_params: None,
    };
    assert!(genome.validate().is_err());
}

#[test]
fn vm_target_input_refs_respect_max_input_slots() {
    let source = NodeGenome {
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
            input_refs: vec![
                InputReference::Packet("a".to_string()),
                InputReference::Packet("b".to_string()),
                InputReference::Packet("c".to_string()),
            ],
            payload_fields: Vec::new(),
        })],
        local_state_init: Vec::new(),
    };
    let vm = NodeGenome {
        node_id: 2,
        node_type: NodeType::Vm,
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 2,
            program: vec![VmInstruction::Noop],
            constants: Vec::new(),
            max_input_slots: 2,
        }),
        output_definitions: Vec::new(),
        local_state_init: Vec::new(),
    };
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![source, vm],
        evolution_params: None,
    };
    assert!(genome.validate().is_err());
}

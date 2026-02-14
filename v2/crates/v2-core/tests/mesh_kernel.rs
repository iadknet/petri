use v2_core::mesh::{
    ActionMetadataField, CreatureGenome, EmittedOutput, ExecutionOutcome, InputReference,
    InternalTargetDef, NodeGenome, NodeType, OutputDefinition, WorldActionDef, WorldActionKind,
    run_mesh_queue,
};

fn minimal_node(id: u32) -> NodeGenome {
    NodeGenome {
        node_id: id,
        node_type: NodeType::Graph,
        output_definitions: vec![],
        local_state_init: vec![],
    }
}

#[test]
fn genome_validation_rejects_missing_entry_and_invalid_targets() {
    let missing_entry = CreatureGenome {
        entry_node_id: 99,
        nodes: vec![minimal_node(1)],
        evolution_params: None,
    };
    assert!(missing_entry.validate().is_err());

    let invalid_internal_target = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Graph,
            output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                target_node_id: 42,
                input_refs: vec![InputReference::World("food_here".to_string())],
                payload_fields: vec![],
            })],
            local_state_init: vec![],
        }],
        evolution_params: None,
    };
    assert!(invalid_internal_target.validate().is_err());

    let invalid_world_action = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Graph,
            output_definitions: vec![OutputDefinition::WorldAction(WorldActionDef {
                action_kind: WorldActionKind::Eat,
                action_metadata_fields: vec![ActionMetadataField::Direction(1)],
            })],
            local_state_init: vec![],
        }],
        evolution_params: None,
    };
    assert!(invalid_world_action.validate().is_err());
}

#[test]
fn routing_is_fifo_and_first_world_action_halts() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![
            minimal_node(1),
            minimal_node(2),
            minimal_node(3),
            minimal_node(4),
        ],
        evolution_params: None,
    };

    let mut dispatch_order = Vec::new();
    let outcome = run_mesh_queue(&genome, |input| {
        dispatch_order.push(input.target_node_id);
        match input.target_node_id {
            1 => vec![
                EmittedOutput::InternalTarget(InternalTargetDef {
                    target_node_id: 2,
                    input_refs: vec![],
                    payload_fields: vec![],
                }),
                EmittedOutput::InternalTarget(InternalTargetDef {
                    target_node_id: 3,
                    input_refs: vec![],
                    payload_fields: vec![],
                }),
            ],
            2 => vec![EmittedOutput::InternalTarget(InternalTargetDef {
                target_node_id: 4,
                input_refs: vec![],
                payload_fields: vec![],
            })],
            3 => vec![EmittedOutput::WorldAction(WorldActionDef {
                action_kind: WorldActionKind::Move,
                action_metadata_fields: vec![ActionMetadataField::Direction(2)],
            })],
            4 => panic!("node 4 should not dispatch after world-action commit"),
            _ => unreachable!(),
        }
    })
    .expect("queue execution should succeed");

    assert_eq!(dispatch_order, vec![1, 2, 3]);
    match outcome {
        ExecutionOutcome::WorldActionCommitted(action) => {
            assert_eq!(action.action_kind, WorldActionKind::Move)
        }
        _ => panic!("expected world-action commit outcome"),
    }
}

#[test]
fn queue_drain_results_in_implicit_noop() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![minimal_node(1)],
        evolution_params: None,
    };

    let outcome = run_mesh_queue(&genome, |_input| Vec::new()).expect("queue execution succeeds");
    assert_eq!(outcome, ExecutionOutcome::ImplicitNoOp);
}

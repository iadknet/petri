use v2_core::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    InputReference, InternalTargetDef, NodeGenome, NodeType, OutputDefinition, VmBackendDef,
    VmInstruction, WorldActionDef, WorldActionKind,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeConfigError, RuntimeContext, RuntimeError,
    RuntimeOutcome, SensorFrame, run_runtime_tick,
};

fn runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        dispatch_entry_cost: 0.5,
        graph_base_tariff: 0.25,
        vm_opcode_cost_multiplier: 1.0,
        sensor_radius: 2,
        action_costs: RuntimeActionCosts {
            move_cost: 0.4,
            eat_cost: 0.4,
            reproduce_cost: 0.6,
            inventory_pickup_cost: 0.5,
            inventory_put_cost: 0.5,
            noop_cost: 0.05,
        },
    }
}

fn runtime_context(energy_before_tick: f32) -> RuntimeContext {
    RuntimeContext {
        config: runtime_config(),
        energy_before_tick,
        memory_bytes: [0; 1024],
        graph_state_slots: vec![0.0; 4],
        sensor_frame: SensorFrame::empty(2),
    }
}

fn graph_node_with_outputs(node_id: u32, outputs: Vec<OutputDefinition>) -> NodeGenome {
    NodeGenome {
        node_id,
        node_type: NodeType::Graph,
        backend_def: BackendDef::Graph(GraphBackendDef {
            operator: GraphOperator::Passthrough,
            inputs: Vec::new(),
            coefficients: Vec::new(),
            bias: 0.0,
            state_slot_count: 0,
        }),
        output_definitions: outputs,
        local_state_init: Vec::new(),
    }
}

#[test]
fn dispatch_action_energy_order_and_commit() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![graph_node_with_outputs(
            1,
            vec![OutputDefinition::WorldAction(WorldActionDef {
                action_kind: WorldActionKind::Move,
                action_metadata_fields: vec![ActionMetadataField::Direction(2)],
            })],
        )],
        evolution_params: None,
    };

    let mut context = runtime_context(2.0);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::CommittedAction {
            energy_spent,
            energy_remaining,
            dispatches,
            ..
        } => {
            assert_eq!(dispatches, 1);
            assert!(energy_spent > 0.0);
            assert!(energy_remaining < 2.0);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn queue_drain_produces_implicit_noop() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![graph_node_with_outputs(1, Vec::new())],
        evolution_params: None,
    };
    let mut context = runtime_context(2.0);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::ImplicitNoOp {
            energy_spent,
            energy_remaining,
            dispatches,
        } => {
            assert_eq!(dispatches, 1);
            assert!(energy_spent > 0.0);
            assert!(energy_remaining > 0.0 && energy_remaining < 2.0);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn vm_exhaustion_returns_energy_exhausted() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![VmInstruction::Noop; 10],
                constants: Vec::new(),
                max_input_slots: 4,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = runtime_context(0.55);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::EnergyExhausted {
            energy_spent,
            dispatches,
        } => {
            assert_eq!(dispatches, 1);
            assert!(energy_spent > context.config.dispatch_entry_cost);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn dispatch_entry_exhaustion_returns_zero_dispatches() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![graph_node_with_outputs(1, Vec::new())],
        evolution_params: None,
    };

    let mut context = runtime_context(0.1);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::EnergyExhausted {
            energy_spent,
            dispatches,
        } => {
            assert_eq!(dispatches, 0);
            assert!((energy_spent - 0.1).abs() < 1e-6);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn invalid_world_action_metadata_maps_to_runtime_error() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        output_index: 0,
                        metadata_field_index: 0,
                        src: 0,
                    },
                    VmInstruction::EmitWorldAction { output_index: 0 },
                ],
                constants: vec![999.0],
                max_input_slots: 4,
            }),
            output_definitions: vec![OutputDefinition::WorldAction(WorldActionDef {
                action_kind: WorldActionKind::Move,
                action_metadata_fields: vec![ActionMetadataField::Direction(1)],
            })],
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };
    let mut context = runtime_context(10.0);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::RuntimeError { error, .. } => {
            assert!(matches!(error, RuntimeError::InvalidActionMetadata(_)));
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn sensor_radius_zero_runtime_config_is_rejected() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![graph_node_with_outputs(1, Vec::new())],
        evolution_params: None,
    };
    let mut context = runtime_context(1.0);
    context.config.sensor_radius = 0;
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::RuntimeError { error, .. } => {
            assert!(matches!(
                error,
                RuntimeError::InvalidConfig(RuntimeConfigError::SensorRadiusZero)
            ));
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn emitted_output_order_stops_on_first_invalid_world_action() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        output_index: 0,
                        metadata_field_index: 0,
                        src: 0,
                    },
                    VmInstruction::EmitWorldAction { output_index: 0 },
                    VmInstruction::EmitWorldAction { output_index: 1 },
                ],
                constants: vec![999.0],
                max_input_slots: 4,
            }),
            output_definitions: vec![
                OutputDefinition::WorldAction(WorldActionDef {
                    action_kind: WorldActionKind::Move,
                    action_metadata_fields: vec![ActionMetadataField::Direction(1)],
                }),
                OutputDefinition::WorldAction(WorldActionDef {
                    action_kind: WorldActionKind::Move,
                    action_metadata_fields: vec![ActionMetadataField::Direction(2)],
                }),
            ],
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = runtime_context(10.0);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::RuntimeError { error, .. } => {
            assert!(matches!(error, RuntimeError::InvalidActionMetadata(_)));
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn first_valid_world_action_commits_and_halts_before_queued_dispatch() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![
            graph_node_with_outputs(
                1,
                vec![
                    OutputDefinition::InternalTarget(InternalTargetDef {
                        target_node_id: 2,
                        input_refs: vec![InputReference::Packet("queued".to_string())],
                        payload_fields: Vec::new(),
                    }),
                    OutputDefinition::WorldAction(WorldActionDef {
                        action_kind: WorldActionKind::Move,
                        action_metadata_fields: vec![ActionMetadataField::Direction(3)],
                    }),
                ],
            ),
            graph_node_with_outputs(
                2,
                vec![OutputDefinition::WorldAction(WorldActionDef {
                    action_kind: WorldActionKind::Move,
                    action_metadata_fields: vec![ActionMetadataField::Direction(1)],
                })],
            ),
        ],
        evolution_params: None,
    };

    let mut context = runtime_context(10.0);
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::CommittedAction {
            action, dispatches, ..
        } => {
            assert_eq!(dispatches, 1);
            assert_eq!(action.action_kind, WorldActionKind::Move);
            assert_eq!(
                action.action_metadata_fields,
                vec![ActionMetadataField::Direction(3)]
            );
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

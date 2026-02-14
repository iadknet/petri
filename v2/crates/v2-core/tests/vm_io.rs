use v2_core::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, NodeGenome, NodeType, OutputDefinition,
    VmBackendDef, VmInstruction, WorldActionDef, WorldActionKind,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeContext, RuntimeOutcome, SensorFrame,
    run_runtime_tick,
};

fn context() -> RuntimeContext {
    RuntimeContext {
        config: RuntimeConfig {
            dispatch_entry_cost: 0.01,
            graph_base_tariff: 0.1,
            vm_opcode_cost_multiplier: 1.0,
            sensor_radius: 2,
            action_costs: RuntimeActionCosts {
                move_cost: 0.1,
                eat_cost: 0.1,
                reproduce_cost: 0.1,
                inventory_pickup_cost: 0.1,
                inventory_put_cost: 0.1,
                noop_cost: 0.01,
            },
        },
        energy_before_tick: 10.0,
        memory_bytes: [0; 1024],
        graph_state_slots: vec![0.0; 2],
        sensor_frame: SensorFrame::empty(2),
    }
}

#[test]
fn vm_can_override_world_action_metadata_before_emit() {
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
                constants: vec![3.0],
                max_input_slots: 2,
            }),
            output_definitions: vec![OutputDefinition::WorldAction(WorldActionDef {
                action_kind: WorldActionKind::Move,
                action_metadata_fields: vec![ActionMetadataField::Direction(1)],
            })],
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = context();
    let outcome = run_runtime_tick(&genome, &mut context);
    match outcome {
        RuntimeOutcome::CommittedAction { action, .. } => {
            assert_eq!(action.action_kind, WorldActionKind::Move);
            assert_eq!(
                action.action_metadata_fields,
                vec![ActionMetadataField::Direction(3)]
            );
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

use v2_core::mesh::{
    BackendDef, CreatureGenome, NodeGenome, NodeType, SensorCellField, SensorCreatureField,
    SensorSummaryField, VmBackendDef, VmInstruction,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeContext, RuntimeOutcome, SensorCellSnapshot,
    SensorCreatureSnapshot, SensorFrame, run_runtime_tick,
};

#[test]
fn vm_read_sensor_opcodes_access_sensor_frame() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        1,
        0,
        SensorCellSnapshot {
            food_density_u8: 255,
            barrier_flag: false,
            occupied_flag: true,
            is_self: false,
            creature: Some(SensorCreatureSnapshot {
                phenotype_rgb: [10, 20, 30],
                energy: 8.0,
                age_ticks: 100,
                generation: 2,
            }),
        },
    );

    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                program: vec![
                    VmInstruction::ReadSensorCell {
                        dst: 0,
                        dx: 1,
                        dy: 0,
                        field: SensorCellField::FoodDensityNorm,
                    },
                    VmInstruction::ReadSensorCreature {
                        dst: 1,
                        dx: 1,
                        dy: 0,
                        field: SensorCreatureField::EnergyNorm,
                    },
                    VmInstruction::ReadSensorSummary {
                        dst: 2,
                        field: SensorSummaryField::VisibleCreatureCountNorm,
                    },
                    VmInstruction::Halt,
                ],
                constants: Vec::new(),
                max_input_slots: 4,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = RuntimeContext {
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
        sensor_frame: frame,
    };

    let outcome = run_runtime_tick(&genome, &mut context);
    assert!(matches!(outcome, RuntimeOutcome::ImplicitNoOp { .. }));
}

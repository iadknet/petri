use v2_core::mesh::{
    BackendDef, CreatureGenome, NeighborCellField, NeighborCreatureField, NeighborDirection,
    NodeGenome, NodeType, VmBackendDef, VmInstruction,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeContext, RuntimeOutcome, SensorCellSnapshot,
    SensorCreatureSnapshot, SensorFrame, run_runtime_tick,
};

#[test]
fn vm_neighbor_queries_map_to_expected_offsets() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        0,
        -1,
        SensorCellSnapshot {
            food_density_u8: 200,
            barrier_flag: false,
            occupied_flag: true,
            is_self: false,
            creature: Some(SensorCreatureSnapshot {
                phenotype_rgb: [10, 20, 30],
                energy: 5.0,
                age_ticks: 10,
                generation: 1,
            }),
        },
    );

    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 3,
                program: vec![
                    VmInstruction::ReadNeighborCell {
                        dst: 0,
                        direction: NeighborDirection::North,
                        field: NeighborCellField::FoodDensityNorm,
                    },
                    VmInstruction::ReadNeighborCreature {
                        dst: 1,
                        direction: NeighborDirection::North,
                        field: NeighborCreatureField::PresentFlag,
                    },
                    VmInstruction::Halt,
                ],
                constants: Vec::new(),
                max_input_slots: 3,
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

use v2_core::mesh::{
    BackendDef, CreatureGenome, NodeGenome, NodeType, VmBackendDef, VmInstruction,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeContext, RuntimeOutcome, SensorFrame,
    run_runtime_tick,
};

fn config() -> RuntimeConfig {
    RuntimeConfig {
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
    }
}

fn context() -> RuntimeContext {
    RuntimeContext {
        config: config(),
        energy_before_tick: 10.0,
        memory_bytes: [0; 1024],
        graph_state_slots: vec![0.0; 2],
        sensor_frame: SensorFrame::empty(2),
    }
}

#[test]
fn vm_arithmetic_and_halt_execute() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Add { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
                constants: vec![2.0, 3.0],
                max_input_slots: 4,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = context();
    let outcome = run_runtime_tick(&genome, &mut context);
    assert!(matches!(outcome, RuntimeOutcome::ImplicitNoOp { .. }));
}

#[test]
fn vm_jump_and_loop_are_energy_bounded() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![VmInstruction::Noop, VmInstruction::Jump { offset: -1 }],
                constants: Vec::new(),
                max_input_slots: 2,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = context();
    context.energy_before_tick = 0.3;
    let outcome = run_runtime_tick(&genome, &mut context);
    assert!(matches!(outcome, RuntimeOutcome::EnergyExhausted { .. }));
}

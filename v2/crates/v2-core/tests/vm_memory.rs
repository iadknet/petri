use v2_core::mesh::{
    BackendDef, CreatureGenome, NodeGenome, NodeType, VmBackendDef, VmInstruction,
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
fn memory_store_and_load_with_address_wrapping() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Vm,
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 3,
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::StoreMem8 {
                        addr_reg: 0,
                        src: 1,
                    },
                    VmInstruction::LoadMem8 {
                        dst: 2,
                        addr_reg: 0,
                    },
                    VmInstruction::Halt,
                ],
                constants: vec![1024.0, 42.0],
                max_input_slots: 2,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = context();
    let outcome = run_runtime_tick(&genome, &mut context);
    assert!(matches!(outcome, RuntimeOutcome::ImplicitNoOp { .. }));
    assert_eq!(context.memory_bytes[0], 42);
}

#[test]
fn immediate_memory_ops_use_fixed_addresses() {
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
                    VmInstruction::StoreMem8Imm { addr: 7, src: 0 },
                    VmInstruction::LoadMem8Imm { dst: 1, addr: 7 },
                    VmInstruction::Halt,
                ],
                constants: vec![255.0],
                max_input_slots: 2,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = context();
    let outcome = run_runtime_tick(&genome, &mut context);
    assert!(matches!(outcome, RuntimeOutcome::ImplicitNoOp { .. }));
    assert_eq!(context.memory_bytes[7], 255);
}

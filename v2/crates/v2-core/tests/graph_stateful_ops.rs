use v2_core::backends::evaluate_graph_operator;
use v2_core::mesh::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphOperator, InputReference, NodeGenome,
    NodeType,
};
use v2_core::runtime::{
    RuntimeActionCosts, RuntimeConfig, RuntimeContext, RuntimeOutcome, SensorFrame,
    run_runtime_tick,
};

#[test]
fn stateful_graph_ops_persist_and_update_state_slots() {
    let graph = GraphBackendDef {
        operator: GraphOperator::DecayIntegrator {
            state_slot: 0,
            alpha: 0.5,
        },
        inputs: Vec::new(),
        coefficients: Vec::new(),
        bias: 1.0,
        state_slot_count: 1,
    };
    let mut state = vec![0.0];
    let v1 = evaluate_graph_operator(&graph, &[1.0], &mut state);
    let v2 = evaluate_graph_operator(&graph, &[1.0], &mut state);
    assert!(v2 >= v1);
}

#[test]
fn runtime_graph_state_slots_persist_across_ticks() {
    let genome = CreatureGenome {
        entry_node_id: 1,
        nodes: vec![NodeGenome {
            node_id: 1,
            node_type: NodeType::Graph,
            backend_def: BackendDef::Graph(GraphBackendDef {
                operator: GraphOperator::Oscillator {
                    phase_slot: 0,
                    frequency: 0.25,
                    amplitude: 1.0,
                    bias: 0.0,
                },
                inputs: vec![InputReference::World(v2_core::mesh::WorldInputKey::FoodHere)],
                coefficients: Vec::new(),
                bias: 0.0,
                state_slot_count: 1,
            }),
            output_definitions: Vec::new(),
            local_state_init: Vec::new(),
        }],
        evolution_params: None,
    };

    let mut context = RuntimeContext {
        config: RuntimeConfig {
            dispatch_entry_cost: 0.1,
            graph_base_tariff: 0.1,
            vm_opcode_cost_multiplier: 1.0,
            sensor_radius: 1,
            action_costs: RuntimeActionCosts {
                move_cost: 0.1,
                eat_cost: 0.1,
                reproduce_cost: 0.1,
                inventory_pickup_cost: 0.1,
                inventory_put_cost: 0.1,
                noop_cost: 0.05,
            },
        },
        energy_before_tick: 1.0,
        memory_bytes: [0; 1024],
        graph_state_slots: vec![0.0],
        sensor_frame: SensorFrame::empty(1),
    };

    let first = run_runtime_tick(&genome, &mut context);
    assert!(matches!(first, RuntimeOutcome::ImplicitNoOp { .. }));
    let phase_after_first = context.graph_state_slots[0];

    let second = run_runtime_tick(&genome, &mut context);
    assert!(matches!(second, RuntimeOutcome::ImplicitNoOp { .. }));
    let phase_after_second = context.graph_state_slots[0];

    assert!(phase_after_first > 0.0);
    assert!(phase_after_second > phase_after_first);
}

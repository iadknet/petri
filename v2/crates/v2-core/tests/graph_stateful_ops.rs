use v2_core::backends::evaluate_graph_operator;
use v2_core::mesh::{GraphBackendDef, GraphOperator};

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

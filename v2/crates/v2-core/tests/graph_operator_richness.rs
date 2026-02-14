use v2_core::backends::graph_operator_cost_multiplier;
use v2_core::mesh::GraphOperator;

#[test]
fn operator_cost_multipliers_are_defined_for_rich_set() {
    assert!(graph_operator_cost_multiplier(GraphOperator::Passthrough) > 0.0);
    assert!(
        graph_operator_cost_multiplier(GraphOperator::Oscillator {
            phase_slot: 0,
            frequency: 1.0,
            amplitude: 1.0,
            bias: 0.0,
        }) > graph_operator_cost_multiplier(GraphOperator::Passthrough)
    );
}

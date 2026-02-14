use v2_core::backends::evaluate_graph_operator;
use v2_core::mesh::{
    GraphBackendDef, GraphOperator, InputReference, NeighborCellField, NeighborDirection,
};
use v2_core::runtime::{SensorCellSnapshot, SensorFrame, resolve_graph_inputs};

#[test]
fn graph_can_read_neighbor_input_references() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        0,
        -1,
        SensorCellSnapshot {
            food_density_u8: 180,
            barrier_flag: false,
            occupied_flag: false,
            is_self: false,
            creature: None,
        },
    );
    let graph = GraphBackendDef {
        operator: GraphOperator::SumPool,
        inputs: vec![InputReference::NeighborCell {
            direction: NeighborDirection::North,
            field: NeighborCellField::FoodDensityNorm,
        }],
        coefficients: Vec::new(),
        bias: 0.0,
        state_slot_count: 0,
    };
    let slots = resolve_graph_inputs(&graph.inputs, &frame);
    let mut state = vec![0.0; 1];
    let value = evaluate_graph_operator(&graph, &slots, &mut state);
    assert!(value > 0.0);
}

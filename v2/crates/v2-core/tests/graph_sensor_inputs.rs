use v2_core::backends::evaluate_graph_operator;
use v2_core::mesh::{GraphBackendDef, GraphOperator, InputReference, SensorCellField};
use v2_core::runtime::{SensorCellSnapshot, SensorFrame, resolve_graph_inputs};

#[test]
fn graph_can_read_sensor_input_references() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        1,
        1,
        SensorCellSnapshot {
            food_density_u8: 255,
            barrier_flag: false,
            occupied_flag: false,
            is_self: false,
            creature: None,
        },
    );
    let graph = GraphBackendDef {
        operator: GraphOperator::MeanPool,
        inputs: vec![InputReference::SensorCell {
            dx: 1,
            dy: 1,
            field: SensorCellField::FoodDensityNorm,
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

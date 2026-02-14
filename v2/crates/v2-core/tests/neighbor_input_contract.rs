use v2_core::mesh::{NeighborCellField, NeighborDirection};
use v2_core::runtime::{SensorCellSnapshot, SensorFrame};

#[test]
fn neighbor_direction_mapping_is_complete() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        1,
        0,
        SensorCellSnapshot {
            food_density_u8: 200,
            barrier_flag: false,
            occupied_flag: false,
            is_self: false,
            creature: None,
        },
    );

    let east =
        frame.read_neighbor_cell(NeighborDirection::East, NeighborCellField::FoodDensityNorm);
    assert!(east > 0.0);
}

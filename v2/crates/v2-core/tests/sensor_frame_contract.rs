use v2_core::mesh::{SensorCellField, SensorCreatureField, SensorSummaryField};
use v2_core::runtime::{SensorCellSnapshot, SensorCreatureSnapshot, SensorFrame};

#[test]
fn sensor_frame_reads_cell_and_creature_fields() {
    let mut frame = SensorFrame::empty(2);
    frame.insert_cell(
        0,
        0,
        SensorCellSnapshot {
            food_density_u8: 128,
            barrier_flag: false,
            occupied_flag: true,
            is_self: true,
            creature: Some(SensorCreatureSnapshot {
                phenotype_rgb: [255, 0, 0],
                energy: 5.0,
                age_ticks: 1,
                generation: 0,
            }),
        },
    );

    assert!(frame.read_sensor_cell(0, 0, SensorCellField::FoodDensityNorm) > 0.0);
    assert_eq!(
        frame.read_sensor_creature(0, 0, SensorCreatureField::PresentFlag),
        1.0
    );
    assert!(frame.read_sensor_summary(SensorSummaryField::VisibleFoodMeanNorm) > 0.0);
}

use v2_core::mesh::{InputReference, WorldInputKey};
use v2_core::runtime::{PacketValue, resolve_input_slots};

#[test]
fn read_input_slots_are_ordered_by_input_refs() {
    let slots = resolve_input_slots(
        &[
            InputReference::Packet("x".to_string()),
            InputReference::World(WorldInputKey::FoodHere),
        ],
        &[("x".to_string(), PacketValue::F32(0.8))],
        &v2_core::runtime::SensorFrame::empty(2),
        12.0,
        1.0,
        11.0,
        5.0,
    );
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0], 0.8);
    assert!((slots[1] - 0.0).abs() < f32::EPSILON);
}

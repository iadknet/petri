use petri_core::{World, WorldConfig};

#[test]
fn frame_payload_size_stays_below_threshold_for_default_world() {
    let world = World::new(WorldConfig::default(), 7);
    let frame = world.frame();
    let payload = rmp_serde::to_vec_named(&frame).expect("frame should serialize");
    assert!(
        payload.len() < 200_000,
        "frame payload unexpectedly large: {} bytes",
        payload.len()
    );
}

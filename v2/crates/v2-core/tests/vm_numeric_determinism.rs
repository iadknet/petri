use v2_core::runtime::sanitize_f32;

#[test]
fn sanitize_rejects_nan_and_clamps_extremes() {
    assert_eq!(sanitize_f32(f32::NAN), 0.0);
    assert_eq!(sanitize_f32(f32::INFINITY), 1_000_000_000.0);
    assert_eq!(sanitize_f32(f32::NEG_INFINITY), -1_000_000_000.0);
    assert_eq!(sanitize_f32(1_500_000_000.0), 1_000_000_000.0);
}

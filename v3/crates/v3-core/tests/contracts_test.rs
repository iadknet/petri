#[test]
fn creature_inputs_has_required_fields() {
    use v3_core::contracts::inputs::CreatureInputs;

    let inputs = CreatureInputs::default();

    // Verify fields exist (contract shape test)
    let _env = inputs.environmental;
    let _intro = inputs.introspection;
}

#[test]
fn creature_outputs_noop_has_correct_defaults() {
    use v3_core::contracts::outputs::{CreatureOutputs, WorldAction};

    let outputs = CreatureOutputs::noop();

    assert!(matches!(outputs.world_action, WorldAction::NoOp));
    assert_eq!(outputs.energy_consumed, 0);
}

#[test]
fn environmental_inputs_defaults_to_zero() {
    use v3_core::contracts::inputs::EnvironmentalInputs;

    let env = EnvironmentalInputs::default();
    assert_eq!(env.food_density_self, 0);
}

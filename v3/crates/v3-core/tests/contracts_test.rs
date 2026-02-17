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

#[test]
fn world_action_reproduce_carries_direction_and_energy_amount() {
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::kernel::types::Direction;

    let action = WorldAction::Reproduce {
        direction: Direction::NE,
        energy_amount: 12,
    };

    assert!(matches!(
        action,
        WorldAction::Reproduce {
            direction: Direction::NE,
            energy_amount: 12
        }
    ));
}

#[test]
fn simulation_config_includes_reproduction_energy_defaults() {
    use v3_core::config::SimulationConfig;

    let config = SimulationConfig::default();

    assert_eq!(config.energy.costs.reproduce_cost, 2);
    assert_eq!(config.energy.lifecycle.min_reproduce_energy, 24);
    assert_eq!(config.energy.lifecycle.default_offspring_energy, 12);
}

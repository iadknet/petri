use v2_core::runtime::{RuntimeActionCosts, RuntimeConfig, RuntimeConfigError};

#[test]
fn sensor_radius_must_be_global_and_non_zero() {
    let valid = RuntimeConfig {
        dispatch_entry_cost: 0.1,
        graph_base_tariff: 0.2,
        vm_opcode_cost_multiplier: 1.0,
        sensor_radius: 1,
        action_costs: RuntimeActionCosts {
            move_cost: 0.1,
            eat_cost: 0.1,
            reproduce_cost: 0.1,
            inventory_pickup_cost: 0.1,
            inventory_put_cost: 0.1,
            noop_cost: 0.05,
        },
    };
    assert!(valid.validate().is_ok());

    let invalid = RuntimeConfig {
        sensor_radius: 0,
        ..valid
    };
    assert_eq!(invalid.validate(), Err(RuntimeConfigError::SensorRadiusZero));
}

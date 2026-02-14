use v2_core::ecology::{
    EcologyConfig, band_multiplier_for_cell, crowding_multiplier, season_multiplier_at_tick,
    update_scarcity_multiplier,
};

#[test]
fn resource_gradient_varies_by_band() {
    let config = EcologyConfig::default();

    let left = band_multiplier_for_cell(0, 8, 64, 64, 0, &config);
    let right = band_multiplier_for_cell(63, 8, 64, 64, 0, &config);

    assert_ne!(left, right);
    assert!(left.is_finite());
    assert!(right.is_finite());
}

#[test]
fn season_shift_transitions_are_smoothed() {
    let config = EcologyConfig::default();

    let before = season_multiplier_at_tick(config.season_length_ticks.saturating_sub(1), &config);
    let entering = season_multiplier_at_tick(config.season_length_ticks + 1, &config);
    let settled = season_multiplier_at_tick(
        config.season_length_ticks + config.season_transition_ticks + 1,
        &config,
    );

    assert_ne!(before, settled);
    assert!(entering >= before.min(settled));
    assert!(entering <= before.max(settled));
}

#[test]
fn crowding_pressure_is_monotonic_non_increasing() {
    let config = EcologyConfig::default();

    assert!((crowding_multiplier(0, &config) - 1.0).abs() < 1e-6);
    assert!(
        (crowding_multiplier(1, &config) - (1.0 - config.crowding_penalty_per_neighbor)).abs()
            < 1e-6
    );

    let mut previous = crowding_multiplier(0, &config);
    for neighbors in 1..=64 {
        let current = crowding_multiplier(neighbors, &config);
        assert!(current <= previous + 1e-6);
        previous = current;
    }

    assert_eq!(crowding_multiplier(10_000, &config), 0.0);
}

#[test]
fn scarcity_attenuates_and_recovers_when_consumption_drops() {
    let config = EcologyConfig::default();

    let mut attenuated = 1.0;
    for _ in 0..20 {
        let next = update_scarcity_multiplier(attenuated, 1.0, &config);
        assert!(next <= attenuated + 1e-6);
        attenuated = next;
    }
    assert!(attenuated >= config.scarcity_multiplier_min);

    let mut recovered = attenuated;
    for _ in 0..20 {
        let next = update_scarcity_multiplier(recovered, 0.0, &config);
        assert!(next + 1e-6 >= recovered);
        recovered = next;
    }
    assert!(recovered <= config.scarcity_multiplier_max + 1e-6);
    assert!(recovered >= attenuated);
}

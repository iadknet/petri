#[test]
fn energy_drain_returns_success_when_sufficient() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(10);

    assert!(energy.drain(5));
    assert_eq!(energy.value(), 5);

    assert!(!energy.drain(10)); // Insufficient
    assert_eq!(energy.value(), 5); // Unchanged
}

#[test]
fn energy_charge_caps_at_max() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(5);
    energy.charge(100, 50);
    assert_eq!(energy.value(), 50); // Capped at max
}

#[test]
fn energy_is_alive_when_nonzero() {
    use v3_core::creature::state::Energy;

    let alive = Energy::new(1);
    assert!(alive.is_alive());

    let dead = Energy::new(0);
    assert!(!dead.is_alive());
}

#[test]
fn creature_state_new_initializes_correctly() {
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;

    let creature = CreatureState::new(Position { x: 10, y: 20 }, 15, 0, [255, 128, 64]);

    assert_eq!(creature.position, Position { x: 10, y: 20 });
    assert_eq!(creature.energy.value(), 15);
    assert_eq!(creature.age, 0);
    assert_eq!(creature.generation, 0);
    assert_eq!(creature.phenotype_r, 255);
    assert_eq!(creature.phenotype_g, 128);
    assert_eq!(creature.phenotype_b, 64);
}

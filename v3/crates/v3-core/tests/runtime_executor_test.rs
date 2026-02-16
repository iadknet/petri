#[test]
fn execute_creature_returns_noop_for_stage1() {
    use v3_core::contracts::inputs::CreatureInputs;
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::runtime::executor::execute_creature_stub;

    let inputs = CreatureInputs::default();
    let outputs = execute_creature_stub(&inputs);

    assert!(matches!(outputs.world_action, WorldAction::NoOp));
    assert_eq!(outputs.energy_consumed, 0);
}

#[test]
fn gather_inputs_stub_returns_creature_energy() {
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs_stub;

    let creature = CreatureState::new(Position { x: 5, y: 5 }, 42, 0, [100, 100, 100]);
    let world = WorldState::new(10, 10, true);

    let inputs = gather_inputs_stub(&creature, &world);

    assert_eq!(inputs.introspection.energy, 42);
    assert_eq!(inputs.introspection.position, Position { x: 5, y: 5 });
    assert_eq!(inputs.environmental.food_density_self, 0);
}

use super::super::{run_tick, sort_by_priority_bid};
use super::support::*;
use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{CreatureId, Position, WorldAction};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::runtime::types::{ComputeCostReport, MeshOutput};
use crate::simulation::seeding::seed_simulation;
use slotmap::SlotMap;

#[test]
fn run_tick_increments_tick_counter() {
    let mut sim = seed_simulation(small_config(), 42);
    assert_eq!(sim.tick_number(), 0);
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.tick_number(), 1);
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.tick_number(), 2);
}

#[test]
fn sort_by_priority_bid_orders_descending() {
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let id_a = insert_test_creature(&mut creatures, Position::new(0, 0));
    let id_b = insert_test_creature(&mut creatures, Position::new(1, 0));
    let id_c = insert_test_creature(&mut creatures, Position::new(2, 0));

    let make_output = |bid: f32| MeshOutput {
        actions: vec![WorldAction::eat(OrdinaryFoodTypeId::default())],
        cost_report: ComputeCostReport::default(),
        priority_bid: bid,
    };

    let mut decisions = vec![
        (id_a, make_output(0.0)),
        (id_b, make_output(10.0)),
        (id_c, make_output(5.0)),
    ];

    sort_by_priority_bid(&mut decisions);

    assert_eq!(decisions[0].0, id_b, "highest bidder should be first");
    assert_eq!(
        decisions[1].0, id_c,
        "second highest bidder should be second"
    );
    assert_eq!(decisions[2].0, id_a, "zero bidder should be last");
}

#[test]
fn sort_by_priority_bid_stable_for_equal_bids() {
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let id_a = insert_test_creature(&mut creatures, Position::new(0, 0));
    let id_b = insert_test_creature(&mut creatures, Position::new(1, 0));
    let id_c = insert_test_creature(&mut creatures, Position::new(2, 0));

    let make_output = |bid: f32| MeshOutput {
        actions: vec![WorldAction::eat(OrdinaryFoodTypeId::default())],
        cost_report: ComputeCostReport::default(),
        priority_bid: bid,
    };

    let mut decisions = vec![
        (id_a, make_output(0.0)),
        (id_b, make_output(0.0)),
        (id_c, make_output(0.0)),
    ];

    sort_by_priority_bid(&mut decisions);

    assert_eq!(decisions[0].0, id_a);
    assert_eq!(decisions[1].0, id_b);
    assert_eq!(decisions[2].0, id_c);
}

fn insert_test_creature(
    creatures: &mut SlotMap<CreatureId, CreatureState>,
    position: Position,
) -> CreatureId {
    creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            position,
            50.0,
            0,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    })
}

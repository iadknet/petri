//! T11.F17: the per-creature dispatch record the mutation engine reads at a
//! birth is written by the live tick loop and never by an observation.

use super::super::{observe_final_actions, observe_temporal_actions, run_tick};
use super::support::*;
use crate::simulation::seeding::seed_simulation;

/// The record after N ticks, as sorted node indices within a window that
/// covers the whole life so far.
fn recorded(sim: &crate::simulation::Simulation, id: crate::contracts::CreatureId) -> Vec<usize> {
    let creature = &sim.creatures[id];
    creature
        .graph_runtime
        .dispatch_record
        .executed_indices(creature.age, u64::MAX)
}

#[test]
fn living_creatures_accumulate_dispatches_and_observations_leave_them_alone() {
    let mut sim = seed_simulation(small_config(), 7);
    for _ in 0..5 {
        run_tick(&mut sim, &mut None);
    }
    let ids: Vec<_> = sim.creatures.keys().collect();
    assert!(!ids.is_empty(), "the fixture needs living creatures");
    let before: Vec<_> = ids.iter().map(|&id| recorded(&sim, id)).collect();
    assert!(
        before.iter().all(|indices| indices.contains(&0)),
        "every living founder dispatched its entry node: {before:?}"
    );

    let _ = observe_final_actions(&sim);
    let _ = observe_temporal_actions(&sim);
    let after: Vec<_> = ids.iter().map(|&id| recorded(&sim, id)).collect();
    assert_eq!(
        before, after,
        "observation clones must never write the live dispatch record"
    );
}

#[test]
fn the_window_forgets_a_node_the_creature_stopped_dispatching() {
    let (mut sim, id) = make_sim_with_one_creature(50.0);
    // One tick of real cognition, then age past a one-tick window.
    run_tick(&mut sim, &mut None);
    let creature = &sim.creatures[id];
    let age = creature.age;
    let record = &creature.graph_runtime.dispatch_record;
    assert_eq!(record.executed_indices(age, 1), vec![0, 1]);
    assert!(record.executed_indices(age + 1, 1).is_empty());
}

#[test]
fn a_newborn_starts_from_its_own_dispatches_not_its_parents() {
    use crate::contracts::Direction;
    use crate::simulation::actions::{apply_reproduce, ReproductionActionResult};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let (mut sim, parent) = make_sim_with_one_creature(80.0);
    for _ in 0..5 {
        run_tick(&mut sim, &mut None);
    }
    assert!(
        !sim.creatures[parent]
            .graph_runtime
            .dispatch_record
            .is_empty(),
        "the parent has dispatched by now"
    );
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    sim.creatures[parent].energy = 80.0;

    let mut rng = SmallRng::seed_from_u64(3);
    let result = apply_reproduce(parent, &mut sim, Direction::N, 20.0, &mut rng);
    assert_eq!(result, ReproductionActionResult::Spawned);
    let child = sim
        .creatures
        .values()
        .find(|creature| creature.id != parent)
        .expect("child not found");
    assert!(
        child.graph_runtime.dispatch_record.is_empty(),
        "a newborn never inherits its parent's record"
    );
}

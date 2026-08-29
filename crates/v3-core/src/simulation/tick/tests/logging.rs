use super::super::run_tick;
use super::support::*;
use crate::creature::action_log::ActionType;
use crate::simulation::seeding::seed_simulation;

#[test]
fn action_log_populated_after_tick() {
    let mut sim = seed_simulation(small_config(), 42);
    run_tick(&mut sim, &mut None);

    let mut creatures_with_log = 0;
    for (id, _) in sim.creatures.iter() {
        if let Some(log) = sim.action_logs.get(id) {
            if !log.is_empty() {
                creatures_with_log += 1;
                let entry = &log.entries()[0];
                assert_eq!(entry.tick, 0, "first tick should be 0");
                assert!(entry.energy_before > 0.0, "energy_before should be > 0");
            }
        }
    }
    assert!(
        creatures_with_log > 0,
        "at least one creature should have action log entries after a tick"
    );
}

#[test]
fn action_log_records_correct_action_types() {
    let mut sim = seed_simulation(small_config(), 42);
    run_tick(&mut sim, &mut None);

    for (id, _) in sim.creatures.iter() {
        if let Some(log) = sim.action_logs.get(id) {
            for entry in log.entries() {
                assert!(matches!(
                    entry.action_type,
                    ActionType::NoOp
                        | ActionType::Eat
                        | ActionType::Move
                        | ActionType::Reproduce
                        | ActionType::StealEnergy
                ));
            }
        }
    }
}

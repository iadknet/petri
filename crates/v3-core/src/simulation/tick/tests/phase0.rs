use super::super::run_phase_0;
use super::support::*;
use crate::config::SimulationConfig;
use crate::contracts::Position;
use crate::mutation::MutationOperator;
use crate::simulation::seeding::seed_simulation;

#[test]
fn phase_0_increments_creature_age() {
    let (mut sim, id) = make_sim_with_one_creature(50.0);
    assert_eq!(sim.creatures[id].age, 0);
    run_phase_0(&mut sim);
    assert_eq!(sim.creatures[id].age, 1);
}

#[test]
fn phase_0_decays_energy() {
    let initial = 50.0f32;
    let (mut sim, id) = make_sim_with_one_creature(initial);
    let decay = sim.config.energy.lifecycle.energy_decay_per_tick;
    run_phase_0(&mut sim);
    let expected = initial - decay;
    assert!(
        (sim.creatures[id].energy - expected).abs() < f32::EPSILON,
        "energy {} vs expected {}",
        sim.creatures[id].energy,
        expected
    );
}

#[test]
fn phase_0_removes_dead_creatures() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
    assert!(sim.creatures.contains_key(id));
    run_phase_0(&mut sim);
    assert!(!sim.creatures.contains_key(id));
}

#[test]
fn phase_0_grows_food() {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 1.0;
    cfg.world.food.initial_coverage = 0.0;
    let mut sim = seed_simulation(cfg, 42);
    let before = sim.world.total_food();
    run_phase_0(&mut sim);
    assert!(sim.world.total_food() > before);
}

#[test]
fn dead_creature_removed_from_occupancy() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, _) = make_sim_with_one_creature(decay * 0.5);
    let pos = Position::new(5, 5);
    assert!(sim.world.creature_at(pos).is_some());
    run_phase_0(&mut sim);
    assert!(sim.world.creature_at(pos).is_none());
}

#[test]
fn dead_creature_action_log_removed() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
    sim.action_logs
        .insert(id, crate::creature::action_log::ActionLog::new(500));

    assert!(sim.action_logs.contains_key(id));
    run_phase_0(&mut sim);
    assert!(!sim.creatures.contains_key(id));
    assert!(!sim.action_logs.contains_key(id));
}

#[test]
fn phase_0_snapshots_shared_memory_to_prev() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    let creature = sim.creatures.get_mut(id).unwrap();
    creature.shared_memory[0] = 1.5;
    creature.shared_memory[3] = -2.0;
    creature.shared_memory[15] = 42.0;
    assert_eq!(creature.prev_shared_memory, [0.0; 16]);

    run_phase_0(&mut sim);

    let creature = sim.creatures.get(id).unwrap();
    assert_eq!(creature.prev_shared_memory[0], 1.5);
    assert_eq!(creature.prev_shared_memory[3], -2.0);
    assert_eq!(creature.prev_shared_memory[15], 42.0);
}

#[test]
fn phase_0_decay_reduces_shared_memory_slots() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.shared_memory.decay_rate = 0.1;
    let creature = sim.creatures.get_mut(id).unwrap();
    creature.shared_memory[0] = 10.0;
    creature.shared_memory[5] = -4.0;

    run_phase_0(&mut sim);

    let creature = sim.creatures.get(id).unwrap();
    assert!((creature.shared_memory[0] - 9.0).abs() < f32::EPSILON);
    assert!((creature.shared_memory[5] - (-3.6)).abs() < f32::EPSILON);
    assert_eq!(creature.shared_memory[1], 0.0);
}

#[test]
fn phase_0_no_decay_when_rate_is_zero() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    assert_eq!(sim.config.shared_memory.decay_rate, 0.0);
    let creature = sim.creatures.get_mut(id).unwrap();
    creature.shared_memory[7] = 5.0;

    run_phase_0(&mut sim);

    let creature = sim.creatures.get(id).unwrap();
    assert_eq!(creature.shared_memory[7], 5.0);
}

#[test]
fn phase_0_records_mutation_value_totals_on_creature_death() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
    let creature = sim.creatures.get_mut(id).expect("creature should exist");
    creature.age = 9;
    creature.offspring_spawned_count = 3;
    creature.birth_mutation_operators = vec![
        MutationOperator::VmInstructionMutation,
        MutationOperator::GraphMutateHebbianRule,
    ]
    .into_boxed_slice();

    run_phase_0(&mut sim);

    for operator in [
        MutationOperator::VmInstructionMutation,
        MutationOperator::GraphMutateHebbianRule,
    ] {
        let totals = sim
            .stats
            .mutation_value_totals_by_operator
            .get(&operator)
            .expect("totals should be recorded");
        assert_eq!(totals.carriers_observed_total, 1);
        assert_eq!(totals.survival_ticks_sum, 10);
        assert_eq!(totals.offspring_spawned_sum, 3);
        assert_eq!(
            totals.helpful_total + totals.neutral_total + totals.detrimental_total,
            totals.carriers_observed_total
        );
        assert_eq!(
            totals.confidence_low_total
                + totals.confidence_medium_total
                + totals.confidence_high_total,
            totals.carriers_observed_total
        );
    }
    assert_eq!(
        sim.stats.mutation_outcome_summary.carriers_observed_total,
        1
    );
    assert_eq!(
        sim.stats.mutation_outcome_summary.helpful_total
            + sim.stats.mutation_outcome_summary.neutral_total
            + sim.stats.mutation_outcome_summary.detrimental_total,
        sim.stats.mutation_outcome_summary.carriers_observed_total
    );
}

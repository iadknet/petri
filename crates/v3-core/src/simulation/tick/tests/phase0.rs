use super::super::{run_phase_0, Phase0Charge};
use super::support::*;
use crate::config::SimulationConfig;
use crate::contracts::Position;
use crate::mutation::MutationOperator;
use crate::simulation::seeding::seed_simulation;
use proptest::prelude::*;

#[test]
fn phase_0_increments_creature_age() {
    let (mut sim, id) = make_sim_with_one_creature(50.0);
    assert_eq!(sim.creatures[id].age, 0);
    run_phase_0(&mut sim);
    assert_eq!(sim.creatures[id].age, 1);
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
    cfg.world.food.types[0].initial_coverage = 0.0;
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

/// `drive_births` (T11.F30): a carrier's outcome lands under its birth edge
/// class by the same law as under its operator; a carrier without a class
/// records nothing there.
#[test]
fn phase_0_records_the_carrier_outcome_under_its_birth_edge_class() {
    use crate::mutation::BirthEdgeClass;
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    for class in [Some(BirthEdgeClass::Drive), None] {
        let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
        let creature = sim.creatures.get_mut(id).expect("creature should exist");
        creature.age = 9;
        creature.offspring_spawned_count = 3;
        creature.birth_mutation_operators =
            vec![MutationOperator::GraphAddGraphEdge].into_boxed_slice();
        creature.birth_edge_class = class;

        run_phase_0(&mut sim);

        let by_operator =
            &sim.stats.mutation_value_totals_by_operator[&MutationOperator::GraphAddGraphEdge];
        let by_class = &sim.stats.mutation_value_totals_by_birth_edge_class;
        match class {
            Some(class) => {
                assert_eq!(by_class.len(), 1);
                assert_eq!(&by_class[&class], by_operator);
            }
            None => assert!(by_class.is_empty()),
        }
    }
}

// ── Genome carrying cost (T03.F08) ──────────────────────────────────────────

#[test]
fn phase_0_charge_is_decay_plus_the_rate_times_the_genome_size() {
    assert_eq!(
        Phase0Charge::new(0.5, 1e-4, 111, 0.0).base,
        0.5 + 111.0 * 1e-4
    );
}

#[test]
fn phase_0_charge_at_rate_one_and_no_decay_is_the_genome_size() {
    assert_eq!(Phase0Charge::new(0.0, 1.0, 250, 0.0).base, 250.0);
}

#[test]
fn phase_0_charge_without_a_rate_or_without_a_genome_is_the_decay_alone() {
    assert_eq!(Phase0Charge::new(0.5, 0.0, 6246, 0.0).base, 0.5);
    assert_eq!(Phase0Charge::new(0.5, 1e-4, 0, 0.0).base, 0.5);
}

#[test]
fn phase_0_charges_the_founder_its_carrying_cost_beside_decay() {
    let initial = 50.0f32;
    let (mut sim, id) = make_sim_with_one_creature(initial);
    let decay = sim.config.energy.lifecycle.energy_decay_per_tick;
    let rate = sim.config.energy.lifecycle.genome_carry_cost_per_unit;
    assert_eq!(rate, 1e-4);
    // The canonical founder carries 80 genome units under the T25.F02 unit
    // rule (97 before it; the old rule restores 97).
    assert_eq!(sim.creatures[id].genome_size(), 80);

    run_phase_0(&mut sim);

    let expected = initial - (decay + 80.0 * rate);
    assert_eq!(sim.creatures[id].energy, expected);
    // The carrying charge is visible: decay alone would leave more energy.
    assert!(sim.creatures[id].energy < initial - decay);
}

#[test]
fn phase_0_charge_scales_with_the_genome_size() {
    let initial = 50.0f32;
    let (mut sim, id) = make_sim_with_custom_genome(initial, sized_genome(7));
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 1.0;
    assert_eq!(sim.creatures[id].genome_size(), 7);

    run_phase_0(&mut sim);

    assert_eq!(sim.creatures[id].energy, initial - 7.0);
}

#[test]
fn phase_0_at_rate_zero_reproduces_the_pre_feature_energy_bit_for_bit() {
    let initial = 50.0f32;
    let (mut sim, id) = make_sim_with_one_creature(initial);
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    let decay = sim.config.energy.lifecycle.energy_decay_per_tick;

    run_phase_0(&mut sim);

    assert_eq!(
        sim.creatures[id].energy.to_bits(),
        (initial - decay).to_bits()
    );
}

#[test]
fn a_creature_the_carrying_charge_takes_to_zero_dies_in_the_same_tick() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, sized_genome(25));
    // Decay alone leaves the creature alive; the carrying charge is what kills it.
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.75;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.01;
    assert_eq!(sim.creatures[id].genome_size(), 25);

    run_phase_0(&mut sim);

    assert!(!sim.creatures.contains_key(id));
    assert!(sim.world.creature_at(Position::new(5, 5)).is_none());
}

#[test]
fn a_creature_the_carrying_charge_leaves_above_zero_survives_the_tick() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, sized_genome(10));
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.75;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.01;
    assert_eq!(sim.creatures[id].genome_size(), 10);

    run_phase_0(&mut sim);

    assert!(sim.creatures.contains_key(id));
    assert_eq!(sim.creatures[id].energy, 1.0 - (0.75 + 0.01 * 10.0));
}

#[test]
fn phase_0_samples_lifetime_energy_after_the_carrying_charge() {
    let initial = 50.0f32;
    let (mut sim, id) = make_sim_with_custom_genome(initial, sized_genome(10));
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 1.0;
    assert_eq!(sim.creatures[id].genome_size(), 10);

    run_phase_0(&mut sim);

    assert_eq!(sim.creatures[id].lifetime_energy_sum, 40.0);
    assert_eq!(sim.creatures[id].lifetime_energy_sample_count, 1);
}

// ── Pure charge invariants (proptest) ───────────────────────────────────────

proptest! {
    /// The charge is exactly the decay plus the rate times the size.
    #[test]
    fn the_charge_is_the_decay_plus_the_rate_times_the_size(
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        size in 0u32..20_000,
    ) {
        prop_assert_eq!(
            Phase0Charge::new(decay, rate, size, 0.0).base,
            decay + rate * size as f32
        );
    }

    /// A creature that carries more structure never pays less.
    #[test]
    fn the_charge_is_monotone_non_decreasing_in_genome_size(
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        size in 0u32..20_000,
        extra in 1u32..20_000,
    ) {
        prop_assert!(
            Phase0Charge::new(decay, rate, size + extra, 0.0).base
                >= Phase0Charge::new(decay, rate, size, 0.0).base
        );
    }

    /// Raising the rate never lowers the charge.
    #[test]
    fn the_charge_is_monotone_non_decreasing_in_the_rate(
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        extra in 0.0f32..1.0,
        size in 0u32..20_000,
    ) {
        prop_assert!(
            Phase0Charge::new(decay, rate + extra, size, 0.0).base
                >= Phase0Charge::new(decay, rate, size, 0.0).base
        );
    }

    /// With the rate at zero, or with no genome to carry, the charge is the
    /// pre-feature decay, bit for bit.
    #[test]
    fn a_zero_rate_or_an_empty_genome_charges_exactly_the_decay(
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        size in 0u32..20_000,
    ) {
        prop_assert_eq!(Phase0Charge::new(decay, 0.0, size, 0.0).base.to_bits(), decay.to_bits());
        prop_assert_eq!(Phase0Charge::new(decay, rate, 0, 0.0).base.to_bits(), decay.to_bits());
    }

    /// A non-negative decay and a non-negative rate never credit energy.
    #[test]
    fn the_charge_is_never_negative(
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        size in 0u32..20_000,
    ) {
        prop_assert!(Phase0Charge::new(decay, rate, size, 0.0).base >= 0.0);
    }

    /// At the reference (upkeep `0.0`) the settled charge is today's bit for
    /// bit: one subtraction of `base`, today's decay-first booking, a zero
    /// upkeep flow and never a death attributed to upkeep.
    #[test]
    fn a_reference_body_settles_todays_charge(
        before in -10.0f32..1_000.0,
        decay in 0.0f32..10.0,
        rate in 0.0f32..1.0,
        size in 0u32..20_000,
    ) {
        use crate::ledger::{decay_partition, DeathCause};
        let charge = Phase0Charge::new(decay, rate, size, 0.0);
        let settled = charge.settle(before);
        prop_assert_eq!(settled.after.to_bits(), (before - charge.base).to_bits());
        let (decay_share, carrying) = decay_partition(before, before - charge.base, decay);
        prop_assert_eq!(settled.decay.to_bits(), decay_share.to_bits());
        prop_assert_eq!(settled.carrying.to_bits(), carrying.to_bits());
        prop_assert_eq!(settled.upkeep, 0.0);
        if before > 0.0 && settled.after <= 0.0 {
            prop_assert_ne!(settled.cause, DeathCause::CapacityUpkeep);
        }
    }

    /// The three shares always sum to the applied debit, the applied charge
    /// is never negative, and a body below the reference books a
    /// non-positive upkeep.
    #[test]
    fn the_settled_shares_sum_to_the_applied_debit(
        before in 0.0f32..1_000.0,
        decay in 0.0f32..10.0,
        rate in 0.0f32..0.01,
        size in 0u32..2_000,
        upkeep in -2.0f32..2.0,
    ) {
        let charge = Phase0Charge::new(decay, rate, size, upkeep);
        let settled = charge.settle(before);
        prop_assert!(charge.applied() >= 0.0);
        prop_assert!(settled.after <= before);
        let applied = f64::from(before) - f64::from(settled.after);
        prop_assert_eq!(settled.decay + settled.carrying + settled.upkeep, applied);
        if upkeep <= 0.0 {
            prop_assert!(settled.upkeep <= 0.0);
        }
    }
}

/// A large body's upkeep is charged beside decay and carrying and booked to
/// `capacity_upkeep`; a death it causes is attributed to `CapacityUpkeep`.
#[test]
fn phase_0_charges_and_attributes_a_large_bodys_upkeep() {
    use crate::ledger::DeathCause;
    let (mut sim, id) = make_sim_with_one_creature(1.0);
    let lifecycle = &mut sim.config.energy.lifecycle;
    lifecycle.energy_decay_per_tick = 0.25;
    lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.capacity.upkeep_per_unit = 0.0025;
    sim.creatures[id].body.capacity = 600.0; // upkeep 0.0025 × 400 = 1.0
    run_phase_0(&mut sim);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.lifecycle_decay, 0.25);
    assert_eq!(flows.genome_carrying, 0.0);
    assert_eq!(flows.capacity_upkeep, 1.0);
    assert!(!sim.creatures.contains_key(id));
    assert_eq!(sim.stats.mortality.count(DeathCause::CapacityUpkeep), 1);
}

/// A small body's negative upkeep lowers the charge, and the zero floor's
/// shortfall lands in `capacity_upkeep`: the creature is never credited.
#[test]
fn a_small_bodys_negative_upkeep_is_floored_at_zero_charge() {
    let (mut sim, id) = make_sim_with_one_creature(10.0);
    let lifecycle = &mut sim.config.energy.lifecycle;
    lifecycle.energy_decay_per_tick = 0.5;
    lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.capacity.upkeep_per_unit = 0.01;
    sim.creatures[id].body.capacity = 50.0; // upkeep 0.01 × -150 = -1.5
    run_phase_0(&mut sim);
    assert_eq!(sim.creatures[id].energy, 10.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.lifecycle_decay, 0.5);
    assert_eq!(flows.capacity_upkeep, -0.5);
}

/// T26.F02 invariant 5's attribution fixture: `before = 1`,
/// `decay = 2^-25`, `carry × size = 1 − 2^-24`. The `f32` base rounds to
/// exactly 1, so `before − base = 0` and the death is the carrying cost's,
/// though the exact sum is below 1: a body at the reference is never
/// attributed to upkeep.
#[test]
fn a_reference_death_on_the_rounded_base_is_attributed_to_carrying() {
    use crate::ledger::DeathCause;
    let decay = 2.0f32.powi(-25);
    let charge = Phase0Charge::new(decay, 1.0 - 2.0f32.powi(-24), 1, 0.0);
    assert_eq!(charge.base, 1.0);
    let settled = charge.settle(1.0);
    assert_eq!(settled.after, 0.0);
    assert_eq!(settled.cause, DeathCause::GenomeCarrying);
    assert_eq!(settled.upkeep, 0.0);
}

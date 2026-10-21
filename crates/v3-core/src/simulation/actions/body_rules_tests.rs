//! Characterization of the body rules (T26.F01): the energy cap at the three
//! credit sites, the uncapped birth, the move charge and the order of the
//! body inheritance draws. Each assertion pins main's behavior before the
//! refactor, so a moved booking or draw is a defect.

use crate::creature::action_log::ActionType;
use rand::SeedableRng;
use slotmap::SlotMap;

use super::{apply_move, apply_reproduce, apply_steal_energy, apply_typed_eat};
use super::{PredationActionResult, ReproductionActionResult};
use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{CreatureId, Direction, Position};
use crate::creature::body::HeritableBody;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::phenotype::Phenotype;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::simulation::energy_accounting::{DeathCause, EnergyFlows};
use crate::simulation::simulation::Simulation;

fn config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.world.food.growth_rate = 0.0;
    // Births are delivered at conception (T03.F12 rate 0.0).
    cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
    cfg
}

/// A simulation holding one founder-phenotype creature per `(position, energy)`.
fn sim_with(creatures_at: &[(Position, f32)]) -> (Simulation, Vec<CreatureId>) {
    let cfg = config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let mut ids = Vec::new();
    for &(position, energy) in creatures_at {
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                position,
                energy,
                0,
                HeritableBody::founder(3, 77),
                [0.0; 16],
            )
        });
        world.place_creature(position, id);
        ids.push(id);
    }
    (Simulation::new(world, creatures, 0, cfg, 42), ids)
}

/// Exact applied debit, as the accounting books it.
fn debit(before: f32, after: f32) -> f64 {
    f64::from(before) - f64::from(after)
}

// ── eat ──────────────────────────────────────────────────────────────────────

/// One eat of 0.5 food of type 0 at `reward` per unit by a creature at
/// `energy`; returns the creature and the flows the eat booked.
fn eat_once(energy: f32, reward: f32) -> (Simulation, CreatureId, EnergyFlows) {
    let pos = Position::new(3, 3);
    let (mut sim, ids) = sim_with(&[(pos, energy)]);
    let id = ids[0];
    sim.config.world.food.types[0].energy_per_unit = Some(reward);
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.5);
    let mut flows = EnergyFlows::default();
    let eaten = apply_typed_eat(
        &mut sim.creatures[id],
        &mut sim.world,
        &sim.config,
        OrdinaryFoodTypeId::new(0),
        0.0,
        &mut flows,
    );
    assert!(eaten > 0.0, "the fixture eat must consume food");
    (sim, id, flows)
}

fn eat_effort(sim: &Simulation, id: CreatureId) -> f32 {
    let creature = &sim.creatures[id];
    sim.config.energy.action_effort(
        sim.config.energy.costs.eat_cost,
        creature.cached_complexity,
        creature.age,
    )
}

#[test]
fn eat_caps_a_credit_above_the_maximum_and_books_the_excess() {
    let (sim, id, flows) = eat_once(195.0, 40.0);
    let max = sim.config.energy.lifecycle.max_energy;
    let credited = 195.0_f32 + 0.5 * 40.0;
    assert_eq!(flows.food_intake_by_type[0], debit(credited, 195.0));
    assert_eq!(flows.maximum_energy_clamp_loss, debit(credited, max));
    assert_eq!(flows.zero_floor_credit, 0.0);
    let effort = eat_effort(&sim, id);
    assert_eq!(sim.creatures[id].energy, max - effort);
}

#[test]
fn eat_floors_a_negative_credit_to_zero_and_books_the_floor() {
    let (sim, id, flows) = eat_once(10.0, -100.0);
    let credited = 10.0_f32 + 0.5 * -100.0;
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(flows.zero_floor_credit, debit(0.0, credited));
    assert_eq!(flows.food_intake_by_type[0], debit(credited, 10.0));
    let effort = eat_effort(&sim, id);
    assert_eq!(sim.creatures[id].energy, 0.0 - effort);
    // Observed once, after the floor: the creature crossed zero at the eat.
    assert_eq!(
        sim.creatures[id].pending_death_cause,
        Some(DeathCause::ActionEat)
    );
}

#[test]
fn eat_caps_a_positive_infinite_credit() {
    let (sim, id, flows) = eat_once(10.0, f32::INFINITY);
    let max = sim.config.energy.lifecycle.max_energy;
    assert_eq!(flows.food_intake_by_type[0], f64::INFINITY);
    assert_eq!(flows.maximum_energy_clamp_loss, f64::INFINITY);
    assert_eq!(flows.zero_floor_credit, 0.0);
    assert_eq!(sim.creatures[id].energy, max - eat_effort(&sim, id));
    assert_eq!(sim.creatures[id].pending_death_cause, None);
}

#[test]
fn eat_floors_a_negative_infinite_credit_to_zero() {
    let (sim, id, flows) = eat_once(10.0, f32::NEG_INFINITY);
    assert_eq!(flows.food_intake_by_type[0], f64::NEG_INFINITY);
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(flows.zero_floor_credit, f64::INFINITY);
    assert_eq!(sim.creatures[id].energy, 0.0 - eat_effort(&sim, id));
    assert_eq!(
        sim.creatures[id].pending_death_cause,
        Some(DeathCause::ActionEat)
    );
}

#[test]
fn eat_leaves_a_nan_credit_nan_and_books_neither_bound() {
    let (sim, id, flows) = eat_once(10.0, f32::NAN);
    assert!(flows.food_intake_by_type[0].is_nan());
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(flows.zero_floor_credit, 0.0);
    assert!(sim.creatures[id].energy.is_nan());
}

#[test]
fn eat_leaves_a_nan_energy_nan() {
    let (sim, id, flows) = eat_once(f32::NAN, 5.0);
    assert!(flows.food_intake_by_type[0].is_nan());
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(flows.zero_floor_credit, 0.0);
    assert!(sim.creatures[id].energy.is_nan());
}

/// Equal as stored values: the same number, or both `NaN`.
fn same_f32(a: f32, b: f32) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

fn same_f64(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

proptest::proptest! {
    /// For every starting energy and reward, finite or not, the eat's floor
    /// and cap give the energy and bookings of `clamp(0.0, max_energy)`
    /// with `max(0.0)` bookings, the form it replaced.
    #[test]
    fn eat_floor_and_cap_equal_the_clamp_form(
        energy in proptest::num::f32::ANY,
        reward in proptest::num::f32::ANY,
    ) {
        let (sim, id, flows) = eat_once(energy, reward);
        let max = sim.config.energy.lifecycle.max_energy;
        let credited = energy + 0.5 * reward;
        let clamped = credited.clamp(0.0, max);
        proptest::prop_assert!(same_f32(sim.creatures[id].energy, clamped - eat_effort(&sim, id)));
        proptest::prop_assert!(same_f64(
            flows.maximum_energy_clamp_loss,
            debit(credited, clamped).max(0.0)
        ));
        proptest::prop_assert!(same_f64(flows.zero_floor_credit, debit(clamped, credited).max(0.0)));
        proptest::prop_assert!(same_f64(flows.food_intake_by_type[0], debit(credited, energy)));
    }
}

// ── predation ────────────────────────────────────────────────────────────────

/// Attacker at 195 north-adjacent to a victim at `victim_energy`, stealing
/// 20 with no rate cost and a kill bonus of 1 per complexity unit.
fn steal(victim_energy: f32) -> (Simulation, CreatureId, PredationActionResult, f32) {
    let (mut sim, ids) = sim_with(&[
        (Position::new(5, 5), 195.0),
        (Position::new(5, 4), victim_energy),
    ]);
    sim.config.predation.steal_cost_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 1.0;
    let attacker = &sim.creatures[ids[0]];
    let effort = sim
        .config
        .energy
        .action_effort(0.0, attacker.cached_complexity, attacker.age);
    let result = apply_steal_energy(ids[0], &mut sim, Direction::N, 20.0).result;
    (sim, ids[0], result, 195.0 - effort)
}

#[test]
fn predation_observes_the_full_transfer_before_capping_it() {
    let (sim, attacker, result, after_effort) = steal(50.0);
    assert_eq!(result, PredationActionResult::Transferred);
    let max = sim.config.energy.lifecycle.max_energy;
    let transferred = after_effort + 20.0;
    assert!(transferred > max, "the fixture must put the cap in play");
    let flows = &sim.stats.energy_flows;
    // The signed reverse of the attacker's observed debit: the whole steal.
    assert_eq!(
        flows.predation_attacker_credit,
        -debit(after_effort, transferred)
    );
    assert_eq!(flows.maximum_energy_clamp_loss, debit(transferred, max));
    assert_eq!(flows.predation_kill_bonus_credit, 0.0);
    assert_eq!(sim.creatures[attacker].energy, max);
}

#[test]
fn predation_caps_after_the_transfer_and_again_after_the_kill_bonus() {
    let (sim, attacker, result, after_effort) = steal(10.0);
    assert_eq!(result, PredationActionResult::TransferredAndKilled);
    let max = sim.config.energy.lifecycle.max_energy;
    let transferred = after_effort + 10.0;
    assert!(
        transferred > max,
        "the fixture must put the first cap in play"
    );
    let bonus = sim.creatures[attacker].cached_complexity as f32;
    assert!(bonus > 0.0, "the fixture must award a kill bonus");
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.predation_attacker_credit,
        -debit(after_effort, transferred)
    );
    assert_eq!(flows.predation_kill_bonus_credit, -debit(max, max + bonus));
    assert_eq!(
        flows.maximum_energy_clamp_loss,
        debit(transferred, max) + debit(max + bonus, max)
    );
    assert_eq!(sim.creatures[attacker].energy, max);
}

#[test]
fn predation_under_the_cap_books_no_clamp_loss() {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 50.0), (Position::new(5, 4), 50.0)]);
    sim.config.predation.steal_cost_rate = 0.0;
    let result = apply_steal_energy(ids[0], &mut sim, Direction::N, 20.0).result;
    assert_eq!(result, PredationActionResult::Transferred);
    assert_eq!(sim.stats.energy_flows.maximum_energy_clamp_loss, 0.0);
}

// ── birth ────────────────────────────────────────────────────────────────────

#[test]
fn a_birth_whose_litter_exceeds_the_maximum_is_not_capped() {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 80.0)]);
    let parent = ids[0];
    sim.config.energy.lifecycle.max_energy = 20.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
    let result = apply_reproduce(parent, &mut sim, Direction::N, 0.5, &mut rng);
    assert_eq!(result, ReproductionActionResult::Conceived);
    let child = sim
        .creatures
        .values()
        .find(|creature| creature.id != parent)
        .expect("one child");
    assert!(
        child.energy > 20.0,
        "litter {} must stay uncapped",
        child.energy
    );
    assert_eq!(sim.stats.energy_flows.maximum_energy_clamp_loss, 0.0);
}

// ── move ─────────────────────────────────────────────────────────────────────

/// One move north from (5, 5); returns the outcome, the move effort the
/// rule names, the energy before, and the creature's state after.
fn move_once(blocked: bool) -> (bool, f32, Simulation, CreatureId) {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 50.0)]);
    let id = ids[0];
    sim.creatures[id].age = 300;
    if blocked {
        sim.world.set_barrier(Position::new(5, 4), true);
    }
    let creature = &sim.creatures[id];
    let effort = sim.config.energy.action_effort(
        sim.config.energy.costs.move_cost,
        creature.cached_complexity,
        creature.age,
    );
    let moved = apply_move(
        id,
        &mut sim.creatures[id],
        &mut sim.world,
        Direction::N,
        &sim.config,
        &mut sim.stats.energy_flows,
    );
    (moved, effort, sim, id)
}

#[test]
fn a_successful_move_charges_the_move_effort() {
    let (moved, effort, sim, id) = move_once(false);
    assert!(moved);
    assert_eq!(sim.creatures[id].energy, 50.0 - effort);
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.action_charges[ActionType::Move],
        debit(50.0, 50.0 - effort)
    );
    assert_eq!(flows.failed_action_effort[ActionType::Move], 0.0);
}

#[test]
fn a_blocked_move_charges_the_move_effort_as_failed() {
    let (moved, effort, sim, id) = move_once(true);
    assert!(!moved);
    assert_eq!(sim.creatures[id].energy, 50.0 - effort);
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.action_charges[ActionType::Move],
        debit(50.0, 50.0 - effort)
    );
    assert_eq!(
        flows.failed_action_effort[ActionType::Move],
        debit(50.0, 50.0 - effort)
    );
}

// ── inheritance ──────────────────────────────────────────────────────────────

/// The parent body every fixture creature carries.
fn parent_body() -> HeritableBody {
    HeritableBody::founder(3, 77)
}

/// A birth from a founder whose genome mutates (`per_unit_rate`) with
/// phenotype switch and flip chances that make every phenotype draw count.
/// Returns the child's body.
fn child_body(per_unit_rate: f64, seed: u64) -> HeritableBody {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 80.0)]);
    let parent = ids[0];
    sim.config.mutation.per_unit_rate = per_unit_rate;
    sim.config.mutation.phenotype.channel_change_chance = 0.5;
    sim.config.mutation.phenotype.polarity_flip_chance = 0.5;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let result = apply_reproduce(parent, &mut sim, Direction::N, 0.5, &mut rng);
    assert_eq!(result, ReproductionActionResult::Conceived);
    sim.creatures
        .values()
        .find(|creature| creature.id != parent)
        .expect("one child")
        .body
}

/// The draws after genome mutation, phenotype first and kin tag second,
/// pinned on main: swapping them moves these values. Re-pinned by T27.F01
/// (rebase: T27.F01 combined with T26.F01 + T25.F10): the 40-wide
/// `NearbyCreature` family and the 30-bucket key draw change the genome
/// mutation's draws before the phenotype step (channels
/// `[0, 0, 92, 91, 138, 138]`, active 3 -> `[0, 0, 93, 92, 138, 138]`,
/// active 2, polarity all adding); the kin tag holds.
#[test]
fn a_mutated_birth_draws_phenotype_then_kin_tag() {
    let parent = parent_body();
    let child = child_body(0.5, 7);
    assert_eq!(child.identity.lineage_id, parent.identity.lineage_id);
    assert_eq!(
        (child.identity.kin_tag ^ parent.identity.kin_tag).count_ones(),
        1
    );
    assert_eq!(
        child,
        HeritableBody {
            phenotype: Phenotype {
                channels: [0, 0, 93, 92, 138, 138],
                active_channel: 2,
                polarity: [true; 6],
            },
            identity: CreatureIdentityState {
                lineage_id: 3,
                kin_tag: 3_756_156_572,
            },
        }
    );
}

#[test]
fn an_unmutated_birth_copies_the_parent_body() {
    assert_eq!(child_body(0.0, 7), parent_body());
}

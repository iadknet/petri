//! Characterization of the body rules (T26.F01): the energy cap at the three
//! credit sites, the uncapped birth, the move charge and the order of the
//! body inheritance draws. Each assertion pins main's behavior before the
//! refactor, so a moved booking or draw is a defect.

use crate::creature::action_log::ActionType;
use rand::SeedableRng;
use slotmap::SlotMap;

use super::{apply_typed_eat, move_effort, resolve};
use super::{PredationActionResult, ReproductionActionResult};
use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{ActionOutcome, CreatureId, Direction, Position, WorldAction};
use crate::creature::body::HeritableBody;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::phenotype::Phenotype;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::ledger::{DeathCause, EnergyFlows};
use crate::simulation::simulation::Simulation;
use crate::simulation::tick::{reproduce_now, steal_now};

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
            {
                let mut founder = crate::creature::birth::Zygote::founder(
                    v3alpha1_founder_genome(),
                    HeritableBody::founder(3, 77, 200.0, 0.2),
                )
                .born_at(id, position, energy);
                // A fixture keeps its requested energy past the body cap.
                founder.energy = energy;
                founder
            }
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
    assert!(eaten.amount > 0.0, "the fixture eat must consume food");
    (sim, id, flows)
}

fn eat_effort(sim: &Simulation, id: CreatureId) -> f32 {
    let creature = &sim.creatures[id];
    sim.config.energy.action_effort(
        sim.config.energy.costs.eat_cost,
        creature.complexity(),
        creature.age,
    )
}

#[test]
fn eat_caps_a_credit_above_the_maximum_and_books_the_excess() {
    let (sim, id, flows) = eat_once(195.0, 40.0);
    let max = sim.creatures[id].energy_cap();
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
    let max = sim.creatures[id].energy_cap();
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
        let max = sim.creatures[id].energy_cap();
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

/// Attacker at 195 north-adjacent to a victim at `victim_energy`, stealing at
/// effort modifier `effort` (intensity `min(1, 0.2 × effort)`) with no rate
/// cost and a kill bonus of 1 per complexity unit.
fn steal(victim_energy: f32, effort: f32) -> (Simulation, CreatureId, PredationActionResult, f32) {
    let (mut sim, ids) = sim_with(&[
        (Position::new(5, 5), 195.0),
        (Position::new(5, 4), victim_energy),
    ]);
    sim.config.exchange.effort_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 1.0;
    let attacker = &sim.creatures[ids[0]];
    let charge = sim
        .config
        .energy
        .action_effort(0.0, attacker.complexity(), attacker.age);
    let outcome = steal_now(ids[0], &mut sim, Direction::N, effort)
        .result
        .outcome;
    let result = PredationActionResult::from_outcome(outcome).expect("a steal key");
    (sim, ids[0], result, 195.0 - charge)
}

#[test]
fn predation_observes_the_full_transfer_before_capping_it() {
    let (sim, attacker, result, after_effort) = steal(50.0, 1.0);
    assert_eq!(result, PredationActionResult::Transferred);
    let max = sim.creatures[attacker].energy_cap();
    let transferred = after_effort + 10.0;
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
    let (sim, attacker, result, after_effort) = steal(10.0, 5.0);
    assert_eq!(result, PredationActionResult::TransferredAndKilled);
    let max = sim.creatures[attacker].energy_cap();
    let transferred = after_effort + 10.0;
    assert!(
        transferred > max,
        "the fixture must put the first cap in play"
    );
    let bonus = sim.creatures[attacker].complexity() as f32;
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
    sim.config.exchange.effort_rate = 0.0;
    let result = steal_now(ids[0], &mut sim, Direction::N, 1.0).result;
    assert_eq!(result.outcome, ActionOutcome::Success);
    assert_eq!(sim.stats.energy_flows.maximum_energy_clamp_loss, 0.0);
}

// ── birth ────────────────────────────────────────────────────────────────────

/// T26.F02 invariant 7's required fixture: a parent at the reference (200)
/// delivers a litter of 100 to a child of capacity 50. Every fired capacity
/// draw lands on 50 in a `[50, 50]` range, so the child is built with 100,
/// capped to 50, and the overflow and the credit book 50 each; the parent's
/// debit is their sum, exactly.
#[test]
fn a_litter_above_the_childs_capacity_is_capped_and_the_overflow_booked() {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 200.0)]);
    let parent = ids[0];
    sim.config.mutation.capacity.chance = 1.0;
    sim.config.energy.lifecycle.max_energy = 50.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    assert_eq!(sim.creatures[parent].body.capacity, 200.0);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
    let result = reproduce_now(parent, &mut sim, Direction::N, 0.75, &mut rng);
    assert_eq!(result, ReproductionActionResult::Conceived);
    let child = sim
        .creatures
        .values()
        .find(|creature| creature.id != parent)
        .expect("one child");
    assert_eq!(child.body.capacity, 50.0);
    assert_eq!(child.energy, 50.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.parental_transfer_debit, 100.0);
    assert_eq!(flows.offspring_capacity_overflow, 50.0);
    assert_eq!(flows.offspring_energy_credit, 50.0);
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
}

/// Every newborn's capacity is clamped to the range in force at its birth
/// (T26.F02 invariant 2): after a live patch lowers `max_energy` to 100, the
/// next unmutated newborn of a 200 parent is born at 100 while the parent
/// keeps 200, and the cap holds each body at its own capacity.
#[test]
fn a_lowered_range_clamps_the_next_unmutated_newborn_but_not_its_parent() {
    let (mut sim, ids) = sim_with(&[(Position::new(2, 2), 200.0), (Position::new(7, 7), 200.0)]);
    let (early, late) = (ids[0], ids[1]);
    sim.config.mutation.capacity.chance = 0.0;
    for &id in &ids {
        sim.creatures[id].age = sim.config.energy.lifecycle.min_reproduce_age;
    }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
    let mut known = ids.clone();
    let mut deliver = |sim: &mut Simulation, parent: CreatureId| {
        let result = reproduce_now(parent, sim, Direction::N, 0.5, &mut rng);
        assert_eq!(result, ReproductionActionResult::Conceived);
        let newborn = sim
            .creatures
            .keys()
            .find(|id| !known.contains(id))
            .expect("one newborn");
        known.push(newborn);
        newborn
    };
    let first = deliver(&mut sim, early);
    assert_eq!(sim.creatures[first].body.capacity, 200.0);

    sim.config.energy.lifecycle.max_energy = 100.0;
    let newborn = deliver(&mut sim, late);
    assert_eq!(sim.creatures[newborn].body.capacity, 100.0);
    assert_eq!(sim.creatures[late].body.capacity, 200.0);

    for (id, cap) in [(late, 200.0), (newborn, 100.0)] {
        sim.creatures[id].energy = 1_000.0;
        assert_eq!(sim.creatures[id].cap_energy(), debit(1_000.0, cap));
        assert_eq!(sim.creatures[id].energy, cap);
    }
}

/// A litter within the child's capacity is credited whole: no overflow.
#[test]
fn a_litter_within_the_childs_capacity_books_no_overflow() {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 80.0)]);
    let parent = ids[0];
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
    let result = reproduce_now(parent, &mut sim, Direction::N, 0.5, &mut rng);
    assert_eq!(result, ReproductionActionResult::Conceived);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.offspring_capacity_overflow, 0.0);
    assert_eq!(flows.offspring_energy_credit, flows.parental_transfer_debit);
}

// ── capacity ─────────────────────────────────────────────────────────────────

/// The three credit sites cap at the body's own capacity, not at
/// `max_energy`: a body of 50 eating from 45 stops at 50.
#[test]
fn eat_caps_at_the_bodys_own_capacity() {
    let pos = Position::new(3, 3);
    let (mut sim, ids) = sim_with(&[(pos, 45.0)]);
    let id = ids[0];
    sim.creatures[id].body.capacity = 50.0;
    sim.config.world.food.types[0].energy_per_unit = Some(40.0);
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.5);
    let mut flows = EnergyFlows {
        food_intake_by_type: vec![0.0; sim.config.world.food.types.len()],
        ..EnergyFlows::default()
    };
    let eaten = apply_typed_eat(
        &mut sim.creatures[id],
        &mut sim.world,
        &sim.config,
        OrdinaryFoodTypeId::new(0),
        0.0,
        &mut flows,
    );
    assert!(eaten.amount > 0.0);
    assert_eq!(flows.maximum_energy_clamp_loss, debit(65.0, 50.0));
    assert_eq!(sim.creatures[id].energy, 50.0 - eat_effort(&sim, id));
}

/// A body away from the reference scales the move base by its move factor
/// before the multipliers: at β = 0.5 a body of 800 pays 2.5× the base.
#[test]
fn a_large_body_pays_its_move_factor_on_the_move_base() {
    let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 50.0)]);
    let id = ids[0];
    sim.creatures[id].body.capacity = 800.0;
    sim.config.energy.capacity.move_cost_slope = 0.5;
    let creature = &sim.creatures[id];
    let expected = sim.config.energy.action_effort(
        sim.config.energy.costs.move_cost * 2.5,
        creature.complexity(),
        creature.age,
    );
    assert_eq!(
        move_effort(
            &creature.body,
            &sim.config.energy,
            creature.complexity(),
            creature.age
        ),
        expected
    );
}

proptest::proptest! {
    /// Over the normalized ranges (`reference` up to 2^24, `min` down to 1,
    /// β up to 1) the move effort is positive, never below the NoOp effort,
    /// and at the reference exactly today's effort.
    #[test]
    fn move_effort_is_floored_positive_and_neutral_at_the_reference(
        reference in proptest::sample::select(vec![1.0f32, 50.0, 200.0, 16_777_216.0]),
        share in 0.0f32..=1.0,
        slope in 0.0f32..=1.0,
        move_cost in 0.0f32..=10.0,
        age in 0u64..=2_000,
    ) {
        let (mut sim, ids) = sim_with(&[(Position::new(5, 5), 50.0)]);
        let id = ids[0];
        let energy = &mut sim.config.energy;
        energy.capacity.reference = reference;
        energy.capacity.min = 1.0;
        energy.capacity.move_cost_slope = slope;
        energy.costs.move_cost = move_cost;
        energy.lifecycle.max_energy = reference * 4.0;
        let creature = &mut sim.creatures[id];
        creature.age = age;
        creature.body.capacity = 1.0 + share * (reference * 4.0 - 1.0);
        let effort = move_effort(&creature.body, &sim.config.energy, creature.complexity(), creature.age);
        let noop = sim.config.energy.action_effort(
            sim.config.energy.costs.noop_cost,
            creature.complexity(),
            creature.age,
        );
        proptest::prop_assert!(effort > 0.0);
        proptest::prop_assert!(effort >= noop);
        creature.body.capacity = reference;
        let today = sim.config.energy.action_effort(
            move_cost,
            creature.complexity(),
            creature.age,
        );
        proptest::prop_assert_eq!(
            move_effort(&creature.body, &sim.config.energy, creature.complexity(), creature.age).to_bits(),
            today.to_bits()
        );
    }
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
        creature.complexity(),
        creature.age,
    );
    let applied = resolve(
        &mut sim.action_inputs(),
        id,
        &WorldAction::Move(Direction::N),
    );
    let moved = applied.result.outcome == ActionOutcome::Success;
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
    HeritableBody::founder(3, 77, 200.0, 0.2)
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
    // The capacity coin is still drawn after the kin tag (T26.F02), but at
    // chance 0 it never fires, so the pins below read the phenotype and
    // kin-tag rules rather than whether the coin missed at this seed.
    sim.config.mutation.capacity.chance = 0.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let result = reproduce_now(parent, &mut sim, Direction::N, 0.5, &mut rng);
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
/// active 2, polarity all adding); the kin tag holds. Re-pinned by T27.F03
/// commit 3: the four-kind bundle draws, the 39 / 106 sink draws and the
/// widened decision sub-index draws change the genome mutation's draws
/// (channels `[0, 0, 93, 92, 138, 138]` -> `[0, 0, 91, 92, 138, 138]`, the
/// third polarity subtracting, kin tag 3_756_156_572 -> 3_756_156_052).
/// Re-pinned by T25.F11 commit 3: the four creation operators replace
/// `InputRef.Add`, recruitment and the four `Add*Group` (Graph weight 78,
/// InputRef 12) and every birth draws whole field sets, which changes the
/// genome mutation's draws (channels `[0, 0, 91, 92, 138, 138]` ->
/// `[1, 0, 92, 92, 138, 138]`, active 2 -> 0, polarity all adding, kin tag
/// 3_756_156_052 -> 3_756_156_044). Re-pinned by T25.F12 commit 2: the
/// learning catalog (Graph weight 78 -> 73), the target draw over edges and
/// slots and the bundle birth draws change the genome mutation's draws
/// (channels `[1, 0, 92, 92, 138, 138]` -> `[0, 0, 92, 92, 137, 138]`,
/// active 0 -> 4, the fifth polarity subtracting, kin tag 3_756_156_044 ->
/// 3_756_156_061).
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
                channels: [0, 0, 92, 92, 137, 138],
                active_channel: 4,
                polarity: [true, true, true, true, false, true],
            },
            identity: CreatureIdentityState {
                lineage_id: 3,
                kin_tag: 3_756_156_061,
            },
            capacity: 200.0,
            steal_share: 0.2,
            give_share: 0.2,
        }
    );
}

#[test]
fn an_unmutated_birth_copies_the_parent_body() {
    assert_eq!(child_body(0.0, 7), parent_body());
}

//! Solo evaluation of one actor on one scene.
//!
//! Genomes run the full production tick from a fresh `Simulation` at world
//! tick `start_tick`. Observation is at tick boundaries: after each
//! `run_tick` the cumulative counters are differenced (so a bite in the
//! death tick counts) and, while the creature is alive, its position,
//! energy, age and dispatch record are snapshotted. On death the last living
//! state is the previous snapshot. The action log is never read.
//!
//! Scripted instruments run on a lab stepper over the production appliers
//! (`apply_typed_eat`, `apply_move`) plus `energy_decay_per_tick`; they pay
//! no brain compute, carrying or penalty charge.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::SlotMap;
use v3_core::config::{OrdinaryFoodTypeId, SimulationConfig};
use v3_core::contracts::{CreatureId, Direction, Position};
use v3_core::creature::genome::CreatureGenome;
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::creature::state::{CreatureState, DispatchRecord, SHARED_MEMORY_SLOTS};
use v3_core::kernel::WorldState;
use v3_core::simulation::actions::{apply_move, apply_typed_eat};
use v3_core::simulation::energy_accounting::EnergyFlows;
use v3_core::simulation::{run_tick, seed_simulation, SimStats, Simulation};

use crate::scene::{centre, torus_distance, Scene};

const FOOD: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(0);

/// The founder phenotype triple, read from one production-seeded founder
/// (`seeding.rs` keeps the constants private).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phenotype {
    pub channels: [u8; 6],
    pub active_channel: usize,
    pub polarity: [bool; 6],
}

impl Phenotype {
    /// Read the triple from one `seed_simulation` founder of `config`.
    ///
    /// # Panics
    ///
    /// If `config` seeds no creature.
    #[must_use]
    pub fn founder(config: &SimulationConfig) -> Self {
        let mut probe = config.clone();
        probe.world.width = 8;
        probe.world.height = 8;
        probe.world.terrain.clear();
        probe.population.initial_creatures = 1;
        let sim = seed_simulation(probe, 0);
        let founder = sim.creatures.values().next().expect("one founder seeded");
        Self {
            channels: founder.phenotype_channels,
            active_channel: founder.phenotype_active_channel,
            polarity: founder.phenotype_channel_polarity,
        }
    }
}

/// What every evaluation of an arm shares.
#[derive(Debug, Clone)]
pub struct Setup {
    pub config: SimulationConfig,
    pub start_energy: f32,
    pub lifetime: u32,
    /// World tick each scene starts at: the failed-action penalty ramp's
    /// endpoint, so the penalty is at its production value.
    pub start_tick: u64,
    pub phenotype: Phenotype,
}

impl Setup {
    #[must_use]
    pub fn new(config: SimulationConfig, start_energy: f32, lifetime: u32) -> Self {
        let start_tick = config.startup.ramps.failed_action_penalty.target_tick;
        let phenotype = Phenotype::founder(&config);
        Self {
            config,
            start_energy,
            lifetime,
            start_tick,
            phenotype,
        }
    }

    fn creature(&self, id: CreatureId, genome: CreatureGenome, seed: u64) -> CreatureState {
        CreatureState::new(
            id,
            genome,
            centre(self.config.world.width),
            self.start_energy,
            0,
            self.phenotype.channels,
            self.phenotype.active_channel,
            self.phenotype.polarity,
            CreatureIdentityState::founder(0, seed),
            [0.0; SHARED_MEMORY_SLOTS],
        )
    }
}

/// Last living energy. A scripted instrument pays no compute, carrying or
/// penalty charge, so its energy serializes as the label `"scripted"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnergyEnd {
    Living(f32),
    Scripted(f32),
}

impl EnergyEnd {
    /// The numeric energy either way.
    #[must_use]
    pub fn value(self) -> f32 {
        match self {
            Self::Living(energy) | Self::Scripted(energy) => energy,
        }
    }
}

impl serde::Serialize for EnergyEnd {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Living(energy) => serializer.serialize_f32(*energy),
            Self::Scripted(_) => serializer.serialize_str("scripted"),
        }
    }
}

/// One scene's reading.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SceneScore {
    /// `food_eaten + progress`.
    pub score: f64,
    pub food_eaten: u32,
    pub intake: f64,
    /// Scene-relative tick (1-based) of the first bite.
    pub ticks_to_first_food: Option<u32>,
    pub progress: f64,
    pub moves_attempted: u64,
    pub moves_blocked: u64,
    pub penalty_charged: f64,
    /// Last living energy (`scripted` for instruments: no compute charges).
    pub energy_end: EnergyEnd,
    /// Scene-relative tick (1-based) of removal.
    pub death_tick: Option<u32>,
    /// Ticks executed, the death tick included (creature-tick accounting).
    #[serde(skip)]
    pub ticks: u32,
}

/// The parent's last living dispatch record and age, frozen for variation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frozen {
    pub record: DispatchRecord,
    pub age: u64,
}

/// Interval progress toward the nearest remaining food, observed at tick
/// boundaries only.
#[derive(Debug, Clone)]
pub struct Progress {
    remaining: Vec<Position>,
    size: u16,
    d0: u16,
    best: u16,
}

impl Progress {
    /// Open the first interval at `start`.
    #[must_use]
    pub fn new(food: &[Position], size: u16, start: Position) -> Self {
        let mut progress = Self {
            remaining: food.to_vec(),
            size,
            d0: 0,
            best: 0,
        };
        progress.open(start, |_| true);
        progress
    }

    fn nearest(&self, at: Position) -> Option<u16> {
        self.remaining
            .iter()
            .map(|&cell| torus_distance(cell, at, self.size))
            .min()
    }

    /// Open a new interval at `at` after a tick in which food was eaten;
    /// `has_food` drops consumed cells.
    pub fn open(&mut self, at: Position, has_food: impl Fn(Position) -> bool) {
        self.remaining.retain(|&cell| has_food(cell));
        let d = self.nearest(at).unwrap_or(0);
        self.d0 = d;
        self.best = d;
    }

    /// Observe a later boundary at `at`.
    pub fn observe(&mut self, at: Position) {
        if let Some(d) = self.nearest(at) {
            self.best = self.best.min(d);
        }
    }

    /// The open interval's value: `1 − min(d_t)/d0` in [0, 1], 1 when
    /// `d0 = 0`, 0 when no food remains.
    #[must_use]
    pub fn value(&self) -> f64 {
        if self.remaining.is_empty() {
            0.0
        } else if self.d0 == 0 {
            1.0
        } else {
            (1.0 - f64::from(self.best) / f64::from(self.d0)).clamp(0.0, 1.0)
        }
    }

    /// Nearest remaining food cell from `at` (first in draw order on ties).
    #[must_use]
    pub fn target(&self, at: Position) -> Option<Position> {
        self.remaining
            .iter()
            .copied()
            .min_by_key(|&cell| torus_distance(cell, at, self.size))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Counters {
    eaten: u64,
    intake: f64,
    moves: u64,
    blocked: u64,
    penalty: f64,
}

impl Counters {
    fn read(stats: &SimStats) -> Self {
        Self {
            eaten: stats.eat_actions_applied_total_by_type.values().sum(),
            intake: stats.energy_flows.food_intake_by_type.iter().sum(),
            moves: stats.move_actions_attempted_total,
            blocked: stats.move_actions_blocked_total_by_cause.values().sum(),
            penalty: stats.energy_flows.failed_action_penalty,
        }
    }
}

/// Running per-scene tallies shared by both evaluators.
#[derive(Debug, Clone, Default)]
struct Tally {
    food_eaten: u32,
    intake: f64,
    first: Option<u32>,
    moves: u64,
    blocked: u64,
    penalty: f64,
}

impl Tally {
    fn finish(
        self,
        progress: &Progress,
        energy_end: EnergyEnd,
        death: Option<u32>,
        ticks: u32,
    ) -> SceneScore {
        let progress = progress.value();
        SceneScore {
            score: f64::from(self.food_eaten) + progress,
            food_eaten: self.food_eaten,
            intake: self.intake,
            ticks_to_first_food: self.first,
            progress,
            moves_attempted: self.moves,
            moves_blocked: self.blocked,
            penalty_charged: self.penalty,
            energy_end,
            death_tick: death,
            ticks,
        }
    }
}

/// Evaluate `genome` alone on `scene` with the full production tick.
#[must_use]
pub fn evaluate_genome(
    setup: &Setup,
    genome: &CreatureGenome,
    scene: &Scene,
) -> (SceneScore, Frozen) {
    let size = setup.config.world.width;
    let start = centre(size);
    let world = WorldState::new(size, size, setup.config.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let id = creatures.insert_with_key(|id| setup.creature(id, genome.clone(), scene.seed));
    let mut sim = Simulation::new(
        world,
        creatures,
        setup.start_tick,
        setup.config.clone(),
        scene.seed,
    );
    sim.world.place_creature(start, id);
    let density = sim.config.world.food.shared.max_density;
    for &cell in &scene.food {
        sim.world.set_food_type(cell, FOOD, density);
    }

    let mut progress = Progress::new(&scene.food, size, start);
    let mut tally = Tally::default();
    let mut counters = Counters::read(&sim.stats);
    let mut energy = setup.start_energy;
    let mut frozen = Frozen::default();
    let mut death = None;
    let mut ticks = 0;
    for tick in 1..=setup.lifetime {
        run_tick(&mut sim, &mut None);
        ticks = tick;
        let now = Counters::read(&sim.stats);
        let bites = now.eaten - counters.eaten;
        tally.food_eaten += u32::try_from(bites).expect("bites per tick fit u32");
        tally.intake += now.intake - counters.intake;
        tally.moves += now.moves - counters.moves;
        tally.blocked += now.blocked - counters.blocked;
        tally.penalty += now.penalty - counters.penalty;
        counters = now;
        if bites > 0 && tally.first.is_none() {
            tally.first = Some(tick);
        }
        let Some(creature) = sim.creatures.get(id) else {
            death = Some(tick);
            break;
        };
        energy = creature.energy;
        frozen = Frozen {
            record: creature.graph_runtime.dispatch_record.clone(),
            age: creature.age,
        };
        if bites > 0 {
            let world = &sim.world;
            progress.open(creature.position, |cell| {
                world.food_at_type(cell, FOOD) > 0.0
            });
        } else {
            progress.observe(creature.position);
        }
    }
    (
        tally.finish(&progress, EnergyEnd::Living(energy), death, ticks),
        frozen,
    )
}

/// A scripted instrument's policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scripted {
    /// Eat if food here, else a uniform draw over the eight directions.
    RandomWalk,
    /// Oracle step on even ticks, random-walk step on odd.
    HalfSeeker,
    /// One step along the toroidal Chebyshev-shortest direction to the
    /// nearest remaining food.
    OracleSeeker,
}

impl Scripted {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::RandomWalk => "random-walk",
            Self::HalfSeeker => "half-seeker",
            Self::OracleSeeker => "oracle-seeker",
        }
    }

    fn direction(
        self,
        tick: u32,
        from: Position,
        target: Option<Position>,
        size: u16,
        rng: &mut SmallRng,
    ) -> Direction {
        let oracle = match self {
            Self::RandomWalk => false,
            Self::HalfSeeker => tick % 2 == 0,
            Self::OracleSeeker => true,
        };
        match target {
            Some(target) if oracle => step_toward(from, target, size),
            _ => Direction::ALL[rng.gen_range(0..Direction::ALL.len())],
        }
    }
}

fn wrapped_sign(from: u16, to: u16, size: u16) -> i32 {
    let forward = (i32::from(to) - i32::from(from)).rem_euclid(i32::from(size));
    if forward == 0 {
        0
    } else if forward <= i32::from(size) / 2 {
        1
    } else {
        -1
    }
}

/// The king step that shortens the toroidal Chebyshev distance to `to`.
#[must_use]
pub fn step_toward(from: Position, to: Position, size: u16) -> Direction {
    let delta = (
        wrapped_sign(from.x, to.x, size),
        wrapped_sign(from.y, to.y, size),
    );
    Direction::ALL
        .into_iter()
        .find(|direction| direction.delta() == delta)
        .unwrap_or(Direction::N)
}

/// Evaluate a scripted instrument on `scene`, the actor built from
/// `founder` so the production cost multipliers see a founder body.
#[must_use]
pub fn evaluate_scripted(
    setup: &Setup,
    founder: &CreatureGenome,
    policy: Scripted,
    scene: &Scene,
    actor_seed: u64,
) -> SceneScore {
    let size = setup.config.world.width;
    let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
    world.reconfigure_food(setup.config.world.food.clone());
    let density = setup.config.world.food.shared.max_density;
    for &cell in &scene.food {
        world.set_food_type(cell, FOOD, density);
    }
    run_scripted(setup, founder, policy, world, &scene.food, actor_seed)
}

/// The scripted stepper on a prepared `world` (tests place barriers here).
#[must_use]
pub fn run_scripted(
    setup: &Setup,
    founder: &CreatureGenome,
    policy: Scripted,
    mut world: WorldState,
    food: &[Position],
    actor_seed: u64,
) -> SceneScore {
    let config = &setup.config;
    let size = config.world.width;
    let mut ids: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = ids.insert(());
    let mut actor = setup.creature(id, founder.clone(), actor_seed);
    world.place_creature(actor.position, id);
    let mut flows = EnergyFlows {
        food_intake_by_type: vec![0.0; config.world.food.types.len()],
        ..EnergyFlows::default()
    };
    let mut rng = SmallRng::seed_from_u64(actor_seed);
    let mut progress = Progress::new(food, size, actor.position);
    let mut tally = Tally::default();
    let mut energy = actor.energy;
    let mut death = None;
    let mut ticks = 0;
    for tick in 1..=setup.lifetime {
        ticks = tick;
        let intake_before: f64 = flows.food_intake_by_type.iter().sum();
        let ate = world.food_at_type(actor.position, FOOD) > 0.0
            && apply_typed_eat(&mut actor, &mut world, config, FOOD, &mut flows);
        if ate {
            tally.food_eaten += 1;
            tally.first.get_or_insert(tick);
            tally.intake += flows.food_intake_by_type.iter().sum::<f64>() - intake_before;
        } else {
            let direction = policy.direction(
                tick,
                actor.position,
                progress.target(actor.position),
                size,
                &mut rng,
            );
            tally.moves += 1;
            if !apply_move(id, &mut actor, &mut world, direction, config, &mut flows) {
                tally.blocked += 1;
            }
        }
        actor.energy -= config.energy.lifecycle.energy_decay_per_tick;
        actor.age += 1;
        if actor.energy <= 0.0 {
            death = Some(tick);
            break;
        }
        energy = actor.energy;
        if ate {
            progress.open(actor.position, |cell| world.food_at_type(cell, FOOD) > 0.0);
        } else {
            progress.observe(actor.position);
        }
    }
    tally.finish(&progress, EnergyEnd::Scripted(energy), death, ticks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::arena_config;
    use crate::scene::draw_scene;
    use v3_core::creature::founder::founder_genome_with_age_gate;
    use v3_core::mutation::reachability::ParentExecuted;

    fn setup(lifetime: u32, start_energy: f32) -> Setup {
        Setup::new(arena_config(48), start_energy, lifetime)
    }

    fn founder(setup: &Setup) -> CreatureGenome {
        founder_genome_with_age_gate(
            setup.config.population.founder_profile,
            &setup.config.energy.lifecycle,
        )
    }

    fn scene(seed: u64) -> Scene {
        draw_scene(&mut SmallRng::seed_from_u64(seed), 48, 0.04, 5).unwrap()
    }

    #[test]
    fn the_comparator_builds_on_the_production_founder() {
        let setup = setup(1, 100.0);
        let founder = founder(&setup);
        let (comparator, structure) = v3_core::neighborhood::opportunity::controllers::controller(
            &founder,
            v3_core::neighborhood::opportunity::Family::Vector,
            false,
        );
        assert_ne!(comparator, founder);
        assert!(structure.edges > 0);
    }

    #[test]
    fn a_first_tick_death_freezes_an_empty_record() {
        let setup = setup(50, 0.01);
        let (score, frozen) = evaluate_genome(&setup, &founder(&setup), &scene(1));
        assert_eq!(score.death_tick, Some(1));
        assert_eq!(score.ticks, 1);
        assert_eq!(frozen, Frozen::default());
        assert_eq!(score.energy_end, EnergyEnd::Living(0.01));
    }

    #[test]
    fn a_death_freezes_the_state_after_the_previous_tick() {
        let dying = setup(400, 3.0);
        let founder = founder(&dying);
        let scene = scene(2);
        let (score, frozen) = evaluate_genome(&dying, &founder, &scene);
        let death = score.death_tick.expect("3 energy cannot last 400 ticks");
        assert!(death > 1, "death at tick {death}");
        // The same run stopped one tick earlier survives with the same state.
        let survivor = setup(death - 1, 3.0);
        let (alive, alive_frozen) = evaluate_genome(&survivor, &founder, &scene);
        assert_eq!(alive.death_tick, None);
        assert_eq!(alive_frozen, frozen);
        assert_eq!(alive.progress, score.progress);
        assert_eq!(alive.energy_end, score.energy_end);
        assert!(
            score.food_eaten >= alive.food_eaten,
            "death-tick bites count"
        );
        assert!(!ParentExecuted::Record(&frozen.record, frozen.age)
            .resolve(100)
            .is_empty());
    }

    #[test]
    fn a_survivor_freezes_the_state_after_its_final_tick() {
        let setup = setup(1, 100.0);
        let (score, frozen) = evaluate_genome(&setup, &founder(&setup), &scene(3));
        assert_eq!(score.death_tick, None);
        assert_eq!(frozen.age, 1);
        assert!(!ParentExecuted::Record(&frozen.record, frozen.age)
            .resolve(100)
            .is_empty());
    }

    #[test]
    fn scene_score_is_bites_plus_progress_in_unit_range() {
        let setup = setup(200, 100.0);
        let founder = founder(&setup);
        for seed in 0..4 {
            let (score, _) = evaluate_genome(&setup, &founder, &scene(seed));
            assert!((0.0..=1.0).contains(&score.progress));
            assert_eq!(score.score, f64::from(score.food_eaten) + score.progress);
            assert_eq!(score.food_eaten > 0, score.ticks_to_first_food.is_some());
        }
    }

    #[test]
    fn the_scripted_stepper_charges_the_production_blocked_move_cost() {
        let setup = setup(5, 100.0);
        let size = setup.config.world.width;
        let start = centre(size);
        let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
        world.reconfigure_food(setup.config.world.food.clone());
        for direction in Direction::ALL {
            let cell = world.resolve_neighbor(start, direction).unwrap();
            world.set_barrier(cell, true);
        }
        let far = Position::new(start.x + 10, start.y);
        world.set_food_type(far, FOOD, 1.0);
        let founder = founder(&setup);
        let score = run_scripted(&setup, &founder, Scripted::OracleSeeker, world, &[far], 9);
        assert_eq!(score.moves_attempted, 5);
        assert_eq!(score.moves_blocked, 5);
        let actor = setup.creature(CreatureId::default(), founder, 9);
        let lifecycle = &setup.config.energy.lifecycle;
        let per_tick = setup.config.energy.adjusted_action_cost(
            setup.config.energy.costs.move_cost,
            actor.cached_complexity,
            0,
        ) + lifecycle.energy_decay_per_tick;
        assert!(per_tick > lifecycle.energy_decay_per_tick);
        let energy = score.energy_end.value();
        assert!((100.0 - energy - 5.0 * per_tick).abs() < 1e-3, "{energy}");
        assert_eq!(
            serde_json::to_value(score.energy_end).unwrap(),
            serde_json::json!("scripted")
        );
        assert_eq!(score.progress, 0.0);
    }

    #[test]
    fn the_oracle_outscores_the_random_walk() {
        let setup = setup(200, 100.0);
        let founder = founder(&setup);
        let (mut oracle, mut random) = (0.0, 0.0);
        for seed in 0..6 {
            let scene = scene(seed);
            oracle +=
                evaluate_scripted(&setup, &founder, Scripted::OracleSeeker, &scene, seed).score;
            random += evaluate_scripted(&setup, &founder, Scripted::RandomWalk, &scene, seed).score;
        }
        assert!(oracle > random + 6.0, "oracle {oracle} random {random}");
    }

    #[test]
    fn step_toward_takes_the_wrapped_short_way() {
        assert_eq!(
            step_toward(Position::new(1, 5), Position::new(9, 5), 10),
            Direction::W
        );
        assert_eq!(
            step_toward(Position::new(5, 5), Position::new(7, 3), 10).delta(),
            (1, -1)
        );
    }

    #[test]
    fn progress_is_graded_and_resets_on_a_bite() {
        let size = 32;
        let start = Position::new(16, 16);
        let food = [Position::new(20, 16)];
        let mut progress = Progress::new(&food, size, start);
        assert_eq!(progress.value(), 0.0);
        progress.observe(Position::new(18, 16));
        assert!((progress.value() - 0.5).abs() < 1e-12);
        progress.observe(Position::new(10, 16));
        assert!(
            (progress.value() - 0.5).abs() < 1e-12,
            "min distance is kept"
        );
        progress.open(Position::new(20, 16), |_| false);
        assert_eq!(progress.value(), 0.0, "no food remains");
    }
}

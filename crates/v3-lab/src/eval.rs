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
//!
//! Every genome evaluation runs an expression-masked copy ([`expressed`]):
//! lifetime learning is off, so improvement across generations is inherited
//! change.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::SlotMap;
use v3_core::config::{OrdinaryFoodTypeId, SimulationConfig};
use v3_core::contracts::{CreatureId, Direction, Position};
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::creature::state::{CreatureState, DispatchRecord, SHARED_MEMORY_SLOTS};
use v3_core::kernel::WorldState;
use v3_core::simulation::actions::{apply_move, apply_typed_eat};
use v3_core::simulation::energy_accounting::EnergyFlows;
use v3_core::simulation::{run_tick, seed_simulation, SimStats, Simulation};

use crate::geodesic::{step_toward, Field, Terrain};
use crate::scene::{nearest, Scene};

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
    pub scoring: Scoring,
}

/// What an interval opened after a bite scores once no reachable food
/// remains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exhausted {
    /// F01's rule: no reachable food is no progress.
    Zero,
    /// The task is complete: the interval scores 1.
    Complete,
}

/// The per-assay scene-score weights and exhausted-interval rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scoring {
    /// Weight of the blocked-move fraction subtracted from the score.
    pub blocked_weight: f64,
    /// Weight of the first-bite efficiency added to the score.
    pub efficiency_weight: f64,
    pub exhausted: Exhausted,
}

impl Scoring {
    /// F01's food-seeking scene score: bites plus progress.
    pub const BITES_AND_PROGRESS: Self = Self {
        blocked_weight: 0.0,
        efficiency_weight: 0.0,
        exhausted: Exhausted::Zero,
    };

    /// Barrier navigation: blocked moves and first-bite efficiency weighed
    /// at 1.0 (wasting every move forfeits one bite, the shortest path earns
    /// one), an exhausted block scored as complete.
    pub const BARRIER_NAVIGATION: Self = Self {
        blocked_weight: 1.0,
        efficiency_weight: 1.0,
        exhausted: Exhausted::Complete,
    };
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
            scoring: Scoring::BITES_AND_PROGRESS,
        }
    }

    /// The same setup scored by `scoring`.
    #[must_use]
    pub fn with_scoring(mut self, scoring: Scoring) -> Self {
        self.scoring = scoring;
        self
    }

    fn creature(
        &self,
        id: CreatureId,
        genome: CreatureGenome,
        seed: u64,
        position: Position,
    ) -> CreatureState {
        CreatureState::new(
            id,
            genome,
            position,
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
    /// `food_eaten + progress + efficiency_weight × efficiency −
    /// blocked_weight × blocked_fraction`.
    pub score: f64,
    pub food_eaten: u32,
    pub intake: f64,
    /// Scene-relative tick (1-based) of the first bite.
    pub ticks_to_first_food: Option<u32>,
    pub progress: f64,
    /// `min(1, d_start / ticks_to_first_food)`, 0 with no bite. Not a row
    /// field (`row_version` 1).
    #[serde(skip)]
    pub efficiency: f64,
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

impl SceneScore {
    /// `moves_blocked / moves_attempted`, 0 when no move was attempted.
    #[must_use]
    pub fn blocked_fraction(&self) -> f64 {
        blocked_fraction(self.moves_blocked, self.moves_attempted)
    }
}

/// `min(1, d_start / first)` for a first bite on scene tick `first`
/// (1-based) with `d_start` the start's geodesic distance to food; 0 with
/// no bite. The shortest path bites on tick `d_start + 1`.
#[must_use]
pub fn efficiency(d_start: Option<u32>, first: Option<u32>) -> f64 {
    match (d_start, first) {
        (Some(d_start), Some(first)) => (f64::from(d_start) / f64::from(first)).min(1.0),
        _ => 0.0,
    }
}

#[allow(clippy::cast_precision_loss)]
fn blocked_fraction(blocked: u64, attempted: u64) -> f64 {
    if attempted == 0 {
        0.0
    } else {
        blocked as f64 / attempted as f64
    }
}

/// `genome` with lifetime learning masked: every Graph compute node's
/// `plasticity` set to `None` (the only lifetime-learning mechanism).
#[must_use]
pub fn expressed(genome: &CreatureGenome) -> CreatureGenome {
    let mut masked = genome.clone();
    for node in &mut masked.nodes {
        if let BackendDef::Graph(graph) = &mut node.backend_def {
            for compute in &mut graph.compute_nodes {
                compute.plasticity = None;
            }
        }
    }
    masked
}

/// The parent's last living dispatch record and age, frozen for variation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frozen {
    pub record: DispatchRecord,
    pub age: u64,
}

/// Interval progress toward the nearest remaining food along the shortest
/// passable path, observed at tick boundaries only. The distance field is
/// recomputed when an interval opens and read once per tick.
#[derive(Debug, Clone)]
pub struct Progress {
    remaining: Vec<Position>,
    terrain: Terrain,
    field: Field,
    /// `None` when the nearest remaining food is unreachable.
    d0: Option<u32>,
    best: u32,
    /// The first interval's `d0`.
    d_start: Option<u32>,
    opened: Opened,
}

/// When the open interval began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opened {
    AtStart,
    AfterBite,
}

impl Progress {
    /// Open the first interval at `start`.
    #[must_use]
    pub fn new(food: &[Position], terrain: Terrain, start: Position) -> Self {
        let field = Field::new(&terrain, food);
        let d0 = field.at(start);
        Self {
            remaining: food.to_vec(),
            terrain,
            field,
            d0,
            best: d0.unwrap_or(0),
            d_start: d0,
            opened: Opened::AtStart,
        }
    }

    /// The scene start's geodesic distance to the nearest food.
    #[must_use]
    pub fn d_start(&self) -> Option<u32> {
        self.d_start
    }

    /// Open a new interval at `at` after a tick in which food was eaten;
    /// `has_food` drops consumed cells.
    pub fn open(&mut self, at: Position, has_food: impl Fn(Position) -> bool) {
        self.remaining.retain(|&cell| has_food(cell));
        self.field = Field::new(&self.terrain, &self.remaining);
        self.d0 = self.field.at(at);
        self.best = self.d0.unwrap_or(0);
        self.opened = Opened::AfterBite;
    }

    /// Observe a later boundary at `at`.
    pub fn observe(&mut self, at: Position) {
        if let Some(d) = self.field.at(at) {
            self.best = self.best.min(d);
        }
    }

    /// The open interval's value: `1 − min(d_t)/d0` in [0, 1], 1 when
    /// `d0 = 0`. With no reachable food left it is 0, except that an
    /// interval opened after a bite scores 1 under [`Exhausted::Complete`].
    #[must_use]
    pub fn value(&self, exhausted: Exhausted) -> f64 {
        match self.d0 {
            None if self.opened == Opened::AfterBite && exhausted == Exhausted::Complete => 1.0,
            None => 0.0,
            Some(0) => 1.0,
            Some(d0) => (1.0 - f64::from(self.best) / f64::from(d0)).clamp(0.0, 1.0),
        }
    }

    /// Nearest remaining food cell from `at` by Chebyshev distance (first in
    /// draw order on ties): F01's target.
    #[must_use]
    pub fn target(&self, at: Position) -> Option<Position> {
        nearest(&self.remaining, at, self.terrain.size())
    }

    /// The path-aware oracle step from `at`: to a neighbour one step nearer
    /// along the shortest passable path, `step_toward` the target when that
    /// neighbour qualifies, else the first qualifying direction in
    /// `Direction::ALL` order. `None` when no food remains or none is
    /// reachable.
    #[must_use]
    pub fn oracle_step(&self, at: Position) -> Option<Direction> {
        let d = self.field.at(at)?;
        let toward = step_toward(at, self.target(at)?, self.terrain.size());
        if d == 0 {
            return Some(toward);
        }
        let descends = |direction: Direction| {
            self.field.at(self.terrain.neighbor(at, direction)) == Some(d - 1)
        };
        if descends(toward) {
            return Some(toward);
        }
        Direction::ALL
            .into_iter()
            .find(|&direction| descends(direction))
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
        scoring: Scoring,
        energy_end: EnergyEnd,
        death: Option<u32>,
        ticks: u32,
    ) -> SceneScore {
        let efficiency = efficiency(progress.d_start(), self.first);
        let progress = progress.value(scoring.exhausted);
        let blocked = blocked_fraction(self.blocked, self.moves);
        SceneScore {
            score: f64::from(self.food_eaten) + progress + scoring.efficiency_weight * efficiency
                - scoring.blocked_weight * blocked,
            food_eaten: self.food_eaten,
            intake: self.intake,
            ticks_to_first_food: self.first,
            progress,
            efficiency,
            moves_attempted: self.moves,
            moves_blocked: self.blocked,
            penalty_charged: self.penalty,
            energy_end,
            death_tick: death,
            ticks,
        }
    }
}

/// A world of the setup's size holding `scene`'s barriers.
fn scene_world(setup: &Setup, scene: &Scene) -> WorldState {
    let size = setup.config.world.width;
    let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
    for &cell in &scene.barriers {
        world.set_barrier(cell, true);
    }
    world
}

/// Evaluate the expression-masked copy of `genome` alone on `scene` with the
/// full production tick.
///
/// # Panics
///
/// If the masked copy records a plasticity update.
#[must_use]
pub fn evaluate_genome(
    setup: &Setup,
    genome: &CreatureGenome,
    scene: &Scene,
) -> (SceneScore, Frozen) {
    let size = setup.config.world.width;
    let start = scene.start;
    let world = scene_world(setup, scene);
    let terrain = Terrain::from_cells(size, &scene.barriers);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let id =
        creatures.insert_with_key(|id| setup.creature(id, expressed(genome), scene.seed, start));
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

    let mut progress = Progress::new(&scene.food, terrain, start);
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
    assert_eq!(
        sim.stats.plasticity_updates_total, 0,
        "an expression-masked genome learned"
    );
    (
        tally.finish(
            &progress,
            setup.scoring,
            EnergyEnd::Living(energy),
            death,
            ticks,
        ),
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
    /// One step along the shortest passable path to the nearest remaining
    /// reachable food (F01's Chebyshev step on a barrier-free arena).
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

    /// The oracle step on oracle ticks while reachable food remains, else
    /// one random-walk draw.
    fn direction(
        self,
        tick: u32,
        from: Position,
        progress: &Progress,
        rng: &mut SmallRng,
    ) -> Direction {
        let oracle = match self {
            Self::RandomWalk => false,
            Self::HalfSeeker => tick.is_multiple_of(2),
            Self::OracleSeeker => true,
        };
        oracle
            .then(|| progress.oracle_step(from))
            .flatten()
            .unwrap_or_else(|| Direction::ALL[rng.gen_range(0..Direction::ALL.len())])
    }
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
    let mut world = scene_world(setup, scene);
    world.reconfigure_food(setup.config.world.food.clone());
    let density = setup.config.world.food.shared.max_density;
    for &cell in &scene.food {
        world.set_food_type(cell, FOOD, density);
    }
    run_scripted(
        setup,
        founder,
        policy,
        world,
        scene.start,
        &scene.food,
        actor_seed,
    )
}

/// The scripted stepper on a prepared `world` whose barriers are set,
/// starting at `start`.
#[must_use]
pub fn run_scripted(
    setup: &Setup,
    founder: &CreatureGenome,
    policy: Scripted,
    mut world: WorldState,
    start: Position,
    food: &[Position],
    actor_seed: u64,
) -> SceneScore {
    let config = &setup.config;
    let mut ids: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = ids.insert(());
    let mut actor = setup.creature(id, founder.clone(), actor_seed, start);
    let terrain = Terrain::from_world(&world);
    world.place_creature(actor.position, id);
    let mut flows = EnergyFlows {
        food_intake_by_type: vec![0.0; config.world.food.types.len()],
        ..EnergyFlows::default()
    };
    let mut rng = SmallRng::seed_from_u64(actor_seed);
    let mut progress = Progress::new(food, terrain, actor.position);
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
            let direction = policy.direction(tick, actor.position, &progress, &mut rng);
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
    tally.finish(
        &progress,
        setup.scoring,
        EnergyEnd::Scripted(energy),
        death,
        ticks,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::arena_config;
    use crate::scene::{centre, SceneSpec};
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
        SceneSpec::sparse(48, 0.04, 5)
            .draw(&mut SmallRng::seed_from_u64(seed))
            .unwrap()
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
            "bites never decrease; a_bite_in_the_death_tick_counts_exactly pins the death-tick bite"
        );
        assert!(!ParentExecuted::Record(&frozen.record, frozen.age)
            .resolve(100)
            .is_empty());
    }

    #[test]
    fn a_bite_in_the_death_tick_counts_exactly() {
        // One bite on the start cell worth 1e-3 energy: at 0.6 start energy
        // the founder eats on tick 1 and still dies in that tick (below
        // about 0.5 it dies before acting; from 0.75 it survives).
        const REWARD: f32 = 1e-3;
        let mut dying = setup(5, 0.6);
        dying.config.world.food.types[0].energy_per_unit = Some(REWARD);
        let scene = Scene::open(4, 48, vec![centre(48)]);
        let (score, frozen) = evaluate_genome(&dying, &founder(&dying), &scene);
        assert_eq!(score.death_tick, Some(1));
        assert_eq!(score.ticks, 1);
        assert_eq!(frozen, Frozen::default());
        assert_eq!(score.food_eaten, 1, "the death-tick bite counts once");
        assert_eq!(score.ticks_to_first_food, Some(1));
        let bite = f64::from(dying.config.world.food.shared.max_density * REWARD);
        assert!(
            (score.intake - bite).abs() < 1e-7,
            "intake {} is one bite {bite}",
            score.intake
        );
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
        let score = run_scripted(
            &setup,
            &founder,
            Scripted::OracleSeeker,
            world,
            start,
            &[far],
            9,
        );
        assert_eq!(score.moves_attempted, 5);
        assert_eq!(score.moves_blocked, 5);
        let actor = setup.creature(CreatureId::default(), founder, 9, start);
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
    fn scripted_names_are_their_serialized_labels() {
        for policy in [
            Scripted::RandomWalk,
            Scripted::HalfSeeker,
            Scripted::OracleSeeker,
        ] {
            assert_eq!(serde_json::to_value(policy).unwrap(), policy.name());
        }
    }

    /// A scripted actor boxed in by barriers pays the aged move cost and
    /// decay every tick, and no eat charge off food (even when eating costs).
    #[test]
    fn a_boxed_scripted_actor_pays_the_age_scaled_move_cost_only() {
        const LIFETIME: u32 = 150;
        let mut setup = setup(LIFETIME, 200.0);
        setup.config.energy.costs.eat_cost = 0.7;
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
        let score = run_scripted(
            &setup,
            &founder,
            Scripted::OracleSeeker,
            world,
            start,
            &[far],
            9,
        );
        assert_eq!(score.death_tick, None);
        let complexity = setup
            .creature(CreatureId::default(), founder, 9, start)
            .cached_complexity;
        let energy_config = &setup.config.energy;
        let mut expected = 200.0_f32;
        for age in 0..u64::from(LIFETIME) {
            expected -=
                energy_config.adjusted_action_cost(energy_config.costs.move_cost, complexity, age);
            expected -= energy_config.lifecycle.energy_decay_per_tick;
        }
        let energy = score.energy_end.value();
        assert!((energy - expected).abs() < 1e-3, "{energy} vs {expected}");
    }

    #[test]
    fn scripted_intake_is_each_bite_once() {
        let setup = setup(9, 100.0);
        let size = setup.config.world.width;
        let start = centre(size);
        let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
        world.reconfigure_food(setup.config.world.food.clone());
        let cells: Vec<Position> = (0..3)
            .map(|i| Position::new(start.x + i, start.y))
            .collect();
        for (cell, density) in cells.iter().zip([1.0, 0.5, 0.25]) {
            world.set_food_type(*cell, FOOD, density);
        }
        let reward = setup.config.energy.costs.eat_reward_per_food;
        let score = run_scripted(
            &setup,
            &founder(&setup),
            Scripted::OracleSeeker,
            world,
            start,
            &cells,
            3,
        );
        assert_eq!(score.food_eaten, 3);
        assert_eq!(score.ticks_to_first_food, Some(1));
        let expected = f64::from(1.75 * reward);
        assert!((score.intake - expected).abs() < 1e-5, "{}", score.intake);
    }

    /// Tallies read from the run's cumulative counters, the first-bite tick
    /// and the boundary progress, replayed beside `evaluate_genome`.
    fn replay(setup: &Setup, genome: &CreatureGenome, scene: &Scene) -> SceneScore {
        let size = setup.config.world.width;
        let start = scene.start;
        let world = WorldState::new(size, size, setup.config.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id =
            creatures.insert_with_key(|id| setup.creature(id, genome.clone(), scene.seed, start));
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
        let initial = Counters::read(&sim.stats);
        let mut before = initial.eaten;
        let mut progress = Progress::new(&scene.food, Terrain::from_cells(size, &[]), start);
        let mut first = None;
        let mut ticks = 0;
        let mut death = None;
        for tick in 1..=setup.lifetime {
            run_tick(&mut sim, &mut None);
            ticks = tick;
            let eaten = Counters::read(&sim.stats).eaten;
            let bit = eaten > before;
            before = eaten;
            if bit && first.is_none() {
                first = Some(tick);
            }
            let Some(creature) = sim.creatures.get(id) else {
                death = Some(tick);
                break;
            };
            if bit {
                let world = &sim.world;
                progress.open(creature.position, |cell| {
                    world.food_at_type(cell, FOOD) > 0.0
                });
            } else {
                progress.observe(creature.position);
            }
        }
        let end = Counters::read(&sim.stats);
        Tally {
            food_eaten: u32::try_from(end.eaten - initial.eaten).unwrap(),
            intake: end.intake - initial.intake,
            first,
            moves: end.moves - initial.moves,
            blocked: end.blocked - initial.blocked,
            penalty: end.penalty - initial.penalty,
        }
        .finish(
            &progress,
            Scoring::BITES_AND_PROGRESS,
            EnergyEnd::Living(0.0),
            death,
            ticks,
        )
    }

    #[test]
    fn genome_tallies_match_the_runs_cumulative_counters() {
        let setup = setup(200, 100.0);
        let founder = founder(&setup);
        let (mut bites, mut penalty, mut partial) = (0, 0.0, false);
        for seed in 0..4 {
            let scene = scene(seed);
            let (score, _) = evaluate_genome(&setup, &founder, &scene);
            let expected = replay(&setup, &founder, &scene);
            let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
            assert_eq!(score.food_eaten, expected.food_eaten, "seed {seed}");
            assert!(close(score.intake, expected.intake), "seed {seed}");
            assert_eq!(score.ticks_to_first_food, expected.ticks_to_first_food);
            assert_eq!(score.moves_attempted, expected.moves_attempted);
            assert_eq!(score.moves_blocked, expected.moves_blocked);
            assert!(close(score.penalty_charged, expected.penalty_charged));
            assert_eq!(score.progress, expected.progress, "seed {seed}");
            assert_eq!(score.death_tick, expected.death_tick);
            bites += score.food_eaten;
            penalty += score.penalty_charged;
            partial |= score.progress > 0.0 && score.progress < 1.0;
        }
        assert!(
            bites > 1 && penalty > 0.0 && partial,
            "{bites} {penalty} {partial}"
        );
    }

    /// Off the lab arena (a bounded 8² world) the founder walks into the
    /// edge; blocked moves are tallied like the other counters.
    #[test]
    fn blocked_moves_match_the_runs_cumulative_counter() {
        let mut config = arena_config(8);
        config.world.edge_mode = v3_core::config::WorldEdgeMode::Bounded;
        let setup = Setup::new(config, 100.0, 200);
        let founder = founder(&setup);
        let mut blocked = 0;
        for seed in 0..4 {
            let scene = Scene::open(seed, 8, vec![Position::new(0, 0)]);
            let (score, _) = evaluate_genome(&setup, &founder, &scene);
            assert_eq!(
                score.moves_blocked,
                replay(&setup, &founder, &scene).moves_blocked
            );
            blocked += score.moves_blocked;
        }
        assert!(blocked > 0, "no blocked move");
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
        let mut progress = Progress::new(&food, Terrain::from_cells(size, &[]), start);
        assert_eq!(progress.value(Exhausted::Zero), 0.0);
        progress.observe(Position::new(18, 16));
        assert!((progress.value(Exhausted::Zero) - 0.5).abs() < 1e-12);
        progress.observe(Position::new(10, 16));
        assert!(
            (progress.value(Exhausted::Zero) - 0.5).abs() < 1e-12,
            "min distance is kept"
        );
        progress.open(Position::new(20, 16), |_| false);
        assert_eq!(progress.value(Exhausted::Zero), 0.0, "no food remains");
    }

    /// Walk the oracle from `start`: every step descends the field by one
    /// and the walk stands on food after exactly `dist(start)` steps.
    fn oracle_descends(
        food: &[Position],
        barriers: &[Position],
        start: Position,
        size: u16,
    ) -> u32 {
        let terrain = Terrain::from_cells(size, barriers);
        let progress = Progress::new(food, terrain.clone(), start);
        let field = Field::new(&terrain, food);
        let total = field.at(start).expect("reachable");
        let mut at = start;
        for step in 0..total {
            let next = terrain.neighbor(at, progress.oracle_step(at).expect("food reachable"));
            assert_eq!(
                field.at(next),
                Some(total - step - 1),
                "step {step} from {at:?}"
            );
            at = next;
        }
        assert!(food.contains(&at));
        total
    }

    #[test]
    fn the_oracle_detours_around_a_wall_by_one_per_step() {
        let wall: Vec<Position> = (5..=15).map(|y| Position::new(10, y)).collect();
        let food = [Position::new(12, 10)];
        assert_eq!(oracle_descends(&food, &wall, Position::new(8, 10), 32), 12);
    }

    #[test]
    fn the_oracle_takes_the_wrapped_path() {
        let wall: Vec<Position> = (0..16).map(|y| Position::new(1, y)).collect();
        let food = [Position::new(0, 0)];
        assert_eq!(oracle_descends(&food, &wall, Position::new(2, 0), 16), 14);
    }

    #[test]
    fn the_oracle_passes_diagonally_between_corner_barriers() {
        let barriers = [Position::new(5, 4), Position::new(4, 5)];
        let food = [Position::new(6, 6)];
        assert_eq!(
            oracle_descends(&food, &barriers, Position::new(3, 3), 16),
            3
        );
        let progress = Progress::new(
            &food,
            Terrain::from_cells(16, &barriers),
            Position::new(4, 4),
        );
        assert_eq!(
            progress.oracle_step(Position::new(4, 4)),
            Some(Direction::SE)
        );
    }

    /// With only unreachable food the oracle and the half-seeker take the
    /// random-walk step with its one draw every tick.
    #[test]
    fn unreachable_food_falls_back_to_the_random_walk() {
        let setup = setup(60, 100.0);
        let size = setup.config.world.width;
        let start = centre(size);
        let food = Position::new(start.x + 10, start.y);
        let world = || {
            let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
            world.reconfigure_food(setup.config.world.food.clone());
            for direction in Direction::ALL {
                world.set_barrier(world.resolve_neighbor(food, direction).unwrap(), true);
            }
            world.set_food_type(food, FOOD, 1.0);
            world
        };
        let founder = founder(&setup);
        let run = |policy| run_scripted(&setup, &founder, policy, world(), start, &[food], 5);
        let random = run(Scripted::RandomWalk);
        assert!(random.moves_attempted == 60 && random.progress == 0.0);
        assert_eq!(run(Scripted::OracleSeeker), random);
        assert_eq!(run(Scripted::HalfSeeker), random);
        let progress = Progress::new(&[food], Terrain::from_world(&world()), start);
        assert_eq!(progress.target(start), Some(food));
        assert_eq!(progress.oracle_step(start), None);
    }

    #[test]
    fn blocked_moves_cost_their_weighted_fraction() {
        let base = setup(5, 100.0);
        let size = base.config.world.width;
        let start = centre(size);
        let far = Position::new(start.x + 10, start.y);
        let boxed = |weight, seed| {
            let setup = base.clone().with_scoring(Scoring {
                blocked_weight: weight,
                ..Scoring::BITES_AND_PROGRESS
            });
            let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
            world.reconfigure_food(setup.config.world.food.clone());
            for direction in Direction::ALL {
                if direction != Direction::E {
                    world.set_barrier(world.resolve_neighbor(start, direction).unwrap(), true);
                }
            }
            world.set_food_type(far, FOOD, 1.0);
            let founder = founder(&setup);
            run_scripted(
                &setup,
                &founder,
                Scripted::RandomWalk,
                world,
                start,
                &[far],
                seed,
            )
        };
        // A seed whose walk is blocked on some moves but not all.
        let seed = (0..64)
            .find(|&seed| {
                let score = boxed(0.0, seed);
                score.moves_blocked > 0 && score.moves_blocked < score.moves_attempted
            })
            .expect("a partly blocked walk");
        let free = boxed(0.0, seed);
        let weighted = boxed(2.5, seed);
        let fraction = free.moves_blocked as f64 / free.moves_attempted as f64;
        assert_eq!(free.blocked_fraction(), fraction);
        assert_eq!(free.score, f64::from(free.food_eaten) + free.progress);
        assert_eq!(weighted.score, free.score - 2.5 * fraction);
    }

    const BARRIER: Scoring = Scoring {
        blocked_weight: 1.0,
        efficiency_weight: 1.0,
        exhausted: Exhausted::Complete,
    };

    #[test]
    fn barrier_navigation_scores_exhausted_blocks_and_efficiency() {
        assert_eq!(crate::scene::Assay::BarrierNavigation.scoring(), BARRIER);
        assert_eq!(
            crate::scene::Assay::FoodSeeking.scoring(),
            Scoring::BITES_AND_PROGRESS
        );
    }

    /// The oracle eats the only food cell at `start + 10` on tick 11; the
    /// rest of the lifetime has no reachable food.
    fn exhausted_block(scoring: Scoring) -> SceneScore {
        let setup = setup(30, 100.0).with_scoring(scoring);
        let size = setup.config.world.width;
        let start = centre(size);
        let food = Position::new(start.x + 10, start.y);
        let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
        world.reconfigure_food(setup.config.world.food.clone());
        world.set_food_type(food, FOOD, 1.0);
        run_scripted(
            &setup,
            &founder(&setup),
            Scripted::OracleSeeker,
            world,
            start,
            &[food],
            7,
        )
    }

    #[test]
    fn an_exhausted_block_is_complete_under_barrier_navigation_only() {
        let barrier = exhausted_block(BARRIER);
        let food_seeking = exhausted_block(Scoring::BITES_AND_PROGRESS);
        for score in [&barrier, &food_seeking] {
            assert_eq!(score.food_eaten, 1);
            assert_eq!(score.ticks_to_first_food, Some(11));
            assert_eq!(score.death_tick, None);
        }
        assert_eq!(barrier.progress, 1.0);
        assert_eq!(food_seeking.progress, 0.0);
    }

    #[test]
    fn the_oracle_scores_d_start_over_d_start_plus_one() {
        let score = exhausted_block(BARRIER);
        assert_eq!(score.efficiency, 10.0 / 11.0);
        assert_eq!(
            score.score,
            1.0 + 1.0 + 10.0 / 11.0 - score.blocked_fraction()
        );
        assert_eq!(score.blocked_fraction(), 0.0);
        // Food seeking weighs efficiency at 0 but still reports it.
        let food_seeking = exhausted_block(Scoring::BITES_AND_PROGRESS);
        assert_eq!(food_seeking.efficiency, 10.0 / 11.0);
        assert_eq!(food_seeking.score, 1.0);
        let row = serde_json::to_value(&food_seeking).unwrap();
        assert!(row.get("efficiency").is_none(), "not a row field");
    }

    #[test]
    fn progress_unreachable_from_the_start_stays_zero_even_when_complete() {
        let food = [Position::new(5, 5)];
        let ring: Vec<Position> = Direction::ALL
            .into_iter()
            .map(|d| {
                let (dx, dy) = d.delta();
                Position::new(
                    u16::try_from(5 + dx).unwrap(),
                    u16::try_from(5 + dy).unwrap(),
                )
            })
            .collect();
        let mut progress =
            Progress::new(&food, Terrain::from_cells(16, &ring), Position::new(12, 12));
        assert_eq!(progress.d_start(), None);
        assert_eq!(progress.value(Exhausted::Complete), 0.0);
        progress.observe(Position::new(10, 10));
        assert_eq!(progress.value(Exhausted::Complete), 0.0);
        // A bite that leaves only unreachable food completes the task.
        let reachable = [Position::new(12, 13), Position::new(5, 5)];
        let mut progress = Progress::new(
            &reachable,
            Terrain::from_cells(16, &ring),
            Position::new(12, 12),
        );
        assert_eq!(progress.d_start(), Some(1));
        progress.open(Position::new(12, 13), |cell| cell == Position::new(5, 5));
        assert_eq!(progress.value(Exhausted::Complete), 1.0);
        assert_eq!(progress.value(Exhausted::Zero), 0.0);
    }

    #[test]
    fn efficiency_is_zero_without_a_bite_and_capped_at_one() {
        assert_eq!(efficiency(Some(5), None), 0.0);
        assert_eq!(efficiency(None, None), 0.0);
        assert_eq!(efficiency(Some(5), Some(3)), 1.0, "faster than d_start");
        assert_eq!(efficiency(Some(5), Some(5)), 1.0);
        assert_eq!(efficiency(Some(5), Some(6)), 5.0 / 6.0);
        assert_eq!(efficiency(Some(0), Some(1)), 0.0);
        // A boxed actor never bites.
        let setup = setup(5, 100.0).with_scoring(BARRIER);
        let size = setup.config.world.width;
        let start = centre(size);
        let mut world = WorldState::new(size, size, setup.config.world.edge_mode);
        world.reconfigure_food(setup.config.world.food.clone());
        for direction in Direction::ALL {
            world.set_barrier(world.resolve_neighbor(start, direction).unwrap(), true);
        }
        let far = Position::new(start.x + 10, start.y);
        world.set_food_type(far, FOOD, 1.0);
        let score = run_scripted(
            &setup,
            &founder(&setup),
            Scripted::OracleSeeker,
            world,
            start,
            &[far],
            9,
        );
        assert_eq!((score.food_eaten, score.efficiency), (0, 0.0));
        assert_eq!(score.score, -1.0, "every move blocked, nothing else");
    }

    /// The oracle walks `D` steps on just enough energy and bites on tick
    /// `D + 1`, the tick it dies in.
    #[test]
    fn a_death_tick_first_bite_counts_toward_efficiency() {
        const D: u16 = 6;
        let mut probe = setup(40, 100.0).with_scoring(BARRIER);
        probe.config.world.food.types[0].energy_per_unit = Some(1e-6);
        let founder = founder(&probe);
        let size = probe.config.world.width;
        let start = centre(size);
        let food = Position::new(start.x + D, start.y);
        let complexity = probe
            .creature(CreatureId::default(), founder.clone(), 3, start)
            .cached_complexity;
        let energy = &probe.config.energy;
        let walk: f32 = (0..u64::from(D))
            .map(|age| {
                energy.adjusted_action_cost(energy.costs.move_cost, complexity, age)
                    + energy.lifecycle.energy_decay_per_tick
            })
            .sum();
        let decay = energy.lifecycle.energy_decay_per_tick;
        probe.start_energy = walk + 0.25 * decay;
        let mut world = WorldState::new(size, size, probe.config.world.edge_mode);
        world.reconfigure_food(probe.config.world.food.clone());
        world.set_food_type(food, FOOD, 1.0);
        let score = run_scripted(
            &probe,
            &founder,
            Scripted::OracleSeeker,
            world,
            start,
            &[food],
            3,
        );
        let bite = u32::from(D) + 1;
        assert_eq!(score.death_tick, Some(bite));
        assert_eq!(score.food_eaten, 1);
        assert_eq!(score.ticks_to_first_food, Some(bite));
        assert_eq!(score.efficiency, f64::from(D) / f64::from(bite));
    }

    proptest::proptest! {
        #[test]
        fn efficiency_is_a_unit_ratio_reaching_one_at_the_shortest_path(
            d_start in proptest::option::of(0u32..10_000),
            first in proptest::option::of(1u32..10_000),
        ) {
            let value = efficiency(d_start, first);
            proptest::prop_assert!((0.0..=1.0).contains(&value));
            match (d_start, first) {
                (Some(d), Some(t)) if d > 0 => {
                    proptest::prop_assert_eq!(value == 1.0, t <= d);
                    proptest::prop_assert!(value > 0.0);
                }
                _ => proptest::prop_assert_eq!(value, 0.0),
            }
        }
    }

    fn plastic(genome: &CreatureGenome) -> CreatureGenome {
        let mut plastic = genome.clone();
        for node in &mut plastic.nodes {
            if let BackendDef::Graph(graph) = &mut node.backend_def {
                for compute in &mut graph.compute_nodes {
                    compute.plasticity = Some(v3_core::creature::genome::PlasticityConfig {
                        rule: v3_core::creature::genome::HebbianRule::Classic,
                        learning_rate: 0.5,
                        weight_clamp: 5.0,
                        lamarckian: false,
                        modulation: None,
                    });
                }
            }
        }
        plastic
    }

    /// Plasticity updates of `genome` run unmasked for the setup's lifetime.
    fn unmasked_updates(setup: &Setup, genome: &CreatureGenome, scene: &Scene) -> u64 {
        let size = setup.config.world.width;
        let world = WorldState::new(size, size, setup.config.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id = creatures
            .insert_with_key(|id| setup.creature(id, genome.clone(), scene.seed, scene.start));
        let mut sim = Simulation::new(
            world,
            creatures,
            setup.start_tick,
            setup.config.clone(),
            scene.seed,
        );
        sim.world.place_creature(scene.start, id);
        let density = sim.config.world.food.shared.max_density;
        for &cell in &scene.food {
            sim.world.set_food_type(cell, FOOD, density);
        }
        for _ in 0..setup.lifetime {
            run_tick(&mut sim, &mut None);
        }
        sim.stats.plasticity_updates_total
    }

    #[test]
    fn a_plastic_genome_evaluates_masked_and_keeps_its_plasticity() {
        let setup = setup(50, 100.0);
        let genome = plastic(&founder(&setup));
        let scene = scene(1);
        assert!(
            unmasked_updates(&setup, &genome, &scene) > 0,
            "unmasked, the genome learns"
        );
        let stored = genome.clone();
        // Would panic on a plasticity update; scores as its masked copy.
        let (score, _) = evaluate_genome(&setup, &genome, &scene);
        assert_eq!(
            score,
            evaluate_genome(&setup, &expressed(&genome), &scene).0
        );
        assert_eq!(genome, stored);
        assert_ne!(expressed(&genome), genome);
        assert_eq!(expressed(&genome), founder(&setup));
    }

    #[test]
    fn a_barrier_scene_starts_at_its_start_among_its_barriers() {
        let setup = setup(1, 100.0);
        let start = Position::new(10, 10);
        let scene = Scene {
            seed: 1,
            start,
            food: vec![Position::new(14, 10)],
            barriers: vec![Position::new(12, 10)],
            redraws: 0,
        };
        let world = scene_world(&setup, &scene);
        assert!(world.is_barrier(Position::new(12, 10)));
        let (score, _) = evaluate_genome(&setup, &founder(&setup), &scene);
        assert_eq!(score.ticks, 1);
        // One barrier on the straight line costs no step: a king path
        // bends diagonally past it.
        let progress = Progress::new(&scene.food, Terrain::from_world(&world), start);
        assert_eq!(progress.field.at(start), Some(4));
        assert_eq!(progress.oracle_step(start), Some(Direction::E));
        assert_eq!(
            progress.oracle_step(Position::new(11, 10)),
            Some(Direction::NE)
        );
    }

    proptest::proptest! {
        #[test]
        fn a_barrier_free_oracle_step_is_step_toward(
            size in 3u16..40,
            food in proptest::collection::vec((0u16..40, 0u16..40), 1..5),
            at in (0u16..40, 0u16..40),
        ) {
            let food: Vec<Position> = food
                .into_iter()
                .map(|(x, y)| Position::new(x % size, y % size))
                .collect();
            let at = Position::new(at.0 % size, at.1 % size);
            let progress = Progress::new(&food, Terrain::from_cells(size, &[]), at);
            let target = progress.target(at).unwrap();
            proptest::prop_assert_eq!(
                progress.oracle_step(at),
                Some(step_toward(at, target, size))
            );
        }
    }
}

//! One competition replicate: eight arms share one world at native costs
//! with mutation off, and each arm's births, living count, extinction tick
//! and exposure are read as the run goes. The arm of a creature is its
//! founder's `lineage_id mod 8`, carried to every descendant by inheritance
//! (`CreatureIdentityState::inherit`), never re-derived from a genome.

use std::collections::BTreeSet;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::genome::CreatureGenome;
use crate::creature::state::CreatureState;
use crate::runtime::mesh::UntracedMeshExecution;
use crate::simulation::{assemble_full_sensor_inputs, run_tick, seed_simulation, Simulation};

use super::super::battery::{execute_scenario_tick, Scenario};
use super::super::input_use::ablated;
use super::controllers::{controller, Family};
use super::verdict::Exposure;

/// Run seed of replicate `r` in world `w`: `RUN_SEED_BASE + 1_000 w + r`.
pub const RUN_SEED_BASE: u64 = 26_000_000;
/// Incumbent draw seed: `INCUMBENT_SEED_BASE + goal_seed`.
pub const INCUMBENT_SEED_BASE: u64 = 27_000_000;
/// Exposure draw seed: `EXPOSURE_SEED_BASE + 16 (run_seed - RUN_SEED_BASE) + tick / 100`.
pub const EXPOSURE_SEED_BASE: u64 = 29_000_000;
pub const INCUMBENTS: usize = 20;
pub const SAMPLE_EVERY: u64 = 100;
pub const EXPOSURE_SAMPLE: usize = 32;

/// The eight arms, in assignment order: the k-th seeded creature carries arm
/// `k mod 8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Arm {
    Founder,
    Incumbent,
    Authored(Family),
    Inert(Family),
}

impl Arm {
    pub const ALL: [Self; 8] = [
        Self::Founder,
        Self::Incumbent,
        Self::Authored(Family::Ring),
        Self::Inert(Family::Ring),
        Self::Authored(Family::Vector),
        Self::Inert(Family::Vector),
        Self::Authored(Family::Scalar),
        Self::Inert(Family::Scalar),
    ];

    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&arm| arm == self)
            .expect("every arm is in the catalog")
    }

    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Founder => "F".to_string(),
            Self::Incumbent => "I".to_string(),
            Self::Authored(family) => format!("A_{}", family.as_key()),
            Self::Inert(family) => format!("Z_{}", family.as_key()),
        }
    }

    /// The controller family of an `A` or `Z` arm.
    #[must_use]
    pub fn family(self) -> Option<Family> {
        match self {
            Self::Authored(family) | Self::Inert(family) => Some(family),
            Self::Founder | Self::Incumbent => None,
        }
    }

    /// The arm a creature of lineage `lineage_id` belongs to.
    #[must_use]
    pub fn of_lineage(lineage_id: u32) -> Self {
        Self::ALL[lineage_id as usize % Self::ALL.len()]
    }
}

/// The genome each arm seeds with.
#[derive(Debug, Clone)]
pub struct ArmGenomes {
    pub founder: CreatureGenome,
    /// In draw order; empty means the founder fallback.
    pub incumbents: Vec<CreatureGenome>,
    pub authored: [CreatureGenome; 3],
    pub inert: [CreatureGenome; 3],
}

impl ArmGenomes {
    /// The controllers built on `founder`, with `incumbents`.
    #[must_use]
    pub fn new(founder: CreatureGenome, incumbents: Vec<CreatureGenome>) -> Self {
        let build = |inert| Family::ALL.map(|family| controller(&founder, family, inert).0);
        Self {
            authored: build(false),
            inert: build(true),
            founder,
            incumbents,
        }
    }

    /// Whether `I` fell back to the founder genome.
    #[must_use]
    pub fn founder_fallback(&self) -> bool {
        self.incumbents.is_empty()
    }

    fn genome(&self, arm: Arm, incumbent_turn: usize) -> &CreatureGenome {
        match arm {
            Arm::Founder => &self.founder,
            Arm::Incumbent => self
                .incumbents
                .get(incumbent_turn % self.incumbents.len().max(1))
                .unwrap_or(&self.founder),
            Arm::Authored(family) => &self.authored[family.index() as usize],
            Arm::Inert(family) => &self.inert[family.index() as usize],
        }
    }
}

/// Up to [`INCUMBENTS`] living genomes of `sim`, drawn uniformly without
/// replacement from the id-sorted population with
/// `SmallRng::seed_from_u64(INCUMBENT_SEED_BASE + goal_seed)`, in draw order.
#[must_use]
pub fn draw_incumbents(sim: &Simulation, goal_seed: u64) -> Vec<CreatureGenome> {
    let ids = sorted_ids(sim, |_| true);
    let amount = INCUMBENTS.min(ids.len());
    if amount == 0 {
        return Vec::new();
    }
    let mut rng = SmallRng::seed_from_u64(INCUMBENT_SEED_BASE + goal_seed);
    rand::seq::index::sample(&mut rng, ids.len(), amount)
        .into_iter()
        .map(|position| sim.creatures[ids[position]].genome.clone())
        .collect()
}

fn sorted_ids(sim: &Simulation, keep: impl Fn(&CreatureState) -> bool) -> Vec<CreatureId> {
    let mut ids: Vec<CreatureId> = sim
        .creatures
        .iter()
        .filter(|(_, creature)| keep(creature))
        .map(|(id, _)| id)
        .collect();
    ids.sort();
    ids
}

/// Seed `config` at `run_seed` and give the k-th seeded creature, in
/// creature order, arm `k mod 8`: each creature is rebuilt around its arm's
/// genome so every cached genome reading is its own.
///
/// # Panics
///
/// If seeding did not number founders by creature order.
#[must_use]
pub fn seed_arms(config: &SimulationConfig, run_seed: u64, genomes: &ArmGenomes) -> Simulation {
    let mut sim = seed_simulation(config.clone(), run_seed);
    let ids: Vec<CreatureId> = sim.creatures.keys().collect();
    let mut incumbent_turn = 0;
    for (k, id) in ids.into_iter().enumerate() {
        let seeded = &sim.creatures[id];
        assert_eq!(
            seeded.identity.lineage_id as usize, k,
            "founders number in creature order"
        );
        let arm = Arm::of_lineage(seeded.identity.lineage_id);
        let genome = genomes.genome(arm, incumbent_turn).clone();
        if arm == Arm::Incumbent {
            incumbent_turn += 1;
        }
        let rebuilt = CreatureState::new(
            id,
            genome,
            seeded.position,
            seeded.energy,
            seeded.generation,
            seeded.phenotype_channels,
            seeded.phenotype_active_channel,
            seeded.phenotype_channel_polarity,
            seeded.identity,
            seeded.shared_memory,
        );
        sim.creatures[id] = rebuilt;
    }
    sim
}

/// One arm's reading in one replicate.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArmRun {
    pub founders: u32,
    /// Births whose parent belongs to the arm, cumulative over the run.
    pub births: u64,
    /// Living creatures of the arm at every `SAMPLE_EVERY`-th tick.
    pub living: Vec<u32>,
    pub extinction_tick: Option<u64>,
    /// `A`/`Z` arms only: pooled exposure samples.
    pub exposure: Exposure,
}

/// One replicate's reading.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicateRun {
    pub run_seed: u64,
    pub ticks: u64,
    /// Indexed by [`Arm::index`].
    pub arms: Vec<ArmRun>,
    /// Every birth of the run (`reproduction_actions_spawned_total`).
    pub births_total: u64,
    /// Newborns that died within their birth tick: their parent's arm is
    /// unobservable without a production seam, so they count in no arm.
    /// `births_total` is every arm's births plus these.
    pub unattributed_births: u64,
}

fn arm_of(creature: &CreatureState) -> Arm {
    Arm::of_lineage(creature.identity.lineage_id)
}

/// The exposure of `family`'s arm creatures `ids` at this tick: `exposed`
/// when any authored channel of the production snapshot (assembled
/// unconditionally) reads nonzero; `applied` when ablating those channels
/// changes the committed actions of one execution from fresh cognition
/// state.
fn expose(sim: &Simulation, family: Family, ids: &[CreatureId]) -> Exposure {
    let channels = family.authored_channels();
    let reference = family.reference();
    let mut exposure = Exposure::default();
    for (id, sensors) in assemble_full_sensor_inputs(sim, ids) {
        let creature = &sim.creatures[id];
        exposure.sampled += 1;
        let resolve_ctx = crate::runtime::inputs::ResolveCtx {
            sensors: &sensors,
            upstream_slots: &[0.0; crate::runtime::OUTPUT_SLOT_COUNT],
            energy: creature.energy,
            energy_consumed: 0.0,
            action_queue: &crate::contracts::ActionQueue::new(1),
            votes: &[0.0; crate::creature::genome::vote::VOTE_SINK_COUNT],
            previous_pass_votes: &[0.0; crate::creature::genome::vote::VOTE_SINK_COUNT],
            commit_counts: &[0; crate::creature::genome::vote::VOTE_KIND_COUNT],
            mesh_hops: 0,
        };
        let exposed = family.authored_sub_indices().iter().any(|&sub| {
            crate::runtime::inputs::resolve_input(&reference, sub, &resolve_ctx) != 0.0
        });
        exposure.exposed += u64::from(exposed);
        let scenario = Scenario {
            sensors,
            energy: creature.energy,
        };
        let runtime = &sim.config.runtime;
        let run = |genome: &CreatureGenome| {
            execute_scenario_tick(genome, &scenario, runtime, UntracedMeshExecution).actions
        };
        let silenced = ablated(&creature.genome, |channel| channels.contains(&channel));
        exposure.applied += u64::from(run(&creature.genome) != run(&silenced));
    }
    exposure
}

/// Run one replicate for `horizon` ticks. `config` must already carry the
/// replicate's settings (mutation off); `exposure` draws the samples.
#[must_use]
pub fn run_replicate(
    config: &SimulationConfig,
    run_seed: u64,
    genomes: &ArmGenomes,
    horizon: u64,
) -> ReplicateRun {
    let mut sim = seed_arms(config, run_seed, genomes);
    let mut arms = vec![ArmRun::default(); Arm::ALL.len()];
    for creature in sim.creatures.values() {
        arms[arm_of(creature).index()].founders += 1;
    }
    let mut alive: BTreeSet<CreatureId> = sim.creatures.keys().collect();
    let spawned_at_seed = sim.stats.reproduction_actions_spawned_total;
    let mut spawned = spawned_at_seed;
    let mut unattributed_births = 0;
    let mut ticks = 0;
    for tick in 1..=horizon {
        run_tick(&mut sim, &mut None);
        ticks = tick;
        let spawned_now = sim.stats.reproduction_actions_spawned_total;
        let mut living = [0u32; 8];
        let mut attributed = 0;
        for (id, creature) in &sim.creatures {
            let arm = arm_of(creature).index();
            living[arm] += 1;
            if !alive.contains(&id) {
                arms[arm].births += 1;
                attributed += 1;
            }
        }
        unattributed_births += (spawned_now - spawned) - attributed;
        spawned = spawned_now;
        alive = sim.creatures.keys().collect();
        for (arm, count) in arms.iter_mut().zip(living) {
            if count == 0 && arm.extinction_tick.is_none() {
                arm.extinction_tick = Some(tick);
            }
        }
        if tick % SAMPLE_EVERY == 0 {
            let mut rng = SmallRng::seed_from_u64(
                EXPOSURE_SEED_BASE + 16 * (run_seed - RUN_SEED_BASE) + tick / SAMPLE_EVERY,
            );
            for (index, arm) in Arm::ALL.into_iter().enumerate() {
                arms[index].living.push(living[index]);
                let Some(family) = arm.family() else {
                    continue;
                };
                let members = sorted_ids(&sim, |creature| arm_of(creature) == arm);
                let amount = EXPOSURE_SAMPLE.min(members.len());
                if amount == 0 {
                    continue;
                }
                let sampled: Vec<CreatureId> =
                    rand::seq::index::sample(&mut rng, members.len(), amount)
                        .into_iter()
                        .map(|position| members[position])
                        .collect();
                arms[index].exposure = arms[index].exposure.merge(expose(&sim, family, &sampled));
            }
        }
        if sim.creatures.is_empty() {
            break;
        }
    }
    // An extinct world holds no creature at every later sampling tick.
    let samples = (horizon / SAMPLE_EVERY) as usize;
    for arm in &mut arms {
        arm.living.resize(samples, 0);
    }
    ReplicateRun {
        run_seed,
        ticks,
        arms,
        births_total: spawned - spawned_at_seed,
        unattributed_births,
    }
}

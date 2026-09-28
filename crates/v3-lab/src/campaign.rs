//! The campaign: per replicate, every arm is scored each generation on that
//! generation's shared training scenes; evolving arms vary by the
//! production engine and select by truncation with elites.

use std::collections::{BTreeMap, HashMap};

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use serde::Serialize;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::CreatureGenome;
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::{MutationEngine, MutationSummary};

use crate::arena::{Policy, Role};
use crate::calibration::scripted_seed;
use crate::eval::{evaluate_genome, evaluate_scripted, Frozen, SceneScore, Scripted, Setup};
use crate::output::RunDir;
use crate::rng::{hash, replicate_seed, stream, tagged, Part};
use crate::scene::{draw_scene, Scene};
use crate::summary::{ArmSummary, Events, Fidelity, ReplicateResult, StoppedBy};
use crate::{GenomeFile, LabError};

/// NDJSON row schema version.
pub const ROW_VERSION: u32 = 1;

/// How an arm produces its individuals.
#[derive(Debug, Clone)]
pub enum ArmKind {
    /// Founder (or genome-file) start, production variation, truncation.
    Evolving {
        start: CreatureGenome,
        /// Permute the scalars by the selection stream before truncation.
        shuffled: bool,
    },
    /// One genome scored every generation; no variation, no selection.
    Fixed(CreatureGenome),
    /// One scripted actor scored every generation on a founder body.
    Scripted {
        policy: Scripted,
        body: CreatureGenome,
    },
}

/// One arm of the run.
#[derive(Debug, Clone)]
pub struct Arm {
    pub name: String,
    pub role: Role,
    pub policy: Policy,
    pub setup: Setup,
    pub kind: ArmKind,
}

impl Arm {
    /// The reach test runs for every non-instrument arm.
    #[must_use]
    pub fn reach_tested(&self) -> bool {
        self.role != Role::Instrument
    }

    /// Its reach counts as native reachability only on a `native` policy; a
    /// `policy-deviation` arm's reach is a diagnostic.
    #[must_use]
    pub fn reach_reported(&self) -> bool {
        self.reach_tested() && self.policy == Policy::Native
    }
}

/// Campaign sizes.
#[derive(Debug, Clone, Copy)]
pub struct Plan {
    pub seed: u64,
    pub replicates: u32,
    pub generations: u32,
    pub population: u32,
    pub elite_fraction: f64,
    pub scenes: u32,
    pub food_fraction: f64,
    pub threshold: f64,
}

impl Plan {
    /// `max(1, floor(elite_fraction × population))`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn survivors(&self) -> usize {
        ((self.elite_fraction * f64::from(self.population)).floor() as usize).max(1)
    }
}

fn add_by_operator<K: Serialize>(into: &mut BTreeMap<String, u64>, from: &HashMap<K, u32>) {
    for (operator, count) in from {
        let key = match serde_json::to_value(operator) {
            Ok(serde_json::Value::String(name)) => name,
            Ok(other) => other.to_string(),
            Err(_) => "unknown".into(),
        };
        *into.entry(key).or_default() += u64::from(*count);
    }
}

/// One birth's events.
fn birth_events(summary: &MutationSummary, identical: bool) -> Events {
    let mut events = Events {
        requested: u64::from(summary.attempted_events),
        applied: u64::from(summary.applied_events),
        skipped: u64::from(summary.skipped_events),
        offspring: 1,
        identical_offspring: u64::from(identical),
        ..Events::default()
    };
    add_by_operator(
        &mut events.requested_by_operator,
        &summary.attempted_by_operator,
    );
    add_by_operator(
        &mut events.applied_by_operator,
        &summary.applied_by_operator,
    );
    add_by_operator(
        &mut events.skipped_by_operator,
        &summary.skipped_by_operator,
    );
    events
}

#[derive(Debug, Clone)]
struct Individual {
    id: u64,
    parent: Option<u64>,
    /// `None` for scripted actors.
    genome: Option<CreatureGenome>,
    /// `None` for founders and carried elites.
    birth: Option<Events>,
    carried: bool,
}

#[derive(Debug, Clone)]
struct Scored {
    scalar: f64,
    scenes: Vec<SceneScore>,
    /// Frozen from the last training scene.
    frozen: Frozen,
    creature_ticks: u64,
}

/// Evaluate one individual on the training scenes. The frozen record is the
/// last scene's: after its final tick on survival, after the tick before
/// death otherwise.
fn score_individual(
    arm: &Arm,
    individual: &Individual,
    scenes: &[Scene],
    scripted_parent: u64,
    first_scene: usize,
) -> Scored {
    let mut results = Vec::with_capacity(scenes.len());
    let mut frozen = Frozen::default();
    let mut creature_ticks = 0;
    for (offset, scene) in scenes.iter().enumerate() {
        if let ArmKind::Scripted { policy, body } = &arm.kind {
            let seed = scripted_seed(scripted_parent, *policy, first_scene + offset);
            results.push(evaluate_scripted(&arm.setup, body, *policy, scene, seed));
        } else {
            let genome = individual
                .genome
                .as_ref()
                .expect("genome arms carry genomes");
            let (score, last) = evaluate_genome(&arm.setup, genome, scene);
            creature_ticks += u64::from(score.ticks);
            frozen = last;
            results.push(score);
        }
    }
    let values: Vec<f64> = results.iter().map(|s| s.score).collect();
    Scored {
        scalar: crate::stats::mean(&values).unwrap_or(0.0),
        scenes: results,
        frozen,
        creature_ticks,
    }
}

/// Per-arm, per-replicate state.
#[derive(Debug, Clone)]
struct Lineage {
    arm: usize,
    population: Vec<Individual>,
    selection: SmallRng,
    mutation_seed: u64,
    next_id: u64,
    generations_run: u32,
    reached: Option<u32>,
    stopped: Option<StoppedBy>,
    final_best: Option<f64>,
    /// Best individual of the last evaluated generation (genome arms).
    elite: Option<CreatureGenome>,
}

impl Lineage {
    fn new(arm_index: usize, arm: &Arm, population: u32, replicate: u64) -> Self {
        let (members, genome) = match &arm.kind {
            ArmKind::Evolving { start, .. } => (population, Some(start)),
            ArmKind::Fixed(genome) => (1, Some(genome)),
            ArmKind::Scripted { .. } => (1, None),
        };
        let population = (0..u64::from(members))
            .map(|id| Individual {
                id,
                parent: None,
                genome: genome.cloned(),
                birth: None,
                carried: false,
            })
            .collect();
        // Every arm draws the replicate's `selection` and `mutation` streams
        // from the same seeds, so adding an arm never moves another's.
        Self {
            arm: arm_index,
            population,
            selection: SmallRng::seed_from_u64(tagged(replicate, "selection")),
            mutation_seed: tagged(replicate, "mutation"),
            next_id: u64::from(members),
            generations_run: 0,
            reached: None,
            stopped: None,
            final_best: None,
            elite: None,
        }
    }

    fn active(&self) -> bool {
        self.stopped.is_none()
    }
}

#[derive(Debug, Clone, Serialize)]
struct RowFidelity {
    #[serde(flatten)]
    events: Events,
    identical_offspring_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct EliteShape {
    genome_size: u32,
    reachable_nodes: usize,
    executed_nodes: usize,
}

/// `[id, parent_id, requested, applied, identical, carried, scalar]`.
type MemberRow = (u64, Option<u64>, u64, u64, bool, bool, f64);

#[derive(Debug, Clone, Serialize)]
struct Row<'a> {
    row_version: u32,
    arm: &'a str,
    role: Role,
    policy: Policy,
    replicate: u32,
    generation: u32,
    scene_seeds: Vec<u64>,
    best: f64,
    median: f64,
    mean: f64,
    best_scenes: &'a [SceneScore],
    carried_over: u32,
    fidelity: RowFidelity,
    /// `[id, parent_id, requested, applied, identical, carried, scalar]`.
    individuals: Vec<MemberRow>,
    elite: Option<EliteShape>,
    validation_mean: Option<f64>,
}

/// Campaign totals the summary needs.
#[derive(Debug, Clone, Default)]
pub struct Totals {
    pub creature_ticks: u64,
    pub byte_cap_hit: bool,
    /// Reference-arm births over the campaign.
    pub reference_events: Events,
    pub reference_carried: u64,
}

/// Run every replicate, writing rows under the byte cap. `arms[0]` is the
/// reference arm.
///
/// # Errors
///
/// I/O failures, or [`LabError::Config`] when a campaign scene is
/// infeasible.
pub fn run_campaign(
    arms: &[Arm],
    plan: &Plan,
    validation: &[Scene],
    dir: &mut RunDir,
) -> Result<(Vec<ArmSummary>, Totals), LabError> {
    let mut totals = Totals::default();
    let mut results: Vec<Vec<ReplicateResult>> = vec![Vec::new(); arms.len()];
    for replicate in 0..plan.replicates {
        if totals.byte_cap_hit {
            for arm_results in &mut results {
                arm_results.push(incomplete(replicate, StoppedBy::NotStarted, 0, None));
            }
            continue;
        }
        let lineages = run_replicate(arms, plan, validation, replicate, dir, &mut totals)?;
        for lineage in &lineages {
            results[lineage.arm].push(replicate_result(&arms[lineage.arm], lineage, replicate));
        }
    }
    let summaries = arms
        .iter()
        .zip(results)
        .map(|(arm, replicates)| arm_summary(arm, replicates))
        .collect();
    Ok((summaries, totals))
}

fn run_replicate(
    arms: &[Arm],
    plan: &Plan,
    validation: &[Scene],
    replicate: u32,
    dir: &mut RunDir,
    totals: &mut Totals,
) -> Result<Vec<Lineage>, LabError> {
    let r_seed = replicate_seed(plan.seed, replicate);
    let reference = &arms[0].setup.config;
    let size = reference.world.width;
    let vision = reference.runtime.perception.vision_radius;
    let scene_count = plan.scenes as usize;
    let mut scene_rng = stream(&[Part::U(tagged(r_seed, "scenes"))]);
    let mut lineages: Vec<Lineage> = arms
        .iter()
        .enumerate()
        .map(|(index, arm)| Lineage::new(index, arm, plan.population, r_seed))
        .collect();
    for generation in 0..plan.generations {
        if lineages.iter().all(|l| !l.active()) {
            break;
        }
        let scenes: Vec<Scene> = (0..scene_count)
            .map(|_| draw_scene(&mut scene_rng, size, plan.food_fraction, vision))
            .collect::<Result<_, _>>()
            .map_err(|_| {
                LabError::Config("a campaign scene failed the exposure predicate 100 times".into())
            })?;
        let first_scene = generation as usize * scene_count;
        let jobs: Vec<(usize, usize)> = lineages
            .iter()
            .enumerate()
            .filter(|(_, l)| l.active())
            .flat_map(|(li, l)| (0..l.population.len()).map(move |ii| (li, ii)))
            .collect();
        let scored: Vec<Scored> = jobs
            .par_iter()
            .map(|&(li, ii)| {
                let lineage = &lineages[li];
                let arm = &arms[lineage.arm];
                score_individual(arm, &lineage.population[ii], &scenes, r_seed, first_scene)
            })
            .collect();
        totals.creature_ticks += scored.iter().map(|s| s.creature_ticks).sum::<u64>();
        let mut scored = scored.into_iter();
        let step = Step {
            plan,
            validation,
            scenes: &scenes,
            replicate,
            generation,
        };
        for lineage in lineages.iter_mut().filter(|l| l.active()) {
            let members: Vec<Scored> = scored.by_ref().take(lineage.population.len()).collect();
            if !advance(&arms[lineage.arm], lineage, members, &step, dir, totals)? {
                totals.byte_cap_hit = true;
                break;
            }
        }
        if totals.byte_cap_hit {
            for lineage in lineages.iter_mut().filter(|l| l.active()) {
                lineage.stopped = Some(StoppedBy::ByteCap);
            }
            return Ok(lineages);
        }
    }
    for lineage in &mut lineages {
        lineage.stopped.get_or_insert(StoppedBy::Horizon);
        if totals.byte_cap_hit {
            lineage.stopped = Some(StoppedBy::ByteCap);
            continue;
        }
        if let Some(elite) = &lineage.elite {
            let bytes =
                serde_json::to_vec(&GenomeFile::new(elite.clone())).expect("genome serializes");
            let name = format!("{}-{replicate}", arms[lineage.arm].name);
            if !dir.write_elite(&name, &bytes)? {
                totals.byte_cap_hit = true;
                lineage.stopped = Some(StoppedBy::ByteCap);
            }
        }
    }
    Ok(lineages)
}

fn incomplete(
    replicate: u32,
    stopped_by: StoppedBy,
    generations_run: u32,
    final_best: Option<f64>,
) -> ReplicateResult {
    ReplicateResult {
        replicate,
        reached: None,
        generation_to_threshold: None,
        censored: None,
        incomplete: true,
        stopped_by,
        generations_run,
        final_best,
    }
}

fn replicate_result(arm: &Arm, lineage: &Lineage, replicate: u32) -> ReplicateResult {
    let stopped_by = lineage.stopped.unwrap_or(StoppedBy::Horizon);
    if stopped_by == StoppedBy::ByteCap {
        return incomplete(
            replicate,
            stopped_by,
            lineage.generations_run,
            lineage.final_best,
        );
    }
    let reach = arm.reach_tested();
    ReplicateResult {
        replicate,
        reached: reach.then_some(lineage.reached.is_some()),
        generation_to_threshold: lineage.reached.filter(|_| reach),
        censored: reach.then_some(lineage.reached.is_none()),
        incomplete: false,
        stopped_by,
        generations_run: lineage.generations_run,
        final_best: lineage.final_best,
    }
}

fn arm_summary(arm: &Arm, replicates: Vec<ReplicateResult>) -> ArmSummary {
    let count = |keep: fn(&ReplicateResult) -> bool| {
        u32::try_from(replicates.iter().filter(|r| keep(r)).count()).unwrap_or(u32::MAX)
    };
    let incomplete = count(|r| r.incomplete);
    let completed = count(|r| !r.incomplete);
    let reached = count(|r| r.reached == Some(true));
    let reportable = arm.reach_tested() && incomplete == 0 && completed > 0;
    ArmSummary {
        name: arm.name.clone(),
        role: arm.role,
        policy: arm.policy,
        reach_reported: arm.reach_reported(),
        replicates,
        reached_fraction: reportable.then(|| f64::from(reached) / f64::from(completed)),
        wilson_95: crate::stats::wilson_95(reached, completed).filter(|_| reportable),
        incomplete_replicates: incomplete,
    }
}

/// Rank by `scalars` descending, ties broken by `keys`.
fn rank(scalars: &[f64], keys: &[u64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..scalars.len()).collect();
    order.sort_by(|&a, &b| {
        scalars[b]
            .total_cmp(&scalars[a])
            .then(keys[a].cmp(&keys[b]))
    });
    order
}

/// What one generation step of every lineage shares.
struct Step<'a> {
    plan: &'a Plan,
    validation: &'a [Scene],
    scenes: &'a [Scene],
    replicate: u32,
    generation: u32,
}

/// Rank, test reach, write the row and build the next generation. Returns
/// `false` when the byte cap refused the row.
fn advance(
    arm: &Arm,
    lineage: &mut Lineage,
    members: Vec<Scored>,
    step: &Step<'_>,
    dir: &mut RunDir,
    totals: &mut Totals,
) -> Result<bool, LabError> {
    let scalars: Vec<f64> = members.iter().map(|m| m.scalar).collect();
    let keys: Vec<u64> = (0..members.len())
        .map(|_| lineage.selection.gen())
        .collect();
    let true_order = rank(&scalars, &keys);
    let best = &members[true_order[0]];
    let candidate = &lineage.population[true_order[0]];

    // Reach: the top-ranked individual by its true training scalar, then a
    // validation evaluation on separate `Simulation`s whose records are
    // dropped (validation never feeds reproduction).
    let mut validation_mean = None;
    if arm.reach_tested() && best.scalar >= step.plan.threshold {
        if let Some(genome) = &candidate.genome {
            let (values, ticks): (Vec<f64>, Vec<u64>) = step
                .validation
                .par_iter()
                .map(|scene| {
                    let score = evaluate_genome(&arm.setup, genome, scene).0;
                    (score.score, u64::from(score.ticks))
                })
                .unzip();
            totals.creature_ticks += ticks.iter().sum::<u64>();
            validation_mean = crate::stats::mean(&values);
            if validation_mean.is_some_and(|m| m >= step.plan.threshold) {
                lineage.reached = Some(step.generation);
            }
        }
    }

    let mut events = Events::default();
    for birth in lineage.population.iter().filter_map(|i| i.birth.as_ref()) {
        events.add(birth);
    }
    let carried_over = lineage.population.iter().filter(|i| i.carried).count();
    let row = Row {
        row_version: ROW_VERSION,
        arm: &arm.name,
        role: arm.role,
        policy: arm.policy,
        replicate: step.replicate,
        generation: step.generation,
        scene_seeds: step.scenes.iter().map(|s| s.seed).collect(),
        best: best.scalar,
        median: crate::stats::median(&scalars).unwrap_or(0.0),
        mean: crate::stats::mean(&scalars).unwrap_or(0.0),
        best_scenes: &best.scenes,
        carried_over: u32::try_from(carried_over).unwrap_or(u32::MAX),
        fidelity: RowFidelity {
            identical_offspring_fraction: events.identical_offspring_fraction(),
            events: events.clone(),
        },
        individuals: lineage
            .population
            .iter()
            .zip(&members)
            .map(|(i, m)| {
                let (requested, applied, identical) = i.birth.as_ref().map_or((0, 0, false), |b| {
                    (b.requested, b.applied, b.identical_offspring > 0)
                });
                (
                    i.id, i.parent, requested, applied, identical, i.carried, m.scalar,
                )
            })
            .collect(),
        elite: candidate.genome.as_ref().map(|genome| EliteShape {
            genome_size: genome.genome_size(),
            reachable_nodes: mesh_reachable_nodes(genome).len(),
            executed_nodes: ParentExecuted::Record(&best.frozen.record, best.frozen.age)
                .resolve(arm.setup.config.mutation.executed_window_ticks)
                .len(),
        }),
        validation_mean,
    };
    let line = serde_json::to_string(&row).expect("row serializes");
    if !dir.write_row(&line)? {
        lineage.reached = None;
        return Ok(false);
    }
    if lineage.arm == 0 {
        totals.reference_events.add(&events);
        totals.reference_carried += carried_over as u64;
    }
    lineage.generations_run = step.generation + 1;
    lineage.final_best = Some(best.scalar);
    lineage.elite.clone_from(&candidate.genome);
    if lineage.reached.is_some() {
        lineage.stopped = Some(StoppedBy::Reached);
        return Ok(true);
    }
    if let ArmKind::Evolving { shuffled, .. } = arm.kind {
        if step.generation + 1 < step.plan.generations {
            let order = if shuffled {
                let mut permuted = scalars;
                permuted.shuffle(&mut lineage.selection);
                rank(&permuted, &keys)
            } else {
                true_order
            };
            breed(arm, lineage, &members, &order, step);
        }
    }
    Ok(true)
}

/// Truncation with elites: the first `survivors` of `order` carry over
/// unchanged; the rest are mutants of parents drawn uniformly with
/// replacement from the survivors, each passed once through the engine
/// with the parent's frozen record.
fn breed(arm: &Arm, lineage: &mut Lineage, members: &[Scored], order: &[usize], step: &Step<'_>) {
    let survivors = step.plan.survivors().min(members.len());
    let mut population: Vec<Individual> = order[..survivors]
        .iter()
        .map(|&index| Individual {
            birth: None,
            carried: true,
            ..lineage.population[index].clone()
        })
        .collect();
    let slots = lineage.population.len() - survivors;
    let picks: Vec<usize> = (0..slots)
        .map(|_| lineage.selection.gen_range(0..survivors))
        .collect();
    let next_generation = u64::from(step.generation) + 1;
    let config = &arm.setup.config;
    let mutation_seed = lineage.mutation_seed;
    let parents = &population;
    // Survivors are drawn repeatedly; compute each one's reachable set once.
    let reachable: Vec<_> = parents
        .iter()
        .map(|parent| {
            mesh_reachable_nodes(parent.genome.as_ref().expect("evolving arms carry genomes"))
        })
        .collect();
    let children: Vec<Individual> = picks
        .par_iter()
        .enumerate()
        .map(|(offset, &pick)| {
            let parent = &parents[pick];
            let frozen = &members[order[pick]].frozen;
            let parent_genome = parent.genome.as_ref().expect("evolving arms carry genomes");
            let mut child = parent_genome.clone();
            let mut rng = SmallRng::seed_from_u64(hash(&[
                Part::U(mutation_seed),
                Part::U(next_generation),
                Part::U((survivors + offset) as u64),
            ]));
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut child,
                &config.mutation,
                &reachable[pick],
                ParentExecuted::Record(&frozen.record, frozen.age),
                &mut rng,
                config.world.food.types.len(),
            );
            let identical = child == *parent_genome;
            Individual {
                id: 0,
                parent: Some(parent.id),
                genome: Some(child),
                birth: Some(birth_events(&summary, identical)),
                carried: false,
            }
        })
        .collect();
    for mut child in children {
        child.id = lineage.next_id;
        lineage.next_id += 1;
        population.push(child);
    }
    lineage.population = population;
}

/// The reference arm's fidelity block over the whole campaign.
#[must_use]
pub fn fidelity(reference: &Arm, totals: &Totals) -> Fidelity {
    let mutation = &reference.setup.config.mutation;
    Fidelity {
        per_unit_rate: mutation.per_unit_rate,
        executed_bias: mutation.executed_bias,
        executed_window_ticks: mutation.executed_window_ticks,
        identical_offspring_fraction: totals.reference_events.identical_offspring_fraction(),
        events: totals.reference_events.clone(),
        elite_carry_overs: totals.reference_carried,
        phenotype_mutation: false,
        learned_weight_capture: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::arena_config;
    use crate::calibration::draw_scenes;
    use v3_core::creature::founder::founder_genome_with_age_gate;

    fn founder_arm(start_energy: f32, lifetime: u32) -> Arm {
        let setup = Setup::new(arena_config(48), start_energy, lifetime);
        let founder = founder_genome_with_age_gate(
            setup.config.population.founder_profile,
            &setup.config.energy.lifecycle,
        );
        Arm {
            name: "native".into(),
            role: Role::Reference,
            policy: Policy::Native,
            setup,
            kind: ArmKind::Evolving {
                start: founder,
                shuffled: false,
            },
        }
    }

    #[test]
    fn the_frozen_record_is_the_last_training_scenes_including_a_death_there() {
        let arm = founder_arm(3.0, 400);
        let ArmKind::Evolving { start, .. } = &arm.kind else {
            unreachable!()
        };
        let individual = Individual {
            id: 0,
            parent: None,
            genome: Some(start.clone()),
            birth: None,
            carried: false,
        };
        let scenes = draw_scenes(5, 3, 48, 0.04, 5).unwrap();
        let scored = score_individual(&arm, &individual, &scenes, 1, 0);
        let (last_score, last_frozen) = evaluate_genome(&arm.setup, start, &scenes[2]);
        let death = last_score
            .death_tick
            .expect("3 energy dies inside the last scene");
        assert_eq!(scored.frozen, last_frozen);
        assert_eq!(scored.frozen.age, u64::from(death - 1));
        assert_eq!(scored.scenes.len(), 3);
        let mean = scored.scenes.iter().map(|s| s.score).sum::<f64>() / 3.0;
        assert!((scored.scalar - mean).abs() < 1e-12);
    }

    #[test]
    fn survivors_floor_the_elite_share_but_keep_one() {
        let plan = |population, elite_fraction| Plan {
            seed: 0,
            replicates: 1,
            generations: 1,
            population,
            elite_fraction,
            scenes: 1,
            food_fraction: 0.04,
            threshold: 0.0,
        };
        assert_eq!(plan(64, 0.25).survivors(), 16);
        assert_eq!(plan(7, 0.25).survivors(), 1);
        assert_eq!(plan(2, 0.1).survivors(), 1);
        assert_eq!(plan(5, 1.0).survivors(), 5);
    }

    fn plan(population: u32, elite_fraction: f64, generations: u32, threshold: f64) -> Plan {
        Plan {
            seed: 3,
            replicates: 1,
            generations,
            population,
            elite_fraction,
            scenes: 1,
            food_fraction: 0.06,
            threshold,
        }
    }

    /// A run directory under the system temp directory; removed on drop.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> (Self, RunDir) {
            let path = std::env::temp_dir()
                .join(format!("petri-lab-campaign-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            let budget = crate::output::Budget::new(64 << 20, 1 << 20).unwrap();
            let dir = RunDir::create(path.clone(), budget).unwrap();
            (Self(path), dir)
        }

        fn rows(&self, arm: &str) -> Vec<serde_json::Value> {
            std::fs::read_to_string(self.0.join("rows.ndjson"))
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                .filter(|row| row["arm"] == arm)
                .collect()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn fixed(arm: &Arm, name: &str, role: Role) -> Arm {
        let ArmKind::Evolving { start, .. } = &arm.kind else {
            unreachable!()
        };
        Arm {
            name: name.into(),
            role,
            policy: Policy::Native,
            setup: arm.setup.clone(),
            kind: ArmKind::Fixed(start.clone()),
        }
    }

    fn operator_total(map: &BTreeMap<String, u64>) -> u64 {
        map.values().sum()
    }

    #[test]
    fn the_reference_arm_alone_accumulates_births_carry_overs_and_events() {
        let mut native = founder_arm(100.0, 40);
        // About one event per birth: some children identical, some not.
        native.setup.config.mutation.per_unit_rate = 0.01;
        let arms = vec![
            native.clone(),
            fixed(&native, "founder-only", Role::Control),
        ];
        // 5 members, 1 survivor: 4 births in each of generations 1..=3.
        let plan = plan(5, 0.2, 4, 1e9);
        let (scratch, mut dir) = Scratch::new("births");
        let (summaries, totals) = run_campaign(&arms, &plan, &[], &mut dir).unwrap();
        let events = &totals.reference_events;
        assert_eq!(events.offspring, 12);
        assert_eq!(totals.reference_carried, 3);
        assert_eq!(summaries[0].replicates[0].generations_run, 4);
        assert!(events.requested > 0 && events.applied > 0 && events.skipped > 0);
        assert!(operator_total(&events.requested_by_operator) > 0);
        assert!(operator_total(&events.applied_by_operator) > 0);
        let identical = events.identical_offspring;
        assert!(
            identical > 0 && identical < 12 && identical * 2 != 12,
            "{identical}"
        );

        let rows = scratch.rows("native");
        assert_eq!(rows.len(), 4);
        let mut flagged = 0;
        for row in &rows {
            let members = row["individuals"].as_array().unwrap();
            let mut ids: Vec<u64> = members.iter().map(|m| m[0].as_u64().unwrap()).collect();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), 5, "ids are unique within a generation");
            for member in members {
                let (applied, identical) = (member[3].as_u64().unwrap(), member[4] == true);
                assert!(applied > 0 || identical || member[1].is_null() || member[5] == true);
                flagged += u64::from(identical);
            }
        }
        assert_eq!(
            flagged, identical,
            "row flags agree with the fidelity count"
        );
    }

    #[test]
    fn a_reached_arm_stops_while_instruments_run_to_the_horizon() {
        let native = founder_arm(100.0, 40);
        let arms = vec![
            native.clone(),
            fixed(&native, "comparator", Role::Instrument),
        ];
        let validation = draw_scenes(11, 2, 48, 0.06, 5).unwrap();
        let (scratch, mut dir) = Scratch::new("reach");
        let (summaries, _) =
            run_campaign(&arms, &plan(2, 0.5, 3, 0.0), &validation, &mut dir).unwrap();
        let reached = &summaries[0].replicates[0];
        assert_eq!(reached.reached, Some(true));
        assert_eq!(reached.generation_to_threshold, Some(0));
        assert_eq!(reached.stopped_by, StoppedBy::Reached);
        assert_eq!(reached.generations_run, 1);
        assert_eq!(scratch.rows("native").len(), 1);
        assert_eq!(scratch.rows("comparator").len(), 3);
        assert_eq!(summaries[1].replicates[0].stopped_by, StoppedBy::Horizon);
    }

    #[test]
    fn scripted_actors_seed_each_scene_by_its_campaign_index() {
        let native = founder_arm(100.0, 30);
        let ArmKind::Evolving { start, .. } = &native.kind else {
            unreachable!()
        };
        let scripted = Arm {
            name: "random-walk".into(),
            role: Role::Instrument,
            policy: Policy::Native,
            setup: native.setup.clone(),
            kind: ArmKind::Scripted {
                policy: Scripted::RandomWalk,
                body: start.clone(),
            },
        };
        let arms = vec![
            fixed(&native, "founder-only", Role::Control),
            scripted.clone(),
        ];
        let mut plan = plan(2, 0.5, 2, 1e9);
        plan.scenes = 2;
        let (scratch, mut dir) = Scratch::new("scripted");
        run_campaign(&arms, &plan, &[], &mut dir).unwrap();

        let r_seed = replicate_seed(plan.seed, 0);
        let mut scene_rng = stream(&[Part::U(tagged(r_seed, "scenes"))]);
        let scenes: Vec<Scene> = (0..4)
            .map(|_| draw_scene(&mut scene_rng, 48, plan.food_fraction, 5).unwrap())
            .collect();
        let score = |scene: usize, index: usize| {
            serde_json::to_value(evaluate_scripted(
                &scripted.setup,
                start,
                Scripted::RandomWalk,
                &scenes[scene],
                scripted_seed(r_seed, Scripted::RandomWalk, index),
            ))
            .unwrap()
        };
        let rows = scratch.rows("random-walk");
        assert_eq!(rows.len(), 2);
        for (scene, row) in [(0, &rows[0]), (1, &rows[0]), (2, &rows[1]), (3, &rows[1])] {
            assert_eq!(
                row["best_scenes"][scene % 2],
                score(scene, scene),
                "scene {scene}"
            );
        }
        // The check separates the campaign index from nearby wrong indices.
        assert_ne!(score(1, 1), score(1, 0));
        assert_ne!(score(2, 2), score(2, 0));
        assert_ne!(score(2, 2), score(2, 3));
        assert_ne!(score(3, 3), score(3, 1));
    }

    #[test]
    fn breeding_carries_survivors_and_mutates_children_under_the_documented_seed() {
        let mut arm = founder_arm(100.0, 40);
        // A high rate makes every child differ from its parent.
        arm.setup.config.mutation.per_unit_rate = 0.2;
        let ArmKind::Evolving { start, .. } = arm.kind.clone() else {
            unreachable!()
        };
        let plan = plan(3, 0.4, 2, 1e9);
        let r_seed = replicate_seed(plan.seed, 0);
        let mut lineage = Lineage::new(0, &arm, plan.population, r_seed);
        lineage.population[1].birth = Some(Events {
            offspring: 1,
            ..Events::default()
        });
        let scene = draw_scenes(5, 1, 48, 0.04, 5).unwrap().remove(0);
        let frozen = evaluate_genome(&arm.setup, &start, &scene).1;
        let members: Vec<Scored> = [1.0, 3.0, 2.0]
            .into_iter()
            .map(|scalar| Scored {
                scalar,
                scenes: Vec::new(),
                frozen: frozen.clone(),
                creature_ticks: 0,
            })
            .collect();
        let step = Step {
            plan: &plan,
            validation: &[],
            scenes: &[],
            replicate: 0,
            generation: 0,
        };
        breed(&arm, &mut lineage, &members, &[1, 2, 0], &step);

        let population = &lineage.population;
        assert_eq!(population.len(), 3);
        let carried = &population[0];
        assert_eq!((carried.id, carried.carried), (1, true));
        assert!(carried.birth.is_none());
        let reachable = mesh_reachable_nodes(&start);
        let child = |generation: u64, slot: u64| {
            let mut genome = start.clone();
            let mut rng = SmallRng::seed_from_u64(hash(&[
                Part::U(lineage.mutation_seed),
                Part::U(generation),
                Part::U(slot),
            ]));
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut genome,
                &arm.setup.config.mutation,
                &reachable,
                ParentExecuted::Record(&frozen.record, frozen.age),
                &mut rng,
                arm.setup.config.world.food.types.len(),
            );
            (genome, summary)
        };
        for (offset, member) in population[1..].iter().enumerate() {
            let (genome, summary) = child(1, 1 + offset as u64);
            assert_eq!(member.id, 3 + offset as u64);
            assert_eq!((member.parent, member.carried), (Some(1), false));
            assert_eq!(member.genome.as_ref(), Some(&genome));
            let birth = member.birth.as_ref().unwrap();
            assert_eq!(birth.requested, u64::from(summary.attempted_events));
            assert_eq!(birth.applied, u64::from(summary.applied_events));
            assert_eq!(birth.skipped, u64::from(summary.skipped_events));
            let keyed = |from: &HashMap<_, u32>| -> BTreeMap<String, u64> {
                from.iter()
                    .map(|(operator, count)| {
                        let key = serde_json::to_value(operator).unwrap();
                        (key.as_str().unwrap().to_owned(), u64::from(*count))
                    })
                    .collect()
            };
            assert_eq!(
                birth.requested_by_operator,
                keyed(&summary.attempted_by_operator)
            );
            assert_eq!(
                birth.applied_by_operator,
                keyed(&summary.applied_by_operator)
            );
            assert_eq!(
                birth.skipped_by_operator,
                keyed(&summary.skipped_by_operator)
            );
            assert_eq!(birth.offspring, 1);
            assert_eq!(birth.identical_offspring, u64::from(genome == start));
            assert!(genome != start && birth.applied > 0 && birth.skipped > 0);
            // Neighbouring seeds give different children.
            for (generation, slot) in [
                (0, 1 + offset as u64),
                (1, offset as u64),
                (1, 2 + offset as u64),
            ] {
                assert_ne!(child(generation, slot).0, genome, "{generation} {slot}");
            }
        }
    }

    #[test]
    fn reach_fractions_need_complete_tested_replicates() {
        let native = founder_arm(100.0, 1);
        let replicate = |reached: bool, incomplete: bool| ReplicateResult {
            replicate: 0,
            reached: Some(reached),
            generation_to_threshold: reached.then_some(1),
            censored: Some(!reached),
            incomplete,
            stopped_by: StoppedBy::Horizon,
            generations_run: 1,
            final_best: None,
        };
        let three = || {
            vec![
                replicate(true, false),
                replicate(true, false),
                replicate(false, false),
            ]
        };
        let summary = arm_summary(&native, three());
        assert_eq!(summary.reached_fraction, Some(2.0 / 3.0));
        assert_eq!(summary.wilson_95, crate::stats::wilson_95(2, 3));
        let instrument = fixed(&native, "comparator", Role::Instrument);
        assert_eq!(arm_summary(&instrument, three()).reached_fraction, None);
        let mut partial = three();
        partial.push(replicate(false, true));
        assert_eq!(arm_summary(&native, partial).reached_fraction, None);
        assert_eq!(arm_summary(&native, Vec::new()).reached_fraction, None);
    }

    #[test]
    fn rank_orders_by_scalar_then_tie_key() {
        assert_eq!(rank(&[1.0, 3.0, 3.0, 2.0], &[0, 9, 4, 1]), vec![2, 1, 3, 0]);
    }
}

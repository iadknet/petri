//! Per-elite brain and sensor change readings (T22.F03).
//!
//! Every genome arm's row carries a `shape` block for its elite (size,
//! reach, executed nodes, ancestry, input use by catalog family, steering);
//! the `--signature-arms` set also carries a `signature` block (causal
//! influence by family under ablation on the generation's scenes, and `n`
//! fresh mutants on the `observation` stream). Readings never read the
//! `scenes`, `mutation` or `selection` streams and never feed the
//! population.

use std::collections::{BTreeMap, BTreeSet};

use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use v3_core::config::SimulationConfig;
use v3_core::contracts::{InputReference, NodeId};
use v3_core::creature::genome::analysis::{functional_complexity, mesh_reachable_nodes};
use v3_core::creature::genome::cgp::GraphSource;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::creature::sensor_census::{creature_sensor_census, DecisionInputKey};
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::MutationEngine;
use v3_core::neighborhood::input_use::catalog::Family;
use v3_core::neighborhood::steering::{SteeringBattery, SteeringReading};
use v3_core::neighborhood::{classify, Battery, Class, Signature as BatterySignature};

use crate::eval::{evaluate_genome_observed, expressed, Boundary, Frozen, SceneScore, Setup};
use crate::rng::{hash, Part};
use crate::scene::Scene;
use crate::summary::Events;

/// `--mutants` bounds.
pub const MUTANTS: std::ops::RangeInclusive<u32> = 1..=64;

/// The `unsupported` label of a genome the ablation rewrite cannot address.
pub const UNSUPPORTED_INPUT_REFS: &str = "input_refs";

/// The ablation sentinel both backends resolve to 0.0; a node with this
/// many references or more cannot be ablated.
pub const ABLATED_REF: u16 = u16::MAX;

/// Which genome arms carry the `signature` block.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureArms {
    /// The arms whose elite can change: reference, user and
    /// `shuffled-score`.
    #[default]
    Changing,
    /// Reference and user arms only.
    Native,
    /// Every genome arm, comparator included.
    All,
}

/// Mutation events summed along an individual's ancestry since the arm's
/// start genome.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ancestry {
    /// Births on the path (every pass through the engine).
    pub births: u64,
    pub requested: u64,
    pub applied: u64,
    pub applied_by_operator: BTreeMap<String, u64>,
}

impl Ancestry {
    /// A child's ancestry: its parent's plus its own birth events.
    #[must_use]
    pub fn child(&self, birth: &Events) -> Self {
        let mut child = self.clone();
        child.births += 1;
        child.requested += birth.requested;
        child.applied += birth.applied;
        for (operator, count) in &birth.applied_by_operator {
            *child
                .applied_by_operator
                .entry(operator.clone())
                .or_default() += count;
        }
        child
    }
}

/// One catalog family with a consumer on a reachable node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyReading {
    pub family: String,
    /// True by construction.
    pub structural: bool,
    /// A consumer on a node in the executed union.
    pub executed_node: bool,
    /// The census's live verdict for world and decision families; null for
    /// the others.
    pub live: Option<bool>,
}

/// The elite's shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub genome_size: u32,
    pub functional_complexity: u32,
    pub nodes: usize,
    pub reachable: usize,
    /// Nodes dispatched through each training scene's last living boundary,
    /// unioned over the scenes.
    pub executed: usize,
    /// Training scenes the elite died in (their death tick is unobserved).
    pub deaths: u32,
    pub ancestry: Ancestry,
    pub families: Vec<FamilyReading>,
    pub stateful_node: bool,
    pub steering: SteeringReading,
}

/// One family's causal reading under ablation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Causal {
    pub family: String,
    /// Some scene's `SceneScore` or `(position, energy)` sequence differs.
    pub causal: bool,
    /// The ablated copy's scalar minus the elite's.
    pub score_delta: f64,
}

/// `n` fresh mutants of the elite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mutants {
    pub n: u32,
    pub identical: u32,
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    /// The `n` scalars, ascending.
    pub scores: Vec<f64>,
    pub improved: u32,
    pub equal: u32,
    pub worse: u32,
    pub mean_delta: f64,
}

/// The elite's signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    /// Null when the rewrite is unsound for this genome.
    pub causal: Option<Vec<Causal>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<String>,
    pub mutants: Mutants,
}

/// A row's `readings` block.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Readings {
    pub shape: Shape,
    pub signature: Option<Signature>,
}

/// The summary's `first` projection: the shape scalars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstProjection {
    pub genome_size: u32,
    pub functional_complexity: u32,
    pub nodes: usize,
    pub reachable: usize,
    pub executed: usize,
    pub deaths: u32,
    pub births: u64,
    pub requested: u64,
    pub applied: u64,
}

/// The summary's signature aggregates (the score vector stays in the rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureAggregates {
    pub causal: Option<Vec<Causal>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<String>,
    pub n: u32,
    pub identical: u32,
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    pub improved: u32,
    pub equal: u32,
    pub worse: u32,
    pub score_min: f64,
    pub score_median: f64,
    pub score_max: f64,
    pub mean_delta: f64,
}

/// The summary's `last` shape: exactly what the report renders. The rest of
/// the shape (`stateful_node`, `ancestry.requested` and the per-operator
/// counts, the other steering counters) stays in the rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapeProjection {
    pub genome_size: u32,
    pub functional_complexity: u32,
    pub nodes: usize,
    pub reachable: usize,
    pub executed: usize,
    pub deaths: u32,
    pub births: u64,
    pub applied: u64,
    pub families: Vec<FamilyReading>,
    pub moves: u64,
    pub exact_hits: u64,
    pub avoidance_trials: u64,
    pub avoided: u64,
}

/// The summary's `last` projection: the rendered shape and, when computed,
/// the signature aggregates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastProjection {
    pub shape: ShapeProjection,
    pub signature: Option<SignatureAggregates>,
}

/// Per arm and replicate: projections of the first and last written rows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicateReadings {
    pub first: Option<FirstProjection>,
    pub last: Option<LastProjection>,
}

impl ReplicateReadings {
    /// Record a written row's readings.
    pub fn record(&mut self, readings: &Readings) {
        if self.first.is_none() {
            self.first = Some(FirstProjection::of(&readings.shape));
        }
        self.last = Some(LastProjection {
            shape: ShapeProjection::of(&readings.shape),
            signature: readings.signature.as_ref().map(SignatureAggregates::of),
        });
    }
}

impl FirstProjection {
    #[must_use]
    pub fn of(shape: &Shape) -> Self {
        Self {
            genome_size: shape.genome_size,
            functional_complexity: shape.functional_complexity,
            nodes: shape.nodes,
            reachable: shape.reachable,
            executed: shape.executed,
            deaths: shape.deaths,
            births: shape.ancestry.births,
            requested: shape.ancestry.requested,
            applied: shape.ancestry.applied,
        }
    }
}

impl ShapeProjection {
    #[must_use]
    pub fn of(shape: &Shape) -> Self {
        Self {
            genome_size: shape.genome_size,
            functional_complexity: shape.functional_complexity,
            nodes: shape.nodes,
            reachable: shape.reachable,
            executed: shape.executed,
            deaths: shape.deaths,
            births: shape.ancestry.births,
            applied: shape.ancestry.applied,
            families: shape.families.clone(),
            moves: shape.steering.moves,
            exact_hits: shape.steering.exact_hits,
            avoidance_trials: shape.steering.avoidance_trials,
            avoided: shape.steering.avoided,
        }
    }
}

impl SignatureAggregates {
    #[must_use]
    pub fn of(signature: &Signature) -> Self {
        let m = &signature.mutants;
        let first = m.scores.first().copied().unwrap_or(0.0);
        Self {
            causal: signature.causal.clone(),
            unsupported: signature.unsupported.clone(),
            n: m.n,
            identical: m.identical,
            silent: m.silent,
            changed: m.changed,
            dead: m.dead,
            improved: m.improved,
            equal: m.equal,
            worse: m.worse,
            score_min: first,
            score_median: crate::stats::median(&m.scores).unwrap_or(first),
            score_max: m.scores.last().copied().unwrap_or(first),
            mean_delta: m.mean_delta,
        }
    }
}

/// The fixed batteries an arm's readings run, generated once per arm.
#[derive(Debug, Clone)]
pub struct Batteries {
    battery: Battery,
    steering: SteeringBattery,
}

impl Batteries {
    #[must_use]
    pub fn new(config: &SimulationConfig) -> Self {
        let food_types = config.world.food.types.len();
        Self {
            battery: Battery::generate(food_types),
            steering: SteeringBattery::generate(food_types),
        }
    }

    /// `genome`'s `neighborhood-v1` signature, masked as the scoring pass
    /// executes it.
    #[must_use]
    pub fn signature(
        &self,
        config: &SimulationConfig,
        genome: &CreatureGenome,
    ) -> BatterySignature {
        self.battery.signature(
            &expressed(genome),
            &config.runtime,
            config.shared_memory.decay_rate,
        )
    }
}

/// What the scoring pass observed of the elite on the training scenes.
#[derive(Debug, Clone, Copy)]
pub struct Observed<'a> {
    pub genome: &'a CreatureGenome,
    pub ancestry: &'a Ancestry,
    pub scenes: &'a [Scene],
    pub scores: &'a [SceneScore],
    /// Per scene: the last living dispatch record.
    pub frozens: &'a [Frozen],
    /// Per scene: the `(position, energy)` sequence.
    pub sequences: &'a [Vec<Boundary>],
    pub scalar: f64,
}

impl Observed<'_> {
    /// The frozen record breeding uses: the last training scene's.
    fn breeding_frozen(&self) -> &Frozen {
        self.frozens.last().expect("at least one training scene")
    }
}

/// Where the mutant stream comes from:
/// `hash(observation, arm, generation, k)`.
#[derive(Debug, Clone, Copy)]
pub struct MutantSeed<'a> {
    /// `hash(r_i, "observation")`.
    pub observation: u64,
    pub arm: &'a str,
    pub generation: u32,
}

impl MutantSeed<'_> {
    fn rng(&self, k: u32) -> SmallRng {
        SmallRng::seed_from_u64(hash(&[
            Part::U(self.observation),
            Part::S(self.arm),
            Part::U(u64::from(self.generation)),
            Part::U(u64::from(k)),
        ]))
    }
}

/// The family a shared-memory read addresses.
fn shared_memory_family(previous: bool) -> Family {
    if previous {
        Family::SharedMemoryPrevious
    } else {
        Family::SharedMemory
    }
}

/// Every catalog family with a consumer, and the node indices holding one.
/// A consumer is an `InputLeaf` edge or `ReadInput` instruction whose
/// in-range `ref_idx` names a reference of the family, a `SharedMemory`
/// source or a `LoadSlot*` instruction. No liveness filter.
#[must_use]
pub fn consumers(genome: &CreatureGenome) -> BTreeMap<Family, BTreeSet<usize>> {
    let mut families: BTreeMap<Family, BTreeSet<usize>> = BTreeMap::new();
    for (index, node) in genome.nodes.iter().enumerate() {
        let mut add = |family: Family| {
            families.entry(family).or_default().insert(index);
        };
        let reference = |ref_idx: u16| node.input_refs.get(usize::from(ref_idx)).map(Family::of);
        match &node.backend_def {
            BackendDef::Graph(graph) => {
                let edges = graph
                    .compute_nodes
                    .iter()
                    .flat_map(|compute| &compute.inputs)
                    .chain(graph.output_sinks.iter().flat_map(|sink| &sink.inputs));
                for edge in edges {
                    match edge.source {
                        GraphSource::InputLeaf { ref_idx, .. } => {
                            if let Some(family) = reference(ref_idx) {
                                add(family);
                            }
                        }
                        GraphSource::SharedMemory { previous, .. } => {
                            add(shared_memory_family(previous));
                        }
                        GraphSource::ComputeNode(_) => {}
                    }
                }
            }
            BackendDef::Vm(vm) => {
                for instruction in &vm.program {
                    match instruction {
                        VmInstruction::ReadInput { ref_idx, .. } => {
                            if let Some(family) = reference(*ref_idx) {
                                add(family);
                            }
                        }
                        VmInstruction::LoadSlot { .. } | VmInstruction::LoadSlotImm { .. } => {
                            add(Family::SharedMemory);
                        }
                        VmInstruction::LoadSlotPrev { .. } => add(Family::SharedMemoryPrevious),
                        _ => {}
                    }
                }
            }
        }
    }
    families
}

/// The census key's family for decision inputs.
fn decision_family(key: DecisionInputKey) -> Family {
    match key {
        DecisionInputKey::ActionVotes => Family::ActionVotes,
        DecisionInputKey::PreviousPassVotes => Family::PreviousPassVotes,
        DecisionInputKey::CommitCounts => Family::CommitCounts,
        DecisionInputKey::HopsThisTick => Family::HopsThisTick,
        DecisionInputKey::PreviousOutcome => Family::PreviousOutcome,
    }
}

/// Whether the sensor census covers `family` (world and decision families).
fn census_covers(family: Family) -> bool {
    matches!(
        family,
        Family::FoodHere(_)
            | Family::NeighborFoodRing(_)
            | Family::NeighborBarrierRing
            | Family::NeighborOccupiedRing
            | Family::AreaFoodSummary(_)
            | Family::AreaBarrierSummary
            | Family::NearbyCreatureCore
            | Family::NearbyCreatureVitals
            | Family::NearbyCreatureIdentity
            | Family::AreaOccupancySummary
            | Family::ActionVotes
            | Family::PreviousPassVotes
            | Family::CommitCounts
            | Family::HopsThisTick
            | Family::PreviousOutcome
    )
}

/// The union over the scenes of the nodes dispatched through each scene's
/// last living boundary, ascending.
#[must_use]
pub fn executed_union(frozens: &[Frozen]) -> BTreeSet<usize> {
    frozens
        .iter()
        .flat_map(|frozen| frozen.record.executed_indices(frozen.age, u64::MAX))
        .collect()
}

/// The elite's static wiring, computed once per reading.
struct Wiring {
    reachable: Vec<usize>,
    /// Every family with a consumer, and the node indices holding one.
    consumers: BTreeMap<Family, BTreeSet<usize>>,
    /// The families with a consumer on a reachable node, in catalog order.
    families: Vec<Family>,
}

impl Wiring {
    fn of(genome: &CreatureGenome) -> Self {
        let reachable = mesh_reachable_nodes(genome);
        let consumers = consumers(genome);
        let on_reachable: BTreeSet<usize> = reachable.iter().copied().collect();
        let families = consumers
            .iter()
            .filter(|(_, nodes)| nodes.iter().any(|node| on_reachable.contains(node)))
            .map(|(&family, _)| family)
            .collect();
        Self {
            reachable,
            consumers,
            families,
        }
    }
}

/// The elite's `shape` block.
fn shape(setup: &Setup, batteries: &Batteries, elite: &Observed<'_>, wiring: &Wiring) -> Shape {
    let genome = elite.genome;
    let reachable = &wiring.reachable;
    let executed = executed_union(elite.frozens);
    let census = creature_sensor_census(genome, reachable);
    let families = wiring
        .families
        .iter()
        .map(|&family| {
            let live = census_covers(family).then(|| {
                census
                    .world_inputs
                    .iter()
                    .any(|&key| Family::of(&InputReference::World(key)) == family)
                    || census
                        .decision_inputs
                        .iter()
                        .any(|&key| decision_family(key) == family)
            });
            FamilyReading {
                family: family.label(),
                structural: true,
                executed_node: wiring.consumers[&family]
                    .iter()
                    .any(|node| executed.contains(node)),
                live,
            }
        })
        .collect();
    let executed_ids: BTreeSet<NodeId> = executed
        .iter()
        .filter_map(|&index| genome.nodes.get(index).map(|node| node.node_id))
        .collect();
    let steering =
        batteries
            .steering
            .read(&expressed(genome), &setup.config.runtime, &executed_ids);
    Shape {
        genome_size: genome.genome_size(),
        functional_complexity: functional_complexity(genome),
        nodes: genome.nodes.len(),
        reachable: reachable.len(),
        executed: executed.len(),
        deaths: u32::try_from(
            elite
                .scores
                .iter()
                .filter(|score| score.death_tick.is_some())
                .count(),
        )
        .unwrap_or(u32::MAX),
        ancestry: elite.ancestry.clone(),
        families,
        stateful_node: census.holds_stateful_node,
        steering,
    }
}

/// Whether the ablation rewrite can be sound for `genome`: every node has
/// fewer than [`ABLATED_REF`] references, so the sentinels are out of range.
/// A genome file failing it is refused at load.
#[must_use]
pub fn ablation_supported(genome: &CreatureGenome) -> bool {
    genome
        .nodes
        .iter()
        .all(|node| node.input_refs.len() < usize::from(ABLATED_REF))
}

/// Hand every input-reference index of `backend` (`InputLeaf` edges and
/// `ReadInput` instructions) to `visit`.
fn for_each_ref_idx(backend: &mut BackendDef, mut visit: impl FnMut(&mut u16)) {
    match backend {
        BackendDef::Graph(graph) => {
            let edges = graph
                .compute_nodes
                .iter_mut()
                .flat_map(|compute| &mut compute.inputs)
                .chain(
                    graph
                        .output_sinks
                        .iter_mut()
                        .flat_map(|sink| &mut sink.inputs),
                );
            for edge in edges {
                if let GraphSource::InputLeaf { ref_idx, .. } = &mut edge.source {
                    visit(ref_idx);
                }
            }
        }
        BackendDef::Vm(vm) => {
            for instruction in &mut vm.program {
                if let VmInstruction::ReadInput { ref_idx, .. } = instruction {
                    visit(ref_idx);
                }
            }
        }
    }
}

/// `genome` with every Graph `InputLeaf` edge and VM `ReadInput` whose
/// in-range reference is of `family` pointed out of range, which both
/// backends resolve to 0.0. Each distinct ablated index gets its own
/// sentinel (from [`ABLATED_REF`] down, skipping indices the node already
/// uses), so the distinct consumed indices `functional_complexity` counts,
/// and with them every charge, are unchanged. `None` when the rewrite is
/// unsound: a node without the headroom.
#[must_use]
pub fn ablate(genome: &CreatureGenome, family: Family) -> Option<CreatureGenome> {
    if !ablation_supported(genome) {
        return None;
    }
    let mut ablated = genome.clone();
    for node in &mut ablated.nodes {
        let len = node.input_refs.len();
        let mut used = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for_each_ref_idx(&mut node.backend_def, |ref_idx| {
            used.insert(*ref_idx);
            if node
                .input_refs
                .get(usize::from(*ref_idx))
                .is_some_and(|reference| Family::of(reference) == family)
            {
                targets.insert(*ref_idx);
            }
        });
        let mut free = (0..=ABLATED_REF)
            .rev()
            .take_while(|&candidate| usize::from(candidate) >= len)
            .filter(|candidate| !used.contains(candidate));
        let mut sentinels = BTreeMap::new();
        for target in targets {
            sentinels.insert(target, free.next()?);
        }
        for_each_ref_idx(&mut node.backend_def, |ref_idx| {
            if let Some(&sentinel) = sentinels.get(ref_idx) {
                *ref_idx = sentinel;
            }
        });
    }
    Some(ablated)
}

/// One copy's evaluation on the training scenes.
struct CopyRun {
    scalar: f64,
    /// Some scene's score or sequence differs from the elite's.
    differs: bool,
    ticks: u64,
}

/// Evaluate `copies` on the elite's scenes, one job per copy and scene,
/// collected in index order.
fn run_copies(setup: &Setup, elite: &Observed<'_>, copies: &[&CreatureGenome]) -> Vec<CopyRun> {
    let scenes = elite.scenes.len();
    let jobs: Vec<(usize, usize)> = (0..copies.len())
        .flat_map(|copy| (0..scenes).map(move |scene| (copy, scene)))
        .collect();
    let runs: Vec<(SceneScore, bool)> = jobs
        .par_iter()
        .map(|&(copy, scene)| {
            let (score, _, sequence) =
                evaluate_genome_observed(setup, copies[copy], &elite.scenes[scene]);
            let differs = score != elite.scores[scene] || sequence != elite.sequences[scene];
            (score, differs)
        })
        .collect();
    runs.chunks(scenes.max(1))
        .take(copies.len())
        .map(|chunk| {
            let values: Vec<f64> = chunk.iter().map(|(score, _)| score.score).collect();
            CopyRun {
                scalar: crate::stats::mean(&values).unwrap_or(0.0),
                differs: chunk.iter().any(|(_, differs)| *differs),
                ticks: chunk.iter().map(|(score, _)| u64::from(score.ticks)).sum(),
            }
        })
        .collect()
}

/// The causal readings of the non-shared-memory families, or `None` when
/// the rewrite is unsound; with the production creature-ticks spent.
fn causal(setup: &Setup, elite: &Observed<'_>, families: &[Family]) -> (Option<Vec<Causal>>, u64) {
    let families: Vec<Family> = families
        .iter()
        .copied()
        .filter(|family| !family.is_shared_memory())
        .collect();
    let Some(copies) = families
        .iter()
        .map(|&family| ablate(elite.genome, family))
        .collect::<Option<Vec<_>>>()
    else {
        return (None, 0);
    };
    let runs = run_copies(setup, elite, &copies.iter().collect::<Vec<_>>());
    let ticks = runs.iter().map(|run| run.ticks).sum();
    let readings = families
        .iter()
        .zip(&runs)
        .map(|(family, run)| Causal {
            family: family.label(),
            causal: run.differs,
            score_delta: run.scalar - elite.scalar,
        })
        .collect();
    (Some(readings), ticks)
}

/// Fold per-mutant results in index order.
fn fold_mutants(elite_scalar: f64, results: &[(Class, f64, bool)]) -> Mutants {
    let n = u32::try_from(results.len()).unwrap_or(u32::MAX);
    let count = |keep: &dyn Fn(&(Class, f64, bool)) -> bool| {
        u32::try_from(results.iter().filter(|r| keep(r)).count()).unwrap_or(u32::MAX)
    };
    let mut scores: Vec<f64> = results.iter().map(|(_, score, _)| *score).collect();
    let delta_sum: f64 = scores.iter().map(|score| score - elite_scalar).sum();
    scores.sort_by(f64::total_cmp);
    Mutants {
        n,
        identical: count(&|r| r.2),
        silent: count(&|r| r.0 == Class::Silent),
        changed: count(&|r| r.0 == Class::Changed),
        dead: count(&|r| r.0 == Class::Dead),
        improved: count(&|r| r.1 > elite_scalar),
        equal: count(&|r| r.1 == elite_scalar),
        worse: count(&|r| r.1 < elite_scalar),
        mean_delta: if results.is_empty() {
            0.0
        } else {
            delta_sum / f64::from(n)
        },
        scores,
    }
}

/// `n` fresh mutants of the elite: each passed once through the engine as
/// breeding passes a child, classed on `neighborhood-v1` against the masked
/// elite and scored on the training scenes (identical ones silent and
/// scored by copy); with the production creature-ticks spent.
fn mutants(
    setup: &Setup,
    batteries: &Batteries,
    elite: &Observed<'_>,
    reachable: &[usize],
    n: u32,
    seed: &MutantSeed<'_>,
) -> (Mutants, u64) {
    let config = &setup.config;
    let frozen = elite.breeding_frozen();
    let genomes: Vec<CreatureGenome> = (0..n)
        .into_par_iter()
        .map(|k| {
            let mut child = elite.genome.clone();
            MutationEngine::apply_mutations_with_food_type_count(
                &mut child,
                &config.mutation,
                reachable,
                ParentExecuted::Record(&frozen.record, frozen.age),
                &mut seed.rng(k),
                config.world.food.types.len(),
            );
            child
        })
        .collect();
    let identical: Vec<bool> = genomes
        .iter()
        .map(|genome| genome == elite.genome)
        .collect();
    let novel: Vec<&CreatureGenome> = genomes
        .iter()
        .zip(&identical)
        .filter_map(|(genome, &same)| (!same).then_some(genome))
        .collect();
    let signature = |genome: &CreatureGenome| batteries.signature(config, genome);
    let (classes, runs) = if novel.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        let base = signature(elite.genome);
        let classes: Vec<Class> = novel
            .par_iter()
            .map(|&genome| classify(&base, &signature(genome)).class)
            .collect();
        (classes, run_copies(setup, elite, &novel))
    };
    let ticks = runs.iter().map(|run| run.ticks).sum();
    let mut novel_results = classes.into_iter().zip(runs);
    let results: Vec<(Class, f64, bool)> = identical
        .iter()
        .map(|&same| {
            if same {
                (Class::Silent, elite.scalar, true)
            } else {
                let (class, run) = novel_results.next().expect("one run per novel mutant");
                (class, run.scalar, false)
            }
        })
        .collect();
    (fold_mutants(elite.scalar, &results), ticks)
}

/// The elite's readings; `signature` carries `n` mutants when `Some`.
/// Returns the readings and the production creature-ticks spent on
/// ablated copies and mutants.
#[must_use]
pub fn read(
    setup: &Setup,
    batteries: &Batteries,
    elite: &Observed<'_>,
    signature: Option<(u32, MutantSeed<'_>)>,
) -> (Readings, u64) {
    let wiring = Wiring::of(elite.genome);
    let shape = shape(setup, batteries, elite, &wiring);
    let Some((n, seed)) = signature else {
        return (
            Readings {
                shape,
                signature: None,
            },
            0,
        );
    };
    let (causal, causal_ticks) = causal(setup, elite, &wiring.families);
    let (mutants, mutant_ticks) = mutants(setup, batteries, elite, &wiring.reachable, n, &seed);
    let unsupported = causal.is_none().then(|| UNSUPPORTED_INPUT_REFS.to_owned());
    (
        Readings {
            shape,
            signature: Some(Signature {
                causal,
                unsupported,
                mutants,
            }),
        },
        causal_ticks + mutant_ticks,
    )
}

#[cfg(test)]
mod tests;

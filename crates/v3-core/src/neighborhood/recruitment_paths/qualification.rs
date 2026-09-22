//! T13.F05: seed-selected production-operator paths from each fixed starting
//! form to a useful, bypass-sensitive contribution. Observation only.

use super::fixtures::{cue, move_sink, stage_for, EAST};
use super::*;
use crate::config::MutationConfig;
use crate::contracts::{InputReference, NodeId};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::engine::{
    graph_operator_key, input_ref_operator_key, topology_operator_key, vm_operator_key,
};
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSets;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::vm::{VmMutator, VmOperator};
use crate::mutation::{MutationOperator, MutationSkipReason, TargetReachability};
use crate::neighborhood::recruitment::ModuleBackend;
use rand::{rngs::SmallRng, SeedableRng};

/// Every pinned step seed is the first in `0..SEARCH_RANGE` whose applied
/// event matches the step's structural acceptance predicate; the one-off
/// search ([`search_seeds`]) finds them, the maintained tests replay them.
pub const SEARCH_RANGE: u64 = 1_000_000;

/// Paths longer than this are recorded growth gaps, not qualified paths.
pub const MAX_PATH_EVENTS: usize = 6;

/// The scaffold node every starting form carries.
const SCAFFOLD: NodeId = NodeId(2);

/// One production mutation event: an explicit operator applied through its
/// domain mutator's production entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionEvent {
    Topology(TopologyOperator),
    Graph(GraphOperator),
    Vm(VmOperator),
    InputRef(InputRefOperator),
}

impl ProductionEvent {
    #[must_use]
    pub fn operator(self) -> MutationOperator {
        match self {
            Self::Topology(op) => topology_operator_key(op),
            Self::Graph(op) => graph_operator_key(op),
            Self::Vm(op) => vm_operator_key(op),
            Self::InputRef(op) => input_ref_operator_key(op),
        }
    }

    /// Apply through the production mutator with a uniform target draw over
    /// the operator's own applicable set, as the constructed F02 stages did.
    ///
    /// # Errors
    /// The operator's own skip when it has no applicable site.
    pub fn apply(
        self,
        genome: &mut CreatureGenome,
        seed: u64,
    ) -> Result<TargetReachability, MutationSkipReason> {
        let reachable = mesh_reachable_nodes(genome);
        let mut selector = TargetSets::new(&reachable, &[]).selector(0.0, 0.0);
        let mut rng = SmallRng::seed_from_u64(seed);
        let config = MutationConfig::default();
        match self {
            Self::Topology(op) => TopologyMutator::apply_with_food_type_count(
                genome,
                op,
                &mut selector,
                &mut rng,
                &config,
                1,
            ),
            Self::Graph(op) => GraphMutator::apply(genome, op, &mut selector, &mut rng, &config),
            Self::Vm(op) => VmMutator::apply(genome, op, &mut selector, &mut rng, &config),
            Self::InputRef(op) => InputRefMutator::apply_with_food_type_count(
                genome,
                op,
                &mut selector,
                &mut rng,
                &config,
                1,
            ),
        }
    }
}

/// One applied path event with its per-step facts.
#[derive(Debug, Clone)]
pub struct PathStep {
    pub event: ProductionEvent,
    pub seed: u64,
    pub stage: ConstructionStage,
    /// Per-scene actions, shared memory and routing equal the previous stage.
    pub surfaces_unchanged: bool,
}

impl PathStep {
    /// Real charges the step's subject incurred on the task battery.
    #[must_use]
    pub fn charges(&self) -> TaskSummary {
        self.stage.task.summary()
    }

    #[must_use]
    pub fn genome_size(&self) -> u32 {
        self.stage.genome.genome_size()
    }
}

/// A form whose shortest complete path exceeds [`MAX_PATH_EVENTS`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrowthGap {
    pub length: usize,
    pub lengthening_step: String,
}

#[derive(Debug, Clone)]
pub struct QualifiedPath {
    pub form: String,
    pub task: Task,
    pub backend: ModuleBackend,
    pub start: ConstructionStage,
    /// The scaffold's `backend_def` at its creation stage (T13.F07).
    pub birth_payload: BackendDef,
    pub steps: Vec<PathStep>,
    pub gap: Option<GrowthGap>,
}

/// The scaffold's T13.F07 counterfactual readings at one path step.
#[derive(Debug, Clone, PartialEq)]
pub struct StepReading {
    pub step: String,
    pub score: u8,
    pub payload_changed: bool,
    pub bypass_loss: i16,
    /// score(step) − score(birth payload on the same route).
    pub ancestral_loss: i16,
    pub specialization: Specialization,
}

/// Read the scaffold of `genome` against `baseline` on `task`: the bypass and
/// birth-payload counterfactuals and the five specialization components.
#[must_use]
pub fn payload_reading(
    step: &str,
    genome: &CreatureGenome,
    birth_payload: &BackendDef,
    baseline: &TaskReading,
    task: Task,
) -> StepReading {
    use super::super::mesh_execution::{ancestral_payload_replacement, static_successor_bypass};
    let reading = evaluate(genome);
    let score = reading.correct(task);
    let bypass = evaluate(&static_successor_bypass(genome, SCAFFOLD));
    let payload_changed = node(genome, SCAFFOLD).backend_def != *birth_payload;
    // A verbatim payload replaced by itself is the identity: loss zero.
    let ancestral_loss = if payload_changed {
        let replaced = evaluate(&ancestral_payload_replacement(
            genome,
            SCAFFOLD,
            birth_payload,
        ));
        i16::from(score) - i16::from(replaced.correct(task))
    } else {
        0
    };
    let bypass_loss = i16::from(score) - i16::from(bypass.correct(task));
    StepReading {
        step: step.into(),
        score,
        payload_changed,
        bypass_loss,
        ancestral_loss,
        specialization: Specialization {
            task_live: reading.live(),
            score_gain: score > baseline.correct(task),
            bypass_loss: bypass_loss >= 1,
            ancestral_loss: ancestral_loss >= 1,
            incumbents_preserved: reading.preserves_correct_scenes(baseline, task),
        },
    }
}

impl QualifiedPath {
    /// Complete within the bound.
    #[must_use]
    pub fn qualified(&self) -> bool {
        self.gap.is_none() && self.steps.len() <= MAX_PATH_EVENTS
    }

    /// T13.F07: the scaffold's counterfactual readings at every step,
    /// against the path's start.
    #[must_use]
    pub fn payload_readings(&self) -> Vec<StepReading> {
        self.steps
            .iter()
            .map(|step| {
                payload_reading(
                    &step.stage.name,
                    &step.stage.genome,
                    &self.birth_payload,
                    &self.start.task,
                    self.task,
                )
            })
            .collect()
    }

    /// T13.F06: each step judged by `Policy::CostSelection` against its
    /// predecessor as a lone sibling, with the charges the step incurred.
    #[must_use]
    pub fn cost_verdicts(&self) -> Vec<CostVerdict> {
        let dead = Candidate {
            live: false,
            score: 0,
            ending_energy_sum: 0.0,
        };
        let mut previous = &self.start;
        self.steps
            .iter()
            .map(|step| {
                let charges = step.charges();
                let verdict = CostVerdict {
                    step: step.stage.name.clone(),
                    retained: choose(
                        Policy::CostSelection,
                        Candidate::new(&previous.task, self.task),
                        [Candidate::new(&step.stage.task, self.task), dead],
                    ) == Some(0),
                    score: step.stage.task.correct(self.task),
                    previous_score: previous.task.correct(self.task),
                    carrying_sum: charges.carrying_sum,
                    ending_energy_sum: charges.ending_energy_sum,
                    previous_ending_energy_sum: previous.task.summary().ending_energy_sum,
                };
                previous = &step.stage;
                verdict
            })
            .collect()
    }
}

/// One path step's cost-visible selection verdict against its predecessor.
#[derive(Debug, Clone, PartialEq)]
pub struct CostVerdict {
    pub step: String,
    pub retained: bool,
    pub score: u8,
    pub previous_score: u8,
    pub carrying_sum: f64,
    pub ending_energy_sum: f64,
    pub previous_ending_energy_sum: f64,
}

impl CostVerdict {
    /// Equal score against the predecessor: the step only costs or saves.
    #[must_use]
    pub fn neutral(&self) -> bool {
        self.score == self.previous_score
    }
}

type Accept = Box<dyn Fn(&CreatureGenome, &CreatureGenome) -> bool>;

struct StepSpec {
    name: &'static str,
    edits: &'static str,
    event: ProductionEvent,
    accept: Accept,
}

fn step(
    name: &'static str,
    edits: &'static str,
    event: ProductionEvent,
    accept: impl Fn(&CreatureGenome, &CreatureGenome) -> bool + 'static,
) -> StepSpec {
    StepSpec {
        name,
        edits,
        event,
        accept: Box::new(accept),
    }
}

/// The event applied with `seed`, if the predicate accepts the result.
fn accepted(
    genome: &CreatureGenome,
    event: ProductionEvent,
    seed: u64,
    accept: &Accept,
) -> Option<CreatureGenome> {
    let mut candidate = genome.clone();
    event.apply(&mut candidate, seed).ok()?;
    accept(genome, &candidate).then_some(candidate)
}

/// The first seed in `0..SEARCH_RANGE` whose applied event the predicate
/// accepts: the one-off search, not the maintained replay.
fn first_seed(
    genome: &CreatureGenome,
    event: ProductionEvent,
    accept: &Accept,
) -> Option<(u64, CreatureGenome)> {
    (0..SEARCH_RANGE).find_map(|seed| Some((seed, accepted(genome, event, seed, accept)?)))
}

fn surfaces_unchanged(previous: &TaskReading, current: &TaskReading) -> bool {
    previous.scenes.len() == current.scenes.len()
        && previous.scenes.iter().zip(&current.scenes).all(|(a, b)| {
            a.actions == b.actions && a.shared_memory == b.shared_memory && a.routing == b.routing
        })
}

/// One starting form's plan with its pinned seeds, one per step.
struct FormPlan {
    form: String,
    task: Task,
    backend: ModuleBackend,
    start: ConstructionStage,
    birth_payload: BackendDef,
    specs: Vec<StepSpec>,
    seeds: &'static [u64],
    gap: Option<GrowthGap>,
}

/// Replay every step with its pinned seed.
///
/// # Panics
/// When a pinned seed's applied event fails the step's acceptance predicate.
fn qualify(plan: FormPlan) -> QualifiedPath {
    let FormPlan {
        form,
        task,
        backend,
        start,
        birth_payload,
        specs,
        seeds,
        gap,
    } = plan;
    assert_eq!(specs.len(), seeds.len(), "{form}: one pinned seed per step");
    let mut genome = start.genome.clone();
    let mut steps: Vec<PathStep> = Vec::with_capacity(specs.len());
    for (spec, &seed) in specs.iter().zip(seeds) {
        let after = accepted(&genome, spec.event, seed, &spec.accept)
            .unwrap_or_else(|| panic!("{form} {}: seed {seed} is not accepted", spec.name));
        let stage = stage_for(
            task,
            spec.name,
            spec.edits,
            Some(seed),
            &genome,
            after.clone(),
        );
        let previous = steps.last().map_or(&start.task, |step| &step.stage.task);
        steps.push(PathStep {
            event: spec.event,
            seed,
            surfaces_unchanged: surfaces_unchanged(previous, &stage.task),
            stage,
        });
        genome = after;
    }
    QualifiedPath {
        form,
        task,
        backend,
        birth_payload,
        start,
        steps,
        gap,
    }
}

/// One form's one-off seed search: the first accepted seed per step in
/// `0..SEARCH_RANGE`, `None` where the range is exhausted (the search stops
/// at the first exhausted step).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSearch {
    pub form: String,
    pub seeds: Vec<(&'static str, Option<u64>)>,
}

/// The one-off search behind every pinned seed: walk each form's plan taking
/// the first accepted seed per step. Slow; run by the ignored test only.
#[must_use]
pub fn search_seeds() -> Vec<SeedSearch> {
    form_plans()
        .into_iter()
        .map(|plan| {
            let mut genome = plan.start.genome.clone();
            let mut seeds = Vec::with_capacity(plan.specs.len());
            for spec in &plan.specs {
                let found = first_seed(&genome, spec.event, &spec.accept);
                seeds.push((spec.name, found.as_ref().map(|(seed, _)| *seed)));
                let Some((_, after)) = found else { break };
                genome = after;
            }
            SeedSearch {
                form: plan.form,
                seeds,
            }
        })
        .collect()
}

// ── Genome readers used by acceptance predicates ────────────────────────────

fn node(genome: &CreatureGenome, id: NodeId) -> &NodeGenome {
    genome.find_node(id).expect("fixture node exists")
}

fn vm(genome: &CreatureGenome) -> &VmBackendDef {
    match &node(genome, SCAFFOLD).backend_def {
        BackendDef::Vm(vm) => vm,
        BackendDef::Graph(_) => unreachable!("VM form"),
    }
}

fn graph(genome: &CreatureGenome) -> &CgpGraphBackendDef {
    match &node(genome, SCAFFOLD).backend_def {
        BackendDef::Graph(graph) => graph,
        BackendDef::Vm(_) => unreachable!("Graph form"),
    }
}

/// Only the scaffold node differs, and only in its backend definition.
fn only_scaffold_backend_changed(before: &CreatureGenome, after: &CreatureGenome) -> bool {
    let delta = GenomeDelta::between(before, after);
    delta.order_before == delta.order_after
        && delta.nodes.len() == 1
        && delta.nodes[0].node == SCAFFOLD
        && node(before, SCAFFOLD).input_refs == node(after, SCAFFOLD).input_refs
        && node(before, SCAFFOLD).targets == node(after, SCAFFOLD).targets
}

/// The entry node's first (statically winning) target is the scaffold.
fn scaffold_is_incumbent(after: &CreatureGenome) -> bool {
    node(after, NodeId::new(0)).targets[0].target_id == SCAFFOLD
}

fn cue_leaf(task: Task) -> GraphSource {
    GraphSource::InputLeaf {
        ref_idx: 0,
        sub_idx: match task {
            Task::A => 0,
            Task::B => 2,
        },
    }
}

fn appended<'a>(before: &[GraphEdge], after: &'a [GraphEdge]) -> Option<&'a GraphEdge> {
    (after.len() == before.len() + 1 && after[..before.len()] == *before)
        .then(|| &after[before.len()])
}

fn swap_activation() -> StepSpec {
    step(
        "activated",
        "Topology.SwapRouteTargets on the entry node: the tied scaffold becomes the static winner",
        ProductionEvent::Topology(TopologyOperator::SwapRouteTargets),
        |_, after| scaffold_is_incumbent(after),
    )
}

// ── Path plans per starting form ────────────────────────────────────────────

fn vm_copy_plan() -> Vec<StepSpec> {
    vec![
        step(
            "leading_halt_removed",
            "VmDeleteInstruction on the dormant copy's leading Halt",
            ProductionEvent::Vm(VmOperator::VmDeleteInstruction),
            |before, after| {
                only_scaffold_backend_changed(before, after)
                    && vm(after).program == vm(before).program[1..]
            },
        ),
        swap_activation(),
    ]
}

/// Graph copy and split: one vote edge from a cue-valued source (T19.F04).
fn graph_copy_plan(cue_sources: Vec<GraphSource>) -> Vec<StepSpec> {
    vec![east_vote_edge_added(cue_sources), swap_activation()]
}

/// `Graph.AddGraphEdge` onto the `Move(E)` vote sink from one of `sources`
/// with a positive weight, nothing else on the scaffold changed: the
/// activation of a Graph module is one edge into a vote sink (T19.F04).
fn east_vote_edge_added(sources: Vec<GraphSource>) -> StepSpec {
    step(
        "vote_edge_added",
        "Graph.AddGraphEdge onto the Move(E) vote sink from a cue-valued source, positive weight",
        ProductionEvent::Graph(GraphOperator::AddGraphEdge),
        move |before, after| {
            let (b, a) = (graph(before), graph(after));
            only_scaffold_backend_changed(before, after)
                && a.compute_nodes == b.compute_nodes
                && a.output_sinks.iter().zip(&b.output_sinks).all(|(x, y)| {
                    x.kind == OutputSinkKind::ActionVote(VoteSink::Move(EAST)) || x == y
                })
                && appended(&move_sink(b, EAST).inputs, &move_sink(a, EAST).inputs)
                    .is_some_and(|edge| sources.contains(&edge.source) && edge.weight > 0.0)
        },
    )
}

/// The Task B-correct VM copy: the ring added to the table, the read's
/// reference then its sub-index moved one raw-field nudge at a time, the
/// vote's `Move(N)` sink replaced by `Move(E)` in one
/// `VmInstructionMutation`, activation: six events.
fn vm_unprepared_plan() -> Vec<StepSpec> {
    let read = |ref_idx: u16, sub_idx: u16| VmInstruction::ReadInput {
        dst: 0,
        ref_idx,
        sub_idx,
    };
    let read_moved = move |before: &CreatureGenome, after: &CreatureGenome, to: VmInstruction| {
        only_scaffold_backend_changed(before, after)
            && vm(after).program[0] == to
            && vm(after).program[1..] == vm(before).program[1..]
            && vm(after).constants == vm(before).constants
    };
    /// The vote: `AddVote { sink: Move(N), src: 2 }` at this position of the
    /// copied incumbent program.
    const VOTE: usize = 3;
    vec![
        ring_added(),
        step(
            "read_ref_idx_1",
            "VmInstructionRawFieldMutation: ReadInput ref_idx 0 to 1 (the ring)",
            ProductionEvent::Vm(VmOperator::VmInstructionRawFieldMutation),
            move |before, after| read_moved(before, after, read(1, 0)),
        ),
        step(
            "read_sub_idx_1",
            "VmInstructionRawFieldMutation: ReadInput sub_idx 0 to 1",
            ProductionEvent::Vm(VmOperator::VmInstructionRawFieldMutation),
            move |before, after| read_moved(before, after, read(1, 1)),
        ),
        step(
            "read_sub_idx_2",
            "VmInstructionRawFieldMutation: ReadInput sub_idx 1 to 2 (east)",
            ProductionEvent::Vm(VmOperator::VmInstructionRawFieldMutation),
            move |before, after| read_moved(before, after, read(1, 2)),
        ),
        step(
            "vote_to_east",
            "VmInstructionMutation replace: the Move(N) vote becomes a Move(E) vote of the comparison",
            ProductionEvent::Vm(VmOperator::VmInstructionMutation),
            |before, after| {
                let (b, a) = (vm(before), vm(after));
                let mut expected = b.program.clone();
                expected[VOTE] = VOTE_EAST;
                only_scaffold_backend_changed(before, after)
                    && a.constants == b.constants
                    && a.register_count == b.register_count
                    && a.program == expected
            },
        ),
        swap_activation(),
    ]
}

/// The ring cue's leaf on the unprepared copy: the table entry `Add`
/// appended after the copied `FoodHere`, at the east sub-index.
const RING_EAST: GraphSource = GraphSource::InputLeaf {
    ref_idx: 1,
    sub_idx: 2,
};

/// The Task B-correct Graph copy: the ring added to the table, the sensing
/// edge to its east sub-index, the `Move(N)` vote edge removed and a
/// `Move(E)` vote edge added, activation: five events. The copied
/// `FoodHere` entry stays, unread.
fn graph_unprepared_plan() -> Vec<StepSpec> {
    let north_removed = step(
        "north_vote_removed",
        "Graph.RemoveGraphEdge: the Move(N) vote edge",
        ProductionEvent::Graph(GraphOperator::RemoveGraphEdge),
        |before, after| {
            let (b, a) = (graph(before), graph(after));
            only_scaffold_backend_changed(before, after)
                && a.compute_nodes == b.compute_nodes
                && move_sink(a, 0).inputs.is_empty()
                && a.output_sinks
                    .iter()
                    .zip(&b.output_sinks)
                    .all(|(x, y)| x.kind == OutputSinkKind::ActionVote(VoteSink::Move(0)) || x == y)
        },
    );
    vec![
        ring_added(),
        step(
            "cue_edge_retargeted",
            "Graph.RetargetGraphEdge: the sensing node's leaf edge to the ring entry, sub-index 2 (east)",
            ProductionEvent::Graph(GraphOperator::RetargetGraphEdge),
            |before, after| {
                let (b, a) = (graph(before), graph(after));
                only_scaffold_backend_changed(before, after)
                    && a.compute_nodes[0].inputs.len() == 1
                    && a.compute_nodes[0].inputs[0].source == RING_EAST
                    && a.compute_nodes[0].inputs[0].weight == b.compute_nodes[0].inputs[0].weight
                    && a.compute_nodes[1..] == b.compute_nodes[1..]
                    && a.output_sinks == b.output_sinks
            },
        ),
        north_removed,
        east_vote_edge_added(vec![GraphSource::ComputeNode(0), RING_EAST]),
        swap_activation(),
    ]
}

/// Blank Graph tissue: the sensor, then one vote edge from the cue into
/// `Move(E)`. A dispatched detour is exposed by the edge (two events);
/// undispatched blank tissue also needs the route swap (three).
fn graph_blank_plan(dispatched: bool) -> Vec<StepSpec> {
    let mut plan = vec![cue_added(), east_vote_edge_added(vec![cue_leaf(Task::A)])];
    if !dispatched {
        plan.push(swap_activation());
    }
    plan
}

const READ_CUE: VmInstruction = VmInstruction::ReadInput {
    dst: 0,
    ref_idx: 0,
    sub_idx: 0,
};
/// The blank program's vote: the cue register into `Move(E)`.
const VOTE_CUE_EAST: VmInstruction = VmInstruction::AddVote { sink: 3, src: 0 };
/// The reactive program's vote retargeted east: the comparison register into
/// `Move(E)`.
const VOTE_EAST: VmInstruction = VmInstruction::AddVote { sink: 3, src: 2 };
const _: () = assert!(VoteSink::Move(EAST).index() == 3);

/// `InputRef.Add` on the scaffold, accepted when the table becomes exactly
/// `expected` and nothing else on the node changes: the entry is wired into
/// nothing yet, so the step is neutral.
fn input_ref_added(
    name: &'static str,
    edits: &'static str,
    expected: Vec<InputReference>,
) -> StepSpec {
    step(
        name,
        edits,
        ProductionEvent::InputRef(InputRefOperator::Add),
        move |before, after| {
            let delta = GenomeDelta::between(before, after);
            delta.nodes.len() == 1
                && delta.nodes[0].node == SCAFFOLD
                && node(after, SCAFFOLD).input_refs == expected
                && node(after, SCAFFOLD).backend_def == node(before, SCAFFOLD).backend_def
        },
    )
}

fn cue_added() -> StepSpec {
    input_ref_added(
        "cue_added",
        "InputRef.Add on the scaffold: FoodHere (type 0)",
        vec![cue(Task::A)],
    )
}

/// T11.F22: `Swap` stays within a kind, so the copied `FoodHere` entry
/// cannot become the ring in one event; the ring is added beside it and each
/// consumer moves to it on its own.
fn ring_added() -> StepSpec {
    input_ref_added(
        "ring_added",
        "InputRef.Add on the copy: NeighborFoodRing (type 0) beside the copied FoodHere",
        vec![cue(Task::A), cue(Task::B)],
    )
}

fn insert_step(name: &'static str, edits: &'static str, expected: Vec<VmInstruction>) -> StepSpec {
    step(
        name,
        edits,
        ProductionEvent::Vm(VmOperator::VmInstructionMutation),
        move |before, after| {
            only_scaffold_backend_changed(before, after)
                && vm(after).constants == vm(before).constants
                && vm(after).register_count == vm(before).register_count
                && vm(after).program == expected
        },
    )
}

/// The one-register Task A program: read the cue, vote it into `Move(E)`.
/// Two instructions, each one `VmInstructionMutation` insert before the
/// blank program's `Halt`.
///
/// Blank tissue is not dispatched, so the route swap exposes it: four
/// events. A dispatched detour is exposed by the vote insert: three.
fn vm_blank_plan(dispatched: bool) -> Vec<StepSpec> {
    let mut plan = vec![
        cue_added(),
        insert_step(
            "read_cue",
            "VmInstructionMutation insert: ReadInput of the cue into register 0",
            vec![READ_CUE, VmInstruction::Halt],
        ),
        insert_step(
            "vote_east",
            "VmInstructionMutation insert: AddVote of the cue into Move(E); the module now votes",
            vec![READ_CUE, VOTE_CUE_EAST, VmInstruction::Halt],
        ),
    ];
    if !dispatched {
        plan.push(swap_activation());
    }
    plan
}

/// Pinned `Topology.AddNode` seeds drawing each blank backend on the
/// entry-to-incumbent edge: the first accepted in the search range.
fn detour_seed(backend: ModuleBackend) -> u64 {
    match backend {
        ModuleBackend::Graph => 1,
        ModuleBackend::Vm => 0,
    }
}

fn detour_start(backend: ModuleBackend, base: &CreatureGenome) -> ConstructionStage {
    let accept: Accept = Box::new(move |_, after: &CreatureGenome| {
        after.nodes.len() == 3
            && node(after, NodeId::new(0)).targets[0].target_id == SCAFFOLD
            && node(after, SCAFFOLD).targets.len() == 1
            && node(after, SCAFFOLD).targets[0].target_id == NodeId::new(1)
            && match &node(after, SCAFFOLD).backend_def {
                BackendDef::Graph(_) => backend == ModuleBackend::Graph,
                BackendDef::Vm(_) => backend == ModuleBackend::Vm,
            }
    });
    let seed = detour_seed(backend);
    let genome = accepted(
        base,
        ProductionEvent::Topology(TopologyOperator::AddNode),
        seed,
        &accept,
    )
    .expect("the pinned AddNode seed draws the blank backend on the entry edge");
    stage_for(
        Task::A,
        "inline_detour",
        "production Topology.AddNode on the entry-to-incumbent edge; the detour is dispatched every tick and forwards to the incumbent",
        Some(seed),
        base,
        genome,
    )
}

/// Split-form cue sources: any compute node that reads the cue leaf at weight
/// 1.0 and forwards it unchanged (the sensing WeightedSum or the identity Add
/// the split inserted before it).
fn cue_valued_compute_sources(graph: &CgpGraphBackendDef) -> Vec<GraphSource> {
    graph
        .compute_nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node.inputs.len() == 1
                && node.inputs[0].weight == 1.0
                && matches!(
                    node.inputs[0].source,
                    GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0
                    }
                )
        })
        .map(|(index, _)| GraphSource::ComputeNode(index as u16))
        .collect()
}

// ── Pinned seeds: the first accepted in `0..SEARCH_RANGE` per step ──────────
//
// Re-pinned by T19.F04: the plans are re-expressed on votes (a Graph module
// activates with one edge into the `Move(E)` vote sink, a VM module with one
// `AddVote`), the fresh-instruction draw lost four opcodes and gained
// `AddVote`, and the edge-surface draw covers the vote and parameter sinks.
// Re-pinned by T19.F05: the input-reference draw grows from 22 to 27
// entries, which moves the cue and ring additions.

const SWAP: u64 = 0;
const GRAPH_BLANK_SEEDS: &[u64] = &[201, 57, SWAP];
const GRAPH_DETOUR_SEEDS: &[u64] = &[201, 57];
const GRAPH_COPY_SEEDS: &[u64] = &[103, SWAP];
const GRAPH_SPLIT_SEEDS: &[u64] = &[103, SWAP];
const VM_COPY_SEEDS: &[u64] = &[1, SWAP];
/// Ring, retarget, remove north, add east, swap.
const GRAPH_UNPREPARED_SEEDS: &[u64] = &[1, 32, 5, 103, SWAP];
/// Ring, ref_idx, sub_idx, sub_idx, vote to east, swap.
const VM_UNPREPARED_SEEDS: &[u64] = &[1, 18, 25, 25, 66_421, SWAP];
/// Cue, read, vote, swap.
const VM_BLANK_SEEDS: &[u64] = &[201, 1279, 2189, SWAP];
/// Cue, read, vote.
const VM_DETOUR_SEEDS: &[u64] = &[201, 1279, 2189];

/// Every starting form's plan with its pinned seeds, in the fixed family
/// order.
fn form_plans() -> Vec<FormPlan> {
    let mut plans = Vec::with_capacity(9);
    let mut bases = Vec::new();
    for start in starting_forms() {
        let name = start.name.as_str();
        let last = start.history.last().expect("history").clone();
        let (specs, seeds, gap) = match name {
            "graph_blank" => (graph_blank_plan(false), GRAPH_BLANK_SEEDS, None),
            "vm_blank" => (vm_blank_plan(false), VM_BLANK_SEEDS, None),
            "graph_copy" => (
                graph_copy_plan(vec![GraphSource::ComputeNode(0), cue_leaf(Task::A)]),
                GRAPH_COPY_SEEDS,
                None,
            ),
            "vm_copy" => (vm_copy_plan(), VM_COPY_SEEDS, None),
            "graph_split" => {
                let BackendDef::Graph(graph) = &node(&start.genome, SCAFFOLD).backend_def else {
                    unreachable!()
                };
                (
                    graph_copy_plan(cue_valued_compute_sources(graph)),
                    GRAPH_SPLIT_SEEDS,
                    None,
                )
            }
            "graph_unprepared" => (graph_unprepared_plan(), GRAPH_UNPREPARED_SEEDS, None),
            "vm_unprepared" => (vm_unprepared_plan(), VM_UNPREPARED_SEEDS, None),
            // F02's prepared controls already dispatch: no path to qualify.
            "graph_prepared" | "vm_prepared" => continue,
            _ => unreachable!("unplanned form {name}"),
        };
        if name.ends_with("_blank") {
            bases.push((start.backend, start.creation_base.clone()));
        }
        plans.push(FormPlan {
            form: name.into(),
            task: start.task,
            backend: start.backend,
            // Authored history is construction, not mutation: the form's
            // starting payload is the scaffold's birth payload.
            birth_payload: node(&last.genome, SCAFFOLD).backend_def.clone(),
            start: last,
            specs,
            seeds,
            gap,
        });
    }
    for (backend, base) in bases {
        let (specs, seeds) = match backend {
            ModuleBackend::Graph => (graph_blank_plan(true), GRAPH_DETOUR_SEEDS),
            ModuleBackend::Vm => (vm_blank_plan(true), VM_DETOUR_SEEDS),
        };
        let start = detour_start(backend, &base);
        plans.push(FormPlan {
            form: format!("{}_detour", backend.as_key()),
            task: Task::A,
            backend,
            birth_payload: node(&start.genome, SCAFFOLD).backend_def.clone(),
            start,
            specs,
            seeds,
            gap: None,
        });
    }
    plans
}

/// Every starting form's pinned-seed path, in the fixed family order.
#[must_use]
pub fn qualified_paths() -> Vec<QualifiedPath> {
    form_plans().into_iter().map(qualify).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `form`'s step specs and the genome before each step, then the last
    /// step's result, replayed with the pinned seeds.
    fn replay(form: &str) -> (Vec<StepSpec>, Vec<CreatureGenome>) {
        let plan = form_plans()
            .into_iter()
            .find(|plan| plan.form == form)
            .expect("the form is planned");
        let mut genomes = vec![plan.start.genome.clone()];
        for (spec, &seed) in plan.specs.iter().zip(plan.seeds) {
            let before = genomes.last().expect("the start is present");
            let after = accepted(before, spec.event, seed, &spec.accept)
                .expect("the pinned seed is accepted");
            genomes.push(after);
        }
        (plan.specs, genomes)
    }

    fn step_index(specs: &[StepSpec], name: &str) -> usize {
        specs
            .iter()
            .position(|spec| spec.name == name)
            .expect("the step is planned")
    }

    fn scaffold_graph_mut(genome: &mut CreatureGenome) -> &mut CgpGraphBackendDef {
        let scaffold = genome
            .nodes
            .iter_mut()
            .find(|node| node.node_id == SCAFFOLD)
            .expect("fixture node exists");
        match &mut scaffold.backend_def {
            BackendDef::Graph(graph) => graph,
            BackendDef::Vm(_) => unreachable!("Graph form"),
        }
    }

    /// The added `Move(E)` vote edge must carry a strictly positive weight:
    /// the same edge at weight zero is rejected.
    #[test]
    fn east_vote_edge_added_rejects_a_zero_weight_edge() {
        let (specs, genomes) = replay("graph_unprepared");
        let index = step_index(&specs, "vote_edge_added");
        let accept = &specs[index].accept;
        let (before, after) = (&genomes[index], &genomes[index + 1]);
        assert!(accept(before, after));

        let mut zero_weight = after.clone();
        scaffold_graph_mut(&mut zero_weight)
            .sink_mut(OutputSinkKind::ActionVote(VoteSink::Move(EAST)))
            .expect("the fixed catalog holds every Move sink")
            .inputs
            .last_mut()
            .expect("the step appended an edge")
            .weight = 0.0;

        assert!(!accept(before, &zero_weight));
    }

    /// Removing the `Move(N)` vote edge qualifies only when the compute
    /// nodes are untouched: the same removal with a changed compute-node
    /// edge weight is rejected.
    #[test]
    fn north_vote_removed_rejects_a_changed_compute_node() {
        let (specs, genomes) = replay("graph_unprepared");
        let index = step_index(&specs, "north_vote_removed");
        let accept = &specs[index].accept;
        let (before, after) = (&genomes[index], &genomes[index + 1]);
        assert!(accept(before, after));

        let mut changed_compute = after.clone();
        scaffold_graph_mut(&mut changed_compute).compute_nodes[0].inputs[0].weight += 1.0;

        assert!(!accept(before, &changed_compute));
    }
}

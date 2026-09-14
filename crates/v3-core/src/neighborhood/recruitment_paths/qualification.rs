//! T13.F05: seed-selected production-operator paths from each fixed starting
//! form to a useful, bypass-sensitive contribution. Observation only.

use super::fixtures::{cue, stage_for};
use super::*;
use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, ComputeNodeKind, GraphEdge, GraphSource,
    WorldActionKind,
};
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
use crate::runtime::vm::jump_target;
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
    pub steps: Vec<PathStep>,
    pub gap: Option<GrowthGap>,
}

impl QualifiedPath {
    /// Complete within the bound.
    #[must_use]
    pub fn qualified(&self) -> bool {
        self.gap.is_none() && self.steps.len() <= MAX_PATH_EVENTS
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

/// Weight the edges of one surface contribute when the cue reads 1.0.
fn cue_weight(edges: &[GraphEdge], sources: &[GraphSource]) -> f32 {
    edges
        .iter()
        .filter(|edge| sources.contains(&edge.source))
        .map(|edge| edge.weight)
        .sum()
}

/// A Move parameter of `[1.5, 2.5)` decodes to east.
fn decodes_east(weight: f32) -> bool {
    (1.5..2.5).contains(&weight)
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

/// Graph copy and split: one gate edge from a cue-valued compute node.
fn graph_copy_plan(cue_sources: Vec<GraphSource>) -> Vec<StepSpec> {
    vec![
        step(
            "gate_edge_added",
            "Graph.AddGraphEdge onto action slot 0's gate from a cue-valued source, positive weight",
            ProductionEvent::Graph(GraphOperator::AddGraphEdge),
            move |before, after| {
                only_scaffold_backend_changed(before, after)
                    && appended(
                        &graph(before).action_bank[0].gate_inputs,
                        &graph(after).action_bank[0].gate_inputs,
                    )
                    .is_some_and(|edge| cue_sources.contains(&edge.source) && edge.weight > 0.0)
            },
        ),
        swap_activation(),
    ]
}

fn vm_unprepared_plan() -> Vec<StepSpec> {
    let read = |sub_idx: u16| VmInstruction::ReadInput {
        dst: 0,
        ref_idx: 0,
        sub_idx,
    };
    let sub_idx_moved = move |before: &CreatureGenome, after: &CreatureGenome, to: u16| {
        only_scaffold_backend_changed(before, after)
            && vm(after).program[0] == read(to)
            && vm(after).program[1..] == vm(before).program[1..]
            && vm(after).constants == vm(before).constants
    };
    let constant_moved = |before: &CreatureGenome, after: &CreatureGenome| {
        only_scaffold_backend_changed(before, after)
            && vm(after).program == vm(before).program
            && vm(after).constants[0] == vm(before).constants[0]
    };
    vec![
        cue_swapped(),
        step(
            "read_sub_idx_1",
            "VmInstructionRawFieldMutation: ReadInput sub_idx 0 to 1",
            ProductionEvent::Vm(VmOperator::VmInstructionRawFieldMutation),
            move |before, after| sub_idx_moved(before, after, 1),
        ),
        step(
            "read_sub_idx_2",
            "VmInstructionRawFieldMutation: ReadInput sub_idx 1 to 2 (east)",
            ProductionEvent::Vm(VmOperator::VmInstructionRawFieldMutation),
            move |before, after| sub_idx_moved(before, after, 2),
        ),
        step(
            "direction_half",
            "VmConstantMutation on the direction constant: at least 0.5 of the way to east",
            ProductionEvent::Vm(VmOperator::VmConstantMutation),
            move |before, after| constant_moved(before, after) && vm(after).constants[1] >= 0.5,
        ),
        step(
            "direction_east",
            "VmConstantMutation on the direction constant: lands in [1.5, 2.5)",
            ProductionEvent::Vm(VmOperator::VmConstantMutation),
            move |before, after| {
                constant_moved(before, after) && decodes_east(vm(after).constants[1])
            },
        ),
        swap_activation(),
    ]
}

/// Compute kinds whose single- or two-input output is the weighted sum of
/// a non-negative cue, so a node of that kind can carry the east code.
fn sums_inputs(kind: &ComputeNodeKind) -> bool {
    matches!(
        kind,
        ComputeNodeKind::Add | ComputeNodeKind::WeightedSum | ComputeNodeKind::Relu
    )
}

/// The Move direction is `param_inputs[0]` alone (positional, not summed)
/// and an edge weight is drawn in `[-1, 1]`, so east (2) needs a compute
/// node reading the cue twice: a bootstrap node with one cue edge, a second
/// cue edge onto it, then the parameter edge that reads the node. `at` is
/// the index the new node takes; `param_step` supplies the third event.
fn direction_node_steps(
    cue_sources: Vec<GraphSource>,
    at: usize,
    param_step: StepSpec,
) -> Vec<StepSpec> {
    let sources = cue_sources.clone();
    let bootstrap = step(
        "direction_node",
        "Graph.AddInternalGraphNode bootstrap form: a summing node reading the cue at weight at least 0.9",
        ProductionEvent::Graph(GraphOperator::AddInternalGraphNode),
        move |before, after| {
            let (b, a) = (graph(before), graph(after));
            only_scaffold_backend_changed(before, after)
                && a.compute_nodes.len() == b.compute_nodes.len() + 1
                && a.compute_nodes[..at] == b.compute_nodes[..at]
                && a.action_bank == b.action_bank
                && a.output_sinks == b.output_sinks
                && a.execute_gate == b.execute_gate
                && sums_inputs(&a.compute_nodes[at].kind)
                && a.compute_nodes[at].plasticity.is_none()
                && matches!(
                    a.compute_nodes[at].inputs.as_slice(),
                    [edge] if sources.contains(&edge.source) && edge.weight >= 0.9
                )
        },
    );
    let sources = cue_sources;
    let second_edge = step(
        "direction_doubled",
        "Graph.AddGraphEdge onto the direction node from the cue: cue-weighted sum at least 1.7",
        ProductionEvent::Graph(GraphOperator::AddGraphEdge),
        move |before, after| {
            only_scaffold_backend_changed(before, after)
                && appended(
                    &graph(before).compute_nodes[at].inputs,
                    &graph(after).compute_nodes[at].inputs,
                )
                .is_some_and(|edge| sources.contains(&edge.source))
                && cue_weight(&graph(after).compute_nodes[at].inputs, &sources) >= 1.7
        },
    );
    vec![bootstrap, second_edge, param_step]
}

fn cue_swapped() -> StepSpec {
    step(
        "cue_swapped",
        "InputRef.Swap on the copy: FoodHere to NeighborFoodRing (type 0)",
        ProductionEvent::InputRef(InputRefOperator::Swap),
        |before, after| {
            let delta = GenomeDelta::between(before, after);
            delta.nodes.len() == 1
                && delta.nodes[0].node == SCAFFOLD
                && node(after, SCAFFOLD).input_refs == [cue(Task::B)]
                && node(after, SCAFFOLD).backend_def == node(before, SCAFFOLD).backend_def
        },
    )
}

/// The Task A-correct copy: cue to the ring, its sensing edge to the east
/// sub-index, a direction node in place of the zeroed constant, activation.
fn graph_unprepared_plan() -> Vec<StepSpec> {
    let sources = vec![GraphSource::ComputeNode(0), cue_leaf(Task::B)];
    let retarget_param = step(
        "direction_read",
        "Graph.RetargetGraphEdge: the Move parameter edge reads the direction node instead of the zeroed constant",
        ProductionEvent::Graph(GraphOperator::RetargetGraphEdge),
        |before, after| {
            let (b, a) = (graph(before), graph(after));
            only_scaffold_backend_changed(before, after)
                && a.compute_nodes == b.compute_nodes
                && a.output_sinks == b.output_sinks
                && a.execute_gate == b.execute_gate
                && a.action_bank[1..] == b.action_bank[1..]
                && a.action_bank[0].gate_inputs == b.action_bank[0].gate_inputs
                && a.action_bank[0].param_inputs.len() == 1
                && a.action_bank[0].param_inputs[0].source == GraphSource::ComputeNode(2)
                && a.action_bank[0].param_inputs[0].weight == b.action_bank[0].param_inputs[0].weight
        },
    );
    let mut plan = vec![
        cue_swapped(),
        step(
            "cue_edge_retargeted",
            "Graph.RetargetGraphEdge: the sensing node's leaf edge to ring sub-index 2 (east)",
            ProductionEvent::Graph(GraphOperator::RetargetGraphEdge),
            |before, after| {
                let (b, a) = (graph(before), graph(after));
                only_scaffold_backend_changed(before, after)
                    && a.compute_nodes[0].inputs.len() == 1
                    && a.compute_nodes[0].inputs[0].source == cue_leaf(Task::B)
                    && a.compute_nodes[0].inputs[0].weight == b.compute_nodes[0].inputs[0].weight
                    && a.compute_nodes[1..] == b.compute_nodes[1..]
                    && a.action_bank == b.action_bank
                    && a.execute_gate == b.execute_gate
                    && a.output_sinks == b.output_sinks
            },
        ),
    ];
    plan.extend(direction_node_steps(sources, 2, retarget_param));
    plan.push(swap_activation());
    plan
}

/// Blank Graph tissue: sensor, Move behavior, the three-event direction
/// node, then the gate edge. A dispatched detour is exposed by the gate edge
/// (six events); undispatched blank tissue also needs the route swap (seven,
/// a recorded growth gap).
fn graph_blank_plan(dispatched: bool) -> Vec<StepSpec> {
    let leaf = cue_leaf(Task::A);
    let param_edge = step(
        "direction_read",
        "Graph.AddGraphEdge onto action slot 0's parameter from the direction node: product in [1.5, 2.5)",
        ProductionEvent::Graph(GraphOperator::AddGraphEdge),
        move |before, after| {
            only_scaffold_backend_changed(before, after)
                && appended(
                    &graph(before).action_bank[0].param_inputs,
                    &graph(after).action_bank[0].param_inputs,
                )
                .is_some_and(|edge| {
                    edge.source == GraphSource::ComputeNode(0)
                        && decodes_east(
                            edge.weight * cue_weight(&graph(after).compute_nodes[0].inputs, &[leaf]),
                        )
                })
        },
    );
    let mut plan = vec![
        cue_added(),
        step(
            "slot_emits_move",
            "Graph.MutateActionSlotBehavior: action slot 0 NoOp to Emit(Move)",
            ProductionEvent::Graph(GraphOperator::MutateActionSlotBehavior),
            |before, after| {
                let (b, a) = (graph(before), graph(after));
                only_scaffold_backend_changed(before, after)
                    && a.action_bank[0].behavior == ActionSlotBehavior::Emit(WorldActionKind::Move)
                    && a.action_bank[1..] == b.action_bank[1..]
            },
        ),
    ];
    plan.extend(direction_node_steps(vec![leaf], 0, param_edge));
    plan.push(step(
        "gate_edge_added",
        "Graph.AddGraphEdge onto action slot 0's gate from the cue, positive weight",
        ProductionEvent::Graph(GraphOperator::AddGraphEdge),
        move |before, after| {
            only_scaffold_backend_changed(before, after)
                && appended(
                    &graph(before).action_bank[0].gate_inputs,
                    &graph(after).action_bank[0].gate_inputs,
                )
                .is_some_and(|edge| {
                    (edge.source == leaf || edge.source == GraphSource::ComputeNode(0))
                        && edge.weight > 0.0
                })
        },
    ));
    if !dispatched {
        plan.push(swap_activation());
    }
    plan
}

/// One expected instruction of a VM program under construction.
#[derive(Clone)]
enum Expect {
    Exact(VmInstruction),
    /// A `JumpIfZero` on register 0 that resolves to the program's closing
    /// Halt. The drawn offset is any of the `-16..=16` values landing there
    /// modulo the length; every later insert before the Halt goes through
    /// `splice_program_with_reference_repair`, which rewrites the offset so
    /// the jump keeps that target.
    JumpToHalt,
}

fn program_matches(program: &[VmInstruction], expected: &[Expect]) -> bool {
    let len = program.len();
    len == expected.len()
        && program
            .iter()
            .enumerate()
            .zip(expected)
            .all(|((pc, actual), expected)| match expected {
                Expect::Exact(instruction) => actual == instruction,
                Expect::JumpToHalt => matches!(
                    actual,
                    VmInstruction::JumpIfZero { cond: 0, offset }
                        if jump_target(pc, *offset, len) + 1 == len
                ),
            })
}

const READ_CUE: VmInstruction = VmInstruction::ReadInput {
    dst: 0,
    ref_idx: 0,
    sub_idx: 0,
};
const DOUBLE: VmInstruction = VmInstruction::Add { dst: 0, a: 0, b: 0 };
const WRITE_DIRECTION: VmInstruction = VmInstruction::WriteWorldActionMeta {
    slot_idx: 0,
    src: 0,
};
const PUSH_MOVE: VmInstruction = VmInstruction::PushAction { action_type: 2 };

fn cue_added() -> StepSpec {
    step(
        "cue_added",
        "InputRef.Add on the scaffold: FoodHere (type 0)",
        ProductionEvent::InputRef(InputRefOperator::Add),
        |before, after| {
            let delta = GenomeDelta::between(before, after);
            delta.nodes.len() == 1
                && delta.nodes[0].node == SCAFFOLD
                && node(after, SCAFFOLD).input_refs == [cue(Task::A)]
                && node(after, SCAFFOLD).backend_def == node(before, SCAFFOLD).backend_def
        },
    )
}

fn insert_step(name: &'static str, edits: &'static str, expected: Vec<Expect>) -> StepSpec {
    step(
        name,
        edits,
        ProductionEvent::Vm(VmOperator::VmInstructionMutation),
        move |before, after| {
            only_scaffold_backend_changed(before, after)
                && vm(after).constants == vm(before).constants
                && vm(after).register_count == vm(before).register_count
                && program_matches(&vm(after).program, &expected)
        },
    )
}

/// The one-register Task A program: read the cue, skip to the Halt when it
/// is zero, double it to the east direction code, write the Move parameter,
/// push the Move. Five instructions, each one `VmInstructionMutation` insert.
///
/// Blank tissue is not dispatched, so the program is built in reading order
/// and the route swap exposes it: seven events, a recorded growth gap. A
/// dispatched detour builds the neutral instructions first, adds the jump
/// to the Halt (a zero cue then does nothing at length five), and exposes
/// the module by inserting the push last: six events.
fn vm_blank_plan(dispatched: bool) -> Vec<StepSpec> {
    use Expect::{Exact, JumpToHalt};
    /// The finished program in position order: each instruction's insert
    /// step name and edit note.
    const PROGRAM: [(&str, &str); 5] = [
        (
            "read_cue",
            "VmInstructionMutation insert: ReadInput of the cue into register 0",
        ),
        (
            "skip_when_zero",
            "VmInstructionMutation insert: JumpIfZero on the cue, landing on the Halt",
        ),
        (
            "double_to_east",
            "VmInstructionMutation insert: Add doubling the cue to the east code 2",
        ),
        (
            "write_direction",
            "VmInstructionMutation insert: WriteWorldActionMeta slot 0 from register 0",
        ),
        (
            "push_move",
            "VmInstructionMutation insert: PushAction(Move); the module now emits",
        ),
    ];
    let expects = [
        Exact(READ_CUE),
        JumpToHalt,
        Exact(DOUBLE),
        Exact(WRITE_DIRECTION),
        Exact(PUSH_MOVE),
    ];
    // Program positions in insertion order.
    let order: [usize; 5] = if dispatched {
        [0, 2, 3, 1, 4]
    } else {
        [0, 1, 2, 3, 4]
    };
    let mut plan = vec![cue_added()];
    for inserted in 1..=order.len() {
        let present = &order[..inserted];
        let (name, edits) = PROGRAM[present[inserted - 1]];
        let expected = (0..expects.len())
            .filter(|position| present.contains(position))
            .map(|position| expects[position].clone())
            .chain([Exact(VmInstruction::Halt)])
            .collect();
        plan.push(insert_step(name, edits, expected));
    }
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

/// Undispatched blank tissue needs every event a dispatched detour needs
/// plus the route swap that dispatches it.
fn blank_growth_gap(backend: ModuleBackend) -> GrowthGap {
    GrowthGap {
        length: 7,
        lengthening_step: match backend {
            ModuleBackend::Graph => "activation: the sensor, the Move behavior, the three-event direction node (a positional Move parameter reads one edge of weight at most 1, so east needs a node summing two cue edges), the gate edge, then the route swap",
            ModuleBackend::Vm => "activation: the sensor, the five-instruction program (one VmInstructionMutation insert each; the motif pairs carry none of the jump, meta write or push), then the route swap",
        }
        .into(),
    }
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

const SWAP: u64 = 0;
const GRAPH_BLANK_SEEDS: &[u64] = &[1, 25, 1020, 1650, 3612, 102, SWAP];
const GRAPH_DETOUR_SEEDS: &[u64] = &[1, 25, 1020, 1650, 3612, 102];
const GRAPH_COPY_SEEDS: &[u64] = &[102, SWAP];
const GRAPH_SPLIT_SEEDS: &[u64] = &[1762, SWAP];
const VM_COPY_SEEDS: &[u64] = &[1, SWAP];
const GRAPH_UNPREPARED_SEEDS: &[u64] = &[25, 32, 64, 6718, 4, SWAP];
const VM_UNPREPARED_SEEDS: &[u64] = &[25, 72, 223, 13, 13, SWAP];
/// Reading order: cue, read, jump, double, write, push, swap.
const VM_BLANK_SEEDS: &[u64] = &[1, 238, 9940, 800, 41_854, 4126, SWAP];
/// Neutral-first order: cue, read, double, write, jump, push.
const VM_DETOUR_SEEDS: &[u64] = &[1, 238, 800, 21_017, 3709, 4126];

/// Every starting form's plan with its pinned seeds, in the fixed family
/// order.
fn form_plans() -> Vec<FormPlan> {
    let mut plans = Vec::with_capacity(9);
    let mut bases = Vec::new();
    for start in starting_forms() {
        let name = start.name.as_str();
        let last = start.history.last().expect("history").clone();
        let (specs, seeds, gap) = match name {
            "graph_blank" => (
                graph_blank_plan(false),
                GRAPH_BLANK_SEEDS,
                Some(blank_growth_gap(start.backend)),
            ),
            "vm_blank" => (
                vm_blank_plan(false),
                VM_BLANK_SEEDS,
                Some(blank_growth_gap(start.backend)),
            ),
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
        plans.push(FormPlan {
            form: format!("{}_detour", backend.as_key()),
            task: Task::A,
            backend,
            start: detour_start(backend, &base),
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

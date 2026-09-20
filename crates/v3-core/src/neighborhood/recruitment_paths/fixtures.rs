use super::*;
use crate::config::MutationConfig;
use crate::contracts::{InputReference, NodeId, OrdinaryFoodTypeId, RouteTarget, WorldInputKey};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    WorldActionKind,
};
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::graph::operators::split_existing_edge;
use crate::mutation::reachability::TargetSets;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::{
    MutationDomain, MutationEventOutcome, MutationEventRecord, MutationOperator, MutationSummary,
};
use crate::neighborhood::recruitment::ModuleBackend;
use rand::{rngs::SmallRng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionStage {
    pub name: String,
    pub edits: String,
    pub seed: Option<u64>,
    pub delta: GenomeDelta,
    pub genome: CreatureGenome,
    pub task: TaskReading,
    pub battery: super::super::Signature,
    pub battery_class: String,
    pub incumbent_actions_unchanged: bool,
    pub useful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructedPath {
    pub backend: ModuleBackend,
    pub base: CreatureGenome,
    pub stages: Vec<ConstructionStage>,
    pub copy_stages: Vec<ConstructionStage>,
    pub split_stage: Option<ConstructionStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Start {
    pub name: String,
    pub task: Task,
    pub backend: ModuleBackend,
    pub scaffold: NodeId,
    pub genome: CreatureGenome,
    pub creation_base: CreatureGenome,
    pub history: Vec<ConstructionStage>,
    pub creation_operator: Option<MutationOperator>,
    pub preparation_difference: Option<GenomeDelta>,
    pub mutable_sites: MutableSites,
    pub genome_size: u32,
    pub task_reading: TaskReading,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutableSites {
    pub input_refs: usize,
    pub route_targets: usize,
    pub vm_instructions: usize,
    pub vm_constants: usize,
    pub graph_compute_nodes: usize,
    pub graph_edges: usize,
    pub graph_action_slots: usize,
}

fn sites(genome: &CreatureGenome) -> MutableSites {
    let mut sites = MutableSites {
        input_refs: 0,
        route_targets: 0,
        vm_instructions: 0,
        vm_constants: 0,
        graph_compute_nodes: 0,
        graph_edges: 0,
        graph_action_slots: 0,
    };
    for node in &genome.nodes {
        sites.input_refs += node.input_refs.len();
        sites.route_targets += node.targets.len();
        match &node.backend_def {
            BackendDef::Vm(vm) => {
                sites.vm_instructions += vm.program.len();
                sites.vm_constants += vm.constants.len();
            }
            BackendDef::Graph(graph) => {
                sites.graph_compute_nodes += graph.compute_nodes.len();
                sites.graph_action_slots += graph.action_bank.len();
                sites.graph_edges += graph
                    .compute_nodes
                    .iter()
                    .map(|node| node.inputs.len())
                    .sum::<usize>()
                    + graph
                        .output_sinks
                        .iter()
                        .map(|sink| sink.inputs.len())
                        .sum::<usize>()
                    + graph
                        .action_bank
                        .iter()
                        .map(|slot| slot.gate_inputs.len() + slot.param_inputs.len())
                        .sum::<usize>()
                    + graph.execute_gate.inputs.len();
            }
        }
    }
    sites
}

pub(super) fn cue(task: Task) -> InputReference {
    InputReference::World(match task {
        Task::A => WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        },
        Task::B => WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::default(),
        },
    })
}

fn edge(source: GraphSource, weight: f32) -> GraphEdge {
    GraphEdge { source, weight }
}

fn blank(backend: ModuleBackend) -> BackendDef {
    match backend {
        ModuleBackend::Graph => BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(
            &MutationConfig::default(),
        )),
        // Identical to topology::birth::minimal_vm_backend's constructor.
        ModuleBackend::Vm => BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
    }
}

fn reactive(backend: ModuleBackend, live: bool) -> NodeGenome {
    let backend_def = match backend {
        ModuleBackend::Vm => {
            let mut program = vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::LoadConst {
                    dst: 1,
                    const_idx: 0,
                },
                VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
                VmInstruction::JumpIfZero { cond: 2, offset: 4 },
                VmInstruction::LoadConst {
                    dst: 3,
                    const_idx: 1,
                },
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 3,
                },
                VmInstruction::PushAction { action_type: 2 },
                VmInstruction::ExecuteActionQueue,
                VmInstruction::PushAction { action_type: 0 },
                VmInstruction::ExecuteActionQueue,
            ];
            if !live {
                program.insert(0, VmInstruction::Halt);
            }
            BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![0.0, 2.0],
                program,
            })
        }
        ModuleBackend::Graph => {
            let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
            graph.compute_nodes = vec![
                ComputeNode {
                    kind: ComputeNodeKind::WeightedSum,
                    inputs: vec![edge(
                        GraphSource::InputLeaf {
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        1.0,
                    )],
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Constant(2.0),
                    inputs: vec![],
                    plasticity: None,
                },
            ];
            graph.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Move);
            graph.action_bank[0].param_inputs = vec![edge(GraphSource::ComputeNode(1), 1.0)];
            if live {
                graph.action_bank[0].gate_inputs = vec![edge(GraphSource::ComputeNode(0), 1.0)];
            }
            graph.execute_gate.inputs = vec![edge(GraphSource::ComputeNode(1), 1.0)];
            BackendDef::Graph(graph)
        }
    };
    NodeGenome {
        node_id: NodeId::new(1),
        input_refs: vec![cue(Task::A)],
        backend_def,
        targets: vec![],
    }
}

pub(super) fn base(backend: ModuleBackend, live: bool) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: blank(ModuleBackend::Vm),
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            },
            reactive(backend, live),
        ],
    }
}

pub(super) fn topology(
    genome: &mut CreatureGenome,
    operator: TopologyOperator,
    seed: u64,
) -> MutationSummary {
    let reachable = mesh_reachable_nodes(genome);
    let mut selector = TargetSets::new(&reachable, &[]).selector(0.0, 0.0);
    let result = TopologyMutator::apply_with_food_type_count(
        genome,
        operator,
        &mut selector,
        &mut SmallRng::seed_from_u64(seed),
        &MutationConfig::default(),
        1,
    )
    .expect("fixed constructed topology operator applies");
    let operator = match operator {
        TopologyOperator::CopyNode => MutationOperator::TopologyCopyNode,
        TopologyOperator::SwapRouteTargets => MutationOperator::TopologySwapRouteTargets,
        _ => unreachable!("only copy and activation are constructed here"),
    };
    let mut summary = MutationSummary::zero();
    summary.record_attempt(MutationDomain::Topology, operator);
    summary.record_applied(MutationDomain::Topology, operator);
    summary.record_reachability(result);
    summary.record_event(MutationEventRecord {
        domain: MutationDomain::Topology,
        operator: Some(operator),
        target: selector
            .first_pick()
            .map(|index| genome.nodes[index].node_id),
        outcome: MutationEventOutcome::Applied(result),
        discarded: vec![],
    });
    summary
}

fn stage(
    name: &str,
    edits: &str,
    seed: Option<u64>,
    before: &CreatureGenome,
    genome: CreatureGenome,
) -> ConstructionStage {
    stage_for(Task::A, name, edits, seed, before, genome)
}

/// One recorded edit; `useful` reads the scaffold's bypass on the active task.
pub(super) fn stage_for(
    active: Task,
    name: &str,
    edits: &str,
    seed: Option<u64>,
    before: &CreatureGenome,
    genome: CreatureGenome,
) -> ConstructionStage {
    let config = task_config();
    let battery = super::super::Battery::generate(1);
    let previous = battery.signature(before, &config.runtime, config.shared_memory.decay_rate);
    let signature = battery.signature(&genome, &config.runtime, config.shared_memory.decay_rate);
    let task = evaluate(&genome);
    let bypass = evaluate(&super::super::mesh_execution::static_successor_bypass(
        &genome,
        NodeId::new(2),
    ));
    ConstructionStage {
        name: name.into(),
        edits: edits.into(),
        seed,
        delta: GenomeDelta::between(before, &genome),
        useful: task.live() && task.correct(active) > bypass.correct(active),
        battery_class: format!("{:?}", super::super::classify(&previous, &signature).class),
        incumbent_actions_unchanged: previous == signature,
        genome,
        task,
        battery: signature,
    }
}

fn activate(before: &CreatureGenome) -> ConstructionStage {
    let mut genome = before.clone();
    topology(&mut genome, TopologyOperator::SwapRouteTargets, 3);
    stage(
        "activated",
        "constructed Topology.SwapRouteTargets",
        Some(3),
        before,
        genome,
    )
}

/// Explicit, fixed authored fixtures; seeds select no discovery outcomes.
#[must_use]
pub fn constructed_paths() -> Vec<ConstructedPath> {
    [ModuleBackend::Graph, ModuleBackend::Vm].into_iter().map(|backend| {
        let base = base(backend, false);
        let mut copied = base.clone();
        topology(&mut copied, TopologyOperator::CopyNode, 7);
        let copy_stage = stage("dormant_copy", "constructed Topology.CopyNode; exact backend/input/slot copy", Some(7), &base, copied.clone());
        let mut prepared = copied.clone();
        prepared.nodes[2].backend_def = reactive(backend, true).backend_def;
        let prepare_copy = stage("prepared_copy", "authored activation of dormant internal material: Graph.AddGraphEdge to action gate, or VM leading Halt removal (VmDeleteInstruction)", None, &copied, prepared);
        let copy_activation = activate(&prepare_copy.genome);
        let split_stage = if backend == ModuleBackend::Graph {
            let mut split = copied.clone();
            let node = &mut split.nodes[2];
            let BackendDef::Graph(graph) = &mut node.backend_def else { unreachable!() };
            split_existing_edge(graph, &node.input_refs, &mut SmallRng::seed_from_u64(7)).expect("constructed graph has splittable edges");
            Some(stage("neutral_split", "constructed mutation::graph::operators::split_existing_edge", Some(7), &copied, split))
        } else { None };
        let mut blank_genome = base.clone();
        blank_genome.nodes[0].targets.push(RouteTarget { target_id: NodeId::new(2), slot: 1, gate_bias: 0.0 });
        blank_genome.nodes.push(NodeGenome { node_id: NodeId::new(2), input_refs: vec![], backend_def: blank(backend), targets: vec![] });
        let blank_stage = stage("blank", "authored dormant attachment; current Graph.new_with_fixed_outputs or topology birth minimal VM [Halt] constructor", None, &base, blank_genome.clone());
        let mut scaffold = blank_genome.clone();
        scaffold.nodes[2].input_refs = vec![cue(Task::A)];
        let sensor = stage("sensor_preparation", "authored InputRef.AddInputRef: FoodHere(type 0)", None, &blank_genome, scaffold.clone());
        scaffold.nodes[2].backend_def = reactive(backend, true).backend_def;
        let prepared_stage = stage("dormant_preparation", "authored VM instruction/constants insertion (A1 fixture) or Graph compute/edge/action-field preparation; complete before/after fields recorded", None, &sensor.genome, scaffold);
        let activated = activate(&prepared_stage.genome);
        ConstructedPath { backend, base, stages: vec![blank_stage, sensor, prepared_stage, activated],
            copy_stages: vec![copy_stage, prepare_copy, copy_activation], split_stage }
    }).collect()
}

fn start(
    name: String,
    task: Task,
    backend: ModuleBackend,
    creation_base: CreatureGenome,
    history: Vec<ConstructionStage>,
    creation_operator: Option<MutationOperator>,
) -> Start {
    let genome = history
        .last()
        .expect("constructed start has a recorded history")
        .genome
        .clone();
    Start {
        name,
        task,
        backend,
        scaffold: NodeId::new(2),
        genome_size: genome.genome_size(),
        mutable_sites: sites(&genome),
        task_reading: evaluate(&genome),
        genome,
        creation_base,
        history,
        creation_operator,
        preparation_difference: None,
    }
}

#[must_use]
pub fn starting_forms() -> Vec<Start> {
    starts_from_paths(&constructed_paths())
}

pub(super) fn starts_from_paths(paths: &[ConstructedPath]) -> Vec<Start> {
    let mut starts = Vec::new();
    for path in paths {
        let prefix = path.backend.as_key();
        starts.push(start(
            format!("{prefix}_blank"),
            Task::A,
            path.backend,
            path.base.clone(),
            vec![path.stages[0].clone()],
            None,
        ));
        starts.push(start(
            format!("{prefix}_copy"),
            Task::A,
            path.backend,
            path.base.clone(),
            vec![path.copy_stages[0].clone()],
            Some(MutationOperator::TopologyCopyNode),
        ));
        if let Some(split) = &path.split_stage {
            starts.push(start(
                "graph_split".into(),
                Task::A,
                path.backend,
                path.base.clone(),
                vec![path.copy_stages[0].clone(), split.clone()],
                Some(MutationOperator::TopologyCopyNode),
            ));
        }
    }
    for backend in [ModuleBackend::Graph, ModuleBackend::Vm] {
        let base = base(backend, true);
        let mut copied = base.clone();
        topology(&mut copied, TopologyOperator::CopyNode, 7);
        let copy = stage(
            "common_copy",
            "constructed Topology.CopyNode of Task A-correct incumbent",
            Some(7),
            &base,
            copied.clone(),
        );
        // The common history makes the copied action point north, still dormant.
        let mut common = copied.clone();
        direction(&mut common.nodes[2], 0.0);
        let common_stage = stage("common_history_before_fork", "authored dormant direction constant E=2 to N=0; VM constant or Graph.MutateGraphOperatorParam", None, &copied, common.clone());
        let mut prepared = common.clone();
        prepared.nodes[2].input_refs[0] = cue(Task::B);
        match &mut prepared.nodes[2].backend_def {
            BackendDef::Vm(vm) => {
                vm.program[0] = VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 2,
                };
            }
            BackendDef::Graph(graph) => {
                graph.compute_nodes[0].inputs[0].source = GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 2,
                };
            }
        }
        direction(&mut prepared.nodes[2], 2.0);
        let preparation = stage("silent_preparation", "authored InputRef field Here to NeighborFoodRing; VM ReadInput/Graph input sub-index 0 to E=2; action direction N=0 to E=2", None, &common, prepared.clone());
        for (label, before, mut history) in [
            (
                "unprepared",
                common.clone(),
                vec![copy.clone(), common_stage.clone()],
            ),
            (
                "prepared",
                prepared,
                vec![copy.clone(), common_stage.clone(), preparation.clone()],
            ),
        ] {
            let mut descendant = before.clone();
            let BackendDef::Vm(entry) = &mut descendant.nodes[0].backend_def else {
                unreachable!()
            };
            entry.constants.push(0.25);
            history.push(stage(
                "common_history_after_fork",
                "replayed identical non-preparation edit: unused entry VM constant append 0.25",
                None,
                &before,
                descendant,
            ));
            starts.push(start(
                format!("{}_{label}", backend.as_key()),
                Task::B,
                backend,
                base.clone(),
                history,
                Some(MutationOperator::TopologyCopyNode),
            ));
        }
        let length = starts.len();
        let difference =
            GenomeDelta::between(&starts[length - 2].genome, &starts[length - 1].genome);
        starts[length - 2].preparation_difference = Some(difference.clone());
        starts[length - 1].preparation_difference = Some(difference);
    }
    starts
}

fn direction(node: &mut NodeGenome, value: f32) {
    match &mut node.backend_def {
        BackendDef::Vm(vm) => vm.constants[1] = value,
        BackendDef::Graph(graph) => graph.compute_nodes[1].kind = ComputeNodeKind::Constant(value),
    }
}

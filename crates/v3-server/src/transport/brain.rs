//! The inspector's brain view and learning payload (T30.F01 invariants 2–5):
//! the genome as the simulation applies it, resolved by the genome's and the
//! runtime's read APIs. `brain` carries the structure and every resolved
//! edge, member, learner and group fact; `learning`, refreshed on every
//! poll, carries the weights in force of the dynamic edges and the learned
//! scales. Neither composes a weight, a pattern or a coordinate: each
//! number is read, never recomputed.

use serde::Serialize;
use v3_core::contracts::{InputReference, NodeId};
use v3_core::creature::genome::cgp::{
    self as core_cgp, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface,
};
use v3_core::creature::genome::mesh_annotations::MeshNodeAnnotation;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmBackendDef};
use v3_core::runtime::plasticity::{LearnedEdge, LearnedState};

use crate::transport::genome_wire::{
    ComputeNodeKind, DirectedVoteKind, GraphSource, HebbianRule, InputRef, MeshReadClass,
    MeshWriteClass, OutcomeChannel, OutputSinkKind, RouteTarget, VmInstruction,
};

// ── Brain ───────────────────────────────────────────────────────────────────

/// A creature's brain: its entry node and every node of its genome.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainView {
    pub entry_node_id: NodeId,
    pub nodes: Vec<BrainNode>,
}

/// One mesh node: its annotation facts, input references, backend and route
/// targets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainNode {
    pub node_id: NodeId,
    pub annotation: NodeAnnotation,
    pub input_refs: Vec<InputRef>,
    pub backend: BrainBackend,
    pub targets: Vec<RouteTarget>,
}

/// What the mesh analysis reads of a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct NodeAnnotation {
    pub reachable: bool,
    pub read_classes: Vec<MeshReadClass>,
    pub write_classes: Vec<MeshWriteClass>,
    pub has_stateful_behavior: bool,
    /// The live VM instruction indices.
    pub live_instruction_indices: Vec<usize>,
    /// The live compute-node indices.
    pub live_internal_node_indices: Vec<usize>,
}

impl From<&MeshNodeAnnotation> for NodeAnnotation {
    fn from(annotation: &MeshNodeAnnotation) -> Self {
        Self {
            reachable: annotation.reachable,
            read_classes: annotation.read_classes.iter().map(|&c| c.into()).collect(),
            write_classes: annotation.write_classes.iter().map(|&c| c.into()).collect(),
            has_stateful_behavior: annotation.has_stateful_behavior,
            live_instruction_indices: annotation.live_instruction_indices.clone(),
            live_internal_node_indices: annotation.live_internal_node_indices.clone(),
        }
    }
}

/// A node's backend.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BrainBackend {
    Vm(VmBrain),
    Graph(GraphBrain),
}

/// A VM backend: its registers, constants and program.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VmBrain {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub program: Vec<VmInstruction>,
}

impl From<&VmBackendDef> for VmBrain {
    fn from(vm: &VmBackendDef) -> Self {
        Self {
            register_count: vm.register_count,
            constants: vm.constants.clone(),
            program: vm.program.iter().map(VmInstruction::from).collect(),
        }
    }
}

/// A Graph backend: its compute nodes and output sinks with their edges and
/// its edge groups in `GroupId` order.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphBrain {
    pub compute_nodes: Vec<BrainComputeNode>,
    pub output_sinks: Vec<BrainSink>,
    pub groups: Vec<BrainGroup>,
}

/// A compute node and its input edges.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainComputeNode {
    pub kind: ComputeNodeKind,
    pub edges: Vec<BrainEdge>,
}

/// An output sink and its input edges.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainSink {
    pub kind: OutputSinkKind,
    pub edges: Vec<BrainEdge>,
}

/// One edge: its source, its own weight, its genomic (composed) weight, the
/// weight in force and, when a member, its group facts and, when plastic,
/// its learner.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainEdge {
    pub source: GraphSource,
    /// The edge's own weight: its genomic weight when ungrouped, its
    /// correction on top of the group term when a member.
    pub own: f32,
    /// The genomic weight (`genomic_weight`): `own` when ungrouped, the
    /// composed weight of a member.
    pub genomic: f32,
    /// The weight in force (`applied_weights`), else `genomic`.
    pub applied: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub member: Option<EdgeMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learner: Option<EdgeLearner>,
}

/// A member's group facts: its group, scope, site, field, pattern and its
/// group's genomic scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct EdgeMember {
    /// The group's index in `groups`.
    pub group: usize,
    pub scope: Scope,
    /// The member's site relative to its group's kind; omitted for a member
    /// off its kind's sinks.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub site: Option<MemberSite>,
    /// The input field the member reads; omitted when it reads no input leaf.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub field: Option<u16>,
    pub pattern: f32,
    pub scale: f32,
}

/// A member's learning scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum Scope {
    Edge,
    Group,
}

impl From<core_cgp::Scope> for Scope {
    fn from(scope: core_cgp::Scope) -> Self {
        match scope {
            core_cgp::Scope::Edge => Self::Edge,
            core_cgp::Scope::Group => Self::Group,
        }
    }
}

/// Where a member sits relative to its group's kind: a direction sink or
/// the kind's drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum MemberSite {
    Direction(u8),
    Drive,
}

impl From<core_cgp::MemberSite> for MemberSite {
    fn from(site: core_cgp::MemberSite) -> Self {
        match site {
            core_cgp::MemberSite::Direction(d) => Self::Direction(d),
            core_cgp::MemberSite::Drive => Self::Drive,
        }
    }
}

/// A learner's scope: an ungrouped edge, an edge-scope or a group-scope
/// member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum LearnerScope {
    Ungrouped,
    Edge,
    Group,
}

impl From<Option<core_cgp::Scope>> for LearnerScope {
    fn from(scope: Option<core_cgp::Scope>) -> Self {
        match scope {
            None => Self::Ungrouped,
            Some(core_cgp::Scope::Edge) => Self::Edge,
            Some(core_cgp::Scope::Group) => Self::Group,
        }
    }
}

/// A learner's reward modulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LearnerModulation {
    pub reward_source: OutcomeChannel,
    pub trace_decay: f32,
}

/// A plastic edge's learner, from its learning attributes and the
/// runtime's view of it (`LearnedEdge`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct EdgeLearner {
    pub rule: HebbianRule,
    pub learning_rate: f32,
    pub weight_clamp: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub modulation: Option<LearnerModulation>,
    pub retention: f32,
    pub scope: LearnerScope,
    pub initialized: bool,
    /// Whether a child inherits what this learner learns: its `lamarckian`
    /// flag, or its group's `inherit_scale` at group scope.
    pub inherits: bool,
    /// The learned own quantity, when initialized; never at group scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned: Option<f32>,
    /// The eligibility trace, when initialized and modulated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub trace: Option<f32>,
}

/// One edge group: its record, its learned scale and its birth coordinates
/// with presence.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainGroup {
    pub shape: GroupShape,
    pub kind: DirectedVoteKind,
    pub offset: u8,
    pub scale: f32,
    pub inherit_scale: bool,
    pub retention: f32,
    /// An initialized scale slot's value; omitted otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned_scale: Option<f32>,
    /// The direction coordinates a member is born on.
    pub coordinates: Vec<GroupCoordinate>,
    /// A pair gene's declared facts on its kind's drive; empty for a ring.
    pub facts: Vec<GroupFact>,
}

/// The geometry a group's members fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum GroupShape {
    Ring { ring_ref: u16 },
    DirectionPair { ref_idx: u16, pair: u8 },
}

impl From<core_cgp::Shape> for GroupShape {
    fn from(shape: core_cgp::Shape) -> Self {
        match shape {
            core_cgp::Shape::Ring { ring_ref } => Self::Ring { ring_ref },
            core_cgp::Shape::DirectionPair { ref_idx, pair } => {
                Self::DirectionPair { ref_idx, pair }
            }
        }
    }
}

/// A birth coordinate on a direction sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GroupCoordinate {
    pub field: u16,
    pub direction: u8,
    pub present: bool,
}

/// A declared fact of a pair gene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GroupFact {
    pub field: u16,
    pub present: bool,
}

// ── Learning ────────────────────────────────────────────────────────────────

/// One Graph node's dynamic edges and learned scales.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct NodeLearning {
    pub node_id: NodeId,
    pub edges: Vec<LearningEdge>,
    pub scales: Vec<LearningScale>,
}

/// Which edge-bearing surface an edge sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum WeightSurface {
    Compute,
    Sink,
}

/// One dynamic edge (a learner or a passive member), keyed by surface,
/// container and index: its weight in force and, for a learner, its
/// learned own quantity, trace and initialization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LearningEdge {
    pub surface: WeightSurface,
    pub container: usize,
    pub edge: usize,
    pub applied: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub trace: Option<f32>,
    /// A learner's initialization; omitted for a passive member.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub initialized: Option<bool>,
}

/// One scale slot: its group, learned value when initialized, and
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LearningScale {
    pub group: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned: Option<f32>,
    pub initialized: bool,
}

// ── Assembly ────────────────────────────────────────────────────────────────

/// One edge of a Graph node in canonical order: its occurrence, its
/// weight in force when dynamic (`applied_weights`; a static edge's weight
/// in force is its genomic weight) and its learner view.
struct WalkedEdge {
    occurrence: EdgeOccurrence,
    dynamic: Option<f32>,
    view: Option<LearnedEdge>,
}

/// Every edge of Graph node `node_idx` in canonical order, by one merge-walk
/// over `edge_sites()` and the runtime's `applied_weights` and `edges`
/// views, all in canonical order.
fn walk_edges<'a>(
    def: &'a CgpGraphBackendDef,
    learned: &'a LearnedState,
    node_idx: usize,
) -> impl Iterator<Item = WalkedEdge> + 'a {
    let mut applied = learned.applied_weights(node_idx).peekable();
    let mut views = learned.edges(node_idx).peekable();
    def.edge_sites().map(move |occurrence| WalkedEdge {
        occurrence,
        dynamic: applied
            .next_if(|&(at, _)| at == occurrence)
            .map(|(_, weight)| weight),
        view: views.next_if(|view| view.occurrence == occurrence),
    })
}

fn surface_of(occurrence: EdgeOccurrence) -> (WeightSurface, usize) {
    match occurrence.surface {
        EdgeSurface::ComputeInput(index) => (WeightSurface::Compute, index),
        EdgeSurface::SinkInput(index) => (WeightSurface::Sink, index),
    }
}

/// The brain view of `genome`, its learned state `learned` and its mesh
/// annotations (`derive_mesh_annotations_with_reachable_indices`, one per
/// node in genome order).
#[must_use]
pub fn brain_view(
    genome: &CreatureGenome,
    learned: &LearnedState,
    annotations: &[MeshNodeAnnotation],
) -> BrainView {
    debug_assert_eq!(annotations.len(), genome.nodes.len());
    BrainView {
        entry_node_id: genome.entry_node_id,
        nodes: genome
            .nodes
            .iter()
            .zip(annotations)
            .enumerate()
            .map(|(node_idx, (node, annotation))| BrainNode {
                node_id: node.node_id,
                annotation: annotation.into(),
                input_refs: node.input_refs.iter().map(InputRef::from).collect(),
                backend: match &node.backend_def {
                    BackendDef::Vm(vm) => BrainBackend::Vm(vm.into()),
                    BackendDef::Graph(def) => {
                        BrainBackend::Graph(graph_brain(def, &node.input_refs, learned, node_idx))
                    }
                },
                targets: node.targets.iter().map(RouteTarget::from).collect(),
            })
            .collect(),
    }
}

fn graph_brain(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    learned: &LearnedState,
    node_idx: usize,
) -> GraphBrain {
    let mut compute_nodes: Vec<BrainComputeNode> = def
        .compute_nodes()
        .iter()
        .map(|node| BrainComputeNode {
            kind: node.kind.into(),
            edges: Vec::with_capacity(node.inputs.len()),
        })
        .collect();
    let mut output_sinks: Vec<BrainSink> = def
        .output_sinks()
        .iter()
        .map(|sink| BrainSink {
            kind: sink.kind.into(),
            edges: Vec::with_capacity(sink.inputs.len()),
        })
        .collect();
    for walked in walk_edges(def, learned, node_idx) {
        let edge = brain_edge(def, input_refs, &walked);
        match walked.occurrence.surface {
            EdgeSurface::ComputeInput(index) => compute_nodes[index].edges.push(edge),
            EdgeSurface::SinkInput(index) => output_sinks[index].edges.push(edge),
        }
    }
    let mut scales = learned.scales(node_idx).peekable();
    let groups = def
        .groups()
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let id = core_cgp::GroupId::new(u16::try_from(index).expect("groups below u16::MAX"));
            let learned_scale = scales
                .next_if(|scale| scale.group == id)
                .and_then(|scale| scale.learned);
            let mut coordinates = Vec::new();
            let mut facts = Vec::new();
            for coordinate in def.birth_coordinates(id, input_refs) {
                match coordinate.site {
                    core_cgp::MemberSite::Direction(direction) => {
                        coordinates.push(GroupCoordinate {
                            field: coordinate.field,
                            direction,
                            present: coordinate.present,
                        });
                    }
                    core_cgp::MemberSite::Drive => facts.push(GroupFact {
                        field: coordinate.field,
                        present: coordinate.present,
                    }),
                }
            }
            BrainGroup {
                shape: record.shape().into(),
                kind: record.kind().into(),
                offset: record.offset(),
                scale: record.scale(),
                inherit_scale: record.inherit_scale(),
                retention: record.retention(),
                learned_scale,
                coordinates,
                facts,
            }
        })
        .collect();
    GraphBrain {
        compute_nodes,
        output_sinks,
        groups,
    }
}

fn brain_edge(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    walked: &WalkedEdge,
) -> BrainEdge {
    let edge = def.edge_at(walked.occurrence);
    let genomic = def.genomic_weight(walked.occurrence, input_refs);
    let member = edge.membership().map(|membership| EdgeMember {
        group: membership.group.index(),
        scope: membership.scope.into(),
        site: def.member_site(walked.occurrence).map(MemberSite::from),
        field: match edge.source() {
            core_cgp::GraphSource::InputLeaf { sub_idx, .. } => Some(sub_idx),
            _ => None,
        },
        pattern: def.pattern(walked.occurrence, input_refs),
        scale: def.groups()[membership.group.index()].scale(),
    });
    let learner = edge
        .plasticity()
        .zip(walked.view)
        .map(|(config, view)| EdgeLearner {
            rule: config.rule.into(),
            learning_rate: config.learning_rate,
            weight_clamp: config.weight_clamp,
            modulation: config.modulation.map(|modulation| LearnerModulation {
                reward_source: modulation.reward_source.into(),
                trace_decay: modulation.trace_decay,
            }),
            retention: view.retention,
            scope: view.scope.into(),
            initialized: view.initialized,
            inherits: view.inherits,
            learned: view.learned,
            trace: view.trace,
        });
    BrainEdge {
        source: edge.source().into(),
        own: edge.own_weight(),
        genomic,
        applied: walked.dynamic.unwrap_or(genomic),
        member,
        learner,
    }
}

/// The `learning` payload: per Graph node with a dynamic edge, every dynamic
/// edge of `applied_weights` (learners adding their facts from `edges`) and
/// every scale slot; `None` when no node carries a dynamic edge.
#[must_use]
pub fn learning_view(genome: &CreatureGenome, learned: &LearnedState) -> Option<Vec<NodeLearning>> {
    let nodes: Vec<NodeLearning> = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(node_idx, node)| {
            let BackendDef::Graph(def) = &node.backend_def else {
                return None;
            };
            let edges: Vec<LearningEdge> = walk_edges(def, learned, node_idx)
                .filter_map(|walked| {
                    let applied = walked.dynamic?;
                    let (surface, container) = surface_of(walked.occurrence);
                    Some(LearningEdge {
                        surface,
                        container,
                        edge: walked.occurrence.index,
                        applied,
                        learned: walked.view.and_then(|view| view.learned),
                        trace: walked.view.and_then(|view| view.trace),
                        initialized: walked.view.map(|view| view.initialized),
                    })
                })
                .collect();
            (!edges.is_empty()).then(|| NodeLearning {
                node_id: node.node_id,
                edges,
                scales: learned
                    .scales(node_idx)
                    .map(|scale| LearningScale {
                        group: scale.group.index(),
                        learned: scale.learned,
                        initialized: scale.learned.is_some(),
                    })
                    .collect(),
            })
        })
        .collect();
    (!nodes.is_empty()).then_some(nodes)
}

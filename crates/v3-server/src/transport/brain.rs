//! The inspector's brain view and learning payload (T30.F01 invariants 2–5):
//! the genome as the simulation applies it, resolved by the genome's and the
//! runtime's read APIs. `brain` carries the structure and every resolved
//! edge, member, learner and bundle fact; `learning`, refreshed on every
//! poll, carries the weights in force of the dynamic edges and the learned
//! scales. Neither composes a weight, a pattern or a coordinate: each
//! number is read, never recomputed.

use serde::Serialize;
use v3_core::contracts::{InputReference, NodeId, SensorContext};
use v3_core::genome::cgp::{self as core_cgp, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface};
use v3_core::genome::learning::LearnerSource as CoreLearnerSource;
use v3_core::genome::mesh_annotations::MeshNodeAnnotation;
use v3_core::genome::{
    BackendDef, CreatureGenome, Learning as CoreLearning, MemberPolicy as CoreMemberPolicy,
    VmBackendDef,
};
use v3_core::runtime::plasticity::{LearnedEdge, LearnedState};

use crate::transport::genome_wire::{
    ComputeNodeKind, DirectedVoteKind, GraphSource, HebbianRule, InputRef, MeshReadClass,
    MeshWriteClass, OutcomeChannel, OutputSinkKind, RouteTarget, VmInstruction, VoteSink,
};

// ── Brain ───────────────────────────────────────────────────────────────────

/// A creature's brain: its entry node and every node of its genome.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainView {
    pub entry_node_id: NodeId,
    pub nodes: Vec<BrainNode>,
}

/// The identity of the sensor context a brain view's bundle coordinates
/// resolve under (T25.F11 invariant 11, T30.F14 invariant 5). Every creature
/// fetch carries it, a poll that excludes the brain included; equal contexts
/// give equal identities and different contexts different ones, so a client
/// holding a brain compares it whole and refetches the brain when it moves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainContext(String);

impl From<&SensorContext> for BrainContext {
    fn from(context: &SensorContext) -> Self {
        // Exhaustive: a field added to `SensorContext` fails to compile here
        // until the identity carries it.
        let SensorContext {
            food_type_count,
            action_queue_slots,
        } = *context;
        Self(format!(
            "food_type_count={food_type_count};action_queue_slots={action_queue_slots}"
        ))
    }
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
/// its edge bundles in `BundleId` order.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphBrain {
    pub compute_nodes: Vec<BrainComputeNode>,
    pub output_sinks: Vec<BrainSink>,
    pub bundles: Vec<BrainBundle>,
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
/// weight in force and, when a member, its bundle facts and, when an edge
/// learner, its learner.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainEdge {
    pub source: GraphSource,
    /// The edge's own weight: its genomic weight when lone, its modifier on
    /// top of the bundle term when a member.
    pub own: f32,
    /// The genomic weight (`genomic_weight`): `own` when lone, the composed
    /// weight of a member.
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

/// A member's bundle facts: its bundle, policy, `credits`, site, field,
/// pattern and its bundle's genomic scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct EdgeMember {
    /// The bundle's index in `bundles`.
    pub bundle: usize,
    pub policy: MemberPolicy,
    /// Whether the member's activity feeds its bundle's scale.
    pub credits: bool,
    /// The member's site relative to its bundle's kind; omitted for a member
    /// off its kind's sinks.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub site: Option<Site>,
    /// The input field the member reads; omitted when it reads no input leaf.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub field: Option<u16>,
    pub pattern: f32,
    pub scale: f32,
}

/// How a member's modifier learns (T25.F12): through its bundle's
/// `member_default`, not at all, or by its own override (the learner's
/// learning, source `own`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum MemberPolicy {
    Inherit,
    Static,
    Own,
}

impl From<CoreMemberPolicy> for MemberPolicy {
    fn from(policy: CoreMemberPolicy) -> Self {
        match policy {
            CoreMemberPolicy::Inherit => Self::Inherit,
            CoreMemberPolicy::Static => Self::Static,
            CoreMemberPolicy::Own(_) => Self::Own,
        }
    }
}

/// Where a member sits under its bundle's wiring: a `Plain` wiring's vote
/// sink, a direction sink of a geometric wiring's kind, or that kind's drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum Site {
    Sink,
    Direction(u8),
    Drive,
}

impl From<core_cgp::Site> for Site {
    fn from(site: core_cgp::Site) -> Self {
        match site {
            core_cgp::Site::Sink => Self::Sink,
            core_cgp::Site::Direction(d) => Self::Direction(d),
            core_cgp::Site::Drive => Self::Drive,
        }
    }
}

/// Where an edge learner's learning is stored (T25.F12): a lone edge's, a
/// member's override, or its bundle's `member_default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum LearnerSource {
    Lone,
    Own,
    MemberDefault,
}

impl From<CoreLearnerSource> for LearnerSource {
    fn from(source: CoreLearnerSource) -> Self {
        match source {
            CoreLearnerSource::Lone => Self::Lone,
            CoreLearnerSource::Own => Self::Own,
            CoreLearnerSource::MemberDefault => Self::MemberDefault,
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

/// One learning configuration (T25.F12): what a lone edge, a member's
/// override and a bundle's two slots store, the one wire `Learning`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Learning {
    pub rule: HebbianRule,
    pub learning_rate: f32,
    pub weight_clamp: f32,
    /// Whether a child inherits what this learning learns.
    pub lamarckian: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub modulation: Option<LearnerModulation>,
    pub retention: f32,
}

impl From<CoreLearning> for Learning {
    fn from(learning: CoreLearning) -> Self {
        Self {
            rule: learning.rule.into(),
            learning_rate: learning.learning_rate,
            weight_clamp: learning.weight_clamp,
            lamarckian: learning.lamarckian,
            modulation: learning.modulation.map(|modulation| LearnerModulation {
                reward_source: modulation.reward_source.into(),
                trace_decay: modulation.trace_decay,
            }),
            retention: learning.retention,
        }
    }
}

/// An edge learner, from the runtime's view of it (`LearnedEdge`): its
/// effective learning and where it is stored.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct EdgeLearner {
    #[serde(flatten)]
    #[cfg_attr(test, ts(flatten))]
    pub learning: Learning,
    pub source: LearnerSource,
    pub initialized: bool,
    /// The learned own quantity, when initialized.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned: Option<f32>,
    /// The eligibility trace, when initialized and modulated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub trace: Option<f32>,
}

/// One bundle (T25.F11): its record, its learned scale and its birth
/// coordinates with presence.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrainBundle {
    pub source: BundleSource,
    pub wiring: Wiring,
    pub scale: f32,
    /// How the scale learns ("learns as a group"); omitted when it does not.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub scale_learning: Option<Learning>,
    /// The learning `inherit` members run ("learns per edge"); omitted when
    /// absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub member_default: Option<Learning>,
    /// An initialized learned scale's value; omitted otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub learned_scale: Option<f32>,
    /// Every coordinate a member is born on, with presence.
    pub coordinates: Vec<BundleCoordinate>,
}

/// What a bundle reads: a declared sensor's field set or a memory bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum BundleSource {
    Sensor { ref_idx: u16, field_set: u8 },
    Memory { previous: bool },
}

impl From<core_cgp::BundleSource> for BundleSource {
    fn from(source: core_cgp::BundleSource) -> Self {
        match source {
            core_cgp::BundleSource::Sensor { ref_idx, field_set } => {
                Self::Sensor { ref_idx, field_set }
            }
            core_cgp::BundleSource::Memory { previous } => Self::Memory { previous },
        }
    }
}

/// How a bundle's members sit on the sinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum Wiring {
    Plain { sink: VoteSink },
    Ring { kind: DirectedVoteKind, offset: u8 },
    DirectionPair { kind: DirectedVoteKind, offset: u8 },
}

impl From<core_cgp::Wiring> for Wiring {
    fn from(wiring: core_cgp::Wiring) -> Self {
        match wiring {
            core_cgp::Wiring::Plain { sink } => Self::Plain { sink: sink.into() },
            core_cgp::Wiring::Ring { kind, offset } => Self::Ring {
                kind: kind.into(),
                offset,
            },
            core_cgp::Wiring::DirectionPair { kind, offset } => Self::DirectionPair {
                kind: kind.into(),
                offset,
            },
        }
    }
}

/// A birth coordinate: the field a member born with the bundle reads, its
/// site, and the location of the member that sits there now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BundleCoordinate {
    pub field: u16,
    pub site: Site,
    /// The member's location, the key `learning` uses; omitted for a hole.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub member: Option<EdgeLocation>,
}

/// An edge's location in a Graph node: its surface, the compute node or
/// sink it feeds, and its index there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct EdgeLocation {
    pub surface: WeightSurface,
    pub container: usize,
    pub edge: usize,
}

impl From<EdgeOccurrence> for EdgeLocation {
    fn from(occurrence: EdgeOccurrence) -> Self {
        let (surface, container) = match occurrence.surface {
            EdgeSurface::ComputeInput(index) => (WeightSurface::Compute, index),
            EdgeSurface::SinkInput(index) => (WeightSurface::Sink, index),
        };
        Self {
            surface,
            container,
            edge: occurrence.index,
        }
    }
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

/// One scale slot: its bundle, learned value when initialized, and
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LearningScale {
    pub bundle: usize,
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

/// The brain view of `genome`, its learned state `learned` and its mesh
/// annotations (`derive_mesh_annotations_with_reachable_indices`, one per
/// node in genome order), its bundle coordinates resolved under the run's
/// `context` from the session config (T25.F11 invariant 11).
#[must_use]
pub fn brain_view(
    genome: &CreatureGenome,
    learned: &LearnedState,
    annotations: &[MeshNodeAnnotation],
    context: &SensorContext,
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
                input_refs: node.input_refs().iter().map(InputRef::from).collect(),
                backend: match node.backend_def() {
                    BackendDef::Vm(vm) => BrainBackend::Vm(vm.into()),
                    BackendDef::Graph(def) => BrainBackend::Graph(graph_brain(
                        def,
                        node.input_refs(),
                        learned,
                        node_idx,
                        context,
                    )),
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
    context: &SensorContext,
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
    let bundles = def
        .indexed_bundles()
        .map(|(id, record)| {
            let learned_scale = scales
                .next_if(|scale| scale.bundle == id)
                .and_then(|scale| scale.learned);
            let coordinates = def
                .birth_coordinates(id, input_refs, context)
                .into_iter()
                .map(|coordinate| BundleCoordinate {
                    field: coordinate.field,
                    site: coordinate.site.into(),
                    member: coordinate.member.map(EdgeLocation::from),
                })
                .collect();
            BrainBundle {
                source: record.source().into(),
                wiring: record.wiring().into(),
                scale: record.scale(),
                scale_learning: record.scale_learning().map(Learning::from),
                member_default: record.member_default().map(Learning::from),
                learned_scale,
                coordinates,
            }
        })
        .collect();
    GraphBrain {
        compute_nodes,
        output_sinks,
        bundles,
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
        bundle: membership.bundle.index(),
        policy: membership.policy.into(),
        credits: membership.credits,
        site: def.site_of(walked.occurrence).map(Site::from),
        field: match edge.source() {
            core_cgp::GraphSource::InputLeaf { sub_idx, .. } => Some(sub_idx),
            core_cgp::GraphSource::SharedMemory { slot, .. } => Some(u16::from(slot)),
            core_cgp::GraphSource::ComputeNode(_) => None,
        },
        pattern: def.pattern(walked.occurrence, input_refs),
        scale: def.bundles()[membership.bundle.index()].scale(),
    });
    let learner = walked.view.map(|view| EdgeLearner {
        learning: view.learning.into(),
        source: view.source.into(),
        initialized: view.initialized,
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
            let BackendDef::Graph(def) = node.backend_def() else {
                return None;
            };
            let edges: Vec<LearningEdge> = walk_edges(def, learned, node_idx)
                .filter_map(|walked| {
                    let applied = walked.dynamic?;
                    let EdgeLocation {
                        surface,
                        container,
                        edge,
                    } = walked.occurrence.into();
                    Some(LearningEdge {
                        surface,
                        container,
                        edge,
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
                        bundle: scale.bundle.index(),
                        learned: scale.learned,
                        initialized: scale.learned.is_some(),
                    })
                    .collect(),
            })
        })
        .collect();
    (!nodes.is_empty()).then_some(nodes)
}

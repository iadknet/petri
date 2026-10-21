//! Group-scope learning (T25.F07): scale slots, pattern-weighted credit,
//! passive members, seeds by scope and the single applied-weight
//! composition.

use super::learned::LearningCounts;
use super::{LearnedState, OutcomeSignalBank};
use crate::config::RuntimeConfig;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::genome::cgp::{
    BirthSeeds, CgpGraphBackendDef, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
    GraphSource, GroupId, OutputSinkKind, Scope, Shape,
};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute::execute_graph_impl;
use crate::runtime::cgp::tracer::GraphTracer;
use crate::runtime::types::{MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

const PAIR_REF: u16 = 0;
const RING_REF: u16 = 1;

/// A pair-declaring reference and a ring.
fn refs() -> Vec<InputReference> {
    vec![
        InputReference::World(WorldInputKey::AreaBarrierSummary),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
    ]
}

fn hebbian(rule: HebbianRule, eta: f32, clamp: f32) -> PlasticityConfig {
    PlasticityConfig {
        rule,
        learning_rate: eta,
        weight_clamp: clamp,
        lamarckian: false,
        modulation: None,
        retention: 1.0,
    }
}

fn modulated(eta: f32, clamp: f32) -> PlasticityConfig {
    PlasticityConfig {
        modulation: Some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.9,
        }),
        ..hebbian(HebbianRule::Classic, eta, clamp)
    }
}

/// A full catalog with one `DirectionPair` group (twelve members, patterns
/// ±1 and ±1/√2) at genomic scale `scale`.
fn pair_def(scale: f32) -> (CgpGraphBackendDef, GroupId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let layout = refs()[usize::from(PAIR_REF)].direction_pairs()[0];
    let id = def.add_pair_group(PAIR_REF, 0, layout, DirectedVoteKind::Move, 0, &[]);
    def.group_mut(id).set_scale(scale);
    (def, id)
}

/// A full catalog with one `Ring` group (eight members on `Move`, pattern
/// 1) at genomic scale `scale`.
fn ring_def(scale: f32) -> (CgpGraphBackendDef, GroupId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_group(
        Shape::Ring { ring_ref: RING_REF },
        DirectedVoteKind::Move,
        0,
    );
    def.group_mut(id).set_scale(scale);
    (def, id)
}

fn members(def: &CgpGraphBackendDef, id: GroupId) -> Vec<EdgeOccurrence> {
    def.members(id).collect()
}

/// The first member whose pattern has the sign of `sign`.
fn member_with_sign(def: &CgpGraphBackendDef, id: GroupId, sign: f32) -> EdgeOccurrence {
    members(def, id)
        .into_iter()
        .find(|&member| def.pattern(member, &refs()) * sign > 0.0)
        .expect("a member of that sign")
}

fn learn(def: &mut CgpGraphBackendDef, at: EdgeOccurrence, config: PlasticityConfig, scope: Scope) {
    let mut edge = def.edge_mut(at);
    edge.set_plasticity(Some(config));
    edge.set_scope(scope);
}

fn genome(def: &CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: refs(),
            backend_def: BackendDef::Graph(def.clone()),
            targets: vec![],
        }],
    }
}

fn state(def: &CgpGraphBackendDef, seeds: Option<BirthSeeds>) -> LearnedState {
    LearnedState::for_genome(&genome(def), &[seeds])
}

/// Seeds for `def`: every row empty, `edits` applied.
fn seeds(def: &CgpGraphBackendDef, edit: impl FnOnce(&mut BirthSeeds)) -> BirthSeeds {
    let mut seeds = BirthSeeds::Rows {
        compute: def
            .compute_nodes()
            .iter()
            .map(|node| vec![None; node.inputs.len()])
            .collect(),
        sinks: def
            .output_sinks()
            .iter()
            .map(|sink| vec![None; sink.inputs.len()])
            .collect(),
        groups: vec![None; def.groups().len()],
    };
    edit(&mut seeds);
    seeds
}

fn seed_edge(seeds: &mut BirthSeeds, at: EdgeOccurrence, value: f32) {
    let BirthSeeds::Rows { compute, sinks, .. } = seeds;
    let row = match at.surface {
        EdgeSurface::ComputeInput(i) => &mut compute[i],
        EdgeSurface::SinkInput(i) => &mut sinks[i],
    };
    row[at.index] = Some(value);
}

fn seed_group(seeds: &mut BirthSeeds, group: GroupId, value: f32) {
    let BirthSeeds::Rows { groups, .. } = seeds;
    groups[group.index()] = Some(value);
}

/// One committed visit driven through the learner in canonical order: each
/// edge reads `pre(occurrence)`, each container's post sample is `post`.
fn drive(
    state: &mut LearnedState,
    def: &CgpGraphBackendDef,
    pre: impl Fn(EdgeOccurrence) -> f32,
    post: f32,
) -> LearningCounts {
    let refs = refs();
    let mut learner = state.learner(0).expect("a learning backend");
    learner.enter();
    let surfaces = (0..def.compute_nodes().len())
        .map(EdgeSurface::ComputeInput)
        .chain((0..def.output_sinks().len()).map(EdgeSurface::SinkInput));
    for surface in surfaces {
        let count = def.surface_edges(surface).len();
        if count == 0 {
            continue;
        }
        learner.begin_row(surface);
        for index in 0..count {
            let at = EdgeOccurrence::new(surface, index);
            learner.weigh(pre(at), def.genomic_weight(at, &refs));
        }
        learner.end_row(post);
    }
    learner.commit()
}

fn view_edge(state: &LearnedState, at: EdgeOccurrence) -> super::LearnedEdge {
    state
        .edges(0)
        .find(|edge| edge.occurrence == at)
        .expect("a learner")
}

fn scale(state: &LearnedState) -> super::LearnedScale {
    let scales: Vec<_> = state.scales(0).collect();
    assert_eq!(scales.len(), 1, "one scale slot");
    scales[0]
}

fn applied(state: &LearnedState, at: EdgeOccurrence) -> f32 {
    state
        .applied_weights(0)
        .find(|&(occurrence, _)| occurrence == at)
        .map(|(_, weight)| weight)
        .expect("a dynamic edge")
}

fn signals(energy: f32) -> OutcomeSignalBank {
    let mut bank = OutcomeSignalBank::default();
    bank.signals[OutcomeChannel::EnergyDelta as usize] = energy;
    bank
}

/// Oja's `w` is the visit-start applied weight `own + scale × pattern`, for
/// an edge learner (its correction moves) and a group learner (the scale
/// moves by `pattern × Δ`).
#[test]
fn oja_group_members_use_visit_start_applied_weight() {
    let (mut def, id) = pair_def(0.5);
    let edge = member_with_sign(&def, id, 1.0);
    let group = member_with_sign(&def, id, -1.0);
    let oja = hebbian(HebbianRule::Oja, 0.5, 4.0);
    def.edge_mut(edge).set_weight(0.25);
    learn(&mut def, edge, oja, Scope::Edge);
    learn(&mut def, group, oja, Scope::Group);
    let refs = refs();
    let (p_edge, p_group) = (def.pattern(edge, &refs), def.pattern(group, &refs));
    let mut learned = state(&def, None);
    let (pre, post) = (0.8f32, 1.5f32);
    drive(&mut learned, &def, |_| pre, post);

    let w_edge = 0.5 * p_edge + 0.25;
    let delta_edge = 0.5 * post * (pre - w_edge * post);
    assert_eq!(view_edge(&learned, edge).learned, Some(0.25 + delta_edge));
    let w_group = 0.5 * p_group;
    let delta_group = 0.5 * post * (pre - w_group * post);
    assert_eq!(scale(&learned).learned, Some(0.5 + p_group * delta_group));
    // Δ would differ under the genomic own weight alone.
    assert_ne!(delta_group, 0.5 * post * pre);
}

/// The scale's change is `clamp(scale + Σ pattern × Δ, ±bound)`, summed
/// over the group learners in canonical order and bounded once; group
/// learners store no own quantity, and a negative-pattern member's
/// contribution has the opposite sign of its `Δ`.
#[test]
fn group_scale_credit_is_pattern_weighted_summed_in_order_and_bounded_once() {
    let (mut def, id) = pair_def(0.0);
    let positive = member_with_sign(&def, id, 1.0);
    let negative = member_with_sign(&def, id, -1.0);
    let classic = hebbian(HebbianRule::Classic, 1.0, 1.0);
    learn(&mut def, positive, classic, Scope::Group);
    learn(&mut def, negative, classic, Scope::Group);
    let refs = refs();
    let (p_pos, p_neg) = (def.pattern(positive, &refs), def.pattern(negative, &refs));

    // A lone negative-pattern learner with Δ > 0 lowers the scale.
    {
        let mut lone = def.clone();
        lone.edge_mut(positive).set_plasticity(None);
        let mut learned = state(&lone, None);
        drive(&mut learned, &lone, |_| 0.5, 0.5);
        assert_eq!(scale(&learned).learned, Some(p_neg * (1.0 * 0.5 * 0.5)));
        assert!(scale(&learned).learned.unwrap() < 0.0);
    }

    // Contributions +3 then −2.5 against bound 1: one bounded write gives
    // 0.5; bounding each would give −1.
    let pre = |at: EdgeOccurrence| {
        if at == positive {
            3.0 / p_pos
        } else if at == negative {
            -2.5 / p_neg
        } else {
            0.0
        }
    };
    let mut learned = state(&def, None);
    let counts = drive(&mut learned, &def, pre, 1.0);
    let contributions = [
        (positive, p_pos * (1.0 * pre(positive) * 1.0)),
        (negative, p_neg * (1.0 * pre(negative) * 1.0)),
    ];
    let mut ordered = contributions;
    let position = |at| def.edge_sites().position(|site| site == at);
    ordered.sort_by_key(|&(at, _)| position(at));
    let sum = ordered.iter().fold(0.0f32, |sum, &(_, c)| sum + c);
    let written = scale(&learned).learned.unwrap();
    assert_eq!(written, sum.clamp(-1.0, 1.0));
    assert!((written - 0.5).abs() < 1e-5, "{written}");
    assert_eq!(counts.updates, 2);
    for at in [positive, negative] {
        let edge = view_edge(&learned, at);
        assert_eq!(edge.learned, None, "a group learner stores no own quantity");
        assert_eq!(edge.genomic, 0.0);
        assert_eq!(edge.applied, written * def.pattern(at, &refs));
    }
}

/// Under a zero reward, committed visits move traces but neither own
/// quantities nor scales, at either scope.
#[test]
fn a_modulated_learner_changes_no_weight_at_a_visit() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    def.edge_mut(all[0]).set_weight(0.25);
    learn(&mut def, all[0], modulated(0.5, 4.0), Scope::Edge);
    learn(&mut def, all[1], modulated(0.5, 4.0), Scope::Group);
    let mut learned = state(&def, None);
    for _ in 0..3 {
        drive(&mut learned, &def, |_| 1.0, 2.0);
    }
    let edge = view_edge(&learned, all[0]);
    assert_eq!(edge.learned, Some(0.25));
    assert!(edge.trace.is_some_and(|trace| trace != 0.0));
    assert!(view_edge(&learned, all[1])
        .trace
        .is_some_and(|trace| trace != 0.0));
    assert_eq!(scale(&learned).learned, Some(0.5));
    let counts = learned.apply_reward(&signals(0.0));
    assert_eq!((counts.updates, counts.changes), (2, 0));
    assert_eq!(view_edge(&learned, all[0]).learned, Some(0.25));
    assert_eq!(scale(&learned).learned, Some(0.5));
}

/// A static member of a learning group samples nothing and moves nothing,
/// and is weighed through the learned scale.
#[test]
fn a_static_member_contributes_nothing() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    learn(
        &mut def,
        all[0],
        hebbian(HebbianRule::Classic, 0.5, 4.0),
        Scope::Group,
    );
    def.edge_mut(all[1]).set_weight(0.125);
    let mut learned = state(&def, None);
    // The static member reads a large value; only the learner's Δ moves the
    // scale.
    drive(
        &mut learned,
        &def,
        |at| if at == all[1] { 100.0 } else { 1.0 },
        2.0,
    );
    let moved = scale(&learned).learned.unwrap();
    assert_eq!(moved, 0.5 * 1.0 * 2.0);
    assert_eq!(applied(&learned, all[1]), moved + 0.125);
    assert_eq!(
        learned.edges(0).count(),
        1,
        "the static member is no learner"
    );
    assert_eq!(
        learned.applied_weights(0).count(),
        8,
        "every member is dynamic"
    );
}

/// In one visit, edge learners move their own corrections and group
/// learners move the scale.
#[test]
fn a_mixed_group_updates_edge_learners_and_the_scale_separately() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    let classic = hebbian(HebbianRule::Classic, 0.5, 4.0);
    learn(&mut def, all[0], classic, Scope::Edge);
    learn(&mut def, all[1], classic, Scope::Group);
    learn(&mut def, all[2], classic, Scope::Group);
    let mut learned = state(&def, None);
    let counts = drive(&mut learned, &def, |_| 1.0, 2.0);
    let delta = 0.5 * 1.0 * 2.0;
    assert_eq!(view_edge(&learned, all[0]).learned, Some(delta));
    assert_eq!(scale(&learned).learned, Some(0.5 + (delta + delta)));
    assert_eq!(view_edge(&learned, all[1]).learned, None);
    assert_eq!(
        applied(&learned, all[0]),
        0.5 + 2.0 * delta + delta,
        "the edge learner composes its correction with the learned scale"
    );
    assert_eq!((counts.updates, counts.changes), (3, 3));
}

/// Phase 2.5 sums the modulated group learners' contributions and writes
/// the scale once; a scale slot that is not initialized takes none.
#[test]
fn reward_credit_to_a_scale_is_one_bounded_write_per_tick() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    learn(&mut def, all[0], modulated(0.5, 1.0), Scope::Group);
    learn(&mut def, all[1], modulated(0.25, 3.0), Scope::Group);

    let mut fresh = state(&def, None);
    let counts = fresh.apply_reward(&signals(1.0));
    assert_eq!(counts, LearningCounts::default(), "an uninitialized scale");
    assert_eq!(scale(&fresh).learned, None);

    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 1.0, 2.0);
    let traces: Vec<f32> = learned.edges(0).map(|edge| edge.trace.unwrap()).collect();
    assert_eq!(traces, vec![2.0, 2.0]);
    let counts = learned.apply_reward(&signals(1.0));
    // 0.5 × 1 × 2 + 0.25 × 1 × 2 = 1.5, bounded once by the smaller clamp.
    assert_eq!(scale(&learned).bound, 1.0);
    assert_eq!(scale(&learned).learned, Some(1.0));
    assert_eq!((counts.updates, counts.changes), (2, 2));
}

/// Inside the bound, a modulated group learner's reward credit to its
/// scale is exactly `pattern × η × signal × trace`.
#[test]
fn reward_credit_to_a_scale_is_pattern_times_rate_times_signal_times_trace() {
    // Arrange
    let (mut def, id) = pair_def(0.5);
    let negative = member_with_sign(&def, id, -1.0);
    learn(&mut def, negative, modulated(0.25, 10.0), Scope::Group);
    let pattern = def.pattern(negative, &refs());
    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 1.0, 2.0);
    let trace = view_edge(&learned, negative).trace.unwrap();
    assert_eq!(scale(&learned).learned, Some(0.5), "no visit write");

    // Act
    let counts = learned.apply_reward(&signals(3.0));

    // Assert
    let expected = 0.5 + pattern * 0.25 * 3.0 * trace;
    assert!(expected.abs() < 10.0, "the credit stays inside the bound");
    assert_eq!(scale(&learned).learned, Some(expected));
    assert_eq!((counts.updates, counts.changes), (1, 1));
}

/// An unseeded genomic scale outside its bound starts at the bound, its
/// birth value (T25.F09 invariant 2), so a zero-`Σ` write leaves it
/// unchanged and counts no change.
#[test]
fn a_genomic_scale_outside_the_bound_starts_at_the_bound() {
    let (mut def, id) = ring_def(7.0);
    let all = members(&def, id);
    learn(
        &mut def,
        all[0],
        hebbian(HebbianRule::Classic, 0.0, 2.0),
        Scope::Group,
    );
    let mut learned = state(&def, None);
    assert_eq!((scale(&learned).genomic, scale(&learned).birth), (7.0, 2.0));
    let first = drive(&mut learned, &def, |_| 1.0, 1.0);
    assert_eq!(scale(&learned).learned, Some(2.0));
    assert_eq!((first.updates, first.changes), (1, 0));
}

/// `inherit_scale` without a group learner seeds nothing (no scale slot);
/// a group learner without `inherit_scale` starts at the genomic scale.
#[test]
fn a_scale_seeds_only_with_a_group_scope_learner() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    def.group_mut(id).set_inherit_scale(true);
    learn(
        &mut def,
        all[0],
        hebbian(HebbianRule::Classic, 0.5, 4.0),
        Scope::Edge,
    );
    let parent_seeds = seeds(&def, |seeds| seed_group(seeds, id, 1.5));
    let edge_only = state(&def, Some(parent_seeds.clone()));
    assert_eq!(edge_only.scales(0).count(), 0);
    assert_eq!(applied(&edge_only, all[0]), 0.5);

    learn(
        &mut def,
        all[0],
        hebbian(HebbianRule::Classic, 0.5, 4.0),
        Scope::Group,
    );
    def.group_mut(id).set_inherit_scale(false);
    let darwinian = state(&def, Some(parent_seeds.clone()));
    assert_eq!(scale(&darwinian).learned, None);
    assert_eq!(applied(&darwinian, all[0]), 0.5);

    def.group_mut(id).set_inherit_scale(true);
    let lamarckian = state(&def, Some(parent_seeds));
    assert_eq!(scale(&lamarckian).learned, Some(1.5));
    assert_eq!(applied(&lamarckian, all[0]), 1.5);
}

/// A Lamarckian group learner ignores a seed on its row, starts at its
/// genomic own weight and is captured as `None`; after `SetScope` to
/// `Edge` its child starts at its genomic correction.
#[test]
fn a_group_learner_stores_no_own_quantity() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    def.edge_mut(all[0]).set_weight(0.25);
    let lamarckian = PlasticityConfig {
        lamarckian: true,
        ..hebbian(HebbianRule::Classic, 0.5, 4.0)
    };
    learn(&mut def, all[0], lamarckian, Scope::Group);
    let parent = state(
        &def,
        Some(seeds(&def, |seeds| seed_edge(seeds, all[0], 3.0))),
    );
    let edge = view_edge(&parent, all[0]);
    assert_eq!((edge.learned, edge.genomic), (None, 0.25));
    assert_eq!(edge.applied, 0.75);
    let mut visited = parent.clone();
    drive(&mut visited, &def, |_| 1.0, 1.0);
    let captured = visited.capture(0, &def).expect("an initialized backend");
    assert_eq!(captured.value(all[0]), None);

    let mut child = def.clone();
    child.edge_mut(all[0]).set_scope(Scope::Edge);
    let child_state = state(&child, Some(captured));
    let edge = view_edge(&child_state, all[0]);
    assert_eq!(edge.learned, None, "unseeded and unvisited");
    assert_eq!(edge.applied, 0.25 + 0.5);
}

/// Initialization is per slot: a seeded scale with no initialized learner,
/// a seeded edge learner with an uninitialized scale, and a failed first
/// visit that initializes both.
#[test]
fn initialization_is_per_slot_across_edges_and_scales() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    def.group_mut(id).set_inherit_scale(true);
    learn(
        &mut def,
        all[0],
        hebbian(HebbianRule::Classic, 0.5, 4.0),
        Scope::Group,
    );
    let lamarckian = PlasticityConfig {
        lamarckian: true,
        ..modulated(0.5, 4.0)
    };
    learn(&mut def, all[1], lamarckian, Scope::Edge);

    let scale_seeded = state(&def, Some(seeds(&def, |seeds| seed_group(seeds, id, 1.25))));
    assert_eq!(scale(&scale_seeded).learned, Some(1.25));
    assert!(scale_seeded
        .edges(0)
        .all(|edge| edge.learned.is_none() && edge.trace.is_none()));
    let captured = scale_seeded.capture(0, &def).expect("a seeded scale");
    assert_eq!(captured.group_value(id), Some(1.25));
    assert_eq!(captured.value(all[1]), None);

    let edge_seeded = state(
        &def,
        Some(seeds(&def, |seeds| seed_edge(seeds, all[1], 0.75))),
    );
    assert_eq!(scale(&edge_seeded).learned, None);
    let edge = view_edge(&edge_seeded, all[1]);
    assert_eq!((edge.learned, edge.trace), (Some(0.75), Some(0.0)));
    let captured = edge_seeded.capture(0, &def).expect("a seeded learner");
    assert_eq!(captured.value(all[1]), Some(0.75));
    assert_eq!(captured.group_value(id), None);
    // The unseeded, uninitialized scale takes no reward.
    let mut rewarded = edge_seeded.clone();
    rewarded.apply_reward(&signals(1.0));
    assert_eq!(scale(&rewarded).learned, None);

    let mut failed = state(&def, None);
    assert!(failed.capture(0, &def).is_none());
    failed.learner(0).expect("a learning backend").enter();
    assert_eq!(scale(&failed).learned, Some(0.5));
    assert_eq!(view_edge(&failed, all[1]).learned, Some(0.0));
    assert!(failed.capture(0, &def).is_some());
}

/// Counters (invariant 9): a group of two contributors whose scale moves
/// counts two updates and two changes; a scale held at its bound counts
/// two updates and no change.
#[test]
fn counters_count_every_contributor_of_a_moved_scale() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    let classic = hebbian(HebbianRule::Classic, 0.5, 1.0);
    learn(&mut def, all[0], classic, Scope::Group);
    learn(&mut def, all[1], classic, Scope::Group);
    let mut learned = state(&def, None);
    let first = drive(&mut learned, &def, |_| 1.0, 0.5);
    assert_eq!((first.updates, first.changes), (2, 2));
    let mut saturated = state(&def, None);
    drive(&mut saturated, &def, |_| 10.0, 10.0);
    assert_eq!(scale(&saturated).learned, Some(1.0));
    let held = drive(&mut saturated, &def, |_| 10.0, 10.0);
    assert_eq!((held.updates, held.changes), (2, 0));
}

/// Records each sink's weighted inputs.
#[derive(Default)]
struct SinkRecorder {
    sinks: Vec<Vec<f32>>,
}

impl GraphTracer for SinkRecorder {
    fn on_pass_start(&mut self, _: u32, _: f32, _: f32) {}
    fn on_node_eval(
        &mut self,
        _: usize,
        _: &ComputeNodeKind,
        _: &[f32],
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) {
    }
    fn on_pass_end(&mut self, _: f32) {}
    fn on_finish(&mut self, _: &[f32], _: bool) {}
    fn on_sink(&mut self, _: bool, weighted_inputs: &[f32], _: f32, _: bool, _: f32) {
        self.sinks.push(weighted_inputs.to_vec());
    }
    fn on_effects(&mut self) {}
}

fn ring_sensors(values: [f32; 8]) -> SensorSnapshot {
    SensorSnapshot {
        world_reads: Default::default(),
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: values,
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: 0.0,
            own_colour: [0.0; 3],
            previous_outcome: [0.0; 4],
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

/// Invariant 3: across an ungrouped learner, an edge learner, a group
/// learner and a passive member, the executor's weighing, `applied_weight`,
/// `applied_weights` and the view's `applied` agree.
#[test]
fn applied_weight_agrees_with_the_executor_and_the_view() {
    let (mut def, id) = ring_def(0.5);
    let silent = def.add_group(
        Shape::Ring { ring_ref: RING_REF },
        DirectedVoteKind::Move,
        1,
    );
    def.group_mut(silent).set_scale(-0.25);
    let all = members(&def, id);
    def.group_mut(id).set_inherit_scale(true);
    let lamarckian = PlasticityConfig {
        lamarckian: true,
        ..hebbian(HebbianRule::Classic, 0.1, 4.0)
    };
    def.edge_mut(all[0]).set_weight(0.25);
    learn(&mut def, all[0], lamarckian, Scope::Edge);
    learn(&mut def, all[1], lamarckian, Scope::Group);
    def.edge_mut(all[2]).set_weight(-0.5);
    let ungrouped_surface = def
        .sink_surface(OutputSinkKind::CustomOutput(0))
        .expect("a custom output sink");
    def.push_edge(
        ungrouped_surface,
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: RING_REF,
                sub_idx: 3,
            },
            0.75,
        ),
    );
    let ungrouped = EdgeOccurrence::new(ungrouped_surface, 0);
    def.edge_mut(ungrouped).set_plasticity(Some(lamarckian));
    let birth = seeds(&def, |seeds| {
        seed_group(seeds, id, 1.25);
        seed_edge(seeds, all[0], 0.5);
        seed_edge(seeds, ungrouped, -1.5);
    });
    let mut runtime = GraphRuntimeState::for_genome(&genome(&def), vec![Some(birth)]);
    let refs = refs();
    let expected = [
        (all[0], 0.5 + 1.25),
        (all[1], 1.25),
        (all[2], -0.5 + 1.25),
        (ungrouped, -1.5),
    ];
    let learned = runtime.learned().clone();
    for (at, weight) in expected {
        assert_eq!(applied(&learned, at), weight);
        assert_eq!(learned.applied_weight(0, &def, &refs, at), weight);
    }
    for edge in learned.edges(0) {
        assert_eq!(edge.applied, applied(&learned, edge.occurrence));
    }
    // A static member of a group without a scale slot is no dynamic edge
    // and resolves to its genomic weight.
    let silent_member = members(&def, silent)[0];
    assert!(learned
        .applied_weights(0)
        .all(|(at, _)| at != silent_member));
    assert_eq!(
        learned.applied_weight(0, &def, &refs, silent_member),
        def.genomic_weight(silent_member, &refs)
    );

    let values = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let mut tracer = SinkRecorder::default();
    let mut energy = 100.0;
    let mut side = MeshSideOutputs::new(10);
    let _ = execute_graph_impl(
        &mut tracer,
        &def,
        &refs,
        &[0.0; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut runtime,
        &ring_sensors(values),
        &RuntimeConfig::default(),
        &mut side,
        &mut [0.0; 16],
        &[0.0; 16],
    );
    for (at, weight) in expected {
        let EdgeSurface::SinkInput(sink) = at.surface else {
            unreachable!("sink edges")
        };
        let GraphSource::InputLeaf { sub_idx, .. } = def.edge_at(at).source() else {
            unreachable!("ring reads")
        };
        let value = values[usize::from(sub_idx)];
        assert_eq!(tracer.sinks[sink][at.index], value * weight, "{at:?}");
    }
}

/// T25.F10 invariant 2: a fact member's pattern is 0 by role, so a
/// `Group`-scope fact learner credits `0 × Δ` and leaves the scale as it
/// starts, an `Edge`-scope one learns its own correction, and a static fact
/// member of a group with a scale slot is a passive dynamic edge weighing
/// its own weight alone.
#[test]
fn a_fact_member_weighs_and_learns_without_the_scale() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let layout = refs()[usize::from(PAIR_REF)].direction_pairs()[0];
    let id = def.add_pair_group(PAIR_REF, 0, layout, DirectedVoteKind::Move, 0, layout.facts);
    def.group_mut(id).set_scale(0.5);
    let facts: Vec<EdgeOccurrence> = members(&def, id)
        .into_iter()
        .filter(|&member| def.is_fact_member(member))
        .collect();
    assert_eq!(facts.len(), 3);
    let classic = hebbian(HebbianRule::Classic, 1.0, 4.0);
    learn(&mut def, facts[0], classic, Scope::Group);
    def.edge_mut(facts[1]).set_weight(0.25);
    learn(&mut def, facts[1], classic, Scope::Edge);
    def.edge_mut(facts[2]).set_weight(-0.125);
    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 0.5, 0.5);
    assert_eq!(scale(&learned).learned, Some(0.5), "0 × Δ moves no scale");
    assert_eq!(view_edge(&learned, facts[0]).applied, 0.0);
    assert_eq!(
        view_edge(&learned, facts[1]).learned,
        Some(0.25 + 1.0 * 0.5 * 0.5)
    );
    assert_eq!(applied(&learned, facts[1]), 0.25 + 1.0 * 0.5 * 0.5);
    assert_eq!(applied(&learned, facts[2]), -0.125, "a passive fact member");
}

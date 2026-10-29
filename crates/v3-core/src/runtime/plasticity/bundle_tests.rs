//! Bundle learning (T25.F07, T25.F12): scale learners fed by crediting
//! members, member modifiers learning by their effective learning, seeds,
//! and the single applied-weight composition, all through one update law.

use super::learned::LearningCounts;
use super::{LearnedState, OutcomeSignalBank};
use crate::config::RuntimeConfig;
use crate::contracts::votes::DirectedVoteKind;
use crate::contracts::OutcomeChannel;
use crate::contracts::OUTPUT_SLOT_COUNT;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::genome::cgp::{
    BirthSeeds, BundleId, CgpGraphBackendDef, ComputeNodeKind, EdgeOccurrence, EdgeSurface,
    GraphEdge, GraphSource, LearningSlot, OutputSinkKind,
};
use crate::genome::learning::LearnerSource;
use crate::genome::{
    BackendDef, CreatureGenome, HebbianRule, Learning, MemberPolicy, NodeGenome,
    RewardModulationConfig,
};
use crate::runtime::cgp::execute::execute_graph_impl;
use crate::runtime::cgp::tracer::GraphTracer;
use crate::runtime::state::GraphRuntimeState;
use crate::runtime::types::MeshSideOutputs;
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

fn hebbian(rule: HebbianRule, eta: f32, clamp: f32) -> Learning {
    Learning {
        rule,
        learning_rate: eta,
        weight_clamp: clamp,
        lamarckian: false,
        modulation: None,
        retention: 1.0,
    }
}

fn modulated(eta: f32, clamp: f32) -> Learning {
    Learning {
        modulation: Some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.9,
        }),
        ..hebbian(HebbianRule::Classic, eta, clamp)
    }
}

fn lamarckian(learning: Learning) -> Learning {
    Learning {
        lamarckian: true,
        ..learning
    }
}

/// A full catalog with one `DirectionPair` bundle (twelve members, patterns
/// ±1 and ±1/√2) at genomic scale `scale`.
fn pair_def(scale: f32) -> (CgpGraphBackendDef, BundleId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let layout = crate::contracts::pair_layout(&refs()[usize::from(PAIR_REF)], 0);
    let id = def.add_pair_bundle(PAIR_REF, 0, layout, DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(scale);
    (def, id)
}

/// A full catalog with one `Ring` bundle (eight members on `Move`, pattern
/// 1) at genomic scale `scale`.
fn ring_def(scale: f32) -> (CgpGraphBackendDef, BundleId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_ring_bundle(RING_REF, DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(scale);
    (def, id)
}

fn members(def: &CgpGraphBackendDef, id: BundleId) -> Vec<EdgeOccurrence> {
    def.members(id).collect()
}

/// The first member whose pattern has the sign of `sign`.
fn member_with_sign(def: &CgpGraphBackendDef, id: BundleId, sign: f32) -> EdgeOccurrence {
    members(def, id)
        .into_iter()
        .find(|&member| def.pattern(member, &refs()) * sign > 0.0)
        .expect("a member of that sign")
}

/// Set the member at `at`'s policy and `credits`.
fn member(def: &mut CgpGraphBackendDef, at: EdgeOccurrence, policy: MemberPolicy, credits: bool) {
    let mut edge = def.edge_mut(at);
    edge.set_policy(policy);
    edge.set_credits(credits);
}

/// Set bundle `id`'s two slots.
fn slots(
    def: &mut CgpGraphBackendDef,
    id: BundleId,
    scale: Option<Learning>,
    default: Option<Learning>,
) {
    let mut record = def.bundle_mut(id);
    record.set_slot(LearningSlot::Scale, scale);
    record.set_slot(LearningSlot::MemberDefault, default);
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
        bundles: vec![None; def.bundles().len()],
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

fn seed_bundle(seeds: &mut BirthSeeds, bundle: BundleId, value: f32) {
    let BirthSeeds::Rows { bundles, .. } = seeds;
    bundles[bundle.index()] = Some(value);
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
        .expect("an edge learner")
}

fn scale(state: &LearnedState) -> super::LearnedScale {
    let scales: Vec<_> = state.scales(0).collect();
    assert_eq!(scales.len(), 1, "one scale learner");
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

/// Oja's `w` is the visit-start applied weight `modifier + scale ×
/// pattern`, for a member learning its modifier and for a contributor of a
/// learning scale (the scale moves by `pattern × Δ` at `scale_learning`'s
/// rate).
#[test]
fn oja_members_use_the_visit_start_applied_weight() {
    let (mut def, id) = pair_def(0.5);
    let own = member_with_sign(&def, id, 1.0);
    let credits = member_with_sign(&def, id, -1.0);
    let oja = hebbian(HebbianRule::Oja, 0.5, 4.0);
    slots(&mut def, id, Some(oja), None);
    def.edge_mut(own).set_weight(0.25);
    member(&mut def, own, MemberPolicy::Own(oja), false);
    member(&mut def, credits, MemberPolicy::Static, true);
    let refs = refs();
    let (p_own, p_credit) = (def.pattern(own, &refs), def.pattern(credits, &refs));
    let mut learned = state(&def, None);
    let (pre, post) = (0.8f32, 1.5f32);
    drive(&mut learned, &def, |_| pre, post);

    let w_own = 0.5 * p_own + 0.25;
    let delta_own = 0.5 * post * (pre - w_own * post);
    assert_eq!(view_edge(&learned, own).learned, Some(0.25 + delta_own));
    let w_credit = 0.5 * p_credit;
    let delta_credit = 0.5 * post * (pre - w_credit * post);
    assert_eq!(scale(&learned).learned, Some(0.5 + p_credit * delta_credit));
}

/// A scale's change is `clamp(scale + Σ pattern × Δ, ±clamp)`, summed over
/// its contributors in canonical order with `scale_learning`'s rule and
/// rate and bounded once; a negative-pattern contributor's contribution has
/// the opposite sign of its `Δ`; the scale counts one update.
#[test]
fn scale_credit_is_pattern_weighted_summed_in_order_and_bounded_once() {
    let (mut def, id) = pair_def(0.0);
    let positive = member_with_sign(&def, id, 1.0);
    let negative = member_with_sign(&def, id, -1.0);
    slots(
        &mut def,
        id,
        Some(hebbian(HebbianRule::Classic, 1.0, 1.0)),
        None,
    );
    member(&mut def, positive, MemberPolicy::Static, true);
    member(&mut def, negative, MemberPolicy::Static, true);
    let refs = refs();
    let (p_pos, p_neg) = (def.pattern(positive, &refs), def.pattern(negative, &refs));

    // A lone negative-pattern contributor with Δ > 0 lowers the scale.
    {
        let mut lone = def.clone();
        lone.edge_mut(positive).set_credits(false);
        let mut learned = state(&lone, None);
        drive(&mut learned, &lone, |_| 0.5, 0.5);
        assert_eq!(scale(&learned).learned, Some(p_neg * (1.0 * 0.5 * 0.5)));
        assert!(scale(&learned).learned.unwrap() < 0.0);
    }

    // Contributions +3 then −2.5 against clamp 1: one bounded write gives
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
    let mut ordered = [
        (positive, p_pos * (1.0 * pre(positive) * 1.0)),
        (negative, p_neg * (1.0 * pre(negative) * 1.0)),
    ];
    ordered.sort_by_key(|&(at, _)| at);
    let sum = ordered.iter().fold(0.0f32, |sum, &(_, c)| sum + c);
    let written = scale(&learned).learned.unwrap();
    assert_eq!(written, sum.clamp(-1.0, 1.0));
    assert!((written - 0.5).abs() < 1e-5, "{written}");
    assert_eq!(
        (counts.updates, counts.changes),
        (1, 1),
        "a scale counts once"
    );
    assert_eq!(
        learned.edges(0).count(),
        0,
        "a Static member is no edge learner"
    );
    for at in [positive, negative] {
        assert_eq!(applied(&learned, at), written * def.pattern(at, &refs));
    }
}

/// A modulated scale and a modulated member change no value at a visit;
/// the scale's one trace takes `Σ pattern × activity`; a zero reward moves
/// nothing and counts one reward step per learner.
#[test]
fn a_modulated_learner_changes_no_value_at_a_visit() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    slots(&mut def, id, Some(modulated(0.5, 4.0)), None);
    def.edge_mut(all[0]).set_weight(0.25);
    member(
        &mut def,
        all[0],
        MemberPolicy::Own(modulated(0.5, 4.0)),
        false,
    );
    member(&mut def, all[1], MemberPolicy::Static, true);
    member(&mut def, all[2], MemberPolicy::Static, true);
    let mut learned = state(&def, None);
    for _ in 0..3 {
        drive(&mut learned, &def, |_| 1.0, 2.0);
    }
    let edge = view_edge(&learned, all[0]);
    assert_eq!(edge.learned, Some(0.25));
    assert_eq!(edge.trace, Some(6.0));
    assert_eq!(learned.scale_trace(0, 0), 3.0 * (1.0 * 2.0 + 1.0 * 2.0));
    assert_eq!(scale(&learned).learned, Some(0.5));
    let counts = learned.apply_reward(&signals(0.0));
    assert_eq!((counts.updates, counts.changes), (2, 0));
    assert_eq!(view_edge(&learned, all[0]).learned, Some(0.25));
    assert_eq!(scale(&learned).learned, Some(0.5));
}

/// A member without `credits` feeds nothing to the scale and is weighed
/// through the learned scale.
#[test]
fn a_member_without_credits_contributes_nothing() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    slots(
        &mut def,
        id,
        Some(hebbian(HebbianRule::Classic, 0.5, 4.0)),
        None,
    );
    member(&mut def, all[0], MemberPolicy::Static, true);
    def.edge_mut(all[1]).set_weight(0.125);
    let mut learned = state(&def, None);
    // The non-crediting member reads a large value; only the contributor's
    // Δ moves the scale.
    drive(
        &mut learned,
        &def,
        |at| if at == all[1] { 100.0 } else { 1.0 },
        2.0,
    );
    let moved = scale(&learned).learned.unwrap();
    assert_eq!(moved, 0.5 * 1.0 * 2.0);
    assert_eq!(applied(&learned, all[1]), moved + 0.125);
    assert_eq!(learned.edges(0).count(), 0);
    assert_eq!(
        learned.applied_weights(0).count(),
        8,
        "every member is dynamic"
    );
}

/// The note's case: a bundle-only learner learns. A bundle with
/// `scale_learning`, no `member_default` and every member `Static` and
/// crediting has one learner, its scale, and it moves.
#[test]
fn a_bundle_only_learner_learns() {
    let (mut def, id) = ring_def(0.25);
    slots(
        &mut def,
        id,
        Some(hebbian(HebbianRule::Classic, 0.25, 4.0)),
        None,
    );
    for at in members(&def, id) {
        member(&mut def, at, MemberPolicy::Static, true);
    }
    let mut learned = state(&def, None);
    assert!(learned.has_learning());
    let counts = drive(&mut learned, &def, |_| 1.0, 1.0);
    assert_eq!(scale(&learned).learned, Some(0.25 + 8.0 * 0.25));
    assert_eq!((counts.updates, counts.changes), (1, 1));
}

/// The note's cases: an `Inherit` member under a learning scale without
/// `member_default` moves only through the scale; `member_default` serves
/// `Inherit` members and leaves `Own` and `Static` ones; a member that
/// learns and credits updates both from one visit.
#[test]
fn member_default_serves_inherit_members_alone() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    let classic = hebbian(HebbianRule::Classic, 0.5, 4.0);
    slots(&mut def, id, Some(classic), None);
    member(&mut def, all[0], MemberPolicy::Inherit, true);
    member(
        &mut def,
        all[1],
        MemberPolicy::Own(hebbian(HebbianRule::AntiHebb, 0.25, 4.0)),
        false,
    );
    member(&mut def, all[2], MemberPolicy::Static, false);

    // Without `member_default`, the `Inherit` member learns no modifier.
    let mut learned = state(&def, None);
    let sources: Vec<_> = learned
        .edges(0)
        .map(|edge| (edge.occurrence, edge.source))
        .collect();
    assert_eq!(sources, vec![(all[1], LearnerSource::Own)]);
    drive(&mut learned, &def, |_| 1.0, 2.0);
    let moved = 0.5 + 1.0 * (0.5 * 1.0 * 2.0);
    assert_eq!(scale(&learned).learned, Some(moved));
    assert_eq!(applied(&learned, all[0]), moved, "the scale alone moves it");

    // With `member_default`, it learns its modifier by the default and
    // credits from the same visit; the `Own` member keeps its override and
    // the `Static` member stays static.
    let default = hebbian(HebbianRule::Classic, 0.125, 3.0);
    slots(&mut def, id, Some(classic), Some(default));
    let mut learned = state(&def, None);
    let sources: Vec<_> = learned
        .edges(0)
        .map(|edge| (edge.occurrence, edge.source))
        .collect();
    assert_eq!(
        sources,
        vec![
            (all[0], LearnerSource::MemberDefault),
            (all[1], LearnerSource::Own)
        ]
    );
    assert_eq!(
        view_edge(&learned, all[0]).learning.weight_clamp,
        default.weight_clamp
    );
    let counts = drive(&mut learned, &def, |_| 1.0, 2.0);
    assert_eq!(view_edge(&learned, all[0]).learned, Some(0.125 * 2.0));
    assert_eq!(view_edge(&learned, all[1]).learned, Some(-0.25 * 2.0));
    assert_eq!(scale(&learned).learned, Some(moved));
    assert_eq!((counts.updates, counts.changes), (3, 3));
    assert_eq!(
        applied(&learned, all[2]),
        moved,
        "a Static member reads the scale"
    );
}

/// The note's case: the slots evolve independently. Clearing
/// `member_default` leaves the scale's learner; clearing `scale_learning`
/// leaves the `Inherit` members' learners.
#[test]
fn the_slots_evolve_independently() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    let classic = hebbian(HebbianRule::Classic, 0.5, 4.0);
    for &at in &all {
        member(&mut def, at, MemberPolicy::Inherit, true);
    }
    slots(&mut def, id, Some(classic), Some(classic));
    let both = state(&def, None);
    assert_eq!((both.edges(0).count(), both.scales(0).count()), (8, 1));
    slots(&mut def, id, Some(classic), None);
    let scale_only = state(&def, None);
    assert_eq!(
        (scale_only.edges(0).count(), scale_only.scales(0).count()),
        (0, 1)
    );
    slots(&mut def, id, None, Some(classic));
    let default_only = state(&def, None);
    assert_eq!(
        (
            default_only.edges(0).count(),
            default_only.scales(0).count()
        ),
        (8, 0)
    );
    assert!(default_only
        .applied_weights(0)
        .all(|(at, weight)| weight == def.genomic_weight(at, &refs())));
}

/// The note's case: switching one member's `credits` off leaves its
/// siblings' credit to the scale.
#[test]
fn disabling_one_member_leaves_its_siblings_credit() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    slots(
        &mut def,
        id,
        Some(hebbian(HebbianRule::Classic, 1.0, 10.0)),
        None,
    );
    for &at in &all {
        member(&mut def, at, MemberPolicy::Static, true);
    }
    def.edge_mut(all[3]).set_credits(false);
    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 1.0, 1.0);
    assert_eq!(scale(&learned).learned, Some(7.0));
}

/// The note's case: a child whose last contributor was switched off keeps,
/// decays and passes on its scale. A scale learner without contributors is
/// seeded, receives no credit, relaxes toward its birth value and is
/// captured.
#[test]
fn a_scale_without_contributors_keeps_decays_and_passes_on() {
    let (mut def, id) = ring_def(0.5);
    let retaining = Learning {
        retention: 0.5,
        ..lamarckian(hebbian(HebbianRule::Classic, 1.0, 4.0))
    };
    slots(&mut def, id, Some(retaining), None);
    let parent = seeds(&def, |seeds| seed_bundle(seeds, id, 2.5));
    let mut child = state(&def, Some(parent));
    assert_eq!(scale(&child).learned, Some(2.5));
    let counts = drive(&mut child, &def, |_| 1.0, 1.0);
    assert_eq!(
        counts,
        LearningCounts::default(),
        "no contributor, no credit"
    );
    assert_eq!(scale(&child).learned, Some(2.5));
    child.begin_tick();
    assert_eq!(
        scale(&child).learned,
        Some(2.5),
        "the birth value is the seed"
    );
    child.set_scale(0, 0, 3.5);
    child.begin_tick();
    assert_eq!(scale(&child).learned, Some(2.5 + 0.5 * (3.5 - 2.5)));
    let captured = child.capture(0, &def).expect("an initialized scale");
    assert_eq!(captured.bundle_value(id), Some(3.0));
}

/// Lamarckian seeding of a scale and a modifier: each seeds by its own
/// learning's `lamarckian`; a Darwinian one starts at its genomic value.
#[test]
fn a_scale_and_a_modifier_seed_by_their_own_lamarckian() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    let classic = hebbian(HebbianRule::Classic, 0.5, 4.0);
    def.edge_mut(all[0]).set_weight(0.25);
    member(&mut def, all[0], MemberPolicy::Inherit, true);
    let parent = |def: &CgpGraphBackendDef| {
        seeds(def, |seeds| {
            seed_bundle(seeds, id, 1.5);
            seed_edge(seeds, all[0], 0.75);
        })
    };
    slots(&mut def, id, Some(lamarckian(classic)), Some(classic));
    let child = state(&def, Some(parent(&def)));
    assert_eq!(scale(&child).learned, Some(1.5));
    assert!(scale(&child).learning.lamarckian);
    let edge = view_edge(&child, all[0]);
    assert_eq!((edge.learned, edge.learning.lamarckian), (None, false));
    assert_eq!(edge.applied, 0.25 + 1.5);

    slots(&mut def, id, Some(classic), Some(lamarckian(classic)));
    let child = state(&def, Some(parent(&def)));
    assert_eq!(scale(&child).learned, None);
    let edge = view_edge(&child, all[0]);
    assert_eq!((edge.learned, edge.learning.lamarckian), (Some(0.75), true));
    assert_eq!(edge.applied, 0.75 + 0.5);
    let captured = child.capture(0, &def).expect("a seeded modifier");
    assert_eq!(captured.value(all[0]), Some(0.75));
    assert_eq!(captured.bundle_value(id), None);
}

/// Reward: one `three_factor` per initialized modulated learner at its own
/// source, rate and clamp; the scale's step reads its one trace and counts
/// once; an uninitialized scale takes none.
#[test]
fn reward_is_one_bounded_step_per_learner() {
    let (mut def, id) = ring_def(0.0);
    let all = members(&def, id);
    slots(&mut def, id, Some(modulated(0.5, 1.0)), None);
    member(&mut def, all[0], MemberPolicy::Static, true);
    member(&mut def, all[1], MemberPolicy::Static, true);

    let mut fresh = state(&def, None);
    let counts = fresh.apply_reward(&signals(1.0));
    assert_eq!(counts, LearningCounts::default(), "an uninitialized scale");
    assert_eq!(scale(&fresh).learned, None);

    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 1.0, 2.0);
    assert_eq!(learned.scale_trace(0, 0), 4.0);
    let counts = learned.apply_reward(&signals(1.0));
    // 0.5 × 1 × 4 = 2, bounded once by the clamp.
    assert_eq!(scale(&learned).learned, Some(1.0));
    assert_eq!((counts.updates, counts.changes), (1, 1));

    let (mut def, id) = pair_def(0.5);
    let negative = member_with_sign(&def, id, -1.0);
    slots(&mut def, id, Some(modulated(0.25, 10.0)), None);
    member(&mut def, negative, MemberPolicy::Static, true);
    let pattern = def.pattern(negative, &refs());
    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 1.0, 2.0);
    let trace = learned.scale_trace(0, 0);
    assert_eq!(trace, pattern * 2.0);
    learned.apply_reward(&signals(3.0));
    assert_eq!(scale(&learned).learned, Some(0.5 + 0.25 * 3.0 * trace));
}

/// An unseeded genomic scale outside its clamp starts at the clamp, its
/// birth value (T25.F09 invariant 2), so a zero-`Σ` write leaves it
/// unchanged and counts no change.
#[test]
fn a_genomic_scale_outside_the_clamp_starts_at_the_clamp() {
    let (mut def, id) = ring_def(7.0);
    let all = members(&def, id);
    slots(
        &mut def,
        id,
        Some(hebbian(HebbianRule::Classic, 0.0, 2.0)),
        None,
    );
    member(&mut def, all[0], MemberPolicy::Static, true);
    let mut learned = state(&def, None);
    assert_eq!((scale(&learned).genomic, scale(&learned).birth), (7.0, 2.0));
    let first = drive(&mut learned, &def, |_| 1.0, 1.0);
    assert_eq!(scale(&learned).learned, Some(2.0));
    assert_eq!((first.updates, first.changes), (1, 0));
}

/// Initialization is per learner: a seeded scale with an uninitialized
/// modifier, a seeded modifier with an uninitialized scale, and a failed
/// first visit that initializes both.
#[test]
fn initialization_is_per_learner_across_edges_and_scales() {
    let (mut def, id) = ring_def(0.5);
    let all = members(&def, id);
    slots(
        &mut def,
        id,
        Some(lamarckian(hebbian(HebbianRule::Classic, 0.5, 4.0))),
        None,
    );
    member(&mut def, all[0], MemberPolicy::Static, true);
    member(
        &mut def,
        all[1],
        MemberPolicy::Own(lamarckian(modulated(0.5, 4.0))),
        false,
    );

    let scale_seeded = state(
        &def,
        Some(seeds(&def, |seeds| seed_bundle(seeds, id, 1.25))),
    );
    assert_eq!(scale(&scale_seeded).learned, Some(1.25));
    assert!(scale_seeded
        .edges(0)
        .all(|edge| edge.learned.is_none() && edge.trace.is_none() && !edge.initialized));
    let captured = scale_seeded.capture(0, &def).expect("a seeded scale");
    assert_eq!(captured.bundle_value(id), Some(1.25));
    assert_eq!(captured.value(all[1]), None);

    let edge_seeded = state(
        &def,
        Some(seeds(&def, |seeds| seed_edge(seeds, all[1], 0.75))),
    );
    assert_eq!(scale(&edge_seeded).learned, None);
    let edge = view_edge(&edge_seeded, all[1]);
    assert_eq!((edge.learned, edge.trace), (Some(0.75), Some(0.0)));
    assert!(edge.initialized && edge.learning.lamarckian);
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
            energy_scale: 200.0,
            age_ticks: 0.0,
            own_colour: [0.0; 3],
            previous_outcome: [0.0; 4],
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

/// Across a lone learner, an `Own` member, a crediting `Static` member and
/// a plain `Static` member, the executor's weighing, `applied_weight`,
/// `applied_weights` and the view's `applied` agree.
#[test]
fn applied_weight_agrees_with_the_executor_and_the_view() {
    let (mut def, id) = ring_def(0.5);
    let silent = def.add_ring_bundle(RING_REF, DirectedVoteKind::Move, 1);
    def.bundle_mut(silent).set_scale(-0.25);
    let all = members(&def, id);
    let learning = lamarckian(hebbian(HebbianRule::Classic, 0.1, 4.0));
    slots(&mut def, id, Some(learning), None);
    def.edge_mut(all[0]).set_weight(0.25);
    member(&mut def, all[0], MemberPolicy::Own(learning), false);
    member(&mut def, all[1], MemberPolicy::Static, true);
    def.edge_mut(all[2]).set_weight(-0.5);
    let lone_surface = def
        .sink_surface(OutputSinkKind::CustomOutput(0))
        .expect("a custom output sink");
    def.push_edge(
        lone_surface,
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: RING_REF,
                sub_idx: 3,
            },
            0.75,
        ),
    );
    let lone = EdgeOccurrence::new(lone_surface, 0);
    def.edge_mut(lone).set_learning(Some(learning));
    let birth = seeds(&def, |seeds| {
        seed_bundle(seeds, id, 1.25);
        seed_edge(seeds, all[0], 0.5);
        seed_edge(seeds, lone, -1.5);
    });
    let mut runtime = GraphRuntimeState::for_genome(&genome(&def), vec![Some(birth)]);
    let refs = refs();
    let expected = [
        (all[0], 0.5 + 1.25),
        (all[1], 1.25),
        (all[2], -0.5 + 1.25),
        (lone, -1.5),
    ];
    let learned = runtime.learned().clone();
    for (at, weight) in expected {
        assert_eq!(applied(&learned, at), weight);
        assert_eq!(learned.applied_weight(0, &def, &refs, at), weight);
    }
    for edge in learned.edges(0) {
        assert_eq!(edge.applied, applied(&learned, edge.occurrence));
    }
    // A member of a bundle without a learning scale is no dynamic edge and
    // resolves to its genomic weight.
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

/// T25.F11 invariant 4, the one member law: a fact member (on the drive)
/// reads pattern 1 like every member, so a crediting fact member credits
/// `1 × Δ` to the scale, an `Own` one learns its modifier on top of the
/// scale, and a `Static` fact member weighs `scale × 1 + modifier`.
#[test]
fn a_fact_member_weighs_and_learns_with_the_scale() {
    use crate::genome::cgp::Site;
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let layout = crate::contracts::pair_layout(&refs()[usize::from(PAIR_REF)], 0);
    let id = def.add_pair_bundle(PAIR_REF, 0, layout, DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(0.5);
    let facts: Vec<EdgeOccurrence> = members(&def, id)
        .into_iter()
        .filter(|&member| def.site_of(member) == Some(Site::Drive))
        .collect();
    assert_eq!(facts.len(), 3);
    assert!(facts.iter().all(|&fact| def.pattern(fact, &refs()) == 1.0));
    let classic = hebbian(HebbianRule::Classic, 1.0, 4.0);
    slots(&mut def, id, Some(classic), None);
    member(&mut def, facts[0], MemberPolicy::Static, true);
    def.edge_mut(facts[1]).set_weight(0.25);
    member(&mut def, facts[1], MemberPolicy::Own(classic), false);
    def.edge_mut(facts[2]).set_weight(-0.125);
    let mut learned = state(&def, None);
    drive(&mut learned, &def, |_| 0.5, 0.5);
    let delta = 1.0 * 0.5 * 0.5;
    let scaled = 0.5 + 1.0 * delta;
    assert_eq!(
        scale(&learned).learned,
        Some(scaled),
        "1 × Δ moves the scale"
    );
    assert_eq!(applied(&learned, facts[0]), scaled);
    assert_eq!(view_edge(&learned, facts[1]).learned, Some(0.25 + delta));
    assert_eq!(applied(&learned, facts[1]), scaled + (0.25 + delta));
    assert_eq!(
        applied(&learned, facts[2]),
        scaled - 0.125,
        "a static fact member"
    );
}

/// Invariant 5: the runtime reads only the layout. `runtime::plasticity`'s
/// code, comments aside, names no member policy, `credits` or bundle slot,
/// so `Inherit` is never resolved during a visit.
#[test]
fn the_runtime_names_no_policy_credits_or_slot() {
    // The sources hold no block comment and no `//` in a string literal, so
    // cutting each line at `//` leaves its code.
    let code_only = |source: &str| -> String {
        source
            .lines()
            .map(|line| line.split("//").next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    };
    for source in [
        include_str!("learned.rs"),
        include_str!("mod.rs"),
        include_str!("rules.rs"),
        include_str!("kind.rs"),
    ] {
        let code = code_only(source);
        for name in [
            "MemberPolicy",
            "credits",
            "scale_learning",
            "member_default",
            "LearningSlot",
        ] {
            assert!(!code.contains(name), "{name}");
        }
    }
}

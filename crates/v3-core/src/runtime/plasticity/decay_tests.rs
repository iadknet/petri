//! Learned-weight decay (T25.F09): the birth clamp, the per-tick law
//! `x ← b + ρ·(x − b)` toward each stored quantity's birth value, and its
//! bounds.

use proptest::prelude::*;

use super::rules::relax;
use super::LearnedState;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::genome::cgp::test_support::plastic_node;
use crate::creature::genome::cgp::{
    BirthSeeds, CgpGraphBackendDef, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
    GraphSource, GroupId, Scope, Shape,
};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, PlasticityConfig,
};

const RING_REF: u16 = 0;

fn refs() -> Vec<InputReference> {
    vec![InputReference::World(WorldInputKey::NeighborBarrierRing)]
}

fn config(rule: HebbianRule, eta: f32, clamp: f32, retention: f32) -> PlasticityConfig {
    PlasticityConfig {
        rule,
        learning_rate: eta,
        weight_clamp: clamp,
        lamarckian: true,
        modulation: None,
        retention,
    }
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

/// One compute node whose single input edge has genomic weight `weight`
/// and learning attributes `config`.
fn ungrouped(weight: f32, config: PlasticityConfig) -> CgpGraphBackendDef {
    CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight,
            )],
            Some(config),
        )])
        .with_output_sinks(vec![])
}

/// A full catalog with one `Ring` group (pattern 1) at genomic scale
/// `scale` and retention `retention`.
fn ring(scale: f32, retention: f32) -> (CgpGraphBackendDef, GroupId, Vec<EdgeOccurrence>) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_group(
        Shape::Ring { ring_ref: RING_REF },
        DirectedVoteKind::Move,
        0,
    );
    def.group_mut(id).set_scale(scale);
    def.group_mut(id).set_retention(retention);
    let members = def.members(id).collect();
    (def, id, members)
}

fn learn(def: &mut CgpGraphBackendDef, at: EdgeOccurrence, config: PlasticityConfig, scope: Scope) {
    let mut edge = def.edge_mut(at);
    edge.set_plasticity(Some(config));
    edge.set_scope(scope);
}

fn edge(state: &LearnedState, slot: usize) -> super::LearnedEdge {
    state.edges(0).nth(slot).expect("a learner")
}

fn scale(state: &LearnedState) -> super::LearnedScale {
    state.scales(0).next().expect("a scale slot")
}

fn applied(state: &LearnedState, at: EdgeOccurrence) -> f32 {
    state
        .applied_weights(0)
        .find(|&(occurrence, _)| occurrence == at)
        .map(|(_, weight)| weight)
        .expect("a dynamic edge")
}

/// One committed visit driven through the learner: every edge reads `pre`,
/// every container's post sample is `post`.
fn drive(state: &mut LearnedState, def: &CgpGraphBackendDef, pre: f32, post: f32) {
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
            learner.weigh(pre, def.genomic_weight(at, &refs()));
        }
        learner.end_row(post);
    }
    learner.commit();
}

fn compute_seed(def: &CgpGraphBackendDef, value: f32) -> BirthSeeds {
    BirthSeeds::Rows {
        compute: def
            .compute_nodes()
            .iter()
            .map(|node| vec![Some(value); node.inputs.len()])
            .collect(),
        sinks: def
            .output_sinks()
            .iter()
            .map(|sink| vec![None; sink.inputs.len()])
            .collect(),
        groups: vec![None; def.groups().len()],
    }
}

// ── Birth value (invariant 2) ───────────────────────────────────────────────

/// An unseeded learner whose genomic weight lies outside its clamp starts
/// at the clamp, which is its birth value; the view keeps the genomic.
#[test]
fn an_out_of_clamp_genomic_own_weight_starts_clamped() {
    let def = ungrouped(5.0, config(HebbianRule::Classic, 0.0, 2.0, 1.0));
    let mut learned = state(&def, None);
    learned.learner(0).expect("a learning backend").enter();
    let view = edge(&learned, 0);
    assert_eq!(
        (view.genomic, view.birth, view.learned),
        (5.0, 2.0, Some(2.0))
    );
    assert_eq!(view.applied, 2.0);
}

/// A seeded Lamarckian learner starts at its clamped seed, its birth value.
#[test]
fn a_lamarckian_learner_starts_at_its_clamped_seed() {
    let def = ungrouped(0.5, config(HebbianRule::Classic, 0.0, 2.0, 1.0));
    let seeds = compute_seed(&def, -7.0);
    let learned = state(&def, Some(seeds));
    let view = edge(&learned, 0);
    assert_eq!((view.birth, view.learned), (-2.0, Some(-2.0)));
}

/// An unseeded scale outside its bound starts at the bound, and a
/// zero-`Σ` write leaves it there.
#[test]
fn an_out_of_bound_genomic_scale_starts_at_its_bound() {
    let (mut def, _, members) = ring(7.0, 1.0);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.0, 2.0, 1.0),
        Scope::Group,
    );
    let mut learned = state(&def, None);
    let view = scale(&learned);
    assert_eq!((view.genomic, view.birth, view.retention), (7.0, 2.0, 1.0));
    drive(&mut learned, &def, 1.0, 1.0);
    assert_eq!(scale(&learned).learned, Some(2.0));
}

/// A scope-`Group` learner's out-of-clamp own correction is not a learned
/// quantity: it composes raw before and after scale learning.
#[test]
fn a_group_learners_out_of_clamp_correction_composes_raw() {
    let (mut def, _, members) = ring(0.5, 1.0);
    def.edge_mut(members[0]).set_weight(5.0);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.25, 2.0, 1.0),
        Scope::Group,
    );
    let mut learned = state(&def, None);
    assert_eq!(applied(&learned, members[0]), 5.5);
    drive(&mut learned, &def, 1.0, 1.0);
    let moved = scale(&learned).learned.expect("an initialized scale");
    assert_eq!(moved, 0.75);
    assert_eq!(applied(&learned, members[0]), 5.0 + moved);
    assert_eq!(edge(&learned, 0).learned, None);
}

// ── The law (invariant 3) ───────────────────────────────────────────────────

/// An initialized ungrouped learner relaxes toward its birth value by its
/// retention once per tick, exactly by the law.
#[test]
fn an_ungrouped_learner_relaxes_toward_its_birth_value() {
    let def = ungrouped(0.5, config(HebbianRule::Classic, 0.0, 2.0, 0.5));
    let mut learned = state(&def, None);
    learned.set_slot(0, 0, 1.5, 0.0);
    learned.begin_tick();
    assert_eq!(edge(&learned, 0).learned, Some(relax(1.5, 0.5, 0.5)));
    assert_eq!(edge(&learned, 0).learned, Some(1.0));
    learned.begin_tick();
    assert_eq!(edge(&learned, 0).learned, Some(0.75));
}

/// An edge-scope member decays toward its birth own correction, not its
/// composed weight.
#[test]
fn an_edge_scope_member_decays_toward_its_own_correction() {
    let (mut def, _, members) = ring(1.0, 1.0);
    def.edge_mut(members[0]).set_weight(0.25);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.0, 2.0, 0.5),
        Scope::Edge,
    );
    let mut learned = state(&def, None);
    learned.set_slot(0, 0, 1.25, 0.0);
    learned.begin_tick();
    let view = edge(&learned, 0);
    assert_eq!((view.birth, view.learned), (0.25, Some(0.75)));
    assert_eq!(view.applied, 1.75);
}

/// A scope-`Group` member has no own decay, whatever its retention, while
/// its scale decays by its group's retention.
#[test]
fn a_group_scale_decays_by_its_groups_retention_and_its_learner_has_none() {
    let (mut def, _, members) = ring(0.5, 0.5);
    def.edge_mut(members[0]).set_weight(0.125);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.0, 4.0, 0.0),
        Scope::Group,
    );
    let mut learned = state(&def, None);
    assert_eq!(
        (scale(&learned).birth, scale(&learned).retention),
        (0.5, 0.5)
    );
    learned.set_scale(0, 0, 2.5);
    learned.begin_tick();
    assert_eq!(scale(&learned).learned, Some(1.5));
    assert_eq!(applied(&learned, members[0]), 1.625);
    assert_eq!(edge(&learned, 0).learned, None);
}

/// `ρ = 1` leaves every stored f32 bit-identical, tick after tick.
#[test]
fn a_retention_of_one_leaves_every_stored_value_bit_identical() {
    let (mut def, _, members) = ring(0.5, 1.0);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.0, 2.0, 1.0),
        Scope::Edge,
    );
    learn(
        &mut def,
        members[1],
        config(HebbianRule::Classic, 0.0, 2.0, 1.0),
        Scope::Group,
    );
    let mut learned = state(&def, None);
    learned.set_slot(0, 0, 0.1, 0.0);
    learned.set_scale(0, 0, -0.3);
    let before = learned.clone();
    for _ in 0..1_000 {
        learned.begin_tick();
    }
    assert_eq!(learned, before);
    assert_eq!(
        edge(&learned, 0).learned.map(f32::to_bits),
        Some(0.1f32.to_bits())
    );
    assert_eq!(
        scale(&learned).learned.map(f32::to_bits),
        Some((-0.3f32).to_bits())
    );
}

/// An uninitialized learner or scale slot holds its birth value and is
/// not decayed, even at `ρ = 0`.
#[test]
fn an_uninitialized_slot_is_untouched() {
    let (mut def, _, members) = ring(3.0, 0.0);
    learn(
        &mut def,
        members[0],
        config(HebbianRule::Classic, 0.0, 2.0, 0.0),
        Scope::Group,
    );
    def.edge_mut(members[1]).set_weight(1.5);
    learn(
        &mut def,
        members[1],
        config(HebbianRule::Classic, 0.0, 2.0, 0.0),
        Scope::Edge,
    );
    let mut learned = state(&def, None);
    let before = learned.clone();
    learned.begin_tick();
    assert_eq!(learned, before);
    assert_eq!(scale(&learned).learned, None);
    assert_eq!(edge(&learned, 1).learned, None);
}

/// Oja's pull and the decay are two bounded attractors: a strongly driven
/// decaying Oja learner never leaves its clamp.
#[test]
fn oja_plus_decay_stays_inside_the_clamp() {
    let def = ungrouped(0.0, config(HebbianRule::Oja, 1.0, 1.5, 0.9));
    let mut learned = state(&def, None);
    for tick in 0..500 {
        learned.begin_tick();
        let pre = if tick % 2 == 0 { 9.0 } else { -9.0 };
        drive(&mut learned, &def, pre, 3.0);
        let value = edge(&learned, 0).learned.expect("initialized");
        assert!(value.is_finite() && value.abs() <= 1.5, "{value} at {tick}");
    }
}

// ── Bounds (invariant 4) ────────────────────────────────────────────────────

/// A clamp, then a birth value and a stored value inside it.
fn bounded() -> impl Strategy<Value = (f32, f32, f32)> {
    (0.01f32..=10.0).prop_flat_map(|clamp| (Just(clamp), -clamp..=clamp, -clamp..=clamp))
}

proptest! {
    /// For `x, b ∈ [−c, c]` and `ρ ∈ [0, 1)` the relaxed value lies between
    /// `b` and `x` inclusive, so it never crosses `b`, never leaves the
    /// clamp and never moves away from `b`.
    #[test]
    fn relax_stays_between_birth_and_value((clamp, birth, value) in bounded(), retention in 0.0f32..1.0) {
        let relaxed = relax(value, birth, retention);
        let (low, high) = (birth.min(value), birth.max(value));
        prop_assert!(low <= relaxed && relaxed <= high, "{relaxed} outside [{low}, {high}]");
        prop_assert!(relaxed.abs() <= clamp);
        prop_assert!((relaxed - birth).abs() <= (value - birth).abs());
    }
}

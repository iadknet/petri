//! T25.F03, T25.F11 bundle operators: catalog, applicability, pinned draw
//! orders, and the operators whose draws a bundle must not change.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::votes::{DirectedVoteKind, VoteSink, VOTE_SINK_COUNT};
use crate::contracts::OutcomeChannel;
use crate::contracts::{FieldSet, InputReference, NodeId, SensorDescriptor, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::cgp::test_support::{add_bundle_on, edit_sinks};
use crate::genome::cgp::{
    compose, detach, validate_graph, BirthSeeds, BundleBirth, BundleId, BundleSource,
    CgpGraphBackendDef, EdgeOccurrence, EdgeSurface, GraphSource, LearningSlot, OutputSinkKind,
    Site, Wiring,
};
use crate::genome::learning::GraphMapping;
use crate::genome::{
    BackendDef, CreatureGenome, HebbianRule, Learning, MemberPolicy, NodeGenome,
    RewardModulationConfig, VmBackendDef,
};
use crate::mutation::event::{ComplexityEffect, MutationSkipReason};
use crate::mutation::graph::{bundle, GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::MutationOperator;

const BUNDLE_OPERATORS: [GraphOperator; 6] = [
    GraphOperator::AddBundleLive,
    GraphOperator::AddBundleSilent,
    GraphOperator::AlterBundleScale,
    GraphOperator::ShiftBundleOffset,
    GraphOperator::RetargetBundle,
    GraphOperator::RemoveBundle,
];

const CREATION: [GraphOperator; 2] = [GraphOperator::AddBundleLive, GraphOperator::AddBundleSilent];

fn barrier() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn food(t: u16) -> InputReference {
    InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        t,
    )))
}

fn area_barrier() -> InputReference {
    InputReference::World(WorldInputKey::AreaBarrierSummary)
}

fn area_food(t: u16) -> InputReference {
    InputReference::World(WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(t)))
}

fn nearby() -> InputReference {
    InputReference::World(WorldInputKey::NearbyCreature)
}

/// The refs every grouped fixture reads: rings at 0 and 2.
fn refs() -> Vec<InputReference> {
    vec![barrier(), InputReference::ActionVotes, food(0)]
}

fn graph_node(refs: Vec<InputReference>, def: CgpGraphBackendDef) -> NodeGenome {
    NodeGenome::new(NodeId::new(0), refs, BackendDef::Graph(def), vec![])
}

fn graph(node: &NodeGenome) -> &CgpGraphBackendDef {
    match node.backend_def() {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("Graph fixture"),
    }
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    match node.backend_edit().1 {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("Graph fixture"),
    }
}

/// Three bundles: Move at 0 (scale 0), Reproduce at 5 (0.5) and StealEnergy
/// at 7 (−2), on rings 0, 2, 0.
fn three() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for (ring_ref, kind, offset, scale) in [
        (0, DirectedVoteKind::Move, 0, 0.0),
        (2, DirectedVoteKind::Reproduce, 5, 0.5),
        (0, DirectedVoteKind::StealEnergy, 7, -2.0),
    ] {
        let id = def.add_ring_bundle(ring_ref, kind, offset);
        def.bundle_mut(id).set_scale(scale);
    }
    def
}

fn id(index: usize) -> BundleId {
    BundleId::new(u16::try_from(index).unwrap())
}

const RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

const CHANNELS: [OutcomeChannel; 4] = [
    OutcomeChannel::EnergyDelta,
    OutcomeChannel::ActionSuccess,
    OutcomeChannel::DamageDelta,
    OutcomeChannel::OffspringSuccess,
];

/// One `draw_mode` sequence, replayed from `rng` in its pinned order
/// (T25.F07 invariant 7): a mode in `0..3`; for a learning mode the rule,
/// rate, clamp and inheritance of `EnableHebbian`, then for mode 2 the
/// channel and decay of `EnableRewardModulation`. Returns the mode and the
/// learning.
fn replay_mode(rng: &mut SmallRng) -> (u8, Option<Learning>) {
    let mode = rng.gen_range(0..3u8);
    let config = (mode > 0).then(|| {
        let rule = RULES[rng.gen_range(0..4)];
        let learning_rate = rng.gen_range(0.01f32..0.2);
        let weight_clamp = rng.gen_range(1.0f32..5.0);
        let lamarckian = rng.gen_bool(0.5);
        let modulation = (mode == 2).then(|| {
            let reward_source = CHANNELS[rng.gen_range(0..4)];
            RewardModulationConfig {
                reward_source,
                trace_decay: rng.gen_range(0.5f32..0.99),
            }
        });
        Learning {
            rule,
            learning_rate,
            weight_clamp,
            lamarckian,
            modulation,
            retention: 1.0,
        }
    });
    (mode, config)
}

/// One member's draws (T25.F12 invariant 10): the policy (`gen_bool(0.5)`
/// true `Inherit`, else a mode: absent `Static`, present `Own`), then the
/// `credits` coin.
fn replay_member(rng: &mut SmallRng) -> (MemberPolicy, bool) {
    let policy = if rng.gen_bool(0.5) {
        MemberPolicy::Inherit
    } else {
        replay_mode(rng)
            .1
            .map_or(MemberPolicy::Static, MemberPolicy::Own)
    };
    (policy, rng.gen_bool(0.5))
}

/// A bundle's learning draws on bundle `bundle` of `want` (T25.F12
/// invariant 10): `scale_learning` then `member_default` by mode, then the
/// uniform form's one member sequence for every member or the randomized
/// form's one per member in canonical order.
fn replay_learning(
    want: &mut CgpGraphBackendDef,
    bundle: BundleId,
    randomized: bool,
    rng: &mut SmallRng,
) {
    let (_, scale) = replay_mode(rng);
    let (_, default) = replay_mode(rng);
    want.bundle_mut(bundle).set_slot(LearningSlot::Scale, scale);
    want.bundle_mut(bundle)
        .set_slot(LearningSlot::MemberDefault, default);
    let uniform = (!randomized).then(|| replay_member(rng));
    for member in want.members(bundle).collect::<Vec<_>>() {
        let (policy, credits) = uniform.unwrap_or_else(|| replay_member(rng));
        let mut edge = want.edge_mut(member);
        edge.set_policy(policy);
        edge.set_credits(credits);
    }
}

#[test]
fn the_catalog_places_the_bundle_operators_at_their_tiers() {
    assert_eq!(GraphOperator::ALL.len(), 33);
    assert_eq!(&GraphOperator::ALL[22..28], &BUNDLE_OPERATORS);
    // T25.F12, T25.F08: `ToggleCredits` (in `SetScope`'s place) and
    // `DetachGraphEdge` follow `RemoveBundle`.
    assert_eq!(
        &GraphOperator::ALL[28..30],
        &[GraphOperator::ToggleCredits, GraphOperator::DetachGraphEdge]
    );
    assert_eq!(
        BUNDLE_OPERATORS.map(GraphOperator::weight),
        [4, 4, 4, 2, 2, 2]
    );
    let total: u16 = GraphOperator::ALL
        .iter()
        .map(|op| u16::from(op.weight()))
        .sum();
    assert_eq!(
        total, 73,
        "the Graph catalog's weight (T25.F12 invariant 9)"
    );
    assert_eq!(
        BUNDLE_OPERATORS.map(|op| MutationOperator::from(op).as_key()),
        [
            "Graph.AddBundleLive",
            "Graph.AddBundleSilent",
            "Graph.AlterBundleScale",
            "Graph.ShiftBundleOffset",
            "Graph.RetargetBundle",
            "Graph.RemoveBundle",
        ]
    );
    for (op, effect) in [
        (GraphOperator::ToggleCredits, ComplexityEffect::Neutral),
        (GraphOperator::DetachGraphEdge, ComplexityEffect::Neutral),
    ] {
        assert_eq!(op.weight(), 2, "{op:?}");
        assert_eq!(op.complexity_effect(), effect, "{op:?}");
    }
    // Restricted size pressure admits only the decreasing operator, and the
    // operator and its key agree on the complexity effect.
    let admitted: Vec<_> = BUNDLE_OPERATORS
        .into_iter()
        .filter(|op| op.complexity_effect().is_decreasing())
        .collect();
    assert_eq!(admitted, vec![GraphOperator::RemoveBundle]);
    for add in CREATION {
        assert_eq!(add.complexity_effect(), ComplexityEffect::Increasing);
    }
    for op in BUNDLE_OPERATORS {
        assert_eq!(
            op.complexity_effect(),
            MutationOperator::from(op).complexity_effect(),
            "{op:?}"
        );
    }
}

/// The creation operators apply wherever a declared sensor's field set or
/// a memory bank has an admissible plan: on every Graph node with a vote
/// sink, a bare one included, never on a Graph node without one or a VM
/// node; the unit operators need a bundle, `ShiftBundleOffset` a geometric
/// one (T25.F11 invariant 7).
#[test]
fn applicability_follows_sources_bundles_and_wirings() {
    let genome = |refs, def| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(refs, def)],
    };
    let full = CgpGraphBackendDef::new_with_fixed_outputs;
    let voteless = || {
        let mut def = full();
        edit_sinks(&mut def, |sinks| {
            sinks.retain(|sink| !matches!(sink.kind, OutputSinkKind::ActionVote(_)));
        });
        def
    };
    let bare = genome(vec![], full());
    let mute = genome(vec![barrier(), area_barrier()], voteless());
    let grouped = genome(refs(), three());
    let mut plain_only = full();
    let source = BundleSource::Memory { previous: false };
    let birth = BundleBirth {
        field_set: 0,
        wiring: Wiring::Plain {
            sink: VoteSink::Eat,
        },
        scale: 0.0,
    };
    add_bundle_on(&mut plain_only, &[], source, birth);
    let plain_only = genome(vec![], plain_only);
    let context = crate::config::test_sensor_context(1);
    let applies =
        |g: &CreatureGenome, op| !GraphMutator::applicable_indices(g, op, &context).is_empty();
    for op in BUNDLE_OPERATORS {
        let creation = CREATION.contains(&op);
        assert_eq!(applies(&bare, op), creation, "{op:?}");
        assert!(!applies(&mute, op), "{op:?}");
        assert!(applies(&grouped, op), "{op:?}");
        let geometric_only = op == GraphOperator::ShiftBundleOffset;
        assert_eq!(applies(&plain_only, op), !geometric_only, "{op:?}");
    }
    let vm = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(
            NodeId::new(0),
            vec![barrier()],
            BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![],
                program: vec![],
            }),
            vec![],
        )],
    };
    for op in BUNDLE_OPERATORS {
        assert!(!applies(&vm, op), "{op:?}");
    }
}

/// The refs the creation tests read: a `Plain` sensor, a ring and a
/// two-pair summary.
fn creation_refs() -> Vec<InputReference> {
    vec![InputReference::ActionVotes, food(1), area_barrier()]
}

/// The admissible sources of [`creation_refs`] on a full catalog, in the
/// order `AddBundle*` draws over: declared sensors by `ref_idx`, field sets
/// ascending, then the two memory banks.
fn creation_sources() -> Vec<(BundleSource, FieldSet)> {
    let context = crate::config::test_sensor_context(1);
    let sensor = |ref_idx: u16, field_set: u8| {
        let reference = &creation_refs()[usize::from(ref_idx)];
        let set = SensorDescriptor::of(reference, &context)
            .field_set(field_set)
            .unwrap();
        (BundleSource::Sensor { ref_idx, field_set }, set)
    };
    vec![
        sensor(0, 0),
        sensor(1, 0),
        sensor(2, 0),
        sensor(2, 1),
        (
            BundleSource::Memory { previous: false },
            crate::contracts::MEMORY_BANK,
        ),
        (
            BundleSource::Memory { previous: true },
            crate::contracts::MEMORY_BANK,
        ),
    ]
}

/// One creation event `op` through the dispatch on an empty full-catalog
/// graph declaring [`creation_refs`]; returns the node and the event's RNG
/// after it.
fn create(op: GraphOperator, seed: u64) -> (NodeGenome, SmallRng) {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(
            creation_refs(),
            CgpGraphBackendDef::new_with_fixed_outputs(),
        )],
    };
    let mut rng = SmallRng::seed_from_u64(seed);
    GraphMutator::apply_to_node(
        &mut genome,
        op,
        0,
        &mut rng,
        &MutationConfig::default(),
        &crate::config::test_sensor_context(1),
    )
    .unwrap();
    (genome.nodes.remove(0), rng)
}

/// `AddBundle*`'s draws, replayed (T25.F11 invariants 6–7): the source over
/// [`creation_sources`]; the plan (a `Plain` set's sink over the 39 vote
/// sinks; a geometric set's kind, then offset); the live scale; the form
/// coin; the learning draws of that form. Returns the bundle the container
/// builds from them and whether the form was randomized.
fn replay_creation(op: GraphOperator, rng: &mut SmallRng) -> (CgpGraphBackendDef, bool) {
    let sources = creation_sources();
    let (source, set) = sources[rng.gen_range(0..sources.len())];
    let wiring = match set {
        FieldSet::Plain { .. } => Wiring::Plain {
            sink: VoteSink::from_index(rng.gen_range(0..VOTE_SINK_COUNT)).unwrap(),
        },
        FieldSet::Ring | FieldSet::Pair(_) => {
            let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
            let offset = rng.gen_range(0..8u8);
            match set {
                FieldSet::Ring => Wiring::Ring { kind, offset },
                _ => Wiring::DirectionPair { kind, offset },
            }
        }
    };
    let scale = if op == GraphOperator::AddBundleLive {
        rng.gen_range(-1.0f32..=1.0)
    } else {
        0.0
    };
    let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
    let birth = BundleBirth {
        field_set: source.field_set(),
        wiring,
        scale,
    };
    let bundle = add_bundle_on(&mut want, &creation_refs(), source, birth);
    let randomized = rng.gen_bool(0.5);
    replay_learning(&mut want, bundle, randomized, rng);
    (want, randomized)
}

/// Whether the edge at `member` is a member of `bundle` reading `field` of
/// its source on `site`: the fact a birth coordinate's location claims.
fn located_member_reads(
    def: &CgpGraphBackendDef,
    bundle: BundleId,
    member: EdgeOccurrence,
    field: u16,
    site: Site,
) -> bool {
    let edge = def.edge_at(member);
    let reads_field = match (edge.source(), def.bundles()[bundle.index()].source()) {
        (GraphSource::InputLeaf { ref_idx, sub_idx }, BundleSource::Sensor { ref_idx: at, .. }) => {
            ref_idx == at && sub_idx == field
        }
        (GraphSource::SharedMemory { slot, previous }, BundleSource::Memory { previous: bank }) => {
            previous == bank && u16::from(slot) == field
        }
        _ => false,
    };
    edge.membership().is_some_and(|m| m.bundle == bundle)
        && reads_field
        && def.site_of(member) == Some(site)
}

proptest::proptest! {
    /// Both creation operators draw the source, the plan, the live scale,
    /// the form coin and that form's learning draws, and nothing else; the
    /// bundle wires its whole field set, live or silent by the operator
    /// (T25.F11 invariants 5–7).
    #[test]
    fn add_bundle_draws_source_plan_scale_form_then_learning(
        seed in proptest::prelude::any::<u64>(),
        live in proptest::prelude::any::<bool>(),
    ) {
        let op = if live { GraphOperator::AddBundleLive } else { GraphOperator::AddBundleSilent };
        let (node, mut rng) = create(op, seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        let (want, _) = replay_creation(op, &mut expected);
        proptest::prop_assert_eq!(graph(&node), &want);
        proptest::prop_assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
        proptest::prop_assert!(validate_graph(&node).is_ok());
        let def = graph(&node);
        let coordinates = def.birth_coordinates(id(0), node.input_refs(), &crate::contracts::SensorContext::WIDEST);
        for c in &coordinates {
            let member = c.member.expect("the whole set");
            proptest::prop_assert!(
                located_member_reads(def, id(0), member, c.field, c.site),
                "{c:?}"
            );
        }
        proptest::prop_assert!(def.edges().all(|edge| edge.own_weight() == 0.0));
        if !live {
            proptest::prop_assert_eq!(def.bundles()[0].scale(), 0.0, "born silent");
        }
    }
}

/// Over seeds the creation operators reach every source, every learning
/// mode and both forms, and the live one both signs.
#[test]
fn creations_reach_every_source_mode_and_form() {
    let mut sources = std::collections::BTreeSet::new();
    let mut forms = [false; 2];
    let mut modes = [false; 3];
    let mut signs = [false; 2];
    for seed in 0..256 {
        for op in CREATION {
            let (node, _) = create(op, seed);
            let def = graph(&node);
            let record = def.bundles()[0];
            sources.insert(format!("{:?}", record.source()));
            let mut replay = SmallRng::seed_from_u64(seed);
            let (_, randomized) = replay_creation(op, &mut replay);
            forms[usize::from(randomized)] = true;
            let slots = LearningSlot::ALL.map(|slot| record.slot(slot));
            let stored = def
                .members(id(0))
                .map(|member| def.edge_at(member).stored_learning());
            for learning in slots.into_iter().chain(stored) {
                let mode = match learning {
                    None => 0,
                    Some(config) if config.modulation.is_none() => 1,
                    Some(_) => 2,
                };
                modes[mode] = true;
            }
            if op == GraphOperator::AddBundleLive {
                signs[usize::from(record.scale() < 0.0)] = true;
            }
        }
    }
    assert_eq!(sources.len(), creation_sources().len());
    assert_eq!((forms, modes, signs), ([true; 2], [true; 3], [true; 2]));
}

/// Randomized members draw their own policies and coins: some seed gives
/// one bundle members of two policies and members on both sides of the
/// `credits` coin.
#[test]
fn randomized_members_differ_within_one_bundle() {
    let mut mixed = [false; 2];
    for seed in 0..64 {
        for op in CREATION {
            let (node, _) = create(op, seed);
            let def = graph(&node);
            let edges: Vec<_> = def.members(id(0)).map(|at| def.edge_at(at)).collect();
            let policy =
                |e: &&crate::genome::cgp::GraphEdge| e.membership().unwrap().policy.as_key();
            mixed[0] |= edges.iter().any(|e| policy(e) != policy(&edges[0]));
            let credits = |e: &&crate::genome::cgp::GraphEdge| e.membership().unwrap().credits;
            mixed[1] |= edges.iter().any(credits) && !edges.iter().all(credits);
        }
    }
    assert_eq!(mixed, [true; 2]);
}

/// A creation on a Graph node without a vote sink has no admissible source
/// and skips without a draw, writing nothing.
#[test]
fn add_bundle_skips_without_a_draw_without_an_admissible_source() {
    let mut voteless = CgpGraphBackendDef::new_with_fixed_outputs();
    edit_sinks(&mut voteless, |sinks| {
        sinks.retain(|sink| !matches!(sink.kind, OutputSinkKind::ActionVote(_)));
    });
    for law in [bundle::BirthLaw::Live, bundle::BirthLaw::Silent] {
        let mut node = graph_node(vec![barrier(), area_barrier()], voteless.clone());
        let before = node.clone();
        let mut rng = SmallRng::seed_from_u64(9);
        let mut untouched = SmallRng::seed_from_u64(9);
        assert_eq!(
            bundle::add_bundle_by_law(
                &mut node,
                law,
                &crate::config::test_sensor_context(1),
                &mut rng
            ),
            Err(MutationSkipReason::NoApplicableTarget)
        );
        assert_eq!(node, before);
        assert_eq!(rng.gen::<u64>(), untouched.gen::<u64>());
    }
}

/// A sensor at index 65,535 is a bundle source; one past it is not.
#[test]
fn bundle_sources_reach_the_last_u16_index_and_none_past_it() {
    let context = crate::config::test_sensor_context(1);
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    let mut refs = vec![InputReference::CommitCounts; usize::from(u16::MAX)];
    refs.push(barrier());
    let last: Vec<_> = bundle::bundle_sources(&def, &refs, &context).collect();
    assert_eq!(last.len(), refs.len() + 2);
    assert!(last.contains(&(
        BundleSource::Sensor {
            ref_idx: u16::MAX,
            field_set: 0
        },
        FieldSet::Ring
    )));
    refs.insert(0, InputReference::CommitCounts);
    let past: Vec<_> = bundle::bundle_sources(&def, &refs, &context).collect();
    assert_eq!(past.len(), usize::from(u16::MAX) + 1 + 2);
    assert!(past.iter().all(|(_, set)| *set != FieldSet::Ring));
}

#[test]
fn unit_operators_draw_the_bundle_first_and_change_its_record() {
    for seed in 0..64 {
        let before = three();

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::alter_scale(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let mut scale = before.bundles()[idx].scale();
        crate::mutation::graph::operators::perturb_weight(&mut scale, &mut expected);
        let mut want = before.clone();
        want.bundle_mut(id(idx)).set_scale(scale);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::shift_offset(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let step = if expected.gen_bool(0.5) { 1 } else { 7 };
        let mut want = before.clone();
        let wiring = before.bundles()[idx].wiring();
        let (kind, offset) = (wiring.kind().unwrap(), wiring.offset().unwrap());
        want.relocate_bundle(id(idx), wiring.rotated(kind, (offset + step) % 8));
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::retarget(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let wiring = before.bundles()[idx].wiring();
        let others: Vec<_> = DirectedVoteKind::ALL
            .into_iter()
            .filter(|&k| Some(k) != wiring.kind())
            .collect();
        let mut want = before.clone();
        want.relocate_bundle(
            id(idx),
            wiring.rotated(
                others[expected.gen_range(0..others.len())],
                wiring.offset().unwrap(),
            ),
        );
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::remove(graph_mut(&mut node), &mut rng).unwrap();
        let mut want = before.clone();
        want.remove_bundle(id(expected.gen_range(0..3)));
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());
    }
    let mut empty = CgpGraphBackendDef::new_with_fixed_outputs();
    assert_eq!(
        bundle::remove(&mut empty, &mut SmallRng::seed_from_u64(0)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}

/// A `Plain` bundle on `sink` reading the first memory bank (a fixture).
fn plain_on(def: &mut CgpGraphBackendDef, sink: VoteSink) -> BundleId {
    let birth = BundleBirth {
        field_set: 0,
        wiring: Wiring::Plain { sink },
        scale: 0.25,
    };
    add_bundle_on(def, &[], BundleSource::Memory { previous: false }, birth)
}

/// `RetargetBundle` on a `Plain` bundle: the bundle, then one `gen_range`
/// over the other present vote sinks in catalog order; every member moves
/// to the new sink. With no other vote sink it skips before that draw
/// (T25.F11 invariant 7).
#[test]
fn retarget_moves_a_plain_bundle_to_another_present_vote_sink() {
    let mut reached = std::collections::BTreeSet::new();
    for seed in 0..512 {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let bundle = plain_on(&mut def, VoteSink::Eat);
        let before = def.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        bundle::retarget(&mut def, &mut rng).unwrap();
        let mut expected = SmallRng::seed_from_u64(seed);
        expected.gen_range(0..1);
        let others: Vec<VoteSink> = VoteSink::all().filter(|&s| s != VoteSink::Eat).collect();
        let sink = others[expected.gen_range(0..others.len())];
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
        let mut want = before;
        want.relocate_bundle(bundle, Wiring::Plain { sink });
        assert_eq!(def, want, "seed {seed}");
        assert_eq!(def.members(bundle).count(), 16);
        reached.insert(sink.index());
    }
    assert_eq!(reached.len(), VOTE_SINK_COUNT - 1);

    let mut lone = CgpGraphBackendDef::new_with_fixed_outputs();
    edit_sinks(&mut lone, |sinks| {
        sinks.retain(
            |sink| !matches!(sink.kind, OutputSinkKind::ActionVote(vote) if vote != VoteSink::Eat),
        );
    });
    plain_on(&mut lone, VoteSink::Eat);
    let before = lone.clone();
    let mut rng = SmallRng::seed_from_u64(3);
    let mut expected = SmallRng::seed_from_u64(3);
    assert_eq!(
        bundle::retarget(&mut lone, &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(lone, before);
    expected.gen_range(0..1);
    assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "the pick only");
}

/// `AddSensor*` on a Graph node draws and writes exactly what it would
/// without bundles: the new bundles are the same, the incumbent bundle
/// untouched.
#[test]
fn add_sensor_is_unchanged_by_bundles() {
    for op in [
        InputRefOperator::AddSensorLive,
        InputRefOperator::AddSensorSilent,
    ] {
        for seed in 0..64 {
            let mut plain = v3alpha1_founder_genome();
            let idx = plain
                .nodes
                .iter()
                .position(|node| matches!(node.backend_def(), BackendDef::Graph(_)))
                .unwrap();
            let ring = u16::try_from(plain.nodes[idx].input_refs().len()).unwrap();
            plain.nodes[idx].input_refs_mut().push(barrier());
            let mut grouped = plain.clone();
            graph_mut(&mut grouped.nodes[idx]).add_ring_bundle(
                ring,
                DirectedVoteKind::StealEnergy,
                2,
            );

            let mut rngs = [SmallRng::seed_from_u64(seed), SmallRng::seed_from_u64(seed)];
            for (genome, rng) in [&mut plain, &mut grouped].into_iter().zip(&mut rngs) {
                InputRefMutator::apply_to_node(
                    genome,
                    op,
                    idx,
                    rng,
                    &crate::config::test_sensor_context(2),
                )
                .unwrap();
            }
            let [a, b] = &mut rngs;
            assert_eq!(a.gen::<u64>(), b.gen::<u64>(), "seed {seed}");
            assert_eq!(
                plain.nodes[idx].input_refs(),
                grouped.nodes[idx].input_refs()
            );
            let (p, q) = (graph(&plain.nodes[idx]), graph(&grouped.nodes[idx]));
            let independent = |def: &CgpGraphBackendDef| -> Vec<_> {
                def.edges()
                    .filter(|edge| edge.membership().is_none())
                    .copied()
                    .collect()
            };
            assert_eq!(independent(p), independent(q));
            assert_eq!(p.compute_nodes(), q.compute_nodes());
            assert_eq!(p.bundles(), &q.bundles()[1..]);
            assert_eq!(q.members(id(0)).count(), 8);
        }
    }
}

/// Each unit operator reaches its own edit through the mutator's dispatch.
#[test]
fn the_mutator_dispatches_each_unit_operator_to_its_bundle_edit() {
    type Edit =
        fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<GraphMapping, MutationSkipReason>;
    let cases: [(GraphOperator, Edit); 4] = [
        (GraphOperator::AlterBundleScale, bundle::alter_scale),
        (GraphOperator::ShiftBundleOffset, bundle::shift_offset),
        (GraphOperator::RetargetBundle, bundle::retarget),
        (GraphOperator::RemoveBundle, |def, rng| {
            bundle::remove(def, rng).map(|(_, map)| map)
        }),
    ];
    let config = MutationConfig::default();
    for (op, edit) in cases {
        for seed in 0..16 {
            // Arrange
            let mut genome = CreatureGenome {
                entry_node_id: NodeId::new(0),
                nodes: vec![graph_node(refs(), three())],
            };
            let mut direct = graph_node(refs(), three());

            // Act
            GraphMutator::apply_to_node(
                &mut genome,
                op,
                0,
                &mut SmallRng::seed_from_u64(seed),
                &config,
                &crate::config::test_sensor_context(1),
            )
            .unwrap();
            edit(graph_mut(&mut direct), &mut SmallRng::seed_from_u64(seed)).unwrap();

            // Assert
            assert_eq!(
                graph(&genome.nodes[0]),
                graph(&direct),
                "{op:?} seed {seed}"
            );
        }
    }
}

/// The unit operators pick one bundle uniformly over a list mixing every
/// wiring, `ShiftBundleOffset` over the geometric ones only.
#[test]
fn unit_operators_pick_uniformly_over_every_wiring() {
    let refs = vec![barrier(), area_food(0)];
    let mixed = || {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.add_ring_bundle(0, DirectedVoteKind::Move, 1);
        plain_on(&mut def, VoteSink::Decide);
        let layout = crate::contracts::pair_layout(&refs[1], 1);
        let id = def.add_pair_bundle(1, 1, layout, DirectedVoteKind::Reproduce, 6);
        def.bundle_mut(id).set_scale(0.25);
        def
    };
    let mut seen = [false; 3];
    for seed in 0..64 {
        let before = mixed();
        let geometric = [0usize, 2];
        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::shift_offset(graph_mut(&mut node), &mut rng).unwrap();
        let idx = geometric[expected.gen_range(0..2)];
        let step = if expected.gen_bool(0.5) { 1 } else { 7 };
        let wiring = before.bundles()[idx].wiring();
        let (kind, offset) = (wiring.kind().unwrap(), wiring.offset().unwrap());
        let mut want = before.clone();
        want.relocate_bundle(id(idx), wiring.rotated(kind, (offset + step) % 8));
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::alter_scale(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        seen[idx] = true;
        let mut scale = before.bundles()[idx].scale();
        crate::mutation::graph::operators::perturb_weight(&mut scale, &mut expected);
        let mut want = before.clone();
        want.bundle_mut(id(idx)).set_scale(scale);
        assert_eq!(graph(&node), &want);

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        bundle::retarget(graph_mut(&mut node), &mut rng).unwrap();
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        bundle::remove(graph_mut(&mut node), &mut rng).unwrap();
        let mut want = before;
        want.remove_bundle(id(expected.gen_range(0..3)));
        assert_eq!(graph(&node), &want);
        assert!(validate_graph(&node).is_ok());
    }
    assert_eq!(seen, [true; 3]);
}

/// The founder's Graph node with an appended `NearbyCreature` sensor wired
/// whole: four pair bundles of ten fields each (T25.F11 invariant 1).
#[test]
fn a_nearby_creature_sensor_wires_four_pair_bundles() {
    let mut node = graph_node(vec![], CgpGraphBackendDef::new_with_fixed_outputs());
    let context = crate::config::test_sensor_context(1);
    bundle::add_sensor_by_law(
        &mut node,
        nearby(),
        bundle::BirthLaw::Silent,
        &context,
        &mut SmallRng::seed_from_u64(1),
    )
    .unwrap();
    let def = graph(&node);
    assert_eq!(def.bundles().len(), 4);
    for (k, record) in def.bundles().iter().enumerate() {
        assert!(matches!(record.wiring(), Wiring::DirectionPair { .. }));
        let members = def.members(id(k)).count();
        assert_eq!(members, 12 + 8, "twelve compass members and eight facts");
    }
    assert!(validate_graph(&node).is_ok());
}

/// A one-ring-bundle node at scale 0.5 whose first member (canonical order)
/// is a Lamarckian learner at own weight 0.25 with an edge seed of 9.0; the
/// other seven are static and unseeded, and the bundle carries no seed.
fn detach_fixture() -> (NodeGenome, EdgeOccurrence) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let bundle = def.add_ring_bundle(0, DirectedVoteKind::Move, 3);
    def.bundle_mut(bundle).set_scale(0.5);
    let first = def.members(bundle).next().unwrap();
    let config = Learning {
        rule: HebbianRule::Oja,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian: true,
        modulation: None,
        retention: 1.0,
    };
    def.edge_mut(first).set_policy(MemberPolicy::Own(config));
    def.edge_mut(first).set_credits(true);
    def.edge_mut(first).set_weight(0.25);
    let compute = def
        .compute_nodes()
        .iter()
        .map(|node| vec![None; node.inputs.len()])
        .collect();
    let mut sinks: Vec<Vec<Option<f32>>> = def
        .output_sinks()
        .iter()
        .map(|sink| vec![None; sink.inputs.len()])
        .collect();
    let EdgeSurface::SinkInput(sink) = first.surface else {
        panic!("a member is a sink edge")
    };
    sinks[sink][first.index] = Some(9.0);
    def.seed_birth(BirthSeeds::Rows {
        compute,
        sinks,
        bundles: vec![None],
    });
    (graph_node(refs(), def), first)
}

proptest::proptest! {
    /// T25.F08 invariant 3: `DetachGraphEdge` draws one member uniformly in
    /// canonical order and nothing else, then detaches it as the F03
    /// primitive does: own weight `compose(own, scale, pattern)` bitwise, no
    /// membership, plasticity kept; the Lamarckian member's seed composes
    /// with the bundle's scale, a static unseeded member carries none.
    #[test]
    fn detach_draws_one_member_and_keeps_its_composed_weight_plasticity_and_seed(
        seed in proptest::prelude::any::<u64>(),
    ) {
        let (original, learner) = detach_fixture();
        let mut node = original.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        bundle::detach_member(&mut node, &mut rng).unwrap();
        let mut expected = SmallRng::seed_from_u64(seed);
        let def = graph(&original);
        let members: Vec<_> = def.members(id(0)).collect();
        let picked = members[expected.gen_range(0..members.len())];
        proptest::prop_assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "one draw");

        let mut want = def.clone();
        let (at, removed, _) = detach(&mut want, picked, original.input_refs());
        proptest::prop_assert!(!removed);
        proptest::prop_assert_eq!(graph(&node), &want);
        let pattern = def.pattern(picked, original.input_refs());
        let before = def.edge_at(picked);
        let after = graph(&node).edge_at(at);
        proptest::prop_assert_eq!(
            after.own_weight().to_bits(),
            compose(before.own_weight(), 0.5, pattern).to_bits()
        );
        proptest::prop_assert_eq!(after.membership(), None);
        proptest::prop_assert_eq!(after.stored_learning(), before.stored_learning());
        let seed_value = graph(&node).birth_seeds().unwrap().value(at);
        if picked == learner {
            proptest::prop_assert_eq!(seed_value, Some(compose(9.0, 0.5, pattern)));
        } else {
            proptest::prop_assert_eq!(seed_value, None);
        }
        proptest::prop_assert_eq!(graph(&node).members(id(0)).count(), 7);
        proptest::prop_assert!(validate_graph(&node).is_ok());
    }
}

/// `DetachGraphEdge` applies exactly where a member exists: not on a graph
/// without a bundle, on one with a bundle, and not once the bundles are gone;
/// it skips without a draw where none is.
#[test]
fn applicability_follows_members_for_detach() {
    let genome = |def| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(refs(), def)],
    };
    let op = GraphOperator::DetachGraphEdge;
    let plain = genome(CgpGraphBackendDef::new_with_fixed_outputs());
    assert!(
        GraphMutator::applicable_indices(&plain, op, &crate::config::test_sensor_context(1))
            .is_empty()
    );
    let grouped = genome(three());
    assert_eq!(
        GraphMutator::applicable_indices(&grouped, op, &crate::config::test_sensor_context(1)),
        vec![0]
    );
    let mut emptied = three();
    while !emptied.bundles().is_empty() {
        emptied.remove_bundle(id(0));
    }
    assert!(GraphMutator::applicable_indices(
        &genome(emptied),
        op,
        &crate::config::test_sensor_context(1)
    )
    .is_empty());

    let mut node = graph_node(refs(), CgpGraphBackendDef::new_with_fixed_outputs());
    let before = node.clone();
    let mut rng = SmallRng::seed_from_u64(5);
    let mut replay = rng.clone();
    assert_eq!(
        bundle::detach_member(&mut node, &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(node, before);
    assert_eq!(rng.gen::<u64>(), replay.gen::<u64>(), "no draw");
}

/// T25.F08 invariant 4: `AddGraphEdge` on a node whose sinks hold bundles
/// pushes an ordinary edge, never a member: every bundle keeps its record
/// and its member set.
#[test]
fn add_graph_edge_never_joins_a_bundle() {
    let mut onto_a_bundle_sink = false;
    for seed in 0..256 {
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![graph_node(refs(), three())],
        };
        let before = graph(&genome.nodes[0]).clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let config = MutationConfig::default();
        let op = GraphOperator::AddGraphEdge;
        if GraphMutator::apply_to_node(
            &mut genome,
            op,
            0,
            &mut rng,
            &config,
            &crate::config::test_sensor_context(2),
        )
        .is_err()
        {
            continue;
        }
        let after = graph(&genome.nodes[0]);
        assert_eq!(after.bundles(), before.bundles(), "seed {seed}");
        for index in 0..before.bundles().len() {
            let members = |def: &CgpGraphBackendDef| -> Vec<_> {
                def.members(id(index)).map(|at| *def.edge_at(at)).collect()
            };
            assert_eq!(members(after), members(&before), "seed {seed}");
        }
        let ordinary = |def: &CgpGraphBackendDef| {
            def.edges()
                .filter(|edge| edge.membership().is_none())
                .count()
        };
        assert_eq!(ordinary(after), ordinary(&before) + 1, "seed {seed}");
        onto_a_bundle_sink |=
            after
                .output_sinks()
                .iter()
                .zip(before.output_sinks())
                .any(|(new, old)| {
                    new.inputs.len() > old.inputs.len()
                        && old.inputs.iter().any(|edge| edge.membership().is_some())
                });
    }
    assert!(
        onto_a_bundle_sink,
        "some push lands on a sink holding a bundle"
    );
}

/// `RetargetBundle` on a pair bundle moves its fact members to the new
/// kind's drive, and skips, after its draws, when that kind lacks a site
/// the wiring needs; a ring bundle needs no drive and retargets regardless
/// (T25.F11 invariant 7).
#[test]
fn retarget_skips_after_its_draws_when_the_kind_lacks_a_site() {
    let refs = vec![area_food(0), barrier()];
    let layout = crate::contracts::pair_layout(&refs[0], 1);
    let gene = |pair: bool| {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        if pair {
            def.add_pair_bundle(0, 1, layout, DirectedVoteKind::Move, 2);
        } else {
            def.add_ring_bundle(1, DirectedVoteKind::Move, 2);
        }
        // Reproduce loses its drive sink.
        let drive = OutputSinkKind::ActionVote(DirectedVoteKind::Reproduce.drive());
        edit_sinks(&mut def, |sinks| sinks.retain(|sink| sink.kind != drive));
        def
    };
    let (mut moved, mut skipped) = (false, false);
    for seed in 0..32 {
        let mut def = gene(true);
        let before = def.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        expected.gen_range(0..1);
        let target = [
            DirectedVoteKind::Reproduce,
            DirectedVoteKind::StealEnergy,
            DirectedVoteKind::GiveEnergy,
        ][expected.gen_range(0..3)];
        let mut ring = gene(false);
        assert_eq!(
            bundle::retarget(&mut ring, &mut SmallRng::seed_from_u64(seed)).map(|_| ()),
            Ok(())
        );
        let result = bundle::retarget(&mut def, &mut rng);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
        if target == DirectedVoteKind::Reproduce {
            assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
            skipped = true;
            continue;
        }
        assert_eq!(result.map(|_| ()), Ok(()));
        moved = true;
        assert_ne!(def, before);
        let drive = def
            .sink_surface(OutputSinkKind::ActionVote(target.drive()))
            .unwrap();
        let facts = def
            .members(id(0))
            .filter(|&member| member.surface == drive)
            .count();
        assert_eq!(facts, layout.facts.len());
        assert!(validate_graph(&graph_node(refs.clone(), def)).is_ok());
    }
    assert!(moved && skipped);
}

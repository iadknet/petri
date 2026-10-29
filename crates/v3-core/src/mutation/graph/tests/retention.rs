//! T25.F09 retention operators over every stored learning (T25.F12): the
//! catalog, applicability, draw orders, the value laws and the reach.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::MutationConfig;
use crate::contracts::votes::DirectedVoteKind;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::genome::cgp::test_support::store_learning;
use crate::genome::cgp::{
    validate_graph, BundleId, CgpGraphBackendDef, EdgeOccurrence, GraphEdge, GraphSource,
    LearningSlot,
};
use crate::genome::{BackendDef, HebbianRule, Learning, NodeGenome};
use crate::mutation::event::{Broadcast, ComplexityEffect};
use crate::mutation::graph::hebbian::{
    disable_retention_in_def, draw_retention, enable_retention_in_def, mutate_retention_in_def,
    step_retention, RETENTION_DEFICIT_MIN,
};
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::MutationOperator;

const RETENTION_OPERATORS: [GraphOperator; 3] = [
    GraphOperator::EnableRetention,
    GraphOperator::DisableRetention,
    GraphOperator::MutateRetention,
];

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn plastic(retention: f32) -> Learning {
    Learning {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian: false,
        modulation: None,
        retention,
    }
}

fn refs() -> Vec<InputReference> {
    vec![InputReference::World(WorldInputKey::NeighborBarrierRing)]
}

/// A full catalog with a lone learning sink edge (retention `ungrouped`)
/// and a ring bundle whose first three members are `Own` members of
/// retentions `members` (the rest `Static`), both slots absent.
fn fixture(ungrouped: f32, members: [f32; 3]) -> (CgpGraphBackendDef, Vec<EdgeOccurrence>) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_ring_bundle(0, DirectedVoteKind::Move, 0);
    let all: Vec<EdgeOccurrence> = def.members(id).collect();
    for (&member, &retention) in all.iter().zip(&members) {
        store_learning(&mut def, member, Some(plastic(retention)));
    }
    let surface = def.sink_surface(crate::genome::cgp::OutputSinkKind::CustomOutput(0));
    def.push_edge(
        surface,
        GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            0.5,
        ),
    );
    let at = EdgeOccurrence::new(surface, def.surface_edges(surface).len() - 1);
    def.edge_mut(at).set_learning(Some(plastic(ungrouped)));
    let mut targets = vec![at];
    targets.extend(all);
    (def, targets)
}

fn node(def: CgpGraphBackendDef) -> NodeGenome {
    NodeGenome::new(NodeId::new(0), refs(), BackendDef::Graph(def), vec![])
}

fn retention_at(def: &CgpGraphBackendDef, at: EdgeOccurrence) -> Option<f32> {
    def.edge_at(at)
        .stored_learning()
        .map(|config| config.retention)
}

/// The edges `after` changed from `before`.
fn edited(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> Vec<EdgeOccurrence> {
    before
        .edge_sites()
        .filter(|&at| before.edge_at(at) != after.edge_at(at))
        .collect()
}

/// The pick and, for a member, the coin, over the edges `matches` admits.
fn replay_pick(
    def: &CgpGraphBackendDef,
    matches: impl Fn(Option<Learning>) -> bool,
    rng: &mut SmallRng,
) -> (EdgeOccurrence, Option<bool>) {
    let matching: Vec<EdgeOccurrence> = def
        .edge_sites()
        .filter(|&at| matches(def.edge_at(at).stored_learning()))
        .collect();
    let picked = matching[rng.gen_range(0..matching.len())];
    let coin = def.edge_at(picked).membership().map(|_| rng.gen_bool(0.5));
    (picked, coin)
}

#[test]
fn the_three_retention_operators_keep_their_slots_keys_and_tiers() {
    assert_eq!(&GraphOperator::ALL[30..33], &RETENTION_OPERATORS);
    let keys = RETENTION_OPERATORS.map(|op| MutationOperator::from(op).as_key());
    assert_eq!(
        keys,
        [
            "Graph.EnableRetention",
            "Graph.DisableRetention",
            "Graph.MutateRetention",
        ]
    );
    assert_eq!(RETENTION_OPERATORS.map(GraphOperator::weight), [1, 1, 4]);
    for op in RETENTION_OPERATORS {
        assert_eq!(op.complexity_effect(), ComplexityEffect::Neutral, "{op:?}");
        assert_eq!(
            MutationOperator::from(op).complexity_effect(),
            ComplexityEffect::Neutral
        );
        // Restricted size pressure admits only decreasing operators.
        assert!(!op.complexity_effect().is_decreasing(), "{op:?}");
    }
}

/// Each operator applies exactly where its predicate admits a stored
/// learning at or below 1: a lone edge, an `Own` member or a present slot.
#[test]
fn the_retention_operators_apply_exactly_where_their_predicate_admits() {
    let applies = |def: &CgpGraphBackendDef| -> Vec<bool> {
        let genome = crate::genome::CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![node(def.clone())],
        };
        RETENTION_OPERATORS
            .iter()
            .map(|&op| {
                !GraphMutator::applicable_indices(
                    &genome,
                    op,
                    &crate::config::test_sensor_context(1),
                )
                .is_empty()
            })
            .collect()
    };
    // Nothing decays: enable only.
    let (def, _) = fixture(1.0, [1.0; 3]);
    assert_eq!(applies(&def), [true, false, false]);
    // Every stored learning decays: disable and mutate only.
    let (def, _) = fixture(0.5, [0.5; 3]);
    assert_eq!(applies(&def), [false, true, true]);
    // A decaying slot alone admits disable and mutate.
    let mut slot_only = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = slot_only.add_ring_bundle(0, DirectedVoteKind::Move, 0);
    slot_only
        .bundle_mut(id)
        .set_slot(LearningSlot::MemberDefault, Some(plastic(0.5)));
    assert_eq!(applies(&slot_only), [false, true, true]);
    // A graph without a stored learning: nothing.
    let bare = CgpGraphBackendDef::new_with_fixed_outputs();
    assert_eq!(applies(&bare), [false; 3]);
}

/// `EnableRetention`: the pick, the coin, one `u = gen_range(−4..=0)`, and
/// `ρ = 1 − 10ᵘ` on the picked edge or every admitted member; the reach it
/// reports is the coin's.
#[test]
fn enable_retention_draws_after_the_coin_and_reports_its_reach() {
    let (original, edges) = fixture(1.0, [1.0, 0.5, 1.0]);
    let mut seen = [false; 3];
    for seed in 0..128 {
        let mut expected = rng(seed);
        let (picked, coin) = replay_pick(
            &original,
            |c| c.is_some_and(|c| c.retention == 1.0),
            &mut expected,
        );
        let u = expected.gen_range(-4.0f32..=0.0);
        let retention = 1.0 - 10f32.powf(u);
        let mut def = original.clone();
        let mut actual = rng(seed);
        let reach = enable_retention_in_def(&mut def, &mut actual).unwrap();
        assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
        let want = if coin == Some(true) {
            // Every admitted member: the decaying one and the `Static` ones
            // are not.
            vec![edges[1], edges[3]]
        } else {
            vec![picked]
        };
        assert_eq!(edited(&original, &def), want, "seed {seed}");
        assert!(want
            .iter()
            .all(|&at| retention_at(&def, at) == Some(retention)));
        assert_eq!(
            reach,
            if coin == Some(true) {
                Broadcast::Members
            } else {
                Broadcast::One
            }
        );
        seen[match coin {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        }] = true;
    }
    assert_eq!(seen, [true; 3], "every case drawn");
}

/// `DisableRetention` writes `ρ = 1` and draws no value; nothing else of
/// the config moves.
#[test]
fn disable_retention_returns_retention_to_one_with_no_value_draw() {
    let (original, edges) = fixture(0.25, [0.5, 1.0, 0.75]);
    for seed in 0..64 {
        let mut expected = rng(seed);
        let (picked, coin) = replay_pick(
            &original,
            |c| c.is_some_and(|c| c.retention < 1.0),
            &mut expected,
        );
        let mut def = original.clone();
        let mut actual = rng(seed);
        disable_retention_in_def(&mut def, &mut actual).unwrap();
        assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
        let want = if coin == Some(true) {
            vec![edges[1], edges[3]]
        } else {
            vec![picked]
        };
        assert_eq!(edited(&original, &def), want, "seed {seed}");
        for at in want {
            assert_eq!(def.edge_at(at).stored_learning(), Some(plastic(1.0)));
        }
    }
}

/// `MutateRetention` draws one `u` and steps each target's own deficit.
#[test]
fn mutate_retention_steps_each_targets_own_deficit() {
    let (original, edges) = fixture(0.9, [0.5, 1.0, 0.999]);
    let mut bundle_wide = false;
    for seed in 0..64 {
        let mut expected = rng(seed);
        let (picked, coin) = replay_pick(
            &original,
            |c| c.is_some_and(|c| c.retention < 1.0),
            &mut expected,
        );
        let u = expected.gen_range(-1.0f32..=1.0);
        let mut def = original.clone();
        let mut actual = rng(seed);
        mutate_retention_in_def(&mut def, &mut actual).unwrap();
        assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
        let targets = if coin == Some(true) {
            bundle_wide = true;
            vec![edges[1], edges[3]]
        } else {
            vec![picked]
        };
        for at in targets {
            let before = retention_at(&original, at).unwrap();
            assert_eq!(retention_at(&def, at), Some(step_retention(before, u)));
        }
    }
    assert!(bundle_wide, "a bundle-wide step");
}

/// The step clamps the deficit to `[10⁻⁴, 1]`; a loaded deficit below the
/// floor steps up to it.
#[test]
fn the_retention_step_clamps_the_deficit() {
    assert_eq!(step_retention(0.0, 1.0), 0.0, "d = 1 cannot grow");
    assert_eq!(
        step_retention(1.0 - RETENTION_DEFICIT_MIN, -1.0),
        1.0 - RETENTION_DEFICIT_MIN
    );
    assert_eq!(step_retention(0.999_999, 0.0), 1.0 - RETENTION_DEFICIT_MIN);
    assert_eq!(step_retention(0.5, 1.0), 1.0 - 0.5 * (1.0 + 0.3 * 1.0));
}

/// Through the dispatch: every retention operator reports its reach and
/// every result passes the graph invariants, slots included.
#[test]
fn the_dispatch_reports_the_reach_and_keeps_the_graph_valid() {
    let (mut def, _) = fixture(1.0, [0.5, 1.0, 0.5]);
    let id = BundleId::new(0);
    def.bundle_mut(id)
        .set_slot(LearningSlot::Scale, Some(plastic(0.5)));
    def.bundle_mut(id)
        .set_slot(LearningSlot::MemberDefault, Some(plastic(1.0)));
    let genome = crate::genome::CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(def)],
    };
    for op in RETENTION_OPERATORS {
        for seed in 0..16 {
            let mut changed = genome.clone();
            let report = GraphMutator::apply_to_node(
                &mut changed,
                op,
                0,
                &mut rng(seed),
                &MutationConfig::default(),
                &crate::config::test_sensor_context(1),
            )
            .unwrap();
            assert!(report.broadcast.is_some(), "{op:?}");
            assert_eq!(
                validate_graph(&changed.nodes[0]),
                Ok(()),
                "{op:?} seed {seed}"
            );
        }
    }
}

proptest! {
    /// The enable draw lands the deficit in `[10⁻⁴, 1]`.
    #[test]
    fn the_enable_draw_keeps_the_deficit_in_range(seed in any::<u64>()) {
        let retention = draw_retention(&mut rng(seed));
        prop_assert!((0.0..=1.0 - RETENTION_DEFICIT_MIN).contains(&retention), "{retention}");
    }

    /// The step keeps every admitted retention decaying and in `[0, 1)`
    /// with its deficit in `[10⁻⁴, 1]`.
    #[test]
    fn the_step_keeps_a_decaying_retention_in_range(retention in 0.0f32..1.0, u in -1.0f32..=1.0) {
        let stepped = step_retention(retention, u);
        prop_assert!((0.0..=1.0 - RETENTION_DEFICIT_MIN).contains(&stepped), "{stepped}");
    }
}

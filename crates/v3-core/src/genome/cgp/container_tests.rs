use proptest::prelude::*;

use super::*;
use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::InputReference;
use crate::genome::cgp::test_support::{plastic_node, set_surface_plasticity};
use crate::genome::cgp::{
    compose, detach, BundleId, BundleSource, Membership, Scope, Site, Wiring,
    FIRST_ACTION_VOTE_SINK,
};
use crate::genome::HebbianRule;

/// A new def field is a compile error here until it is placed inside or
/// outside genome identity on purpose (I9).
#[test]
fn def_fields_are_exhaustively_accounted_for() {
    let CgpGraphBackendDef {
        birth_seeds: _,
        compute_nodes: _,
        output_sinks: _,
        bundles: _,
    } = CgpGraphBackendDef::new_with_fixed_outputs();
    let GraphEdge {
        source: _,
        own_weight: _,
        plasticity: _,
        membership: _,
    } = GraphEdge::new(leaf(0), 0.0);
    let Bundle {
        source: _,
        wiring: _,
        scale: _,
        inherit_scale: _,
        retention: _,
    } = Bundle::new(
        RING_SOURCE,
        Wiring::Ring {
            kind: DirectedVoteKind::Move,
            offset: 0,
        },
    );
    let Membership {
        bundle: _,
        scope: _,
    } = Membership {
        bundle: BundleId::new(0),
        scope: Scope::Edge,
    };
}

#[test]
fn a_graph_edge_is_forty_bytes() {
    assert_eq!(std::mem::size_of::<GraphEdge>(), 40);
}

/// `Bundle::new` builds the silent record: scale 0, no `inherit_scale`,
/// retention 1 (T25.F09 invariant 1).
#[test]
fn a_new_bundle_record_is_silent_with_retention_one() {
    let source = BundleSource::Sensor {
        ref_idx: 3,
        field_set: 0,
    };
    let wiring = Wiring::Ring {
        kind: DirectedVoteKind::Move,
        offset: 2,
    };
    let record = Bundle::new(source, wiring);
    assert_eq!(
        (record.scale(), record.inherit_scale(), record.retention()),
        (0.0, false, 1.0)
    );
    assert_eq!((record.source(), record.wiring()), (source, wiring));
}

/// An edge's and a bundle's `retention` are always written and required
/// (T25.F09 invariant 1, `GENOME_FORMAT` 6).
#[test]
fn retention_is_always_written_and_required() {
    let plastic = r#"{"source":{"ComputeNode":0},"own_weight":0.5,"plasticity":{"rule":"Oja","learning_rate":0.1,"weight_clamp":2.0,"lamarckian":true,"retention":0.75}}"#;
    let edge: GraphEdge = serde_json::from_str(plastic).unwrap();
    assert_eq!(edge.plasticity().unwrap().retention, 0.75);
    assert_eq!(serde_json::to_string(&edge).unwrap(), plastic);
    let without = r#"{"source":{"ComputeNode":0},"own_weight":0.5,"plasticity":{"rule":"Oja","learning_rate":0.1,"weight_clamp":2.0,"lamarckian":true}}"#;
    assert!(serde_json::from_str::<GraphEdge>(without).is_err());

    let bundle = r#"{"source":{"Sensor":{"ref_idx":0,"field_set":0}},"wiring":{"Ring":{"kind":"Move","offset":0}},"scale":0.0,"inherit_scale":false,"retention":0.5}"#;
    let record: Bundle = serde_json::from_str(bundle).unwrap();
    assert_eq!(record.retention(), 0.5);
    assert_eq!(serde_json::to_string(&record).unwrap(), bundle);
    let without = r#"{"source":{"Sensor":{"ref_idx":0,"field_set":0}},"wiring":{"Ring":{"kind":"Move","offset":0}},"scale":0.0,"inherit_scale":false}"#;
    assert!(serde_json::from_str::<Bundle>(without).is_err());
}

/// I1 (T25.F09): an edge retention outside `[0, 1]` or non-finite is out of
/// range; both ends are in range.
#[test]
fn plasticity_in_range_bounds_the_retention() {
    let config = PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian: false,
        modulation: None,
        retention: 1.0,
    };
    for retention in [0.0, 0.5, 1.0] {
        assert!(PlasticityConfig {
            retention,
            ..config
        }
        .in_range());
    }
    for retention in [-0.0001, 1.0001, f32::NAN, f32::INFINITY] {
        assert!(!PlasticityConfig {
            retention,
            ..config
        }
        .in_range());
    }
}

/// The edge's plasticity key round-trips and is omitted when `None`
/// (T25.F06, invariant 1).
#[test]
fn edge_plasticity_round_trips_and_is_omitted_when_none() {
    let static_json = r#"{"source":{"ComputeNode":0},"own_weight":0.5}"#;
    let edge: GraphEdge = serde_json::from_str(static_json).unwrap();
    assert!(edge.plasticity().is_none());
    assert_eq!(serde_json::to_string(&edge).unwrap(), static_json);

    let plastic_json = r#"{"source":{"ComputeNode":0},"own_weight":0.5,"plasticity":{"rule":"Oja","learning_rate":0.1,"weight_clamp":2.0,"lamarckian":true,"modulation":{"reward_source":"DamageDelta","trace_decay":0.75},"retention":1.0}}"#;
    let edge: GraphEdge = serde_json::from_str(plastic_json).unwrap();
    let plasticity = edge.plasticity().expect("the key is read");
    assert_eq!(plasticity.rule, HebbianRule::Oja);
    assert!(plasticity.lamarckian);
    assert_eq!(serde_json::to_string(&edge).unwrap(), plastic_json);
}

/// A member's `scope` and a bundle's `inherit_scale` are always written and
/// required: a membership or a bundle record without them is refused at the
/// load boundary (T25.F07 invariant 1, `GENOME_FORMAT` 5).
#[test]
fn scope_and_inherit_scale_are_always_written_and_required() {
    let member = r#"{"source":{"InputLeaf":{"ref_idx":0,"sub_idx":1}},"own_weight":0.0,"membership":{"bundle":0,"scope":"Group"}}"#;
    let edge: GraphEdge = serde_json::from_str(member).unwrap();
    assert_eq!(
        edge.membership(),
        Some(Membership {
            bundle: BundleId::new(0),
            scope: Scope::Group,
        })
    );
    assert_eq!(serde_json::to_string(&edge).unwrap(), member);
    let without_scope = r#"{"source":{"InputLeaf":{"ref_idx":0,"sub_idx":1}},"own_weight":0.0,"membership":{"bundle":0}}"#;
    assert!(serde_json::from_str::<GraphEdge>(without_scope).is_err());

    let bundle = r#"{"source":{"Sensor":{"ref_idx":0,"field_set":0}},"wiring":{"Ring":{"kind":"Move","offset":0}},"scale":0.0,"inherit_scale":true,"retention":1.0}"#;
    let record: Bundle = serde_json::from_str(bundle).unwrap();
    assert!(record.inherit_scale());
    assert_eq!(serde_json::to_string(&record).unwrap(), bundle);
    let without_flag = r#"{"source":{"Sensor":{"ref_idx":0,"field_set":0}},"wiring":{"Ring":{"kind":"Move","offset":0}},"scale":0.0,"retention":1.0}"#;
    assert!(serde_json::from_str::<Bundle>(without_flag).is_err());
}

/// Bundle identity is the `BundleId` alone (R5): a member at scope `Bundle`
/// is still a member of its bundle.
#[test]
fn is_member_of_ignores_scope() {
    let mut edge = GraphEdge::new(leaf(0), 0.0);
    edge.membership = Some(Membership {
        bundle: BundleId::new(2),
        scope: Scope::Group,
    });
    assert!(edge.is_member_of(BundleId::new(2)));
    assert!(!edge.is_member_of(BundleId::new(1)));
}

/// A scope's transport key is the genome JSON spelling of that scope, and
/// `index` is its position in `ALL`, so a telemetry series and a saved
/// genome name the same scope.
#[test]
fn a_scope_key_matches_its_genome_json_spelling_and_index() {
    assert_eq!(Scope::Edge.as_key(), "Edge");
    assert_eq!(Scope::Group.as_key(), "Group");
    for (position, scope) in Scope::ALL.into_iter().enumerate() {
        assert_eq!(scope.index(), position);
        assert_eq!(
            serde_json::to_string(&scope).unwrap(),
            format!("\"{}\"", scope.as_key())
        );
    }
}

#[test]
fn genome_json_uses_own_weight_and_rejects_unknown_keys() {
    let edge: GraphEdge =
        serde_json::from_str(r#"{"source":{"ComputeNode":0},"own_weight":0.5}"#).unwrap();
    assert_eq!(edge, GraphEdge::new(GraphSource::ComputeNode(0), 0.5));
    assert!(edge.membership().is_none());
    assert!(
        serde_json::from_str::<GraphEdge>(r#"{"source":{"ComputeNode":0},"weight":0.5}"#).is_err()
    );
    assert!(serde_json::from_str::<GraphEdge>(
        r#"{"source":{"ComputeNode":0},"own_weight":0.5,"extra":1}"#
    )
    .is_err());
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    let mut json = serde_json::to_value(&def).unwrap();
    json["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<CgpGraphBackendDef>(json).is_err());
    let mut json = serde_json::to_value(&def).unwrap();
    json["output_sinks"][0]["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<CgpGraphBackendDef>(json).is_err());
}

#[test]
fn compose_is_scale_times_pattern_plus_own() {
    assert_eq!(compose(0.25, 0.5, 1.0), 0.75);
    assert_eq!(compose(-0.5, 2.0, 0.0), -0.5);
    // Pattern 1 is bitwise an ungrouped edge of weight `scale + own`.
    let (own, scale) = (0.1_f32, 0.2_f32);
    assert_eq!(compose(own, scale, 1.0).to_bits(), (scale + own).to_bits());
}

fn leaf(ref_idx: u16) -> GraphSource {
    GraphSource::InputLeaf {
        ref_idx,
        sub_idx: 0,
    }
}

fn plastic() -> Option<PlasticityConfig> {
    Some(PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 5.0,
        lamarckian: true,
        modulation: None,
        retention: 1.0,
    })
}

/// Two compute nodes (node 1 reads node 0) and one wired sink, seeded with
/// one value per edge.
fn seeded() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![
        plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(leaf(0), 0.5), GraphEdge::new(leaf(1), 0.25)],
            plastic(),
        ),
        plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.5)],
            plastic(),
        ),
    ]);
    def.push_edge(
        EdgeSurface::SinkInput(0),
        GraphEdge::new(GraphSource::ComputeNode(1), 1.0),
    );
    def.seed_compute_rows(vec![vec![Some(1.0), Some(2.0)], vec![Some(3.0)]]);
    def
}

fn rows(def: &CgpGraphBackendDef) -> Vec<Vec<Option<f32>>> {
    def.birth_seeds()
        .expect("seed rows")
        .compute_rows()
        .to_vec()
}

fn compute(node: usize, index: usize) -> EdgeOccurrence {
    EdgeOccurrence::new(EdgeSurface::ComputeInput(node), index)
}

#[test]
fn the_json_and_debug_text_of_a_def_name_own_weight_and_print_bundles() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
        )])
        .with_output_sinks(vec![OutputSink::new(
            OutputSinkKind::CustomOutput(0),
            vec![GraphEdge::new(leaf(1), 1.0)],
        )]);
    assert_eq!(
        serde_json::to_string(&def).unwrap(),
        r#"{"compute_nodes":[{"kind":"Add","inputs":[{"source":{"ComputeNode":0},"own_weight":0.5}]}],"output_sinks":[{"kind":{"CustomOutput":0},"inputs":[{"source":{"InputLeaf":{"ref_idx":1,"sub_idx":0}},"own_weight":1.0}]}]}"#
    );
    assert_eq!(
        format!("{def:?}"),
        "CgpGraphBackendDef { compute_nodes: [ComputeNode { kind: Add, inputs: \
         [GraphEdge { source: ComputeNode(0), own_weight: 0.5, plasticity: None, \
         membership: None }] }], output_sinks: [OutputSink { kind: CustomOutput(0), inputs: \
         [GraphEdge { source: InputLeaf { ref_idx: 1, sub_idx: 0 }, own_weight: 1.0, \
         plasticity: None, membership: None }] }], bundles: [] }"
    );
}

#[test]
fn seeds_stay_out_of_identity_and_travel_with_clones() {
    let def = seeded();
    let mut plain = def.clone();
    assert_eq!(plain.birth_seeds(), def.birth_seeds());
    plain.take_birth_seeds();
    assert_eq!(def, plain);
    assert_eq!(format!("{def:?}"), format!("{plain:?}"));
    assert_eq!(
        serde_json::to_string(&def).unwrap(),
        serde_json::to_string(&plain).unwrap()
    );
    let decoded: CgpGraphBackendDef =
        serde_json::from_str(&serde_json::to_string(&def).unwrap()).unwrap();
    assert!(decoded.birth_seeds().is_none());
}

#[test]
fn a_same_source_set_keeps_the_seed_and_a_changed_one_clears_it() {
    let mut def = seeded();
    def.edge_mut(compute(0, 0)).set_source(leaf(0));
    assert_eq!(rows(&def)[0], vec![Some(1.0), Some(2.0)]);
    def.edge_mut(compute(0, 0)).set_source(leaf(5));
    assert_eq!(rows(&def)[0], vec![None, Some(2.0)]);
    def.edge_mut(compute(0, 1)).set_weight(0.25);
    assert_eq!(rows(&def)[0], vec![None, None]);
    // Sink edges carry no seeds.
    let sink = EdgeOccurrence::new(EdgeSurface::SinkInput(0), 0);
    def.edge_mut(sink).set_weight(2.0);
    assert_eq!(def.edge_at(sink).own_weight(), 2.0);
    assert_eq!(rows(&def)[1], vec![Some(3.0)]);
}

#[test]
fn edge_primitives_keep_rows_aligned_with_their_edges() {
    let mut def = seeded();
    def.push_edge(EdgeSurface::ComputeInput(1), GraphEdge::new(leaf(2), 1.0));
    assert_eq!(rows(&def)[1], vec![Some(3.0), None]);
    def.remove_edge(compute(0, 0));
    assert_eq!(rows(&def)[0], vec![Some(2.0)]);
    def.copy_edges(0, 1);
    assert_eq!(rows(&def)[1], vec![Some(3.0), None, Some(2.0)]);
    assert_eq!(
        def.compute_nodes()[1].inputs[2],
        def.compute_nodes()[0].inputs[0]
    );
    def.push_edge(EdgeSurface::SinkInput(3), GraphEdge::new(leaf(2), 1.0));
    assert_eq!(rows(&def).len(), 2);
}

#[test]
fn node_primitives_apply_the_seed_rules_of_the_sites_they_replace() {
    // Remove: the reader of the removed node loses its seed.
    let mut removed = seeded();
    removed.remove_compute_node(0);
    assert_eq!(rows(&removed), vec![vec![None]]);
    assert_eq!(
        removed.compute_nodes()[0].inputs[0].source(),
        GraphSource::ComputeNode(u16::MAX)
    );
    // Insert and append: a row of no values sized to the node.
    let mut inserted = seeded();
    inserted.insert_compute_node(
        1,
        ComputeNode::new(ComputeNodeKind::Add, vec![GraphEdge::new(leaf(3), 1.0)]),
    );
    assert_eq!(rows(&inserted)[1], vec![None]);
    assert_eq!(
        inserted.compute_nodes()[2].inputs[0].source(),
        GraphSource::ComputeNode(0)
    );
    inserted.append_compute_node(ComputeNode::new(ComputeNodeKind::Add, Vec::new()));
    assert_eq!(rows(&inserted).len(), 4);
    // Duplicate: the copy's row is its original's.
    let mut duplicated = seeded();
    duplicated.duplicate_compute_nodes(&[0]);
    assert_eq!(rows(&duplicated)[1], rows(&duplicated)[0]);
    // Kind and plasticity writes leave the seeds alone.
    let mut rewritten = seeded();
    rewritten.set_compute_kind(0, ComputeNodeKind::Tanh);
    set_surface_plasticity(&mut rewritten, EdgeSurface::ComputeInput(1), None);
    assert_eq!(rows(&rewritten), rows(&seeded()));
}

#[test]
fn a_split_keeps_the_consumers_seed_and_starts_the_identity_edge_without_one() {
    let identity = || ComputeNode::new(ComputeNodeKind::Add, Vec::new());
    // Compute consumer: the identity node is inserted at the consumer's index.
    let mut def = seeded();
    def.split_edge(compute(1, 0), identity());
    assert_eq!(
        rows(&def),
        vec![vec![Some(1.0), Some(2.0)], vec![None], vec![Some(3.0)]]
    );
    assert_eq!(
        def.compute_nodes()[1].inputs[0],
        GraphEdge::new(GraphSource::ComputeNode(0), 1.0)
    );
    // The consumer keeps its weight and learning attributes; the identity
    // edge is static (T25.F06 invariant 10).
    let consumer = def.compute_nodes()[2].inputs[0];
    assert_eq!(
        (
            consumer.source(),
            consumer.own_weight(),
            consumer.plasticity()
        ),
        (GraphSource::ComputeNode(1), 1.5, plastic())
    );
    // Sink consumer: the identity node is appended.
    let mut def = seeded();
    def.split_edge(
        EdgeOccurrence::new(EdgeSurface::SinkInput(0), 0),
        identity(),
    );
    assert_eq!(rows(&def)[2], vec![None]);
    assert_eq!(
        def.output_sinks()[0].inputs[0].source(),
        GraphSource::ComputeNode(2)
    );
}

#[test]
fn redirected_leaf_reads_change_no_seed() {
    let mut def = seeded();
    def.redirect_leaf_reads(|ref_idx, _| (ref_idx == 1).then_some(u16::MAX));
    assert_eq!(def.compute_nodes()[0].inputs[1].source(), leaf(u16::MAX));
    assert_eq!(def.compute_nodes()[0].inputs[0].source(), leaf(0));
    assert_eq!(rows(&def), rows(&seeded()));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "one seed row per compute node")]
fn misaligned_seeds_are_a_debug_assertion() {
    let mut def = seeded();
    def.seed_compute_rows(vec![vec![Some(1.0), Some(2.0)]]);
}

fn source_strategy() -> impl Strategy<Value = GraphSource> {
    prop_oneof![
        (0u16..8, 0u16..8)
            .prop_map(|(ref_idx, sub_idx)| GraphSource::InputLeaf { ref_idx, sub_idx }),
        (0u8..16, any::<bool>())
            .prop_map(|(slot, previous)| GraphSource::SharedMemory { slot, previous }),
        (0u16..6).prop_map(GraphSource::ComputeNode),
    ]
}

fn edges_strategy() -> impl Strategy<Value = Vec<GraphEdge>> {
    prop::collection::vec(
        (source_strategy(), -4.0f32..4.0)
            .prop_map(|(source, weight)| GraphEdge::new(source, weight)),
        0..4,
    )
}

fn def_strategy() -> impl Strategy<Value = CgpGraphBackendDef> {
    (
        prop::collection::vec((edges_strategy(), any::<bool>()), 0..5),
        prop::collection::vec((0u8..24, edges_strategy()), 0..4),
    )
        .prop_map(|(nodes, sinks)| {
            CgpGraphBackendDef::new_with_fixed_outputs()
                .with_compute_nodes(
                    nodes
                        .into_iter()
                        .map(|(inputs, learns)| {
                            plastic_node(
                                ComputeNodeKind::Add,
                                inputs,
                                learns.then(plastic).flatten(),
                            )
                        })
                        .collect(),
                )
                .with_output_sinks(
                    sinks
                        .into_iter()
                        .map(|(slot, inputs)| {
                            OutputSink::new(OutputSinkKind::CustomOutput(slot), inputs)
                        })
                        .collect(),
                )
        })
}

proptest! {
    /// JSON round-trips a def to an equal def with the same `Debug` text,
    /// and seeds never reach the JSON.
    #[test]
    fn serde_round_trip_preserves_identity_and_drops_seeds(def in def_strategy()) {
        let mut seeded = def.clone();
        seeded.seed_compute_rows(
            def.compute_nodes().iter().map(|node| vec![Some(1.0); node.inputs.len()]).collect(),
        );
        let json = serde_json::to_string(&seeded).unwrap();
        prop_assert_eq!(&json, &serde_json::to_string(&def).unwrap());
        let decoded: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded, &def);
        prop_assert_eq!(format!("{decoded:?}"), format!("{def:?}"));
        prop_assert!(decoded.birth_seeds().is_none());
    }

    /// The canonical enumeration visits every edge once, compute inputs in
    /// node order then sink inputs in catalog order, matching `edges()`.
    #[test]
    fn edge_sites_enumerate_every_edge_in_canonical_order(def in def_strategy()) {
        let sites: Vec<GraphEdge> = def.edge_sites().map(|site| *def.edge_at(site)).collect();
        let edges: Vec<GraphEdge> = def.edges().copied().collect();
        prop_assert_eq!(sites, edges);
        let compute_edges: usize = def.compute_nodes().iter().map(|node| node.inputs.len()).sum();
        prop_assert!(def
            .edge_sites()
            .take(compute_edges)
            .all(|site| matches!(site.surface, EdgeSurface::ComputeInput(_))));
    }
}

// ── Edge bundles (T25.F03) ───────────────────────────────────────────────────

const RING: u16 = 1;

fn ring_refs() -> Vec<InputReference> {
    vec![
        InputReference::ActionVotes,
        InputReference::World(crate::contracts::WorldInputKey::NeighborBarrierRing),
    ]
}

/// The ring sensor's one field set, the source of every ring fixture.
const RING_SOURCE: BundleSource = BundleSource::Sensor {
    ref_idx: RING,
    field_set: 0,
};

/// A `Ring` wiring into `kind` at `offset`.
fn ring_wiring(kind: DirectedVoteKind, offset: u8) -> Wiring {
    Wiring::Ring { kind, offset }
}

fn vote_sink(kind: DirectedVoteKind, d: u8) -> usize {
    FIRST_ACTION_VOTE_SINK + kind.sink(d).index()
}

fn sink_occurrence(sink: usize, index: usize) -> EdgeOccurrence {
    EdgeOccurrence::new(EdgeSurface::SinkInput(sink), index)
}

/// A full catalog with an independent edge on `Move(0)` and a bundle.
fn grouped(kind: DirectedVoteKind, offset: u8) -> (CgpGraphBackendDef, BundleId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.push_edge(
        EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 0)),
        GraphEdge::new(leaf(0), 0.5),
    );
    let id = def.add_ring_bundle(RING, kind, offset);
    (def, id)
}

fn bundle_strategy() -> impl Strategy<Value = (usize, u8, f32, Vec<f32>)> {
    (
        0usize..3,
        0u8..8,
        -4.0f32..4.0,
        prop::collection::vec(-4.0f32..4.0, 8),
    )
}

/// Set a bundle's scale and each member's correction (in member order).
fn weigh(def: &mut CgpGraphBackendDef, id: BundleId, scale: f32, corrections: &[f32]) {
    def.bundle_mut(id).set_scale(scale);
    let members: Vec<_> = def.members(id).collect();
    for (occurrence, &own) in members.into_iter().zip(corrections) {
        def.edge_mut(occurrence).set_weight(own);
    }
}

#[test]
fn a_new_ring_bundle_is_eight_silent_members_on_its_rotated_sinks() {
    let (def, id) = grouped(DirectedVoteKind::Reproduce, 3);
    assert_eq!(def.bundles().len(), 1);
    assert_eq!(def.bundles()[0].scale(), 0.0);
    let members: Vec<_> = def.members(id).collect();
    assert_eq!(members.len(), 8);
    for occurrence in members {
        let edge = def.edge_at(occurrence);
        assert_eq!(edge.own_weight(), 0.0);
        let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source() else {
            panic!("a ring member reads a leaf");
        };
        assert_eq!(ref_idx, RING);
        let d = ((sub_idx + 3) % 8) as u8;
        assert_eq!(
            occurrence.surface,
            EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Reproduce, d))
        );
        assert_eq!(def.genomic_weight(occurrence, &ring_refs()), 0.0);
    }
}

#[test]
fn push_edge_inserts_an_independent_edge_before_the_first_member() {
    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    let sink = vote_sink(DirectedVoteKind::Move, 0);
    def.push_edge(EdgeSurface::SinkInput(sink), GraphEdge::new(leaf(5), 0.25));
    let inputs = &def.output_sinks()[sink].inputs;
    assert_eq!(inputs.len(), 3);
    assert_eq!(inputs[0].source(), leaf(0));
    assert_eq!(inputs[1].source(), leaf(5));
    assert_eq!(
        inputs[2].membership(),
        Some(Membership {
            bundle: id,
            scope: Scope::Edge
        })
    );
}

#[test]
fn remove_bundle_leaves_no_member_and_renumbers_higher_bundles() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let first = def.add_ring_bundle(RING, DirectedVoteKind::Move, 0);
    let second = def.add_ring_bundle(RING, DirectedVoteKind::StealEnergy, 2);
    let third = def.add_ring_bundle(RING, DirectedVoteKind::Move, 5);
    assert_eq!(def.remove_bundle(second), 8);
    assert_eq!(def.bundles().len(), 2);
    assert!(def.edges().all(|edge| edge.membership()
        != Some(Membership {
            bundle: third,
            scope: Scope::Edge
        })));
    // The third bundle is now `BundleId(1)`, its members with it.
    let renumbered = BundleId::new(1);
    assert_eq!(
        def.bundles()[1]
            .wiring()
            .offset()
            .expect("a geometric bundle"),
        5
    );
    assert_eq!(def.members(renumbered).count(), 8);
    assert_eq!(def.members(first).count(), 8);
    assert!(def.check_local_invariants().is_ok());
}

#[test]
fn a_last_member_removed_or_detached_removes_its_bundle() {
    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    for _ in 0..7 {
        let occurrence = def.members(id).next().unwrap();
        def.remove_edge(occurrence);
        assert_eq!(def.bundles().len(), 1);
    }
    let last = def.members(id).next().unwrap();
    let (_, removed) = detach(&mut def, last, &ring_refs());
    assert!(removed);
    assert!(def.bundles().is_empty());
    assert!(def.edges().all(|edge| edge.membership().is_none()));

    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    for _ in 0..7 {
        let occurrence = def.members(id).next().unwrap();
        def.remove_edge(occurrence);
    }
    let last = def.members(id).next().unwrap();
    def.remove_edge(last);
    assert!(def.bundles().is_empty());
}

#[test]
fn detaching_a_later_bundles_member_ahead_of_another_member_keeps_the_edit_on_the_detached_edge() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let first = def.add_ring_bundle(RING, DirectedVoteKind::Move, 0);
    let second = def.add_ring_bundle(RING, DirectedVoteKind::Move, 0);
    weigh(&mut def, second, 0.5, &[0.25; 8]);
    let sink = vote_sink(DirectedVoteKind::Move, 0);
    // The sink holds the first bundle's member, then the second's.
    let later = sink_occurrence(sink, 1);
    assert_eq!(
        def.edge_at(later).membership(),
        Some(Membership {
            bundle: second,
            scope: Scope::Edge
        })
    );
    let (moved, removed) = detach(&mut def, later, &ring_refs());
    assert!(!removed);
    assert_eq!(moved, sink_occurrence(sink, 0));
    def.edge_mut(moved).set_source(leaf(9));
    let edge = def.edge_at(moved);
    assert_eq!(
        (edge.source(), edge.own_weight(), edge.membership()),
        (leaf(9), 0.75, None)
    );
    assert_eq!(
        def.edge_at(sink_occurrence(sink, 1)).membership(),
        Some(Membership {
            bundle: first,
            scope: Scope::Edge
        })
    );
    assert_eq!(def.members(second).count(), 7);
}

/// `plans` admits a geometric kind only with every sink it needs (T25.F11
/// invariant 5): a ring the eight direction sinks, a pair those and the
/// drive; a `Plain` set one plan per present vote sink.
#[test]
fn plans_need_every_sink_a_wiring_names() {
    use crate::contracts::FieldSet;
    let full = CgpGraphBackendDef::new_with_fixed_outputs();
    let plain = FieldSet::Plain { width: 3 };
    let pair = FieldSet::Pair(pair_layout());
    assert_eq!(plans(&full, &plain).len(), VoteSink::all().count());
    assert_eq!(plans(&full, &FieldSet::Ring).len(), 4 * 8);
    assert_eq!(plans(&full, &pair).len(), 4 * 8);
    let without = |vote: VoteSink| {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        crate::genome::cgp::test_support::edit_sinks(&mut def, |sinks| {
            sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(vote));
        });
        def
    };
    let steal = DirectedVoteKind::StealEnergy;
    let partial = without(VoteSink::Directed(steal, 7));
    for set in [FieldSet::Ring, pair] {
        let kinds: Vec<_> = plans(&partial, &set)
            .iter()
            .filter_map(|w| w.kind())
            .collect();
        assert_eq!(kinds.len(), 3 * 8, "{set:?}");
        assert!(!kinds.contains(&steal));
    }
    assert_eq!(plans(&partial, &plain).len(), VoteSink::all().count() - 1);
    let driveless = without(steal.drive());
    assert_eq!(plans(&driveless, &FieldSet::Ring).len(), 4 * 8);
    assert_eq!(plans(&driveless, &pair).len(), 3 * 8);
    // Kinds in `ALL` order, offsets ascending.
    assert_eq!(
        plans(&full, &FieldSet::Ring)[..9],
        (0..8)
            .map(|offset| ring_wiring(DirectedVoteKind::Move, offset))
            .chain([ring_wiring(DirectedVoteKind::Reproduce, 0)])
            .collect::<Vec<_>>()[..]
    );
}

/// `add_bundle` refuses an inadmissible wiring and a full table, writing
/// nothing (T25.F11 invariant 5).
#[test]
fn add_bundle_refuses_an_inadmissible_wiring_or_a_full_table() {
    use crate::contracts::FieldSet;
    use crate::genome::cgp::BundleBirth;
    let birth = |wiring| BundleBirth {
        field_set: 0,
        wiring,
        scale: 0.0,
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let before = def.clone();
    let plain = Wiring::Plain {
        sink: VoteSink::Eat,
    };
    assert_eq!(
        def.add_bundle(RING_SOURCE, FieldSet::Ring, birth(plain)),
        Err(NodeEditError::Inadmissible)
    );
    assert_eq!(
        def.add_bundle(
            RING_SOURCE,
            FieldSet::Ring,
            birth(ring_wiring(DirectedVoteKind::Move, 8))
        ),
        Err(NodeEditError::Inadmissible)
    );
    assert_eq!(def, before);
    def.exhaust_bundle_capacity();
    let full = def.clone();
    assert_eq!(
        def.add_bundle(
            RING_SOURCE,
            FieldSet::Ring,
            birth(ring_wiring(DirectedVoteKind::Move, 0))
        ),
        Err(NodeEditError::BundleCapacity)
    );
    assert_eq!(def, full);
}

#[test]
fn validate_graph_rejects_each_broken_invariant() {
    let node = |def: CgpGraphBackendDef, input_refs: Vec<InputReference>| NodeGenome {
        node_id: crate::contracts::NodeId(0),
        input_refs,
        backend_def: BackendDef::Graph(def),
        targets: Vec::new(),
    };
    let (def, _) = grouped(DirectedVoteKind::Move, 1);
    assert_eq!(validate_graph(&node(def.clone(), ring_refs())), Ok(()));
    // A sensor source whose set does not resolve to the wiring's geometry.
    assert_eq!(
        validate_graph(&node(def.clone(), vec![InputReference::ActionVotes; 2])),
        Err(GraphInvariantViolation::ShapeReference)
    );
    // I6: a record without members.
    let mut empty = def.clone();
    empty.bundles.push(empty.bundles[0]);
    assert_eq!(
        validate_graph(&node(empty, ring_refs())),
        Err(GraphInvariantViolation::EmptyBundle)
    );
    // I7: offset out of range.
    let mut offset = def.clone();
    offset.bundles[0].wiring = ring_wiring(DirectedVoteKind::Move, 8);
    assert_eq!(
        offset.check_local_invariants(),
        Err(GraphInvariantViolation::OffsetOutOfRange)
    );
    // T25.F09 invariant 1: a bundle retention outside [0, 1] or non-finite;
    // 0 is in range.
    for retention in [-0.5, 1.5, f32::NAN] {
        let mut out = def.clone();
        out.bundles[0].retention = retention;
        assert_eq!(
            out.check_local_invariants(),
            Err(GraphInvariantViolation::BundleRetentionOutOfRange)
        );
    }
    let mut zero = def.clone();
    zero.bundles[0].retention = 0.0;
    assert_eq!(zero.check_local_invariants(), Ok(()));
    // I4: a `Ring` member on a direction its offset does not map its cell
    // to (record and edge only, so the local check).
    let mut shifted = def.clone();
    shifted.bundles[0].wiring = ring_wiring(DirectedVoteKind::Move, 2);
    assert_eq!(
        shifted.check_local_invariants(),
        Err(GraphInvariantViolation::MemberShape)
    );
    // I10: an independent edge after a member.
    let mut order = def.clone();
    let sink = vote_sink(DirectedVoteKind::Move, 1);
    order.output_sinks.0[sink]
        .inputs
        .0
        .push(GraphEdge::new(leaf(3), 1.0));
    assert_eq!(
        order.check_local_invariants(),
        Err(GraphInvariantViolation::SinkOrder)
    );
    // I2: a compute-input member.
    let member = *def.edge_at(def.members(BundleId::new(0)).next().unwrap());
    let compute = def
        .clone()
        .with_compute_nodes(vec![ComputeNode::new(ComputeNodeKind::Add, vec![member])]);
    assert_eq!(
        compute.check_local_invariants(),
        Err(GraphInvariantViolation::MemberOnComputeInput)
    );
    // I3: a membership naming no bundle.
    let mut dangling = def;
    for sink in &mut dangling.output_sinks.0 {
        for edge in &mut sink.inputs.0 {
            if edge.membership.is_some() {
                edge.membership = Some(Membership {
                    bundle: BundleId::new(4),
                    scope: Scope::Edge,
                });
            }
        }
    }
    assert_eq!(
        dangling.check_local_invariants(),
        Err(GraphInvariantViolation::BundleIdOutOfRange)
    );
}

/// `u16::MAX` bundle records are the most a def may hold; one more is
/// `TooManyBundles`. Records without members fail only later, as `EmptyBundle`.
#[test]
fn the_bundle_count_limit_is_u16_max_records() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let record = Bundle::new(RING_SOURCE, ring_wiring(DirectedVoteKind::Move, 0));
    def.bundles = vec![record; usize::from(u16::MAX)];
    assert_eq!(
        def.check_local_invariants(),
        Err(GraphInvariantViolation::EmptyBundle)
    );
    def.bundles.push(record);
    assert_eq!(
        def.check_local_invariants(),
        Err(GraphInvariantViolation::TooManyBundles)
    );
}

/// A member reading ring channel 8 is out of shape even though its
/// direction wraps onto the sink of channel 0.
#[test]
fn a_member_reading_channel_eight_is_out_of_shape() {
    let (mut def, _) = grouped(DirectedVoteKind::Move, 3);
    let edge = def
        .output_sinks
        .0
        .iter_mut()
        .flat_map(|sink| sink.inputs.0.iter_mut())
        .find(|edge| {
            edge.membership.is_some()
                && edge.source
                    == GraphSource::InputLeaf {
                        ref_idx: RING,
                        sub_idx: 0,
                    }
        })
        .expect("the bundle's channel-0 member");
    edge.source = GraphSource::InputLeaf {
        ref_idx: RING,
        sub_idx: 8,
    };
    assert_eq!(
        def.check_local_invariants(),
        Err(GraphInvariantViolation::MemberShape)
    );
}

/// A bundle-touching primitive checks the local invariants of a def with
/// bundles at its exit in debug builds.
#[test]
#[should_panic(expected = "graph invariant violated")]
fn a_primitive_on_a_broken_grouped_def_is_a_debug_assertion() {
    let (mut def, _) = grouped(DirectedVoteKind::Move, 1);
    def.bundles[0].wiring = ring_wiring(DirectedVoteKind::Move, 2);
    def.push_edge(
        EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 0)),
        GraphEdge::new(leaf(0), 0.25),
    );
}

#[test]
fn a_grouped_def_round_trips_json_and_identity_both_ways() {
    let (mut def, id) = grouped(DirectedVoteKind::StealEnergy, 6);
    weigh(&mut def, id, -0.5, &[0.125; 8]);
    let json = serde_json::to_string(&def).unwrap();
    assert!(json.contains("\"bundles\"") && json.contains("\"membership\""));
    let decoded: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, def);
    assert_eq!(format!("{decoded:?}"), format!("{def:?}"));
    // Every heritable field reaches identity: a changed scale, offset,
    // correction or membership is a different def, in `PartialEq`, `Debug`
    // and JSON alike.
    let mut scaled = def.clone();
    scaled.bundle_mut(id).set_scale(0.5);
    let mut moved = def.clone();
    moved.relocate_bundle(id, ring_wiring(DirectedVoteKind::StealEnergy, 5));
    let mut corrected = def.clone();
    let first = corrected.members(id).next().unwrap();
    corrected.edge_mut(first).set_weight(1.0);
    let mut detached = def.clone();
    detach(&mut detached, first, &ring_refs());
    for other in [scaled, moved, corrected, detached] {
        assert_ne!(other, def);
        assert_ne!(format!("{other:?}"), format!("{def:?}"));
        assert_ne!(serde_json::to_string(&other).unwrap(), json);
    }
    // An ungrouped def omits both keys and prints `bundles: []`.
    let plain = CgpGraphBackendDef::new_with_fixed_outputs();
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("bundles") && !json.contains("membership"));
    assert!(format!("{plain:?}").ends_with("bundles: [] }"));
}

proptest! {
    /// Detach rewrites a member as the ungrouped edge of its genomic weight,
    /// bitwise, whatever the scale and correction.
    #[test]
    fn detach_preserves_genomic_weight_bitwise(
        (kind, offset, scale, corrections) in bundle_strategy(),
        pick in 0usize..8,
    ) {
        let (mut def, id) = grouped(DirectedVoteKind::ALL[kind], offset);
        weigh(&mut def, id, scale, &corrections);
        let refs = ring_refs();
        let occurrence = def.members(id).nth(pick).unwrap();
        let before = def.genomic_weight(occurrence, &refs);
        let source = def.edge_at(occurrence).source();
        let (moved, _) = detach(&mut def, occurrence, &refs);
        let edge = def.edge_at(moved);
        prop_assert_eq!(edge.membership(), None);
        prop_assert_eq!(edge.source(), source);
        prop_assert_eq!(edge.own_weight().to_bits(), before.to_bits());
        prop_assert_eq!(def.genomic_weight(moved, &refs).to_bits(), before.to_bits());
        prop_assert!(def.check_local_invariants().is_ok());
    }

    /// A shift or kind retarget followed by its inverse returns an equal def
    /// under I10.
    #[test]
    fn shift_then_unshift_is_identity(
        (kind, offset, scale, corrections) in bundle_strategy(),
        step in 1u8..8,
        other_kind in 0usize..3,
        second in any::<bool>(),
    ) {
        let (mut def, id) = grouped(DirectedVoteKind::ALL[kind], offset);
        if second {
            def.add_ring_bundle(RING, DirectedVoteKind::ALL[other_kind], (offset + 3) % 8);
        }
        weigh(&mut def, id, scale, &corrections);
        let original = def.clone();
        def.relocate_bundle(id, ring_wiring(DirectedVoteKind::ALL[other_kind], (offset + step) % 8));
        prop_assert!(def.check_local_invariants().is_ok());
        def.relocate_bundle(id, ring_wiring(DirectedVoteKind::ALL[kind], offset));
        prop_assert_eq!(&def, &original);
        prop_assert_eq!(format!("{def:?}"), format!("{original:?}"));
    }

    /// A `Ring` member weighs `scale + own` bitwise (pattern 1).
    #[test]
    fn a_ring_member_weighs_scale_plus_correction(
        (kind, offset, scale, corrections) in bundle_strategy(),
    ) {
        let (mut def, id) = grouped(DirectedVoteKind::ALL[kind], offset);
        weigh(&mut def, id, scale, &corrections);
        let members: Vec<_> = def.members(id).collect();
        for occurrence in members {
            let own = def.edge_at(occurrence).own_weight();
            prop_assert_eq!(
                def.genomic_weight(occurrence, &ring_refs()).to_bits(),
                (scale + own).to_bits()
            );
        }
    }
}

// ── Direction-pair bundles (T25.F04, T25.F11) ──────────────────────────────

/// `input_refs[PAIR_REF]` is `AreaFoodSummary`; its pair 1 is
/// `(nearest_dx, nearest_dy)`, fields `(3, 4)`, with the facts `0, 5, 6`.
const PAIR_REF: u16 = 2;
const PX: u16 = 3;
const PY: u16 = 4;

fn pair_refs() -> Vec<InputReference> {
    let mut refs = ring_refs();
    refs.push(InputReference::World(
        crate::contracts::WorldInputKey::area_food_summary(
            crate::contracts::OrdinaryFoodTypeId::new(0),
        ),
    ));
    refs
}

fn pair_layout() -> crate::contracts::DirectionPair {
    crate::contracts::pair_layout(&pair_refs()[usize::from(PAIR_REF)], 1)
}

fn pair_wiring(kind: DirectedVoteKind, offset: u8) -> Wiring {
    Wiring::DirectionPair { kind, offset }
}

fn paired(kind: DirectedVoteKind, offset: u8) -> (CgpGraphBackendDef, BundleId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.push_edge(
        EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 0)),
        GraphEdge::new(leaf(0), 0.5),
    );
    let id = def.add_pair_bundle(PAIR_REF, 1, pair_layout(), kind, offset);
    (def, id)
}

/// A member's `(field, site)`.
fn coordinates_of(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> (u16, Site) {
    let GraphSource::InputLeaf { sub_idx, .. } = def.edge_at(occurrence).source() else {
        unreachable!("members read leaves")
    };
    (
        sub_idx,
        def.site_of(occurrence).expect("a member on its wiring"),
    )
}

fn coordinates(def: &CgpGraphBackendDef, id: BundleId) -> Vec<(u16, Site)> {
    def.members(id).map(|o| coordinates_of(def, o)).collect()
}

fn pair_node(def: CgpGraphBackendDef, input_refs: Vec<InputReference>) -> NodeGenome {
    NodeGenome {
        node_id: crate::contracts::NodeId(0),
        input_refs,
        backend_def: BackendDef::Graph(def),
        targets: Vec::new(),
    }
}

/// Re-sort a hand-edited sink into the I10 order.
fn resort(def: &mut CgpGraphBackendDef, sink: usize) {
    def.output_sinks.0[sink]
        .inputs
        .0
        .sort_by_key(CgpGraphBackendDef::member_key);
}

/// A pair bundle is born with its twelve non-zero compass members and one
/// drive member per fact, every member at own weight 0 and a non-zero
/// pattern (T25.F11 invariant 3).
#[test]
fn pair_creation_materializes_12_compass_members_and_the_facts() {
    for offset in 0..8u8 {
        let (def, id) = paired(DirectedVoteKind::StealEnergy, offset);
        let mut got = coordinates(&def, id);
        got.sort_unstable();
        let rel = |field: u16, relatives: [u8; 6]| {
            relatives.map(|r| (field, Site::Direction((r + offset) % 8)))
        };
        // x at NE, E, SE, SW, W, NW; y at N, NE, SE, S, SW, NW.
        let mut want: Vec<_> = rel(PX, [1, 2, 3, 5, 6, 7])
            .into_iter()
            .chain(rel(PY, [0, 1, 3, 4, 5, 7]))
            .chain(pair_layout().facts.iter().map(|&fact| (fact, Site::Drive)))
            .collect();
        want.sort_unstable();
        assert_eq!(got, want, "offset {offset}");
        assert_eq!(def.bundles()[0].scale(), 0.0);
        for occurrence in def.members(id) {
            assert_eq!(def.edge_at(occurrence).own_weight(), 0.0);
            assert_eq!(def.genomic_weight(occurrence, &pair_refs()), 0.0);
            assert_ne!(def.member_pattern(occurrence, &pair_refs()), 0.0);
        }
        assert_eq!(validate_graph(&pair_node(def, pair_refs())), Ok(()));
    }
}

/// A geometric wiring's coordinates reduce every offset mod 8.
#[test]
fn wiring_coordinates_reduce_every_offset_mod_8() {
    use crate::contracts::FieldSet;
    let pair = FieldSet::Pair(pair_layout());
    for offset in 0..=u8::MAX {
        for kind in DirectedVoteKind::ALL {
            assert_eq!(
                pair_wiring(kind, offset).coordinates(&pair),
                pair_wiring(kind, offset % 8).coordinates(&pair),
                "offset {offset}"
            );
            assert_eq!(
                ring_wiring(kind, offset).coordinates(&FieldSet::Ring),
                ring_wiring(kind, offset % 8).coordinates(&FieldSet::Ring),
                "offset {offset}"
            );
        }
    }
}

/// Every member weighs its wiring's pattern at its coordinate: the compass
/// pattern of its relative direction for `x` and `y`, 1 for a fact on the
/// drive (T25.F11 invariant 3).
#[test]
fn a_pair_member_weighs_the_pattern_at_its_coordinate() {
    let (mut def, id) = paired(DirectedVoteKind::Move, 3);
    def.bundle_mut(id).set_scale(1.0);
    for occurrence in def.members(id).collect::<Vec<_>>() {
        let want = match coordinates_of(&def, occurrence) {
            (field, Site::Direction(d)) => {
                let relative = (d + 8 - 3) % 8;
                if field == PX {
                    crate::contracts::compass_x(relative)
                } else {
                    crate::contracts::compass_y(relative)
                }
            }
            (field, Site::Drive) => {
                assert!(pair_layout().facts.contains(&field));
                1.0
            }
            (_, Site::Sink) => unreachable!("a pair has no Plain site"),
        };
        assert_eq!(
            def.genomic_weight(occurrence, &pair_refs()).to_bits(),
            want.to_bits()
        );
    }
}

/// A wiring has no pattern off its coordinates: a field outside the set, an
/// `x` on the drive, a fact on a direction, an `x` where its compass is 0,
/// a direction past the compass, a site of another wiring (T25.F11
/// invariant 3).
#[test]
fn a_wiring_has_no_pattern_off_its_coordinates() {
    use crate::contracts::FieldSet;
    let set = FieldSet::Pair(pair_layout());
    let wiring = pair_wiring(DirectedVoteKind::Move, 0);
    assert_eq!(wiring.pattern(&set, 1, Site::Direction(2)), None);
    assert_eq!(wiring.pattern(&set, PX, Site::Drive), None);
    assert_eq!(wiring.pattern(&set, 0, Site::Direction(2)), None);
    assert_eq!(wiring.pattern(&set, PX, Site::Direction(0)), None);
    assert_eq!(wiring.pattern(&set, PX, Site::Direction(8)), None);
    assert_eq!(wiring.pattern(&set, PX, Site::Sink), None);
    assert_eq!(wiring.pattern(&set, 0, Site::Drive), Some(1.0));
    assert_eq!(wiring.pattern(&FieldSet::Ring, 0, Site::Direction(0)), None);
    let ring = ring_wiring(DirectedVoteKind::Move, 2);
    assert_eq!(ring.pattern(&FieldSet::Ring, 8, Site::Direction(2)), None);
    assert_eq!(ring.pattern(&FieldSet::Ring, 0, Site::Direction(3)), None);
    assert_eq!(ring.pattern(&FieldSet::Ring, 0, Site::Drive), None);
    assert_eq!(
        ring.pattern(&FieldSet::Ring, 0, Site::Direction(2)),
        Some(1.0)
    );
    let plain = Wiring::Plain {
        sink: VoteSink::Eat,
    };
    let width = FieldSet::Plain { width: 3 };
    assert_eq!(plain.pattern(&width, 2, Site::Sink), Some(1.0));
    assert_eq!(plain.pattern(&width, 3, Site::Sink), None);
    assert_eq!(plain.pattern(&width, 0, Site::Drive), None);
}

#[test]
fn identical_source_members_on_different_sinks_resolve_different_weights() {
    let (mut def, id) = paired(DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(1.0);
    let x_members: Vec<_> = def
        .members(id)
        .filter(|&o| coordinates_of(&def, o).0 == PX)
        .collect();
    assert_eq!(x_members.len(), 6);
    let source = def.edge_at(x_members[0]).source();
    assert!(x_members.iter().all(|&o| def.edge_at(o).source() == source));
    let weights: std::collections::BTreeSet<u32> = x_members
        .iter()
        .map(|&o| def.genomic_weight(o, &pair_refs()).to_bits())
        .collect();
    // +1, +FRAC_1_SQRT_2, −FRAC_1_SQRT_2, −1.
    assert_eq!(weights.len(), 4);
    // The runtime weight table is the same resolver.
    let rows = def.sink_weight_rows(&pair_refs());
    for o in x_members {
        let EdgeSurface::SinkInput(sink) = o.surface else {
            unreachable!()
        };
        assert_eq!(
            rows[sink][o.index].to_bits(),
            def.genomic_weight(o, &pair_refs()).to_bits()
        );
    }
}

#[test]
fn a_pair_bundles_last_member_removed_removes_the_bundle() {
    let (mut def, id) = paired(DirectedVoteKind::Reproduce, 5);
    for remaining in (1..15).rev() {
        let occurrence = def.members(id).next().unwrap();
        def.remove_edge(occurrence);
        assert_eq!(def.members(id).count(), remaining);
        assert_eq!(def.bundles().len(), 1);
    }
    let last = def.members(id).next().unwrap();
    def.remove_edge(last);
    assert!(def.bundles().is_empty());
    assert!(def.edges().all(|edge| edge.membership().is_none()));
}

/// Relocation preserves every member's relative coordinate, holes and the
/// facts on the drive included; a hand-built member where its compass is 0
/// is a `MemberShape` violation (T25.F11 invariants 3–4).
#[test]
fn a_pair_bundle_with_holes_relocates_by_relative_coordinate() {
    let (mut def, id) = paired(DirectedVoteKind::Move, 2);
    // A hole: the x member at relative E (direction 4).
    let hole = def
        .members(id)
        .find(|&o| coordinates_of(&def, o) == (PX, Site::Direction(4)))
        .unwrap();
    def.remove_edge(hole);
    assert_eq!(validate_graph(&pair_node(def.clone(), pair_refs())), Ok(()));
    let relative = |def: &CgpGraphBackendDef| {
        let offset = def.bundles()[0]
            .wiring()
            .offset()
            .expect("a geometric bundle");
        let mut set: Vec<_> = coordinates(def, id)
            .into_iter()
            .map(|(field, site)| match site {
                Site::Direction(d) => (field, Site::Direction((d + 8 - offset) % 8)),
                other => (field, other),
            })
            .collect();
        set.sort_unstable();
        set
    };
    let before = relative(&def);
    assert_eq!(before.len(), 14);
    let original = def.clone();
    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::StealEnergy, 7));
    assert_eq!(relative(&def), before);
    assert_eq!(def.check_local_invariants(), Ok(()));
    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::Move, 2));
    assert_eq!(def, original);

    // x at relative N (direction 2) has compass 0: off the wiring.
    let sink = vote_sink(DirectedVoteKind::Move, 2);
    def.output_sinks.0[sink].inputs.0.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: PAIR_REF,
            sub_idx: PX,
        },
        own_weight: 0.25,
        plasticity: None,
        membership: Some(Membership {
            bundle: id,
            scope: Scope::Edge,
        }),
    });
    resort(&mut def, sink);
    assert_eq!(def.check_local_invariants(), Ok(()));
    assert_eq!(
        validate_graph(&pair_node(def, pair_refs())),
        Err(GraphInvariantViolation::MemberShape)
    );
}

#[test]
fn validate_graph_rejects_a_bad_pair_role_range_or_duplicate() {
    let (def, id) = paired(DirectedVoteKind::Move, 1);
    assert_eq!(validate_graph(&pair_node(def.clone(), pair_refs())), Ok(()));
    // The field set index is out of range for the reference.
    let mut range = def.clone();
    range.bundles[0].source = BundleSource::Sensor {
        ref_idx: PAIR_REF,
        field_set: 2,
    };
    assert_eq!(
        validate_graph(&pair_node(range, pair_refs())),
        Err(GraphInvariantViolation::ShapeReference)
    );
    // The reference declares no pairs.
    let mut no_pairs = pair_refs();
    no_pairs[usize::from(PAIR_REF)] = InputReference::ActionVotes;
    assert_eq!(
        validate_graph(&pair_node(def.clone(), no_pairs)),
        Err(GraphInvariantViolation::ShapeReference)
    );
    // A fact member on a direction sink.
    let first = def.members(id).next().unwrap();
    let EdgeSurface::SinkInput(sink) = first.surface else {
        unreachable!()
    };
    let mut role = def.clone();
    role.output_sinks.0[sink].inputs.0[first.index].source = GraphSource::InputLeaf {
        ref_idx: PAIR_REF,
        sub_idx: 0,
    };
    resort(&mut role, sink);
    assert_eq!(role.check_local_invariants(), Ok(()));
    assert_eq!(
        validate_graph(&pair_node(role, pair_refs())),
        Err(GraphInvariantViolation::MemberShape)
    );
    // A duplicate (field, site).
    let mut duplicate = def.clone();
    let copied = *duplicate.edge_at(first);
    duplicate.output_sinks.0[sink]
        .inputs
        .0
        .insert(first.index, copied);
    assert_eq!(
        duplicate.check_local_invariants(),
        Err(GraphInvariantViolation::DuplicateMember)
    );
    // An equal member key is a duplicate before its source is checked: the
    // repeat reads another reference at the same field.
    let mut repeated = def.clone();
    let mut stray = *repeated.edge_at(first);
    let GraphSource::InputLeaf { sub_idx, .. } = stray.source else {
        unreachable!("a pair member reads an input leaf")
    };
    stray.source = GraphSource::InputLeaf {
        ref_idx: RING,
        sub_idx,
    };
    repeated.output_sinks.0[sink]
        .inputs
        .0
        .insert(first.index + 1, stray);
    assert_eq!(
        repeated.check_local_invariants(),
        Err(GraphInvariantViolation::DuplicateMember)
    );
    // I10: two members of one sink out of key order.
    let mut swapped = def.clone();
    let (two, members) = (0..swapped.output_sinks.0.len())
        .map(|s| {
            let n = swapped.output_sinks.0[s]
                .inputs
                .0
                .iter()
                .filter(|e| e.membership.is_some())
                .count();
            (s, n)
        })
        .find(|&(_, n)| n >= 2)
        .expect("a sink holding both pair fields");
    assert!(members >= 2);
    let inputs = &mut swapped.output_sinks.0[two].inputs.0;
    let at: Vec<usize> = (0..inputs.len())
        .filter(|&i| inputs[i].membership.is_some())
        .collect();
    inputs.swap(at[0], at[1]);
    assert_eq!(
        swapped.check_local_invariants(),
        Err(GraphInvariantViolation::SinkOrder)
    );
    // A member reading another reference.
    let mut other = def;
    other.output_sinks.0[sink].inputs.0[first.index].source = GraphSource::InputLeaf {
        ref_idx: RING,
        sub_idx: PX,
    };
    assert_eq!(
        other.check_local_invariants(),
        Err(GraphInvariantViolation::MemberShape)
    );
}

/// T25.F10: a pair gene adds one unit, its fact members included.
#[test]
fn a_pair_gene_adds_one_unit() {
    let base = CgpGraphBackendDef::new_with_fixed_outputs();
    let genome = |def| crate::genome::CreatureGenome {
        entry_node_id: crate::contracts::NodeId(0),
        nodes: vec![pair_node(def, pair_refs())],
    };
    let mut grouped = base.clone();
    grouped.add_pair_bundle(PAIR_REF, 1, pair_layout(), DirectedVoteKind::Move, 0);
    assert_eq!(
        genome(grouped).genome_size() - genome(base).genome_size(),
        1
    );
}

/// A record's `source` and `wiring` are externally tagged in the genome
/// JSON (T25.F11 invariant 2).
#[test]
fn a_bundle_record_round_trips_json_with_its_source_and_wiring() {
    let (mut def, id) = paired(DirectedVoteKind::StealEnergy, 6);
    def.bundle_mut(id).set_scale(-0.5);
    let json = serde_json::to_string(&def).unwrap();
    assert!(
        json.contains(
            r#""source":{"Sensor":{"ref_idx":2,"field_set":1}},"wiring":{"DirectionPair":{"kind":"StealEnergy","offset":6}}"#
        ),
        "{json}"
    );
    let decoded: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, def);
    assert_eq!(format!("{decoded:?}"), format!("{def:?}"));
    let (ring_def, _) = grouped(DirectedVoteKind::Move, 0);
    assert!(serde_json::to_string(&ring_def)
        .unwrap()
        .contains(r#""wiring":{"Ring":{"kind":"Move","offset":0}}"#));
}

proptest! {
    /// Detach of a pair member, a fact member included, is bitwise
    /// `compose` at its pattern.
    #[test]
    fn detaching_a_pair_member_preserves_its_genomic_weight_bitwise(
        kind in 0usize..3,
        offset in 0u8..8,
        scale in -4.0f32..4.0,
        own in -4.0f32..4.0,
        pick in 0usize..15,
    ) {
        let (mut def, id) = paired(DirectedVoteKind::ALL[kind], offset);
        def.bundle_mut(id).set_scale(scale);
        let occurrence = def.members(id).nth(pick).unwrap();
        def.edge_mut(occurrence).set_weight(own);
        let pattern = def.member_pattern(occurrence, &pair_refs());
        let before = def.genomic_weight(occurrence, &pair_refs());
        prop_assert_eq!(before.to_bits(), compose(own, scale, pattern).to_bits());
        let source = def.edge_at(occurrence).source();
        let (moved, removed) = detach(&mut def, occurrence, &pair_refs());
        prop_assert!(!removed);
        let edge = def.edge_at(moved);
        prop_assert_eq!(edge.membership(), None);
        prop_assert_eq!(edge.source(), source);
        prop_assert_eq!(edge.own_weight().to_bits(), before.to_bits());
        prop_assert_eq!(def.members(id).count(), 14);
    }

    /// A pair bundle's shift or retarget followed by its inverse returns an
    /// equal def, direction and fact holes and a ring bundle beside it
    /// included.
    #[test]
    fn pair_shift_then_unshift_is_identity(
        kind in 0usize..3,
        offset in 0u8..8,
        step in 1u8..8,
        other_kind in 0usize..3,
        holes in any::<u16>(),
    ) {
        let (mut def, id) = paired(DirectedVoteKind::ALL[kind], offset);
        def.add_ring_bundle(RING, DirectedVoteKind::ALL[other_kind], (offset + 3) % 8);
        let members: Vec<_> = def.members(id).collect();
        // Punch holes, keeping the first member.
        for (i, occurrence) in members.into_iter().enumerate().skip(1).rev() {
            if holes & (1 << i) != 0 {
                def.remove_edge(occurrence);
            }
        }
        let original = def.clone();
        def.relocate_bundle(id, pair_wiring(DirectedVoteKind::ALL[other_kind], (offset + step) % 8));
        prop_assert!(def.check_local_invariants().is_ok());
        prop_assert!(validate_graph(&pair_node(def.clone(), pair_refs())).is_ok());
        def.relocate_bundle(id, pair_wiring(DirectedVoteKind::ALL[kind], offset));
        prop_assert_eq!(&def, &original);
    }
}

// ── Learning attributes (T25.F06) ───────────────────────────────────────────

fn learning_node(def: CgpGraphBackendDef, input_refs: Vec<InputReference>) -> NodeGenome {
    NodeGenome {
        node_id: crate::contracts::NodeId(0),
        input_refs,
        backend_def: BackendDef::Graph(def),
        targets: Vec::new(),
    }
}

/// A `ClearSlot` edge never learns (its sum is ignored): it is not
/// eligible, and `validate_graph` rejects a plastic one.
#[test]
fn clear_slot_edges_are_never_eligible() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let clear = def.sink_surface(OutputSinkKind::ClearSlot(3)).unwrap();
    let write = def.sink_surface(OutputSinkKind::WriteSlot(3)).unwrap();
    def.push_edge(clear, GraphEdge::new(leaf(0), 1.0));
    def.push_edge(write, GraphEdge::new(leaf(0), 1.0));
    assert!(!def.can_learn(EdgeOccurrence::new(clear, 0)));
    assert!(def.can_learn(EdgeOccurrence::new(write, 0)));
    assert!(def.can_learn(compute_edge_free_occurrence()));
    def.edge_mut(EdgeOccurrence::new(write, 0))
        .set_plasticity(plastic());
    assert_eq!(
        validate_graph(&learning_node(def.clone(), Vec::new())),
        Ok(())
    );
    let EdgeSurface::SinkInput(clear_index) = clear else {
        unreachable!()
    };
    def.output_sinks.0[clear_index].inputs.0[0].plasticity = plastic();
    assert_eq!(
        validate_graph(&learning_node(def, Vec::new())),
        Err(GraphInvariantViolation::IneligiblePlasticEdge)
    );
}

/// Any compute-input occurrence may learn.
fn compute_edge_free_occurrence() -> EdgeOccurrence {
    compute(0, 0)
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "a ClearSlot edge does not learn")]
fn set_plasticity_on_a_clear_slot_edge_is_a_debug_assertion() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let clear = def.sink_surface(OutputSinkKind::ClearSlot(0)).unwrap();
    def.push_edge(clear, GraphEdge::new(leaf(0), 1.0));
    def.edge_mut(EdgeOccurrence::new(clear, 0))
        .set_plasticity(plastic());
}

/// A member learns at its scope (T25.F07 invariant 1): it is eligible and
/// a plastic member, at either scope, passes `validate_graph`; a plastic
/// `ClearSlot` edge is still rejected.
#[test]
fn validate_graph_admits_a_plastic_member_and_rejects_a_plastic_clear_slot_edge() {
    let (mut def, id) = grouped(DirectedVoteKind::Move, 1);
    let members: Vec<EdgeOccurrence> = def.members(id).collect();
    assert!(members.iter().all(|&member| def.can_learn(member)));
    def.edge_mut(members[0]).set_plasticity(plastic());
    def.edge_mut(members[1]).set_plasticity(plastic());
    def.edge_mut(members[1]).set_scope(Scope::Group);
    assert_eq!(
        validate_graph(&learning_node(def.clone(), ring_refs())),
        Ok(())
    );
    let clear = def.sink_surface(OutputSinkKind::ClearSlot(0)).unwrap();
    def.push_edge(clear, GraphEdge::new(leaf(0), 1.0));
    let EdgeSurface::SinkInput(sink) = clear else {
        unreachable!()
    };
    assert!(!def.can_learn(EdgeOccurrence::new(clear, 0)));
    def.output_sinks.0[sink].inputs.0[0].plasticity = plastic();
    assert_eq!(
        validate_graph(&learning_node(def, ring_refs())),
        Err(GraphInvariantViolation::IneligiblePlasticEdge)
    );
}

/// The node-level key is gone (T25.F06 invariant 1): a `ComputeNode` with
/// `plasticity` is refused, and the edge-level key reads.
#[test]
fn a_node_level_plasticity_key_is_refused() {
    let config = r#"{"rule":"Classic","learning_rate":0.1,"weight_clamp":5.0,"lamarckian":true,"retention":1.0}"#;
    let node_level = format!(
        r#"{{"kind":"Add","inputs":[{{"source":{{"ComputeNode":0}},"own_weight":0.5}}],"plasticity":{config}}}"#
    );
    assert!(serde_json::from_str::<ComputeNode>(&node_level).is_err());
    let edge_level = format!(
        r#"{{"kind":"Add","inputs":[{{"source":{{"ComputeNode":0}},"own_weight":0.5,"plasticity":{config}}}]}}"#
    );
    let node: ComputeNode = serde_json::from_str(&edge_level).unwrap();
    assert_eq!(node.inputs[0].plasticity(), plastic());
    assert_eq!(serde_json::to_string(&node).unwrap(), edge_level);
}

/// `plastic_edges` enumerates every plastic edge on both surfaces in
/// canonical order, and nothing else.
#[test]
fn plastic_edges_enumerate_both_surfaces_in_canonical_order() {
    let mut def = seeded();
    let vote = EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 2));
    def.push_edge(vote, GraphEdge::new(leaf(0), 1.0));
    def.edge_mut(EdgeOccurrence::new(vote, 0))
        .set_plasticity(plastic());
    let sites: Vec<EdgeOccurrence> = def.plastic_edges().map(|(site, _)| site).collect();
    assert_eq!(
        sites,
        vec![
            compute(0, 0),
            compute(0, 1),
            compute(1, 0),
            EdgeOccurrence::new(vote, 0)
        ]
    );
    // `seeded`'s sink edge is static.
    assert!(def
        .plastic_edges()
        .all(|(site, config)| Some(config) == def.edge_at(site).plasticity()));
}

/// One operation of the seed-row model.
#[derive(Debug, Clone)]
enum SeedOp {
    PushCompute(usize),
    PushSink(usize),
    Remove(usize),
    Split(usize),
    SetWeight(usize),
    SetSource(usize, bool),
    SetPlasticity(usize),
    AddBundle(u8),
    AddPairGene(u8),
    /// A `Plain` bundle on vote sink `index % 39`, reading a memory bank
    /// (`Some(previous)`) or the `ActionVotes` sensor.
    AddPlain(usize, Option<bool>),
    Relocate(u8),
    Detach(usize),
    RemoveBundle,
    SetScope(usize),
    SetInheritScale,
    SetScale,
    RemoveNode(usize),
    Clone,
}

fn seed_op_strategy() -> impl Strategy<Value = SeedOp> {
    prop_oneof![
        (0usize..64).prop_map(SeedOp::PushCompute),
        (0usize..64).prop_map(SeedOp::PushSink),
        (0usize..64).prop_map(SeedOp::Remove),
        (0usize..64).prop_map(SeedOp::Split),
        (0usize..64).prop_map(SeedOp::SetWeight),
        (0usize..64, any::<bool>()).prop_map(|(i, same)| SeedOp::SetSource(i, same)),
        (0usize..64).prop_map(SeedOp::SetPlasticity),
        (0u8..8).prop_map(SeedOp::AddBundle),
        (0u8..8).prop_map(SeedOp::AddPairGene),
        (0usize..64, prop::option::of(any::<bool>()))
            .prop_map(|(sink, memory)| SeedOp::AddPlain(sink, memory)),
        (0u8..8).prop_map(SeedOp::Relocate),
        (0usize..64).prop_map(SeedOp::Detach),
        Just(SeedOp::RemoveBundle),
        (0usize..64).prop_map(SeedOp::SetScope),
        Just(SeedOp::SetInheritScale),
        Just(SeedOp::SetScale),
        (0usize..64).prop_map(SeedOp::RemoveNode),
        Just(SeedOp::Clone),
    ]
}

/// The sink indices the model wires: two outputs, a write, three votes.
fn model_sinks() -> [usize; 6] {
    [
        0,
        1,
        CUSTOM_OUTPUT_COUNT as usize + MAX_GATE_SLOTS,
        vote_sink(DirectedVoteKind::Move, 0),
        vote_sink(DirectedVoteKind::Move, 5),
        vote_sink(DirectedVoteKind::Reproduce, 3),
    ]
}

proptest! {
    /// T25.F06 invariant 8, T25.F07 invariant 6, model-based: every edge
    /// carries a unique tag as its own weight and every bundle a unique tag
    /// as its scale (in `(0.9, 1.5)`, so an integer own tag plus a scale tag
    /// is unique too) and, when seeded, the tag as its seed. Through any
    /// sequence of primitives on both surfaces (push, remove, split, the
    /// source and weight writes, creation of every wiring from both sources
    /// (`Ring`, `DirectionPair` with its facts, `Plain` on a sensor or a
    /// memory bank, T25.F11), relocation, detach and removal, scope,
    /// `inherit_scale` and scale writes, node removal, a draft's clone) each
    /// row and the bundle slots stay aligned with their edges and records
    /// (node duplication copies rows and is pinned by `cgp_reproduction`'s
    /// copy-remove property): a seed is its own tag exactly while the model
    /// keeps the tag seeded, `None` otherwise; a detached member's seed is
    /// its composed weight when its edge or its bundle was seeded, for
    /// every member (the one member law).
    #[test]
    fn seed_rows_follow_their_edges_through_every_primitive(
        ops in prop::collection::vec(seed_op_strategy(), 1..40),
    ) {
        let mut next_tag = 1000.0f32;
        let mut fresh = || { next_tag += 1.0; next_tag };
        let mut seeded_tags = std::collections::BTreeSet::new();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for node in 0..3 {
            let inputs = (0..2).map(|_| GraphEdge::new(leaf(0), fresh())).collect();
            def.append_compute_node(ComputeNode::new(ComputeNodeKind::Add, inputs));
            if node > 0 {
                def.push_edge(EdgeSurface::ComputeInput(node), GraphEdge::new(GraphSource::ComputeNode(node as u16 - 1), fresh()));
            }
        }
        for sink in model_sinks() {
            def.push_edge(EdgeSurface::SinkInput(sink), GraphEdge::new(GraphSource::ComputeNode(2), fresh()));
        }
        // Two bundles, every member and record tagged.
        let tag_bundle = |def: &mut CgpGraphBackendDef, id: BundleId, fresh: &mut dyn FnMut() -> f32| {
            for member in def.members(id).collect::<Vec<_>>() {
                def.edge_mut(member).set_weight(fresh());
            }
            def.bundle_mut(id).set_scale(fresh() / 1024.0);
        };
        for (kind, offset) in [(DirectedVoteKind::Move, 2), (DirectedVoteKind::StealEnergy, 5)] {
            let id = def.add_ring_bundle(RING, kind, offset);
            tag_bundle(&mut def, id, &mut fresh);
        }
        // Seed every edge and bundle with its tag.
        let tag_rows = |def: &CgpGraphBackendDef, surface_count: usize, surface: fn(usize) -> EdgeSurface| -> Vec<Vec<Option<f32>>> {
            (0..surface_count)
                .map(|i| def.surface_edges(surface(i)).iter().map(|e| Some(e.own_weight())).collect())
                .collect()
        };
        let compute = tag_rows(&def, def.compute_nodes().len(), EdgeSurface::ComputeInput);
        let sinks = tag_rows(&def, def.output_sinks().len(), EdgeSurface::SinkInput);
        let bundles = def.bundles().iter().map(|bundle| Some(bundle.scale())).collect();
        def.seed_birth(BirthSeeds::Rows { compute, sinks, bundles });
        for edge in def.edges() {
            seeded_tags.insert(edge.own_weight().to_bits());
        }
        let mut seeded_scales: std::collections::BTreeSet<u32> =
            def.bundles().iter().map(|bundle| bundle.scale().to_bits()).collect();
        let refs = pair_refs();
        for op in ops {
            let sites: Vec<EdgeOccurrence> = def.edge_sites().collect();
            let pick = |i: usize| sites.get(i % sites.len().max(1)).copied();
            let independent = |def: &CgpGraphBackendDef, site: EdgeOccurrence| def.edge_at(site).membership().is_none();
            match op {
                SeedOp::PushCompute(i) => {
                    let n = def.compute_nodes().len();
                    if n > 0 {
                        def.push_edge(EdgeSurface::ComputeInput(i % n), GraphEdge::new(leaf(0), fresh()));
                    }
                }
                SeedOp::PushSink(i) => {
                    let sinks = model_sinks();
                    def.push_edge(EdgeSurface::SinkInput(sinks[i % sinks.len()]), GraphEdge::new(leaf(0), fresh()));
                }
                SeedOp::Remove(i) => {
                    if let Some(site) = pick(i) {
                        seeded_tags.remove(&def.edge_at(site).own_weight().to_bits());
                        def.remove_edge(site);
                    }
                }
                SeedOp::Split(i) => {
                    if let Some(site) = pick(i).filter(|&site| independent(&def, site)) {
                        def.split_edge(site, ComputeNode::new(ComputeNodeKind::Add, Vec::new()));
                    }
                }
                SeedOp::SetWeight(i) => {
                    if let Some(site) = pick(i).filter(|&site| independent(&def, site)) {
                        seeded_tags.remove(&def.edge_at(site).own_weight().to_bits());
                        def.edge_mut(site).set_weight(fresh());
                    }
                }
                SeedOp::SetSource(i, same) => {
                    if let Some(site) = pick(i).filter(|&site| independent(&def, site)) {
                        let current = def.edge_at(site).source();
                        let source = if same { current } else { leaf(7) };
                        let before = def.birth_seeds().and_then(|seeds| seeds.value(site));
                        def.edge_mut(site).set_source(source);
                        let after = def.birth_seeds().and_then(|seeds| seeds.value(site));
                        if source == current {
                            prop_assert_eq!(after, before, "a same-source write keeps the seed");
                        } else {
                            prop_assert_eq!(after, None, "a changed source clears the seed");
                            seeded_tags.remove(&def.edge_at(site).own_weight().to_bits());
                        }
                    }
                }
                SeedOp::SetPlasticity(i) => {
                    if let Some(site) = pick(i).filter(|&site| def.can_learn(site)) {
                        let config = if def.edge_at(site).plasticity().is_some() { None } else { plastic() };
                        def.edge_mut(site).set_plasticity(config);
                    }
                }
                SeedOp::AddBundle(offset) => {
                    if def.bundles().len() < 3 {
                        let id = def.add_ring_bundle(RING, DirectedVoteKind::Move, offset);
                        prop_assert_eq!(def.birth_seeds().unwrap().bundle_value(id), None);
                        tag_bundle(&mut def, id, &mut fresh);
                    }
                }
                SeedOp::AddPairGene(offset) => {
                    if def.bundles().len() < 3 {
                        let id = def.add_pair_bundle(PAIR_REF, 1, pair_layout(), DirectedVoteKind::Move, offset);
                        prop_assert_eq!(def.birth_seeds().unwrap().bundle_value(id), None);
                        tag_bundle(&mut def, id, &mut fresh);
                    }
                }
                SeedOp::AddPlain(sink, memory) => {
                    if def.bundles().len() < 3 {
                        let (source, set) = match memory {
                            Some(previous) => (BundleSource::Memory { previous }, crate::contracts::MEMORY_BANK),
                            None => (
                                BundleSource::Sensor { ref_idx: 0, field_set: 0 },
                                crate::contracts::FieldSet::Plain { width: 39 },
                            ),
                        };
                        let wiring = Wiring::Plain { sink: VoteSink::from_index(sink % 39).unwrap() };
                        let birth = crate::genome::cgp::BundleBirth { field_set: 0, wiring, scale: 0.0 };
                        let id = def.add_bundle(source, set, birth).unwrap();
                        prop_assert_eq!(def.birth_seeds().unwrap().bundle_value(id), None);
                        tag_bundle(&mut def, id, &mut fresh);
                    }
                }
                SeedOp::Relocate(offset) => {
                    if let Some(bundle) = def.bundles().first().copied() {
                        let wiring = match bundle.wiring() {
                            Wiring::Plain { .. } => Wiring::Plain {
                                sink: VoteSink::from_index(usize::from(offset) * 4).unwrap(),
                            },
                            geometric => geometric.rotated(DirectedVoteKind::Reproduce, offset),
                        };
                        def.relocate_bundle(BundleId::new(0), wiring);
                    }
                }
                SeedOp::Detach(i) => {
                    let members: Vec<EdgeOccurrence> = if def.bundles().is_empty() { Vec::new() } else { def.members(BundleId::new(0)).collect() };
                    if let Some(&member) = members.get(i % members.len().max(1)) {
                        let pattern = def.member_pattern(member, &refs);
                        let own = def.edge_at(member).own_weight();
                        let scale = def.bundles()[0].scale();
                        let edge_seeded = seeded_tags.remove(&own.to_bits());
                        // Every member's seed composes when its edge or its
                        // bundle was seeded (T25.F11 invariant 4).
                        let seeded = edge_seeded || seeded_scales.contains(&scale.to_bits());
                        let (detached, _) = def.detach_at(member, pattern);
                        let composed = def.edge_at(detached).own_weight();
                        prop_assert_eq!(composed, compose(own, scale, pattern));
                        if seeded {
                            seeded_tags.insert(composed.to_bits());
                        }
                    }
                }
                SeedOp::RemoveBundle => {
                    if !def.bundles().is_empty() {
                        def.remove_bundle(BundleId::new(0));
                    }
                }
                SeedOp::SetScope(i) => {
                    let members: Vec<EdgeOccurrence> = def.edge_sites().filter(|&site| !independent(&def, site)).collect();
                    if let Some(&member) = members.get(i % members.len().max(1)) {
                        let scope = def.edge_at(member).membership().unwrap().scope.other();
                        def.edge_mut(member).set_scope(scope);
                    }
                }
                SeedOp::SetInheritScale => {
                    if let Some(bundle) = def.bundles().first().copied() {
                        def.bundle_mut(BundleId::new(0)).set_inherit_scale(!bundle.inherit_scale());
                    }
                }
                SeedOp::SetScale => {
                    if let Some(bundle) = def.bundles().first().copied() {
                        seeded_scales.remove(&bundle.scale().to_bits());
                        def.bundle_mut(BundleId::new(0)).set_scale(fresh() / 1024.0);
                    }
                }
                SeedOp::RemoveNode(i) => {
                    let n = def.compute_nodes().len();
                    if n > 0 {
                        let removed = i % n;
                        for site in def.edge_sites() {
                            let edge = def.edge_at(site);
                            let doomed = site.surface == EdgeSurface::ComputeInput(removed)
                                || edge.source() == GraphSource::ComputeNode(removed as u16);
                            if doomed {
                                seeded_tags.remove(&edge.own_weight().to_bits());
                            }
                        }
                        def.remove_compute_node(removed);
                    }
                }
                SeedOp::Clone => def = def.clone(),
            }
            let seeds = def.birth_seeds().expect("the draft keeps its seeds").clone();
            prop_assert_eq!(seeds.compute_rows().len(), def.compute_nodes().len());
            prop_assert_eq!(seeds.sink_rows().len(), def.output_sinks().len());
            prop_assert_eq!(seeds.bundle_slots().len(), def.bundles().len());
            for (index, bundle) in def.bundles().iter().enumerate() {
                let tag = bundle.scale();
                let expected = seeded_scales.contains(&tag.to_bits()).then_some(tag);
                prop_assert_eq!(seeds.bundle_slots()[index], expected, "bundle {}", index);
            }
            for site in def.edge_sites() {
                let tag = def.edge_at(site).own_weight();
                let expected = seeded_tags.contains(&tag.to_bits()).then_some(tag);
                prop_assert_eq!(seeds.value(site), expected, "{:?}", site);
            }
            prop_assert_eq!(validate_graph(&learning_node(def.clone(), refs.clone())), Ok(()));
        }
    }
}

/// Seed `def`'s draft: every edge row empty, `edges` and `bundles` set.
fn seed_with(
    def: &mut CgpGraphBackendDef,
    edges: &[(EdgeOccurrence, f32)],
    bundles: &[(BundleId, f32)],
) {
    let mut compute: Vec<Vec<Option<f32>>> = def
        .compute_nodes()
        .iter()
        .map(|node| vec![None; node.inputs.len()])
        .collect();
    let mut sinks: Vec<Vec<Option<f32>>> = def
        .output_sinks()
        .iter()
        .map(|sink| vec![None; sink.inputs.len()])
        .collect();
    for &(at, value) in edges {
        let row = match at.surface {
            EdgeSurface::ComputeInput(i) => &mut compute[i],
            EdgeSurface::SinkInput(i) => &mut sinks[i],
        };
        row[at.index] = Some(value);
    }
    let mut slots = vec![None; def.bundles().len()];
    for &(bundle, value) in bundles {
        slots[bundle.index()] = Some(value);
    }
    def.seed_birth(BirthSeeds::Rows {
        compute,
        sinks,
        bundles: slots,
    });
}

/// Invariant 6: a detached member's seed composes the edge's seed (or its
/// own weight) with the bundle's seed (or its scale) at its pattern, with no
/// clamp; with neither seed it keeps none; a last-member detach drops the
/// bundle's slot.
#[test]
fn a_detached_member_composes_its_seed_with_the_learned_scale() {
    let (mut def, id) = paired(DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(0.5);
    let refs = pair_refs();
    let all: Vec<EdgeOccurrence> = def.members(id).collect();
    let (first, second) = (all[0], all[1]);
    def.edge_mut(first).set_weight(0.25);
    def.edge_mut(second).set_weight(-0.75);
    seed_with(&mut def, &[(first, 9.0)], &[(id, 12.0)]);
    let pattern = def.member_pattern(first, &refs);
    let (at, removed) = def.detach_at(first, pattern);
    assert!(!removed);
    assert_eq!(
        def.birth_seeds().unwrap().value(at),
        Some(compose(9.0, 12.0, pattern)),
        "both seeds, no clamp"
    );
    assert_eq!(def.edge_at(at).own_weight(), compose(0.25, 0.5, pattern));

    let mut bundle_only = def.clone();
    let second_at = bundle_only.members(id).next().unwrap();
    assert_eq!(second_at, def.members(id).next().unwrap());
    let pattern = bundle_only.member_pattern(second_at, &refs);
    let own = bundle_only.edge_at(second_at).own_weight();
    let (at, _) = bundle_only.detach_at(second_at, pattern);
    assert_eq!(
        bundle_only.birth_seeds().unwrap().value(at),
        Some(compose(own, 12.0, pattern)),
        "the bundle seed with the edge's own weight"
    );

    let mut neither = def.clone();
    neither.bundle_mut(id).set_scale(0.5);
    assert_eq!(neither.birth_seeds().unwrap().bundle_value(id), None);
    let member = neither.members(id).next().unwrap();
    let pattern = neither.member_pattern(member, &refs);
    let (at, _) = neither.detach_at(member, pattern);
    assert_eq!(neither.birth_seeds().unwrap().value(at), None);

    // Detaching every member drops the bundle and its slot.
    let mut last = def.clone();
    while let Some(member) = last.bundles().first().and_then(|_| last.members(id).next()) {
        let pattern = last.member_pattern(member, &refs);
        last.detach_at(member, pattern);
    }
    assert!(last.bundles().is_empty());
    assert!(last.birth_seeds().unwrap().bundle_slots().is_empty());
}

/// R5: a bundle with members at both scopes, distinct edge seeds and a bundle
/// seed survives a relocation and its inverse unchanged, seeds included.
#[test]
fn a_mixed_scope_bundle_survives_a_relocation_and_its_inverse() {
    let (mut def, id) = paired(DirectedVoteKind::Move, 3);
    let all: Vec<EdgeOccurrence> = def.members(id).collect();
    for (k, &member) in all.iter().enumerate() {
        def.edge_mut(member).set_weight(k as f32 + 0.5);
        if k % 2 == 1 {
            def.edge_mut(member).set_scope(Scope::Group);
        }
    }
    let edge_seeds: Vec<(EdgeOccurrence, f32)> = all
        .iter()
        .enumerate()
        .map(|(k, &member)| (member, 100.0 + k as f32))
        .collect();
    seed_with(&mut def, &edge_seeds, &[(id, 7.5)]);
    let before = def.clone();
    let seeds_before = def.birth_seeds().unwrap().clone();
    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::Reproduce, 6));
    assert_eq!(
        def.members(id).count(),
        all.len(),
        "scope never splits a bundle"
    );
    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::Move, 3));
    assert_eq!(def, before);
    assert_eq!(def.birth_seeds().unwrap(), &seeds_before);
}

/// `set_scope`, `set_inherit_scale` and `set_retention` (T25.F09) keep
/// every seed; `set_scale` clears the bundle's.
#[test]
fn scope_and_inherit_scale_writes_keep_seeds_and_set_scale_clears_the_bundle_seed() {
    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    let member = def.members(id).next().unwrap();
    seed_with(&mut def, &[(member, 2.0)], &[(id, 3.0)]);
    def.edge_mut(member).set_scope(Scope::Group);
    def.bundle_mut(id).set_inherit_scale(true);
    def.bundle_mut(id).set_retention(0.25);
    assert_eq!(def.bundles()[id.index()].retention(), 0.25);
    let seeds = def.birth_seeds().unwrap();
    assert_eq!(
        (seeds.value(member), seeds.bundle_value(id)),
        (Some(2.0), Some(3.0))
    );
    assert!(def.bundles()[id.index()].inherit_scale());
    def.bundle_mut(id).set_scale(0.5);
    let seeds = def.birth_seeds().unwrap();
    assert_eq!(
        (seeds.value(member), seeds.bundle_value(id)),
        (Some(2.0), None)
    );
}

// ── Fact members (T25.F10, T25.F11) ─────────────────────────────────────────

/// The pair's declared facts.
fn facts() -> &'static [u16] {
    pair_layout().facts
}

/// `paired` after an independent edge on the kind's drive: one gene, twelve
/// direction members and three facts.
fn fact_gene(kind: DirectedVoteKind, offset: u8) -> (CgpGraphBackendDef, BundleId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.push_edge(
        EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 0)),
        GraphEdge::new(leaf(0), 0.5),
    );
    let drive = drive_sink(&def, kind);
    def.push_edge(drive, GraphEdge::new(leaf(0), -0.5));
    let id = def.add_pair_bundle(PAIR_REF, 1, pair_layout(), kind, offset);
    (def, id)
}

/// The fact members of `id`, in canonical order.
fn fact_members(def: &CgpGraphBackendDef, id: BundleId) -> Vec<EdgeOccurrence> {
    def.members(id)
        .filter(|&member| def.site_of(member) == Some(Site::Drive))
        .collect()
}

fn drive_sink(def: &CgpGraphBackendDef, kind: DirectedVoteKind) -> EdgeSurface {
    def.sink_surface(OutputSinkKind::ActionVote(kind.drive()))
        .unwrap()
}

/// The pair gene is the record, twelve direction members and one fact
/// member per declared fact on its kind's drive, after the drive's
/// independent edge, every one born 0, static, at `Edge`, with pattern 1
/// (T25.F11 invariant 3: one member law).
#[test]
fn a_pair_gene_places_one_fact_member_per_fact_on_its_kinds_drive() {
    for kind in DirectedVoteKind::ALL {
        let (mut def, id) = fact_gene(kind, 5);
        assert_eq!(def.members(id).count(), 12 + facts().len());
        let found = fact_members(&def, id);
        let mut fields: Vec<u16> = found
            .iter()
            .map(|&member| match def.edge_at(member).source() {
                GraphSource::InputLeaf { ref_idx, sub_idx } => {
                    assert_eq!(ref_idx, PAIR_REF);
                    sub_idx
                }
                other => panic!("a fact member reads a leaf, not {other:?}"),
            })
            .collect();
        let mut want = facts().to_vec();
        want.sort_unstable();
        fields.sort_unstable();
        assert_eq!(fields, want);
        let drive = drive_sink(&def, kind);
        for (k, &member) in found.iter().enumerate() {
            assert_eq!(member.surface, drive);
            assert_eq!(member.index, k + 1, "after the independent prefix");
            let edge = def.edge_at(member);
            assert_eq!(edge.own_weight(), 0.0);
            assert_eq!(edge.plasticity(), None);
            assert_eq!(edge.membership().unwrap().scope, Scope::Edge);
            assert_eq!(def.member_pattern(member, &pair_refs()), 1.0);
        }
        // Pattern 1: the scale reaches a fact member as any member.
        def.bundle_mut(id).set_scale(3.0);
        for &member in &found {
            def.edge_mut(member).set_weight(0.25);
            assert_eq!(def.genomic_weight(member, &pair_refs()), 3.25);
        }
        assert_eq!(validate_graph(&pair_node(def, pair_refs())), Ok(()));
    }
}

/// Relocation: a shift leaves fact members on their drive, a retarget
/// moves them to the new kind's drive keeping own weight, plasticity, scope
/// and seed; the inverse restores the def and its seeds.
#[test]
fn fact_members_stay_on_their_drive_under_a_shift_and_follow_a_retarget() {
    let (mut def, id) = fact_gene(DirectedVoteKind::Move, 2);
    for (k, member) in fact_members(&def, id).into_iter().enumerate() {
        def.edge_mut(member).set_weight(k as f32 - 0.5);
        def.edge_mut(member).set_plasticity(plastic());
        def.edge_mut(member).set_scope(Scope::Group);
    }
    let seeds: Vec<(EdgeOccurrence, f32)> = def
        .members(id)
        .enumerate()
        .map(|(k, member)| (member, 50.0 + k as f32))
        .collect();
    seed_with(&mut def, &seeds, &[(id, 4.0)]);
    let snapshot = |def: &CgpGraphBackendDef| {
        fact_members(def, id)
            .into_iter()
            .map(|member| {
                (
                    *def.edge_at(member),
                    def.birth_seeds().unwrap().value(member),
                )
            })
            .collect::<Vec<_>>()
    };
    let facts_before = snapshot(&def);
    let original = def.clone();
    let original_seeds = def.birth_seeds().unwrap().clone();

    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::Move, 3));
    let move_drive = drive_sink(&def, DirectedVoteKind::Move);
    assert!(fact_members(&def, id)
        .iter()
        .all(|member| member.surface == move_drive));
    assert_eq!(snapshot(&def), facts_before);

    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::StealEnergy, 3));
    let steal_drive = drive_sink(&def, DirectedVoteKind::StealEnergy);
    let moved = fact_members(&def, id);
    assert_eq!(moved.len(), facts().len());
    assert!(moved.iter().all(|member| member.surface == steal_drive));
    assert_eq!(snapshot(&def), facts_before);
    assert_eq!(validate_graph(&pair_node(def.clone(), pair_refs())), Ok(()));

    def.relocate_bundle(id, pair_wiring(DirectedVoteKind::Move, 2));
    assert_eq!(def, original);
    assert_eq!(def.birth_seeds().unwrap(), &original_seeds);
}

/// Validation by role: a fact member reads one of its pair's facts on its
/// own kind's drive, only in a `DirectionPair` gene; a direction member
/// never reads a fact; a repeated fact is a duplicate.
#[test]
fn validate_graph_checks_each_member_role_by_its_site() {
    let (def, id) = fact_gene(DirectedVoteKind::Move, 1);
    let refs = pair_refs();
    assert_eq!(
        validate_graph(&pair_node(def.clone(), refs.clone())),
        Ok(())
    );
    let fact = fact_members(&def, id)[0];
    let EdgeSurface::SinkInput(drive) = fact.surface else {
        unreachable!()
    };
    let rewrite = |def: &mut CgpGraphBackendDef, at: EdgeOccurrence, sub_idx: u16| {
        let EdgeSurface::SinkInput(sink) = at.surface else {
            unreachable!()
        };
        def.output_sinks.0[sink].inputs.0[at.index].source = GraphSource::InputLeaf {
            ref_idx: PAIR_REF,
            sub_idx,
        };
        resort(def, sink);
    };
    // A fact member reading the pair's x field.
    let mut x_on_drive = def.clone();
    rewrite(&mut x_on_drive, fact, PX);
    assert_eq!(x_on_drive.check_local_invariants(), Ok(()));
    assert_eq!(
        validate_graph(&pair_node(x_on_drive, refs.clone())),
        Err(GraphInvariantViolation::MemberShape)
    );
    // A direction member reading a fact.
    let direction = def.members(id).next().unwrap();
    let mut fact_on_direction = def.clone();
    rewrite(&mut fact_on_direction, direction, facts()[0]);
    assert_eq!(
        validate_graph(&pair_node(fact_on_direction, refs.clone())),
        Err(GraphInvariantViolation::MemberShape)
    );
    // A fact member on another kind's drive.
    let mut other_drive = def.clone();
    let edge = other_drive.output_sinks.0[drive]
        .inputs
        .0
        .remove(fact.index);
    let EdgeSurface::SinkInput(steal) = drive_sink(&def, DirectedVoteKind::StealEnergy) else {
        unreachable!()
    };
    other_drive.output_sinks.0[steal].inputs.0.push(edge);
    assert_eq!(
        other_drive.check_local_invariants(),
        Err(GraphInvariantViolation::MemberShape)
    );
    // A repeated fact member.
    let mut duplicate = def.clone();
    let copied = *duplicate.edge_at(fact);
    duplicate.output_sinks.0[drive]
        .inputs
        .0
        .insert(fact.index, copied);
    assert_eq!(
        duplicate.check_local_invariants(),
        Err(GraphInvariantViolation::DuplicateMember)
    );
    // A `Ring` member on its kind's drive.
    let (mut ringed, ring_id) = grouped(DirectedVoteKind::Move, 0);
    let member = ringed.members(ring_id).next().unwrap();
    let EdgeSurface::SinkInput(sink) = member.surface else {
        unreachable!()
    };
    let edge = ringed.output_sinks.0[sink].inputs.0.remove(member.index);
    let EdgeSurface::SinkInput(move_drive) = drive_sink(&ringed, DirectedVoteKind::Move) else {
        unreachable!()
    };
    ringed.output_sinks.0[move_drive].inputs.0.push(edge);
    assert_eq!(
        ringed.check_local_invariants(),
        Err(GraphInvariantViolation::MemberShape)
    );
}

/// Deletion: removing a fact member leaves a fact hole; a gene outlives its
/// direction members, and a fact-only survivor validates and round-trips
/// genome JSON; its last member's removal drops the record.
#[test]
fn a_fact_only_survivor_is_a_valid_gene_until_its_last_member_goes() {
    let (mut def, id) = fact_gene(DirectedVoteKind::Reproduce, 4);
    let hole = fact_members(&def, id)[1];
    def.remove_edge(hole);
    assert_eq!(fact_members(&def, id).len(), facts().len() - 1);
    loop {
        let direction = def
            .members(id)
            .find(|&member| def.site_of(member) != Some(Site::Drive));
        let Some(direction) = direction else { break };
        def.remove_edge(direction);
    }
    assert_eq!(def.bundles().len(), 1);
    assert_eq!(def.members(id).count(), facts().len() - 1);
    let node = pair_node(def.clone(), pair_refs());
    assert_eq!(validate_graph(&node), Ok(()));
    let loaded: NodeGenome = serde_json::from_str(&serde_json::to_string(&node).unwrap()).unwrap();
    assert_eq!(loaded, node);
    assert_eq!(validate_graph(&loaded), Ok(()));
    while let Some(member) = def.bundles().first().and_then(|_| def.members(id).next()) {
        def.remove_edge(member);
    }
    assert!(def.bundles().is_empty());
}

/// A fact member detaches as every member (T25.F11 invariant 4): its own
/// weight becomes `compose(own, scale, 1)`, its seed composes from the
/// edge's and the bundle's seeds; it moves to the independent prefix of its
/// drive; the last member's detach drops the record.
#[test]
fn a_fact_member_detaches_by_the_one_member_law() {
    let (mut def, id) = fact_gene(DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_scale(0.5);
    let found = fact_members(&def, id);
    def.edge_mut(found[0]).set_weight(-0.25);
    def.edge_mut(found[1]).set_weight(0.75);
    seed_with(&mut def, &[(found[1], 9.0)], &[(id, 12.0)]);
    let refs = pair_refs();
    let drive = drive_sink(&def, DirectedVoteKind::Move);

    let (at, removed) = detach(&mut def, found[0], &refs);
    assert!(!removed);
    assert_eq!(at.surface, drive);
    assert_eq!(at.index, 1, "the end of the independent prefix");
    let edge = def.edge_at(at);
    assert_eq!(edge.membership(), None);
    assert_eq!(edge.own_weight(), compose(-0.25, 0.5, 1.0));
    assert_eq!(
        def.birth_seeds().unwrap().value(at),
        Some(compose(-0.25, 12.0, 1.0)),
        "the bundle seed with the edge's own weight"
    );

    let seeded = fact_members(&def, id)[0];
    assert_eq!(def.edge_at(seeded).own_weight(), 0.75);
    let (at, _) = detach(&mut def, seeded, &refs);
    assert_eq!(def.edge_at(at).own_weight(), compose(0.75, 0.5, 1.0));
    assert_eq!(
        def.birth_seeds().unwrap().value(at),
        Some(compose(9.0, 12.0, 1.0))
    );
    assert_eq!(
        validate_graph(&pair_node(def.clone(), refs.clone())),
        Ok(())
    );

    while let Some(member) = def.bundles().first().and_then(|_| def.members(id).next()) {
        detach(&mut def, member, &refs);
    }
    assert!(def.bundles().is_empty());
    assert!(def.birth_seeds().unwrap().bundle_slots().is_empty());
}

// ── Birth coordinates (T30.F01) ─────────────────────────────────────────────

/// The `(field, site)` of every member of `id`.
fn site_ofs(def: &CgpGraphBackendDef, id: BundleId) -> Vec<(u16, Site)> {
    def.members(id)
        .map(|member| {
            let GraphSource::InputLeaf { sub_idx, .. } = def.edge_at(member).source() else {
                unreachable!("members read leaves")
            };
            (sub_idx, def.site_of(member).expect("a member on its kind"))
        })
        .collect()
}

/// The `(field, site)` of every coordinate, in the API's order.
fn coordinate_sites(coordinates: &[BirthCoordinate]) -> Vec<(u16, Site)> {
    coordinates.iter().map(|c| (c.field, c.site)).collect()
}

proptest! {
    /// A new bundle's birth coordinates are exactly its members, each
    /// present: a ring's eight channels on `(channel + offset) mod 8`, a
    /// pair's twelve direction members, then its declared facts on the
    /// drive.
    #[test]
    fn a_new_bundles_coordinates_are_its_members_all_present(
        kind in 0usize..4,
        offset in 0u8..8,
        pair in any::<bool>(),
    ) {
        let kind = DirectedVoteKind::ALL[kind];
        let (def, id, refs) = if pair {
            let (def, id) = fact_gene(kind, offset);
            (def, id, pair_refs())
        } else {
            let (def, id) = grouped(kind, offset);
            (def, id, ring_refs())
        };
        let coordinates = def.birth_coordinates(id, &refs, &crate::contracts::SensorContext::WIDEST);
        prop_assert!(coordinates.iter().all(|c| c.present));
        let mut sites = coordinate_sites(&coordinates);
        let expected = if pair { 12 + facts().len() } else { 8 };
        prop_assert_eq!(sites.len(), expected);
        if !pair {
            for c in &coordinates {
                let channel = u8::try_from(c.field).expect("a ring channel");
                prop_assert_eq!(c.site, Site::Direction((channel + offset) % 8));
            }
        }
        let mut members = site_ofs(&def, id);
        sites.sort_unstable();
        members.sort_unstable();
        prop_assert_eq!(sites, members);
    }

    /// Removing one member marks exactly its coordinate absent.
    #[test]
    fn a_removed_member_is_its_coordinate_absent(offset in 0u8..8, pick in 0usize..15) {
        let (mut def, id) = fact_gene(DirectedVoteKind::Move, offset);
        let refs = pair_refs();
        let members: Vec<_> = def.members(id).collect();
        let pick = pick % members.len();
        let site = site_ofs(&def, id)[pick];
        def.remove_edge(members[pick]);
        let absent: Vec<_> = def
            .birth_coordinates(id, &refs, &crate::contracts::SensorContext::WIDEST)
            .into_iter()
            .filter(|c| !c.present)
            .map(|c| (c.field, c.site))
            .collect();
        prop_assert_eq!(absent, vec![site]);
    }
}

/// A pair gene lists every compass and fact coordinate present; a pair
/// whose reference no longer declares it has no coordinates.
#[test]
fn an_unresolvable_pair_has_no_coordinates() {
    let (def, id) = paired(DirectedVoteKind::Move, 0);
    let coordinates =
        def.birth_coordinates(id, &pair_refs(), &crate::contracts::SensorContext::WIDEST);
    let (directions, drives): (Vec<&BirthCoordinate>, Vec<_>) = coordinates
        .iter()
        .partition(|c| matches!(c.site, Site::Direction(_)));
    assert_eq!(directions.len(), 12);
    let fields: Vec<u16> = drives.iter().map(|c| c.field).collect();
    assert_eq!(fields, facts());
    assert!(coordinates.iter().all(|c| c.present));
    assert!(def
        .birth_coordinates(id, &ring_refs(), &crate::contracts::SensorContext::WIDEST)
        .is_empty());
}

/// A bundle over the widest `ActionQueue` (65,536 fields) lists every birth
/// coordinate present: presence is a keyed lookup, not a scan of the members
/// per coordinate (quadratic at this width, about 2.15 billion comparisons).
#[test]
fn the_widest_queue_bundle_lists_every_coordinate_present() {
    // Arrange: one bundle over every queue field under the widest context,
    // its members appended in I10 order (the record and edges `add_bundle`
    // writes, without its per-member insertion search).
    let refs = [InputReference::ActionQueue];
    let context = crate::contracts::SensorContext::WIDEST;
    let source = BundleSource::Sensor {
        ref_idx: 0,
        field_set: 0,
    };
    let set = source
        .resolve_in(&refs, &context)
        .expect("the queue resolves");
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let wiring = *plans(&def, &set).first().expect("an admissible plan");
    let coordinates = wiring.coordinates(&set);
    assert_eq!(coordinates.len(), 65_536);
    let id = BundleId::new(0);
    def.bundles.push(Bundle::new(source, wiring));
    for &(field, site) in &coordinates {
        let sink = def.site_sink(&wiring, site).expect("a sink of the wiring");
        let member = CgpGraphBackendDef::new_member(id, source, field);
        def.output_sinks.0[sink].inputs.0.push(member);
    }

    // Act
    let births = def.birth_coordinates(id, &refs, &context);

    // Assert
    assert_eq!(births.len(), 65_536);
    assert!(births.iter().all(|c| c.present));
}

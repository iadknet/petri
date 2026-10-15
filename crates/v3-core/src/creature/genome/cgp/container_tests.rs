use proptest::prelude::*;

use super::*;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{
    compose, detach, GroupId, Membership, Shape, FIRST_ACTION_VOTE_SINK,
};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink};
use crate::creature::genome::HebbianRule;

/// A new def field is a compile error here until it is placed inside or
/// outside genome identity on purpose (I9).
#[test]
fn def_fields_are_exhaustively_accounted_for() {
    let CgpGraphBackendDef {
        birth_seeds: _,
        compute_nodes: _,
        output_sinks: _,
        groups: _,
    } = CgpGraphBackendDef::new_with_fixed_outputs();
    let GraphEdge {
        source: _,
        own_weight: _,
        membership: _,
    } = GraphEdge::new(leaf(0), 0.0);
    let Group {
        shape: _,
        kind: _,
        offset: _,
        scale: _,
    } = Group {
        shape: Shape::Ring { ring_ref: 0 },
        kind: DirectedVoteKind::Move,
        offset: 0,
        scale: 0.0,
    };
    let Membership { group: _ } = Membership {
        group: GroupId::new(0),
    };
}

#[test]
fn a_graph_edge_is_sixteen_bytes() {
    assert_eq!(std::mem::size_of::<GraphEdge>(), 16);
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
    })
}

/// Two compute nodes (node 1 reads node 0) and one wired sink, seeded with
/// one value per edge.
fn seeded() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![
        ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(leaf(0), 0.5), GraphEdge::new(leaf(1), 0.25)],
            plastic(),
        ),
        ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.5)],
            plastic(),
        ),
    ]);
    def.push_edge(
        EdgeSurface::SinkInput(0),
        GraphEdge::new(GraphSource::ComputeNode(1), 1.0),
    );
    def.seed_birth(BirthSeeds::Rows(vec![
        vec![Some(1.0), Some(2.0)],
        vec![Some(3.0)],
    ]));
    def
}

fn rows(def: &CgpGraphBackendDef) -> Vec<Vec<Option<f32>>> {
    def.birth_seeds()
        .and_then(BirthSeeds::rows)
        .expect("seed rows")
        .to_vec()
}

fn compute(node: usize, index: usize) -> EdgeOccurrence {
    EdgeOccurrence::new(EdgeSurface::ComputeInput(node), index)
}

#[test]
fn the_json_and_debug_text_of_a_def_name_own_weight_and_print_groups() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
            None,
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
         [GraphEdge { source: ComputeNode(0), own_weight: 0.5, membership: None }], \
         plasticity: None }], output_sinks: [OutputSink { kind: CustomOutput(0), inputs: \
         [GraphEdge { source: InputLeaf { ref_idx: 1, sub_idx: 0 }, own_weight: 1.0, \
         membership: None }] }], groups: [] }"
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
        ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(leaf(3), 1.0)],
            None,
        ),
    );
    assert_eq!(rows(&inserted)[1], vec![None]);
    assert_eq!(
        inserted.compute_nodes()[2].inputs[0].source(),
        GraphSource::ComputeNode(0)
    );
    inserted.append_compute_node(ComputeNode::new(ComputeNodeKind::Add, Vec::new(), None));
    assert_eq!(rows(&inserted).len(), 4);
    // Duplicate: the copy's row is its original's.
    let mut duplicated = seeded();
    duplicated.duplicate_compute_nodes(&[0]);
    assert_eq!(rows(&duplicated)[1], rows(&duplicated)[0]);
    // Kind and plasticity writes leave the seeds alone.
    let mut rewritten = seeded();
    rewritten.set_compute_kind(0, ComputeNodeKind::Tanh);
    rewritten.set_node_plasticity(1, None);
    assert_eq!(rows(&rewritten), rows(&seeded()));
}

#[test]
fn a_split_keeps_the_consumers_seed_and_starts_the_identity_edge_without_one() {
    let identity = || ComputeNode::new(ComputeNodeKind::Add, Vec::new(), None);
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
    assert_eq!(
        def.compute_nodes()[2].inputs[0],
        GraphEdge::new(GraphSource::ComputeNode(1), 1.5)
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
fn the_empty_form_carries_no_rows_through_any_edit() {
    let mut def = seeded();
    def.seed_birth(BirthSeeds::Empty);
    def.push_edge(EdgeSurface::ComputeInput(0), GraphEdge::new(leaf(2), 1.0));
    def.append_compute_node(ComputeNode::new(ComputeNodeKind::Add, Vec::new(), None));
    def.remove_compute_node(0);
    assert_eq!(def.birth_seeds(), Some(&BirthSeeds::Empty));
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
    def.seed_birth(BirthSeeds::Rows(vec![vec![Some(1.0), Some(2.0)]]));
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
                            ComputeNode::new(
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
        seeded.seed_birth(BirthSeeds::Rows(
            def.compute_nodes().iter().map(|node| vec![Some(1.0); node.inputs.len()]).collect(),
        ));
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

// ── Edge groups (T25.F03) ───────────────────────────────────────────────────

const RING: u16 = 1;

fn ring_refs() -> Vec<InputReference> {
    vec![
        InputReference::ActionVotes,
        InputReference::World(crate::contracts::WorldInputKey::NeighborBarrierRing),
    ]
}

fn ring() -> Shape {
    Shape::Ring { ring_ref: RING }
}

fn vote_sink(kind: DirectedVoteKind, d: u8) -> usize {
    FIRST_ACTION_VOTE_SINK + kind.sink(d).index()
}

fn sink_occurrence(sink: usize, index: usize) -> EdgeOccurrence {
    EdgeOccurrence::new(EdgeSurface::SinkInput(sink), index)
}

/// A full catalog with an independent edge on `Move(0)` and a group.
fn grouped(kind: DirectedVoteKind, offset: u8) -> (CgpGraphBackendDef, GroupId) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.push_edge(
        EdgeSurface::SinkInput(vote_sink(DirectedVoteKind::Move, 0)),
        GraphEdge::new(leaf(0), 0.5),
    );
    let id = def.add_group(ring(), kind, offset);
    (def, id)
}

fn group_strategy() -> impl Strategy<Value = (usize, u8, f32, Vec<f32>)> {
    (
        0usize..3,
        0u8..8,
        -4.0f32..4.0,
        prop::collection::vec(-4.0f32..4.0, 8),
    )
}

/// Set a group's scale and each member's correction (in member order).
fn weigh(def: &mut CgpGraphBackendDef, id: GroupId, scale: f32, corrections: &[f32]) {
    def.group_mut(id).set_scale(scale);
    let members: Vec<_> = def.members(id).collect();
    for (occurrence, &own) in members.into_iter().zip(corrections) {
        def.edge_mut(occurrence).set_weight(own);
    }
}

#[test]
fn a_new_ring_group_is_eight_silent_members_on_its_rotated_sinks() {
    let (def, id) = grouped(DirectedVoteKind::Reproduce, 3);
    assert_eq!(def.groups().len(), 1);
    assert_eq!(def.groups()[0].scale(), 0.0);
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
    assert_eq!(inputs[2].membership(), Some(Membership { group: id }));
}

#[test]
fn remove_group_leaves_no_member_and_renumbers_higher_groups() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let first = def.add_group(ring(), DirectedVoteKind::Move, 0);
    let second = def.add_group(ring(), DirectedVoteKind::StealEnergy, 2);
    let third = def.add_group(ring(), DirectedVoteKind::Move, 5);
    assert_eq!(def.remove_group(second), 8);
    assert_eq!(def.groups().len(), 2);
    assert!(def
        .edges()
        .all(|edge| edge.membership() != Some(Membership { group: third })));
    // The third group is now `GroupId(1)`, its members with it.
    let renumbered = GroupId::new(1);
    assert_eq!(def.groups()[1].offset(), 5);
    assert_eq!(def.members(renumbered).count(), 8);
    assert_eq!(def.members(first).count(), 8);
    assert!(def.check_local_invariants().is_ok());
}

#[test]
fn a_last_member_removed_or_detached_removes_its_group() {
    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    for _ in 0..7 {
        let occurrence = def.members(id).next().unwrap();
        def.remove_edge(occurrence);
        assert_eq!(def.groups().len(), 1);
    }
    let last = def.members(id).next().unwrap();
    let (_, removed) = detach(&mut def, last, &ring_refs());
    assert!(removed);
    assert!(def.groups().is_empty());
    assert!(def.edges().all(|edge| edge.membership().is_none()));

    let (mut def, id) = grouped(DirectedVoteKind::Move, 0);
    for _ in 0..7 {
        let occurrence = def.members(id).next().unwrap();
        def.remove_edge(occurrence);
    }
    let last = def.members(id).next().unwrap();
    def.remove_edge(last);
    assert!(def.groups().is_empty());
}

#[test]
fn detaching_a_later_groups_member_ahead_of_another_member_keeps_the_edit_on_the_detached_edge() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let first = def.add_group(ring(), DirectedVoteKind::Move, 0);
    let second = def.add_group(ring(), DirectedVoteKind::Move, 0);
    weigh(&mut def, second, 0.5, &[0.25; 8]);
    let sink = vote_sink(DirectedVoteKind::Move, 0);
    // The sink holds the first group's member, then the second's.
    let later = sink_occurrence(sink, 1);
    assert_eq!(
        def.edge_at(later).membership(),
        Some(Membership { group: second })
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
        Some(Membership { group: first })
    );
    assert_eq!(def.members(second).count(), 7);
}

#[test]
fn can_host_ring_group_needs_every_direction_sink_and_capacity() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    assert!(def.can_host_ring_group());
    def.groups = vec![
        Group {
            shape: ring(),
            kind: DirectedVoteKind::Move,
            offset: 0,
            scale: 0.0,
        };
        usize::from(u16::MAX)
    ];
    assert!(!def.can_host_ring_group());
    let mut partial = CgpGraphBackendDef::new_with_fixed_outputs();
    crate::creature::genome::cgp::test_support::edit_sinks(&mut partial, |sinks| {
        sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(VoteSink::StealEnergy(7)));
    });
    assert!(!partial.can_host_ring_group());
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
    // I7: the shape reference is not a ring.
    assert_eq!(
        validate_graph(&node(def.clone(), vec![InputReference::ActionVotes; 2])),
        Err(GraphInvariantViolation::ShapeReference)
    );
    // I6: a record without members.
    let mut empty = def.clone();
    empty.groups.push(empty.groups[0]);
    assert_eq!(
        validate_graph(&node(empty, ring_refs())),
        Err(GraphInvariantViolation::EmptyGroup)
    );
    // I7: offset out of range.
    let mut offset = def.clone();
    offset.groups[0].offset = 8;
    assert_eq!(
        offset.check_local_invariants(),
        Err(GraphInvariantViolation::OffsetOutOfRange)
    );
    // I4: a member on the wrong sink.
    let mut shifted = def.clone();
    shifted.groups[0].offset = 2;
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
    let member = *def.edge_at(def.members(GroupId::new(0)).next().unwrap());
    let compute = def.clone().with_compute_nodes(vec![ComputeNode::new(
        ComputeNodeKind::Add,
        vec![member],
        None,
    )]);
    assert_eq!(
        compute.check_local_invariants(),
        Err(GraphInvariantViolation::MemberOnComputeInput)
    );
    // I3: a membership naming no group.
    let mut dangling = def;
    for sink in &mut dangling.output_sinks.0 {
        for edge in &mut sink.inputs.0 {
            if edge.membership.is_some() {
                edge.membership = Some(Membership {
                    group: GroupId::new(4),
                });
            }
        }
    }
    assert_eq!(
        dangling.check_local_invariants(),
        Err(GraphInvariantViolation::GroupIdOutOfRange)
    );
}

#[test]
fn a_grouped_def_round_trips_json_and_identity_both_ways() {
    let (mut def, id) = grouped(DirectedVoteKind::StealEnergy, 6);
    weigh(&mut def, id, -0.5, &[0.125; 8]);
    let json = serde_json::to_string(&def).unwrap();
    assert!(json.contains("\"groups\"") && json.contains("\"membership\""));
    let decoded: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, def);
    assert_eq!(format!("{decoded:?}"), format!("{def:?}"));
    // Every heritable field reaches identity: a changed scale, offset,
    // correction or membership is a different def, in `PartialEq`, `Debug`
    // and JSON alike.
    let mut scaled = def.clone();
    scaled.group_mut(id).set_scale(0.5);
    let mut moved = def.clone();
    moved.relocate_group(id, DirectedVoteKind::StealEnergy, 5);
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
    // An ungrouped def omits both keys and prints `groups: []`.
    let plain = CgpGraphBackendDef::new_with_fixed_outputs();
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("groups") && !json.contains("membership"));
    assert!(format!("{plain:?}").ends_with("groups: [] }"));
}

proptest! {
    /// Detach rewrites a member as the ungrouped edge of its genomic weight,
    /// bitwise, whatever the scale and correction.
    #[test]
    fn detach_preserves_genomic_weight_bitwise(
        (kind, offset, scale, corrections) in group_strategy(),
        pick in 0usize..8,
        resolves in any::<bool>(),
    ) {
        let (mut def, id) = grouped(DirectedVoteKind::ALL[kind], offset);
        weigh(&mut def, id, scale, &corrections);
        let refs = if resolves { ring_refs() } else { vec![InputReference::ActionVotes; 2] };
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
        (kind, offset, scale, corrections) in group_strategy(),
        step in 1u8..8,
        other_kind in 0usize..3,
        second in any::<bool>(),
    ) {
        let (mut def, id) = grouped(DirectedVoteKind::ALL[kind], offset);
        if second {
            def.add_group(ring(), DirectedVoteKind::ALL[other_kind], (offset + 3) % 8);
        }
        weigh(&mut def, id, scale, &corrections);
        let original = def.clone();
        def.relocate_group(id, DirectedVoteKind::ALL[other_kind], (offset + step) % 8);
        prop_assert!(def.check_local_invariants().is_ok());
        def.relocate_group(id, DirectedVoteKind::ALL[kind], offset);
        prop_assert_eq!(&def, &original);
        prop_assert_eq!(format!("{def:?}"), format!("{original:?}"));
    }

    /// A `Ring` member weighs `scale + own` bitwise when its ring resolves
    /// and `own` when it does not.
    #[test]
    fn a_member_weighs_scale_plus_correction_when_its_ring_resolves(
        (kind, offset, scale, corrections) in group_strategy(),
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
            prop_assert_eq!(
                def.genomic_weight(occurrence, &[InputReference::ActionVotes]).to_bits(),
                own.to_bits()
            );
        }
    }
}

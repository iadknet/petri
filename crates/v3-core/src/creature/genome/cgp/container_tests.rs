use proptest::prelude::*;

use super::*;
use crate::creature::genome::HebbianRule;

/// A new def field is a compile error here until it is placed inside or
/// outside genome identity on purpose (I9).
#[test]
fn def_fields_are_exhaustively_accounted_for() {
    let CgpGraphBackendDef {
        birth_seeds: _,
        compute_nodes: _,
        output_sinks: _,
        projections: _,
    } = CgpGraphBackendDef::new_with_fixed_outputs();
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
fn the_json_and_debug_text_of_a_def_are_the_vec_shapes_they_replace() {
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
        r#"{"compute_nodes":[{"kind":"Add","inputs":[{"source":{"ComputeNode":0},"weight":0.5}]}],"output_sinks":[{"kind":{"CustomOutput":0},"inputs":[{"source":{"InputLeaf":{"ref_idx":1,"sub_idx":0}},"weight":1.0}]}]}"#
    );
    assert_eq!(
        format!("{def:?}"),
        "CgpGraphBackendDef { compute_nodes: [ComputeNode { kind: Add, inputs: \
         [GraphEdge { source: ComputeNode(0), weight: 0.5 }], plasticity: None }], \
         output_sinks: [OutputSink { kind: CustomOutput(0), inputs: [GraphEdge { \
         source: InputLeaf { ref_idx: 1, sub_idx: 0 }, weight: 1.0 }] }] }"
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
    assert_eq!(def.edge_at(sink).weight(), 2.0);
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

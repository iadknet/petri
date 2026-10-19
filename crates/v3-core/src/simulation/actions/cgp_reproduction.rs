//! CGP graph backend reproduction helpers: the birth payload's capture.
//!
//! Reproduction captures a parent's learned weights as the child draft's
//! [`BirthSeeds`](crate::creature::genome::cgp::BirthSeeds) before mutation
//! and writes no runtime field; the creature constructors consume the seeds
//! (`creature::newborn`, `runtime::plasticity::LearnedState`).

use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::runtime::plasticity::LearnedState;

/// Seed every graph node of a reproduction draft, a copy of the parent's
/// genome, with the parent's learned values before any structural mutation
/// (T25.F06 invariant 8). A parent backend with nothing initialized seeds
/// nothing; a parent without a plastic edge costs one check.
pub(crate) fn capture_birth_seeds(genome: &mut CreatureGenome, parent: &LearnedState) {
    if !parent.has_learning() {
        return;
    }
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            if let Some(seeds) = parent.capture(idx, def) {
                def.seed_birth(seeds);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::cgp::test_support::{plastic_node, set_surface_plasticity};
    use crate::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
        GraphSource, FIRST_ACTION_VOTE_SINK, FIXED_SINK_COUNT,
    };
    use crate::creature::genome::{HebbianRule, NodeGenome, PlasticityConfig};
    use crate::creature::state::GraphRuntimeState;

    /// The compute seed rows a draft carries.
    fn rows(def: &CgpGraphBackendDef) -> &[Vec<Option<f32>>] {
        def.birth_seeds()
            .expect("the draft carries seed rows")
            .compute_rows()
    }

    fn genome_of(def: &CgpGraphBackendDef) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: Vec::new(),
                backend_def: BackendDef::Graph(def.clone()),
                targets: Vec::new(),
            }],
        }
    }

    fn config(lamarckian: bool, rule: HebbianRule) -> PlasticityConfig {
        PlasticityConfig {
            rule,
            learning_rate: 0.1,
            weight_clamp: 10.0,
            lamarckian,
            modulation: None,
        }
    }

    /// A parent of `def` whose plastic slots, in canonical order, learned
    /// `values` (the backend visited).
    fn parent_with(def: &CgpGraphBackendDef, values: &[f32]) -> GraphRuntimeState {
        let mut parent = GraphRuntimeState::for_genome(&genome_of(def), Vec::new());
        for (slot, &value) in values.iter().enumerate() {
            parent.learned.set_slot(0, slot, value, 0.0);
        }
        parent
    }

    /// The child of `parent` (on `def`) without mutation: its learned value
    /// per plastic slot, `None` where uninitialized.
    fn child_of(def: &CgpGraphBackendDef, parent: &GraphRuntimeState) -> Vec<Option<f32>> {
        let mut draft = genome_of(def);
        capture_birth_seeds(&mut draft, parent.learned());
        let seeds = crate::creature::newborn::take_birth_seeds(&mut draft);
        let child = GraphRuntimeState::for_genome(&draft, seeds);
        child.learned().edges(0).map(|edge| edge.learned).collect()
    }

    fn simple_cgp_def() -> CgpGraphBackendDef {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        // CN0: no plasticity
        def.append_compute_node(ComputeNode::new(ComputeNodeKind::Constant(1.0), Vec::new()));
        // CN1: Lamarckian plasticity
        def.append_compute_node(plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
            Some(config(true, HebbianRule::Classic)),
        ));
        // CN2: Darwinian plasticity
        def.append_compute_node(plastic_node(
            ComputeNodeKind::Sigmoid,
            vec![GraphEdge::new(GraphSource::ComputeNode(1), 0.3)],
            Some(config(false, HebbianRule::Oja)),
        ));
        def
    }

    #[test]
    fn lamarckian_copies_parent_weights_and_darwinian_starts_uninitialized() {
        let def = simple_cgp_def();
        let parent = parent_with(&def, &[0.9, 0.7]);
        assert_eq!(child_of(&def, &parent), vec![Some(0.9), None]);
    }

    #[test]
    fn a_parent_with_nothing_initialized_seeds_nothing() {
        let def = simple_cgp_def();
        let parent = GraphRuntimeState::for_genome(&genome_of(&def), Vec::new());
        let mut draft = genome_of(&def);
        capture_birth_seeds(&mut draft, parent.learned());
        let BackendDef::Graph(captured) = &draft.nodes[0].backend_def else {
            unreachable!()
        };
        assert!(captured.birth_seeds().is_none());
        assert_eq!(child_of(&def, &parent), vec![None, None]);
    }

    #[test]
    fn a_graph_without_plastic_edges_seeds_nothing() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();
        let parent = GraphRuntimeState::for_genome(&genome_of(&def), Vec::new());
        assert!(child_of(&def, &parent).is_empty());
    }

    /// The capture covers both surfaces: a Lamarckian vote edge's learned
    /// weight reaches the child's sink row.
    #[test]
    fn capture_seeds_sink_edges_on_their_rows() {
        let mut def = simple_cgp_def();
        let vote = EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK);
        def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(1), 1.0));
        set_surface_plasticity(&mut def, vote, Some(config(true, HebbianRule::Classic)));
        let parent = parent_with(&def, &[0.9, 0.7, -1.25]);
        let mut draft = genome_of(&def);
        capture_birth_seeds(&mut draft, parent.learned());
        let BackendDef::Graph(captured) = &draft.nodes[0].backend_def else {
            unreachable!()
        };
        let seeds = captured.birth_seeds().expect("seeds");
        assert_eq!(seeds.sink_rows()[FIRST_ACTION_VOTE_SINK], vec![Some(-1.25)]);
        assert_eq!(child_of(&def, &parent), vec![Some(0.9), None, Some(-1.25)]);
    }

    fn tracked_def() -> CgpGraphBackendDef {
        let edge = GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            0.25,
        );
        let node = plastic_node(
            ComputeNodeKind::Add,
            vec![edge; 2],
            Some(config(true, HebbianRule::Classic)),
        );
        let mut def =
            CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![node; 3]);
        def.seed_compute_rows(tracked_parent());
        def
    }

    /// The learned values `tracked_def` carries: edge `e` of node `n` holds
    /// `2n + e + 1`.
    fn tracked_parent() -> Vec<Vec<Option<f32>>> {
        vec![
            vec![Some(1.0), Some(2.0)],
            vec![Some(3.0), Some(4.0)],
            vec![Some(5.0), Some(6.0)],
        ]
    }

    /// `tracked_def` with its edges rewired by `rewire` before the capture,
    /// so the rewired edges carry their parent values.
    fn tracked_def_rewired(rewire: impl FnOnce(&mut CgpGraphBackendDef)) -> CgpGraphBackendDef {
        let mut def = tracked_def();
        def.take_birth_seeds();
        rewire(&mut def);
        def.seed_compute_rows(tracked_parent());
        def
    }

    #[test]
    fn birth_correspondence_composes_insert_copy_remove_and_parallel_edges() {
        let mut def = tracked_def();
        let new = ComputeNode::new(ComputeNodeKind::Add, vec![]);
        def.insert_compute_node(1, new);
        def.duplicate_compute_nodes(&[0, 2]);
        def.remove_compute_node(0);
        let birth = rows(&def).to_vec();
        assert_eq!(birth[0], vec![Some(1.0), Some(2.0)]);
        assert!(birth[1].is_empty());
        assert_eq!(birth[2], vec![Some(3.0), Some(4.0)]);
        assert_eq!(birth[3], vec![Some(3.0), Some(4.0)]);
        assert_eq!(birth[4], vec![Some(5.0), Some(6.0)]);
        let seeds = def.take_birth_seeds();
        let child = GraphRuntimeState::for_genome(&genome_of(&def), vec![seeds]);
        let learned: Vec<_> = child.learned().edges(0).map(|edge| edge.learned).collect();
        assert_eq!(
            learned,
            [1.0, 2.0, 3.0, 4.0, 3.0, 4.0, 5.0, 6.0].map(Some).to_vec()
        );
        assert!(def.birth_seeds().is_none());
    }

    #[test]
    fn birth_source_deletion_and_input_pruning_keep_occurrence_alignment() {
        let mut def = tracked_def_rewired(|def| {
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 0))
                .set_source(GraphSource::ComputeNode(0));
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 1))
                .set_source(GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 2,
                });
        });
        // The edge reading the removed node loses its seed.
        def.remove_compute_node(0);
        assert_eq!(rows(&def)[0], vec![None, Some(4.0)]);
        // Pruning the unaddressed reference 1 renumbers 2 -> 1 and removes
        // nothing, so every seed stays on its edge.
        def.reindex_input_refs_after_removal(1);
        assert_eq!(rows(&def)[0], vec![None, Some(4.0)]);
        assert_eq!(
            def.compute_nodes()[0].inputs[1].source(),
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 2,
            }
        );
    }

    #[test]
    fn birth_split_preserves_consumer_and_starts_identity_without_origin() {
        use crate::mutation::graph::operators::split_existing_edge;
        use rand::SeedableRng;
        let mut def = tracked_def();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(19);
        split_existing_edge(&mut def, &[], &mut rng).unwrap();
        let birth = rows(&def);
        let identity = birth.iter().position(|v| v == &[None]).unwrap();
        assert_eq!(def.compute_nodes()[identity].inputs[0].own_weight(), 1.0);
        assert!(def.compute_nodes()[identity].inputs[0]
            .plasticity()
            .is_none());
        let retained: Vec<_> = birth.iter().flatten().filter_map(|v| *v).collect();
        assert_eq!(retained, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn birth_edge_operators_reset_only_changed_occurrences_and_copy_bundles() {
        use crate::mutation::graph::operators::{
            alter_edge_weight_in_def, copy_edge_bundle, raw_field_mutation, remove_edge,
            retarget_edge,
        };
        use rand::SeedableRng;
        let original = tracked_def();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(71);
        for operation in 0..5 {
            let mut def = original.clone();
            match operation {
                0 => alter_edge_weight_in_def(&mut def, &mut rng).unwrap(),
                1 => retarget_edge(&mut def, &[], &mut rng).unwrap(),
                2 => raw_field_mutation(&mut def, &[], &mut rng).unwrap(),
                3 => remove_edge(&mut def, &mut rng).unwrap(),
                _ => copy_edge_bundle(&mut def, &mut rng).unwrap(),
            }
            let birth = rows(&def);
            for (idx, node) in def.compute_nodes().iter().enumerate() {
                assert_eq!(node.inputs.len(), birth[idx].len());
            }
            match operation {
                0..=2 => {
                    for (n, node) in def.compute_nodes().iter().enumerate() {
                        for (e, edge) in node.inputs.iter().enumerate() {
                            let changed = edge != &original.compute_nodes()[n].inputs[e];
                            assert_eq!(birth[n][e].is_none(), changed);
                        }
                    }
                }
                3 => assert_eq!(birth.iter().map(Vec::len).sum::<usize>(), 5),
                _ => {
                    let copied = birth.iter().find(|v| v.len() == 4).unwrap();
                    assert!(rows(&original).iter().any(|v| v == &copied[2..]));
                }
            }
        }
    }

    /// `from_seeds` reads the child's final flags: a seed on an edge made
    /// Lamarckian after the capture initializes it, an unseeded edge of the
    /// same backend stays uninitialized, a seed on an edge whose plasticity
    /// was removed is ignored, and zero is an available value.
    #[test]
    fn birth_config_eligibility_uses_final_flag_and_zero_is_available() {
        let mut def = tracked_def();
        set_surface_plasticity(
            &mut def,
            EdgeSurface::ComputeInput(0),
            Some(config(false, HebbianRule::Classic)),
        );
        def.take_birth_seeds();
        def.seed_compute_rows(vec![
            vec![Some(0.0), None],
            vec![None, None],
            vec![None, None],
        ]);
        set_surface_plasticity(
            &mut def,
            EdgeSurface::ComputeInput(0),
            Some(config(true, HebbianRule::Classic)),
        );
        set_surface_plasticity(&mut def, EdgeSurface::ComputeInput(2), None);
        let seeds = def.take_birth_seeds();
        let child = GraphRuntimeState::for_genome(&genome_of(&def), vec![seeds]);
        let learned: Vec<_> = child.learned().edges(0).map(|edge| edge.learned).collect();
        assert_eq!(learned, vec![Some(0.0), None, None, None]);
    }

    #[test]
    fn birth_metadata_is_not_genome_identity_or_serialized_state() {
        let mut def = tracked_def();
        let serialized = serde_json::to_string(&def).unwrap();
        let mut plain = def.clone();
        plain.take_birth_seeds();
        let original_debug = format!("{plain:?}");
        assert_eq!(format!("{def:?}"), original_debug);
        assert_eq!(def, plain);
        assert_eq!(serialized, serde_json::to_string(&plain).unwrap());
        let decoded: CgpGraphBackendDef = serde_json::from_str(&serialized).unwrap();
        assert!(decoded.birth_seeds().is_none());
        def.take_birth_seeds();
        assert!(def.birth_seeds().is_none());
        assert_eq!(format!("{def:?}"), original_debug);
    }

    proptest::proptest! {
        #[test]
        fn birth_genome_equality_requires_every_genetic_field(value in -100.0f32..100.0) {
            let original = tracked_def();
            let edge = GraphEdge::new(GraphSource::SharedMemory { slot: 0, previous: false }, value);
            for field in 0..5 {
                let mut changed = original.clone();
                match field {
                    0 => changed.set_compute_kind(0, ComputeNodeKind::Constant(value)),
                    1 => changed.push_edge(EdgeSurface::SinkInput(0), edge),
                    2 => changed.push_edge(EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK), edge),
                    3 => changed.push_edge(EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1), edge),
                    _ => changed.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0)).set_plasticity(None),
                }
                proptest::prop_assert_ne!(&original, &changed);
                proptest::prop_assert_ne!(&changed, &original);
            }
            let mut without_metadata = original.clone();
            without_metadata.take_birth_seeds();
            proptest::prop_assert_eq!(original, without_metadata);
        }

        #[test]
        fn birth_copy_remove_is_an_identity_for_every_occurrence(values in proptest::collection::vec(proptest::collection::vec(-100.0f32..100.0, 0..8), 1..12), choice in 0usize..100) {
            let mut def = tracked_def();
            let template = def.compute_nodes()[0].clone();
            def = def.with_compute_nodes(values.iter().map(|row| {
                ComputeNode::new(template.kind, vec![template.inputs[0]; row.len()])
            }).collect());
            let parent: Vec<Vec<Option<f32>>> = values.iter().map(|row| row.iter().copied().map(Some).collect()).collect();
            def.seed_compute_rows(parent.clone());
            let source = choice % values.len();
            def.duplicate_compute_nodes(&[source]);
            def.remove_compute_node(source);
            proptest::prop_assert_eq!(rows(&def).to_vec(), parent);
        }
    }

    #[test]
    fn birth_append_preserves_existing_dangling_source_indices() {
        use crate::mutation::graph::operators::add_disconnected_node;
        use rand::SeedableRng;
        let mut def = tracked_def_rewired(|def| {
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0))
                .set_source(GraphSource::ComputeNode(3));
        });
        add_disconnected_node(&mut def, &mut rand::rngs::SmallRng::seed_from_u64(11)).unwrap();
        assert_eq!(
            def.compute_nodes()[0].inputs[0].source(),
            GraphSource::ComputeNode(3)
        );
        assert_eq!(rows(&def)[0][0], Some(1.0));
    }

    #[test]
    fn birth_graph_copy_operators_preserve_available_values_and_new_edges_have_none() {
        use crate::mutation::graph::operators::{
            add_edge, copy_cgp_subgraph, copy_compute_node, retarget_edge,
        };
        use rand::SeedableRng;
        let base = tracked_def_rewired(|def| {
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 0))
                .set_source(GraphSource::ComputeNode(0));
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(2), 0))
                .set_source(GraphSource::ComputeNode(1));
        });
        for copy in [copy_compute_node, copy_cgp_subgraph] {
            let mut def = base.clone();
            copy(&mut def, &mut rand::rngs::SmallRng::seed_from_u64(8)).unwrap();
            assert!(def.compute_nodes().len() > 3);
            for row in rows(&def) {
                assert!(rows(&base).contains(row));
            }
        }
        let mut def = tracked_def();
        def = def.with_output_sinks(Vec::new());
        def.seed_compute_rows(tracked_parent());
        for seed in 0..12 {
            add_edge(
                &mut def,
                &[],
                &mut rand::rngs::SmallRng::seed_from_u64(seed),
            )
            .unwrap();
        }
        for row in rows(&def) {
            assert!(row.iter().skip(2).all(Option::is_none));
        }
        let mut draw = tracked_def();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(22);
        retarget_edge(&mut draw, &[], &mut rng).unwrap();
        // Replay the draw against its already selected source: no actual rewiring.
        draw.take_birth_seeds();
        draw.seed_compute_rows(tracked_parent());
        let before = draw.birth_seeds().cloned();
        retarget_edge(&mut draw, &[], &mut rand::rngs::SmallRng::seed_from_u64(22)).unwrap();
        assert_eq!(draw.birth_seeds().cloned(), before);
    }
}

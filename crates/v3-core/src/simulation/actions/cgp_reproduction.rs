//! CGP graph backend reproduction helpers: the birth payload's capture.
//!
//! Reproduction captures a parent's learned weights as the child draft's
//! [`BirthSeeds`] before mutation and writes no runtime field; the creature
//! constructors consume the seeds (`creature::newborn`).

use crate::creature::genome::cgp::{BirthSeeds, CgpGraphBackendDef};
use crate::creature::genome::{BackendDef, CreatureGenome};

/// Seed every graph node of a reproduction draft with the parent's learned
/// values (`parent[node][compute][edge]`), before any structural mutation.
pub(crate) fn capture_birth_seeds(genome: &mut CreatureGenome, parent: &[Vec<Box<[f32]>>]) {
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            capture_def_seeds(def, parent.get(idx).map_or(&[], Vec::as_slice));
        }
    }
}

/// Snapshot values per edge occurrence before mutation, independent of parent
/// flags. A parent with no learned value on any edge gives the empty form.
pub(crate) fn capture_def_seeds(def: &mut CgpGraphBackendDef, parent: &[Box<[f32]>]) {
    let nodes = def.compute_nodes();
    let seeds = if nodes
        .iter()
        .zip(parent)
        .any(|(node, values)| !node.inputs.is_empty() && !values.is_empty())
    {
        BirthSeeds::Rows(
            nodes
                .iter()
                .enumerate()
                .map(|(node, compute)| {
                    let values = parent.get(node).map_or(&[][..], |values| &values[..]);
                    (0..compute.inputs.len())
                        .map(|edge| values.get(edge).copied())
                        .collect()
                })
                .collect(),
        )
    } else {
        BirthSeeds::Empty
    };
    def.seed_birth(seeds);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge, GraphSource,
        FIRST_ACTION_VOTE_SINK, FIXED_SINK_COUNT,
    };
    use crate::creature::genome::{HebbianRule, PlasticityConfig};
    use crate::creature::newborn::take_graph_plasticity_weights;

    /// The seed rows a draft carries.
    fn rows(def: &CgpGraphBackendDef) -> &[Vec<Option<f32>>] {
        def.birth_seeds()
            .and_then(BirthSeeds::rows)
            .expect("the draft carries seed rows")
    }

    #[test]
    fn short_parent_slice_mixes_genomic_defaults_without_cross_talk() {
        let mut def = simple_cgp_def();
        def.push_edge(
            EdgeSurface::ComputeInput(1),
            GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                0.25,
            ),
        );
        let child = build_cgp_child_plasticity_weights(&def, &[Box::new([]), Box::new([0.9])]);
        assert_eq!(&*child[1], &[0.9, 0.25]);
    }

    fn build_cgp_child_plasticity_weights(
        def: &CgpGraphBackendDef,
        parent: &[Box<[f32]>],
    ) -> Vec<Box<[f32]>> {
        let mut child = def.clone();
        capture_def_seeds(&mut child, parent);
        take_graph_plasticity_weights(&mut child)
    }

    fn simple_cgp_def() -> CgpGraphBackendDef {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

        // CN0: no plasticity
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Constant(1.0),
            Vec::new(),
            None,
        ));

        // CN1: Lamarckian plasticity
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        ));

        // CN2: Darwinian plasticity
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Sigmoid,
            vec![GraphEdge::new(GraphSource::ComputeNode(1), 0.3)],
            Some(PlasticityConfig {
                rule: HebbianRule::Oja,
                learning_rate: 0.05,
                weight_clamp: 3.0,
                lamarckian: false,
                modulation: None,
            }),
        ));

        def
    }

    #[test]
    fn lamarckian_copies_parent_weights() {
        let def = simple_cgp_def();
        let parent_weights: Vec<Box<[f32]>> = vec![
            Box::new([]),    // CN0: no plasticity
            Box::new([0.9]), // CN1: learned weight
            Box::new([0.7]), // CN2: learned weight
        ];

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 3);
        // CN0: no plasticity → empty
        assert!(child[0].is_empty());
        // CN1: Lamarckian → copied from parent
        assert_eq!(child[1].len(), 1);
        assert!((child[1][0] - 0.9).abs() < 1e-6);
        // CN2: Darwinian → empty (lazy-init)
        assert!(child[2].is_empty());
    }

    #[test]
    fn lamarckian_with_no_parent_weights() {
        let def = simple_cgp_def();
        let parent_weights: Vec<Box<[f32]>> = Vec::new();

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 3);
        // All empty — parent had no weights yet
        for w in &child {
            assert!(w.is_empty());
        }
    }

    #[test]
    fn empty_graph_produces_empty_weights() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();
        let parent_weights: Vec<Box<[f32]>> = Vec::new();

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);
        assert!(child.is_empty());
    }

    #[test]
    fn darwinian_always_resets() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                0.5,
            )],
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
            }),
        ));

        let parent_weights: Vec<Box<[f32]>> = vec![Box::new([0.9])];
        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 1);
        assert!(child[0].is_empty()); // Darwinian: reset to empty
    }
    fn tracked_def() -> CgpGraphBackendDef {
        let template = simple_cgp_def().compute_nodes()[1].clone();
        let edge = GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            0.25,
        );
        let node = ComputeNode::new(template.kind, vec![edge; 2], template.plasticity);
        let mut def =
            CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![node; 3]);
        capture_def_seeds(&mut def, &tracked_parent());
        def
    }

    /// The learned values `tracked_def` captures: edge `e` of node `n` holds
    /// `2n + e + 1`.
    fn tracked_parent() -> Vec<Box<[f32]>> {
        vec![
            Box::new([1.0, 2.0]),
            Box::new([3.0, 4.0]),
            Box::new([5.0, 6.0]),
        ]
    }

    /// `tracked_def` with its edges rewired by `rewire` before the capture,
    /// so the rewired edges carry their parent values.
    fn tracked_def_rewired(rewire: impl FnOnce(&mut CgpGraphBackendDef)) -> CgpGraphBackendDef {
        let mut def = tracked_def();
        def.take_birth_seeds();
        rewire(&mut def);
        capture_def_seeds(&mut def, &tracked_parent());
        def
    }

    #[test]
    fn birth_correspondence_composes_insert_copy_remove_and_parallel_edges() {
        let mut def = tracked_def();
        let new = ComputeNode::new(ComputeNodeKind::Add, vec![], None);
        def.insert_compute_node(1, new);
        def.duplicate_compute_nodes(&[0, 2]);
        def.remove_compute_node(0);
        let weights = take_graph_plasticity_weights(&mut def);
        assert_eq!(&*weights[0], &[1.0, 2.0]);
        assert!(weights[1].is_empty());
        assert_eq!(&*weights[2], &[3.0, 4.0]);
        assert_eq!(&*weights[3], &[3.0, 4.0]);
        assert_eq!(&*weights[4], &[5.0, 6.0]);
        assert!(def.birth_seeds().is_none());
        let mut independent = weights;
        independent[2][0] = 99.0;
        assert_eq!(independent[3][0], 3.0);
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
        split_existing_edge(&mut def, &mut rng).unwrap();
        let birth = rows(&def);
        let identity = birth.iter().position(|v| v == &[None]).unwrap();
        assert_eq!(def.compute_nodes()[identity].inputs[0].weight(), 1.0);
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
                    assert!(original
                        .birth_seeds()
                        .and_then(BirthSeeds::rows)
                        .unwrap()
                        .iter()
                        .any(|v| v == &copied[2..]));
                }
            }
        }
    }

    #[test]
    fn birth_config_eligibility_uses_final_flag_and_zero_is_available() {
        let mut def = tracked_def();
        def.node_plasticity_mut(0).unwrap().lamarckian = false;
        capture_def_seeds(&mut def, &[Box::new([0.0]), Box::new([])]);
        def.node_plasticity_mut(0).unwrap().lamarckian = true;
        def.set_node_plasticity(2, None);
        let child = take_graph_plasticity_weights(&mut def);
        assert_eq!(&*child[0], &[0.0, 0.25]);
        assert!(child[1].is_empty());
        assert!(child[2].is_empty());
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
        take_graph_plasticity_weights(&mut def);
        assert!(def.birth_seeds().is_none());
        assert_eq!(format!("{def:?}"), original_debug);
    }

    proptest::proptest! {
        #[test]
        fn birth_genome_equality_requires_every_genetic_field(value in -100.0f32..100.0) {
            let original = tracked_def();
            let edge = GraphEdge::new(GraphSource::SharedMemory { slot: 0, previous: false }, value);
            for field in 0..4 {
                let mut changed = original.clone();
                match field {
                    0 => changed.set_compute_kind(0, ComputeNodeKind::Constant(value)),
                    1 => changed.push_edge(EdgeSurface::SinkInput(0), edge),
                    2 => changed.push_edge(EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK), edge),
                    _ => changed.push_edge(EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1), edge),
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
                ComputeNode::new(template.kind, vec![template.inputs[0]; row.len()], template.plasticity)
            }).collect());
            let parent: Vec<Box<[f32]>> = values.iter().map(|row| row.clone().into_boxed_slice()).collect();
            capture_def_seeds(&mut def, &parent);
            let source = choice % values.len();
            def.duplicate_compute_nodes(&[source]);
            def.remove_compute_node(source);
            let child = take_graph_plasticity_weights(&mut def);
            proptest::prop_assert_eq!(child, parent);
            proptest::prop_assert!(def.birth_seeds().is_none());
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
        capture_def_seeds(
            &mut draw,
            &[
                Box::new([1.0, 2.0]),
                Box::new([3.0, 4.0]),
                Box::new([5.0, 6.0]),
            ],
        );
        let before = draw.birth_seeds().cloned();
        retarget_edge(&mut draw, &[], &mut rand::rngs::SmallRng::seed_from_u64(22)).unwrap();
        assert_eq!(draw.birth_seeds().cloned(), before);
    }
    #[test]
    fn birth_without_available_parent_values_carries_the_empty_form() {
        let mut def = tracked_def();
        capture_def_seeds(&mut def, &[]);
        assert_eq!(def.birth_seeds(), Some(&BirthSeeds::Empty));
        capture_def_seeds(&mut def, &[Box::new([])]);
        assert_eq!(def.birth_seeds(), Some(&BirthSeeds::Empty));
    }
}

//! CGP graph backend reproduction helpers.
//!
//! Ports `build_child_plasticity_weights` to work with `CgpGraphBackendDef`
//! (iterates `compute_nodes` instead of `internal_nodes`).

use crate::creature::genome::cgp::CgpGraphBackendDef;

/// Snapshot values per edge occurrence before mutation, independent of parent flags.
pub(crate) fn capture_birth_weights(def: &mut CgpGraphBackendDef, parent: &[Box<[f32]>]) {
    if !def
        .compute_nodes
        .iter()
        .zip(parent)
        .any(|(node, values)| !node.inputs.is_empty() && !values.is_empty())
    {
        def.birth_weights = None;
        return;
    }
    def.birth_weights = Some(
        def.compute_nodes
            .iter()
            .enumerate()
            .map(|(node, compute)| {
                compute
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(edge, _)| {
                        parent
                            .get(node)
                            .and_then(|values| values.get(edge))
                            .copied()
                    })
                    .collect()
            })
            .collect(),
    );
}

/// Consume birth correspondence. All-absent or ordinary nodes remain lazily initialized.
pub(crate) fn build_cgp_child_plasticity_weights(def: &mut CgpGraphBackendDef) -> Vec<Box<[f32]>> {
    let inherited = def.birth_weights.take();
    def.compute_nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let values = inherited.as_ref().and_then(|nodes| nodes.get(idx));
            if !node.plasticity.as_ref().is_some_and(|cfg| cfg.lamarckian)
                || !values.is_some_and(|values| values.iter().any(Option::is_some))
            {
                return Box::default();
            }
            node.inputs
                .iter()
                .enumerate()
                .map(|(edge, genomic)| {
                    values
                        .and_then(|values| values.get(edge))
                        .copied()
                        .flatten()
                        .unwrap_or(genomic.weight)
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::creature::genome::{HebbianRule, PlasticityConfig};

    #[test]
    fn short_parent_slice_mixes_genomic_defaults_without_cross_talk() {
        let mut def = simple_cgp_def();
        def.compute_nodes[1].inputs.push(GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            weight: 0.25,
        });
        let child = build_cgp_child_plasticity_weights(&def, &[Box::new([]), Box::new([0.9])]);
        assert_eq!(&*child[1], &[0.9, 0.25]);
    }

    fn build_cgp_child_plasticity_weights(
        def: &CgpGraphBackendDef,
        parent: &[Box<[f32]>],
    ) -> Vec<Box<[f32]>> {
        let mut child = def.clone();
        capture_birth_weights(&mut child, parent);
        super::build_cgp_child_plasticity_weights(&mut child)
    }

    fn simple_cgp_def() -> CgpGraphBackendDef {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        // CN0: no plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });

        // CN1: Lamarckian plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        });

        // CN2: Darwinian plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Sigmoid,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 0.3,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Oja,
                learning_rate: 0.05,
                weight_clamp: 3.0,
                lamarckian: false,
                modulation: None,
            }),
        });

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
        let config = MutationConfig::default();
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        let parent_weights: Vec<Box<[f32]>> = Vec::new();

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);
        assert!(child.is_empty());
    }

    #[test]
    fn darwinian_always_resets() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
            }),
        });

        let parent_weights: Vec<Box<[f32]>> = vec![Box::new([0.9])];
        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 1);
        assert!(child[0].is_empty()); // Darwinian: reset to empty
    }
    fn tracked_def() -> CgpGraphBackendDef {
        let mut def = simple_cgp_def();
        def.compute_nodes = vec![def.compute_nodes[1].clone(); 3];
        for node in &mut def.compute_nodes {
            node.inputs = vec![
                GraphEdge {
                    source: GraphSource::SharedMemory {
                        slot: 0,
                        previous: false
                    },
                    weight: 0.25
                };
                2
            ];
        }
        capture_birth_weights(
            &mut def,
            &[
                Box::new([1.0, 2.0]),
                Box::new([3.0, 4.0]),
                Box::new([5.0, 6.0]),
            ],
        );
        def
    }

    #[test]
    fn birth_correspondence_composes_insert_copy_remove_and_parallel_edges() {
        let mut def = tracked_def();
        let new = ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![],
            plasticity: None,
        };
        def.insert_compute_node_at(1, new);
        def.duplicate_compute_nodes_in_place(&[0, 2]);
        def.remove_compute_node_at(0);
        let weights = super::build_cgp_child_plasticity_weights(&mut def);
        assert_eq!(&*weights[0], &[1.0, 2.0]);
        assert!(weights[1].is_empty());
        assert_eq!(&*weights[2], &[3.0, 4.0]);
        assert_eq!(&*weights[3], &[3.0, 4.0]);
        assert_eq!(&*weights[4], &[5.0, 6.0]);
        assert!(def.birth_weights.is_none());
        let mut independent = weights;
        independent[2][0] = 99.0;
        assert_eq!(independent[3][0], 3.0);
    }

    #[test]
    fn birth_source_deletion_and_input_pruning_keep_occurrence_alignment() {
        let mut def = tracked_def();
        def.compute_nodes[1].inputs[0].source = GraphSource::ComputeNode(0);
        def.remove_compute_node_at(0);
        assert_eq!(
            def.birth_weights.as_ref().unwrap()[0],
            vec![None, Some(4.0)]
        );
        def.compute_nodes[0].inputs[0].source = GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        };
        def.compute_nodes[0].inputs[1].source = GraphSource::InputLeaf {
            ref_idx: 1,
            sub_idx: 2,
        };
        def.reindex_input_refs_after_removal(0);
        assert_eq!(def.birth_weights.as_ref().unwrap()[0], vec![Some(4.0)]);
        def.clamp_sub_idx_after_swap(0, 2);
        assert!(def.birth_weights.as_ref().unwrap()[0].is_empty());
    }

    #[test]
    fn birth_split_preserves_consumer_and_starts_identity_without_origin() {
        use crate::mutation::graph::operators::split_existing_edge;
        use rand::SeedableRng;
        let mut def = tracked_def();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(19);
        split_existing_edge(&mut def, &[], &mut rng).unwrap();
        let birth = def.birth_weights.as_ref().unwrap();
        let identity = birth.iter().position(|v| v == &[None]).unwrap();
        assert_eq!(def.compute_nodes[identity].inputs[0].weight, 1.0);
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
                1 => retarget_edge(&mut def, &[], &MutationConfig::default(), &mut rng).unwrap(),
                2 => {
                    raw_field_mutation(&mut def, &[], &MutationConfig::default(), &mut rng).unwrap()
                }
                3 => remove_edge(&mut def, &mut rng).unwrap(),
                _ => copy_edge_bundle(&mut def, &mut rng).unwrap(),
            }
            let birth = def.birth_weights.as_ref().unwrap();
            for (idx, node) in def.compute_nodes.iter().enumerate() {
                assert_eq!(node.inputs.len(), birth[idx].len());
            }
            match operation {
                0..=2 => {
                    for (n, node) in def.compute_nodes.iter().enumerate() {
                        for (e, edge) in node.inputs.iter().enumerate() {
                            let changed = edge != &original.compute_nodes[n].inputs[e];
                            assert_eq!(birth[n][e].is_none(), changed);
                        }
                    }
                }
                3 => assert_eq!(birth.iter().map(Vec::len).sum::<usize>(), 5),
                _ => {
                    let copied = birth.iter().find(|v| v.len() == 4).unwrap();
                    assert!(original
                        .birth_weights
                        .as_ref()
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
        def.compute_nodes[0].plasticity.as_mut().unwrap().lamarckian = false;
        capture_birth_weights(&mut def, &[Box::new([0.0]), Box::new([])]);
        def.compute_nodes[0].plasticity.as_mut().unwrap().lamarckian = true;
        def.compute_nodes[2].plasticity = None;
        let child = super::build_cgp_child_plasticity_weights(&mut def);
        assert_eq!(&*child[0], &[0.0, 0.25]);
        assert!(child[1].is_empty());
        assert!(child[2].is_empty());
    }

    #[test]
    fn birth_metadata_is_not_genome_identity_or_serialized_state() {
        let mut def = tracked_def();
        let serialized = serde_json::to_string(&def).unwrap();
        let mut plain = def.clone();
        plain.birth_weights = None;
        let original_debug = format!("{plain:?}");
        assert_eq!(format!("{def:?}"), original_debug);
        assert_eq!(def, plain);
        assert_eq!(serialized, serde_json::to_string(&plain).unwrap());
        let decoded: CgpGraphBackendDef = serde_json::from_str(&serialized).unwrap();
        assert!(decoded.birth_weights.is_none());
        super::build_cgp_child_plasticity_weights(&mut def);
        assert!(def.birth_weights.is_none());
        assert_eq!(format!("{def:?}"), original_debug);
    }

    proptest::proptest! {
        #[test]
        fn birth_genome_equality_requires_every_genetic_field(value in -100.0f32..100.0) {
            let original = tracked_def();
            let edge = GraphEdge {
                source: GraphSource::SharedMemory { slot: 0, previous: false },
                weight: value,
            };
            for field in 0..4 {
                let mut changed = original.clone();
                match field {
                    0 => changed.compute_nodes[0].kind = ComputeNodeKind::Constant(value),
                    1 => changed.output_sinks[0].inputs.push(edge),
                    2 => changed.action_bank[0].gate_inputs.push(edge),
                    _ => changed.execute_gate.inputs.push(edge),
                }
                proptest::prop_assert_ne!(&original, &changed);
                proptest::prop_assert_ne!(&changed, &original);
            }
            let mut without_metadata = original.clone();
            without_metadata.birth_weights = None;
            proptest::prop_assert_eq!(original, without_metadata);
        }

        #[test]
        fn birth_copy_remove_is_an_identity_for_every_occurrence(values in proptest::collection::vec(proptest::collection::vec(-100.0f32..100.0, 0..8), 1..12), choice in 0usize..100) {
            let mut def = tracked_def();
            let template = def.compute_nodes[0].clone();
            def.compute_nodes = values.iter().map(|row| {
                let mut node = template.clone();
                node.inputs = vec![node.inputs[0]; row.len()];
                node
            }).collect();
            let parent: Vec<Box<[f32]>> = values.iter().map(|row| row.clone().into_boxed_slice()).collect();
            capture_birth_weights(&mut def, &parent);
            let source = choice % values.len();
            def.duplicate_compute_nodes_in_place(&[source]);
            def.remove_compute_node_at(source);
            let child = super::build_cgp_child_plasticity_weights(&mut def);
            proptest::prop_assert_eq!(child, parent);
            proptest::prop_assert!(def.birth_weights.is_none());
        }
    }
    #[test]
    fn birth_append_preserves_existing_dangling_source_indices() {
        use crate::mutation::graph::operators::add_disconnected_node;
        use rand::SeedableRng;
        let mut def = tracked_def();
        def.compute_nodes[0].inputs[0].source = GraphSource::ComputeNode(3);
        add_disconnected_node(&mut def, &mut rand::rngs::SmallRng::seed_from_u64(11)).unwrap();
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::ComputeNode(3)
        );
        assert_eq!(def.birth_weights.as_ref().unwrap()[0][0], Some(1.0));
    }
    #[test]
    fn birth_graph_copy_operators_preserve_available_values_and_new_edges_have_none() {
        use crate::mutation::graph::operators::{
            add_edge, copy_cgp_subgraph, copy_compute_node, retarget_edge,
        };
        use rand::SeedableRng;
        let mut base = tracked_def();
        base.compute_nodes[1].inputs[0].source = GraphSource::ComputeNode(0);
        base.compute_nodes[2].inputs[0].source = GraphSource::ComputeNode(1);
        for copy in [copy_compute_node, copy_cgp_subgraph] {
            let mut def = base.clone();
            copy(&mut def, &mut rand::rngs::SmallRng::seed_from_u64(8)).unwrap();
            assert!(def.compute_nodes.len() > 3);
            for row in def.birth_weights.as_ref().unwrap() {
                assert!(base.birth_weights.as_ref().unwrap().contains(row));
            }
        }
        let mut def = tracked_def();
        def.output_sinks.clear();
        def.action_bank.clear();
        for seed in 0..12 {
            add_edge(
                &mut def,
                &[],
                &MutationConfig::default(),
                &mut rand::rngs::SmallRng::seed_from_u64(seed),
            )
            .unwrap();
        }
        for row in def.birth_weights.as_ref().unwrap() {
            assert!(row.iter().skip(2).all(Option::is_none));
        }
        let mut draw = tracked_def();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(22);
        retarget_edge(&mut draw, &[], &MutationConfig::default(), &mut rng).unwrap();
        // Replay the draw against its already selected source: no actual rewiring.
        capture_birth_weights(
            &mut draw,
            &[
                Box::new([1.0, 2.0]),
                Box::new([3.0, 4.0]),
                Box::new([5.0, 6.0]),
            ],
        );
        let before = draw.birth_weights.clone();
        retarget_edge(
            &mut draw,
            &[],
            &MutationConfig::default(),
            &mut rand::rngs::SmallRng::seed_from_u64(22),
        )
        .unwrap();
        assert_eq!(draw.birth_weights, before);
    }
    #[test]
    fn birth_without_available_parent_values_needs_no_tracking() {
        let mut def = tracked_def();
        capture_birth_weights(&mut def, &[]);
        assert!(def.birth_weights.is_none());
        capture_birth_weights(&mut def, &[Box::new([])]);
        assert!(def.birth_weights.is_none());
    }
}

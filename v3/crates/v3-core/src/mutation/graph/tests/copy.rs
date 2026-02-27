use super::*;

// ── GraphCopyInternalNode tests ──

#[test]
fn copy_internal_node_increases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = graph_node_internal_count(&genome);
    let mut r = rng(0);
    GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
    let after = graph_node_internal_count(&genome);
    assert_eq!(after, before + 1);
}

#[test]
fn copy_internal_node_on_empty_returns_no_applicable_target() {
    let mut genome = graph_only_genome(vec![]);
    let mut r = rng(0);
    let result = GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_internal_node_preserves_kind() {
    // The copied node must have the same kind as the source.
    for seed in 0u64..20 {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::Sigmoid,
            inputs: vec![],
            hebbian: None,
        }]);
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert_eq!(g.internal_nodes.len(), 2);
            assert!(
                matches!(g.internal_nodes[1].kind, GraphNodeKind::Sigmoid),
                "copied node must preserve kind"
            );
        }
    }
}

#[test]
fn copy_internal_node_sometimes_copies_edges_sometimes_not() {
    let source_node = GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![GraphInput {
            source_idx: 0,
            weight: 1.0,
        }],
        hebbian: None,
    };
    let mut saw_with_edges = false;
    let mut saw_without_edges = false;
    for seed in 0u64..200 {
        let mut genome = graph_only_genome(vec![source_node.clone()]);
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let copy = &g.internal_nodes[1];
            if copy.inputs.is_empty() {
                saw_without_edges = true;
            } else {
                saw_with_edges = true;
            }
        }
        if saw_with_edges && saw_without_edges {
            break;
        }
    }
    assert!(saw_with_edges, "must sometimes copy edges");
    assert!(saw_without_edges, "must sometimes clear edges");
}

#[test]
fn copy_internal_node_sometimes_adds_backlink_sometimes_not() {
    let source_node = GraphInternalNode {
        kind: GraphNodeKind::Relu,
        inputs: vec![],
        hebbian: None,
    };
    let mut saw_backlink = false;
    let mut saw_no_backlink = false;
    for seed in 0u64..200 {
        let mut genome = graph_only_genome(vec![source_node.clone()]);
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            // Check if any existing node got a new input pointing to the copy (index 1).
            let has_backlink = g.internal_nodes.iter().enumerate().any(|(idx, n)| {
                idx < g.internal_nodes.len() - 1
                    && n.inputs
                        .iter()
                        .any(|e| e.source_idx as usize == g.internal_nodes.len() - 1)
            });
            if has_backlink {
                saw_backlink = true;
            } else {
                saw_no_backlink = true;
            }
        }
        if saw_backlink && saw_no_backlink {
            break;
        }
    }
    assert!(saw_backlink, "must sometimes add backlink");
    assert!(saw_no_backlink, "must sometimes skip backlink");
}

// ── GraphCopySubgraph tests ──

#[test]
fn copy_subgraph_increases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = graph_node_internal_count(&genome);
    let mut r = rng(0);
    GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).unwrap();
    let after = graph_node_internal_count(&genome);
    assert!(after > before, "subgraph copy must add nodes");
}

#[test]
fn copy_subgraph_fewer_than_2_returns_no_applicable_target() {
    let mut genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![],
        hebbian: None,
    }]);
    let mut r = rng(0);
    let result = GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_subgraph_remaps_intra_cluster_edges() {
    // Two nodes with edges between them — clone should remap internal edges.
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            hebbian: None,
        },
    ];
    let mut found_remapped = false;
    for seed in 0u64..100 {
        let mut genome = graph_only_genome(nodes.clone());
        let mut r = rng(seed);
        let result = GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r);
        if result.is_ok() {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // New nodes start at index 2. Check if any cloned node has an edge
                // pointing to another cloned node (index >= 2).
                for node in &g.internal_nodes[2..] {
                    for edge in &node.inputs {
                        if edge.source_idx as usize >= 2 {
                            found_remapped = true;
                        }
                    }
                }
            }
        }
        if found_remapped {
            break;
        }
    }
    assert!(
        found_remapped,
        "subgraph copy must remap intra-cluster edges"
    );
}

#[test]
fn copy_subgraph_preserves_external_edges() {
    // Node 1 has an edge to node 0. If only node 1 is in the cluster,
    // the edge should stay pointing to node 0 (external).
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Relu,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 0.5,
            }],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 1,
                weight: 1.0,
            }],
            hebbian: None,
        },
    ];
    let mut found_external_preserved = false;
    for seed in 0u64..200 {
        let mut genome = graph_only_genome(nodes.clone());
        let mut r = rng(seed);
        if GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).is_ok() {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // Check cloned nodes for edges pointing to external indices (< 3).
                for node in &g.internal_nodes[3..] {
                    for edge in &node.inputs {
                        if (edge.source_idx as usize) < 3 {
                            found_external_preserved = true;
                        }
                    }
                }
            }
        }
        if found_external_preserved {
            break;
        }
    }
    assert!(
        found_external_preserved,
        "subgraph copy must preserve external edge targets"
    );
}

#[test]
fn copy_subgraph_preserves_node_kinds() {
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Sigmoid,
            inputs: vec![],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Tanh,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            hebbian: None,
        },
    ];
    let mut r = rng(0);
    let mut genome = graph_only_genome(nodes);
    GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).unwrap();
    if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        let cloned_kinds: Vec<_> = g.internal_nodes[2..]
            .iter()
            .map(|n| std::mem::discriminant(&n.kind))
            .collect();
        // Cloned kinds must be a subset of {Sigmoid, Tanh}.
        let sigmoid_disc = std::mem::discriminant(&GraphNodeKind::Sigmoid);
        let tanh_disc = std::mem::discriminant(&GraphNodeKind::Tanh);
        for k in &cloned_kinds {
            assert!(
                *k == sigmoid_disc || *k == tanh_disc,
                "cloned node kind must match original"
            );
        }
    }
}

// ── GraphCopyEdgeBundle tests ──

#[test]
fn copy_edge_bundle_copies_all_edges() {
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![
                GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                },
                GraphInput {
                    source_idx: 0,
                    weight: 2.0,
                },
            ],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Relu,
            inputs: vec![],
            hebbian: None,
        },
    ];
    let mut found_copied = false;
    for seed in 0u64..100 {
        let mut genome = graph_only_genome(nodes.clone());
        let mut r = rng(seed);
        if GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r).is_ok() {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // The target node (initially empty Relu at idx 2) should now have edges.
                // Or node 1's edges were copied onto node 0 or 2.
                let total_edges: usize = g.internal_nodes.iter().map(|n| n.inputs.len()).sum();
                if total_edges > 2 {
                    found_copied = true;
                    break;
                }
            }
        }
    }
    assert!(found_copied, "edge bundle must copy edges to target");
}

#[test]
fn copy_edge_bundle_fewer_than_2_returns_no_applicable_target() {
    let mut genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![GraphInput {
            source_idx: 0,
            weight: 1.0,
        }],
        hebbian: None,
    }]);
    let mut r = rng(0);
    let result = GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_edge_bundle_source_no_edges_returns_no_applicable_target() {
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Relu,
            inputs: vec![],
            hebbian: None,
        },
    ];
    let mut all_skip = true;
    for seed in 0u64..100 {
        let mut genome = graph_only_genome(nodes.clone());
        let mut r = rng(seed);
        let result = GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r);
        if result.is_ok() {
            all_skip = false;
            break;
        }
    }
    assert!(
        all_skip,
        "with no edges on any node, all attempts must return NoApplicableTarget"
    );
}

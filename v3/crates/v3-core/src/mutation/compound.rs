use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{
    BackendDef, GraphInput, GraphInternalNode, GraphNodeKind, NodeGenome,
};

/// Number of sub-values for a compound input. Returns 1 for scalar inputs.
///
/// World key widths delegate to `WorldInputKey::compound_width()`.
/// ActionQueue width is config-dependent (`action_queue_cap * 3`).
#[must_use]
pub fn sub_value_count(reference: &InputReference, config: &MutationConfig) -> u16 {
    match reference {
        InputReference::ActionQueue => (config.action_queue_cap as u16) * 3,
        InputReference::World(key) => key.compound_width(),
        _ => 1,
    }
}

/// Create fan-out InputRef leaf nodes in a graph backend for a compound input.
///
/// For each sub-value index `0..count`, adds a `GraphInternalNode` with
/// `InputRef { ref_idx, sub_idx }` and no inputs (leaf node).
///
/// No-op if the node has a VM backend (VMs discover sub-values via ReadInput
/// instruction mutation).
pub(super) fn create_fan_out_nodes(node: &mut NodeGenome, ref_idx: u16, count: u16) {
    if let BackendDef::Graph(ref mut gd) = node.backend_def {
        for sub in 0..count {
            gd.internal_nodes.push(GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx,
                    sub_idx: sub,
                },
                inputs: vec![],
                plasticity: None,
            });
        }
    }
}

/// Create InputRef leaf nodes for a new input reference and optionally wire each
/// into the graph. Each new leaf independently rolls `connect_chance` to decide
/// if it gets a bootstrap edge to a random pre-existing internal node.
///
/// Handles both scalar (count=1) and compound (count>1). No-op for VM backends.
pub fn create_and_connect_input_leaves(
    node: &mut NodeGenome,
    ref_idx: u16,
    count: u16,
    connect_chance: f32,
    rng: &mut impl Rng,
) {
    let first_new_idx = match &node.backend_def {
        BackendDef::Graph(gd) => gd.internal_nodes.len(),
        _ => return,
    };

    create_fan_out_nodes(node, ref_idx, count);

    if first_new_idx == 0 {
        return;
    }

    if let BackendDef::Graph(ref mut gd) = node.backend_def {
        let total = gd.internal_nodes.len();
        debug_assert!(
            total <= u16::MAX as usize,
            "internal_nodes index exceeds u16"
        );
        for new_leaf_idx in first_new_idx..total {
            if rng.gen::<f32>() < connect_chance {
                let target_idx = rng.gen_range(0..first_new_idx);
                gd.internal_nodes[target_idx].inputs.push(GraphInput {
                    source_idx: new_leaf_idx as u16,
                    weight: rng.gen_range(-1.0f32..=1.0),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::contracts::{InputReference, NodeId, WorldInputKey};
    use crate::creature::genome::{
        BackendDef, GraphBackendDef, GraphInput, NodeGenome, VmBackendDef, VmInstruction,
    };
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn sub_value_count_scalar_returns_one() {
        let config = MutationConfig::default();
        assert_eq!(
            sub_value_count(&InputReference::World(WorldInputKey::FoodHere), &config),
            1
        );
        assert_eq!(
            sub_value_count(&InputReference::UpstreamSlot(5), &config),
            1
        );
    }

    #[test]
    fn sub_value_count_action_queue_uses_config_cap() {
        let config = MutationConfig::default(); // action_queue_cap = 4
        assert_eq!(
            sub_value_count(&InputReference::ActionQueue, &config),
            12 // 4 * 3
        );

        let config_8 = MutationConfig {
            action_queue_cap: 8,
            ..MutationConfig::default()
        };
        assert_eq!(
            sub_value_count(&InputReference::ActionQueue, &config_8),
            24 // 8 * 3
        );
    }

    #[test]
    fn sub_value_count_extended_perception_widths() {
        let config = MutationConfig::default();
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::AreaFoodSummary),
                &config
            ),
            7
        );
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::AreaBarrierSummary),
                &config
            ),
            7
        );
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::AreaOccupancySummary),
                &config
            ),
            7
        );
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::NearbyCreatureCore),
                &config
            ),
            16
        );
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::NearbyCreatureVitals),
                &config
            ),
            8
        );
        assert_eq!(
            sub_value_count(
                &InputReference::World(WorldInputKey::NearbyCreatureIdentity),
                &config
            ),
            12
        );
    }

    #[test]
    fn create_fan_out_nodes_adds_input_ref_leaves() {
        let mut node = NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::ActionQueue],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![],
            }),
            targets: vec![],
        };
        create_fan_out_nodes(&mut node, 0, 6);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            assert_eq!(gd.internal_nodes.len(), 6);
            for (i, n) in gd.internal_nodes.iter().enumerate() {
                assert_eq!(
                    n.kind,
                    GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: i as u16
                    }
                );
                assert!(n.inputs.is_empty());
                assert!(n.plasticity.is_none());
            }
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn create_fan_out_nodes_noop_for_vm_backend() {
        let mut node = NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::ActionQueue],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        };
        create_fan_out_nodes(&mut node, 0, 12);
        // VM backend unchanged — fan-out is graph-only
        if let BackendDef::Vm(ref vm) = node.backend_def {
            assert_eq!(vm.program.len(), 1); // only Halt
        } else {
            panic!("expected VM backend");
        }
    }

    // -- Helper for create_and_connect tests --

    fn graph_node_with_preexisting(n_preexisting: usize) -> NodeGenome {
        let mut internal_nodes: Vec<GraphInternalNode> = Vec::new();
        for i in 0..n_preexisting {
            internal_nodes.push(GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: if i > 0 {
                    vec![GraphInput {
                        source_idx: (i - 1) as u16,
                        weight: 1.0,
                    }]
                } else {
                    vec![]
                },
                plasticity: None,
            });
        }
        NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
            backend_def: BackendDef::Graph(GraphBackendDef { internal_nodes }),
            targets: vec![],
        }
    }

    // -- create_and_connect_input_leaves tests --

    #[test]
    fn create_and_connect_scalar_creates_one_leaf() {
        let mut node = graph_node_with_preexisting(3);
        let mut rng = SmallRng::seed_from_u64(0);
        create_and_connect_input_leaves(&mut node, 1, 1, 0.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            assert_eq!(gd.internal_nodes.len(), 4); // 3 pre-existing + 1 new
            assert_eq!(
                gd.internal_nodes[3].kind,
                GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 0
                }
            );
            // chance=0.0 -> no new edges on pre-existing nodes
            let total_inputs: usize = gd.internal_nodes[..3].iter().map(|n| n.inputs.len()).sum();
            // pre-existing nodes had: 0 + 1 + 1 = 2 inputs
            assert_eq!(total_inputs, 2);
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn create_and_connect_compound_creates_n_leaves() {
        let mut node = graph_node_with_preexisting(2);
        let mut rng = SmallRng::seed_from_u64(42);
        create_and_connect_input_leaves(&mut node, 0, 8, 0.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            assert_eq!(gd.internal_nodes.len(), 10); // 2 + 8
            for i in 0..8 {
                assert_eq!(
                    gd.internal_nodes[2 + i].kind,
                    GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: i as u16
                    }
                );
            }
            // No edges added (chance=0.0)
            let total_inputs: usize = gd.internal_nodes[..2].iter().map(|n| n.inputs.len()).sum();
            assert_eq!(total_inputs, 1); // only the chain edge from node[1]->node[0]
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn create_and_connect_all_leaves_when_chance_one() {
        let mut node = graph_node_with_preexisting(3);
        let mut rng = SmallRng::seed_from_u64(7);
        create_and_connect_input_leaves(&mut node, 0, 8, 1.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            // 3 pre-existing + 8 new leaves
            assert_eq!(gd.internal_nodes.len(), 11);
            // Each new leaf should have exactly one edge pointing to it
            // from some pre-existing node.
            for new_idx in 3..11u16 {
                let referencing_count: usize = gd.internal_nodes[..3]
                    .iter()
                    .flat_map(|n| &n.inputs)
                    .filter(|inp| inp.source_idx == new_idx)
                    .count();
                assert_eq!(
                    referencing_count, 1,
                    "new leaf {} should be referenced by exactly one edge",
                    new_idx
                );
            }
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn create_and_connect_scalar_bootstrap_edge() {
        let mut node = graph_node_with_preexisting(2);
        let mut rng = SmallRng::seed_from_u64(99);
        create_and_connect_input_leaves(&mut node, 1, 1, 1.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            assert_eq!(gd.internal_nodes.len(), 3);
            // The new leaf at index 2 should be sourced by exactly one edge
            let edge_count: usize = gd.internal_nodes[..2]
                .iter()
                .flat_map(|n| &n.inputs)
                .filter(|inp| inp.source_idx == 2)
                .count();
            assert_eq!(edge_count, 1);
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn create_and_connect_no_edge_when_chance_zero() {
        let mut node = graph_node_with_preexisting(5);
        let pre_edge_count: usize = if let BackendDef::Graph(ref gd) = node.backend_def {
            gd.internal_nodes.iter().map(|n| n.inputs.len()).sum()
        } else {
            panic!()
        };
        let mut rng = SmallRng::seed_from_u64(0);
        create_and_connect_input_leaves(&mut node, 0, 4, 0.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            let post_edge_count: usize = gd.internal_nodes.iter().map(|n| n.inputs.len()).sum();
            assert_eq!(post_edge_count, pre_edge_count, "no edges should be added");
        }
    }

    #[test]
    fn create_and_connect_no_edge_when_no_preexisting() {
        let mut node = NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![],
            }),
            targets: vec![],
        };
        let mut rng = SmallRng::seed_from_u64(42);
        create_and_connect_input_leaves(&mut node, 0, 4, 1.0, &mut rng);
        if let BackendDef::Graph(ref gd) = node.backend_def {
            assert_eq!(gd.internal_nodes.len(), 4);
            // No pre-existing nodes means no edges possible
            let total_edges: usize = gd.internal_nodes.iter().map(|n| n.inputs.len()).sum();
            assert_eq!(total_edges, 0);
        }
    }

    #[test]
    fn create_and_connect_noop_for_vm() {
        let mut node = NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::ActionQueue],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        };
        let mut rng = SmallRng::seed_from_u64(0);
        create_and_connect_input_leaves(&mut node, 0, 8, 1.0, &mut rng);
        if let BackendDef::Vm(ref vm) = node.backend_def {
            assert_eq!(vm.program.len(), 1);
        } else {
            panic!("expected VM backend");
        }
    }
}

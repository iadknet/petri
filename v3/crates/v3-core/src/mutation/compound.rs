use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{BackendDef, GraphInternalNode, GraphNodeKind, NodeGenome};

/// Number of sub-values for a compound input. Returns 1 for scalar inputs.
#[must_use]
pub fn sub_value_count(reference: &InputReference, config: &MutationConfig) -> u16 {
    match reference {
        InputReference::ActionQueue => (config.action_queue_cap as u16) * 3,
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
pub fn create_fan_out_nodes(node: &mut NodeGenome, ref_idx: u16, count: u16) {
    if let BackendDef::Graph(ref mut gd) = node.backend_def {
        for sub in 0..count {
            gd.internal_nodes.push(GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx,
                    sub_idx: sub,
                },
                inputs: vec![],
                hebbian: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::contracts::{InputReference, NodeId, WorldInputKey};
    use crate::creature::genome::{
        BackendDef, GraphBackendDef, NodeGenome, VmBackendDef, VmInstruction,
    };

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
                assert!(n.hebbian.is_none());
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
}

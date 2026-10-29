//! The genome's mesh edits (T29.F01): node removal with successor
//! rewiring, a node copy attached to a predecessor, a slice clone attached
//! through a free slot, and a node append. Mutation draws every choice and
//! allocates every new id; these operations only write the mesh.

use std::collections::HashMap;

use crate::contracts::{NodeId, RouteTarget};
use crate::genome::learning::{GraphMapping, InheritanceMapping, NodeInheritance};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome};

/// The route a copied node or slice is attached through: node `from` gains a
/// target at `slot` with `gate_bias`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attachment {
    pub from: usize,
    pub slot: u8,
    pub gate_bias: f32,
}

impl CreatureGenome {
    /// Remove node `idx`: every target naming it is pointed at `successor`,
    /// or dropped when there is none. Its learnables drop and every later
    /// node keeps its own at the index one lower.
    pub fn remove_node(&mut self, idx: usize, successor: Option<NodeId>) -> InheritanceMapping {
        let removed = self.nodes.remove(idx).node_id;
        for node in &mut self.nodes {
            match successor {
                Some(successor) => {
                    for target in &mut node.targets {
                        if target.target_id == removed {
                            target.target_id = successor;
                        }
                    }
                }
                None => node.targets.retain(|target| target.target_id != removed),
            }
        }
        InheritanceMapping::Nodes(
            (0..self.nodes.len())
                .map(|post| NodeInheritance::From {
                    node: if post < idx { post } else { post + 1 },
                    graph: GraphMapping::Identity,
                })
                .collect(),
        )
    }

    /// Append a copy of node `source` named `new_id` (its targets naming
    /// itself renamed with it), running `backend` instead of the original's
    /// when given, and attach it: node `attach.from` gains a target naming
    /// the copy. The copy's learnables copy the original's; a replaced
    /// backend inherits nothing.
    pub fn copy_node_attached(
        &mut self,
        source: usize,
        new_id: NodeId,
        backend: Option<BackendDef>,
        attach: Attachment,
    ) -> InheritanceMapping {
        let inherits = if backend.is_some() {
            NodeInheritance::Fresh
        } else {
            NodeInheritance::From {
                node: source,
                graph: GraphMapping::Copied,
            }
        };
        let original = &self.nodes[source];
        let old_id = original.node_id;
        let targets = original
            .targets
            .iter()
            .map(|&target| RouteTarget {
                target_id: if target.target_id == old_id {
                    new_id
                } else {
                    target.target_id
                },
                ..target
            })
            .collect();
        let copy = NodeGenome::new(
            new_id,
            original.input_refs().to_vec(),
            backend.unwrap_or_else(|| original.backend_def().clone()),
            targets,
        );
        self.nodes[attach.from].targets.push(RouteTarget {
            target_id: new_id,
            slot: attach.slot,
            gate_bias: attach.gate_bias,
        });
        self.nodes.push(copy);
        let mut nodes = InheritanceMapping::kept(self.nodes.len() - 1);
        nodes.push(inherits);
        InheritanceMapping::Nodes(nodes)
    }

    /// Append copies of the nodes at `indices`, the `i`-th named
    /// `new_ids[i]`; a target naming a node of the slice names its copy, any
    /// other target is kept. Node `from` then gains a target at `slot`, gate
    /// bias 0, naming copy `link`. Each copy's learnables copy its
    /// original's.
    pub fn clone_slice(
        &mut self,
        indices: &[usize],
        new_ids: &[NodeId],
        from: usize,
        slot: u8,
        link: usize,
    ) -> InheritanceMapping {
        let mut nodes = InheritanceMapping::kept(self.nodes.len());
        nodes.extend(indices.iter().map(|&node| NodeInheritance::From {
            node,
            graph: GraphMapping::Copied,
        }));
        debug_assert_eq!(indices.len(), new_ids.len(), "one new id per copy");
        let id_map: HashMap<NodeId, NodeId> = indices
            .iter()
            .map(|&idx| self.nodes[idx].node_id)
            .zip(new_ids.iter().copied())
            .collect();
        let copies: Vec<NodeGenome> = indices
            .iter()
            .map(|&idx| {
                let original = &self.nodes[idx];
                let targets = original
                    .targets
                    .iter()
                    .map(|&target| RouteTarget {
                        target_id: id_map
                            .get(&target.target_id)
                            .copied()
                            .unwrap_or(target.target_id),
                        ..target
                    })
                    .collect();
                NodeGenome::new(
                    id_map[&original.node_id],
                    original.input_refs().to_vec(),
                    original.backend_def().clone(),
                    targets,
                )
            })
            .collect();
        self.nodes.extend(copies);
        self.nodes[from].targets.push(RouteTarget {
            target_id: new_ids[link],
            slot,
            gate_bias: 0.0,
        });
        InheritanceMapping::Nodes(nodes)
    }

    /// Append `node`, a node new to the genome: it inherits nothing.
    pub fn append_node(&mut self, node: NodeGenome) -> InheritanceMapping {
        let mut nodes = InheritanceMapping::kept(self.nodes.len());
        nodes.push(NodeInheritance::Fresh);
        self.nodes.push(node);
        InheritanceMapping::Nodes(nodes)
    }
}

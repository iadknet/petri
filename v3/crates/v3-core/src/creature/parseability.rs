use std::collections::HashSet;

use crate::creature::genome::CreatureGenome;

/// Errors detected by the parseability gate.
#[derive(Debug, PartialEq)]
pub enum ParseabilityError {
    /// The genome has no nodes; cannot execute.
    EmptyNodeList,
    /// Two or more nodes share the same NodeId.
    DuplicateNodeId(u32),
}

/// Lightweight structural validator applied at genome load time.
///
/// Does NOT check: entry_node_id resolution, route target resolution, or
/// behavioral viability — those are handled by runtime soft defaults.
pub struct ParseabilityGate;

impl ParseabilityGate {
    /// Validate the genome's structural invariants.
    pub fn validate(genome: &CreatureGenome) -> Result<(), ParseabilityError> {
        if genome.nodes.is_empty() {
            return Err(ParseabilityError::EmptyNodeList);
        }
        let mut seen: HashSet<u32> = HashSet::new();
        for node in &genome.nodes {
            if !seen.insert(node.node_id.0) {
                return Err(ParseabilityError::DuplicateNodeId(node.node_id.0));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

    fn make_minimal_node(id: u32) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(id),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }
    }

    #[test]
    fn founder_genome_passes_parseability() {
        let genome = crate::creature::founder::v3alpha1_founder_genome();
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }

    #[test]
    fn empty_node_list_fails() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(
            ParseabilityGate::validate(&genome),
            Err(ParseabilityError::EmptyNodeList)
        );
    }

    #[test]
    fn duplicate_node_ids_fail() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![make_minimal_node(0), make_minimal_node(0)],
        };
        assert_eq!(
            ParseabilityGate::validate(&genome),
            Err(ParseabilityError::DuplicateNodeId(0))
        );
    }

    #[test]
    fn unresolved_entry_node_passes_parseability() {
        // entry_node_id resolution is NOT a parseability requirement
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(99),
            nodes: vec![make_minimal_node(0)],
        };
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }

    #[test]
    fn dangling_route_targets_pass_parseability() {
        // targets may reference non-existent nodes (junk DNA)
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(999)], // dangling
            }],
        };
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }
}

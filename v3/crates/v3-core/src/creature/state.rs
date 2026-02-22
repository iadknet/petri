use std::collections::HashMap;

use crate::contracts::{CreatureId, NodeId, Position};
use crate::creature::genome::CreatureGenome;

/// Full runtime state of a creature in the simulation.
pub struct CreatureState {
    pub id: CreatureId,
    pub genome: CreatureGenome,
    pub position: Position,
    pub energy: f32,
    pub age: u64,
    pub generation: u64,
    /// 1024-byte persistent memory, copied on reproduction.
    pub memory: [u8; 1024],
    /// Per-node stateful operator state for the Graph backend.
    /// Keyed by NodeId; lazily initialized on first access.
    pub graph_state: HashMap<NodeId, Vec<f32>>,
    /// RGB phenotype color. Channel values in [0, 255].
    pub phenotype_rgb: [u8; 3],
    /// Per-channel weights for phenotype mutation (internal, not API-exposed).
    pub phenotype_channel_weights: [f32; 3],
    /// Per-channel polarity flags for phenotype mutation (internal, not API-exposed).
    pub phenotype_channel_polarity: [bool; 3],
}

impl CreatureState {
    /// Create a new creature with zeroed memory and empty graph state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CreatureId,
        genome: CreatureGenome,
        position: Position,
        energy: f32,
        generation: u64,
        phenotype_rgb: [u8; 3],
        phenotype_channel_weights: [f32; 3],
        phenotype_channel_polarity: [bool; 3],
    ) -> Self {
        Self {
            id,
            genome,
            position,
            energy,
            age: 0,
            generation,
            memory: [0u8; 1024],
            graph_state: HashMap::new(),
            phenotype_rgb,
            phenotype_channel_weights,
            phenotype_channel_polarity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use slotmap::SlotMap;

    fn minimal_genome() -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        }
    }

    #[test]
    fn new_creature_has_zero_age_and_empty_memory() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(5, 5),
            20.0,
            0,
            [128, 64, 32],
            [1.0f32; 3],
            [true; 3],
        );
        assert_eq!(state.age, 0);
        assert_eq!(state.memory, [0u8; 1024]);
        assert!(state.graph_state.is_empty());
        assert_eq!(state.generation, 0);
        assert!((state.energy - 20.0).abs() < f32::EPSILON);
        assert_eq!(state.phenotype_rgb, [128, 64, 32]);
        assert_eq!(state.phenotype_channel_weights, [1.0f32; 3]);
        assert_eq!(state.phenotype_channel_polarity, [true; 3]);
    }

    #[test]
    fn creature_state_position_stored() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(3, 7),
            50.0,
            2,
            [0, 0, 0],
            [1.0f32; 3],
            [true; 3],
        );
        assert_eq!(state.position, Position::new(3, 7));
        assert_eq!(state.generation, 2);
    }
}

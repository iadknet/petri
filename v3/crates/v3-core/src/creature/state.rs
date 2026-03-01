use crate::contracts::{CreatureId, Position};
use crate::creature::genome::CreatureGenome;
use crate::creature::identity::CreatureIdentityState;

/// Per-creature runtime state for Graph backends.
/// Groups all mutable state that graph evaluation reads/writes.
pub struct GraphRuntimeState {
    /// Per-node stateful operator state. Indexed [mesh_node_idx][internal_node_idx].
    pub node_state: Vec<Vec<f32>>,
    /// Per-edge learned Hebbian weights. Indexed [mesh_node_idx][internal_node_idx][edge_idx].
    /// Empty inner vec = use genome weights. Lazily initialized on first Hebbian evaluation.
    /// Uses `Box<[f32]>` since edge count per node is fixed after init.
    pub hebbian_weights: Vec<Vec<Box<[f32]>>>,
    /// Scratch: prev_outputs buffer reused across graph evaluations.
    pub(crate) scratch_prev: Vec<f32>,
    /// Scratch: curr_outputs buffer reused across graph evaluations.
    pub(crate) scratch_curr: Vec<f32>,
    /// Scratch: state backup for energy-exhaustion rollback.
    pub(crate) scratch_backup: Vec<f32>,
    /// Scratch: weighted-inputs buffer reused across graph evaluations.
    pub(crate) scratch_w_inputs: Vec<f32>,
}

impl GraphRuntimeState {
    /// Create a new empty graph runtime state.
    pub fn new() -> Self {
        Self {
            node_state: Vec::new(),
            hebbian_weights: Vec::new(),
            scratch_prev: Vec::new(),
            scratch_curr: Vec::new(),
            scratch_backup: Vec::new(),
            scratch_w_inputs: Vec::new(),
        }
    }
}

impl Default for GraphRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Per-node runtime state for Graph backends (stateful operators + Hebbian weights).
    pub graph_runtime: GraphRuntimeState,
    /// Lifecycle-owned identity state for kin recognition and lineage tracking.
    pub identity: CreatureIdentityState,
    /// 6-channel internal phenotype (HSL-mapped). Converted to RGB for wire format.
    pub phenotype_channels: [u8; 6],
    /// Active channel for phenotype mutation (0..6; internal, not API-exposed).
    pub phenotype_active_channel: usize,
    /// Per-channel polarity flags for phenotype mutation (internal, not API-exposed).
    pub phenotype_channel_polarity: [bool; 6],
}

impl CreatureState {
    /// Create a new creature with zeroed memory and empty graph runtime state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CreatureId,
        genome: CreatureGenome,
        position: Position,
        energy: f32,
        generation: u64,
        phenotype_channels: [u8; 6],
        phenotype_active_channel: usize,
        phenotype_channel_polarity: [bool; 6],
        identity: CreatureIdentityState,
    ) -> Self {
        Self {
            id,
            genome,
            position,
            energy,
            age: 0,
            generation,
            memory: [0u8; 1024],
            graph_runtime: GraphRuntimeState::new(),
            identity,
            phenotype_channels,
            phenotype_active_channel,
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
            [128, 64, 32, 10, 20, 30],
            0,
            [true; 6],
            CreatureIdentityState::default(),
        );
        assert_eq!(state.age, 0);
        assert_eq!(state.memory, [0u8; 1024]);
        assert!(state.graph_runtime.node_state.is_empty());
        assert!(state.graph_runtime.hebbian_weights.is_empty());
        assert_eq!(state.generation, 0);
        assert!((state.energy - 20.0).abs() < f32::EPSILON);
        assert_eq!(state.phenotype_channels, [128, 64, 32, 10, 20, 30]);
        assert_eq!(state.phenotype_active_channel, 0);
        assert_eq!(state.phenotype_channel_polarity, [true; 6]);
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
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
        );
        assert_eq!(state.position, Position::new(3, 7));
        assert_eq!(state.generation, 2);
    }
}

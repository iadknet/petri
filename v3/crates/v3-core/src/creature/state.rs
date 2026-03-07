use std::mem::size_of;

use crate::contracts::{CreatureId, Position};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::CreatureGenome;
use crate::creature::identity::CreatureIdentityState;

/// Number of f32 slots in shared memory, accessible by both VM and Graph backends.
pub const SHARED_MEMORY_SLOTS: usize = 16;

/// Per-creature runtime state for Graph backends.
/// Groups all mutable state that graph evaluation reads/writes.
pub struct GraphRuntimeState {
    /// Per-node stateful operator state. Indexed [mesh_node_idx][internal_node_idx].
    pub node_state: Vec<Vec<f32>>,
    /// Per-edge learned plasticity weights. Indexed [mesh_node_idx][internal_node_idx][edge_idx].
    /// Empty inner vec = use genome weights. Lazily initialized on first plasticity evaluation.
    /// Uses `Box<[f32]>` since edge count per node is fixed after init.
    pub plasticity_weights: Vec<Vec<Box<[f32]>>>,
    /// Per-edge eligibility traces for reward-modulated plasticity.
    /// Indexed [mesh_node_idx][internal_node_idx][edge_idx], parallel to `plasticity_weights`.
    /// Lazily initialized on first reward-modulated evaluation.
    /// Uses `Box<[f32]>` since edge count per node is fixed after init.
    /// Always reset (not inherited) on reproduction.
    pub eligibility_traces: Vec<Vec<Box<[f32]>>>,
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
            plasticity_weights: Vec::new(),
            eligibility_traces: Vec::new(),
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
    /// Shared f32 memory slots, accessible by both VM and Graph backends.
    /// Copied from parent to child on reproduction.
    pub shared_memory: [f32; SHARED_MEMORY_SLOTS],
    /// Snapshot of shared_memory from the previous tick (set at tick start).
    pub prev_shared_memory: [f32; SHARED_MEMORY_SLOTS],
    /// Per-node runtime state for Graph backends (stateful operators + plasticity weights).
    pub graph_runtime: GraphRuntimeState,
    /// Lifecycle-owned identity state for kin recognition and lineage tracking.
    pub identity: CreatureIdentityState,
    /// 6-channel internal phenotype (HSL-mapped). Converted to RGB for wire format.
    pub phenotype_channels: [u8; 6],
    /// Active channel for phenotype mutation (0..6; internal, not API-exposed).
    pub phenotype_active_channel: usize,
    /// Per-channel polarity flags for phenotype mutation (internal, not API-exposed).
    pub phenotype_channel_polarity: [bool; 6],
    /// Functional complexity cached at birth. Genome is immutable after creation,
    /// so this value is always current.
    pub cached_complexity: u32,
    /// Sorted indices of mesh nodes reachable from the entry node, cached at birth.
    /// Used by the mutation engine to bias target selection toward functional structure.
    pub cached_reachable_nodes: Box<[usize]>,
}

impl CreatureState {
    /// Create a new creature with empty graph runtime state and zeroed prev_shared_memory.
    /// Computes `cached_complexity` from the genome.
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
        shared_memory: [f32; SHARED_MEMORY_SLOTS],
    ) -> Self {
        let cached_complexity = genome.complexity();
        let cached_reachable_nodes = mesh_reachable_nodes(&genome).into_boxed_slice();
        Self {
            id,
            genome,
            position,
            energy,
            age: 0,
            generation,
            shared_memory,
            prev_shared_memory: [0.0; SHARED_MEMORY_SLOTS],
            graph_runtime: GraphRuntimeState::new(),
            identity,
            phenotype_channels,
            phenotype_active_channel,
            phenotype_channel_polarity,
            cached_complexity,
            cached_reachable_nodes,
        }
    }

    /// Create a new creature with pre-computed cached fields.
    /// Used in the no-mutation reproduction fast path where the offspring
    /// genome is identical to the parent's.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_cached_fields(
        id: CreatureId,
        genome: CreatureGenome,
        position: Position,
        energy: f32,
        generation: u64,
        phenotype_channels: [u8; 6],
        phenotype_active_channel: usize,
        phenotype_channel_polarity: [bool; 6],
        identity: CreatureIdentityState,
        shared_memory: [f32; SHARED_MEMORY_SLOTS],
        cached_complexity: u32,
        cached_reachable_nodes: Box<[usize]>,
    ) -> Self {
        Self {
            id,
            genome,
            position,
            energy,
            age: 0,
            generation,
            shared_memory,
            prev_shared_memory: [0.0; SHARED_MEMORY_SLOTS],
            graph_runtime: GraphRuntimeState::new(),
            identity,
            phenotype_channels,
            phenotype_active_channel,
            phenotype_channel_polarity,
            cached_complexity,
            cached_reachable_nodes,
        }
    }
}

// Compile-time size assertion: shared_memory is 2x16x4=128 bytes vs old 1024-byte memory.
// Lock in the size reduction and catch future bloat.
const _: () = assert!(size_of::<[f32; SHARED_MEMORY_SLOTS]>() == 64);

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
    fn new_creature_has_zero_age_and_zeroed_shared_memory() {
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
            [0.0; SHARED_MEMORY_SLOTS],
        );
        assert_eq!(state.age, 0);
        assert_eq!(state.shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
        assert_eq!(state.prev_shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
        assert!(state.graph_runtime.node_state.is_empty());
        assert!(state.graph_runtime.plasticity_weights.is_empty());
        assert!(state.graph_runtime.eligibility_traces.is_empty());
        assert_eq!(state.generation, 0);
        assert!((state.energy - 20.0).abs() < f32::EPSILON);
        assert_eq!(state.phenotype_channels, [128, 64, 32, 10, 20, 30]);
        assert_eq!(state.phenotype_active_channel, 0);
        assert_eq!(state.phenotype_channel_polarity, [true; 6]);
    }

    #[test]
    fn new_creature_preserves_passed_shared_memory() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut mem = [0.0f32; SHARED_MEMORY_SLOTS];
        mem[3] = 1.5;
        mem[7] = -2.0;
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(5, 5),
            20.0,
            0,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            mem,
        );
        assert!((state.shared_memory[3] - 1.5).abs() < f32::EPSILON);
        assert!((state.shared_memory[7] - -2.0).abs() < f32::EPSILON);
        // prev is always zeroed for new creatures
        assert_eq!(state.prev_shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
    }

    #[test]
    fn new_creature_populates_cached_reachable_nodes() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        // minimal_genome has 1 node (the entry), which is reachable
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(0, 0),
            20.0,
            0,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; SHARED_MEMORY_SLOTS],
        );
        assert_eq!(&*state.cached_reachable_nodes, &[0]);
    }

    #[test]
    fn new_creature_reachable_excludes_disconnected_nodes() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        // Genome with 2 nodes: entry at index 0, disconnected at index 1
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(0),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![], // no targets => node 1 is unreachable
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![],
                },
            ],
        };
        let state = CreatureState::new(
            id,
            genome,
            Position::new(0, 0),
            20.0,
            0,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; SHARED_MEMORY_SLOTS],
        );
        // Only node 0 should be reachable
        assert_eq!(&*state.cached_reachable_nodes, &[0]);
    }

    #[test]
    fn new_with_cached_fields_preserves_provided_reachable() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let reachable: Box<[usize]> = vec![0, 2, 5].into_boxed_slice();
        let state = CreatureState::new_with_cached_fields(
            id,
            minimal_genome(),
            Position::new(0, 0),
            20.0,
            0,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; SHARED_MEMORY_SLOTS],
            42,
            reachable.clone(),
        );
        assert_eq!(&*state.cached_reachable_nodes, &[0, 2, 5]);
        assert_eq!(state.cached_complexity, 42);
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
            [0.0; SHARED_MEMORY_SLOTS],
        );
        assert_eq!(state.position, Position::new(3, 7));
        assert_eq!(state.generation, 2);
    }
}

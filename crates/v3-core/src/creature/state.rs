use std::collections::BTreeMap;
use std::mem::size_of;

use crate::contracts::{CreatureId, Position, WorldInputKey};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::mesh_annotations::{
    collect_live_vm_instruction_indices, derive_mesh_annotations_with_reachable_indices,
    MeshReadClass,
};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmInstruction};
use crate::creature::identity::CreatureIdentityState;
use crate::mutation::MutationOperator;

/// Number of f32 slots in shared memory, accessible by both VM and Graph backends.
pub const SHARED_MEMORY_SLOTS: usize = 16;

/// Which mesh nodes a creature's brain dispatched, and how recently.
///
/// One entry per mesh node index, holding the creature age at which that node
/// last dispatched (`None` until it dispatches at all). The mesh executor
/// writes one entry per hop; the mutation engine reads the recent window at a
/// birth to bias the offspring's mutation targets toward executed structure
/// (T11.F17). Indices belong to the creature's own genome, so a newborn always
/// starts empty rather than inheriting its parent's record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchRecord {
    last_dispatch_age: Vec<Option<u64>>,
    /// Age of the tick currently being executed, set by `begin_tick`.
    age: u64,
}

impl DispatchRecord {
    /// Adopt the creature's current age for the dispatches of this tick.
    fn begin_tick(&mut self, age: u64) {
        self.age = age;
    }

    /// Record that mesh node `node_idx` dispatched in the current tick.
    pub fn record_dispatch(&mut self, node_idx: usize) {
        if node_idx >= self.last_dispatch_age.len() {
            self.last_dispatch_age.resize(node_idx + 1, None);
        }
        self.last_dispatch_age[node_idx] = Some(self.age);
    }

    /// Sorted ascending mesh node indices dispatched less than `window` ticks
    /// before `age`. An empty record yields an empty set.
    #[must_use]
    pub fn executed_indices(&self, age: u64, window: u64) -> Vec<usize> {
        self.last_dispatch_age
            .iter()
            .enumerate()
            .filter(|(_, last)| {
                last.is_some_and(|last| age.saturating_sub(last) < window && last <= age)
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// Whether any dispatch has ever been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.last_dispatch_age.iter().all(Option::is_none)
    }
}

/// Per-creature runtime state for Graph backends.
/// Groups all mutable state that graph evaluation reads/writes.
#[derive(Clone)]
pub struct GraphRuntimeState {
    /// Per-node stateful operator state. Indexed as `[mesh_node_idx][internal_node_idx]`.
    pub node_state: Vec<Vec<f32>>,
    /// Last successful compute outputs, held when a module is unvisited.
    pub node_outputs: Vec<Vec<f32>>,
    /// Frozen temporal read bases for the current world tick.
    pub(crate) tick_start_state: Vec<Vec<f32>>,
    pub(crate) tick_start_outputs: Vec<Vec<f32>>,
    /// Per-edge learned plasticity weights. Indexed as
    /// `[mesh_node_idx][internal_node_idx][edge_idx]`.
    /// Empty inner vec = use genome weights. Lazily initialized on first plasticity evaluation.
    /// Uses `Box<[f32]>` since edge count per node is fixed after init.
    pub plasticity_weights: Vec<Vec<Box<[f32]>>>,
    /// Per-edge eligibility traces for reward-modulated plasticity.
    /// Indexed as `[mesh_node_idx][internal_node_idx][edge_idx]`, parallel to
    /// `plasticity_weights`.
    /// Lazily initialized on first reward-modulated evaluation.
    /// Uses `Box<[f32]>` since edge count per node is fixed after init.
    /// Always reset (not inherited) on reproduction.
    pub eligibility_traces: Vec<Vec<Box<[f32]>>>,
    /// Decayed credit frozen at the world boundary; visits replace activity from this base.
    pub(crate) tick_start_eligibility_traces: Vec<Vec<Box<[f32]>>>,
    /// Scratch: prev_outputs buffer reused across graph evaluations.
    pub(crate) scratch_prev: Vec<f32>,
    /// Scratch: curr_outputs buffer reused across graph evaluations.
    pub(crate) scratch_curr: Vec<f32>,
    /// Scratch: candidate operator state, committed only after affordability checks.
    pub(crate) scratch_backup: Vec<f32>,
    /// Scratch: weighted-inputs buffer reused across graph evaluations.
    pub(crate) scratch_w_inputs: Vec<f32>,
    /// Which mesh nodes this creature dispatched, and at what age (T11.F17).
    /// Lives here because it is the per-creature state the mesh loop already
    /// borrows mutably; observation clones therefore never touch the live one.
    pub dispatch_record: DispatchRecord,
}

impl GraphRuntimeState {
    /// Begin one world tick before any mesh visits, using the creature genome
    /// nodes and the creature's current age.
    /// Unvisited graph outputs hold; initialized eligibility decays with elapsed time.
    pub fn begin_tick(&mut self, nodes: &[NodeGenome], age: u64) {
        self.dispatch_record.begin_tick(age);
        self.tick_start_state.clone_from(&self.node_state);
        self.tick_start_outputs.clone_from(&self.node_outputs);
        crate::runtime::plasticity::traces::decay_eligibility_traces(
            nodes,
            &mut self.eligibility_traces,
        );
        self.tick_start_eligibility_traces
            .clone_from(&self.eligibility_traces);
    }

    /// Create a new empty graph runtime state.
    pub fn new() -> Self {
        Self {
            node_state: Vec::new(),
            node_outputs: Vec::new(),
            tick_start_state: Vec::new(),
            tick_start_outputs: Vec::new(),
            plasticity_weights: Vec::new(),
            eligibility_traces: Vec::new(),
            tick_start_eligibility_traces: Vec::new(),
            scratch_prev: Vec::new(),
            scratch_curr: Vec::new(),
            scratch_backup: Vec::new(),
            scratch_w_inputs: Vec::new(),
            dispatch_record: DispatchRecord::default(),
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
    /// Total genome size cached at birth, junk included. Genome is immutable
    /// after creation, so this value is always current. Phase 0 charges the
    /// per-unit carrying cost against it without walking the genome.
    pub cached_genome_size: u32,
    /// Sorted indices of mesh nodes reachable from the entry node, cached at birth.
    /// Used by the mutation engine to bias target selection toward functional structure.
    pub cached_reachable_nodes: Box<[usize]>,
    /// Whether any reachable node has a live barrier read path.
    pub cached_has_barrier_reader: bool,
    /// Exact world-input keys read by live VM `ReadInput` instructions on reachable nodes.
    pub cached_live_vm_world_inputs: Box<[(WorldInputKey, u16)]>,
    /// Mutation operators applied when this creature was born.
    pub birth_mutation_operators: Box<[MutationOperator]>,
    /// Number of offspring this creature has spawned.
    pub offspring_spawned_count: u64,
    /// Lifetime action attempts executed by this creature.
    pub lifetime_action_attempted_count: u64,
    /// Lifetime move actions rejected as blocked.
    pub lifetime_blocked_move_count: u64,
    /// Lifetime reproduction actions rejected as invalid target.
    pub lifetime_invalid_reproduce_count: u64,
    /// Lifetime running sum of energy samples (for mean-lifetime-energy telemetry).
    pub lifetime_energy_sum: f64,
    /// Number of lifetime energy samples accumulated.
    pub lifetime_energy_sample_count: u64,
}

impl CreatureState {
    /// Create a new creature with empty graph runtime state and zeroed prev_shared_memory.
    /// Computes `cached_complexity` and `cached_genome_size` from the genome.
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
        let cached_genome_size = genome.genome_size();
        let cached_reachable_nodes = mesh_reachable_nodes(&genome).into_boxed_slice();
        let cached_has_barrier_reader =
            compute_has_barrier_reader(&genome, cached_reachable_nodes.as_ref());
        let cached_live_vm_world_inputs =
            compute_live_vm_world_inputs(&genome, cached_reachable_nodes.as_ref());
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
            cached_genome_size,
            cached_reachable_nodes,
            cached_has_barrier_reader,
            cached_live_vm_world_inputs,
            birth_mutation_operators: Vec::new().into_boxed_slice(),
            offspring_spawned_count: 0,
            lifetime_action_attempted_count: 0,
            lifetime_blocked_move_count: 0,
            lifetime_invalid_reproduce_count: 0,
            lifetime_energy_sum: 0.0,
            lifetime_energy_sample_count: 0,
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
        cached_genome_size: u32,
        cached_reachable_nodes: Box<[usize]>,
    ) -> Self {
        let cached_has_barrier_reader =
            compute_has_barrier_reader(&genome, cached_reachable_nodes.as_ref());
        let cached_live_vm_world_inputs =
            compute_live_vm_world_inputs(&genome, cached_reachable_nodes.as_ref());
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
            cached_genome_size,
            cached_reachable_nodes,
            cached_has_barrier_reader,
            cached_live_vm_world_inputs,
            birth_mutation_operators: Vec::new().into_boxed_slice(),
            offspring_spawned_count: 0,
            lifetime_action_attempted_count: 0,
            lifetime_blocked_move_count: 0,
            lifetime_invalid_reproduce_count: 0,
            lifetime_energy_sum: 0.0,
            lifetime_energy_sample_count: 0,
        }
    }
}

fn compute_has_barrier_reader(genome: &CreatureGenome, reachable_indices: &[usize]) -> bool {
    derive_mesh_annotations_with_reachable_indices(genome, reachable_indices)
        .iter()
        .any(|node| {
            node.reachable
                && node
                    .read_classes
                    .iter()
                    .any(|class| matches!(class, MeshReadClass::Barrier))
        })
}

fn compute_live_vm_world_inputs(
    genome: &CreatureGenome,
    reachable_indices: &[usize],
) -> Box<[(WorldInputKey, u16)]> {
    let mut counts = BTreeMap::<WorldInputKey, u16>::new();
    for &node_idx in reachable_indices {
        let Some(node) = genome.nodes.get(node_idx) else {
            continue;
        };
        let BackendDef::Vm(vm) = &node.backend_def else {
            continue;
        };
        for instruction_idx in collect_live_vm_instruction_indices(vm) {
            let Some(VmInstruction::ReadInput { ref_idx, .. }) = vm.program.get(instruction_idx)
            else {
                continue;
            };
            let Some(crate::contracts::InputReference::World(key)) =
                node.input_refs.get(*ref_idx as usize)
            else {
                continue;
            };
            *counts.entry(*key).or_insert(0) += 1;
        }
    }
    counts.into_iter().collect::<Vec<_>>().into_boxed_slice()
}

// Compile-time size assertion: shared_memory is 2x16x4=128 bytes vs old 1024-byte memory.
// Lock in the size reduction and catch future bloat.
const _: () = assert!(size_of::<[f32; SHARED_MEMORY_SLOTS]>() == 64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId, WorldInputKey};
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
        assert!(state.graph_runtime.node_outputs.is_empty());
        assert!(state.graph_runtime.tick_start_state.is_empty());
        assert!(state.graph_runtime.tick_start_outputs.is_empty());
        assert!(state.graph_runtime.plasticity_weights.is_empty());
        assert!(state.graph_runtime.eligibility_traces.is_empty());
        assert!(state.graph_runtime.tick_start_eligibility_traces.is_empty());
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
    fn new_creature_starts_with_an_empty_dispatch_record() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
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
        assert!(state.graph_runtime.dispatch_record.is_empty());
        assert!(state
            .graph_runtime
            .dispatch_record
            .executed_indices(0, 100)
            .is_empty());
    }

    #[test]
    fn dispatch_record_keeps_the_latest_age_per_node_within_the_window() {
        let mut record = DispatchRecord::default();
        record.begin_tick(10);
        record.record_dispatch(2);
        record.begin_tick(40);
        record.record_dispatch(0);
        // Node 2 last ran at age 10, node 0 at age 40.
        assert_eq!(record.executed_indices(40, 100), vec![0, 2]);
        assert_eq!(record.executed_indices(40, 31), vec![0, 2]);
        assert_eq!(record.executed_indices(40, 30), vec![0]);
        assert_eq!(record.executed_indices(140, 100), Vec::<usize>::new());
        assert!(!record.is_empty());
    }

    proptest::proptest! {
        /// The window is sorted, monotone in the window length, and, from the
        /// last recorded dispatch on, only ever loses nodes as the creature ages.
        #[test]
        fn dispatch_window_membership_is_monotone_in_age_and_window(
            dispatches in proptest::collection::vec((0usize..8, 0u64..200), 0..12),
            elapsed in 0u64..400,
            window in 1u64..150,
            growth in 0u64..400,
        ) {
            let mut record = DispatchRecord::default();
            let mut newest = 0;
            for (node, at) in dispatches {
                record.begin_tick(at);
                record.record_dispatch(node);
                newest = newest.max(at);
            }
            let age = newest + elapsed;
            let now = record.executed_indices(age, window);
            let later = record.executed_indices(age + growth, window);
            let wider = record.executed_indices(age, window + growth);
            proptest::prop_assert!(now.windows(2).all(|pair| pair[0] < pair[1]));
            for index in &later {
                proptest::prop_assert!(
                    now.contains(index),
                    "ageing past a dispatch never brings it back into the window"
                );
            }
            for index in &now {
                proptest::prop_assert!(
                    wider.contains(index),
                    "a wider window keeps every member of a narrower one"
                );
            }
        }
    }

    #[test]
    fn dispatch_record_ignores_dispatches_recorded_after_the_queried_age() {
        let mut record = DispatchRecord::default();
        record.begin_tick(50);
        record.record_dispatch(1);
        assert_eq!(record.executed_indices(50, 100), vec![1]);
        assert_eq!(record.executed_indices(49, 100), Vec::<usize>::new());
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
            7,
            reachable.clone(),
        );
        assert_eq!(&*state.cached_reachable_nodes, &[0, 2, 5]);
        assert_eq!(state.cached_complexity, 42);
        assert_eq!(state.cached_genome_size, 7);
    }

    #[test]
    fn a_new_creature_caches_its_own_genome_size() {
        use crate::config::FounderProfile;
        use crate::creature::founder::founder_genome;

        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let genomes = [
            minimal_genome(),
            founder_genome(FounderProfile::V3Alpha1),
            founder_genome(FounderProfile::ForageFirstSparse),
            founder_genome(FounderProfile::ForageFirstSparseConservative),
            founder_genome(FounderProfile::ForageFirstSparseRichOffspring),
            founder_genome(FounderProfile::ForageFirstSparseBalanced),
        ];
        for genome in genomes {
            let id = sm.insert(());
            let expected = genome.genome_size();
            assert!(expected > 0, "every fixture genome has a nonzero size");
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
            assert_eq!(state.cached_genome_size, expected);
            assert_eq!(state.cached_genome_size, state.genome.genome_size());
        }
    }

    /// The carrying charge's founder sizing is read from this number: the spec's
    /// 96 is stale (it predates the founder's current program), so the founder
    /// pays `111 * genome_carry_cost_per_unit` per tick.
    #[test]
    fn the_canonical_founder_genome_is_one_hundred_eleven_units() {
        use crate::config::FounderProfile;
        use crate::creature::founder::founder_genome;

        let genome = founder_genome(FounderProfile::V3Alpha1);
        assert_eq!(genome.genome_size(), 111);
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

    #[test]
    fn new_creature_caches_live_vm_world_inputs() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(0),
                    input_refs: vec![
                        InputReference::World(WorldInputKey::AreaFoodSummary {
                            type_idx: crate::config::OrdinaryFoodTypeId::default(),
                        }),
                        InputReference::World(WorldInputKey::NeighborBarrierRing),
                        InputReference::DynamicIntrospection(
                            DynamicIntrospectionKey::EnergyCurrent,
                        ),
                    ],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 4,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput {
                                dst: 0,
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            VmInstruction::ReadInput {
                                dst: 1,
                                ref_idx: 1,
                                sub_idx: 0,
                            },
                            VmInstruction::ReadInput {
                                dst: 2,
                                ref_idx: 0,
                                sub_idx: 3,
                            },
                            VmInstruction::WriteInternalPayload {
                                slot_idx: 0,
                                src: 0,
                            },
                            VmInstruction::WriteInternalPayload {
                                slot_idx: 1,
                                src: 1,
                            },
                            VmInstruction::WriteInternalPayload {
                                slot_idx: 2,
                                src: 2,
                            },
                        ],
                    }),
                    targets: vec![],
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![InputReference::World(WorldInputKey::AreaOccupancySummary)],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 2,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput {
                                dst: 0,
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            VmInstruction::WriteInternalPayload {
                                slot_idx: 0,
                                src: 0,
                            },
                        ],
                    }),
                    targets: vec![],
                },
            ],
        };

        let state = CreatureState::new(
            id,
            genome,
            Position::new(0, 0),
            10.0,
            0,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; SHARED_MEMORY_SLOTS],
        );

        let counts: std::collections::HashMap<_, _> =
            state.cached_live_vm_world_inputs.iter().cloned().collect();
        assert_eq!(
            counts.get(&WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default()
            }),
            Some(&2)
        );
        assert_eq!(counts.get(&WorldInputKey::NeighborBarrierRing), Some(&1));
        assert!(!counts.contains_key(&WorldInputKey::AreaOccupancySummary));
    }
}

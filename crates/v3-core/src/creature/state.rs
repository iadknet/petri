use std::mem::size_of;

use crate::config::{EnergyCapacityConfig, RuntimeConfig};
use crate::contracts::OUTCOME_CHANNEL_COUNT;
use crate::contracts::{
    CreatureId, Direction, OrdinaryFoodTypeId, PerceptionRequirements, Position, WorldInputKey,
    SHARED_MEMORY_SLOTS,
};
use crate::creature::action_log::{ActionType, ACTION_TYPE_COUNT};
use crate::creature::body::HeritableBody;
use crate::creature::colour::unit_rgb;
use crate::genome::analysis::mesh_reachable_nodes;
use crate::genome::learning::Seeds;
use crate::genome::sensor_analysis::{
    declared_world_keys, has_barrier_reader, live_vm_world_reads, perception_requirements,
    sensor_census,
};
use crate::genome::CreatureGenome;
use crate::mutation::{BirthEdgeClass, MutationOperator};
use crate::runtime::mesh::execute_creature_mesh;
use crate::runtime::plasticity::{LearningCounts, OutcomeSignalBank};
use crate::runtime::state::GraphRuntimeState;
#[cfg(not(feature = "telemetry-seams"))]
use crate::runtime::trace::domain::{MeshHopTrace, MeshPassTrace};
#[cfg(not(feature = "telemetry-seams"))]
use crate::runtime::traced_mesh::execute_creature_mesh_traced;
#[cfg(feature = "telemetry-seams")]
use crate::runtime::traced_mesh::{execute_creature_mesh_budgeted, RecordLimit, RecordedMesh};
use crate::runtime::types::MeshOutput;
use crate::sensors::perception::SensorSnapshot;
use crate::sensors::reads::WorldKeySet;
use crate::sensors::static_inputs::BodyView;

/// A litter conceived and not yet delivered (T03.F12). The parent paid the
/// charge and the litter's energy at conception; the child is born at the
/// parent's Phase 2 slot on `deliver_tick`, or the litter is lost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PendingBirth {
    /// The direction the parent chose at conception, resolved again from its
    /// position at delivery.
    pub direction: Direction,
    /// The child's starting energy, already debited from the parent.
    pub litter_energy: f32,
    /// First tick on which the birth is delivered.
    pub deliver_tick: u64,
}

/// Full runtime state of a creature in the simulation.
pub struct CreatureState {
    pub id: CreatureId,
    /// Immutable after construction: set with its derived state and runtime
    /// at birth ([`crate::creature::birth::Zygote::finalize`]) and read
    /// through [`Self::genome`].
    genome: CreatureGenome,
    pub position: Position,
    pub energy: f32,
    /// Observation only: first exhausting sink since the last positive recovery.
    pub(crate) pending_death_cause: Option<crate::ledger::DeathCause>,
    pub age: u64,
    pub generation: u64,
    /// Shared f32 memory slots, accessible by both VM and Graph backends.
    /// Copied from parent to child on reproduction.
    pub shared_memory: [f32; SHARED_MEMORY_SLOTS],
    /// Snapshot of shared_memory from the previous tick (set at tick start).
    pub prev_shared_memory: [f32; SHARED_MEMORY_SLOTS],
    /// Per-node runtime state for Graph backends (stateful operators +
    /// plasticity weights), built from the genome; read through
    /// [`Self::graph_runtime`] and run through the brain operations.
    graph_runtime: GraphRuntimeState,
    /// The heritable body: colour phenotype and kin identity (T26.F01).
    pub body: HeritableBody,
    /// What the record derives from `genome` at birth.
    derived: GenomeDerived,
    /// Mutation operators applied when this creature was born.
    pub birth_mutation_operators: Box<[MutationOperator]>,
    /// The class of the vote sink its birth's sole `AddGraphEdge` or
    /// `InputRef.Add` pushed an edge onto (T11.F30); `None` for any other
    /// birth.
    pub birth_edge_class: Option<BirthEdgeClass>,
    /// Number of offspring this creature has spawned.
    pub offspring_spawned_count: u64,
    /// Lifetime action attempts executed by this creature.
    pub lifetime_action_attempted_count: u64,
    /// The same attempts split by `ActionType` discriminant; the array's sum
    /// always equals `lifetime_action_attempted_count` (T14.F07).
    pub lifetime_actions_attempted_by_type: [u64; ACTION_TYPE_COUNT as usize],
    /// Lifetime applied Eat actions indexed by `OrdinaryFoodTypeId`, grown
    /// lazily on the first applied eat of a type; a type index past the end
    /// reads as zero.
    pub lifetime_eats_applied_by_type: Vec<u64>,
    /// Lifetime predation kills this creature made as the attacker.
    pub lifetime_predation_kills_count: u64,
    /// Lifetime predation transfers this creature absorbed and outlived.
    pub lifetime_predation_hits_taken_count: u64,
    /// Lifetime move actions rejected as blocked.
    pub lifetime_blocked_move_count: u64,
    /// Lifetime reproduction actions rejected as invalid target.
    pub lifetime_invalid_reproduce_count: u64,
    /// Lifetime running sum of energy samples (for mean-lifetime-energy telemetry).
    pub lifetime_energy_sum: f64,
    /// Number of lifetime energy samples accumulated.
    pub lifetime_energy_sample_count: u64,
    /// The outcome signal bank of this creature's last full tick, indexed by
    /// `OutcomeChannel` discriminant (T19.F05). Written once per tick in
    /// Phase 2.5; zeros until the first full tick, never inherited.
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
    /// The litter this creature carries (T03.F12); `None` when not
    /// gestating. A gestating creature commits no action.
    pub pending_birth: Option<PendingBirth>,
    /// The world keys this creature's most recent mesh run resolved
    /// (observation only, for the sensor census telemetry); empty at birth.
    pub last_mesh_world_reads: WorldKeySet,
    /// The direction of this creature's last committed `Move` (T11.F30); at
    /// birth the direction from its parent to its cell, at seeding a drawn
    /// direction, `N` at every other construction. The pass end breaks a tie
    /// within a directed kind toward it. Never inherited, never sensed.
    pub heading: Direction,
}

impl CreatureState {
    /// What sensors read of this body this tick, every body rule applied:
    /// the energy sensor denominator ([`HeritableBody::energy_scale`]) and
    /// the seen colour ([`unit_rgb`]).
    #[must_use]
    pub fn sensed_body(&self, capacity: &EnergyCapacityConfig) -> BodyView {
        BodyView {
            position: self.position,
            age: self.age,
            energy: self.energy,
            energy_scale: self.body.energy_scale(capacity),
            colour: unit_rgb(self.body.phenotype.channels),
            kin_tag: self.body.identity.kin_tag,
            previous_outcome: self.previous_outcome,
        }
    }

    /// Count one Phase 2 action attempt in both the lifetime total and its
    /// per-type slot, so the two can never disagree.
    pub(crate) fn record_action_attempt(&mut self, action: ActionType) {
        self.lifetime_action_attempted_count += 1;
        self.lifetime_actions_attempted_by_type[action as usize] += 1;
    }

    /// Count one applied Eat of `type_idx`, growing the by-type vector on
    /// the first eat of a type index past its current end.
    pub(crate) fn record_applied_eat(&mut self, type_idx: OrdinaryFoodTypeId) {
        let index = usize::from(type_idx.get());
        if index >= self.lifetime_eats_applied_by_type.len() {
            self.lifetime_eats_applied_by_type.resize(index + 1, 0);
        }
        self.lifetime_eats_applied_by_type[index] += 1;
    }

    /// The upper energy bound a credit is capped at (T26.F02 body rule):
    /// the body's own `capacity`. Read by the action layer's cap step and the
    /// creature detail response.
    #[must_use]
    pub fn energy_cap(&self) -> f32 {
        self.body.capacity
    }

    /// Observe the already-applied arithmetic without changing the stored energy.
    pub(crate) fn observe_energy(&mut self, before: f32, sink: crate::ledger::DeathCause) -> f64 {
        use crate::ledger::{applied_debit, observe_energy_change};
        observe_energy_change(
            &mut self.pending_death_cause,
            f64::from(before),
            f64::from(self.energy),
            sink,
        );
        applied_debit(before, self.energy)
    }

    /// The only constructor, called only by the birth's finalization
    /// ([`crate::creature::birth::Zygote::finalize`]): a generation-0
    /// creature with every per-life state zeroed, whose derived state and
    /// brain runtime are built from `genome`, the learned state from the
    /// genome and the inherited `seeds` (T30.F08: rebuilt on every birth).
    pub(in crate::creature) fn new(
        id: CreatureId,
        genome: CreatureGenome,
        position: Position,
        energy: f32,
        body: HeritableBody,
        shared_memory: [f32; SHARED_MEMORY_SLOTS],
        seeds: &Seeds,
    ) -> Self {
        let derived = GenomeDerived::of(&genome);
        let graph_runtime = GraphRuntimeState::for_genome(&genome, seeds);
        Self {
            id,
            genome,
            position,
            energy,
            pending_death_cause: None,
            age: 0,
            generation: 0,
            shared_memory,
            prev_shared_memory: [0.0; SHARED_MEMORY_SLOTS],
            graph_runtime,
            body,
            derived,
            last_mesh_world_reads: WorldKeySet::default(),
            birth_mutation_operators: Vec::new().into_boxed_slice(),
            birth_edge_class: None,
            offspring_spawned_count: 0,
            lifetime_action_attempted_count: 0,
            lifetime_actions_attempted_by_type: [0; ACTION_TYPE_COUNT as usize],
            lifetime_eats_applied_by_type: Vec::new(),
            lifetime_predation_kills_count: 0,
            lifetime_predation_hits_taken_count: 0,
            lifetime_blocked_move_count: 0,
            lifetime_invalid_reproduce_count: 0,
            lifetime_energy_sum: 0.0,
            lifetime_energy_sample_count: 0,
            previous_outcome: [0.0; OUTCOME_CHANNEL_COUNT],
            pending_birth: None,
            heading: Direction::N,
        }
    }

    // ── The genome and what is derived from it ──────────────────────────────

    /// The genome this creature was built from; it never changes.
    #[must_use]
    pub fn genome(&self) -> &CreatureGenome {
        &self.genome
    }

    /// Functional complexity of the genome.
    #[must_use]
    pub fn complexity(&self) -> u32 {
        self.derived.complexity
    }

    /// Total genome size, junk included. Phase 0 charges the per-unit
    /// carrying cost against it without walking the genome.
    #[must_use]
    pub fn genome_size(&self) -> u32 {
        self.derived.genome_size
    }

    /// Sorted indices of mesh nodes reachable from the entry node. The
    /// mutation engine biases target selection toward them.
    #[must_use]
    pub fn reachable_nodes(&self) -> &[usize] {
        &self.derived.reachable_nodes
    }

    /// Whether any reachable node has a live barrier read path.
    #[must_use]
    pub fn has_barrier_reader(&self) -> bool {
        self.derived.has_barrier_reader
    }

    /// Exact world-input keys read by live VM `ReadInput` instructions on
    /// reachable nodes, with their read counts.
    #[must_use]
    pub fn live_vm_world_inputs(&self) -> &[(WorldInputKey, u16)] {
        &self.derived.live_vm_world_inputs
    }

    /// The snapshots the brain needs assembled each turn.
    #[must_use]
    pub fn perception_requirements(&self) -> PerceptionRequirements {
        self.derived.perception_requirements
    }

    /// World keys a reachable node's `input_refs` holds.
    #[must_use]
    pub fn declared_world_inputs(&self) -> &WorldKeySet {
        &self.derived.declared_world_inputs
    }

    /// World keys the sensor census wires (live read, wired edge or ring
    /// projection).
    #[must_use]
    pub fn wired_world_inputs(&self) -> &WorldKeySet {
        &self.derived.wired_world_inputs
    }

    // ── The brain ───────────────────────────────────────────────────────────

    /// The brain's runtime state, built for [`Self::genome`].
    #[must_use]
    pub fn graph_runtime(&self) -> &GraphRuntimeState {
        &self.graph_runtime
    }

    /// Per-tick and post-birth runtime state for an in-crate test to arrange
    /// on a constructed creature, whose genome cannot change under it.
    #[cfg(test)]
    pub(crate) fn runtime_mut(&mut self) -> &mut GraphRuntimeState {
        &mut self.graph_runtime
    }

    /// Prepare the brain for a new world tick
    /// ([`GraphRuntimeState::begin_tick`] at this creature's age).
    pub(crate) fn begin_tick(&mut self) {
        self.graph_runtime.begin_tick(self.age);
    }

    /// Run the brain once on `sensors` from this creature's heading
    /// ([`execute_creature_mesh`]), spending its energy and writing its
    /// shared memory and runtime; the output is the runtime's, unchanged.
    pub fn think(&mut self, sensors: &SensorSnapshot, config: &RuntimeConfig) -> MeshOutput {
        execute_creature_mesh(
            &self.genome,
            sensors,
            &mut self.energy,
            &mut self.shared_memory,
            &self.prev_shared_memory,
            &mut self.graph_runtime,
            config,
            self.heading,
        )
    }

    /// [`Self::think`] with trace recording ([`execute_creature_mesh_traced`]).
    #[cfg(not(feature = "telemetry-seams"))]
    pub(crate) fn think_traced(
        &mut self,
        sensors: &SensorSnapshot,
        config: &RuntimeConfig,
    ) -> (MeshOutput, Vec<MeshHopTrace>, Vec<MeshPassTrace>) {
        execute_creature_mesh_traced(
            &self.genome,
            sensors,
            &mut self.energy,
            &mut self.shared_memory,
            &self.prev_shared_memory,
            &mut self.graph_runtime,
            config,
            self.heading,
        )
    }

    /// [`Self::think`] with trace recording under a record budget
    /// ([`execute_creature_mesh_budgeted`]), the `telemetry-seams` build's
    /// form of `think_traced`.
    #[cfg(feature = "telemetry-seams")]
    pub(crate) fn think_budgeted(
        &mut self,
        sensors: &SensorSnapshot,
        config: &RuntimeConfig,
        limit: Option<RecordLimit>,
    ) -> RecordedMesh {
        execute_creature_mesh_budgeted(
            &self.genome,
            sensors,
            &mut self.energy,
            &mut self.shared_memory,
            &self.prev_shared_memory,
            &mut self.graph_runtime,
            config,
            self.heading,
            limit,
        )
    }

    /// Drive the three-factor update of every initialized reward-modulated
    /// edge from `signals`: the learned state's update and change counts.
    pub(crate) fn apply_reward(&mut self, signals: &OutcomeSignalBank) -> LearningCounts {
        self.graph_runtime.learned.apply_reward(signals)
    }
}

/// The eight analyses a creature derives from its genome at birth, computed
/// together by [`GenomeDerived::of`] so they always describe one genome.
struct GenomeDerived {
    complexity: u32,
    genome_size: u32,
    reachable_nodes: Box<[usize]>,
    has_barrier_reader: bool,
    live_vm_world_inputs: Box<[(WorldInputKey, u16)]>,
    perception_requirements: PerceptionRequirements,
    declared_world_inputs: WorldKeySet,
    wired_world_inputs: WorldKeySet,
}

impl GenomeDerived {
    /// Every analysis of `genome`; the reachable set feeds the sensor ones.
    fn of(genome: &CreatureGenome) -> Self {
        let complexity = genome.complexity();
        let genome_size = genome.genome_size();
        let reachable_nodes = mesh_reachable_nodes(genome).into_boxed_slice();
        let reachable = reachable_nodes.as_ref();
        let has_barrier_reader = has_barrier_reader(genome, reachable);
        let live_vm_world_inputs = live_vm_world_reads(genome, reachable);
        let perception_requirements = perception_requirements(genome);
        let declared_world_inputs = declared_world_keys(genome, reachable).into_iter().collect();
        let wired_world_inputs = sensor_census(genome, reachable)
            .world_inputs
            .into_iter()
            .collect();
        Self {
            complexity,
            genome_size,
            reachable_nodes,
            has_barrier_reader,
            live_vm_world_inputs,
            perception_requirements,
            declared_world_inputs,
            wired_world_inputs,
        }
    }
}

// Compile-time size assertion: shared_memory is 2x16x4=128 bytes vs old 1024-byte memory.
// Lock in the size reduction and catch future bloat.
const _: () = assert!(size_of::<[f32; SHARED_MEMORY_SLOTS]>() == 64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId, WorldInputKey};
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::phenotype::Phenotype;
    use crate::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
    use proptest::prelude::*;
    use slotmap::SlotMap;

    /// A body with `channels`, the founder's active channel and polarity, and
    /// the default identity.
    fn body(channels: [u8; 6]) -> HeritableBody {
        HeritableBody {
            phenotype: Phenotype {
                channels,
                ..Phenotype::FOUNDER
            },
            identity: CreatureIdentityState::default(),
            capacity: 200.0,
            steal_share: 0.2,
            give_share: 0.2,
        }
    }

    fn minimal_genome() -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome::new(
                NodeId::new(0),
                vec![],
                BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                vec![],
            )],
        }
    }

    #[test]
    fn new_creature_has_zero_age_and_zeroed_shared_memory() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = crate::creature::birth::Zygote::founder(
            minimal_genome(),
            body([128, 64, 32, 10, 20, 30]),
        )
        .finalize(
            id,
            crate::creature::birth::BirthPlace {
                position: Position::new(5, 5),
                heading: crate::contracts::Direction::N,
                energy: 20.0,
            },
        )
        .creature;
        assert_eq!(state.age, 0);
        assert_eq!(state.shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
        assert_eq!(state.prev_shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
        assert!(state.graph_runtime().node_state.is_empty());
        assert!(state.graph_runtime().node_outputs.is_empty());
        assert!(!state.graph_runtime().learned().has_learning());
        assert_eq!(state.generation, 0);
        assert!((state.energy - 20.0).abs() < f32::EPSILON);
        assert_eq!(state.body.phenotype.channels, [128, 64, 32, 10, 20, 30]);
        assert_eq!(state.body.phenotype.active_channel, 0);
        assert_eq!(state.body.phenotype.polarity, [true; 6]);
    }

    #[test]
    fn new_creature_preserves_passed_shared_memory() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut mem = [0.0f32; SHARED_MEMORY_SLOTS];
        mem[3] = 1.5;
        mem[7] = -2.0;
        let state = crate::creature::birth::Zygote::founder(minimal_genome(), body([0; 6]))
            .with_payload(crate::creature::birth::BirthPayload::new(
                crate::genome::learning::Seeds::default(),
                mem,
            ))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(5, 5),
                    heading: crate::contracts::Direction::N,
                    energy: 20.0,
                },
            )
            .creature;
        assert!((state.shared_memory[3] - 1.5).abs() < f32::EPSILON);
        assert!((state.shared_memory[7] - -2.0).abs() < f32::EPSILON);
        // prev is always zeroed for new creatures
        assert_eq!(state.prev_shared_memory, [0.0; SHARED_MEMORY_SLOTS]);
    }

    #[test]
    fn new_creature_starts_with_an_empty_dispatch_record() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = crate::creature::birth::Zygote::founder(minimal_genome(), body([0; 6]))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(0, 0),
                    heading: crate::contracts::Direction::N,
                    energy: 20.0,
                },
            )
            .creature;
        assert!(state.graph_runtime().dispatch_record.is_empty());
        assert!(state
            .graph_runtime()
            .dispatch_record
            .executed_indices(0, 100)
            .is_empty());
    }

    #[test]
    fn new_creature_populates_cached_reachable_nodes() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        // minimal_genome has 1 node (the entry), which is reachable
        let state = crate::creature::birth::Zygote::founder(minimal_genome(), body([0; 6]))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(0, 0),
                    heading: crate::contracts::Direction::N,
                    energy: 20.0,
                },
            )
            .creature;
        assert_eq!(state.reachable_nodes(), &[0]);
    }

    #[test]
    fn new_creature_reachable_excludes_disconnected_nodes() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        // Genome with 2 nodes: entry at index 0, disconnected at index 1
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome::new(
                    NodeId::new(0),
                    vec![],
                    BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    // no targets => node 1 is unreachable
                    vec![],
                ),
                NodeGenome::new(
                    NodeId::new(1),
                    vec![],
                    BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    vec![],
                ),
            ],
        };
        let state = crate::creature::birth::Zygote::founder(genome, body([0; 6]))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(0, 0),
                    heading: crate::contracts::Direction::N,
                    energy: 20.0,
                },
            )
            .creature;
        // Only node 0 should be reachable
        assert_eq!(state.reachable_nodes(), &[0]);
    }

    /// A founder of a drawn profile after `rounds` births' worth of mutation
    /// at a raised rate, carrying its parent's birth seeds when the drawn
    /// parent learned: VM beside Graph nodes, unreachable nodes and emptied
    /// sinks all occur.
    fn mutated_genome() -> impl Strategy<Value = CreatureGenome> {
        use crate::config::{FounderProfile, MutationConfig};
        use crate::creature::founder::founder_genome;
        use crate::mutation::reachability::ParentExecuted;
        use crate::mutation::MutationEngine;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;

        let profiles = [
            FounderProfile::V3Alpha1,
            FounderProfile::ForageFirstSparse,
            FounderProfile::ForageFirstSparseBalanced,
        ];
        (0..profiles.len(), any::<u64>(), 0usize..6).prop_map(move |(profile, seed, rounds)| {
            let config = MutationConfig {
                per_unit_rate: 0.05,
                ..MutationConfig::default()
            };
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut genome = founder_genome(profiles[profile]);
            for _ in 0..rounds {
                let reachable = mesh_reachable_nodes(&genome);
                MutationEngine::apply_mutations_in(
                    &mut genome,
                    &config,
                    &reachable,
                    ParentExecuted::Indices(&reachable),
                    &mut rng,
                    &crate::config::test_sensor_context(1),
                )
                .0;
            }
            genome
        })
    }

    proptest! {
        /// Every accessor of a constructed record is the analysis of the
        /// genome it stores, which carries no seeds, and its runtime is the
        /// one built for that genome; replaces the removed constructor that
        /// took the analyses as arguments.
        #[test]
        fn every_derived_accessor_is_the_analysis_of_the_stored_genome(
            genome in mutated_genome(),
        ) {
            let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
            let state = crate::creature::birth::Zygote::founder(genome, body([0; 6])).finalize(sm.insert(()), crate::creature::birth::BirthPlace { position: Position::new(0, 0), heading: crate::contracts::Direction::N, energy: 20.0 }).creature;
            let genome = state.genome();
            let reachable = mesh_reachable_nodes(genome);
            prop_assert_eq!(state.complexity(), genome.complexity());
            prop_assert_eq!(state.genome_size(), genome.genome_size());
            prop_assert_eq!(state.reachable_nodes(), reachable.as_slice());
            prop_assert_eq!(
                state.has_barrier_reader(),
                has_barrier_reader(genome, &reachable)
            );
            prop_assert_eq!(
                state.live_vm_world_inputs(),
                &*live_vm_world_reads(genome, &reachable)
            );
            prop_assert_eq!(
                state.perception_requirements(),
                perception_requirements(genome)
            );
            let declared: WorldKeySet =
                declared_world_keys(genome, &reachable).into_iter().collect();
            prop_assert_eq!(state.declared_world_inputs(), &declared);
            let wired: WorldKeySet = sensor_census(genome, &reachable)
                .world_inputs
                .into_iter()
                .collect();
            prop_assert_eq!(state.wired_world_inputs(), &wired);
            prop_assert_eq!(state.lifetime_actions_attempted_by_type, [0; 6]);
            prop_assert!(state.lifetime_eats_applied_by_type.is_empty());
            prop_assert_eq!(state.lifetime_predation_kills_count, 0);
            prop_assert_eq!(state.lifetime_predation_hits_taken_count, 0);
        }
    }

    #[test]
    fn typed_attempts_and_eats_grow_their_own_slots() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut state = crate::creature::birth::Zygote::founder(minimal_genome(), body([0; 6]))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(0, 0),
                    heading: crate::contracts::Direction::N,
                    energy: 20.0,
                },
            )
            .creature;
        state.record_action_attempt(ActionType::StealEnergy);
        state.record_action_attempt(ActionType::Eat);
        state.record_action_attempt(ActionType::Eat);
        assert_eq!(state.lifetime_action_attempted_count, 3);
        assert_eq!(state.lifetime_actions_attempted_by_type, [0, 2, 0, 0, 1, 0]);
        state.record_applied_eat(OrdinaryFoodTypeId::new(2));
        state.record_applied_eat(OrdinaryFoodTypeId::new(0));
        state.record_applied_eat(OrdinaryFoodTypeId::new(2));
        assert_eq!(state.lifetime_eats_applied_by_type, vec![1, 0, 2]);
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
            let state = crate::creature::birth::Zygote::founder(genome, body([0; 6]))
                .finalize(
                    id,
                    crate::creature::birth::BirthPlace {
                        position: Position::new(0, 0),
                        heading: crate::contracts::Direction::N,
                        energy: 20.0,
                    },
                )
                .creature;
            assert_eq!(state.genome_size(), expected);
            assert_eq!(state.genome_size(), state.genome().genome_size());
        }
    }

    /// The carrying charge's founder sizing is read from this number. Under the
    /// T25.F02 unit rule (one unit per edge, nothing per wired sink) the
    /// canonical founder is node 0 = 1 mesh node + 5 input references + 1 route
    /// target + 3 compute nodes + 4 compute edges + 6 sink edges = 20 and
    /// node 1 = 1 + 7 references + 10 compute nodes + 13 compute edges + 29
    /// sink edges = 60, so it pays `80 * genome_carry_cost_per_unit` per tick.
    /// The replication cost's anchor `FOUNDER_GENOME_SIZE_UNITS` is pinned to
    /// the same measurement, so the founder's multiplier is exactly 1.0. The
    /// ForageFirst profiles carry one more 2-input compute node: 83. No
    /// founder carries an edge bundle, so the T25.F10 unit rule (a gene one
    /// unit, its members nothing) leaves every founder size and the anchor
    /// unchanged: remeasured, not recalibrated.
    #[test]
    fn founder_is_eighty_units() {
        use crate::config::FounderProfile;
        use crate::creature::founder::{founder_genome, FOUNDER_GENOME_SIZE_UNITS};

        let no_bundle = |genome: &CreatureGenome| {
            genome.nodes.iter().all(|node| match node.backend_def() {
                BackendDef::Graph(def) => def.bundles().is_empty(),
                BackendDef::Vm(_) => true,
            })
        };
        let genome = founder_genome(FounderProfile::V3Alpha1);
        assert!(no_bundle(&genome), "the founder carries no edge bundle");
        assert_eq!(genome.genome_size(), 80);
        assert_eq!(genome.genome_size(), FOUNDER_GENOME_SIZE_UNITS);
        for profile in [
            FounderProfile::ForageFirstSparse,
            FounderProfile::ForageFirstSparseConservative,
            FounderProfile::ForageFirstSparseRichOffspring,
            FounderProfile::ForageFirstSparseBalanced,
        ] {
            let genome = founder_genome(profile);
            assert!(no_bundle(&genome), "{profile:?}");
            assert_eq!(genome.genome_size(), 83, "{profile:?}");
        }
    }

    #[test]
    fn creature_state_position_stored() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = {
            let mut born = crate::creature::birth::Zygote::founder(minimal_genome(), body([0; 6]))
                .finalize(
                    id,
                    crate::creature::birth::BirthPlace {
                        position: Position::new(3, 7),
                        heading: crate::contracts::Direction::N,
                        energy: 50.0,
                    },
                )
                .creature;
            born.generation = 2;
            born
        };
        assert_eq!(state.position, Position::new(3, 7));
        assert_eq!(state.generation, 2);
    }

    /// The body view carries the record's world-facing state and resolves the
    /// two body rules: the seen colour and the energy sensor denominator,
    /// the reference capacity under `Reference` and the body's own under
    /// `Own` (T26.F02 invariant 6).
    #[test]
    fn the_sensed_body_resolves_colour_and_energy_scale() {
        use crate::config::SensorScale;
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let mut state = crate::creature::birth::Zygote::founder(
            minimal_genome(),
            body([150, 0, 200, 200, 60, 60]),
        )
        .finalize(
            sm.insert(()),
            crate::creature::birth::BirthPlace {
                position: Position::new(3, 7),
                heading: crate::contracts::Direction::N,
                energy: 50.0,
            },
        )
        .creature;
        state.age = 9;
        state.body.capacity = 50.0;
        state.body.identity.kin_tag = 0xABCD;
        state.previous_outcome = [1.0, 0.0, -2.0, 1.0];
        let mut capacity = EnergyCapacityConfig {
            reference: 150.0,
            ..EnergyCapacityConfig::default()
        };
        for (scale, expected) in [(SensorScale::Reference, 150.0), (SensorScale::Own, 50.0)] {
            capacity.sensor_scale = scale;
            assert_eq!(
                state.sensed_body(&capacity),
                BodyView {
                    position: Position::new(3, 7),
                    age: 9,
                    energy: 50.0,
                    energy_scale: expected,
                    colour: [13.0 / 255.0, 89.0 / 255.0, 174.0 / 255.0],
                    kin_tag: 0xABCD,
                    previous_outcome: [1.0, 0.0, -2.0, 1.0],
                },
                "{scale:?}"
            );
        }
    }

    #[test]
    fn new_creature_caches_live_vm_world_inputs() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome::new(
                    NodeId::new(0),
                    vec![
                        InputReference::World(WorldInputKey::AreaFoodSummary {
                            type_idx: crate::config::OrdinaryFoodTypeId::default(),
                        }),
                        InputReference::World(WorldInputKey::NeighborBarrierRing),
                        InputReference::DynamicIntrospection(
                            DynamicIntrospectionKey::EnergyCurrent,
                        ),
                    ],
                    BackendDef::Vm(VmBackendDef {
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
                    vec![],
                ),
                NodeGenome::new(
                    NodeId::new(1),
                    vec![InputReference::World(WorldInputKey::AreaOccupancySummary)],
                    BackendDef::Vm(VmBackendDef {
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
                    vec![],
                ),
            ],
        };

        let state = crate::creature::birth::Zygote::founder(genome, body([0; 6]))
            .finalize(
                id,
                crate::creature::birth::BirthPlace {
                    position: Position::new(0, 0),
                    heading: crate::contracts::Direction::N,
                    energy: 10.0,
                },
            )
            .creature;

        let counts: std::collections::HashMap<_, _> =
            state.live_vm_world_inputs().iter().cloned().collect();
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

use proptest::prelude::*;

use super::catalog::ACTION_QUEUE_DRAW_WIDTH;
use super::*;
use crate::config::{FounderProfile, OrdinaryFoodTypeId, RuntimeConfig, SimulationConfig};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, NodeId, RouteTarget, WorldInputKey,
};
use crate::creature::founder::founder_genome;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

const FOOD_0: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(0);

fn food_here() -> InputReference {
    InputReference::World(WorldInputKey::food_here(FOOD_0))
}

fn barriers() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn leaf(ref_idx: u16, sub_idx: u16) -> GraphEdge {
    GraphEdge {
        source: GraphSource::InputLeaf { ref_idx, sub_idx },
        weight: 1.0,
    }
}

fn channel(reference: &InputReference, sub_idx: u16) -> Channel {
    match addressed(reference, sub_idx) {
        Addressed::Channel(channel) => channel,
        Addressed::OutOfWidth(family) => panic!("{family:?} read out of width"),
    }
}

/// A Graph node with `refs`, compute nodes, and edges into `sink`.
fn graph_node(
    id: u32,
    refs: Vec<InputReference>,
    compute: Vec<ComputeNode>,
    sink: OutputSinkKind,
    edges: Vec<GraphEdge>,
) -> NodeGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.compute_nodes = compute;
    def.sink_mut(sink).expect("fixed catalog").inputs = edges;
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: refs,
        backend_def: BackendDef::Graph(def),
        targets: Vec::new(),
    }
}

fn genome(nodes: Vec<NodeGenome>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: nodes[0].node_id,
        nodes,
    }
}

const EAT: OutputSinkKind = OutputSinkKind::ActionVote(VoteSink::Eat);

/// One genome whose single node votes `Eat` from `refs[0]` at `sub_idx`.
fn eat_from(reference: InputReference, sub_idx: u16) -> CreatureGenome {
    genome(vec![graph_node(
        0,
        vec![reference],
        Vec::new(),
        EAT,
        vec![leaf(0, sub_idx)],
    )])
}

struct Fixture {
    config: SimulationConfig,
    runtime: RuntimeConfig,
    battery: Battery,
    extension: Extension,
}

impl Fixture {
    fn new() -> Self {
        let config = SimulationConfig::default();
        let battery = Battery::generate(config.world.food.types.len());
        let extension = Extension::new(Vec::new(), &battery);
        Self {
            runtime: config.runtime.clone(),
            config,
            battery,
            extension,
        }
    }

    fn with_runtime(runtime: RuntimeConfig) -> Self {
        Self {
            runtime,
            ..Self::new()
        }
    }

    fn read(&self, genome: &CreatureGenome) -> ParentUse {
        self.read_with(genome, Vec::new())
    }

    fn read_with(&self, genome: &CreatureGenome, children: Vec<CreatureGenome>) -> ParentUse {
        let context = EvalContext {
            runtime: &self.runtime,
            shared_memory_decay_rate: self.config.shared_memory.decay_rate,
            food_type_count: self.config.world.food.types.len(),
        };
        let scenes = Scenes {
            battery: &self.battery,
            extension: &self.extension,
            context: &context,
        };
        evaluate_parent(genome, |_| children, &scenes)
    }
}

#[test]
fn a_declared_reference_without_consumers_is_declared_only() {
    let parent = Fixture::new().read(&genome(vec![graph_node(
        0,
        vec![barriers()],
        Vec::new(),
        EAT,
        Vec::new(),
    )]));
    assert_eq!(parent.declared_channels.len(), 8);
    assert!(parent.connected.is_empty());
    assert!(parent.executed.is_empty());
    assert!(parent.causal.is_empty());
}

#[test]
fn connected_channels_returns_the_exact_live_consumer_set() {
    let reference = food_here();
    let genome = eat_from(reference.clone(), 0);
    assert_eq!(
        connected_channels(&genome),
        BTreeSet::from([channel(&reference, 0)])
    );
}

#[test]
fn a_connected_consumer_in_an_undispatched_node_is_not_executed() {
    let runtime = RuntimeConfig {
        max_mesh_hops: 1,
        ..RuntimeConfig::default()
    };
    let mut entry = graph_node(0, Vec::new(), Vec::new(), EAT, Vec::new());
    entry.targets = vec![RouteTarget {
        target_id: NodeId::new(1),
        slot: 0,
        gate_bias: 0.0,
    }];
    let reader = graph_node(1, vec![food_here()], Vec::new(), EAT, vec![leaf(0, 0)]);
    let parent = Fixture::with_runtime(runtime).read(&genome(vec![entry, reader]));
    let food = channel(&food_here(), 0);
    assert!(parent.connected.contains(&food));
    assert!(!parent.executed.contains(&food));
    assert!(parent.consistency_ok());
}

#[test]
fn an_executed_read_into_a_custom_output_has_no_effect() {
    let energy = InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent);
    let parent = Fixture::new().read(&genome(vec![graph_node(
        0,
        vec![energy.clone()],
        Vec::new(),
        OutputSinkKind::CustomOutput(0),
        vec![leaf(0, 0)],
    )]));
    let channel = channel(&energy, 0);
    assert!(parent.connected.contains(&channel));
    assert!(parent.executed_live.contains(&channel));
    assert!(!parent.causal.contains(&channel));
}

#[test]
fn a_structurally_dead_compute_node_still_reads_its_inputs() {
    let dead = ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![leaf(0, 0)],
        plasticity: None,
    };
    let parent = Fixture::new().read(&genome(vec![graph_node(
        0,
        vec![food_here()],
        vec![dead],
        OutputSinkKind::CustomOutput(0),
        vec![GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 3,
                previous: true,
            },
            weight: 1.0,
        }],
    )]));
    let food = channel(&food_here(), 0);
    assert!(!parent.connected.contains(&food));
    assert!(parent.executed.contains(&food));
    assert!(!parent.executed_live.contains(&food));
    let slot = shared_memory(3, true);
    assert!(parent.connected.contains(&slot) && parent.executed_live.contains(&slot));
}

/// A VM read used only as a `JumpIfZero` condition gates an `Eat` vote: the
/// census slice omits the control dependency, so the read is causal outside
/// the live walk.
#[test]
fn a_vm_branch_condition_is_causal_outside_the_live_walk() {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::JumpIfZero { cond: 0, offset: 2 },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        },
        VmInstruction::AddVote {
            sink: VoteSink::Eat.index() as u8,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let vm = genome(vec![NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![food_here()],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 4,
            constants: vec![1.0],
            program,
        }),
        targets: Vec::new(),
    }]);
    let parent = Fixture::new().read(&vm);
    let food = channel(&food_here(), 0);
    assert!(!parent.connected.contains(&food));
    assert!(parent.executed.contains(&food));
    assert!(parent.causal_original.contains(&food));
    let cohort = fold_cohort(Cohort::Founder, 1, &[parent], RETENTION_CHILDREN);
    let row = cohort.rows.iter().find(|row| row.channel == food).unwrap();
    assert_eq!(row.causal_outside_live, Some(1));
    assert_eq!(row.executed_outside_live, 1);
}

#[test]
fn a_barrier_read_is_causal_on_extended_scenes_only() {
    let parent = Fixture::new().read(&eat_from(barriers(), 0));
    let north = channel(&barriers(), 0);
    assert!(parent.causal.contains(&north));
    assert!(!parent.causal_original.contains(&north));
}

#[test]
fn a_food_read_is_causal_on_the_original_battery() {
    let parent = Fixture::new().read(&eat_from(food_here(), 0));
    let food = channel(&food_here(), 0);
    assert!(parent.causal_original.contains(&food));
    assert!(parent.causal.contains(&food));
}

#[test]
fn world_compound_channels_wrap_to_one_row() {
    let wrapped = genome(vec![graph_node(
        0,
        vec![barriers()],
        Vec::new(),
        EAT,
        vec![leaf(0, 2), leaf(0, 10)],
    )]);
    let parent = Fixture::new().read(&wrapped);
    assert_eq!(parent.connected, BTreeSet::from([channel(&barriers(), 2)]));
}

#[test]
fn action_queue_channels_past_the_draw_width_are_kept_and_labelled() {
    let parent = Fixture::new().read(&eat_from(InputReference::ActionQueue, 14));
    let beyond = channel(&InputReference::ActionQueue, 14);
    assert!(beyond.beyond_draw_width());
    assert!(parent.connected.contains(&beyond) && parent.executed.contains(&beyond));
    let cohort = fold_cohort(Cohort::Founder, 1, &[parent], RETENTION_CHILDREN);
    let row = cohort
        .rows
        .iter()
        .find(|row| row.channel == beyond)
        .unwrap();
    assert_eq!(row.declared, Some(1));
    assert_eq!(
        cohort
            .rows
            .iter()
            .filter(|row| row.channel.family == Family::ActionQueue)
            .count(),
        usize::from(ACTION_QUEUE_DRAW_WIDTH) + 1
    );
}

#[test]
fn decision_reads_past_their_width_count_as_out_of_width_consumers() {
    let width = sub_value_count(&InputReference::CommitCounts);
    let parent = Fixture::new().read(&eat_from(InputReference::CommitCounts, width + 1));
    assert_eq!(parent.out_of_width.get(&Family::CommitCounts), Some(&1));
    assert!(parent.connected.is_empty() && parent.executed.is_empty());
    let cohort = fold_cohort(Cohort::Drift, 1, &[parent], RETENTION_CHILDREN);
    let family = cohort
        .families
        .iter()
        .find(|row| row.family == Family::CommitCounts)
        .expect("a declared decision compound keeps its family row");
    assert_eq!(family.out_of_width_consumers, 1);
    assert_eq!(family.declared, Some(1));
    assert_eq!((family.connected, family.executed), (0, 0));
}

/// Two parents reading three channels of one family: each parent counts once
/// in the family union, so the connected and executed unions (2) lie strictly
/// between the largest channel row (1) and the channel-row sum (3).
#[test]
fn a_family_union_counts_each_parent_once() {
    let fixture = Fixture::new();
    let both = genome(vec![graph_node(
        0,
        vec![barriers()],
        Vec::new(),
        EAT,
        vec![leaf(0, 0), leaf(0, 1)],
    )]);
    let parents = [fixture.read(&both), fixture.read(&eat_from(barriers(), 2))];
    let cohort = fold_cohort(Cohort::Drift, 2, &parents, RETENTION_CHILDREN);
    let rows: Vec<&Row> = cohort
        .rows
        .iter()
        .filter(|row| row.channel.family == Family::NeighborBarrierRing && row.connected > 0)
        .collect();
    assert_eq!(rows.len(), 3);
    assert!(rows
        .iter()
        .all(|row| row.connected == 1 && row.executed == 1));
    let family = cohort
        .families
        .iter()
        .find(|row| row.family == Family::NeighborBarrierRing)
        .expect("a family row");
    assert_eq!(family.declared, Some(2));
    assert_eq!(family.connected, 2);
    assert_eq!(family.executed, 2);
    assert_family_bounds(&cohort);
}

/// Ablating a channel no consumer reads leaves the genome untouched, and
/// ablating a channel every scene reads as zero leaves every action
/// unchanged, on both backends.
#[test]
fn ablation_is_the_identity_for_unread_and_all_zero_channels() {
    let fixture = Fixture::new();
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let unread = channel(&barriers(), 0);
    assert_eq!(ablated(&founder, |c| c == unread), founder);
    let vm = crate::creature::founder::vm_decision_founder_genome();
    assert_eq!(ablated(&vm, |c| c == unread), vm);

    // Barriers are zero on the original battery.
    let context = EvalContext {
        runtime: &fixture.runtime,
        shared_memory_decay_rate: fixture.config.shared_memory.decay_rate,
        food_type_count: 1,
    };
    let scenes = Scenes {
        battery: &fixture.battery,
        extension: &fixture.extension,
        context: &context,
    };
    let split = scenes.original_executions();
    for reader in [
        eat_from(barriers(), 0),
        genome(vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![barriers()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: Vec::new(),
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Eat.index() as u8,
                        src: 0,
                    },
                ],
            }),
            targets: Vec::new(),
        }]),
    ] {
        let base = scenes.actions(&reader);
        let zeroed = scenes.actions(&ablated(&reader, |c| c == channel(&barriers(), 0)));
        assert_eq!(base[..split], zeroed[..split]);
        assert_ne!(
            base, zeroed,
            "the extension's barrier contexts read nonzero"
        );
    }
}

#[test]
fn retention_counts_pairs_and_still_causal_children() {
    let parent_genome = eat_from(food_here(), 0);
    let kept = parent_genome.clone();
    let lost = genome(vec![graph_node(
        0,
        vec![food_here()],
        Vec::new(),
        EAT,
        Vec::new(),
    )]);
    let parent = Fixture::new().read_with(&parent_genome, vec![kept, lost]);
    let food = channel(&food_here(), 0);
    assert_eq!(parent.retention.get(&food), Some(&(2, 1)));
    assert_eq!(parent.children_sampled, 2);
    let cohort = fold_cohort(Cohort::Selected, 3, &[parent], RETENTION_CHILDREN);
    assert_eq!(cohort.retention_children_requested, RETENTION_CHILDREN);
    assert_eq!(cohort.retention_children_sampled, 2);
    assert_eq!(cohort.parents_requested, 3);
    assert_eq!(cohort.parents_evaluated, 1);
}

/// On the founder, every row keeps the funnel's order.
#[test]
fn founder_rows_keep_the_stage_invariant() {
    let parent = Fixture::new().read(&founder_genome(FounderProfile::V3Alpha1));
    let cohort = fold_cohort(Cohort::Founder, 1, &[parent], RETENTION_CHILDREN);
    assert_eq!(cohort.consistency_violations, 0);
    assert!(cohort.rows.iter().any(|row| row.causal_original > Some(0)));
    for row in &cohort.rows {
        assert_stage_order(row);
    }
}

fn assert_stage_order(row: &Row) {
    match row.declared {
        Some(declared) => {
            let causal = row.causal.unwrap();
            assert!(declared >= row.connected, "{row:?}");
            assert!(declared >= row.executed, "{row:?}");
            assert!(row.executed >= causal, "{row:?}");
            assert!(causal >= row.causal_original.unwrap(), "{row:?}");
            assert!(row.retained_causal_pairs <= row.retention_pairs, "{row:?}");
        }
        None => {
            assert!(row.channel.family.is_shared_memory());
            assert_eq!(row.causal, None);
        }
    }
    assert!(
        row.declared.unwrap_or(0) + row.connected + row.executed > 0,
        "rows with no parent at any stage are omitted: {row:?}"
    );
}

/// A channel row's count at one stage.
type RowStage = fn(&Row) -> Option<u32>;

/// Every family with a channel row has exactly one family row, and each
/// family union lies between its largest channel row and
/// `min(channel-row sum, family declared)`, in the funnel's order.
fn assert_family_bounds(cohort: &CohortUse) {
    let families: Vec<Family> = cohort.families.iter().map(|row| row.family).collect();
    let mut expected: Vec<Family> = cohort.rows.iter().map(|row| row.channel.family).collect();
    expected.dedup();
    assert_eq!(families, expected);
    for family in &cohort.families {
        let rows: Vec<&Row> = cohort
            .rows
            .iter()
            .filter(|row| row.channel.family == family.family)
            .collect();
        let shared = family.family.is_shared_memory();
        assert_eq!(family.declared.is_none(), shared, "{family:?}");
        assert_eq!(family.causal.is_none(), shared, "{family:?}");
        assert_eq!(family.causal_original.is_none(), shared, "{family:?}");
        let cap = family.declared.unwrap_or(cohort.parents_evaluated);
        let stages: [(Option<u32>, RowStage); 5] = [
            (family.declared, |row| row.declared),
            (Some(family.connected), |row| Some(row.connected)),
            (Some(family.executed), |row| Some(row.executed)),
            (family.causal, |row| row.causal),
            (family.causal_original, |row| row.causal_original),
        ];
        for (union, stage) in stages {
            let Some(union) = union else { continue };
            let channels: Vec<u32> = rows.iter().filter_map(|row| stage(row)).collect();
            let largest = channels.iter().copied().max().unwrap_or(0);
            let sum: u32 = channels.iter().sum();
            assert!(union >= largest, "{family:?} {rows:?}");
            assert!(union <= sum.min(cap), "{family:?} {rows:?}");
        }
        if let (Some(causal), Some(original)) = (family.causal, family.causal_original) {
            assert!(family.executed >= causal, "{family:?}");
            assert!(causal >= original, "{family:?}");
        }
    }
}

/// An extinct world: the selected cohort and the recorded group are
/// undefined with their reason, and the reading is identical across thread
/// counts.
#[test]
fn an_extinct_world_is_undefined_and_thread_count_does_not_matter() {
    let mut config = SimulationConfig::default();
    config.world.width = 12;
    config.world.height = 12;
    config.population.initial_creatures = 3;
    let mut sim = crate::simulation::seed_simulation(config.clone(), 5);
    let ids: Vec<_> = sim.creatures.keys().collect();
    for id in ids {
        sim.remove_creature(id);
    }
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let drift = [CohortParent {
        index: 0,
        depth_or_generation: 7,
        genome: eat_from(food_here(), 0),
    }];
    let battery = Battery::generate(config.world.food.types.len());
    let context = EvalContext::from_config(&config);
    let read = || {
        observe(
            WorldInputs {
                founder: &founder,
                drift: &drift,
                selected: Err("extinct: no living creature at the terminal tick"),
                sim: &sim,
                world_seed: 5,
            },
            &battery,
            &config.mutation,
            &context,
            Sizes::default(),
            2,
        )
    };
    let pool = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
    };
    let one = pool(1).install(read);
    let four = pool(4).install(read);
    assert_eq!(one, four);
    assert!(one.selected.is_err());
    assert!(one.recorded_contexts.is_err());
    assert_eq!(one.drift.parents_evaluated, 1);
    assert_eq!(one.drift.retention_children_requested, 2);
    assert!(one.drift.retention_children_sampled <= 2);
    assert_eq!(one.sequence_source, "authored");
}

impl ParentUse {
    fn consistency_ok(&self) -> bool {
        self.causal.is_subset(&self.executed)
    }
}

/// The founder after `events` production mutation events drawn from `seed`.
fn mutated_founder(seed: u64, events: u32) -> CreatureGenome {
    use rand::SeedableRng;
    let config = SimulationConfig::default();
    let mut genome = founder_genome(FounderProfile::V3Alpha1);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let one_event = crate::config::MutationConfig {
        per_unit_rate: 1.0,
        ..config.mutation.clone()
    };
    for _ in 0..events {
        let reachable = mesh_reachable_nodes(&genome);
        crate::mutation::MutationEngine::apply_mutations_on_units(
            &mut genome,
            1,
            &one_event,
            &reachable,
            crate::mutation::reachability::ParentExecuted::Indices(&reachable),
            &mut rng,
            config.world.food.types.len(),
        );
    }
    genome
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// On cohorts of genomes a few production mutation events away from the
    /// founder, every row keeps the funnel's order, causal channels were
    /// executed, rows exist only for channels some parent reached, and every
    /// family union keeps its bounds.
    #[test]
    fn mutated_founders_keep_the_stage_invariant(
        seeds in proptest::collection::vec(any::<u64>(), 1..4),
        events in 1u32..4,
    ) {
        let fixture = Fixture::new();
        let parents: Vec<ParentUse> = seeds
            .iter()
            .map(|&seed| fixture.read(&mutated_founder(seed, events)))
            .collect();
        for parent in &parents {
            prop_assert!(parent.causal.is_subset(&parent.executed));
        }
        let cohort = fold_cohort(Cohort::Drift, parents.len() as u32, &parents, RETENTION_CHILDREN);
        prop_assert_eq!(cohort.consistency_violations, 0);
        for row in &cohort.rows {
            assert_stage_order(row);
        }
        assert_family_bounds(&cohort);
    }

    /// Ablation never changes a genome's size or its node structure, is
    /// idempotent, and moves only reference indices of the ablated channels.
    #[test]
    fn ablation_preserves_structure(channels in proptest::collection::btree_set(0u16..8, 0..8)) {
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let pick = |c: Channel| c.family == Family::UpstreamSlot && channels.contains(&c.channel);
        let once = ablated(&founder, pick);
        prop_assert_eq!(once.genome_size(), founder.genome_size());
        prop_assert_eq!(once.nodes.len(), founder.nodes.len());
        prop_assert_eq!(ablated(&once, pick), once.clone());
        let reachable = mesh_reachable_nodes(&founder);
        let before = Inventory::new(&founder, &reachable).input_channels();
        let after = Inventory::new(&once, &reachable).input_channels();
        let expected: BTreeSet<Channel> = before.iter().copied().filter(|&c| !pick(c)).collect();
        prop_assert_eq!(after, expected);
    }
}

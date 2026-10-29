//! The birth transaction's acceptance guard (T29.F01): shared finalization
//! assertions through the founder, unchanged-child and mutated-child paths,
//! the mapping contract over the operator catalogs, its composition, and the
//! inherited-value cases the deleted seed rows pinned, now against the
//! mapping contract.

use std::ops::RangeInclusive;

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use slotmap::SlotMap;

use super::*;
use crate::config::{MutationConfig, SimulationConfig};
use crate::contracts::{NodeId, OUTCOME_CHANNEL_COUNT};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::body::BodyGenome;
use crate::genome::cgp::test_support::{plastic_node, set_surface_plasticity};
use crate::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
    GraphSource, FIRST_ACTION_VOTE_SINK, FIXED_SINK_COUNT,
};
use crate::genome::learning::{GraphMapping, LearnableId, LearnableKey, NodeSeeds};
use crate::genome::{BackendDef, HebbianRule, Learning, NodeGenome};
use crate::mutation::engine::apply_event;
use crate::mutation::reachability::TargetSets;
use crate::mutation::{BrainOperator, MutationOperator, MutationSkipReason};
use crate::runtime::state::GraphRuntimeState;

fn body(capacity: f32) -> BodyGenome {
    let mut config = SimulationConfig::default();
    config.energy.capacity.founder = capacity;
    crate::creature::founder::founder_body(0, 7, &config)
}

fn place(energy: f32) -> BirthPlace {
    BirthPlace {
        position: Position::new(3, 4),
        heading: Direction::SE,
        energy,
    }
}

fn learning(lamarckian: bool, rule: HebbianRule) -> Learning {
    Learning {
        rule,
        learning_rate: 0.1,
        weight_clamp: 10.0,
        lamarckian,
        modulation: None,
        retention: 1.0,
    }
}

/// A one-node genome running `def`.
fn genome_of(def: &CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome::new(
        NodeId::new(0),
        vec![NodeGenome::new(
            NodeId::new(0),
            Vec::new(),
            BackendDef::Graph(def.clone()),
            Vec::new(),
        )],
        crate::creature::founder::default_founder_body(),
    )
}

/// What every finalization guarantees (T29.F01 invariant 10): the stored
/// genome's analyses, per-life state zeroed, the place's heading, the
/// payload's memory and the energy given capped at the body's capacity.
fn assert_finalized(born: &Newborn, genome: &CreatureGenome, given: f32, memory: [f32; 16]) {
    let creature = &born.creature;
    assert_eq!(creature.genome(), genome);
    assert_eq!(creature.genome_size(), genome.genome_size());
    assert_eq!(
        creature.reachable_nodes(),
        crate::genome::analysis::mesh_reachable_nodes(genome).as_slice()
    );
    assert_eq!(creature.age, 0);
    assert_eq!(creature.prev_shared_memory, [0.0; 16]);
    assert_eq!(creature.previous_outcome, [0.0; OUTCOME_CHANNEL_COUNT]);
    assert_eq!(creature.offspring_spawned_count, 0);
    assert_eq!(creature.lifetime_action_attempted_count, 0);
    assert_eq!(creature.lifetime_energy_sample_count, 0);
    assert!(creature.pending_birth.is_none());
    assert!(creature.graph_runtime().dispatch_record.is_empty());
    assert!(creature.graph_runtime().node_state.is_empty());
    assert_eq!(creature.heading, Direction::SE);
    assert_eq!(creature.shared_memory, memory);
    let cap = creature.energy_cap();
    assert_eq!(creature.energy, given.min(cap));
    let overflow = if given > cap {
        crate::ledger::applied_debit(given, cap)
    } else {
        0.0
    };
    assert_eq!(born.overflow.to_bits(), overflow.to_bits());
}

fn creature_of(zygote: Zygote, energy: f32) -> Newborn {
    let mut ids: SlotMap<CreatureId, ()> = SlotMap::with_key();
    zygote.finalize(ids.insert(()), place(energy))
}

#[test]
fn a_founder_is_generation_zero_with_an_empty_record_and_its_energy_capped() {
    let genome = v3alpha1_founder_genome().with_body(body(100.0));
    for given in [20.0, 100.0, 250.0] {
        let born = creature_of(Zygote::founder(genome.clone(), 5), given);
        assert_finalized(&born, &genome, given, [0.0; 16]);
        assert_eq!(born.creature.generation, 0);
        assert_eq!(born.creature.lineage_id, 5);
        assert!(born.creature.birth_mutation_operators.is_empty());
        assert_eq!(born.creature.birth_edge_class, None);
        assert!(!born.creature.graph_runtime().learned().has_learning());
    }
}

/// A parent whose one Lamarckian edge learned 0.9, with `memory`.
fn learned_parent(memory: [f32; 16]) -> CreatureState {
    let def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![plastic_node(
        ComputeNodeKind::Add,
        vec![GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            0.5,
        )],
        Some(learning(true, HebbianRule::Classic)),
    )]);
    let zygote = Zygote::founder(genome_of(&def).with_body(body(100.0)), 2)
        .with_payload(BirthPayload::new(Seeds::default(), memory));
    let mut parent = creature_of(zygote, 50.0).creature;
    parent.runtime_mut().learned.set_slot(0, 0, 0.9, 0.0);
    parent.generation = 4;
    parent
}

#[test]
fn an_unchanged_child_inherits_its_parents_learning_memory_and_body() {
    let memory = [0.5; 16];
    let parent = learned_parent(memory);
    let mut config = SimulationConfig::default();
    config.mutation.per_unit_rate = 0.0;
    let (zygote, summary) = Zygote::conceive(&parent, &config, &mut SmallRng::seed_from_u64(3));
    assert_eq!(summary.applied_events, 0);
    let born = creature_of(zygote, 30.0);
    assert_finalized(&born, parent.genome(), 30.0, memory);
    let child = &born.creature;
    assert_eq!(child.generation, 5);
    assert_eq!(child.lineage_id, 2, "a child carries its parent's lineage");
    assert!(child.birth_mutation_operators.is_empty());
    let edge = child.graph_runtime().learned().edges(0).next().unwrap();
    assert_eq!(edge.learned, Some(0.9));
    assert_eq!(child.genome().body(), parent.genome().body());
}

/// An instrument's mutated genome equals conception's for the same draws
/// (T29.F02 invariant 3, B11): the engine entry under the config's body
/// ranges is conception's mutation, and conception's clamp is idempotent
/// under unchanged bounds.
#[test]
fn an_instruments_mutated_genome_equals_conceptions_for_the_same_draws() {
    let parent = learned_parent([0.0; 16]);
    let mut config = SimulationConfig::default();
    config.mutation.per_unit_rate = 0.3;
    for seed in 0..64 {
        let (zygote, summary) =
            Zygote::conceive(&parent, &config, &mut SmallRng::seed_from_u64(seed));
        let mut genome = parent.genome().clone();
        let executed = |window| {
            parent
                .graph_runtime()
                .dispatch_record
                .executed_indices(parent.age, window)
        };
        let (reading, _) = MutationEngine::apply_mutations_in(
            &mut genome,
            &config.mutation,
            &crate::creature::body::body_ranges(&config),
            parent.reachable_nodes(),
            ParentExecuted::Query(&executed),
            &mut SmallRng::seed_from_u64(seed),
            &config.sensor_context(config.world.food.types.len()),
        );
        assert_eq!(&genome, zygote.genome(), "seed {seed}");
        assert_eq!(reading.events, summary.events);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A mutated child: its learned values are the parent's captured
    /// values carried by the birth's mapping, its record the summary's.
    #[test]
    fn a_mutated_child_inherits_through_the_births_mapping(seed in any::<u64>()) {
        let parent = learned_parent([0.25; 16]);
        let mut config = SimulationConfig::default();
        config.mutation.per_unit_rate = 1.0;
        let (zygote, summary) =
            Zygote::conceive(&parent, &config, &mut SmallRng::seed_from_u64(seed));
        let expected_seeds = zygote.inheritance.apply(zygote.payload.seeds());
        let genome = zygote.genome().clone();
        let born = creature_of(zygote, 120.0);
        assert_finalized(&born, &genome, 120.0, [0.25; 16]);
        let child = &born.creature;
        prop_assert_eq!(child.generation, 5);
        let mut operators: Vec<_> = summary.applied_by_operator.keys().copied().collect();
        operators.sort_by_key(|operator| operator.as_key());
        prop_assert_eq!(&*child.birth_mutation_operators, operators.as_slice());
        prop_assert_eq!(child.birth_edge_class, summary.birth_edge_class());
        let rebuilt = GraphRuntimeState::for_genome(&genome, &expected_seeds);
        prop_assert_eq!(child.graph_runtime().learned(), rebuilt.learned());
    }
}

// ── The mapping contract over the operator catalogs ─────────────────────────

/// The founder grown by `growth` rate-1.0 births of four events, every edge
/// and bundle of which then reads as the parent's value.
fn grown(seed: u64, growth: u8) -> CreatureGenome {
    let config = MutationConfig {
        per_unit_rate: 1.0,
        ..MutationConfig::default()
    };
    let mut genome = v3alpha1_founder_genome();
    let mut rng = SmallRng::seed_from_u64(seed);
    for _ in 0..growth {
        let _ = MutationEngine::apply_mutations_on_units(
            &mut genome,
            crate::mutation::engine::ChromosomeUnits { brain: 4, body: 0 },
            &config,
            &crate::creature::body::default_body_ranges(),
            &[],
            ParentExecuted::NONE,
            &mut rng,
            &crate::config::test_sensor_context(2),
        );
    }
    genome
}

/// Every learnable of `genome` with its genomic own value: a lone edge's or
/// a member's own weight, a bundle's scale.
fn genomic(genome: &CreatureGenome) -> Seeds {
    let mut seeds = Seeds::default();
    for (node, genome_node) in genome.nodes.iter().enumerate() {
        let BackendDef::Graph(def) = genome_node.backend_def() else {
            continue;
        };
        for site in def.edge_sites() {
            seeds.set_edge(node, site, def.edge_at(site).own_weight());
        }
        for (id, bundle) in def.indexed_bundles() {
            seeds.set_scale(node, id, bundle.scale());
        }
    }
    seeds
}

/// How many of the child's learnables one committed event of `operator`
/// may leave `Fresh`, by the seed rule (invariant 4): a weight or scale
/// write and an added edge exactly one; a retarget, an added node (a split's
/// identity edge), a raw-field step and a route that wires a new input at
/// most one; every other operator keeps, copies, composes or drops, so none.
/// `None`: the count follows the event (an added bundle or sensor wires
/// fresh edges, a node removal rewires its consumers' sources, an added or
/// swapped backend is fresh), and only the value check applies.
fn fresh_bound(operator: MutationOperator) -> Option<RangeInclusive<usize>> {
    use crate::mutation::graph::GraphOperator as G;
    use crate::mutation::input_ref::InputRefOperator as I;
    use crate::mutation::topology::TopologyOperator as T;
    match operator {
        MutationOperator::Graph(
            G::AlterGraphEdgeWeight | G::AlterBundleScale | G::AddGraphEdge,
        ) => Some(1..=1),
        MutationOperator::Graph(
            G::RetargetGraphEdge | G::AddInternalGraphNode | G::GraphRawFieldMutation,
        )
        | MutationOperator::Topology(T::AddRouteTarget | T::AddFeedbackRoute) => Some(0..=1),
        MutationOperator::Graph(
            G::AddBundleLive | G::AddBundleSilent | G::RemoveInternalGraphNode,
        )
        | MutationOperator::InputRef(I::AddSensorLive | I::AddSensorSilent)
        | MutationOperator::Topology(T::AddNode | T::SwapNodeBackend) => None,
        _ => Some(0..=0),
    }
}

/// Every value `carried` holds is `child`'s genomic own value at that
/// learnable, bitwise; returns how many of the child's learnables it leaves
/// without one.
fn assert_carried(
    child: &CreatureGenome,
    carried: &Seeds,
    what: impl std::fmt::Debug,
) -> Result<usize, TestCaseError> {
    let child_values = genomic(child);
    for (id, value) in carried.iter() {
        prop_assert_eq!(
            child_values.get(id).map(f32::to_bits),
            Some(value.to_bits()),
            "{:?} at {:?}",
            what,
            id
        );
    }
    Ok(child_values.iter().count() - carried.iter().count())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Through the engine's gate, with the parent's values its genomic own
    /// values: every value a committed event's mapping carries is the
    /// child's genomic own value at that learnable, bitwise (a kept or
    /// copied learnable keeps its own value, a detach composes to the
    /// member's new own weight), and the learnables it leaves without a
    /// value are within the operator's [`fresh_bound`]. A rejected event restores the genome.
    #[test]
    fn every_carried_value_is_the_childs_genomic_value_through_every_catalog(
        seed in any::<u64>(),
        growth in 0u8..6,
    ) {
        let parent = grown(seed, growth);
        let reachable: Vec<usize> = (0..parent.nodes.len()).collect();
        let parent_seeds = genomic(&parent);
        let config = MutationConfig::default();
        let context = crate::config::test_sensor_context(2);
        let sets = TargetSets::new(&reachable, &[]);
        for (index, &operator) in BrainOperator::ALL.iter().enumerate() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed ^ index as u64);
            let mut targets = sets.selector(0.5, 0.0);
            match apply_event(&mut child, operator, &mut targets, &mut rng, &config, &context) {
                Ok((_, report)) => {
                    let carried = report.inheritance.apply(&parent_seeds);
                    let fresh = assert_carried(&child, &carried, operator)?;
                    if let Some(bound) = fresh_bound(operator.into()) {
                        prop_assert!(bound.contains(&fresh), "{:?} left {} fresh", operator, fresh);
                    }
                }
                Err(_) => prop_assert_eq!(&child, &parent, "{:?} rolled back", operator),
            }
        }
    }

    /// A gate-rejected event of every operator restores the draft and
    /// reports no mapping, and a whole birth of rejected events inherits
    /// every value unchanged: the parent carries a node id three times, which
    /// fails the parseability gate after any applied edit (one removal leaves
    /// a duplicate).
    #[test]
    fn a_rejected_event_of_every_catalog_restores_the_draft_and_its_values(
        seed in any::<u64>(),
        growth in 0u8..6,
    ) {
        let mut parent = grown(seed, growth);
        let duplicate = parent.nodes[0].clone();
        parent.nodes.extend([duplicate.clone(), duplicate]);
        let reachable: Vec<usize> = (0..parent.nodes.len()).collect();
        let parent_seeds = genomic(&parent);
        let config = MutationConfig::default();
        let context = crate::config::test_sensor_context(2);
        let sets = TargetSets::new(&reachable, &[]);
        let mut gated = false;
        for (index, &operator) in BrainOperator::ALL.iter().enumerate() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed ^ index as u64);
            let mut targets = sets.selector(0.5, 0.0);
            let result = apply_event(&mut child, operator, &mut targets, &mut rng, &config, &context);
            prop_assert!(result.is_err(), "{:?} committed on an unparseable draft", operator);
            gated |= result == Err(MutationSkipReason::ParseabilityViolation);
            prop_assert_eq!(&child, &parent, "{:?} rolled back", operator);
        }
        prop_assert!(gated, "the gate rejected no applied event");
        let mut child = parent.clone();
        let birth = MutationConfig { per_unit_rate: 1.0, ..config };
        let (summary, inheritance) = MutationEngine::apply_mutations_in(&mut child, &birth, &crate::creature::body::default_body_ranges(), &reachable, ParentExecuted::NONE, &mut SmallRng::seed_from_u64(seed), &context);
        // Every brain event is rejected; the body's five events, which no
        // gate reads, all apply and leave the brain as it was.
        prop_assert!(summary.attempted_events > 0);
        let body = summary.applied_by_domain.get(&crate::mutation::MutationDomain::Body);
        prop_assert_eq!(summary.applied_events, 5);
        prop_assert_eq!(body, Some(&5));
        prop_assert_eq!(&child.clone().with_body(*parent.body()), &parent);
        prop_assert_eq!(inheritance.apply(&parent_seeds), parent_seeds);
    }

    /// Applying a composed mapping equals applying each mapping in turn,
    /// over the gated events of whole births; every composed value is the
    /// child's genomic own value, and a birth whose committed events leave
    /// nothing fresh ([`fresh_bound`]) carries every learnable of the child.
    #[test]
    fn a_composed_mapping_applies_as_its_events_in_turn(
        seed in any::<u64>(),
        growth in 0u8..5,
        events in 1usize..10,
    ) {
        let mut child = grown(seed, growth);
        let parent_seeds = genomic(&child);
        let config = MutationConfig::default();
        let context = crate::config::test_sensor_context(2);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut composed = InheritanceMapping::Identity;
        let mut stepped = parent_seeds.clone();
        let mut fresh_free = true;
        for event in 0..events {
            let operator = BrainOperator::ALL[(seed as usize + event * 7) % BrainOperator::ALL.len()];
            let reachable: Vec<usize> = (0..child.nodes.len()).collect();
            let sets = TargetSets::new(&reachable, &[]);
            let mut targets = sets.selector(0.5, 0.0);
            if let Ok((_, report)) =
                apply_event(&mut child, operator, &mut targets, &mut rng, &config, &context)
            {
                stepped = report.inheritance.apply(&stepped);
                composed = report.inheritance.after(&composed);
                fresh_free &= fresh_bound(operator.into()) == Some(0..=0);
            }
        }
        let carried = composed.apply(&parent_seeds);
        prop_assert_eq!(&carried, &stepped);
        let fresh = assert_carried(&child, &carried, "the birth")?;
        if fresh_free {
            prop_assert_eq!(fresh, 0);
        }
    }
}

// ── Inherited-value cases, against the mapping contract ─────────────────────

/// The child of a parent on `def` whose plastic slots learned `values`, in
/// canonical order, without mutation: its learned value per plastic slot.
fn child_of(def: &CgpGraphBackendDef, values: &[f32]) -> Vec<Option<f32>> {
    let genome = genome_of(def);
    let mut parent = GraphRuntimeState::for_genome(&genome, &Seeds::default());
    for (slot, &value) in values.iter().enumerate() {
        parent.learned.set_slot(0, slot, value, 0.0);
    }
    let seeds = parent.learned().capture();
    let child = GraphRuntimeState::for_genome(&genome, &seeds);
    child.learned().edges(0).map(|edge| edge.learned).collect()
}

fn simple_cgp_def() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let _ = def.append_compute_node(ComputeNode::new(ComputeNodeKind::Constant(1.0), Vec::new()));
    let _ = def.append_compute_node(plastic_node(
        ComputeNodeKind::Add,
        vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
        Some(learning(true, HebbianRule::Classic)),
    ));
    let _ = def.append_compute_node(plastic_node(
        ComputeNodeKind::Sigmoid,
        vec![GraphEdge::new(GraphSource::ComputeNode(1), 0.3)],
        Some(learning(false, HebbianRule::Oja)),
    ));
    def
}

#[test]
fn lamarckian_copies_parent_weights_and_darwinian_starts_uninitialized() {
    assert_eq!(
        child_of(&simple_cgp_def(), &[0.9, 0.7]),
        vec![Some(0.9), None]
    );
}

#[test]
fn a_parent_with_nothing_initialized_carries_nothing() {
    let def = simple_cgp_def();
    let parent = GraphRuntimeState::for_genome(&genome_of(&def), &Seeds::default());
    assert!(parent.learned().capture().is_empty());
    assert_eq!(child_of(&def, &[]), vec![None, None]);
}

#[test]
fn a_graph_without_plastic_edges_carries_nothing() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    assert!(child_of(&def, &[]).is_empty());
}

/// The capture covers both surfaces: a Lamarckian vote edge's learned
/// weight reaches the child's sink edge.
#[test]
fn the_capture_carries_sink_edges() {
    let mut def = simple_cgp_def();
    let vote = EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK);
    let _ = def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(1), 1.0));
    set_surface_plasticity(&mut def, vote, Some(learning(true, HebbianRule::Classic)));
    let genome = genome_of(&def);
    let mut parent = GraphRuntimeState::for_genome(&genome, &Seeds::default());
    for (slot, value) in [0.9, 0.7, -1.25].into_iter().enumerate() {
        parent.learned.set_slot(0, slot, value, 0.0);
    }
    let captured = parent.learned().capture();
    assert_eq!(
        captured.edge_at(0, EdgeOccurrence::new(vote, 0)),
        Some(-1.25)
    );
    assert_eq!(
        child_of(&def, &[0.9, 0.7, -1.25]),
        vec![Some(0.9), None, Some(-1.25)]
    );
}

/// Three Lamarckian nodes of two edges each; edge `e` of node `n` carries
/// `2n + e + 1`.
fn tracked_def() -> CgpGraphBackendDef {
    let edge = GraphEdge::new(
        GraphSource::SharedMemory {
            slot: 0,
            previous: false,
        },
        0.25,
    );
    let node = plastic_node(
        ComputeNodeKind::Add,
        vec![edge; 2],
        Some(learning(true, HebbianRule::Classic)),
    );
    CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![node; 3])
}

fn tracked_parent(def: &CgpGraphBackendDef) -> NodeSeeds {
    (0..def.compute_nodes().len())
        .flat_map(|n| (0..2).map(move |e| (n, e)))
        .map(|(n, e)| {
            (
                LearnableKey::Edge(EdgeOccurrence::new(EdgeSurface::ComputeInput(n), e)),
                (2 * n + e + 1) as f32,
            )
        })
        .collect()
}

/// The inherited value of every compute edge of `def`, one row per node.
fn rows(def: &CgpGraphBackendDef, seeds: &NodeSeeds) -> Vec<Vec<Option<f32>>> {
    def.compute_nodes()
        .iter()
        .enumerate()
        .map(|(n, node)| {
            (0..node.inputs.len())
                .map(|e| {
                    seeds
                        .get(&LearnableKey::Edge(EdgeOccurrence::new(
                            EdgeSurface::ComputeInput(n),
                            e,
                        )))
                        .copied()
                })
                .collect()
        })
        .collect()
}

fn node_seeds(seeds: NodeSeeds) -> Seeds {
    seeds
        .into_iter()
        .map(|(key, value)| (LearnableId { node: 0, key }, value))
        .collect()
}

#[test]
fn a_correspondence_composes_insert_copy_remove_and_parallel_edges() {
    let mut def = tracked_def();
    let parent = tracked_parent(&def);
    let new = ComputeNode::new(ComputeNodeKind::Add, vec![]);
    let map = def.insert_compute_node(1, new);
    let map = def.duplicate_compute_nodes(&[0, 2]).after(&map);
    let map = def.remove_compute_node(0).after(&map);
    let birth = rows(&def, &map.apply(&parent));
    assert_eq!(birth[0], vec![Some(1.0), Some(2.0)]);
    assert!(birth[1].is_empty());
    assert_eq!(birth[2], vec![Some(3.0), Some(4.0)]);
    assert_eq!(birth[3], vec![Some(3.0), Some(4.0)]);
    assert_eq!(birth[4], vec![Some(5.0), Some(6.0)]);
    let seeds = node_seeds(map.apply(&parent));
    let child = GraphRuntimeState::for_genome(&genome_of(&def), &seeds);
    let learned: Vec<_> = child.learned().edges(0).map(|edge| edge.learned).collect();
    assert_eq!(
        learned,
        [1.0, 2.0, 3.0, 4.0, 3.0, 4.0, 5.0, 6.0].map(Some).to_vec()
    );
}

#[test]
fn source_deletion_and_input_pruning_keep_identities_aligned() {
    let mut def = tracked_def();
    let parent = tracked_parent(&def);
    // The rewiring is the parent's genome, so its values stand.
    let _ = def.retarget_edge(
        EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 0),
        GraphSource::ComputeNode(0),
        &[],
    );
    let _ = def.retarget_edge(
        EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 1),
        GraphSource::InputLeaf {
            ref_idx: 2,
            sub_idx: 2,
        },
        &[],
    );
    // The edge reading the removed node inherits nothing.
    let map = def.remove_compute_node(0);
    assert_eq!(rows(&def, &map.apply(&parent))[0], vec![None, Some(4.0)]);
    // Pruning the unaddressed reference 1 renumbers 2 -> 1 and moves no
    // learnable.
    def.reindex_input_refs_after_removal(1);
    assert_eq!(rows(&def, &map.apply(&parent))[0], vec![None, Some(4.0)]);
    assert_eq!(
        def.compute_nodes()[0].inputs[1].source(),
        GraphSource::InputLeaf {
            ref_idx: 1,
            sub_idx: 2,
        }
    );
}

#[test]
fn a_split_keeps_the_consumer_and_starts_the_identity_edge_fresh() {
    use crate::mutation::graph::operators::{split_existing_edge, test_inputs};
    let mut def = tracked_def();
    let parent = tracked_parent(&def);
    let (_, map) =
        split_existing_edge(&mut def, test_inputs(&[]), &mut SmallRng::seed_from_u64(19)).unwrap();
    let birth = rows(&def, &map.apply(&parent));
    let identity = birth.iter().position(|v| v == &[None]).unwrap();
    assert_eq!(def.compute_nodes()[identity].inputs[0].own_weight(), 1.0);
    assert!(def.compute_nodes()[identity].inputs[0]
        .stored_learning()
        .is_none());
    let retained: Vec<_> = birth.iter().flatten().filter_map(|v| *v).collect();
    assert_eq!(retained, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn edge_operators_reset_only_changed_occurrences_and_copy_bundles() {
    use crate::mutation::graph::operators::{
        alter_edge_weight_in_def, copy_node_inputs, raw_field_mutation, remove_edge, retarget_edge,
        test_inputs,
    };
    let original = tracked_def();
    let parent = tracked_parent(&original);
    let mut rng = SmallRng::seed_from_u64(71);
    for operation in 0..5 {
        let mut def = original.clone();
        let map: GraphMapping = match operation {
            0 => alter_edge_weight_in_def(&mut def, &mut rng).unwrap(),
            1 => {
                retarget_edge(&mut def, test_inputs(&[]), &mut rng)
                    .unwrap()
                    .1
            }
            2 => {
                raw_field_mutation(&mut def, test_inputs(&[]), &mut rng)
                    .unwrap()
                    .1
            }
            3 => remove_edge(&mut def, &mut rng).unwrap().1,
            _ => copy_node_inputs(&mut def, &mut rng).unwrap(),
        };
        let birth = rows(&def, &map.apply(&parent));
        for (idx, node) in def.compute_nodes().iter().enumerate() {
            assert_eq!(node.inputs.len(), birth[idx].len());
        }
        match operation {
            0..=2 => {
                for (n, node) in def.compute_nodes().iter().enumerate() {
                    for (e, edge) in node.inputs.iter().enumerate() {
                        let changed = edge != &original.compute_nodes()[n].inputs[e];
                        assert_eq!(birth[n][e].is_none(), changed);
                    }
                }
            }
            3 => assert_eq!(birth.iter().flatten().filter(|v| v.is_some()).count(), 5),
            _ => {
                let copied = birth.iter().find(|v| v.len() == 4).unwrap();
                assert!(rows(&original, &parent)
                    .iter()
                    .any(|v| v.as_slice() == &copied[2..]));
            }
        }
    }
}

/// The child's own final flags decide (T25.F12 invariant 7): a value on an
/// edge made Lamarckian after the capture initializes it, an edge without a
/// value stays uninitialized, a value on an edge whose learning was removed
/// is ignored, and zero is an available value.
#[test]
fn eligibility_uses_the_childs_final_flag_and_zero_is_available() {
    let mut def = tracked_def();
    set_surface_plasticity(
        &mut def,
        EdgeSurface::ComputeInput(0),
        Some(learning(false, HebbianRule::Classic)),
    );
    let mut seeds = Seeds::default();
    seeds.set_edge(0, EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0), 0.0);
    set_surface_plasticity(
        &mut def,
        EdgeSurface::ComputeInput(0),
        Some(learning(true, HebbianRule::Classic)),
    );
    set_surface_plasticity(&mut def, EdgeSurface::ComputeInput(2), None);
    let child = GraphRuntimeState::for_genome(&genome_of(&def), &seeds);
    let learned: Vec<_> = child.learned().edges(0).map(|edge| edge.learned).collect();
    assert_eq!(learned, vec![Some(0.0), None, None, None]);
}

proptest! {
    #[test]
    fn genome_equality_requires_every_genetic_field(value in -100.0f32..100.0) {
        let original = tracked_def();
        let edge = GraphEdge::new(GraphSource::SharedMemory { slot: 0, previous: false }, value);
        for field in 0..5 {
            let mut changed = original.clone();
            match field {
                0 => changed.set_compute_kind(0, ComputeNodeKind::Constant(value)),
                1 => { let _ = changed.push_edge(EdgeSurface::SinkInput(0), edge); }
                2 => { let _ = changed.push_edge(EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK), edge); }
                3 => { let _ = changed.push_edge(EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1), edge); }
                _ => changed.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0)).set_learning(None),
            }
            prop_assert_ne!(&original, &changed);
            prop_assert_ne!(&changed, &original);
        }
    }

    #[test]
    fn a_copy_then_remove_is_an_identity_for_every_learnable(
        values in prop::collection::vec(prop::collection::vec(-100.0f32..100.0, 0..8), 1..12),
        choice in 0usize..100,
    ) {
        let template = tracked_def().compute_nodes()[0].clone();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(
            values
                .iter()
                .map(|row| ComputeNode::new(template.kind, vec![template.inputs[0]; row.len()]))
                .collect(),
        );
        let parent: NodeSeeds = values
            .iter()
            .enumerate()
            .flat_map(|(n, row)| {
                row.iter().enumerate().map(move |(e, &value)| {
                    (LearnableKey::Edge(EdgeOccurrence::new(EdgeSurface::ComputeInput(n), e)), value)
                })
            })
            .collect();
        let source = choice % values.len();
        let map = def.duplicate_compute_nodes(&[source]);
        let map = def.remove_compute_node(source).after(&map);
        prop_assert_eq!(map.apply(&parent), parent);
    }
}

#[test]
fn an_append_preserves_existing_dangling_source_indices() {
    use crate::mutation::graph::operators::add_disconnected_node;
    let mut def = tracked_def();
    let _ = def.retarget_edge(
        EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0),
        GraphSource::ComputeNode(3),
        &[],
    );
    let parent = tracked_parent(&def);
    let map = add_disconnected_node(&mut def, &mut SmallRng::seed_from_u64(11)).unwrap();
    assert_eq!(
        def.compute_nodes()[0].inputs[0].source(),
        GraphSource::ComputeNode(3)
    );
    assert_eq!(rows(&def, &map.apply(&parent))[0][0], Some(1.0));
}

#[test]
fn graph_copy_operators_carry_available_values_and_new_edges_carry_none() {
    use crate::mutation::graph::operators::{
        add_edge, copy_cgp_subgraph, copy_compute_node, retarget_edge, test_inputs,
    };
    let mut base = tracked_def();
    let _ = base.retarget_edge(
        EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 0),
        GraphSource::ComputeNode(0),
        &[],
    );
    let _ = base.retarget_edge(
        EdgeOccurrence::new(EdgeSurface::ComputeInput(2), 0),
        GraphSource::ComputeNode(1),
        &[],
    );
    let parent = tracked_parent(&base);
    for copy in [copy_compute_node, copy_cgp_subgraph] {
        let mut def = base.clone();
        let map = copy(&mut def, &mut SmallRng::seed_from_u64(8)).unwrap();
        assert!(def.compute_nodes().len() > 3);
        let base_rows = rows(&base, &parent);
        for row in rows(&def, &map.apply(&parent)) {
            assert!(base_rows.contains(&row));
        }
    }
    let mut def = tracked_def();
    let parent = tracked_parent(&def);
    let mut map = GraphMapping::Identity;
    for seed in 0..12 {
        let (_, added) = add_edge(
            &mut def,
            test_inputs(&[]),
            &mut SmallRng::seed_from_u64(seed),
        )
        .unwrap();
        map = added.after(&map);
    }
    for row in rows(&def, &map.apply(&parent)) {
        assert!(row.iter().skip(2).all(Option::is_none));
    }
    // Replaying a retarget's draw against its already selected source
    // rewires nothing and carries every value.
    let mut draw = tracked_def();
    let _ = retarget_edge(
        &mut draw,
        test_inputs(&[]),
        &mut SmallRng::seed_from_u64(22),
    )
    .unwrap();
    let parent = tracked_parent(&draw);
    let (_, map) = retarget_edge(
        &mut draw,
        test_inputs(&[]),
        &mut SmallRng::seed_from_u64(22),
    )
    .unwrap();
    assert_eq!(map.apply(&parent), parent);
}

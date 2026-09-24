use super::controls::graph_genome;
use super::*;
use crate::config::{FounderProfile, SimulationConfig};
use crate::creature::founder::founder_genome;
use crate::creature::genome::cgp::{
    ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::ActionParamField;
use crate::creature::genome::{BackendDef, NodeGenome};
use proptest::prelude::*;

fn fixture() -> (SimulationConfig, Battery) {
    let config = SimulationConfig::default();
    let battery = Battery::generate(config.world.food.types.len());
    (config, battery)
}

fn constant(value: f32) -> ComputeNode {
    ComputeNode {
        kind: ComputeNodeKind::Constant(value),
        inputs: Vec::new(),
        plasticity: None,
    }
}

fn from_node(index: u16, weight: f32) -> GraphEdge {
    GraphEdge {
        source: GraphSource::ComputeNode(index),
        weight,
    }
}

/// A genome that votes `Eat` with `weight` from a constant 1.0, plus extra
/// compute nodes after it.
fn eater(weight: f32, extra: Vec<ComputeNode>) -> CreatureGenome {
    let mut nodes = vec![constant(1.0)];
    nodes.extend(extra);
    graph_genome(Vec::new(), nodes, vec![from_node(0, weight)])
}

fn graph_def(
    genome: &mut CreatureGenome,
    node: usize,
) -> &mut crate::creature::genome::cgp::CgpGraphBackendDef {
    match &mut genome.nodes[node].backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => unreachable!("fixtures are Graph genomes"),
    }
}

fn attribution(parent: &CreatureGenome, child: &CreatureGenome) -> Attribution {
    let (config, battery) = fixture();
    let context = EvalContext::from_config(&config);
    let trace = ParentTrace::new(parent, &battery, &context);
    attribute(&trace, child, &battery, &context)
}

/// A stateful compute node wired to nothing: a rate edit changes graph
/// runtime state only (the node count, and so the cost, is unchanged).
fn unwired_integrator(rate: f32) -> ComputeNode {
    ComputeNode {
        kind: ComputeNodeKind::DecayIntegrator(rate),
        inputs: vec![from_node(0, 1.0)],
        plasticity: None,
    }
}

#[test]
fn traced_battery_actions_equal_the_battery_signature() {
    let (config, battery) = fixture();
    let context = EvalContext::from_config(&config);
    for genome in [
        founder_genome(FounderProfile::V3Alpha1),
        eater(1.0, vec![unwired_integrator(0.5)]),
        controls::integrator_pair().1,
    ] {
        let traced = TracedBattery::run(
            &battery,
            &genome,
            context.runtime,
            context.shared_memory_decay_rate,
        );
        assert_eq!(traced.executions.len(), 80);
        assert_eq!(
            traced.signature(&battery),
            battery.signature(&genome, context.runtime, context.shared_memory_decay_rate)
        );
        assert_eq!(
            traced.dispatched,
            battery.executed_node_ids(&genome, context.runtime, context.shared_memory_decay_rate)
        );
    }
}

#[test]
fn action_changes_and_deaths_take_precedence() {
    let silent_parent = eater(0.0, Vec::new());
    let acting = eater(1.0, Vec::new());
    assert_eq!(
        attribution(&silent_parent, &acting).category,
        Category::ActionChanged
    );
    assert_eq!(
        attribution(&acting, &silent_parent).category,
        Category::ActionDead
    );
    assert!(!attribution(&acting, &silent_parent).action_silent);
}

#[test]
fn an_identical_child_is_genome_identical() {
    let parent = eater(1.0, Vec::new());
    let result = attribution(&parent, &parent.clone());
    assert_eq!(result.category, Category::GenomeIdentical);
    assert!(result.action_silent && !result.silent_with_state_or_cost);
}

#[test]
fn an_edit_to_a_never_dispatched_node_is_unexecuted() {
    let mut parent = eater(1.0, Vec::new());
    let stray = graph_genome(Vec::new(), vec![constant(3.0)], Vec::new()).nodes[0].clone();
    parent.nodes.push(NodeGenome {
        node_id: NodeId::new(9),
        ..stray
    });
    let mut child = parent.clone();
    graph_def(&mut child, 1).compute_nodes[0].kind = ComputeNodeKind::Constant(4.0);
    assert_eq!(
        edited_node_ids(&parent, &child),
        BTreeSet::from([NodeId::new(9)])
    );
    let result = attribution(&parent, &child);
    assert_eq!(result.category, Category::UnexecutedEdit);
    assert!(!result.consistency_violation);
}

#[test]
fn a_changed_vote_with_the_same_action_is_masked_before_selection() {
    let result = attribution(&eater(1.0, Vec::new()), &eater(0.5, Vec::new()));
    assert_eq!(result.category, Category::MaskedBeforeSelection);
    assert!(result.action_silent && !result.silent_with_state_or_cost);
}

/// A Graph parameter change that decodes to the same action (both values
/// round to food type 0) differs in computation only.
#[test]
fn a_graph_parameter_decoded_to_the_same_action_is_masked_before_selection() {
    let with_param = |value: f32| {
        let mut genome = eater(1.0, Vec::new());
        graph_def(&mut genome, 0)
            .sink_mut(OutputSinkKind::ActionParam(ActionParamField::EatFoodType))
            .expect("fixed catalog")
            .inputs = vec![from_node(0, value)];
        genome
    };
    let result = attribution(&with_param(0.1), &with_param(0.2));
    assert_eq!(result.category, Category::MaskedBeforeSelection);
}

#[test]
fn a_graph_state_only_change_is_state_or_cost_only() {
    let result = attribution(
        &eater(1.0, vec![unwired_integrator(0.5)]),
        &eater(1.0, vec![unwired_integrator(0.25)]),
    );
    assert_eq!(result.category, Category::StateOrCostOnly);
    assert!(result.silent_with_state_or_cost);
}

#[test]
fn computation_and_state_changes_count_masked_and_silent_with_state() {
    let result = attribution(
        &eater(1.0, vec![unwired_integrator(0.5)]),
        &eater(0.5, vec![unwired_integrator(0.25)]),
    );
    assert_eq!(result.category, Category::MaskedBeforeSelection);
    assert!(result.silent_with_state_or_cost);
}

/// Neighbor barriers are zero on every `neighborhood-v1` execution, so a
/// barrier weight edit on a dispatched node changes nothing the battery sees.
#[test]
fn a_dispatched_edit_nothing_records_is_unresolved() {
    let mut parent = controls::barrier_pair().0;
    let barrier = |weight| GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        weight,
    };
    graph_def(&mut parent, 0)
        .sink_mut(OutputSinkKind::ActionVote(
            crate::creature::genome::vote::VoteSink::Eat,
        ))
        .expect("fixed catalog")
        .inputs = vec![barrier(1.0)];
    let mut child = parent.clone();
    graph_def(&mut child, 0)
        .sink_mut(OutputSinkKind::ActionVote(
            crate::creature::genome::vote::VoteSink::Eat,
        ))
        .expect("fixed catalog")
        .inputs = vec![barrier(2.0)];
    let result = attribution(&parent, &child);
    assert_eq!(result.category, Category::Unresolved);
    assert!(!result.silent_with_state_or_cost);
}

#[test]
fn an_entry_change_marks_both_entries_edited() {
    let mut parent = eater(1.0, Vec::new());
    let mut second = parent.nodes[0].clone();
    second.node_id = NodeId::new(5);
    parent.nodes.push(second);
    let mut child = parent.clone();
    child.entry_node_id = NodeId::new(5);
    assert_eq!(
        edited_node_ids(&parent, &child),
        BTreeSet::from([NodeId::new(0), NodeId::new(5)])
    );
}

#[test]
fn genome_identity_distinguishes_negative_zero_and_ignores_nan_payloads_and_birth_weights() {
    let with = |value: f32| graph_genome(Vec::new(), vec![constant(value)], Vec::new());
    assert_ne!(genome_identity(&with(0.0)), genome_identity(&with(-0.0)));
    assert_eq!(
        genome_identity(&with(f32::NAN)),
        genome_identity(&with(f32::from_bits(0x7fc0_0001)))
    );
    let mut learned = with(1.0);
    graph_def(&mut learned, 0).birth_weights = Some(vec![vec![Some(0.5)]]);
    assert_eq!(genome_identity(&learned), genome_identity(&with(1.0)));
}

#[test]
fn controls_pass_on_the_authored_extension() {
    let (config, battery) = fixture();
    let context = EvalContext::from_config(&config);
    let extension = Extension::new(Vec::new(), &battery);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let controls = run_controls(&founder, &extension, &battery, &context);
    assert_eq!(controls.len(), 3);
    for control in &controls {
        assert!(control.passed, "{control:?}");
    }
    let barrier = &controls[0];
    assert!(barrier.differs_by_group[Group::Authored.index()]);
    let integrator = &controls[1];
    assert!(!integrator.differs_by_group[Group::Authored.index()]);
    assert!(!integrator.differs_by_group[Group::SequenceEarly.index()]);
    assert!(integrator.differs_by_group[Group::SequenceLate.index()]);
}

#[test]
fn authored_contexts_set_only_their_channel_group_in_production_ranges() {
    let (_, battery) = fixture();
    let authored = contexts::authored_contexts(&battery);
    assert_eq!(authored.len(), contexts::AUTHORED_CONTEXTS);
    for (index, context) in authored.iter().enumerate() {
        let original = &battery.snapshots()[index];
        assert_eq!(context.energy, original.energy);
        assert_eq!(
            context.sensors.local.food_here,
            original.sensors.local.food_here
        );
        let group = index % 8;
        if group != 0 {
            assert_eq!(context.sensors.local.neighbor_barrier, [0.0; 8]);
        }
        if group != 1 {
            assert_eq!(context.sensors.local.previous_outcome, [0.0; 4]);
        }
    }
    let outcomes = authored[1].sensors.local.previous_outcome;
    assert!((-1.0..=1.0).contains(&outcomes[0]));
    assert!((0.0..=1.0).contains(&outcomes[1]));
    assert!((-1.0..=0.0).contains(&outcomes[2]));
    assert!(outcomes[3] == 0.0 || outcomes[3] == 1.0);
    // Pinned draws: a change to the authored rules bumps the coverage version.
    assert_eq!(authored[0].sensors.local.neighbor_barrier, PINNED_BARRIERS);
    assert_eq!(outcomes.map(f32::to_bits), PINNED_OUTCOME_BITS);
}

const PINNED_BARRIERS: [f32; 8] = [1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0];
const PINNED_OUTCOME_BITS: [u32; 4] = [1_060_529_828, 1_039_038_914, 3_198_444_774, 0];

#[test]
fn the_recorded_sample_is_a_seeded_draw_in_draw_order() {
    let sample = contexts::recorded_sample(10, 4, 11);
    assert_eq!(sample, PINNED_RECORDED_SAMPLE);
    assert!(contexts::recorded_sample(0, 4, 11).is_empty());
    assert_eq!(contexts::recorded_sample(3, 32, 11).len(), 3);
    assert_eq!(selected_positions(50, 20, 11).len(), 20);
    assert!(selected_positions(50, 20, 11)
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    assert_eq!(selected_positions(5, 20, 11), vec![0, 1, 2, 3, 4]);
}

const PINNED_RECORDED_SAMPLE: [usize; 4] = [4, 2, 6, 9];

#[test]
fn sequences_cycle_recorded_contexts_or_fall_back_to_authored() {
    let (_, battery) = fixture();
    let authored = contexts::authored_contexts(&battery);
    let recorded: Vec<_> = battery.snapshots()[..5].to_vec();
    let sequences = contexts::sequences(&recorded, &authored);
    assert_eq!(sequences.len(), contexts::SEQUENCES);
    assert_eq!(sequences[1][3], recorded[(8 + 3) % 5]);
    let short = contexts::sequences(&recorded[..3], &authored);
    assert_eq!(short[2][30], authored[(16 + 30) % 24]);
}

fn small_sim(creatures: u32) -> Simulation {
    let mut config = SimulationConfig::default();
    config.world.width = 24;
    config.world.height = 24;
    config.population.initial_creatures = creatures.max(1);
    let mut sim = crate::simulation::seed_simulation(config, 5);
    if creatures == 0 {
        sim.creatures.clear();
    }
    sim
}

fn small_reading(sim: &Simulation) -> Reading {
    let (config, battery) = fixture();
    let context = EvalContext::from_config(&config);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let drift = vec![CohortParent {
        index: 0,
        depth_or_generation: 2,
        genome: eater(1.0, vec![unwired_integrator(0.5)]),
    }];
    let selected: Vec<CohortParent> = sim
        .creatures
        .values()
        .take(2)
        .enumerate()
        .map(|(index, creature)| CohortParent {
            index: index as u64,
            depth_or_generation: creature.generation,
            genome: creature.genome.clone(),
        })
        .collect();
    let selected = if selected.is_empty() {
        Err("extinct")
    } else {
        Ok(selected.as_slice())
    };
    observe(
        WorldInputs {
            founder: &founder,
            drift: &drift,
            selected,
            sim,
            world_seed: 11,
        },
        &battery,
        &config.mutation,
        &context,
        Sizes::default(),
    )
}

fn assert_reconciles(cohort: &CohortReading) {
    let applied: u32 = cohort.totals.categories.iter().sum();
    assert_eq!(applied, cohort.totals.applied());
    let rows = cohort
        .parents
        .iter()
        .fold(EffectCounts::default(), |sum, row| sum.merge(&row.counts));
    assert_eq!(rows, cohort.totals);
    let operators = cohort
        .operators
        .values()
        .fold(EffectCounts::default(), |sum, row| sum.merge(row));
    assert_eq!(operators, cohort.totals);
    let targets = cohort
        .targets
        .iter()
        .fold(EffectCounts::default(), |sum, row| sum.merge(row));
    assert_eq!(targets, cohort.totals);
    for key in cohort.operators.keys() {
        if let OperatorKey::DomainExhausted(_) = key {
            let row = cohort.operators[key];
            assert_eq!(
                row.proposals, row.skipped,
                "an exhausted domain never applies"
            );
        }
    }
    assert!(cohort.coverage.pairs_sampled <= cohort.coverage.pairs_requested);
    assert!(
        cohort.coverage.differ_any + cohort.coverage.state_or_cost_only
            <= cohort.coverage.pairs_sampled
    );
}

#[test]
fn a_world_reading_reconciles_and_is_identical_across_thread_counts() {
    let sim = small_sim(6);
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("pool")
            .install(|| small_reading(&sim))
    };
    let reading = run(1);
    assert_eq!(reading, run(4));
    let selected = reading
        .selected
        .as_ref()
        .expect("a living world defines the selected cohort");
    for cohort in [&reading.founder, &reading.drift, selected] {
        assert_reconciles(cohort);
        assert_eq!(
            cohort.totals.proposals,
            cohort.parents.len() as u32 * Sizes::default().proposals
        );
        assert_eq!(
            cohort.coverage.parents_evaluated,
            cohort.parents.len() as u32
        );
    }
    assert_eq!(reading.recorded_contexts, Ok(4));
    assert_eq!(reading.sequence_source, "recorded");
    assert!(reading.controls.iter().all(|control| control.passed));
}

#[test]
fn an_extinct_world_leaves_the_selected_cohort_and_recorded_group_undefined() {
    let sim = small_sim(0);
    let reading = small_reading(&sim);
    assert_eq!(reading.selected, Err("extinct".to_string()));
    assert!(reading.recorded_contexts.is_err());
    assert_eq!(reading.sequence_source, "authored");
    assert_eq!(
        reading.founder.coverage.differ_by_group[Group::Recorded.index()],
        0
    );
    assert!(reading.controls.iter().all(|control| control.passed));
}

#[test]
fn plurality_is_undefined_without_applied_proposals_and_breaks_ties_by_precedence() {
    assert_eq!(EffectCounts::default().plurality(), None);
    let mut counts = EffectCounts::default();
    counts.categories[Category::Unresolved.index()] = 3;
    counts.categories[Category::UnexecutedEdit.index()] = 3;
    assert_eq!(counts.plurality(), Some(Category::UnexecutedEdit));
}

fn attribution_strategy() -> impl Strategy<Value = Option<Attribution>> {
    prop::option::of((0usize..7, any::<bool>(), any::<bool>()).prop_map(
        |(category, state, violation)| {
            let category = Category::ALL[category];
            Attribution {
                category,
                silent_with_state_or_cost: state,
                consistency_violation: violation && category == Category::UnexecutedEdit,
                action_silent: !matches!(category, Category::ActionChanged | Category::ActionDead),
            }
        },
    ))
}

proptest! {
    /// Recording then merging is order-free, and categories always sum to
    /// applied proposals, whatever attributions are drawn.
    #[test]
    fn effect_counts_fold_is_a_partition_under_any_grouping(
        attributions in prop::collection::vec(attribution_strategy(), 0..60),
        split in 0usize..60,
    ) {
        let fold = |items: &[Option<Attribution>]| {
            let mut counts = EffectCounts::default();
            for item in items {
                counts.record(*item);
            }
            counts
        };
        let split = split.min(attributions.len());
        let whole = fold(&attributions);
        let (left, right) = (fold(&attributions[..split]), fold(&attributions[split..]));
        prop_assert_eq!(left.merge(&right), whole);
        prop_assert_eq!(right.merge(&left), whole);
        prop_assert_eq!(whole.categories.iter().sum::<u32>(), whole.applied());
        prop_assert_eq!(whole.proposals as usize, attributions.len());
        prop_assert!(whole.consistency_violations <= whole.categories[Category::UnexecutedEdit.index()]);
    }
}

#[test]
fn recorded_contexts_assemble_perception_whatever_the_donor_reads() {
    let sim = small_sim(6);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    assert!(!crate::sensors::perception::genome_uses_extended_perception(&founder));
    let contexts = contexts::recorded_contexts(&sim, 11, 4);
    assert_eq!(contexts.len(), 4);
    let food_types = sim.config.world.food.types.len();
    assert!(contexts
        .iter()
        .all(
            |context| context.sensors.perception.typed_area_food.len() == food_types
                && context.sensors.typed_local_food.food_here_by_type.len() == food_types
        ));
    assert!(contexts
        .iter()
        .any(|context| context.sensors.perception.area_food != [0.0; 7]));
}

/// Genome content copied into traces (here an input reference nothing
/// reads) is never compared, so a dispatched edit of it is unresolved.
#[test]
fn copied_genome_content_is_not_part_of_the_computation_record() {
    let parent = eater(1.0, Vec::new());
    let mut child = parent.clone();
    child.nodes[0]
        .input_refs
        .push(crate::contracts::InputReference::World(
            crate::contracts::WorldInputKey::NeighborBarrierRing,
        ));
    assert_eq!(attribution(&parent, &child).category, Category::Unresolved);
}

/// A child identical under `Debug` (NaN payloads render alike) is
/// `genome_identical` after the classifier, and still runs its records.
#[test]
fn a_nan_payload_edit_is_genome_identical() {
    let with =
        |bits: u32| graph_genome(Vec::new(), vec![constant(f32::from_bits(bits))], Vec::new());
    let result = attribution(&with(0x7fc0_0000), &with(0x7fc0_0001));
    assert_eq!(result.category, Category::GenomeIdentical);
    assert!(result.action_silent);
}

#[test]
fn a_pair_counts_once_in_the_union_and_state_only_needs_no_action_difference() {
    let mut coverage = Coverage::default();
    coverage.record_pair([true, true, false, true], true);
    coverage.record_pair([false; 4], true);
    coverage.record_pair([false; 4], false);
    assert_eq!(coverage.differ_by_group, [1, 1, 0, 1]);
    assert_eq!(coverage.differ_any, 1);
    assert_eq!(coverage.state_or_cost_only, 1);
}

/// A Graph-only parent has no VM site, so VM-domain events are exhausted:
/// they count in a `domain_exhausted` row as skips. The coverage pairs are
/// the first qualifying proposals in proposal order.
#[test]
fn exhausted_domains_count_as_skips_and_pairs_are_the_first_qualifying_proposals() {
    let (config, battery) = fixture();
    let context = EvalContext::from_config(&config);
    let one_event = MutationConfig {
        per_unit_rate: 1.0,
        ..config.mutation.clone()
    };
    let sizes = Sizes {
        proposals: 40,
        pairs_per_parent: 2,
        ..Sizes::default()
    };
    let parent = CohortParent {
        index: 0,
        depth_or_generation: 0,
        genome: eater(1.0, vec![unwired_integrator(0.5)]),
    };
    let evaluation = evaluate_parent(
        Cohort::Drift,
        0,
        &parent,
        &battery,
        &one_event,
        &context,
        sizes,
    );
    let exhausted: Vec<_> = evaluation
        .outcomes
        .iter()
        .filter(|outcome| matches!(outcome.operator, OperatorKey::DomainExhausted(_)))
        .collect();
    assert!(!exhausted.is_empty());
    assert!(exhausted
        .iter()
        .all(|outcome| outcome.attribution.is_none()));

    let qualifying: Vec<usize> = evaluation
        .outcomes
        .iter()
        .enumerate()
        .filter(|(_, outcome)| {
            outcome.attribution.is_some_and(|attribution| {
                attribution.action_silent && attribution.category != Category::GenomeIdentical
            })
        })
        .map(|(index, _)| index)
        .take(2)
        .collect();
    assert_eq!(
        qualifying.len(),
        2,
        "the fixture yields two qualifying proposals"
    );
    let replay: Vec<CreatureGenome> = qualifying
        .iter()
        .map(|&proposal| {
            let mut child = parent.genome.clone();
            let reachable = mesh_reachable_nodes(&parent.genome);
            let executed = battery.executed_indices(
                &parent.genome,
                context.runtime,
                context.shared_memory_decay_rate,
            );
            let seed = PROPOSAL_SEED_BASE + COHORT_SEED_MULTIPLIER + PARENT_SEED_MULTIPLIER;
            MutationEngine::apply_mutations_on_units(
                &mut child,
                1,
                &one_event,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut SmallRng::seed_from_u64(seed + proposal as u64),
                context.food_type_count,
            );
            child
        })
        .collect();
    assert_eq!(evaluation.pairs, replay);
}

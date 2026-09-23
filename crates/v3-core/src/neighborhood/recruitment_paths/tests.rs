use super::*;
use crate::contracts::NodeId;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::{MutationEngine, MutationOperator};
use crate::neighborhood::mesh_execution::indices_for_node_ids;
use crate::neighborhood::recruitment::ModuleBackend;
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn recruitment_paths_memory_effect_requires_every_paired_observation(
        left in prop::collection::vec(prop::option::of(-1i8..=1), 8),
        right in prop::collection::vec(prop::option::of(-1i8..=1), 8)
    ) {
        let reading = |values: &[Option<i8>]| TaskReading { scenes: values.iter().map(|value| Scene {
            food: [false; 3], correct_a: false, correct_b: false, survived: value.is_some(),
            energy: value.map(|_| 1.0), maintenance: 0.0, carrying: 0.0, work: Work::default(),
            actions: vec![], position: None, dispatched: vec![], routing: vec![], output_slots: vec![],
            shared_memory: value.map(|value| [f32::from(value); 16]),
        }).collect() };
        let expected = if left.iter().chain(&right).any(Option::is_none) { None }
            else { Some(left != right) };
        prop_assert_eq!(reading(&left).memory_effect(&reading(&right)), expected);
        prop_assert_eq!(reading(&right).memory_effect(&reading(&left)), expected);
        prop_assert_eq!(reading(&left).memory_effect(&reading(&right[..7])), None);
    }

    #[test]
    fn recruitment_paths_retention_prioritizes_death_and_deletion(live in any::<bool>(), present in any::<bool>(), loss in -8i16..=8) {
        let outcome = classify_retention(live, present.then_some(loss));
        if !live { prop_assert_eq!(outcome, RetentionOutcome::TaskDead); }
        else if !present { prop_assert_eq!(outcome, RetentionOutcome::Deleted); }
        else if loss >= 1 { prop_assert_eq!(outcome, RetentionOutcome::Useful); }
        else { prop_assert_eq!(outcome, RetentionOutcome::NoLongerUseful); }
    }

    #[test]
    fn recruitment_paths_selection_never_reduces_score_and_resolves_ties(
        parent in 0u8..=8, a in 0u8..=8, b in 0u8..=8, alive_a in any::<bool>(), alive_b in any::<bool>(),
        energies in prop::collection::vec(0.0f64..400.0, 3)
    ) {
        let parent_candidate = Candidate { live: true, score: parent, ending_energy_sum: energies[0] };
        let children = [
            Candidate { live: alive_a, score: a, ending_energy_sum: energies[1] },
            Candidate { live: alive_b, score: b, ending_energy_sum: energies[2] },
        ];
        let selected = choose(Policy::Selection, parent_candidate, children);
        if let Some(index) = selected {
            prop_assert!(children[index].live);
            prop_assert!(children[index].score >= parent);
            for (other, child) in children.iter().enumerate() {
                if child.live && child.score >= parent {
                    prop_assert!(children[index].score >= child.score);
                    if child.score == children[index].score { prop_assert!(index <= other); }
                }
            }
        } else { prop_assert!(children.iter().all(|child| !child.live || child.score < parent)); }
        // Selection never reads energy.
        let flat = |candidate: Candidate| Candidate { live: candidate.live, score: candidate.score, ending_energy_sum: 0.0 };
        prop_assert_eq!(
            choose(Policy::Selection, flat(parent_candidate), [flat(children[0]), flat(children[1])]),
            selected
        );
        prop_assert_eq!(choose(Policy::Drift, parent_candidate, children), Some(0));
    }

    #[test]
    fn recruitment_paths_cost_selection_orders_by_score_then_strictly_higher_energy(
        parent in 0u8..=8, a in 0u8..=8, b in 0u8..=8, alive_a in any::<bool>(), alive_b in any::<bool>(),
        energies in prop::collection::vec(0u8..4, 3)
    ) {
        // Small integer energies make equal-energy draws common.
        let energy = |index: usize| f64::from(energies[index]) * 100.0;
        let parent_candidate = Candidate { live: true, score: parent, ending_energy_sum: energy(0) };
        let children = [
            Candidate { live: alive_a, score: a, ending_energy_sum: energy(1) },
            Candidate { live: alive_b, score: b, ending_energy_sum: energy(2) },
        ];
        let selected = choose(Policy::CostSelection, parent_candidate, children);
        let eligible: Vec<_> = children.iter().enumerate()
            .filter(|(_, child)| child.live && child.score >= parent).collect();
        match selected {
            Some(index) => {
                let chosen = children[index];
                prop_assert!(chosen.live);
                prop_assert!(chosen.score >= parent);
                // Beats the parent: higher score, or equal score with at least the parent's energy (ties go to siblings).
                prop_assert!(chosen.score > parent || chosen.ending_energy_sum >= parent_candidate.ending_energy_sum);
                for &(other, child) in &eligible {
                    prop_assert!(chosen.score >= child.score);
                    if chosen.score == child.score {
                        prop_assert!(chosen.ending_energy_sum >= child.ending_energy_sum);
                        if chosen.ending_energy_sum == child.ending_energy_sum { prop_assert!(index <= other); }
                    }
                }
            }
            None => {
                for (_, child) in &eligible {
                    prop_assert!(child.score == parent && child.ending_energy_sum < parent_candidate.ending_energy_sum);
                }
            }
        }
        // With every energy equal the rule is exactly F02's selection.
        let flat = |candidate: Candidate| Candidate { live: candidate.live, score: candidate.score, ending_energy_sum: 1.0 };
        let tied = [flat(children[0]), flat(children[1])];
        prop_assert_eq!(
            choose(Policy::CostSelection, flat(parent_candidate), tied),
            choose(Policy::Selection, flat(parent_candidate), tied)
        );
    }

    #[test]
    fn recruitment_paths_time_to_first_sorts_and_censors_every_lineage(
        first in prop::collection::vec(prop::option::of(1u32..=32), 0..40)
    ) {
        let reading = TimeToFirst::of(first.iter().copied());
        prop_assert_eq!(reading.generations.len() + reading.censored as usize, first.len());
        prop_assert_eq!(reading.censored as usize, first.iter().filter(|value| value.is_none()).count());
        prop_assert!(reading.generations.windows(2).all(|pair| pair[0] <= pair[1]));
        let mut expected: Vec<_> = first.iter().flatten().copied().collect();
        expected.sort_unstable();
        prop_assert_eq!(&reading.generations, &expected);
        match reading.median {
            None => prop_assert!(expected.is_empty()),
            Some(median) => {
                let n = expected.len();
                let expected_median = if n % 2 == 1 { f64::from(expected[n / 2]) }
                    else { (f64::from(expected[n / 2 - 1]) + f64::from(expected[n / 2])) / 2.0 };
                prop_assert_eq!(median, expected_median);
            }
        }
    }

    #[test]
    fn recruitment_paths_spread_brackets_the_median(values in prop::collection::vec(-1000.0f64..1000.0, 0..40)) {
        match Spread::of(values.clone()) {
            None => prop_assert!(values.is_empty()),
            Some(spread) => {
                let min = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                prop_assert_eq!(spread.min, min);
                prop_assert_eq!(spread.max, max);
                prop_assert!(spread.min <= spread.median && spread.median <= spread.max);
                let below = values.iter().filter(|&&value| value <= spread.median).count();
                let above = values.iter().filter(|&&value| value >= spread.median).count();
                prop_assert!(below * 2 >= values.len() && above * 2 >= values.len());
            }
        }
    }

    #[test]
    fn recruitment_paths_wilson_is_bounded_and_contains_the_fraction(n in 1u32..1000, raw in any::<u32>()) {
        let successes = raw % (n + 1);
        let reading = estimate(successes, n);
        let [lower, upper] = reading.wilson_95.unwrap();
        let fraction = reading.fraction.unwrap();
        prop_assert!(lower >= 0.0 && upper <= 1.0);
        prop_assert!(lower <= fraction + 1e-14 && upper + 1e-14 >= fraction);
    }

    #[test]
    fn recruitment_paths_delta_replays_order_entry_additions_and_deletions(
        before_ids in prop::collection::btree_set(0u32..10, 1..8),
        after_ids in prop::collection::btree_set(0u32..10, 1..8), reverse in any::<bool>(), value in -100f32..100f32
    ) {
        let template = crate::creature::genome::NodeGenome {
            node_id: NodeId::new(0), input_refs: vec![], targets: vec![],
            backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
                register_count: 1, constants: vec![], program: vec![VmInstruction::Halt],
            }),
        };
        let genome = |ids: std::collections::BTreeSet<u32>, reverse, changed| {
            let mut nodes: Vec<_> = ids.into_iter().map(|id| {
                let mut node = template.clone(); node.node_id = NodeId::new(id);
                node.targets.clear();
                if let BackendDef::Vm(vm) = &mut node.backend_def { vm.constants = vec![changed]; }
                node
            }).collect();
            if reverse { nodes.reverse(); }
            CreatureGenome { entry_node_id: nodes[0].node_id, nodes }
        };
        let before = genome(before_ids, false, 0.0);
        let after = genome(after_ids, reverse, value);
        let delta = GenomeDelta::between(&before, &after);
        prop_assert_eq!(delta.apply(&before), Some(after.clone()));
        prop_assert_eq!(GenomeDelta::between(&after, &before).apply(&after), Some(before));
    }
}

#[test]
fn recruitment_paths_matched_starts_and_history_preserve_incumbents() {
    let starts = starting_forms();
    let main: Vec<_> = starts
        .iter()
        .filter(|start| start.task == Task::A)
        .collect();
    assert_eq!(main.len(), 5);
    let battery = &main[0].history.last().unwrap().battery;
    for start in main {
        assert_eq!(start.task_reading.correct(Task::A), 4);
        assert_eq!(&start.history.last().unwrap().battery, battery);
        assert!(!start.task_reading.dispatched().contains(&start.scaffold));
        let mut genome = start.creation_base.clone();
        for stage in &start.history {
            genome = stage.delta.apply(&genome).unwrap();
            assert_eq!(genome, stage.genome);
            assert!(stage.incumbent_actions_unchanged);
        }
    }
    for pair in starts
        .iter()
        .filter(|start| start.task == Task::B)
        .collect::<Vec<_>>()
        .chunks_exact(2)
    {
        let (unprepared, prepared) = (pair[0], pair[1]);
        assert_eq!(prepared.task_reading.correct(Task::A), 8);
        assert_eq!(prepared.task_reading.correct(Task::B), 4);
        let delta = prepared.preparation_difference.as_ref().unwrap();
        assert_eq!(delta.nodes.len(), 1);
        assert_eq!(delta.nodes[0].node, prepared.scaffold);
        assert_eq!(
            delta.apply(&unprepared.genome),
            Some(prepared.genome.clone())
        );
        for stage in &unprepared.history {
            let matched = prepared
                .history
                .iter()
                .find(|other| other.name == stage.name)
                .unwrap();
            assert_eq!(stage.battery, matched.battery);
            assert_eq!(stage.task.correct(Task::A), matched.task.correct(Task::A));
            assert_eq!(&stage.genome.nodes[..2], &matched.genome.nodes[..2]);
        }
        let mut activated = prepared.genome.clone();
        fixtures::topology(
            &mut activated,
            crate::mutation::topology::TopologyOperator::SwapRouteTargets,
            3,
        );
        assert_eq!(evaluate(&activated).correct(Task::B), 8);
        assert_eq!(evaluate(&activated).correct(Task::A), 4);
    }
}

#[test]
fn recruitment_paths_exact_copy_activation_preserves_actions_and_memory() {
    for path in constructed_paths() {
        let copied = &path.copy_stages[0];
        let mut activated = copied.genome.clone();
        fixtures::topology(
            &mut activated,
            crate::mutation::topology::TopologyOperator::SwapRouteTargets,
            3,
        );
        let config = task_config();
        assert_eq!(
            super::super::Battery::generate(1).signature(
                &activated,
                &config.runtime,
                config.shared_memory.decay_rate
            ),
            copied.battery
        );
        for (before, after) in copied.task.scenes.iter().zip(evaluate(&activated).scenes) {
            assert_eq!(before.actions, after.actions);
            assert_eq!(before.shared_memory, after.shared_memory);
            assert_eq!(before.position, after.position);
        }
        for stage in &path.copy_stages {
            assert!(stage.task.live());
            assert_eq!(
                stage.delta.apply(if stage.name == "dormant_copy" {
                    &path.base
                } else if stage.name == "prepared_copy" {
                    &path.copy_stages[0].genome
                } else {
                    &path.copy_stages[1].genome
                }),
                Some(stage.genome.clone())
            );
        }
        if let Some(split) = path.split_stage {
            assert_eq!(split.battery, copied.battery);
            assert_eq!(split.task.correct(Task::A), 4);
            assert!(split.genome.genome_size() > copied.genome.genome_size());
        }
    }
    // The changed-task history also pins exact-copy activation of a useful
    // incumbent, so this guarantee is not supported by silence alone.
    for start in starting_forms()
        .iter()
        .filter(|start| start.task == Task::B)
    {
        let copied = &start.history[0];
        let mut activated = copied.genome.clone();
        fixtures::topology(
            &mut activated,
            crate::mutation::topology::TopologyOperator::SwapRouteTargets,
            3,
        );
        let config = task_config();
        assert_eq!(
            super::super::Battery::generate(1).signature(
                &activated,
                &config.runtime,
                config.shared_memory.decay_rate
            ),
            copied.battery
        );
        assert_eq!(evaluate(&activated).correct(Task::A), 8);
        assert_eq!(
            activated.nodes[1].backend_def,
            activated.nodes[2].backend_def
        );
        assert_eq!(activated.nodes[1].input_refs, activated.nodes[2].input_refs);
    }
}

#[test]
fn recruitment_paths_dead_subjects_have_no_fabricated_ending_energy() {
    let mut genome = constructed_paths().remove(1).base;
    let BackendDef::Vm(vm) = &mut genome.nodes[0].backend_def else {
        unreachable!()
    };
    vm.constants = vec![100.0];
    vm.program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::SetPriorityBid { src: 0 },
    ];
    let reading = evaluate(&genome);
    assert!(!reading.live());
    assert!(reading
        .scenes
        .iter()
        .all(|scene| scene.energy.is_none() && !scene.survived));
    assert!(reading
        .scenes
        .iter()
        .all(|scene| scene.shared_memory.is_none()));
    assert_eq!(reading.summary().surviving_scenes, 0);
    let living = evaluate(&constructed_paths().remove(1).base);
    assert!(living
        .scenes
        .iter()
        .all(|scene| scene.shared_memory == Some([0.0; 16])));
    assert_eq!(living.memory_effect(&living), Some(false));
    assert_eq!(living.memory_effect(&reading), None);
    assert_eq!(reading.memory_effect(&living), None);
    assert_eq!(reading.memory_effect(&reading), None);
    let mut partially_observed = living.clone();
    partially_observed.scenes[0].shared_memory = Some([1.0; 16]);
    assert_eq!(living.memory_effect(&partially_observed), Some(true));
    partially_observed.scenes[7].shared_memory = None;
    assert_eq!(living.memory_effect(&partially_observed), None);
}

#[test]
fn recruitment_paths_stationary_wrong_actions_do_not_count_as_noop() {
    let mut genome = constructed_paths().remove(1).base;
    let BackendDef::Vm(vm) = &mut genome.nodes[0].backend_def else {
        unreachable!()
    };
    // The entry votes one `Eat` in every scene: a stationary wrong action.
    vm.constants = vec![1.0];
    vm.program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::AddVote { sink: 0, src: 0 },
        VmInstruction::Halt,
    ];
    let task = evaluate(&genome);
    assert!(task.live());
    assert_eq!(task.correct(Task::A), 0);
    assert_eq!(task.correct(Task::B), 0);
}

#[test]
fn recruitment_paths_every_observed_sibling_replays_the_unmodified_engine() {
    let report = observe(Sizes::TEST);
    for arm in &report.arms {
        let start = report
            .starts
            .iter()
            .find(|start| start.name == arm.start)
            .unwrap();
        let mut parent = start.genome.clone();
        for pair in arm.lineages[0].proposals.chunks_exact(2) {
            let task = evaluate(&parent);
            let reachable = mesh_reachable_nodes(&parent);
            let executed = indices_for_node_ids(&parent, &task.dispatched());
            let mut retained = parent.clone();
            for record in pair {
                let mut child = parent.clone();
                let mut rng = SmallRng::seed_from_u64(record.seed);
                let summary = MutationEngine::apply_mutations_with_food_type_count(
                    &mut child,
                    &super::experiment::proposal_mutation_config(),
                    &reachable,
                    ParentExecuted::Indices(&executed),
                    &mut rng,
                    1,
                );
                let events: Vec<_> = summary.events.iter().map(Event::from).collect();
                assert_eq!(events, record.events);
                // T13.F07: the bare engine, with no readings taken at all,
                // leaves the RNG and the fingerprint exactly where the
                // observed proposal recorded them, so the bypass, ancestral
                // and route readings consumed no mutation RNG.
                let rng_after = rand::RngCore::next_u64(&mut rng.clone());
                assert_eq!(rng_after, record.rng_after);
                assert_eq!(
                    super::experiment::fingerprint(&(
                        &GenomeDelta::between(&parent, &child),
                        &events,
                        rng_after
                    )),
                    record.mutation_fingerprint
                );
                assert_eq!(evaluate(&child).summary(), record.outcome);
                assert_eq!(
                    record.opportunities.attempted,
                    u64::from(summary.attempted_events)
                );
                assert_eq!(record.opportunities.births, 1);
                let delta = GenomeDelta::between(&parent, &child);
                assert_eq!(delta.apply(&parent), Some(child.clone()));
                if record.chosen {
                    retained = child;
                }
            }
            parent = retained;
        }
        assert_eq!(parent, arm.lineages[0].checkpoints.last().unwrap().genome);
    }
}

/// The T13.F06 readings are derived from the same lineages as the estimates;
/// check each against its source rows.
pub(super) fn assert_summary_readings_are_consistent(arm: &Arm) {
    let summary = &arm.summary;
    let lineages = arm.lineages.len() as u32;
    let first = |reading: &TimeToFirst, discovery: fn(&Lineage) -> Option<&Discovery>| {
        let mut expected: Vec<_> = arm
            .lineages
            .iter()
            .filter_map(|lineage| discovery(lineage).map(|discovery| discovery.generation))
            .collect();
        expected.sort_unstable();
        assert_eq!(reading.generations, expected, "{}", arm.start);
        assert_eq!(
            reading.generations.len() as u32 + reading.censored,
            lineages
        );
        assert!(reading
            .generations
            .iter()
            .all(|&generation| generation >= 1));
    };
    first(&summary.time_to_first_retained, |lineage| {
        lineage.retained_discovery.as_ref()
    });
    first(&summary.time_to_first_proposal, |lineage| {
        lineage.proposal_discovery.as_ref()
    });
    assert_eq!(
        summary.time_to_first_retained.generations.len() as u32,
        summary.retained_discovery.numerator
    );
    assert_eq!(
        summary.time_to_first_proposal.generations.len() as u32,
        summary.proposal_discovery.numerator
    );
    assert_eq!(
        summary.retention_outcomes.total(),
        summary.retained_discovery.numerator,
        "retention never censors: every discoverer has an outcome"
    );
    assert_eq!(
        summary.retention_outcomes.useful,
        summary.retained_useful.numerator
    );
    let proposals: Vec<_> = arm
        .lineages
        .iter()
        .flat_map(|lineage| &lineage.proposals)
        .collect();
    let dead = proposals
        .iter()
        .filter(|proposal| !proposal.outcome.live())
        .count() as u32;
    assert_eq!(summary.damage.task_dead.denominator, proposals.len() as u32);
    assert_eq!(summary.damage.task_dead.numerator, dead);
    assert_eq!(
        summary.damage.task_live_loss.denominator,
        proposals.len() as u32 - dead
    );
    let loss = proposals
        .iter()
        .filter(|proposal| proposal.outcome.live())
        .filter(|proposal| proposal.outcome.correct(arm.task) < proposal.parent_score)
        .count() as u32;
    assert_eq!(summary.damage.task_live_loss.numerator, loss);
    assert_eq!(
        summary
            .checkpoint_cost
            .iter()
            .map(|cost| cost.generation)
            .collect::<Vec<_>>(),
        arm.lineages[0]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.generation)
            .collect::<Vec<_>>()
    );
    for (index, cost) in summary.checkpoint_cost.iter().enumerate() {
        let at: Vec<_> = arm
            .lineages
            .iter()
            .map(|lineage| &lineage.checkpoints[index])
            .collect();
        let bracket = |spread: Spread, value: &dyn Fn(&Checkpoint) -> f64| {
            let values: Vec<_> = at.iter().map(|checkpoint| value(checkpoint)).collect();
            assert_eq!(
                spread.min,
                values.iter().copied().fold(f64::INFINITY, f64::min)
            );
            assert_eq!(
                spread.max,
                values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            );
            assert!(spread.min <= spread.median && spread.median <= spread.max);
        };
        bracket(cost.genome_size, &|checkpoint| {
            f64::from(checkpoint.genome.genome_size())
        });
        bracket(cost.modules, &|checkpoint| {
            checkpoint.genome.nodes.len() as f64
        });
        bracket(cost.carrying_sum, &|checkpoint| {
            checkpoint.task.summary().carrying_sum
        });
        bracket(cost.ending_energy_sum, &|checkpoint| {
            checkpoint.task.summary().ending_energy_sum
        });
    }
}

#[test]
fn recruitment_paths_reduced_run_has_complete_supply_and_replay() {
    let report = observe(Sizes::TEST);
    assert_eq!(report.total_proposals, Sizes::TEST.proposals());
    assert_eq!(report.arms.len(), ARMS);
    assert_eq!(report.starts.len(), STARTS);
    // The eighteen T13.F02 arms lead in their original order (start-major,
    // drift then selection); the nine cost arms follow in start order.
    let order: Vec<_> = report
        .arms
        .iter()
        .map(|arm| (arm.start.as_str(), arm.policy))
        .collect();
    let mut expected: Vec<_> = report
        .starts
        .iter()
        .flat_map(|start| Policy::F02.map(|policy| (start.name.as_str(), policy)))
        .collect();
    expected.extend(
        report
            .starts
            .iter()
            .map(|start| (start.name.as_str(), Policy::CostSelection)),
    );
    assert_eq!(order, expected);
    assert_eq!((order[0].0, order[0].1), ("graph_blank", Policy::Drift));
    // Five Task A starts and four Task B starts: C(15,2) + C(12,2).
    assert_eq!(report.pairs.len(), 105 + 66);
    assert_eq!(
        (report.pairs[0].left_arm, report.pairs[0].right_arm),
        (0, 1)
    );
    assert!(report.pairs.iter().all(|pair| {
        pair.left_arm < pair.right_arm
            && report.arms[pair.left_arm].task == report.arms[pair.right_arm].task
    }));
    assert_eq!(
        report.opportunities.attempted,
        report.opportunities.applied + report.opportunities.skipped
    );
    for arm in &report.arms {
        assert_eq!(arm.summary.retained_discovery.denominator, 1);
        assert_eq!(arm.batches.len(), 1);
        assert_summary_readings_are_consistent(arm);
        assert_eq!(
            [
                arm.batches[0].proposal_discovery.numerator,
                arm.batches[0].retained_discovery.numerator,
                arm.batches[0].viable_retained_discovery.numerator,
                arm.batches[0].retained_useful.numerator,
            ],
            [
                arm.summary.proposal_discovery.numerator,
                arm.summary.retained_discovery.numerator,
                arm.summary.viable_retained_discovery.numerator,
                arm.summary.retained_useful.numerator,
            ]
        );
        assert_eq!(arm.batches[0].proposal_discovery.denominator, 1);
        for lineage in &arm.lineages {
            assert_eq!(lineage.proposals.len(), 6);
            assert_eq!(lineage.checkpoints.len(), 3);
            for proposal in &lineage.proposals {
                let by_backend: u64 = proposal
                    .selected_inapplicable_by_backend_operator
                    .values()
                    .flat_map(|operators| operators.values())
                    .sum();
                assert_eq!(
                    by_backend + proposal.selected_inapplicable_backend_unresolved,
                    proposal
                        .opportunities
                        .discarded_selected_inapplicable_by_operator
                        .values()
                        .sum::<u64>()
                );
                assert!(
                    proposal.useful_modules.is_empty() || proposal.outcome.surviving_scenes == 8
                );
            }
            let start = report
                .starts
                .iter()
                .find(|start| start.name == arm.start)
                .unwrap();
            let mut genome = start.genome.clone();
            for step in &lineage.first_successful_path {
                genome = step
                    .delta
                    .apply(&genome)
                    .expect("exact replay before-values");
                assert_eq!(evaluate(&genome), step.outcome);
            }
        }
    }
    assert_eq!(estimate(0, 0).fraction, None);
    assert_eq!(estimate(0, 0).wilson_95, None);
}

#[test]
fn recruitment_paths_seed_streams_cover_the_fixed_disjoint_replicates() {
    let mut seeds = std::collections::BTreeSet::new();
    for batch in 0..4 {
        for lineage in 0..8 {
            for generation in 0..48 {
                for sibling in 0..2 {
                    let seed = experiment::proposal_seed(batch, lineage, generation, sibling);
                    assert!(seeds.insert(seed));
                }
            }
        }
    }
    assert_eq!(seeds.len(), 4 * 8 * 48 * 2);
    assert_eq!(seeds.first(), Some(&13_020_000));
    assert_eq!(seeds.last(), Some(&16_090_095));
    // T13.F06 re-pin: the nine `CostSelection` arms raise 18 arms to 27.
    assert_eq!(Sizes::PRODUCTION.proposals(), 82_944);
    let zero = estimate(0, 32);
    assert!((zero.wilson_95.unwrap()[1] - 0.107_179_198_255_070_6).abs() < 1e-12);
}

#[test]
fn recruitment_paths_scene_encoding_covers_each_food_bit_and_aggregates_work() {
    let reading = evaluate(&constructed_paths().remove(1).base);
    let lifecycle_decay = f64::from(task_config().energy.lifecycle.energy_decay_per_tick);
    for (index, scene) in reading.scenes.iter().enumerate() {
        assert_eq!(scene.food, [index & 4 != 0, index & 2 != 0, index & 1 != 0]);
        assert!((scene.maintenance - scene.carrying - lifecycle_decay).abs() < 1e-12);
    }

    let mut synthetic = reading.clone();
    synthetic.scenes.truncate(2);
    synthetic.scenes[0].dispatched = vec![NodeId::new(4), NodeId::new(4)];
    synthetic.scenes[1].dispatched = vec![NodeId::new(7)];
    synthetic.scenes[0].work = Work {
        mesh_hops: 2,
        vm_steps: 3,
        graph_visits: 5,
        plasticity: 7,
    };
    synthetic.scenes[1].work = Work {
        mesh_hops: 11,
        vm_steps: 13,
        graph_visits: 17,
        plasticity: 19,
    };
    synthetic.scenes[0].energy = Some(23.0);
    synthetic.scenes[1].energy = Some(29.0);
    synthetic.scenes[0].maintenance = 31.0;
    synthetic.scenes[1].maintenance = 37.0;
    synthetic.scenes[0].carrying = 41.0;
    synthetic.scenes[1].carrying = 43.0;

    assert_eq!(
        synthetic.dispatched(),
        [NodeId::new(4), NodeId::new(7)].into_iter().collect()
    );
    let summary = synthetic.summary();
    assert_eq!(summary.ending_energy_sum, 52.0);
    assert_eq!(summary.maintenance_sum, 68.0);
    assert_eq!(summary.carrying_sum, 84.0);
    assert_eq!(
        summary.work,
        Work {
            mesh_hops: 13,
            vm_steps: 16,
            graph_visits: 22,
            plasticity: 26,
        }
    );
}

#[test]
fn recruitment_paths_delta_rejects_each_mismatched_precondition() {
    let before = constructed_paths().remove(1).base;
    let mut after = before.clone();
    after.nodes.reverse();
    let delta = GenomeDelta::between(&before, &after);

    let mut wrong_entry = before.clone();
    wrong_entry.entry_node_id = NodeId::new(99);
    assert_eq!(delta.apply(&wrong_entry), None);

    let mut wrong_order = before.clone();
    wrong_order.nodes.reverse();
    assert_eq!(delta.apply(&wrong_order), None);

    let mut changed_node = before.clone();
    changed_node.nodes[0].targets.clear();
    let content_delta = GenomeDelta::between(&before, &changed_node);
    let mut wrong_content = before.clone();
    wrong_content.nodes[0].targets[0].slot = 99;
    assert_eq!(content_delta.apply(&wrong_content), None);
}

#[test]
fn recruitment_paths_fixture_sites_and_preparation_are_exact() {
    let starts = starting_forms();
    let expected = [
        // The T19.F04 vote-based modules: one vote edge (Graph) and one
        // `AddVote` (VM) select the action.
        ("graph_blank", [1, 2, 1, 0, 1, 1], 9),
        ("graph_copy", [2, 2, 1, 0, 2, 2], 12),
        ("graph_split", [2, 2, 1, 0, 3, 3], 14),
        ("graph_unprepared", [2, 2, 1, 1, 2, 4], 17),
        ("graph_prepared", [2, 2, 1, 1, 2, 4], 17),
        ("vm_blank", [1, 2, 7, 1, 0, 0], 14),
        ("vm_copy", [2, 2, 11, 2, 0, 0], 20),
        ("vm_unprepared", [2, 2, 9, 3, 0, 0], 19),
        ("vm_prepared", [2, 2, 9, 3, 0, 0], 19),
    ];
    for (name, sites, genome_size) in expected {
        let start = starts.iter().find(|start| start.name == name).unwrap();
        assert_eq!(
            [
                start.mutable_sites.input_refs,
                start.mutable_sites.route_targets,
                start.mutable_sites.vm_instructions,
                start.mutable_sites.vm_constants,
                start.mutable_sites.graph_compute_nodes,
                start.mutable_sites.graph_edges,
            ],
            sites
        );
        assert_eq!(start.genome_size, genome_size);
    }

    for backend in ["graph", "vm"] {
        let unprepared = starts
            .iter()
            .find(|start| start.name == format!("{backend}_unprepared"))
            .unwrap();
        let prepared = starts
            .iter()
            .find(|start| start.name == format!("{backend}_prepared"))
            .unwrap();
        let mut unprepared_active = unprepared.genome.clone();
        fixtures::topology(
            &mut unprepared_active,
            crate::mutation::topology::TopologyOperator::SwapRouteTargets,
            3,
        );
        let mut prepared_active = prepared.genome.clone();
        fixtures::topology(
            &mut prepared_active,
            crate::mutation::topology::TopologyOperator::SwapRouteTargets,
            3,
        );
        assert_eq!(evaluate(&unprepared_active).correct(Task::B), 2);
        assert_eq!(evaluate(&prepared_active).correct(Task::B), 8);
        assert!(!prepared.history.last().unwrap().useful);
        assert!(!unprepared.history.last().unwrap().useful);
        assert_eq!(
            unprepared.preparation_difference,
            prepared.preparation_difference
        );
        assert!(unprepared.preparation_difference.is_some());
    }
    for path in constructed_paths() {
        assert_eq!(
            path.stages
                .iter()
                .map(|stage| (stage.name.as_str(), stage.useful))
                .collect::<Vec<_>>(),
            vec![
                ("blank", false),
                ("sensor_preparation", false),
                ("dormant_preparation", false),
                ("activated", true),
            ]
        );
        assert_eq!(
            path.copy_stages
                .iter()
                .map(|stage| (stage.name.as_str(), stage.useful))
                .collect::<Vec<_>>(),
            vec![
                ("dormant_copy", false),
                ("prepared_copy", false),
                ("activated", true),
            ]
        );
    }
}

#[test]
fn recruitment_paths_wilson_midpoint_has_the_predeclared_interval() {
    let reading = estimate(1, 2);
    assert_eq!(reading.fraction, Some(0.5));
    let [lower, upper] = reading.wilson_95.unwrap();
    assert!((lower - 0.094_531_205_734_230_74).abs() < 1e-14);
    assert!((upper - 0.905_468_794_265_769_3).abs() < 1e-14);
}

/// The one-off seed search behind every pinned seed: the first accepted
/// seed per step in `0..SEARCH_RANGE`. It walks about 70,000 applied
/// events (the largest pinned seed is 31,060) in under two seconds, and it
/// is the only reading of each step's acceptance predicate: a weakened
/// predicate accepts an earlier seed, a broken one exhausts the range.
#[test]
fn recruitment_paths_seed_search_finds_the_pinned_seeds() {
    let found: Vec<_> = search_seeds()
        .into_iter()
        .inspect(|search| println!("{}: {:?}", search.form, search.seeds))
        .map(|search| (search.form, search.seeds))
        .collect();
    let pinned: Vec<_> = paths()
        .iter()
        .map(|path| {
            (
                path.form.clone(),
                path.steps
                    .iter()
                    .map(|step| (step.stage.name.as_str(), Some(step.seed)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(found, pinned);
}

fn paths() -> &'static [QualifiedPath] {
    static PATHS: std::sync::OnceLock<Vec<QualifiedPath>> = std::sync::OnceLock::new();
    PATHS.get_or_init(qualified_paths)
}

#[test]
fn recruitment_paths_qualified_family_is_the_fixed_nine_forms() {
    let family: Vec<_> = paths()
        .iter()
        .map(|path| (path.form.as_str(), path.task, path.backend))
        .collect();
    assert_eq!(
        family,
        [
            ("graph_blank", Task::A, ModuleBackend::Graph),
            ("graph_copy", Task::A, ModuleBackend::Graph),
            ("graph_split", Task::A, ModuleBackend::Graph),
            ("vm_blank", Task::A, ModuleBackend::Vm),
            ("vm_copy", Task::A, ModuleBackend::Vm),
            ("graph_unprepared", Task::B, ModuleBackend::Graph),
            ("vm_unprepared", Task::B, ModuleBackend::Vm),
            ("graph_detour", Task::A, ModuleBackend::Graph),
            ("vm_detour", Task::A, ModuleBackend::Vm),
        ]
    );
    for path in paths() {
        assert_eq!(path.start.task.correct(path.task), 4, "{}", path.form);
        assert!(path.start.task.live());
        let scaffold = path
            .start
            .genome
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::new(2));
        assert!(scaffold.is_some(), "{}", path.form);
        let dispatched = path.start.task.dispatched().contains(&NodeId::new(2));
        assert_eq!(dispatched, path.form.ends_with("_detour"), "{}", path.form);
    }
    let detour_seeds: Vec<_> = paths()
        .iter()
        .filter(|path| path.form.ends_with("_detour"))
        .map(|path| {
            assert_eq!(path.start.name, "inline_detour");
            assert!(path.start.incumbent_actions_unchanged);
            assert_eq!(
                path.start.genome.nodes[0].targets[0].target_id,
                NodeId::new(2)
            );
            assert_eq!(
                path.start.genome.nodes[2].targets[0].target_id,
                NodeId::new(1)
            );
            path.start.seed.unwrap()
        })
        .collect();
    assert_eq!(detour_seeds, [1, 0]);
}

#[test]
fn recruitment_paths_qualified_steps_hold_the_per_step_invariants() {
    for path in paths() {
        let mut previous = path.start.task.correct(path.task);
        for (index, step) in path.steps.iter().enumerate() {
            let last = index + 1 == path.steps.len();
            let label = format!("{} step {}", path.form, step.stage.name);
            assert!(step.seed < SEARCH_RANGE, "{label}");
            assert_eq!(step.stage.seed, Some(step.seed), "{label}");
            assert_eq!(step.stage.delta.nodes.len(), 1, "{label}");
            assert_eq!(step.charges(), step.stage.task.summary(), "{label}");
            assert_eq!(
                step.genome_size(),
                step.stage.genome.genome_size(),
                "{label}"
            );
            assert!(step.stage.task.live(), "{label}");
            let score = step.stage.task.correct(path.task);
            assert!(score + 1 >= previous, "{label}");
            if last {
                assert!(step.stage.useful, "{label}");
                assert!(
                    step.stage.task.dispatched().contains(&NodeId::new(2)),
                    "{label}"
                );
                assert!(score > path.start.task.correct(path.task), "{label}");
                assert!(!step.surfaces_unchanged, "{label}");
            } else {
                assert!(step.stage.incumbent_actions_unchanged, "{label}");
                assert!(step.surfaces_unchanged, "{label}");
                assert_eq!(
                    step.stage.task.dispatched().contains(&NodeId::new(2)),
                    path.form.ends_with("_detour"),
                    "{label}"
                );
            }
            previous = score;
        }
    }
}

#[test]
fn recruitment_paths_qualified_requires_no_gap_and_at_most_the_bound() {
    let complete = paths()
        .iter()
        .find(|path| path.form == "graph_copy")
        .unwrap();
    assert!(complete.qualified());
    let mut gapped = complete.clone();
    gapped.gap = Some(GrowthGap {
        length: 7,
        lengthening_step: "cue_added".into(),
    });
    assert!(!gapped.qualified());
    let mut long = complete.clone();
    long.steps = vec![complete.steps[0].clone(); MAX_PATH_EVENTS + 1];
    assert!(!long.qualified());
}

#[test]
fn recruitment_paths_qualified_paths_replay_through_deltas() {
    for path in paths() {
        let mut genome = path.start.genome.clone();
        for step in &path.steps {
            genome = step.stage.delta.apply(&genome).unwrap();
            assert_eq!(
                genome, step.stage.genome,
                "{} {}",
                path.form, step.stage.name
            );
        }
    }
}

/// The VM insert and `AddGraphEdge` seeds are the ones the one-off search
/// found after T19.F04 introduced vote-based action selection (the
/// fresh-instruction draw gains `AddVote` and the edge surface the 27 vote
/// sinks, which remaps every seeded draw). A blank module now reaches a
/// `Move(E)` vote in one edge or one instruction, so both blank forms
/// qualify beside the copied, split, unprepared, and detour forms. The
/// unprepared forms open with `InputRef.Add` of the ring beside the copied
/// `FoodHere` and move each consumer onto it; `vm_unprepared` replaces the
/// zeroed direction vote with the `Move(E)` vote in one
/// `VmInstructionMutation`. T19.F05 moved the `InputRef.Add` seeds: the
/// input-reference draw grows from 22 to 27 entries.
#[test]
fn recruitment_paths_qualified_outcomes_and_seeds_are_pinned() {
    use MutationOperator::*;
    let record: Vec<_> = paths()
        .iter()
        .map(|path| {
            (
                path.form.as_str(),
                path.steps
                    .iter()
                    .map(|step| (step.event.operator(), step.seed))
                    .collect::<Vec<_>>(),
                path.gap.as_ref().map(|gap| gap.length),
            )
        })
        .collect();
    let swap = (TopologySwapRouteTargets, 0);
    assert_eq!(
        record,
        [
            (
                "graph_blank",
                vec![(InputRefAdd, 201), (GraphAddGraphEdge, 57), swap],
                None,
            ),
            ("graph_copy", vec![(GraphAddGraphEdge, 103), swap], None),
            ("graph_split", vec![(GraphAddGraphEdge, 103), swap], None),
            (
                "vm_blank",
                vec![
                    (InputRefAdd, 201),
                    (VmInstructionMutation, 1279),
                    (VmInstructionMutation, 2189),
                    swap,
                ],
                None,
            ),
            ("vm_copy", vec![(VmDeleteInstruction, 1), swap], None),
            (
                "graph_unprepared",
                vec![
                    (InputRefAdd, 1),
                    (GraphRetargetGraphEdge, 32),
                    (GraphRemoveGraphEdge, 5),
                    (GraphAddGraphEdge, 103),
                    swap,
                ],
                None,
            ),
            (
                "vm_unprepared",
                vec![
                    (InputRefAdd, 1),
                    (VmInstructionRawFieldMutation, 18),
                    (VmInstructionRawFieldMutation, 25),
                    (VmInstructionRawFieldMutation, 25),
                    (VmInstructionMutation, 66_421),
                    swap,
                ],
                None,
            ),
            (
                "graph_detour",
                vec![(InputRefAdd, 201), (GraphAddGraphEdge, 57)],
                None,
            ),
            (
                "vm_detour",
                vec![
                    (InputRefAdd, 201),
                    (VmInstructionMutation, 1279),
                    (VmInstructionMutation, 2189),
                ],
                None,
            ),
        ]
    );
    let qualified: Vec<_> = paths()
        .iter()
        .filter(|path| path.qualified())
        .map(|path| path.form.as_str())
        .collect();
    assert_eq!(
        qualified,
        [
            "graph_blank",
            "graph_copy",
            "graph_split",
            "vm_blank",
            "vm_copy",
            "graph_unprepared",
            "vm_unprepared",
            "graph_detour",
            "vm_detour"
        ]
    );
    for path in paths().iter().filter(|path| path.qualified()) {
        assert!(path.steps.len() <= MAX_PATH_EVENTS);
        assert_eq!(path.steps.last().unwrap().stage.task.correct(path.task), 8);
    }
}

#[test]
fn recruitment_paths_qualified_last_step_is_one_bounded_edit_on_a_dispatched_module() {
    for path in paths() {
        let last = path.steps.last().unwrap();
        let before = path
            .steps
            .len()
            .checked_sub(2)
            .map_or(&path.start, |i| &path.steps[i].stage);
        let change = &last.stage.delta.nodes[0];
        let (before_node, after_node) = (
            change.before.as_ref().unwrap(),
            change.after.as_ref().unwrap(),
        );
        assert_eq!(
            before_node.input_refs, after_node.input_refs,
            "{}",
            path.form
        );
        match last.event.operator() {
            MutationOperator::TopologySwapRouteTargets => {
                assert_eq!(change.node, NodeId::new(0));
                assert_eq!(before_node.backend_def, after_node.backend_def);
                assert_eq!(before_node.targets.len(), after_node.targets.len());
            }
            MutationOperator::GraphAddGraphEdge => {
                assert_eq!(change.node, NodeId::new(2));
                assert_eq!(before_node.targets, after_node.targets);
                let (BackendDef::Graph(b), BackendDef::Graph(a)) =
                    (&before_node.backend_def, &after_node.backend_def)
                else {
                    panic!()
                };
                assert_eq!(a.compute_nodes, b.compute_nodes);
                assert_eq!(
                    fixtures::move_sink(a, fixtures::EAST).inputs.len(),
                    fixtures::move_sink(b, fixtures::EAST).inputs.len() + 1
                );
            }
            MutationOperator::VmInstructionMutation => {
                assert_eq!(change.node, NodeId::new(2));
                assert_eq!(before_node.targets, after_node.targets);
                let (BackendDef::Vm(b), BackendDef::Vm(a)) =
                    (&before_node.backend_def, &after_node.backend_def)
                else {
                    panic!()
                };
                assert_eq!(a.constants, b.constants);
                // The exposing insert is the `Move(E)` vote before the Halt.
                assert_eq!(a.program.len(), b.program.len() + 1);
                assert_eq!(
                    a.program[a.program.len() - 2],
                    VmInstruction::AddVote { sink: 3, src: 0 }
                );
            }
            other => panic!("{}: unexpected exposing operator {other:?}", path.form),
        }
        assert!(
            !before.task.dispatched().contains(&NodeId::new(2)) || path.form.ends_with("_detour")
        );
        let bypass = evaluate(
            &crate::neighborhood::mesh_execution::static_successor_bypass(
                &last.stage.genome,
                NodeId::new(2),
            ),
        );
        assert!(
            bypass.correct(path.task) < last.stage.task.correct(path.task),
            "{}",
            path.form
        );
        assert_eq!(bypass.correct(path.task), 4, "{}", path.form);
    }
}

/// T13.F06 fixture cost reading. The last-step assertion is a regression
/// guard against energy ever overriding score; the neutral-step rejection
/// counts by form and backend are the barrier reading and are printed, not
/// scored.
#[test]
fn recruitment_paths_qualified_cost_verdicts_retain_every_useful_last_step() {
    let mut rejected_neutral: BTreeMap<(ModuleBackend, &str), (usize, usize)> = BTreeMap::new();
    for path in paths() {
        let verdicts = path.cost_verdicts();
        assert_eq!(verdicts.len(), path.steps.len(), "{}", path.form);
        for (verdict, step) in verdicts.iter().zip(&path.steps) {
            let label = format!("{} step {}", path.form, verdict.step);
            assert_eq!(verdict.step, step.stage.name, "{label}");
            assert_eq!(verdict.carrying_sum, step.charges().carrying_sum, "{label}");
            assert_eq!(
                verdict.ending_energy_sum,
                step.charges().ending_energy_sum,
                "{label}"
            );
            let expected = verdict.score > verdict.previous_score
                || (verdict.neutral()
                    && verdict.ending_energy_sum >= verdict.previous_ending_energy_sum);
            assert_eq!(verdict.retained, expected, "{label}");
            if verdict.neutral() {
                let entry = rejected_neutral
                    .entry((path.backend, path.form.as_str()))
                    .or_default();
                entry.1 += 1;
                entry.0 += usize::from(!verdict.retained);
            }
            eprintln!(
                "{:<16} {:<28} retained={:<5} score {}->{} carrying {:.4} energy {:.4}->{:.4}",
                path.form,
                verdict.step,
                verdict.retained,
                verdict.previous_score,
                verdict.score,
                verdict.carrying_sum,
                verdict.previous_ending_energy_sum,
                verdict.ending_energy_sum
            );
        }
        let last = verdicts.last().expect("every path has a step");
        assert!(last.retained, "{} last step {}", path.form, last.step);
        assert!(last.score > last.previous_score, "{}", path.form);
    }
    for ((backend, form), (rejected, neutral)) in &rejected_neutral {
        eprintln!("{backend:?} {form}: {rejected} of {neutral} neutral steps rejected");
    }
}

fn task_summary(surviving_scenes: u8, correct_a: u8, correct_b: u8) -> TaskSummary {
    TaskSummary {
        correct_a,
        correct_b,
        surviving_scenes,
        ending_energy_sum: 0.0,
        maintenance_sum: 0.0,
        carrying_sum: 0.0,
        work: Work::default(),
    }
}

#[test]
fn recruitment_paths_task_summary_is_live_only_with_all_eight_scenes() {
    // Arrange
    let all_scenes = task_summary(8, 3, 5);
    let one_death = task_summary(7, 3, 5);
    let no_scenes = task_summary(0, 0, 0);

    // Act & Assert
    assert!(all_scenes.live());
    assert!(!one_death.live());
    assert!(!no_scenes.live());
}

#[test]
fn recruitment_paths_task_summary_correct_reads_the_named_task_count() {
    // Arrange
    let summary = task_summary(8, 3, 5);

    // Act & Assert
    assert_eq!(summary.correct(Task::A), 3);
    assert_eq!(summary.correct(Task::B), 5);
    assert_eq!(task_summary(8, 0, 8).correct(Task::A), 0);
    assert_eq!(task_summary(8, 0, 8).correct(Task::B), 8);
}

#[test]
fn recruitment_paths_cost_verdict_is_neutral_only_at_equal_scores() {
    // Arrange
    let verdict = |score, previous_score| CostVerdict {
        step: "step".into(),
        retained: false,
        score,
        previous_score,
        carrying_sum: 0.0,
        ending_energy_sum: 0.0,
        previous_ending_energy_sum: 0.0,
    };

    // Act & Assert
    assert!(verdict(4, 4).neutral());
    assert!(!verdict(5, 4).neutral());
    assert!(!verdict(3, 4).neutral());
}

#[test]
fn recruitment_paths_choose_gates_siblings_but_never_the_parent_by_liveness() {
    // Arrange: a task-dead parent that still holds more ending energy than a
    // live, score-neutral sibling.
    let dead_parent = Candidate {
        live: false,
        score: 4,
        ending_energy_sum: 300.0,
    };
    let neutral_sibling = Candidate {
        live: true,
        score: 4,
        ending_energy_sum: 200.0,
    };
    let dead_sibling = Candidate {
        live: false,
        score: 8,
        ending_energy_sum: 400.0,
    };
    let children = [neutral_sibling, dead_sibling];

    // Act & Assert: only sibling candidates are dropped for being dead; the
    // parent's candidate always enters the tie, so its higher energy rejects
    // the neutral sibling under `CostSelection` and ties go to the sibling
    // under `Selection`.
    assert_eq!(choose(Policy::CostSelection, dead_parent, children), None);
    assert_eq!(choose(Policy::Selection, dead_parent, children), Some(0));
    let richer_sibling = Candidate {
        ending_energy_sum: 300.0,
        ..neutral_sibling
    };
    assert_eq!(
        choose(
            Policy::CostSelection,
            dead_parent,
            [richer_sibling, dead_sibling]
        ),
        Some(0)
    );
}

proptest! {
    #[test]
    fn recruitment_paths_retention_outcomes_count_each_outcome_and_total_them(
        outcomes in prop::collection::vec(0u8..4, 0..24)
    ) {
        let outcomes: Vec<_> = outcomes.iter().map(|outcome| match outcome {
            0 => RetentionOutcome::Useful,
            1 => RetentionOutcome::NoLongerUseful,
            2 => RetentionOutcome::Deleted,
            _ => RetentionOutcome::TaskDead,
        }).collect();
        let count = |wanted: RetentionOutcome| {
            outcomes.iter().filter(|&&outcome| outcome == wanted).count() as u32
        };

        let mut recorded = RetentionOutcomes::default();
        for &outcome in &outcomes {
            recorded.record(outcome);
        }

        prop_assert_eq!(
            recorded.clone(),
            RetentionOutcomes {
                useful: count(RetentionOutcome::Useful),
                no_longer_useful: count(RetentionOutcome::NoLongerUseful),
                deleted: count(RetentionOutcome::Deleted),
                task_dead: count(RetentionOutcome::TaskDead),
            }
        );
        prop_assert_eq!(recorded.total() as usize, outcomes.len());
    }
}

// ── T13.F07 transitions ─────────────────────────────────────────────────────

fn ladder_strategy() -> impl Strategy<Value = Ladder> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        prop::option::of(prop::sample::select(vec![
            HorizonOutcome::Retained,
            HorizonOutcome::TaskDead,
            HorizonOutcome::Deleted,
            HorizonOutcome::NoLongerUseful,
            HorizonOutcome::Despecialized,
        ])),
    )
        .prop_map(
            |(
                eligibility,
                local_edit,
                expression,
                specialized,
                proposal_specialized,
                bypass_only,
                at_primary_horizon,
            )| Ladder {
                eligibility,
                local_edit,
                expression,
                specialized,
                proposal_specialized,
                bypass_only,
                at_primary_horizon,
            },
        )
}

proptest! {
    /// Every ladder classifies to exactly one class, and that class names
    /// the first stage that failed (exclusive and total).
    #[test]
    fn recruitment_paths_classification_names_the_first_failing_stage(ladder in ladder_strategy()) {
        let class = LineageClass::of(ladder);
        let stages = [ladder.eligibility, ladder.local_edit, ladder.expression, ladder.specialized];
        let first_failure = stages.iter().position(|reached| !reached);
        match first_failure {
            Some(0) => prop_assert_eq!(class, LineageClass::NoEligibility),
            Some(1) => prop_assert_eq!(class, LineageClass::NoEdit),
            Some(2) => prop_assert_eq!(class, LineageClass::NoExpression),
            Some(_) => {
                if ladder.proposal_specialized {
                    prop_assert_eq!(class, LineageClass::Loss(LossKind::NotSelected));
                } else {
                    prop_assert_eq!(class, LineageClass::NoBenefit);
                }
            }
            None => match ladder.at_primary_horizon {
                None => prop_assert_eq!(class, LineageClass::CensoredAt64),
                Some(HorizonOutcome::Retained) => prop_assert_eq!(class, LineageClass::Retained),
                Some(outcome) => {
                    let LineageClass::Loss(kind) = class else {
                        return Err(TestCaseError::fail(format!("a lost recruit classifies as a loss: {class:?}")));
                    };
                    prop_assert_ne!(kind, LossKind::NotSelected);
                    prop_assert_eq!(format!("{kind:?}"), format!("{outcome:?}"));
                }
            },
        }
        // Retained never coexists with a failed stage.
        prop_assert_eq!(class == LineageClass::Retained, stages.iter().all(|reached| *reached)
            && ladder.at_primary_horizon == Some(HorizonOutcome::Retained));
        prop_assert!(!class_key(class).is_empty());
    }

    /// The horizon reading is total over liveness and presence and ranks
    /// task death before absence before the bypass before the components.
    #[test]
    fn recruitment_paths_horizon_outcome_ranks_death_absence_bypass_then_components(
        live in any::<bool>(), present in any::<bool>(), score_loss in -8i16..=8, holds in any::<bool>()
    ) {
        let module = present.then(|| {
            let mut module = synthetic_module_use(score_loss);
            module.specialization = Specialization {
                task_live: holds, score_gain: holds, bypass_loss: holds, ancestral_loss: holds, incumbents_preserved: holds,
            };
            module
        });
        let outcome = HorizonOutcome::of(live, module.as_ref());
        let expected = if !live { HorizonOutcome::TaskDead }
            else if !present { HorizonOutcome::Deleted }
            else if score_loss < 1 { HorizonOutcome::NoLongerUseful }
            else if !holds { HorizonOutcome::Despecialized }
            else { HorizonOutcome::Retained };
        prop_assert_eq!(outcome, expected);
    }
}

fn synthetic_module_use(score_loss: i16) -> ModuleUse {
    ModuleUse {
        node: NodeId::new(2),
        created_depth: 0,
        created_backend: ModuleBackend::Vm,
        current_backend: ModuleBackend::Vm,
        dispatched: true,
        score_loss,
        queue_effect: false,
        memory_effect: None,
        output_effect: false,
        routing_effect: false,
        payload_changed: false,
        ancestral_loss: None,
        current_ending_energy_sum: 0.0,
        ancestral_ending_energy_sum: None,
        destination_kind: DestinationKind::Vm,
        specialization: Specialization::default(),
    }
}

#[test]
fn recruitment_paths_destination_kind_reads_canonical_and_prepared_forms() {
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    let starts = starting_forms();
    let scaffold = |name: &str| {
        let start = starts.iter().find(|start| start.name == name).unwrap();
        start
            .genome
            .nodes
            .iter()
            .find(|node| node.node_id == start.scaffold)
            .unwrap()
            .backend_def
            .clone()
    };
    // The canonical blank Graph start and the dormant copy (its sensing
    // node wired into nothing) are pure with no wired effect surface; the
    // prepared Graph form carries its `Move(E)` vote edge.
    assert_eq!(
        DestinationKind::of(&scaffold("graph_blank")),
        DestinationKind::GraphPureNoEffect
    );
    assert_eq!(
        DestinationKind::of(&scaffold("graph_copy")),
        DestinationKind::GraphPureNoEffect
    );
    assert_eq!(
        DestinationKind::of(&scaffold("graph_prepared")),
        DestinationKind::GraphPureWithEffect
    );
    for name in ["vm_blank", "vm_copy", "vm_prepared"] {
        assert_eq!(DestinationKind::of(&scaffold(name)), DestinationKind::Vm);
    }
    // Statefulness: a stateful kind, a self reference, a forward reference,
    // a previous-tick memory read, or plasticity.
    let BackendDef::Graph(pure) = scaffold("graph_prepared") else {
        unreachable!()
    };
    let edge = |source| GraphEdge {
        source,
        weight: 1.0,
    };
    let mut stateful_kind = pure.clone();
    stateful_kind.compute_nodes[0].kind = ComputeNodeKind::Momentum(0.5);
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(stateful_kind)),
        DestinationKind::GraphStateful
    );
    let mut self_reference = pure.clone();
    self_reference.compute_nodes[0]
        .inputs
        .push(edge(GraphSource::ComputeNode(0)));
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(self_reference)),
        DestinationKind::GraphStateful
    );
    let mut forward = pure.clone();
    forward.compute_nodes[0]
        .inputs
        .push(edge(GraphSource::ComputeNode(1)));
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(forward)),
        DestinationKind::GraphStateful
    );
    let mut backward = pure.clone();
    backward.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Relu,
        inputs: vec![edge(GraphSource::ComputeNode(0))],
        plasticity: None,
    });
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(backward)),
        DestinationKind::GraphPureWithEffect
    );
    let mut previous = pure.clone();
    previous.output_sinks[0]
        .inputs
        .push(edge(GraphSource::SharedMemory {
            slot: 0,
            previous: true,
        }));
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(previous)),
        DestinationKind::GraphStateful
    );
    let mut current_memory = pure.clone();
    current_memory.output_sinks[0]
        .inputs
        .push(edge(GraphSource::SharedMemory {
            slot: 0,
            previous: false,
        }));
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(current_memory)),
        DestinationKind::GraphPureWithEffect
    );
    let mut plastic = pure;
    plastic.compute_nodes[0].plasticity = Some(crate::creature::genome::PlasticityConfig {
        rule: crate::creature::genome::HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 1.0,
        lamarckian: false,
        modulation: None,
    });
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(plastic)),
        DestinationKind::GraphStateful
    );
}

#[test]
fn recruitment_paths_destination_kind_counts_total_every_module() {
    let mut counts = DestinationKindCounts::default();
    for kind in [
        DestinationKind::Vm,
        DestinationKind::GraphStateful,
        DestinationKind::GraphStateful,
        DestinationKind::GraphPureNoEffect,
        DestinationKind::GraphPureWithEffect,
    ] {
        counts.record(kind);
    }
    let mut pooled = counts;
    pooled.merge(counts);
    assert_eq!((counts.total(), pooled.total()), (5, 10));
    assert_eq!(pooled.graph_stateful, 4);
}

/// A verbatim copy of the correct incumbent, routed in front of it, is
/// useful by bypass on both backends but reads an ancestral loss of zero:
/// copied computation alone never scores as specialization.
#[test]
fn recruitment_paths_verbatim_copy_reads_zero_ancestral_loss_on_each_backend() {
    use crate::mutation::topology::TopologyOperator;
    use crate::neighborhood::recruitment::{BirthObservation, RecruitmentTracker};
    for backend in [ModuleBackend::Graph, ModuleBackend::Vm] {
        let base = fixtures::base(backend, true);
        let baseline = evaluate(&base);
        assert_eq!(baseline.correct(Task::A), 8);
        let mut copied = base.clone();
        let copy = fixtures::topology(&mut copied, TopologyOperator::CopyNode, 7);
        let birth = copied.nodes[2].backend_def.clone();
        assert_eq!(birth, base.nodes[1].backend_def);
        let mut activated = copied.clone();
        activated.entry_node_id = NodeId::new(2);
        let reading = payload_reading("verbatim", &activated, &birth, &baseline, Task::A);
        assert!(reading.specialization.task_live, "{backend:?}");
        assert!(!reading.payload_changed, "{backend:?}");
        assert_eq!(reading.score, 8, "{backend:?}");
        assert!(reading.bypass_loss >= 1, "{backend:?}: {reading:?}");
        assert!(reading.specialization.bypass_loss);
        assert_eq!(reading.ancestral_loss, 0, "{backend:?}");
        assert!(!reading.specialization.ancestral_loss);
        assert!(!reading.specialization.holds());
        // The same payload read through the tracker path is the same zero,
        // with the current energy standing in for the unevaluated replacement.
        let mut tracker = RecruitmentTracker::new(1);
        tracker.seed_founder(0, &base.nodes);
        tracker.record_birth(BirthObservation {
            lineage: 0,
            depth: 1,
            after: &copied.nodes,
            summary: &copy,
        });
        tracker.record_birth(BirthObservation {
            lineage: 0,
            depth: 2,
            after: &activated.nodes,
            summary: &crate::mutation::MutationSummary::zero(),
        });
        let uses = super::experiment::uses(
            &activated,
            &evaluate(&activated),
            Task::A,
            &baseline,
            &tracker,
            &task_config(),
        );
        assert_eq!(uses.len(), 1);
        assert!(uses[0].dispatched);
        assert!(!uses[0].payload_changed);
        assert_eq!(uses[0].ancestral_loss, Some(0));
        assert_eq!(
            uses[0].ancestral_ending_energy_sum,
            Some(uses[0].current_ending_energy_sum)
        );
        assert!(!uses[0].specialization.holds());
    }
}

/// The qualified copy paths on each backend end in a step whose payload
/// diverged from birth and loses at least one scene when restored, with the
/// generation-0 correct scenes preserved: the ancestral test scores real
/// specialization.
#[test]
fn recruitment_paths_qualified_copy_paths_specialize_by_the_ancestral_reading() {
    let mut seen = Vec::new();
    for path in paths().iter().filter(|path| path.qualified()) {
        let readings = path.payload_readings();
        assert_eq!(readings.len(), path.steps.len());
        for (reading, step) in readings.iter().zip(&path.steps) {
            assert_eq!(reading.step, step.stage.name);
            assert_eq!(reading.score, step.stage.task.correct(path.task));
            assert_eq!(reading.specialization.bypass_loss, reading.bypass_loss >= 1);
            assert_eq!(
                reading.specialization.ancestral_loss,
                reading.ancestral_loss >= 1
            );
            if !reading.payload_changed {
                assert_eq!(reading.ancestral_loss, 0, "{} {}", path.form, reading.step);
            }
        }
        let last = readings.last().unwrap();
        if last.specialization.holds() {
            seen.push((path.backend, path.form.clone()));
        }
    }
    assert!(
        seen.iter()
            .any(|(backend, _)| *backend == ModuleBackend::Graph),
        "{seen:?}"
    );
    assert!(
        seen.iter()
            .any(|(backend, _)| *backend == ModuleBackend::Vm),
        "{seen:?}"
    );
    for form in ["graph_copy", "vm_copy"] {
        let path = paths().iter().find(|path| path.form == form).unwrap();
        let last = path.payload_readings().pop().unwrap();
        assert!(last.payload_changed, "{form}");
        assert!(last.ancestral_loss >= 1, "{form}: {last:?}");
        assert!(last.specialization.incumbents_preserved, "{form}");
        assert!(last.specialization.holds(), "{form}: {last:?}");
    }
}

#[test]
fn recruitment_paths_task_reading_route_variation_separates_position_from_destination() {
    let scene = |routing: Vec<(NodeId, usize, NodeId)>| Scene {
        food: [false; 3],
        correct_a: true,
        correct_b: false,
        survived: true,
        energy: Some(1.0),
        maintenance: 0.0,
        carrying: 0.0,
        work: Work::default(),
        actions: vec![],
        position: None,
        dispatched: routing.iter().map(|route| route.0).collect(),
        routing,
        output_slots: vec![],
        shared_memory: None,
    };
    let router = NodeId::new(0);
    let same_destination = TaskReading {
        scenes: vec![
            scene(vec![(router, 0, NodeId::new(1))]),
            scene(vec![(router, 1, NodeId::new(1))]),
        ],
    };
    assert!(same_destination.route_position_varies());
    assert!(!same_destination.route_destination_varies());
    let different = TaskReading {
        scenes: vec![
            scene(vec![(router, 0, NodeId::new(1))]),
            scene(vec![(router, 0, NodeId::new(2))]),
        ],
    };
    assert!(!different.route_position_varies());
    assert!(different.route_destination_varies());
    let fixed = TaskReading {
        scenes: vec![
            scene(vec![(router, 0, NodeId::new(1))]),
            scene(vec![]),
            scene(vec![(router, 0, NodeId::new(1))]),
        ],
    };
    assert!(!fixed.route_position_varies());
    assert!(!fixed.route_destination_varies());
    assert_eq!(fixed.scenes_dispatched(), BTreeMap::from([(router, 2)]));
}

/// Every transition reading of the reduced legacy run is consistent with
/// its lineage's ladder, discoveries and horizons, and the +64 horizon is
/// censored on this panel.
#[test]
fn recruitment_paths_reduced_run_transitions_are_consistent() {
    use crate::neighborhood::recruitment::{payload_hash, CohortFact};
    let report = observe(Sizes::TEST);
    assert_eq!(report.supply, Supply::Legacy);
    assert_eq!(report.supply_rule, Supply::Legacy.rule());
    for arm in &report.arms {
        let transitions = &arm.summary.transitions;
        assert_eq!(transitions.lineages, arm.lineages.len() as u32);
        assert_eq!(
            transitions.classes.values().sum::<u32>(),
            transitions.lineages
        );
        assert!(!transitions.retained_at.contains_key(&PRIMARY_HORIZON));
        assert_eq!(
            transitions.eligibility,
            arm.lineages
                .iter()
                .filter(|lineage| lineage.ladder.eligibility)
                .count() as u32
        );
        for lineage in &arm.lineages {
            let ladder = lineage.ladder;
            assert_eq!(lineage.classification, LineageClass::of(ladder));
            assert_eq!(lineage.task, arm.task);
            assert_eq!(ladder.specialized, lineage.specialized_discovery.is_some());
            assert_eq!(
                lineage.discovery_checkpoint.as_ref().map(|c| c.generation),
                lineage.specialized_discovery.as_ref().map(|d| d.generation)
            );
            assert!(ladder.eligibility || !ladder.local_edit);
            assert!(ladder.proposal_specialized || !ladder.specialized);
            assert_eq!(ladder.at_primary_horizon, None);
            for horizon in &lineage.horizons {
                let discovery = lineage.specialized_discovery.as_ref().unwrap();
                assert!(HORIZONS.contains(&horizon.offset));
                assert_eq!(horizon.at_generation, discovery.generation + horizon.offset);
                assert_eq!(
                    horizon.outcome,
                    HorizonOutcome::of(horizon.live, horizon.module.as_ref())
                );
            }
            for proposal in &lineage.proposals {
                assert!(!proposal.specialized || proposal.discovery);
                for module in &proposal.useful_modules {
                    assert!(module.dispatched && module.specialization.bypass_loss);
                    assert!(module.ancestral_loss.is_some());
                    if !module.payload_changed {
                        assert_eq!(module.ancestral_loss, Some(0));
                    }
                }
            }
            // Generation 0: every starting payload is its own birth payload,
            // so authored preparation can never read as an ancestral loss.
            let first = &lineage.checkpoints[0];
            assert!(first.task_use.iter().all(|module| !module.payload_changed));
            for module in &first.modules {
                let node = first
                    .genome
                    .nodes
                    .iter()
                    .find(|node| node.node_id == module.module.node);
                if let Some(node) = node.filter(|_| module.module.is_present()) {
                    assert_eq!(
                        module.module.birth_payload_hash,
                        payload_hash(&node.backend_def),
                        "{} {:?}",
                        arm.start,
                        module.module.node
                    );
                }
            }
            for checkpoint in lineage
                .checkpoints
                .iter()
                .chain(&lineage.discovery_checkpoint)
            {
                assert_eq!(
                    checkpoint.destination_kinds.total(),
                    checkpoint.task_use.len() as u64
                );
                assert_eq!(
                    u64::from(checkpoint.eligible_site_fraction.denominator),
                    checkpoint.cohort.lineage_rows[0].created
                );
                assert_eq!(
                    checkpoint.route_position_varies,
                    checkpoint.task.route_position_varies()
                );
                assert_eq!(
                    checkpoint.route_destination_varies,
                    checkpoint.task.route_destination_varies()
                );
                for module in &checkpoint.modules {
                    assert_eq!(
                        module.birth_payload.is_some(),
                        module.module.first(CohortFact::Dispatch).is_some()
                    );
                    if let Some(payload) = &module.birth_payload {
                        assert_eq!(payload_hash(payload), module.module.birth_payload_hash);
                    }
                }
            }
        }
    }
}

/// Two observations at the same sizes are byte-identical: the assay has no
/// hidden state.
#[test]
fn recruitment_paths_legacy_panel_observation_is_deterministic() {
    let first = serde_json::to_vec(&observe(Sizes::TEST)).unwrap();
    let second = serde_json::to_vec(&observe(Sizes::TEST)).unwrap();
    assert_eq!(first, second);
}

#[test]
fn recruitment_paths_panels_enforce_their_size_caps() {
    assert_eq!(Panel::legacy(Sizes::PRODUCTION).supply, Supply::Legacy);
    assert_eq!(Panel::s0(Sizes::S0).supply, Supply::Production);
    assert_eq!(Panel::s0(Sizes::S0_PILOT).version(), S0_VERSION);
    assert_eq!(Panel::legacy(Sizes::TEST).version(), VERSION);
    assert_eq!(Sizes::S0.proposals(), 1_769_472);
    assert_eq!(Sizes::S0_PILOT.proposals(), 55_296);
    assert!(!Sizes::S0.fits(Sizes::PRODUCTION));
    assert!(Sizes::S0_PILOT.fits(Sizes::S0));
    for sizes in [
        Sizes {
            lineages: 17,
            ..Sizes::S0
        },
        Sizes {
            discovery: 257,
            ..Sizes::S0
        },
        Sizes {
            followup: 0,
            ..Sizes::S0
        },
        Sizes {
            batches: 5,
            ..Sizes::S0
        },
    ] {
        assert!(std::panic::catch_unwind(move || Panel::s0(sizes)).is_err());
    }
    assert!(std::panic::catch_unwind(|| Panel::legacy(Sizes::S0)).is_err());
    assert!(Supply::Production.mutation_config().per_unit_supply_enabled);
    assert!(!Supply::Legacy.mutation_config().per_unit_supply_enabled);
}

/// The S0 panel path: the compact record replays to identical fingerprints
/// from the initial genome, the chosen chain and the seeds; the chosen deltas
/// rebuild the final checkpoint genome; a pilot-sized panel's lineage is
/// byte-identical to the same lineage inside a larger panel.
#[test]
fn recruitment_paths_s0_compact_records_replay_and_prefix_the_panel() {
    let tiny = Sizes {
        batches: 1,
        lineages: 1,
        discovery: 2,
        followup: 1,
    };
    let assay = Assay::new(Panel::s0(tiny));
    assert_eq!(assay.arm_count(), ARMS);
    assert_eq!(assay.arm(0).0.name, "graph_blank");
    assert_eq!(assay.arm(0).1, Policy::Drift);
    let wider = Assay::new(Panel::s0(Sizes {
        lineages: 2,
        batches: 2,
        ..tiny
    }));
    for arm in [0, 13, ARMS - 1] {
        let record = assay.compact_lineage(arm, 0, 0);
        assert_eq!(record.arm, arm);
        assert_eq!(record.proposals.len(), 6);
        assert_eq!(
            record
                .proposals
                .iter()
                .filter(|p| p.delta.is_some())
                .count(),
            record.proposals.iter().filter(|p| p.chosen).count()
        );
        let check = assay.replay(&record);
        assert_eq!(check.proposals, 6);
        assert_eq!(check.matched, 6, "{check:?}");
        assert_eq!(check.first_mismatch, None);
        let start = assay.arm(arm).0;
        let mut genome = start.genome.clone();
        for proposal in record.proposals.iter().filter(|p| p.chosen) {
            genome = proposal.delta.as_ref().unwrap().apply(&genome).unwrap();
        }
        assert_eq!(genome, record.checkpoints.last().unwrap().genome);
        let inside = wider.compact_lineage(arm, 0, 0);
        assert_eq!(
            serde_json::to_vec(&record).unwrap(),
            serde_json::to_vec(&inside).unwrap()
        );
        // A different lineage index draws different seeds.
        let other = wider.compact_lineage(arm, 1, 1);
        assert_ne!(other.proposals[0].seed, record.proposals[0].seed);
        // A corrupted fingerprint is caught by the replay.
        let mut corrupted = record.clone();
        corrupted.proposals[3].mutation_fingerprint.clear();
        let check = assay.replay(&corrupted);
        assert_eq!((check.matched, check.first_mismatch), (5, Some((2, 1))));
        let facts = LineageFacts::of_compact(&record);
        let full = assay.lineage(arm, 0, 0);
        assert_eq!(facts, LineageFacts::of(&full));
        let (arm_summary, batches) = summaries(&[facts], 1);
        assert_eq!(arm_summary.transitions.lineages, 1);
        assert_eq!(batches.len(), 1);
    }
}

/// The S0 panel's arm 22 (`vm_copy` / CostSelection) batch 0 lineage 3
/// first specializes at generation 106 on the T19.F04 vote founder
/// (`docs/progress/readings/t19-f04.md`).
/// Seeds depend on `(batch, lineage, generation, sibling)` only, so the same
/// chain replays at sizes that just reach the +64 primary horizon, and the
/// lineage classifies `Retained` through `specialized_horizons` →
/// `at_primary_horizon`, not through a ladder default.
#[test]
fn recruitment_paths_known_specializing_lineage_classifies_retained_at_the_primary_horizon() {
    let sizes = Sizes {
        batches: 2,
        lineages: 14,
        discovery: 106,
        followup: PRIMARY_HORIZON,
    };
    let assay = Assay::new(Panel::s0(sizes));
    let (start, policy) = assay.arm(22);
    assert_eq!(
        (start.name.as_str(), policy),
        ("vm_copy", Policy::CostSelection)
    );

    let lineage = assay.lineage(22, 0, 3);

    let discovery = lineage.specialized_discovery.as_ref().unwrap();
    assert_eq!(discovery.generation, 106);
    assert!(discovery.module.specialization.holds());
    assert_eq!(
        lineage.discovery_checkpoint.as_ref().map(|c| c.generation),
        Some(106)
    );
    let horizons: Vec<_> = lineage
        .horizons
        .iter()
        .map(|h| (h.offset, h.at_generation, h.outcome))
        .collect();
    assert_eq!(
        horizons,
        [
            (16, 122, HorizonOutcome::Retained),
            (PRIMARY_HORIZON, 170, HorizonOutcome::Retained)
        ]
    );
    assert!(lineage.ladder.specialized);
    assert!(lineage.ladder.proposal_specialized);
    assert_eq!(
        lineage.ladder.at_primary_horizon,
        Some(HorizonOutcome::Retained)
    );
    assert_eq!(lineage.classification, LineageClass::Retained);
    // The recruit scores 8 against a bypass and a birth-payload replacement
    // that both score 4: the two counterfactual losses are exact differences.
    assert!(discovery.module.payload_changed);
    assert_eq!(discovery.module.score_loss, 4);
    assert_eq!(discovery.module.ancestral_loss, Some(4));
    let retention = lineage.retention.as_ref().unwrap();
    assert_eq!(
        (
            retention.at_generation,
            retention.outcome,
            retention.score_loss
        ),
        (170, RetentionOutcome::Useful, Some(4))
    );
    // Every specialized proposal is counted once in the lineage facts.
    let specialized = lineage
        .proposals
        .iter()
        .filter(|proposal| proposal.specialized)
        .count() as u64;
    assert_eq!(specialized, 129);
    assert_eq!(
        LineageFacts::of(&lineage).specialized_proposals,
        specialized
    );
}

/// Prints the readings-file table of `QualifiedPath::payload_readings()`:
/// one row per qualified path and step, then the two verbatim-copy controls.
/// Run with `cargo test -p v3-core recruitment_paths_print_payload_readings_table
/// -- --ignored --nocapture`.
#[test]
#[ignore = "prints the T13.F07 readings table; not an assertion"]
fn recruitment_paths_print_payload_readings_table() {
    use crate::mutation::topology::TopologyOperator;
    let row = |form: &str, backend: ModuleBackend, task: Task, reading: &StepReading| {
        let s = reading.specialization;
        println!(
            "| {form} | {backend:?} | {task:?} | {} | {} | {} | {} | {} | {} / {} / {} / {} / {} | {} |",
            reading.step,
            reading.score,
            reading.payload_changed,
            reading.bypass_loss,
            reading.ancestral_loss,
            s.task_live,
            s.score_gain,
            s.bypass_loss,
            s.ancestral_loss,
            s.incumbents_preserved,
            s.holds()
        );
    };
    println!(
        "| Form | Backend | Task | Step | Score | payload_changed | bypass_loss | ancestral_loss | task_live / score_gain / bypass_loss / ancestral_loss / incumbents_preserved | holds |"
    );
    println!("| --- | --- | --- | --- | ---: | --- | ---: | ---: | --- | --- |");
    for path in paths().iter().filter(|path| path.qualified()) {
        for reading in path.payload_readings() {
            row(&path.form, path.backend, path.task, &reading);
        }
    }
    for backend in [ModuleBackend::Graph, ModuleBackend::Vm] {
        let base = fixtures::base(backend, true);
        let mut copied = base.clone();
        fixtures::topology(&mut copied, TopologyOperator::CopyNode, 7);
        let birth = copied.nodes[2].backend_def.clone();
        copied.entry_node_id = NodeId::new(2);
        let reading = payload_reading("verbatim", &copied, &birth, &evaluate(&base), Task::A);
        row("verbatim_copy (control)", backend, Task::A, &reading);
    }
}

/// Each reduced-run ladder flag agrees with the record it summarizes: a
/// local edit names a cohort module, expression is a dispatched cohort
/// module, bypass-only is a retained zero-ancestral-loss module, retention
/// reads the discovery module's presence, and generation 0 reads no score
/// gain against itself.
#[test]
fn recruitment_paths_reduced_run_ladders_agree_with_their_records() {
    let report = observe(Sizes::TEST);
    for arm in &report.arms {
        for lineage in &arm.lineages {
            let ladder = lineage.ladder;
            let label = format!("{} {:?}", arm.start, arm.policy);
            let cohort_nodes: std::collections::BTreeSet<NodeId> = lineage
                .checkpoints
                .iter()
                .flat_map(|checkpoint| &checkpoint.modules)
                .filter(|module| module.module.provenance.is_cohort())
                .map(|module| module.module.node)
                .collect();
            let chosen = || lineage.proposals.iter().filter(|proposal| proposal.chosen);
            let cohort_edit = chosen().flat_map(|proposal| &proposal.events).any(|event| {
                event.outcome.starts_with("Applied")
                    && event
                        .target
                        .is_some_and(|target| cohort_nodes.contains(&target))
            });
            assert!(!ladder.local_edit || cohort_edit, "{label}");
            let last = lineage.checkpoints.last().unwrap();
            let expressed = last.modules.iter().any(|module| {
                module.module.provenance.is_cohort() && module.module.scenes_dispatched >= 1
            });
            assert_eq!(ladder.expression, expressed, "{label}");
            let bypass_only = chosen()
                .flat_map(|proposal| &proposal.useful_modules)
                .any(|module| module.ancestral_loss == Some(0));
            assert_eq!(ladder.bypass_only, bypass_only, "{label}");
            if let Some(retention) = &lineage.retention {
                let checkpoint = lineage
                    .checkpoints
                    .iter()
                    .find(|checkpoint| checkpoint.generation == retention.at_generation)
                    .unwrap();
                let present = checkpoint.modules.iter().any(|module| {
                    module.module.node == retention.discovery.module.node
                        && module.module.created_depth == retention.discovery.module.created_depth
                        && module.module.is_present()
                });
                assert_eq!(retention.score_loss.is_some(), present, "{label}");
                assert_eq!(
                    retention.outcome,
                    classify_retention(checkpoint.task.live(), retention.score_loss),
                    "{label}"
                );
            }
            let first = &lineage.checkpoints[0];
            assert!(!first.task_use.is_empty(), "{label}");
            for module in &first.task_use {
                assert!(!module.specialization.score_gain, "{label}");
                assert!(module.specialization.incumbents_preserved, "{label}");
            }
        }
    }
}

fn synthetic_facts(
    lineage: u32,
    ladder: Ladder,
    horizons: Vec<(u32, HorizonOutcome)>,
    specialized_proposals: u64,
    final_applicable: (u64, u64),
    destination_kinds: DestinationKindCounts,
) -> LineageFacts {
    LineageFacts {
        batch: 0,
        lineage,
        proposals: vec![],
        specialized_proposals,
        checkpoints: vec![],
        proposal_discovery: None,
        retained_discovery: None,
        specialized_discovery: None,
        viable_retained_discovery: false,
        retention: None,
        horizons,
        ladder,
        classification: LineageClass::of(ladder),
        final_applicable,
        destination_kinds,
    }
}

/// The arm transitions count each ladder flag, horizon retention, class,
/// specialized proposal and destination kind across lineages, and pool the
/// eligible-site fraction.
#[test]
fn recruitment_paths_transitions_count_every_ladder_flag_across_lineages() {
    let retained = Ladder {
        eligibility: true,
        local_edit: true,
        expression: true,
        specialized: true,
        proposal_specialized: true,
        bypass_only: true,
        at_primary_horizon: Some(HorizonOutcome::Retained),
    };
    let not_selected = Ladder {
        specialized: false,
        at_primary_horizon: None,
        ..retained
    };
    let no_edit = Ladder {
        eligibility: true,
        ..Ladder::default()
    };
    let kinds = |vm, graph_stateful, graph_pure_with_effect| DestinationKindCounts {
        vm,
        graph_stateful,
        graph_pure_no_effect: 0,
        graph_pure_with_effect,
    };
    let facts = [
        synthetic_facts(
            0,
            retained,
            vec![
                (16, HorizonOutcome::Retained),
                (64, HorizonOutcome::Retained),
            ],
            3,
            (2, 3),
            kinds(1, 2, 0),
        ),
        synthetic_facts(
            1,
            not_selected,
            vec![
                (16, HorizonOutcome::TaskDead),
                (64, HorizonOutcome::Retained),
            ],
            2,
            (1, 1),
            kinds(2, 0, 1),
        ),
        synthetic_facts(
            2,
            no_edit,
            vec![(16, HorizonOutcome::Retained)],
            0,
            (0, 2),
            kinds(0, 0, 0),
        ),
    ];

    let transitions = summary(&facts).transitions;

    assert_eq!(
        transitions,
        Transitions {
            lineages: 3,
            eligibility: 3,
            local_edit: 2,
            expression: 2,
            specialized: 1,
            proposal_specialized_lineages: 2,
            specialized_proposals: 5,
            retained_at: BTreeMap::from([(16, 2), (64, 2)]),
            classes: BTreeMap::from([
                ("retained".into(), 1),
                ("loss_not_selected".into(), 1),
                ("no_edit".into(), 1),
            ]),
            bypass_only: 2,
            eligible_site_fraction: estimate(3, 6),
            destination_kinds: kinds(3, 2, 1),
        }
    );
}

#[test]
fn recruitment_paths_class_keys_name_every_class() {
    let keys: Vec<String> = [
        LineageClass::Retained,
        LineageClass::NoEligibility,
        LineageClass::NoEdit,
        LineageClass::NoExpression,
        LineageClass::NoBenefit,
        LineageClass::Loss(LossKind::NotSelected),
        LineageClass::Loss(LossKind::Deleted),
        LineageClass::Loss(LossKind::Despecialized),
        LineageClass::Loss(LossKind::TaskDead),
        LineageClass::Loss(LossKind::NoLongerUseful),
        LineageClass::CensoredAt64,
    ]
    .into_iter()
    .map(class_key)
    .collect();
    assert_eq!(
        keys,
        [
            "retained",
            "no_eligibility",
            "no_edit",
            "no_expression",
            "no_benefit",
            "loss_not_selected",
            "loss_deleted",
            "loss_despecialized",
            "loss_task_dead",
            "loss_no_longer_useful",
            "censored_at_64",
        ]
    );
}

#[test]
fn recruitment_paths_replay_check_merge_sums_counts_and_keeps_the_first_mismatch() {
    let mut merged = ReplayCheck {
        proposals: 5,
        matched: 3,
        first_mismatch: None,
    };
    merged.merge(ReplayCheck {
        proposals: 4,
        matched: 2,
        first_mismatch: Some((2, 1)),
    });
    merged.merge(ReplayCheck {
        proposals: 7,
        matched: 7,
        first_mismatch: Some((1, 0)),
    });
    assert_eq!(
        merged,
        ReplayCheck {
            proposals: 16,
            matched: 12,
            first_mismatch: Some((2, 1)),
        }
    );
}

#[test]
fn recruitment_paths_assay_exposes_the_nine_starts_in_order() {
    let assay = Assay::new(Panel::s0(Sizes::S0_PILOT));
    assert_eq!(
        assay
            .starts()
            .iter()
            .map(|start| start.name.as_str())
            .collect::<Vec<_>>(),
        [
            "graph_blank",
            "graph_copy",
            "graph_split",
            "vm_blank",
            "vm_copy",
            "graph_unprepared",
            "graph_prepared",
            "vm_unprepared",
            "vm_prepared",
        ]
    );
}

#[test]
fn recruitment_paths_sizes_reject_every_zero_dimension_and_name_the_supply_rules() {
    let zero = |sizes: Sizes| sizes.fits(Sizes::S0);
    assert!(zero(Sizes::S0_PILOT));
    assert!(!zero(Sizes {
        batches: 0,
        ..Sizes::S0_PILOT
    }));
    assert!(!zero(Sizes {
        lineages: 0,
        ..Sizes::S0_PILOT
    }));
    assert!(!zero(Sizes {
        discovery: 0,
        ..Sizes::S0_PILOT
    }));
    assert!(!zero(Sizes {
        followup: 0,
        ..Sizes::S0_PILOT
    }));
    assert_eq!(
        Supply::Legacy.rule(),
        "legacy per-birth supply (per_unit_supply_enabled forced false)"
    );
    assert_eq!(
        Supply::Production.rule(),
        "production per-unit supply on the child's own genome_size()"
    );
}

#[test]
fn recruitment_paths_destination_kind_counts_merge_asymmetric_pools() {
    let mut pooled = DestinationKindCounts {
        vm: 1,
        graph_stateful: 2,
        graph_pure_no_effect: 3,
        graph_pure_with_effect: 4,
    };
    pooled.merge(DestinationKindCounts {
        vm: 5,
        graph_stateful: 1,
        graph_pure_no_effect: 2,
        graph_pure_with_effect: 3,
    });
    assert_eq!(
        pooled,
        DestinationKindCounts {
            vm: 6,
            graph_stateful: 3,
            graph_pure_no_effect: 5,
            graph_pure_with_effect: 7,
        }
    );
}

#[test]
fn recruitment_paths_specialization_holds_only_with_every_component() {
    let all = Specialization {
        task_live: true,
        score_gain: true,
        bypass_loss: true,
        ancestral_loss: true,
        incumbents_preserved: true,
    };
    assert!(all.holds());
    let missing = [
        Specialization {
            task_live: false,
            ..all
        },
        Specialization {
            score_gain: false,
            ..all
        },
        Specialization {
            bypass_loss: false,
            ..all
        },
        Specialization {
            ancestral_loss: false,
            ..all
        },
        Specialization {
            incumbents_preserved: false,
            ..all
        },
    ];
    for specialization in missing {
        assert!(!specialization.holds(), "{specialization:?}");
    }
}

#[test]
fn recruitment_paths_incumbents_are_preserved_only_when_no_correct_scene_is_lost() {
    let reading = |correct: [bool; 2]| TaskReading {
        scenes: correct
            .into_iter()
            .map(|correct_a| Scene {
                food: [false; 3],
                correct_a,
                correct_b: !correct_a,
                survived: true,
                energy: Some(1.0),
                maintenance: 0.0,
                carrying: 0.0,
                work: Work::default(),
                actions: vec![],
                position: None,
                dispatched: vec![],
                routing: vec![],
                output_slots: vec![],
                shared_memory: None,
            })
            .collect(),
    };
    let baseline = reading([true, false]);
    assert!(reading([true, false]).preserves_correct_scenes(&baseline, Task::A));
    assert!(reading([true, true]).preserves_correct_scenes(&baseline, Task::A));
    assert!(!reading([false, false]).preserves_correct_scenes(&baseline, Task::A));
    assert!(!reading([false, true]).preserves_correct_scenes(&baseline, Task::A));
    // Task B is correct only in the second scene of the baseline.
    assert!(!reading([true, true]).preserves_correct_scenes(&baseline, Task::B));
    assert!(reading([false, false]).preserves_correct_scenes(&baseline, Task::B));
    // Nothing correct at baseline: nothing to lose.
    let empty = reading([false, false]);
    assert!(reading([false, false]).preserves_correct_scenes(&empty, Task::A));
}

/// A wired vote sink alone is an effect surface (T19.F04).
#[test]
fn recruitment_paths_destination_kind_reads_a_wired_vote_sink_alone_as_an_effect() {
    let starts = starting_forms();
    let start = starts
        .iter()
        .find(|start| start.name == "graph_prepared")
        .unwrap();
    let BackendDef::Graph(prepared) = start
        .genome
        .nodes
        .iter()
        .find(|node| node.node_id == start.scaffold)
        .unwrap()
        .backend_def
        .clone()
    else {
        unreachable!()
    };
    assert!(!fixtures::move_sink(&prepared, fixtures::EAST)
        .inputs
        .is_empty());
    let mut inert = prepared.clone();
    for sink in &mut inert.output_sinks {
        sink.inputs.clear();
    }
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(inert.clone())),
        DestinationKind::GraphPureNoEffect
    );
    let mut vote_only = inert;
    vote_only
        .output_sinks
        .iter_mut()
        .find(|sink| {
            sink.kind
                == crate::creature::genome::cgp::OutputSinkKind::ActionVote(
                    crate::creature::genome::vote::VoteSink::Terminate,
                )
        })
        .unwrap()
        .inputs
        .push(crate::creature::genome::cgp::GraphEdge {
            source: crate::creature::genome::cgp::GraphSource::ComputeNode(0),
            weight: 1.0,
        });
    assert_eq!(
        DestinationKind::of(&BackendDef::Graph(vote_only)),
        DestinationKind::GraphPureWithEffect
    );
}

/// The readings-file table of `QualifiedPath::payload_readings()`, pinned:
/// every step before the last scores 4 with no bypass or ancestral loss and
/// no score gain; every useful last step scores 8 with bypass loss 4 and an
/// ancestral loss of 4 (6 on the unprepared forms), all components holding.
#[test]
fn recruitment_paths_qualified_payload_readings_match_the_recorded_table() {
    let last_ancestral_loss = |form: &str| match form {
        "graph_unprepared" | "vm_unprepared" => 6,
        _ => 4,
    };
    let mut forms = 0;
    for path in paths().iter().filter(|path| path.qualified()) {
        forms += 1;
        let readings = path.payload_readings();
        let (last, earlier) = readings.split_last().unwrap();
        for reading in earlier {
            let label = format!("{} {}", path.form, reading.step);
            assert_eq!(
                (reading.score, reading.bypass_loss, reading.ancestral_loss),
                (4, 0, 0),
                "{label}"
            );
            assert_eq!(
                reading.specialization,
                Specialization {
                    task_live: true,
                    score_gain: false,
                    bypass_loss: false,
                    ancestral_loss: false,
                    incumbents_preserved: true,
                },
                "{label}"
            );
        }
        assert_eq!(
            (last.score, last.bypass_loss, last.ancestral_loss),
            (8, 4, last_ancestral_loss(&path.form)),
            "{}",
            path.form
        );
        assert!(last.payload_changed, "{}", path.form);
        assert_eq!(
            last.specialization,
            Specialization {
                task_live: true,
                score_gain: true,
                bypass_loss: true,
                ancestral_loss: true,
                incumbents_preserved: true,
            },
            "{}",
            path.form
        );
    }
    assert_eq!(forms, 9);
}

/// The same arm 22 lineage replayed to a +16 follow-up only: its +16 horizon
/// is retained, the primary horizon is unobserved, and the class is censored
/// rather than retained.
#[test]
fn recruitment_paths_known_specializing_lineage_is_censored_before_the_primary_horizon() {
    let sizes = Sizes {
        batches: 2,
        lineages: 14,
        discovery: 106,
        followup: 16,
    };
    let assay = Assay::new(Panel::s0(sizes));

    let lineage = assay.lineage(22, 0, 3);

    assert_eq!(
        lineage.specialized_discovery.as_ref().map(|d| d.generation),
        Some(106)
    );
    assert_eq!(
        lineage
            .horizons
            .iter()
            .map(|h| (h.offset, h.at_generation, h.outcome))
            .collect::<Vec<_>>(),
        [(16, 122, HorizonOutcome::Retained)]
    );
    assert!(lineage.ladder.specialized);
    assert_eq!(lineage.ladder.at_primary_horizon, None);
    assert_eq!(lineage.classification, LineageClass::CensoredAt64);
}

/// S0 arm 12 (`graph_prepared` / Drift) batch 2 lineage 2 retains its F06
/// discovery (module 2) at generation 9 and deletes that module at depth 51
/// on the T19.F04 vote founder (`docs/progress/readings/t19-f04.md`), so the
/// T13.F06 retention reading at +256 is `Deleted` with no score loss even
/// though the founder modules at the same creation depth are still present.
#[test]
fn recruitment_paths_known_deleting_lineage_reads_deleted_retention() {
    let sizes = Sizes {
        batches: 3,
        lineages: 3,
        discovery: 9,
        followup: 256,
    };
    let assay = Assay::new(Panel::s0(sizes));
    let (start, policy) = assay.arm(12);
    assert_eq!(
        (start.name.as_str(), policy),
        ("graph_prepared", Policy::Drift)
    );

    let lineage = assay.lineage(12, 2, 2);

    let retention = lineage.retention.as_ref().unwrap();
    assert_eq!(retention.discovery.generation, 9);
    assert_eq!(retention.discovery.module.node, NodeId::new(2));
    assert_eq!(retention.at_generation, 265);
    assert_eq!(
        (retention.outcome, retention.score_loss),
        (RetentionOutcome::Deleted, None)
    );
    let last = lineage.checkpoints.last().unwrap();
    let module = last
        .modules
        .iter()
        .find(|module| module.module.node == NodeId::new(2))
        .unwrap();
    assert_eq!(module.module.deleted_depth, Some(51));
    assert!(!module.module.is_present());
    assert!(last
        .modules
        .iter()
        .any(|module| module.module.created_depth == 0 && module.module.is_present()));
}

/// S0 arm 25 (`vm_unprepared` / CostSelection) batch 3 lineage 8 leaves its
/// only cohort module unreachable after each of its first three generations,
/// so eligibility rests on the generation-0 reading alone and the lineage
/// classifies `NoEdit`, never `NoEligibility`.
#[test]
fn recruitment_paths_known_lineage_keeps_generation_zero_eligibility_once_its_cohort_goes_unreachable(
) {
    let sizes = Sizes {
        batches: 4,
        lineages: 9,
        discovery: 2,
        followup: 1,
    };
    let assay = Assay::new(Panel::s0(sizes));
    let (start, policy) = assay.arm(25);
    assert_eq!(
        (start.name.as_str(), policy),
        ("vm_unprepared", Policy::CostSelection)
    );

    let record = assay.compact_lineage(25, 3, 8);

    let mut genome = start.genome.clone();
    let chosen: Vec<_> = record.proposals.iter().filter(|p| p.chosen).collect();
    assert_eq!(chosen.len(), 3);
    for proposal in chosen {
        genome = proposal.delta.as_ref().unwrap().apply(&genome).unwrap();
        let reachable: Vec<NodeId> = mesh_reachable_nodes(&genome)
            .iter()
            .map(|&index| genome.nodes[index].node_id)
            .collect();
        assert!(
            reachable.iter().all(|node| node.0 < 2),
            "generation {}: {reachable:?}",
            proposal.generation
        );
        assert!(genome.nodes.iter().any(|node| node.node_id.0 >= 2));
    }
    assert!(record.ladder.eligibility);
    assert_eq!(record.classification, LineageClass::NoEdit);
}

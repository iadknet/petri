use super::*;
use crate::config::MutationConfig;
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
    vm.program = vec![
        VmInstruction::PushAction { action_type: 1 },
        VmInstruction::ExecuteActionQueue,
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
                let summary = MutationEngine::apply_mutations_with_food_type_count(
                    &mut child,
                    &MutationConfig::default(),
                    &reachable,
                    ParentExecuted::Indices(&executed),
                    &mut SmallRng::seed_from_u64(record.seed),
                    1,
                );
                assert_eq!(
                    summary.events.iter().map(Event::from).collect::<Vec<_>>(),
                    record.events
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
        ("graph_blank", [1, 2, 1, 0, 2, 3, 8], 14),
        ("graph_copy", [2, 2, 1, 0, 4, 6, 8], 22),
        ("graph_split", [2, 2, 1, 0, 5, 7, 8], 24),
        ("graph_unprepared", [2, 2, 1, 1, 4, 8, 8], 25),
        ("graph_prepared", [2, 2, 1, 1, 4, 8, 8], 25),
        ("vm_blank", [1, 2, 13, 2, 0, 0, 0], 21),
        ("vm_copy", [2, 2, 23, 4, 0, 0, 0], 34),
        ("vm_unprepared", [2, 2, 21, 5, 0, 0, 0], 33),
        ("vm_prepared", [2, 2, 21, 5, 0, 0, 0], 33),
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
                start.mutable_sites.graph_action_slots,
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
/// seed per step in `0..SEARCH_RANGE`. It walks about 110,000 applied
/// events (the largest pinned seed is 41,854) in under two seconds, and it
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

/// The VM insert seeds are the ones the one-off search found after the
/// `JumpToHalt` acceptance fix and the `PushAction` draw repair (readings,
/// "Seed search"); only the two `write_direction` seeds exceed 10,000.
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
                vec![
                    (InputRefAdd, 1),
                    (GraphMutateActionSlotBehavior, 25),
                    (GraphAddInternalGraphNode, 1020),
                    (GraphAddGraphEdge, 1650),
                    (GraphAddGraphEdge, 3612),
                    (GraphAddGraphEdge, 102),
                    swap,
                ],
                Some(7),
            ),
            ("graph_copy", vec![(GraphAddGraphEdge, 102), swap], None),
            ("graph_split", vec![(GraphAddGraphEdge, 1762), swap], None),
            (
                "vm_blank",
                vec![
                    (InputRefAdd, 1),
                    (VmInstructionMutation, 238),
                    (VmInstructionMutation, 9940),
                    (VmInstructionMutation, 800),
                    (VmInstructionMutation, 41_854),
                    (VmInstructionMutation, 4126),
                    swap,
                ],
                Some(7),
            ),
            ("vm_copy", vec![(VmDeleteInstruction, 1), swap], None),
            (
                "graph_unprepared",
                vec![
                    (InputRefSwap, 25),
                    (GraphRetargetGraphEdge, 32),
                    (GraphAddInternalGraphNode, 64),
                    (GraphAddGraphEdge, 6718),
                    (GraphRetargetGraphEdge, 4),
                    swap,
                ],
                None,
            ),
            (
                "vm_unprepared",
                vec![
                    (InputRefSwap, 25),
                    (VmInstructionRawFieldMutation, 72),
                    (VmInstructionRawFieldMutation, 223),
                    (VmConstantMutation, 13),
                    (VmConstantMutation, 13),
                    swap,
                ],
                None,
            ),
            (
                "graph_detour",
                vec![
                    (InputRefAdd, 1),
                    (GraphMutateActionSlotBehavior, 25),
                    (GraphAddInternalGraphNode, 1020),
                    (GraphAddGraphEdge, 1650),
                    (GraphAddGraphEdge, 3612),
                    (GraphAddGraphEdge, 102),
                ],
                None,
            ),
            (
                "vm_detour",
                vec![
                    (InputRefAdd, 1),
                    (VmInstructionMutation, 238),
                    (VmInstructionMutation, 800),
                    (VmInstructionMutation, 21_017),
                    (VmInstructionMutation, 3709),
                    (VmInstructionMutation, 4126),
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
            "graph_copy",
            "graph_split",
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
                    a.action_bank[0].gate_inputs.len(),
                    b.action_bank[0].gate_inputs.len() + 1
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
                assert_eq!(a.program.len(), b.program.len() + 1);
                assert_eq!(a.program[4], VmInstruction::PushAction { action_type: 2 });
                assert_eq!(a.program[0], b.program[0]);
                assert_eq!(a.program[2..4], b.program[2..4]);
                assert_eq!(a.program[5..], b.program[4..]);
                // The insert's reference repair keeps the jump on the Halt.
                let jump = |offset| VmInstruction::JumpIfZero { cond: 0, offset };
                assert_eq!((&b.program[1], &a.program[1]), (&jump(2), &jump(3)));
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

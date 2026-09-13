use super::*;
use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use crate::neighborhood::mesh_execution::indices_for_node_ids;
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

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
        parent in 0u8..=8, a in 0u8..=8, b in 0u8..=8, alive_a in any::<bool>(), alive_b in any::<bool>()
    ) {
        let children = [(alive_a, a), (alive_b, b)];
        let selected = choose(Policy::Selection, (true, parent), children);
        if let Some(index) = selected {
            prop_assert!(children[index].0);
            prop_assert!(children[index].1 >= parent);
            for (other, &(live, score)) in children.iter().enumerate() {
                if live && score >= parent {
                    prop_assert!(children[index].1 >= score);
                    if score == children[index].1 { prop_assert!(index <= other); }
                }
            }
        } else { prop_assert!(children.iter().all(|&(live, score)| !live || score < parent)); }
        prop_assert_eq!(choose(Policy::Drift, (true, parent), children), Some(0));
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

#[test]
fn recruitment_paths_reduced_run_has_complete_supply_and_replay() {
    let report = observe(Sizes::TEST);
    assert_eq!(report.total_proposals, Sizes::TEST.proposals());
    assert_eq!(report.arms.len(), 18);
    assert_eq!(
        report.opportunities.attempted,
        report.opportunities.applied + report.opportunities.skipped
    );
    for arm in &report.arms {
        assert_eq!(arm.summary.retained_discovery.denominator, 1);
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
    assert_eq!(Sizes::PRODUCTION.proposals(), 55_296);
    let zero = estimate(0, 32);
    assert!((zero.wilson_95.unwrap()[1] - 0.107_179_198_255_070_6).abs() < 1e-12);
}

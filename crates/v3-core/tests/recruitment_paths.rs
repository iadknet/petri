use v3_core::neighborhood::recruitment_paths::{constructed_paths, evaluate, Task};

#[test]
fn recruitment_paths_constructed_stages_are_live_and_reach_useful_control() {
    let paths = constructed_paths();
    assert_eq!(paths.len(), 2);
    for path in paths {
        let mut previous = 4;
        for stage in &path.stages {
            let reading = evaluate(&stage.genome);
            assert!(reading.live());
            assert!(reading.correct(Task::A) + 1 >= previous);
            assert_eq!(reading, stage.task);
            if stage.name != "activated" {
                assert!(stage.incumbent_actions_unchanged);
                assert!(!reading
                    .dispatched()
                    .contains(&v3_core::contracts::NodeId::new(2)));
            }
            previous = reading.correct(Task::A);
        }
        assert_eq!(previous, 8);
        assert!(path.stages.last().unwrap().useful);
    }
}

#[test]
fn recruitment_paths_apply_both_cues_with_irrelevant_north_food() {
    for path in constructed_paths() {
        let stage = path.stages.last().unwrap();
        let reading = evaluate(&stage.genome);
        assert_eq!(reading.correct(Task::A), 8);
        assert_eq!(reading.correct(Task::B), 4);
        assert!(reading.scenes.iter().all(|scene| scene.maintenance > 0.0));
        for pair in reading.scenes.chunks_exact(2) {
            assert_eq!(pair[0].actions, pair[1].actions);
            assert_eq!(pair[0].position, pair[1].position);
        }
    }
}

#[test]
fn recruitment_paths_qualified_paths_replay_through_the_production_operators() {
    use v3_core::neighborhood::recruitment_paths::{
        qualified_paths, MAX_PATH_EVENTS, SEARCH_RANGE,
    };

    let paths = qualified_paths();
    assert_eq!(paths.len(), 9);
    let mut qualified = 0;
    for path in &paths {
        let mut genome = path.start.genome.clone();
        assert_eq!(evaluate(&genome), path.start.task);
        for step in &path.steps {
            assert!(step.seed < SEARCH_RANGE, "{}", path.form);
            step.event
                .apply(&mut genome, step.seed)
                .unwrap_or_else(|reason| panic!("{} {}: {reason:?}", path.form, step.stage.name));
            assert_eq!(
                genome, step.stage.genome,
                "{} {}",
                path.form, step.stage.name
            );
            assert_eq!(
                evaluate(&genome),
                step.stage.task,
                "{} {}",
                path.form,
                step.stage.name
            );
        }
        if path.qualified() {
            qualified += 1;
            assert!(path.steps.len() <= MAX_PATH_EVENTS);
            let last = path.steps.last().unwrap();
            assert!(last.stage.useful, "{}", path.form);
            assert_eq!(last.stage.task.correct(path.task), 8, "{}", path.form);
        } else {
            assert!(path.gap.is_some(), "{}", path.form);
            assert_eq!(path.steps.last().unwrap().stage.task.correct(path.task), 8);
        }
    }
    // T19.F04: a blank module reaches a `Move(E)` vote in one edge or one
    // instruction, so the two blank forms qualify beside the other seven.
    assert_eq!(qualified, 9);
}

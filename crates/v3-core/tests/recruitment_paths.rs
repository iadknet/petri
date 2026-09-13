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

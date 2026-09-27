use super::{Family, Scene, SceneReading};
use crate::contracts::{Direction, NodeId, WorldAction};
use crate::creature::action_log::{ActionResult, ActionType};
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::neighborhood::input_use::{
    ablated,
    catalog::{addressed, Addressed},
    connected_channels,
};
use crate::neighborhood::opportunity::fixtures::Local;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize)]
pub struct Panel {
    pub family: String,
    pub held_out: bool,
    pub scenes: Vec<Scene>,
    pub focal: Vec<bool>,
    pub baseline: Reading,
    #[serde(skip)]
    family_kind: Family,
}

impl Panel {
    pub fn new(family: Family, held_out: bool, founder: &CreatureGenome) -> Self {
        let ring_food: &[Option<u8>] = if family == Family::Ring {
            &[None, Some(0), Some(2), Some(4), Some(6)]
        } else if held_out {
            &[None, Some(0), Some(4), Some(6)]
        } else {
            &[None, Some(2)]
        };
        let variants: Vec<(Option<u8>, Option<u8>)> = match family {
            Family::Scalar => vec![(None, None)],
            Family::Vector => [0, 2, 4, 6]
                .map(|direction| (None, Some(direction)))
                .to_vec(),
            Family::Ring => [0, 2, 4, 6]
                .into_iter()
                .flat_map(|direction| [(Some(direction), Some(direction)), (None, Some(direction))])
                .collect(),
        };
        let mut scenes = Vec::new();
        for local in Local::ALL {
            for &ring in ring_food {
                for eligible in [false, true] {
                    for &(barrier, far) in &variants {
                        let distances: &[i32] = if held_out && family == Family::Vector {
                            &[3, 4]
                        } else {
                            &[2]
                        };
                        let diagonals: &[bool] = if held_out && family == Family::Ring {
                            &[false, true]
                        } else {
                            &[false]
                        };
                        for &distance in distances {
                            for &diagonal in diagonals {
                                let mut scene = Scene::new(
                                    local,
                                    ring,
                                    eligible,
                                    barrier,
                                    if family == Family::Ring { None } else { far },
                                    held_out,
                                );
                                if family == Family::Ring {
                                    let offsets = if held_out {
                                        [(3, -4), (4, 3), (-3, 4), (-4, -3)]
                                    } else {
                                        [(2, -3), (3, 2), (-2, 3), (-3, -2)]
                                    };
                                    scene.far_offset = Some(
                                        offsets[usize::from(far.expect("ring cue direction") / 2)],
                                    );
                                }
                                scene.distance = distance;
                                scene.diagonal_barrier = diagonal;
                                scenes.push(scene);
                            }
                        }
                    }
                }
            }
        }
        let baseline_scenes: Vec<_> = scenes.iter().map(|scene| scene.run(founder)).collect();
        let focal = scenes
            .iter()
            .zip(&baseline_scenes)
            .map(|(scene, baseline)| match family {
                Family::Scalar => scene.local_fruit && !scene.local_grass && !scene.eligible,
                Family::Vector => {
                    !scene.local_grass && scene.ring_food.is_none() && !scene.eligible
                }
                Family::Ring => scene.barrier.is_some_and(|barrier| {
                    baseline
                        .actions
                        .contains(&WorldAction::Move(Direction::ALL[usize::from(barrier)]))
                }),
            })
            .collect();
        let mut panel = Self {
            family: family.as_key().into(),
            held_out,
            scenes,
            focal,
            baseline: Reading::default(),
            family_kind: family,
        };
        panel.baseline = panel.read(baseline_scenes, None);
        panel
    }

    pub fn evaluate(&self, genome: &CreatureGenome) -> Reading {
        self.read(
            self.scenes.iter().map(|scene| scene.run(genome)).collect(),
            Some(&self.baseline),
        )
    }

    fn read(&self, scenes: Vec<SceneReading>, baseline: Option<&Reading>) -> Reading {
        let mut result = Reading::default();
        for (index, ((scene, reading), focal)) in
            self.scenes.iter().zip(&scenes).zip(&self.focal).enumerate()
        {
            result.alive &= reading.alive;
            result.dispatched.extend(&reading.dispatched);
            if *focal {
                result.opportunities += 1;
                let correct = match self.family_kind {
                    Family::Scalar => reading.food_intake[1] > 0.0,
                    Family::Vector => {
                        directed_progress(scene, reading)
                            && baseline.is_none_or(|base| {
                                nonmoves(&reading.actions) == nonmoves(&base.scenes[index].actions)
                            })
                    }
                    Family::Ring => {
                        let incumbent = baseline.map_or(reading, |base| &base.scenes[index]);
                        nonmoves(&reading.actions) == nonmoves(&incumbent.actions)
                            && !reading.applied.iter().any(|action| action.action_type == ActionType::Move && action.result != ActionResult::Success)
                            && !reading.actions.iter().any(|action| matches!(action, WorldAction::Move(direction) if Some(direction.to_index() as u8) == scene.barrier))
                    }
                } && reading.alive;
                result.correct += u32::from(correct);
            } else {
                result.incumbent_scenes += 1;
                let preserved = baseline.is_none_or(|base| {
                    let before = &base.scenes[index];
                    reading.actions == before.actions
                        && reading.position == before.position
                        && reading
                            .applied
                            .iter()
                            .map(|a| (a.action_type, a.result, a.direction, a.food_type, a.amount))
                            .eq(before.applied.iter().map(|a| {
                                (a.action_type, a.result, a.direction, a.food_type, a.amount)
                            }))
                });
                result.incumbent_preserved += u32::from(preserved);
            }
        }
        result.scenes = scenes;
        result
    }
}

fn nonmoves(actions: &[WorldAction]) -> Vec<WorldAction> {
    actions
        .iter()
        .filter(|action| !matches!(action, WorldAction::Move(_) | WorldAction::NoOp))
        .cloned()
        .collect()
}

fn directed_progress(scene: &Scene, reading: &SceneReading) -> bool {
    let Some(direction) = scene.far_food else {
        return false;
    };
    let moves: Vec<_> = reading
        .applied
        .iter()
        .filter(|action| action.action_type == ActionType::Move)
        .collect();
    let Some(position) = reading.position else {
        return false;
    };
    let (dx, dy) = Direction::ALL[usize::from(direction)].delta();
    let x = i32::from(position.x) - i32::from(scene.position.x);
    let y = i32::from(position.y) - i32::from(scene.position.y);
    let distance = x * dx + y * dy;
    !moves.is_empty()
        && moves
            .iter()
            .all(|action| action.direction == direction && action.result == ActionResult::Success)
        && x * dy == y * dx
        && distance > 0
        && distance <= scene.distance
}

#[derive(Debug, Clone, Serialize)]
pub struct Reading {
    pub correct: u32,
    pub opportunities: u32,
    pub incumbent_scenes: u32,
    pub incumbent_preserved: u32,
    pub alive: bool,
    pub dispatched: BTreeSet<NodeId>,
    pub scenes: Vec<SceneReading>,
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            correct: 0,
            opportunities: 0,
            incumbent_scenes: 0,
            incumbent_preserved: 0,
            alive: true,
            dispatched: BTreeSet::new(),
            scenes: Vec::new(),
        }
    }
}

impl Reading {
    pub fn fraction(&self) -> f64 {
        if self.opportunities == 0 {
            0.0
        } else {
            f64::from(self.correct) / f64::from(self.opportunities)
        }
    }
    pub fn acceptable(&self) -> bool {
        self.alive && self.incumbent_scenes == self.incumbent_preserved
    }
}

pub fn qualifies_score(reading: &Reading, start: &Reading) -> bool {
    reading.opportunities > 0
        && reading.acceptable()
        && reading.fraction() >= 0.75
        && reading.fraction() - start.fraction() >= 0.125
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelReading {
    pub channel: u16,
    pub declared: u32,
    pub connected: bool,
    pub executed_scenes: u32,
    pub focal_scenes: u32,
    pub causal_focal_scenes: u32,
    pub score_loss: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Qualification {
    pub learning_mask_removed: usize,
    pub inherited_size: u32,
    pub expressed_size: u32,
    pub reading: Reading,
    pub all_backend_ablated: Reading,
    pub graph_ablated: Reading,
    pub vm_ablated: Reading,
    pub channels: Vec<ChannelReading>,
    pub coverage: bool,
    pub graph_discovery: bool,
    pub vm_discovery: bool,
    pub joint_effect: bool,
}

pub fn checkpoint(genome: &CreatureGenome, panel: &Panel, reading: Reading) -> Qualification {
    let family = panel.family_kind;
    let reference = family.reference();
    let channels: Vec<_> = (0..match &reference {
        crate::contracts::InputReference::World(key) => key.compound_width(),
        _ => unreachable!(),
    })
        .map(|sub| match addressed(&reference, sub) {
            Addressed::Channel(channel) => channel,
            _ => unreachable!(),
        })
        .collect();
    let all = ablated(genome, |channel| channels.contains(&channel));
    let backend_mask = |graph: bool| {
        let mut mask = all.clone();
        for (node, original) in mask.nodes.iter_mut().zip(&genome.nodes) {
            if matches!(original.backend_def, BackendDef::Graph(_)) != graph {
                node.backend_def = original.backend_def.clone();
            }
        }
        mask
    };
    let all_backend_ablated = panel.evaluate(&all);
    let graph_ablated = panel.evaluate(&backend_mask(true));
    let vm_ablated = panel.evaluate(&backend_mask(false));
    let connected = connected_channels(genome);
    let channel_readings: Vec<_> = channels
        .iter()
        .enumerate()
        .map(|(sub, channel)| {
            let ablated_reading =
                panel.evaluate(&ablated(genome, |candidate| candidate == *channel));
            ChannelReading {
                channel: sub as u16,
                declared: genome
                    .nodes
                    .iter()
                    .filter(|node| node.input_refs.contains(&reference))
                    .count() as u32,
                connected: connected.contains(channel),
                executed_scenes: reading
                    .scenes
                    .iter()
                    .filter(|scene| {
                        scene.reads.iter().any(|(id, index, read_sub)| {
                            *read_sub == sub as u16
                                && genome
                                    .nodes
                                    .iter()
                                    .find(|node| node.node_id == *id)
                                    .is_some_and(|node| {
                                        node.input_refs.get(usize::from(*index)) == Some(&reference)
                                    })
                        })
                    })
                    .count() as u32,
                focal_scenes: panel.focal.iter().filter(|focal| **focal).count() as u32,
                causal_focal_scenes: reading
                    .scenes
                    .iter()
                    .zip(&ablated_reading.scenes)
                    .zip(&panel.focal)
                    .filter(|((before, after), focal)| {
                        **focal
                            && (before.actions != after.actions
                                || before.position != after.position
                                || before.food_intake != after.food_intake)
                    })
                    .count() as u32,
                score_loss: reading.fraction() - ablated_reading.fraction(),
            }
        })
        .collect();
    let required: &[u16] = match family {
        Family::Scalar => &[0],
        Family::Vector => &[
            crate::sensors::perception::food_idx::NEAREST_DX as u16,
            crate::sensors::perception::food_idx::NEAREST_DY as u16,
        ],
        Family::Ring => &[0, 2, 4, 6],
    };
    let coverage = required
        .iter()
        .all(|index| channel_readings[usize::from(*index)].score_loss > 0.0);
    let common = qualifies_score(&reading, &panel.baseline)
        && coverage
        && reading.fraction() - all_backend_ablated.fraction() >= 0.125;
    let graph_loss = reading.fraction() - graph_ablated.fraction();
    let vm_loss = reading.fraction() - vm_ablated.fraction();
    let (masked, learning_mask_removed) = super::learning_off(genome);
    Qualification {
        learning_mask_removed,
        inherited_size: genome.genome_size(),
        expressed_size: masked.genome_size(),
        graph_discovery: common && graph_loss >= 0.125,
        vm_discovery: common && vm_loss >= 0.125,
        joint_effect: graph_loss > 0.0 && vm_loss > 0.0,
        reading,
        all_backend_ablated,
        graph_ablated,
        vm_ablated,
        channels: channel_readings,
        coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::creature::action_log::ActionLogEntry;
    use crate::creature::founder::founder_genome_with_age_gate;

    fn move_entry(direction: u8, result: ActionResult) -> ActionLogEntry {
        ActionLogEntry {
            tick: 0,
            action_type: ActionType::Move,
            result,
            direction,
            energy_before: 25.0,
            energy_after: 24.0,
            amount: 0.0,
            food_type: None,
            priority_bid: 0.0,
        }
    }

    #[test]
    fn directed_progress_accepts_legal_one_and_two_step_paths_only() {
        let scene = Scene::new(Local::None, None, false, None, Some(2), true);
        for distance in [1, 2, 3] {
            let reading = SceneReading {
                applied: vec![move_entry(2, ActionResult::Success); distance as usize],
                position: Some(scene.offset(2, distance)),
                ..SceneReading::default()
            };
            assert!(directed_progress(&scene, &reading));
        }
        for (applied, position) in [
            (vec![], scene.position),
            (
                vec![move_entry(2, ActionResult::Success)],
                scene.offset(2, 4),
            ),
            (vec![move_entry(2, ActionResult::Blocked)], scene.position),
            (
                vec![
                    move_entry(6, ActionResult::Success),
                    move_entry(2, ActionResult::Success),
                ],
                scene.offset(2, 1),
            ),
            (
                vec![move_entry(2, ActionResult::Success)],
                scene.offset(4, 1),
            ),
        ] {
            assert!(!directed_progress(
                &scene,
                &SceneReading {
                    applied,
                    position: Some(position),
                    ..SceneReading::default()
                }
            ));
        }
    }

    #[test]
    fn nonmoves_keeps_every_nonmovement_action_in_order() {
        let eat = WorldAction::eat(OrdinaryFoodTypeId::new(1));
        let reproduce = WorldAction::Reproduce {
            direction: Direction::S,
            energy_transfer_fraction: 0.25,
        };
        assert_eq!(
            nonmoves(&[
                WorldAction::NoOp,
                WorldAction::Move(Direction::N),
                eat,
                reproduce,
            ]),
            vec![eat, reproduce]
        );
    }

    #[test]
    fn directed_progress_covers_every_direction_and_rejects_zero_distance() {
        for (direction, _) in Direction::ALL.iter().enumerate() {
            let scene = Scene::new(Local::None, None, false, None, Some(direction as u8), true);
            assert!(directed_progress(
                &scene,
                &SceneReading {
                    applied: vec![move_entry(direction as u8, ActionResult::Success)],
                    position: Some(scene.offset(direction as u8, 1)),
                    ..SceneReading::default()
                }
            ));
            assert!(!directed_progress(
                &scene,
                &SceneReading {
                    applied: vec![move_entry(direction as u8, ActionResult::Success)],
                    position: Some(scene.position),
                    ..SceneReading::default()
                }
            ));
        }
    }

    fn panel(family_kind: Family, scene: Scene, focal: bool, baseline: Reading) -> Panel {
        Panel {
            family: family_kind.as_key().into(),
            held_out: false,
            scenes: vec![scene],
            focal: vec![focal],
            baseline,
            family_kind,
        }
    }

    #[test]
    fn panel_read_requires_liveness_and_preserves_the_alive_fold() {
        let scene = Scene::new(Local::Fruit, None, false, None, None, false);
        let panel = panel(Family::Scalar, scene, true, Reading::default());
        let result = panel.read(
            vec![SceneReading {
                food_intake: vec![0.0, 1.0],
                alive: false,
                ..SceneReading::default()
            }],
            None,
        );
        assert!(!result.alive);
        assert_eq!(result.correct, 0);
        assert_eq!(result.opportunities, 1);
    }

    #[test]
    fn vector_panel_requires_progress_and_incumbent_nonmoves() {
        let scene = Scene::new(Local::None, None, false, None, Some(2), false);
        let eat = WorldAction::eat(OrdinaryFoodTypeId::new(0));
        let baseline = Reading {
            scenes: vec![SceneReading {
                actions: vec![eat],
                ..SceneReading::default()
            }],
            ..Reading::default()
        };
        let panel = panel(Family::Vector, scene.clone(), true, baseline.clone());
        let reading = |actions| SceneReading {
            actions,
            applied: vec![move_entry(2, ActionResult::Success)],
            position: Some(scene.offset(2, 1)),
            alive: true,
            ..SceneReading::default()
        };
        assert_eq!(
            panel
                .read(vec![reading(vec![eat])], Some(&baseline))
                .correct,
            1
        );
        assert_eq!(
            panel
                .read(vec![reading(vec![WorldAction::NoOp])], Some(&baseline))
                .correct,
            0
        );
    }

    #[test]
    fn ring_panel_rejects_failed_moves_and_moves_into_the_barrier() {
        let scene = Scene::new(Local::None, None, false, Some(2), None, false);
        let eat = WorldAction::eat(OrdinaryFoodTypeId::new(0));
        let baseline = Reading {
            scenes: vec![SceneReading {
                actions: vec![eat],
                ..SceneReading::default()
            }],
            ..Reading::default()
        };
        let panel = panel(Family::Ring, scene, true, baseline.clone());
        let reading = |actions, applied| SceneReading {
            actions,
            applied,
            alive: true,
            ..SceneReading::default()
        };
        assert_eq!(
            panel
                .read(
                    vec![reading(
                        vec![eat],
                        vec![move_entry(0, ActionResult::Success)]
                    )],
                    Some(&baseline)
                )
                .correct,
            1
        );
        assert_eq!(
            panel
                .read(
                    vec![reading(
                        vec![eat],
                        vec![move_entry(0, ActionResult::Blocked)]
                    )],
                    Some(&baseline)
                )
                .correct,
            0
        );
        assert_eq!(
            panel
                .read(
                    vec![reading(
                        vec![eat, WorldAction::Move(Direction::E)],
                        vec![move_entry(0, ActionResult::Success)]
                    )],
                    Some(&baseline)
                )
                .correct,
            0
        );
    }

    #[test]
    fn incumbent_preservation_requires_actions_position_and_applied_identity() {
        let scene = Scene::new(Local::None, None, false, None, None, false);
        let expected = SceneReading {
            actions: vec![WorldAction::NoOp],
            applied: vec![move_entry(0, ActionResult::Success)],
            position: Some(scene.position),
            alive: true,
            ..SceneReading::default()
        };
        let baseline = Reading {
            scenes: vec![expected.clone()],
            ..Reading::default()
        };
        let panel = panel(Family::Scalar, scene.clone(), false, baseline.clone());
        assert_eq!(
            panel
                .read(vec![expected.clone()], Some(&baseline))
                .incumbent_preserved,
            1
        );
        let mut changed_actions = expected.clone();
        changed_actions.actions.clear();
        let mut changed_position = expected.clone();
        changed_position.position = Some(scene.offset(2, 1));
        let mut changed_applied = expected;
        changed_applied.applied[0].result = ActionResult::Blocked;
        for changed in [changed_actions, changed_position, changed_applied] {
            assert_eq!(
                panel
                    .read(vec![changed], Some(&baseline))
                    .incumbent_preserved,
                0
            );
        }
    }

    fn reading(correct: u32, opportunities: u32) -> Reading {
        Reading {
            correct,
            opportunities,
            incumbent_scenes: 2,
            incumbent_preserved: 2,
            alive: true,
            ..Reading::default()
        }
    }

    fn founder() -> CreatureGenome {
        let config = super::super::scene_config();
        founder_genome_with_age_gate(config.population.founder_profile, &config.energy.lifecycle)
    }

    fn single_scene_panel(family: Family, scene: Scene) -> Panel {
        Panel {
            family: family.as_key().into(),
            held_out: false,
            scenes: vec![scene],
            focal: vec![true],
            baseline: Reading {
                scenes: vec![SceneReading::default()],
                ..Reading::default()
            },
            family_kind: family,
        }
    }

    #[test]
    fn causal_channel_detection_accepts_each_independent_observable_difference() {
        let family = Family::Scalar;
        let start = founder();
        let control = super::super::instrument_control(&start, family, false);
        let panel = single_scene_panel(
            family,
            Scene::new(Local::Fruit, None, false, None, None, false),
        );
        let actual = checkpoint(&control, &panel, panel.evaluate(&control));
        let ablated = actual.all_backend_ablated.scenes[0].clone();

        let mut action_only = ablated.clone();
        action_only.actions = vec![WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 0.5,
        }];
        let action_only = checkpoint(
            &control,
            &panel,
            Reading {
                scenes: vec![action_only],
                alive: true,
                ..Reading::default()
            },
        );
        assert_eq!(action_only.channels[0].causal_focal_scenes, 1);

        let mut position_only = ablated;
        position_only.position = Some(panel.scenes[0].offset(2, 1));
        let position_only = checkpoint(
            &control,
            &panel,
            Reading {
                scenes: vec![position_only],
                alive: true,
                ..Reading::default()
            },
        );
        assert_eq!(position_only.channels[0].causal_focal_scenes, 1);
    }

    #[test]
    fn backend_loss_below_the_qualification_delta_does_not_discover() {
        use crate::contracts::RouteTarget;
        use crate::creature::genome::{vote::VoteSink, NodeGenome, VmBackendDef, VmInstruction};

        let family = Family::Vector;
        let start = super::super::family_start(&founder(), family);
        let control = super::super::instrument_control(&start, family, false);
        let full = Panel::new(family, false, &start);
        let reference = family.reference();
        let width = match &reference {
            crate::contracts::InputReference::World(key) => key.compound_width(),
            _ => unreachable!(),
        };
        let channels: Vec<_> = (0..width)
            .map(|sub| match addressed(&reference, sub) {
                Addressed::Channel(channel) => channel,
                _ => unreachable!(),
            })
            .collect();
        let all = ablated(&control, |candidate| channels.contains(&candidate));
        let required = [
            crate::sensors::perception::food_idx::NEAREST_DX as u16,
            crate::sensors::perception::food_idx::NEAREST_DY as u16,
        ]
        .map(|sub| match addressed(&reference, sub) {
            Addressed::Channel(channel) => channel,
            _ => unreachable!(),
        });
        let each: Vec<_> = required
            .iter()
            .map(|channel| ablated(&control, |candidate| candidate == *channel))
            .collect();

        let mut shared = Vec::new();
        let mut losses = vec![None; required.len()];
        for scene in &full.scenes {
            let panel = single_scene_panel(family, scene.clone());
            let control_correct = panel.evaluate(&control).correct == 1;
            let all_correct = panel.evaluate(&all).correct == 1;
            if control_correct && all_correct {
                shared.push(scene.clone());
            }
            for (index, one) in each.iter().enumerate() {
                if losses[index].is_none() && control_correct && panel.evaluate(one).correct == 0 {
                    losses[index] = Some(scene.clone());
                }
            }
        }
        assert!(!shared.is_empty());
        let mut scenes: Vec<_> = shared.iter().cycle().take(16).cloned().collect();
        scenes.extend(
            losses
                .into_iter()
                .map(|scene| scene.expect("each vector channel has a fixed causal scene")),
        );
        let scene_count = scenes.len();
        let panel = Panel {
            family: family.as_key().into(),
            held_out: false,
            focal: vec![true; scenes.len()],
            scenes,
            baseline: Reading {
                scenes: vec![SceneReading::default(); scene_count],
                ..Reading::default()
            },
            family_kind: family,
        };
        let reading = panel.evaluate(&control);
        let mut hybrid = control.clone();
        let vm_id = NodeId::new(2);
        let decision_id = hybrid.nodes[0].targets[0].target_id;
        hybrid.nodes[0].targets[0].target_id = vm_id;
        hybrid.nodes.push(NodeGenome {
            node_id: vm_id,
            input_refs: vec![reference],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![1.0],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: crate::sensors::perception::food_idx::NEAREST_DIST as u16,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Move(1).index() as u8,
                        src: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 0,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Decide.index() as u8,
                        src: 1,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![RouteTarget {
                target_id: decision_id,
                slot: 0,
                gate_bias: 0.0,
            }],
        });
        let result = checkpoint(&hybrid, &panel, reading);
        let loss = result.reading.fraction() - result.all_backend_ablated.fraction();
        assert!(qualifies_score(&result.reading, &panel.baseline));
        assert!(result.coverage);
        assert!(loss > 0.0 && loss < 0.125);
        assert!(result.reading.fraction() - result.graph_ablated.fraction() >= 0.125);
        assert!(!result.graph_discovery);
    }

    #[test]
    fn acceptable_requires_both_liveness_and_complete_incumbent_preservation() {
        assert!(reading(0, 0).acceptable());
        assert!(!Reading {
            alive: false,
            ..reading(0, 0)
        }
        .acceptable());
        assert!(!Reading {
            incumbent_preserved: 1,
            ..reading(0, 0)
        }
        .acceptable());
    }

    #[test]
    fn score_qualification_enforces_each_boundary_and_the_score_delta() {
        let start = reading(5, 8);
        assert!(qualifies_score(&reading(6, 8), &start));
        assert!(!qualifies_score(&reading(0, 0), &Reading::default()));
        assert!(!qualifies_score(&reading(5, 8), &Reading::default()));
        assert!(!qualifies_score(&reading(6, 8), &reading(6, 8)));
        assert!(!qualifies_score(
            &Reading {
                alive: false,
                ..reading(6, 8)
            },
            &start
        ));
        assert!(!qualifies_score(
            &Reading {
                incumbent_preserved: 1,
                ..reading(6, 8)
            },
            &start
        ));
    }
}

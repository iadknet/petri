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
    use crate::creature::action_log::ActionLogEntry;

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
}

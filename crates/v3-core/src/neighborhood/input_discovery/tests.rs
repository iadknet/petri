use super::*;
use crate::config::OrdinaryFoodTypeId;
use crate::creature::founder::founder_genome_with_age_gate;
use crate::neighborhood::opportunity::controllers::{controller, VOTE_NODE};
use crate::neighborhood::opportunity::fixtures::Local;

fn founder() -> crate::creature::genome::CreatureGenome {
    let config = scene_config();
    founder_genome_with_age_gate(config.population.founder_profile, &config.energy.lifecycle)
}

#[test]
fn scalar_scene_requires_applied_typed_consumption_and_keeps_native_costs() {
    let scene = Scene::new(Local::Fruit, Some(2), false, None, None, false);
    let start = founder();
    let baseline = scene.run(&start);
    let (control, _) = controller(&start, Family::Scalar, false);
    let result = scene.run(&control);
    assert_eq!(baseline.food_intake[1], 0.0);
    assert!(result.food_intake[1] > 0.0);
    assert!(result
        .applied
        .iter()
        .any(|action| action.food_type == Some(OrdinaryFoodTypeId::new(1)) && action.amount > 0.0));
    assert!(result.carrying > baseline.carrying);
    assert!(result.action_charge > 0.0);
    assert_eq!(result.plasticity_updates, 0);
    assert_eq!(result.learning_charge, 0.0);
}

#[test]
fn mixed_local_food_is_reconstructed_without_replacing_either_type() {
    let scene = Scene::new(Local::GrassAndFruit, None, false, None, None, false);
    let world = scene.world();
    assert_eq!(
        world.food_at_type(scene.position, OrdinaryFoodTypeId::new(0)),
        1.0
    );
    assert_eq!(
        world.food_at_type(scene.position, OrdinaryFoodTypeId::new(1)),
        1.0
    );
}

fn vm_scalar_control(
    start: &crate::creature::genome::CreatureGenome,
) -> crate::creature::genome::CreatureGenome {
    use crate::creature::genome::{
        vote::{ActionParamField, VoteSink},
        BackendDef, VmBackendDef, VmInstruction,
    };
    let mut vm = start.clone();
    let node = &mut vm.nodes[1];
    node.input_refs = vec![Family::Scalar.reference()];
    node.backend_def = BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::WriteActionParam {
                field_idx: ActionParamField::EatFoodType.index() as u8,
                src: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Eat.index() as u8,
                src: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Decide.index() as u8,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    });
    vm
}

#[test]
fn vm_only_focal_effect_is_not_misattributed_to_graph() {
    let start = founder();
    let panel = Panel::new(Family::Scalar, false, &start);
    let vm = vm_scalar_control(&start);
    let result = checkpoint(&vm, &panel, panel.evaluate(&vm));
    assert_eq!(result.reading.fraction(), 1.0);
    assert_eq!(result.graph_ablated.fraction(), 1.0);
    assert_eq!(result.vm_ablated.fraction(), 0.0);
    assert_eq!(result.all_backend_ablated.fraction(), 0.0);
    assert!(!qualifies_score(&result.reading, &panel.baseline));
    assert!(result.coverage);
    assert!(result.reading.fraction() - result.all_backend_ablated.fraction() >= 0.125);
    assert!(!result.graph_discovery);
    assert!(!result.vm_discovery);
    assert!(!result.joint_effect);
    assert!(result.channels[0].executed_scenes > 0);
}

#[test]
fn joint_effect_requires_each_backend_loss_to_be_positive() {
    let start = founder();
    let panel = Panel::new(Family::Scalar, false, &start);
    let reading = Reading {
        correct: 1,
        opportunities: 2,
        alive: true,
        ..Reading::default()
    };

    let vm = checkpoint(&vm_scalar_control(&start), &panel, reading.clone());
    assert!(vm.graph_ablated.fraction() > reading.fraction());
    assert!(vm.vm_ablated.fraction() < reading.fraction());
    assert!(!vm.joint_effect);

    let graph = instrument_control(&start, Family::Scalar, false);
    let graph = checkpoint(&graph, &panel, reading.clone());
    assert!(graph.graph_ablated.fraction() < reading.fraction());
    assert!(graph.vm_ablated.fraction() > reading.fraction());
    assert!(!graph.joint_effect);
}

#[test]
fn founder_starts_have_no_focal_reference_or_authored_payload() {
    let start = founder();
    for family in FAMILIES {
        assert!(start
            .nodes
            .iter()
            .all(|node| !node.input_refs.contains(&family.reference())));
    }
    assert_eq!(learning_off(&start).0, start);
}

proptest::proptest! {
    #[test]
    fn learning_mask_is_clone_only_and_idempotent(rate in 0.0f32..1.0, lamarckian in proptest::prelude::any::<bool>()) {
        use crate::creature::genome::{BackendDef, PlasticityConfig, HebbianRule, RewardModulationConfig, OutcomeChannel};
        let mut inherited = founder();
        let BackendDef::Graph(graph) = &mut inherited.nodes[0].backend_def else { unreachable!() };
        graph.compute_nodes[0].plasticity = Some(PlasticityConfig { rule:HebbianRule::Classic, learning_rate:rate, weight_clamp:2.0, lamarckian, modulation:Some(RewardModulationConfig { reward_source:OutcomeChannel::EnergyDelta, trace_decay:0.5 }) });
        let before = inherited.clone();
        let (masked, removed) = learning_off(&inherited);
        proptest::prop_assert_eq!(removed, 1);
        proptest::prop_assert_eq!(learning_off(&masked), (masked, 0));
        proptest::prop_assert_eq!(inherited, before);
    }
}

#[test]
fn authored_controls_qualify_each_native_panel_and_ablations_do_not() {
    let start = founder();
    for family in FAMILIES {
        let start = family_start(&start, family);
        for held_out in [false, true] {
            let panel = Panel::new(family, held_out, &start);
            let positive = instrument_control(&start, family, false);
            let zero = instrument_control(&start, family, true);
            let positive = checkpoint(&positive, &panel, panel.evaluate(&positive));
            let expected_causal: &[u32] = match (family, held_out) {
                (Family::Scalar, false) => &[2],
                (Family::Scalar, true) => &[4],
                (Family::Vector, false) => &[0, 0, 0, 4, 2, 6, 0],
                (Family::Vector, true) => &[0, 0, 0, 8, 8, 16, 0],
                (Family::Ring, false) => &[8, 0, 4, 0, 4, 0, 4, 0],
                (Family::Ring, true) => &[16, 0, 8, 0, 8, 0, 8, 0],
            };
            assert_eq!(
                positive
                    .channels
                    .iter()
                    .map(|row| row.causal_focal_scenes)
                    .collect::<Vec<_>>(),
                expected_causal
            );
            assert_eq!(positive.reading.fraction(), 1.0);
            assert!(positive.reading.acceptable());
            assert_eq!(
                positive.all_backend_ablated.fraction(),
                positive.graph_ablated.fraction()
            );
            assert!(positive.reading.fraction() - positive.graph_ablated.fraction() >= 0.125);
            assert_eq!(positive.vm_ablated.fraction(), 1.0);
            assert!(positive.coverage);
            assert!(!positive.vm_discovery);
            assert!(!positive.joint_effect);
            for (index, channel) in positive.channels.iter().enumerate() {
                assert_eq!(usize::from(channel.channel), index);
                assert_eq!(channel.declared, 1);
                assert_eq!(channel.focal_scenes, positive.reading.opportunities);
                if channel.connected {
                    assert_eq!(
                        channel.executed_scenes,
                        positive.reading.scenes.len() as u32
                    );
                    assert!(channel.causal_focal_scenes > 0);
                    assert!(channel.score_loss > 0.0);
                } else {
                    assert_eq!(channel.executed_scenes, 0);
                    assert_eq!(channel.causal_focal_scenes, 0);
                    assert_eq!(channel.score_loss, 0.0);
                }
            }
            assert!(positive.graph_discovery, "{family:?} held_out={held_out}: score={}/{}, baseline={}, preserve={}/{}, coverage={}, channels={:?}", positive.reading.correct, positive.reading.opportunities, panel.baseline.fraction(), positive.reading.incumbent_preserved, positive.reading.incumbent_scenes, positive.coverage, positive.channels);
            assert!(!qualifies_score(&panel.evaluate(&zero), &panel.baseline));
        }
    }
}

#[test]
fn ring_score_without_every_required_channel_fails_coverage() {
    use crate::creature::genome::{cgp::GraphSource, BackendDef};

    let start = family_start(&founder(), Family::Ring);
    let panel = Panel::new(Family::Ring, false, &start);
    let mut incomplete = instrument_control(&start, Family::Ring, false);
    let ring_ref = u16::try_from(incomplete.nodes[VOTE_NODE].input_refs.len() - 1).unwrap();
    let BackendDef::Graph(graph) = &mut incomplete.nodes[VOTE_NODE].backend_def else {
        unreachable!()
    };
    for sink in &mut graph.output_sinks {
        sink.inputs.retain(|edge| {
            !matches!(
                edge.source,
                GraphSource::InputLeaf {
                    ref_idx,
                    sub_idx: 6
                } if ref_idx == ring_ref
            )
        });
    }
    let result = checkpoint(&incomplete, &panel, panel.evaluate(&incomplete));
    assert!(qualifies_score(&result.reading, &panel.baseline));
    assert!(result.reading.fraction() - result.all_backend_ablated.fraction() >= 0.125);
    assert!(!result.coverage);
    assert!(!result.graph_discovery);
}

#[test]
fn predeclared_ring_incumbent_sees_off_axis_food_and_attempts_all_cardinal_barriers() {
    use crate::creature::action_log::{ActionResult, ActionType};
    let start = controller(&founder(), Family::Vector, false).0;
    assert!(start
        .nodes
        .iter()
        .all(|node| !node.input_refs.contains(&Family::Ring.reference())));
    for held_out in [false, true] {
        let offsets = if held_out {
            [(3, -4), (4, 3), (-3, 4), (-4, -3)]
        } else {
            [(2, -3), (3, 2), (-2, 3), (-3, -2)]
        };
        for (direction, offset) in [0, 2, 4, 6].into_iter().zip(offsets) {
            let mut scene = Scene::new(Local::None, None, false, Some(direction), None, held_out);
            scene.far_offset = Some(offset);
            let reading = scene.run(&start);
            let area = reading.area_food.expect("native perception snapshot");
            assert_eq!(
                area[crate::sensors::perception::food_idx::NEAREST_DX],
                offset.0 as f32 / 5.0
            );
            assert_eq!(
                area[crate::sensors::perception::food_idx::NEAREST_DY],
                offset.1 as f32 / 5.0
            );
            assert!(
                reading
                    .applied
                    .iter()
                    .any(|action| action.action_type == ActionType::Move
                        && action.direction == direction
                        && action.result == ActionResult::Blocked),
                "held_out={held_out}, direction={direction}, actions={:?}, applied={:?}",
                reading.actions,
                reading.applied
            );
        }
    }
}

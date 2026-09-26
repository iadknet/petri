use super::*;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::NodeGenome;
use rand::{rngs::SmallRng, SeedableRng};

fn fixture(weight: f32) -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for d in 0..2 {
        def.output_sinks
            .iter_mut()
            .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Move(d)))
            .unwrap()
            .inputs
            .push(GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: u16::from(d),
                },
                weight,
            });
    }
    CreatureGenome {
        entry_node_id: NodeId::new(7),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(7),
            input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

#[test]
fn refinement_configuration_is_opt_in_and_omission_is_false() {
    let mut value = serde_json::to_value(MutationConfig::default()).unwrap();
    assert_eq!(value["structured_heritable_refinement"], false);
    value
        .as_object_mut()
        .unwrap()
        .remove("structured_heritable_refinement");
    let omitted: MutationConfig = serde_json::from_value(value.clone()).unwrap();
    assert!(!omitted.structured_heritable_refinement);
    value["structured_heritable_refinement"] = serde_json::json!(true);
    let enabled: MutationConfig = serde_json::from_value(value).unwrap();
    assert!(GraphOperator::RefineHeritableStructure.enabled(&enabled));
    let original = fixture(0.0);
    let mut genome = original.clone();
    let mut rng = SmallRng::seed_from_u64(3);
    let mut replay = rng.clone();
    assert_eq!(
        GraphMutator::apply(
            &mut genome,
            GraphOperator::RefineHeritableStructure,
            &mut TargetSelector::reachable_only(&[0], 0.0),
            &mut rng,
            &omitted
        ),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(genome, original);
    assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
    assert_eq!(GraphOperator::RefineHeritableStructure.weight(), 4);
    assert_eq!(
        GraphOperator::RefineHeritableStructure.complexity_effect(),
        crate::mutation::types::ComplexityEffect::Neutral
    );
}

#[test]
fn refinement_engine_accounts_one_event_under_normal_and_restricted_pressure() {
    let key = MutationOperator::GraphRefineHeritableStructure;
    for restricted in [false, true] {
        let config = MutationConfig {
            structured_heritable_refinement: true,
            per_unit_rate: 1.0,
            mesh_layer_probability: 0.0,
            genome_size_pressure_enabled: restricted,
            genome_size_cap: 1,
            ..MutationConfig::default()
        };
        let mut counts = [0, 0];
        for seed in 0..512 {
            for weight in [0.0, 1_048_576.0] {
                let original = fixture(weight);
                let mut genome = original.clone();
                let summary = MutationEngine::apply_mutations_on_units(
                    &mut genome,
                    1,
                    &config,
                    &[0],
                    ParentExecuted::NONE,
                    &mut SmallRng::seed_from_u64(seed),
                    1,
                );
                assert_eq!(summary.attempted_events, 1);
                assert_eq!(summary.applied_events + summary.skipped_events, 1);
                let event = &summary.events[0];
                if event.operator != Some(key) {
                    continue;
                }
                assert_eq!(event.target, Some(NodeId::new(7)));
                let funnel = summary.operator_funnel_by_operator[&key];
                assert_eq!(funnel.attempted, 1);
                assert_eq!(funnel.applicable, 1);
                assert_eq!(funnel.applied + funnel.skipped, 1);
                assert_eq!(genome.genome_size(), original.genome_size());
                match event.outcome {
                    MutationEventOutcome::Applied(_) => {
                        counts[0] += 1;
                        assert_eq!(funnel.applied, 1);
                    }
                    MutationEventOutcome::Skipped(reason) => {
                        counts[1] += 1;
                        assert_eq!(reason, MutationSkipReason::NumericProposalRejected);
                        assert_eq!(genome, original);
                        assert_eq!(funnel.skipped, 1);
                        assert_eq!(summary.skip_reasons_by_operator[&key][&reason], 1);
                    }
                }
            }
        }
        assert!(
            counts.iter().all(|n| *n > 0),
            "both applied and atomic numeric skips under restricted={restricted}: {counts:?}"
        );
    }
}

proptest::proptest! {
    #[test]
    fn refinement_ignores_inapplicable_padding_and_uses_existing_target_selection(seed in proptest::prelude::any::<u64>(), padding in 0usize..30) {
        let mut genome = fixture(0.0);
        let empty = NodeGenome { node_id: NodeId::new(99), input_refs: vec![], backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()), targets: vec![] };
        genome.nodes.splice(0..0, vec![empty; padding]);
        let mut target = TargetSelector::reachable_only(&[], 0.0);
        let config = MutationConfig { structured_heritable_refinement: true, ..MutationConfig::default() };
        let result = GraphMutator::apply(&mut genome, GraphOperator::RefineHeritableStructure, &mut target, &mut SmallRng::seed_from_u64(seed), &config);
        proptest::prop_assert!(result.is_ok());
        proptest::prop_assert_eq!(target.first_pick(), Some(padding));
        proptest::prop_assert_eq!(GraphMutator::applicable_indices(&genome, GraphOperator::RefineHeritableStructure), vec![padding]);
    }
}

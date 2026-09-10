use super::*;
use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::cgp::{
    ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ExecuteGate, WorldActionKind,
};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::types::MutationSkipReason;
use rand::rngs::SmallRng;
use rand::SeedableRng;

mod copy;
mod extensions;
mod f08;
mod operators;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn single_graph_genome_with_action_bank(action_bank: Vec<ActionSlot>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: Vec::new(),
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: Vec::new(),
                output_sinks: Vec::new(),
                action_bank,
                execute_gate: ExecuteGate { inputs: Vec::new() },
            }),
            targets: Vec::new(),
        }],
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &GraphOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

#[test]
fn graph_operator_catalog_includes_mutate_action_slot_behavior() {
    assert!(
        GraphOperator::ALL.contains(&GraphOperator::MutateActionSlotBehavior),
        "MutateActionSlotBehavior must be part of the canonical GraphOperator set"
    );
}

#[test]
fn mutate_action_slot_behavior_operator_changes_slot_behavior() {
    let original = ActionSlotBehavior::Emit(WorldActionKind::Eat);
    let base = single_graph_genome_with_action_bank(vec![ActionSlot {
        behavior: original,
        gate_inputs: Vec::new(),
        param_inputs: Vec::new(),
    }]);

    let mut changed = false;
    for seed in 0u64..128 {
        let mut genome = base.clone();
        let mut r = rng(seed);
        GraphMutator::apply(
            &mut genome,
            GraphOperator::MutateActionSlotBehavior,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();

        let mutated = match &genome.nodes[0].backend_def {
            BackendDef::Graph(def) => def.action_bank[0].behavior,
            BackendDef::Vm(_) => panic!("expected graph backend"),
        };
        if mutated != original {
            changed = true;
            break;
        }
    }

    assert!(
        changed,
        "MutateActionSlotBehavior should be able to change slot behavior"
    );
}

#[test]
fn mutate_action_slot_behavior_operator_skips_when_action_bank_empty() {
    let mut genome = single_graph_genome_with_action_bank(Vec::new());
    let mut r = rng(0);

    let result = GraphMutator::apply(
        &mut genome,
        GraphOperator::MutateActionSlotBehavior,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    );

    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

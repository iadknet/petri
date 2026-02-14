use v2_core::backends::{
    BackendExecutionRequest, EnergyBudget, execute_graph_backend, execute_vm_backend,
};
use v2_core::mesh::{
    ActionMetadataField, EmittedOutput, OutputDefinition, WorldActionDef, WorldActionKind,
};

fn assert_world_action_move(output: &EmittedOutput) {
    match output {
        EmittedOutput::WorldAction(action) => assert_eq!(action.action_kind, WorldActionKind::Move),
        _ => panic!("expected world action output"),
    }
}

#[test]
fn graph_backend_applies_static_tariff_and_emits_node_outputs() {
    let request = BackendExecutionRequest {
        remaining_energy: 0.10,
        energy_budget: EnergyBudget {
            graph_static_tariff: 0.03,
            vm_per_op_cost: 0.02,
        },
        output_definitions: vec![OutputDefinition::WorldAction(WorldActionDef {
            action_kind: WorldActionKind::Move,
            action_metadata_fields: vec![ActionMetadataField::Direction(2)],
        })],
        vm_requested_ops: 0,
    };

    let execution = execute_graph_backend(&request);
    assert_eq!(execution.emitted.len(), 1);
    assert_world_action_move(&execution.emitted[0]);
    assert!((execution.compute_energy - 0.03).abs() < 1e-6);
    assert!(!execution.exhausted_energy);
}

#[test]
fn vm_backend_is_bounded_by_remaining_energy() {
    let request = BackendExecutionRequest {
        remaining_energy: 0.07,
        energy_budget: EnergyBudget {
            graph_static_tariff: 0.03,
            vm_per_op_cost: 0.03,
        },
        output_definitions: vec![],
        vm_requested_ops: usize::MAX,
    };

    let execution = execute_vm_backend(&request);
    assert_eq!(execution.emitted, Vec::<EmittedOutput>::new());
    assert!((execution.compute_energy - 0.07).abs() < 1e-6);
    assert!(execution.exhausted_energy);
}

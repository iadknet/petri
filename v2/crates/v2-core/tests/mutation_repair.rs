use v2_core::evolution::{
    MutationConfig, repair_genome, repair_or_discard, validate_mutation_invariants,
};
use v2_core::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    InternalTargetDef, NodeGenome, NodeType, OutputDefinition, PayloadField, VmBackendDef,
    VmInstruction, WorldActionDef, WorldActionKind,
};

fn invalid_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: 99,
        nodes: vec![
            NodeGenome {
                node_id: 1,
                node_type: NodeType::Graph,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    operator: GraphOperator::Passthrough,
                    inputs: Vec::new(),
                    coefficients: Vec::new(),
                    bias: 0.0,
                    state_slot_count: 0,
                }),
                output_definitions: vec![OutputDefinition::InternalTarget(InternalTargetDef {
                    target_node_id: 777,
                    input_refs: Vec::new(),
                    payload_fields: vec![
                        PayloadField::Scalar {
                            key: "dup".to_string(),
                            value: 1,
                        },
                        PayloadField::Scalar {
                            key: "dup".to_string(),
                            value: 2,
                        },
                    ],
                })],
                local_state_init: Vec::new(),
            },
            NodeGenome {
                node_id: 1,
                node_type: NodeType::Vm,
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 0,
                    program: vec![VmInstruction::Noop; 200],
                    constants: Vec::new(),
                    max_input_slots: 0,
                }),
                output_definitions: vec![OutputDefinition::WorldAction(WorldActionDef {
                    action_kind: WorldActionKind::Move,
                    action_metadata_fields: vec![
                        ActionMetadataField::Direction(9),
                        ActionMetadataField::Direction(1),
                    ],
                })],
                local_state_init: Vec::new(),
            },
        ],
        evolution_params: None,
    }
}

#[test]
fn repair_passes_fix_invalid_mutation_outputs() {
    let config = MutationConfig::default();
    let mut genome = invalid_genome();

    assert!(validate_mutation_invariants(&genome, &config).is_err());

    let repaired = repair_genome(&mut genome, &config, 3);
    assert!(repaired, "repair should succeed within bounded passes");
    assert!(validate_mutation_invariants(&genome, &config).is_ok());

    assert!(genome
        .nodes
        .iter()
        .any(|node| node.node_id == genome.entry_node_id));

    let internal = genome
        .nodes
        .iter()
        .flat_map(|node| node.output_definitions.iter())
        .find_map(|output| match output {
            OutputDefinition::InternalTarget(target) => Some(target),
            OutputDefinition::WorldAction(_) => None,
        })
        .expect("repaired internal target should exist");
    assert!(genome
        .nodes
        .iter()
        .any(|node| node.node_id == internal.target_node_id));
    assert_eq!(internal.payload_fields.len(), 1);

    let vm = genome
        .nodes
        .iter()
        .find_map(|node| match &node.backend_def {
            BackendDef::Vm(vm) => Some(vm),
            BackendDef::Graph(_) => None,
        })
        .expect("vm node should exist");
    assert!((1..=32).contains(&vm.register_count));
    assert!((1..=64).contains(&vm.max_input_slots));
    assert!(vm.program.len() <= config.max_vm_program_len);
}

#[test]
fn repair_or_discard_drops_candidate_when_config_is_unsatisfiable() {
    let impossible = MutationConfig {
        min_nodes: 5,
        max_nodes: 2,
        ..MutationConfig::default()
    };

    let candidate = invalid_genome();
    let repaired = repair_or_discard(candidate, &impossible, 3);

    assert!(repaired.is_none());
}

#[test]
fn repair_or_discard_drops_candidate_when_repair_budget_is_zero() {
    let config = MutationConfig::default();
    let candidate = invalid_genome();

    let repaired = repair_or_discard(candidate, &config, 0);

    assert!(repaired.is_none());
}

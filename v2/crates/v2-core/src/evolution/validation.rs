use std::collections::{HashMap, HashSet};

use super::config::{MutationConfig, MutationConfigError};
use crate::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    MeshSchemaError, NodeGenome, NodeType, OutputDefinition, VmBackendDef, VmInstruction,
    WorldActionDef, WorldActionKind, world_action_metadata_is_valid,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MutationInvariantError {
    InvalidConfig(MutationConfigError),
    MeshSchema(MeshSchemaError),
    NodeCountOutOfBounds {
        count: usize,
        min: usize,
        max: usize,
    },
    OutputCountExceeded {
        node_id: u32,
        count: usize,
        max: usize,
    },
    VmProgramTooLong {
        node_id: u32,
        len: usize,
        max: usize,
    },
}

pub fn validate_mutation_invariants(
    genome: &CreatureGenome,
    config: &MutationConfig,
) -> Result<(), MutationInvariantError> {
    config
        .validate()
        .map_err(MutationInvariantError::InvalidConfig)?;

    genome
        .validate()
        .map_err(MutationInvariantError::MeshSchema)?;

    let node_count = genome.nodes.len();
    if node_count < config.min_nodes || node_count > config.max_nodes {
        return Err(MutationInvariantError::NodeCountOutOfBounds {
            count: node_count,
            min: config.min_nodes,
            max: config.max_nodes,
        });
    }

    let max_program = config.max_vm_program_len.min(128);
    for node in &genome.nodes {
        let output_count = node.output_definitions.len();
        if output_count > config.max_outputs_per_node {
            return Err(MutationInvariantError::OutputCountExceeded {
                node_id: node.node_id,
                count: output_count,
                max: config.max_outputs_per_node,
            });
        }

        if let BackendDef::Vm(vm) = &node.backend_def {
            let length = vm.program.len();
            if length > max_program {
                return Err(MutationInvariantError::VmProgramTooLong {
                    node_id: node.node_id,
                    len: length,
                    max: max_program,
                });
            }
        }
    }

    Ok(())
}

#[must_use]
pub fn repair_genome(
    genome: &mut CreatureGenome,
    config: &MutationConfig,
    max_passes: usize,
) -> bool {
    if config.validate().is_err() {
        return false;
    }

    for _ in 0..max_passes {
        if validate_mutation_invariants(genome, config).is_ok() {
            return true;
        }

        repair_once(genome, config);
    }

    validate_mutation_invariants(genome, config).is_ok()
}

#[must_use]
pub fn repair_or_discard(
    mut candidate: CreatureGenome,
    config: &MutationConfig,
    max_passes: usize,
) -> Option<CreatureGenome> {
    if repair_genome(&mut candidate, config, max_passes) {
        Some(candidate)
    } else {
        None
    }
}

fn repair_once(genome: &mut CreatureGenome, config: &MutationConfig) {
    if genome.nodes.is_empty() {
        genome.nodes.push(default_graph_node(1));
        genome.entry_node_id = 1;
    }

    repair_node_ids(genome);
    enforce_node_bounds(genome, config);
    repair_entry(genome);
    repair_backends(genome, config);
    repair_outputs(genome, config);
}

fn repair_node_ids(genome: &mut CreatureGenome) {
    let mut seen = HashSet::new();
    let mut max_id = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .max()
        .unwrap_or(0);

    for node in &mut genome.nodes {
        if seen.insert(node.node_id) {
            continue;
        }
        max_id = max_id.saturating_add(1);
        node.node_id = max_id;
        seen.insert(node.node_id);
    }
}

fn enforce_node_bounds(genome: &mut CreatureGenome, config: &MutationConfig) {
    while genome.nodes.len() > config.max_nodes {
        let _ = genome.nodes.pop();
    }

    let mut next_id = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    while genome.nodes.len() < config.min_nodes {
        genome.nodes.push(default_graph_node(next_id));
        next_id = next_id.saturating_add(1);
    }
}

fn repair_entry(genome: &mut CreatureGenome) {
    if !genome
        .nodes
        .iter()
        .any(|node| node.node_id == genome.entry_node_id)
        && let Some(first) = genome.nodes.first()
    {
        genome.entry_node_id = first.node_id;
    }
}

fn repair_backends(genome: &mut CreatureGenome, config: &MutationConfig) {
    let max_program = config.max_vm_program_len.min(128);
    for node in &mut genome.nodes {
        match node.node_type {
            NodeType::Graph => {
                if !matches!(node.backend_def, BackendDef::Graph(_)) {
                    node.backend_def = default_graph_backend();
                }

                if let BackendDef::Graph(graph) = &mut node.backend_def {
                    graph.state_slot_count = graph.state_slot_count.min(8);
                }
            }
            NodeType::Vm => {
                if !matches!(node.backend_def, BackendDef::Vm(_)) {
                    node.backend_def = default_vm_backend();
                }

                if let BackendDef::Vm(vm) = &mut node.backend_def {
                    vm.register_count = vm.register_count.clamp(1, 32);
                    vm.max_input_slots = vm.max_input_slots.clamp(1, 64);
                    if vm.constants.len() > 64 {
                        vm.constants.truncate(64);
                    }

                    if vm.program.is_empty() {
                        vm.program.push(VmInstruction::Halt);
                    }
                    if vm.program.len() > max_program {
                        vm.program.truncate(max_program);
                    }
                }
            }
        }
    }
}

fn repair_outputs(genome: &mut CreatureGenome, config: &MutationConfig) {
    let node_vm_slots = genome
        .nodes
        .iter()
        .filter_map(|node| {
            if let BackendDef::Vm(vm) = &node.backend_def {
                Some((node.node_id, usize::from(vm.max_input_slots)))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    let existing_ids = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<HashSet<_>>();
    let fallback_target = genome
        .nodes
        .first()
        .map(|node| node.node_id)
        .unwrap_or(genome.entry_node_id);

    for node in &mut genome.nodes {
        if node.output_definitions.len() > config.max_outputs_per_node {
            node.output_definitions
                .truncate(config.max_outputs_per_node);
        }

        for output in &mut node.output_definitions {
            match output {
                OutputDefinition::InternalTarget(target) => {
                    if !existing_ids.contains(&target.target_node_id) {
                        target.target_node_id = fallback_target;
                    }

                    if let Some(max_slots) = node_vm_slots.get(&target.target_node_id)
                        && target.input_refs.len() > *max_slots
                    {
                        target.input_refs.truncate(*max_slots);
                    }

                    let mut seen_keys = HashSet::new();
                    target.payload_fields.retain(|field| {
                        let key = match field {
                            crate::mesh::PayloadField::Scalar { key, .. }
                            | crate::mesh::PayloadField::Flag { key, .. } => key,
                        };
                        seen_keys.insert(key.clone())
                    });
                }
                OutputDefinition::WorldAction(action) => {
                    normalize_world_action_metadata(action);
                }
            }
        }
    }
}

fn normalize_world_action_metadata(action: &mut WorldActionDef) {
    if world_action_metadata_is_valid(action) {
        return;
    }

    action.action_metadata_fields = match action.action_kind {
        WorldActionKind::Move => vec![ActionMetadataField::Direction(0)],
        WorldActionKind::Eat => vec![
            ActionMetadataField::Direction(0),
            ActionMetadataField::Amount(1),
        ],
        WorldActionKind::Reproduce => vec![ActionMetadataField::Amount(1)],
        WorldActionKind::InventoryPickup => {
            vec![
                ActionMetadataField::Direction(0),
                ActionMetadataField::Amount(1),
            ]
        }
        WorldActionKind::InventoryPut => vec![
            ActionMetadataField::Direction(0),
            ActionMetadataField::Amount(1),
            ActionMetadataField::Slot(0),
        ],
        WorldActionKind::NoOp => Vec::new(),
    };
}

fn default_graph_node(node_id: u32) -> NodeGenome {
    NodeGenome {
        node_id,
        node_type: NodeType::Graph,
        backend_def: default_graph_backend(),
        output_definitions: Vec::new(),
        local_state_init: Vec::new(),
    }
}

fn default_graph_backend() -> BackendDef {
    BackendDef::Graph(GraphBackendDef {
        operator: GraphOperator::Passthrough,
        inputs: Vec::new(),
        coefficients: Vec::new(),
        bias: 0.0,
        state_slot_count: 0,
    })
}

fn default_vm_backend() -> BackendDef {
    BackendDef::Vm(VmBackendDef {
        register_count: 4,
        program: vec![VmInstruction::Noop, VmInstruction::Halt],
        constants: vec![0.0],
        max_input_slots: 8,
    })
}

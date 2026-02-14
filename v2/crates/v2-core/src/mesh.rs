use std::collections::{HashSet, VecDeque};

pub type NodeId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Graph,
    Vm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputReference {
    World(String),
    Introspection(String),
    Packet(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadField {
    Scalar { key: String, value: i32 },
    Flag { key: String, value: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionMetadataField {
    Direction(i32),
    Amount(u8),
    Slot(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldActionKind {
    Move,
    Eat,
    Reproduce,
    InventoryPickup,
    InventoryPut,
    NoOp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalTargetDef {
    pub target_node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub payload_fields: Vec<PayloadField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldActionDef {
    pub action_kind: WorldActionKind,
    pub action_metadata_fields: Vec<ActionMetadataField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputDefinition {
    InternalTarget(InternalTargetDef),
    WorldAction(WorldActionDef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmittedOutput {
    InternalTarget(InternalTargetDef),
    WorldAction(WorldActionDef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvolutionParams {
    pub mutation_bias: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeGenome {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub output_definitions: Vec<OutputDefinition>,
    pub local_state_init: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureGenome {
    pub entry_node_id: NodeId,
    pub nodes: Vec<NodeGenome>,
    pub evolution_params: Option<EvolutionParams>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshSchemaError {
    EmptyGenome,
    DuplicateNodeId(NodeId),
    MissingEntryNode(NodeId),
    InvalidInternalTarget { source: NodeId, target: NodeId },
    InvalidWorldActionMetadata(WorldActionKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshExecutionError {
    InvalidSchema(MeshSchemaError),
    InvalidTargetNode(NodeId),
    InvalidWorldActionMetadata(WorldActionKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionOutcome {
    WorldActionCommitted(WorldActionDef),
    ImplicitNoOp,
}

pub struct NodeExecutionInput<'a> {
    pub source_node_id: Option<NodeId>,
    pub target_node_id: NodeId,
    pub input_refs: &'a [InputReference],
    pub payload_fields: &'a [PayloadField],
    pub queue_depth: usize,
}

struct QueueItem {
    source_node_id: Option<NodeId>,
    target_node_id: NodeId,
    input_refs: Vec<InputReference>,
    payload_fields: Vec<PayloadField>,
}

impl CreatureGenome {
    pub fn validate(&self) -> Result<(), MeshSchemaError> {
        if self.nodes.is_empty() {
            return Err(MeshSchemaError::EmptyGenome);
        }

        let mut seen = HashSet::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if !seen.insert(node.node_id) {
                return Err(MeshSchemaError::DuplicateNodeId(node.node_id));
            }
        }

        if !seen.contains(&self.entry_node_id) {
            return Err(MeshSchemaError::MissingEntryNode(self.entry_node_id));
        }

        for node in &self.nodes {
            for output in &node.output_definitions {
                match output {
                    OutputDefinition::InternalTarget(target) => {
                        if !seen.contains(&target.target_node_id) {
                            return Err(MeshSchemaError::InvalidInternalTarget {
                                source: node.node_id,
                                target: target.target_node_id,
                            });
                        }
                    }
                    OutputDefinition::WorldAction(action) => {
                        if !world_action_metadata_is_valid(action) {
                            return Err(MeshSchemaError::InvalidWorldActionMetadata(
                                action.action_kind,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn run_mesh_queue<F>(
    genome: &CreatureGenome,
    mut execute_node: F,
) -> Result<ExecutionOutcome, MeshExecutionError>
where
    F: FnMut(NodeExecutionInput<'_>) -> Vec<EmittedOutput>,
{
    genome
        .validate()
        .map_err(MeshExecutionError::InvalidSchema)?;
    let node_ids = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<HashSet<_>>();
    let mut queue = VecDeque::from([QueueItem {
        source_node_id: None,
        target_node_id: genome.entry_node_id,
        input_refs: Vec::new(),
        payload_fields: Vec::new(),
    }]);

    while let Some(item) = queue.pop_front() {
        if !node_ids.contains(&item.target_node_id) {
            return Err(MeshExecutionError::InvalidTargetNode(item.target_node_id));
        }

        let emitted = execute_node(NodeExecutionInput {
            source_node_id: item.source_node_id,
            target_node_id: item.target_node_id,
            input_refs: &item.input_refs,
            payload_fields: &item.payload_fields,
            queue_depth: queue.len(),
        });

        for output in emitted {
            match output {
                EmittedOutput::InternalTarget(target) => {
                    if !node_ids.contains(&target.target_node_id) {
                        return Err(MeshExecutionError::InvalidTargetNode(target.target_node_id));
                    }
                    queue.push_back(QueueItem {
                        source_node_id: Some(item.target_node_id),
                        target_node_id: target.target_node_id,
                        input_refs: target.input_refs,
                        payload_fields: target.payload_fields,
                    });
                }
                EmittedOutput::WorldAction(action) => {
                    if !world_action_metadata_is_valid(&action) {
                        return Err(MeshExecutionError::InvalidWorldActionMetadata(
                            action.action_kind,
                        ));
                    }
                    return Ok(ExecutionOutcome::WorldActionCommitted(action));
                }
            }
        }
    }

    Ok(ExecutionOutcome::ImplicitNoOp)
}

fn has_direction(fields: &[ActionMetadataField]) -> bool {
    fields
        .iter()
        .any(|field| matches!(field, ActionMetadataField::Direction(_)))
}

fn has_amount(fields: &[ActionMetadataField]) -> bool {
    fields
        .iter()
        .any(|field| matches!(field, ActionMetadataField::Amount(_)))
}

fn has_slot(fields: &[ActionMetadataField]) -> bool {
    fields
        .iter()
        .any(|field| matches!(field, ActionMetadataField::Slot(_)))
}

fn world_action_metadata_is_valid(action: &WorldActionDef) -> bool {
    match action.action_kind {
        WorldActionKind::Move => has_direction(&action.action_metadata_fields),
        WorldActionKind::Eat => {
            has_direction(&action.action_metadata_fields)
                && has_amount(&action.action_metadata_fields)
        }
        WorldActionKind::Reproduce => true,
        WorldActionKind::InventoryPickup | WorldActionKind::InventoryPut => {
            has_direction(&action.action_metadata_fields)
                && has_slot(&action.action_metadata_fields)
        }
        WorldActionKind::NoOp => action.action_metadata_fields.is_empty(),
    }
}

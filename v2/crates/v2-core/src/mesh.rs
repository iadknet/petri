use std::collections::{HashMap, HashSet, VecDeque};

pub type NodeId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Graph,
    Vm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldInputKey {
    FoodHere,
    NearestFoodDistance,
    NearestFoodDirection,
    NearestCreatureDistance,
    NearestCreatureDirection,
    OccupiedHere,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrospectionInputKey {
    EnergyCurrent,
    EnergySpentThisTick,
    EnergyRemainingThisTick,
    AgeTicks,
    MemoryBytesTotal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorCellField {
    FoodDensityNorm,
    BarrierFlag,
    OccupiedFlag,
    IsSelfFlag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorCreatureField {
    PresentFlag,
    PhenotypeRNorm,
    PhenotypeGNorm,
    PhenotypeBNorm,
    EnergyNorm,
    AgeNorm,
    GenerationNorm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorSummaryField {
    VisibleCreatureCountNorm,
    VisibleFoodMeanNorm,
    VisibleFoodTotalNorm,
    CrowdingNorm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl NeighborDirection {
    #[must_use]
    pub fn to_offset(self) -> (i16, i16) {
        match self {
            Self::North => (0, -1),
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (1, 1),
            Self::South => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (-1, -1),
        }
    }

    #[must_use]
    pub fn from_ordinal(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::North),
            1 => Some(Self::NorthEast),
            2 => Some(Self::East),
            3 => Some(Self::SouthEast),
            4 => Some(Self::South),
            5 => Some(Self::SouthWest),
            6 => Some(Self::West),
            7 => Some(Self::NorthWest),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborCellField {
    FoodDensityNorm,
    BarrierFlag,
    OccupiedFlag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborCreatureField {
    PresentFlag,
    PhenotypeRNorm,
    PhenotypeGNorm,
    PhenotypeBNorm,
    EnergyNorm,
    AgeNorm,
    GenerationNorm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputReference {
    World(WorldInputKey),
    Introspection(IntrospectionInputKey),
    Packet(String),
    SensorCell {
        dx: i16,
        dy: i16,
        field: SensorCellField,
    },
    SensorCreature {
        dx: i16,
        dy: i16,
        field: SensorCreatureField,
    },
    SensorSummary(SensorSummaryField),
    NeighborCell {
        direction: NeighborDirection,
        field: NeighborCellField,
    },
    NeighborCreature {
        direction: NeighborDirection,
        field: NeighborCreatureField,
    },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum GraphOperator {
    Passthrough,
    WeightedSum,
    Threshold {
        threshold: f32,
    },
    Clamp01,
    DecayIntegrator {
        state_slot: u8,
        alpha: f32,
    },
    Momentum {
        state_slot: u8,
        beta: f32,
    },
    Oscillator {
        phase_slot: u8,
        frequency: f32,
        amplitude: f32,
        bias: f32,
    },
    SumPool,
    MeanPool,
    MaxPool,
    AdaptiveGain {
        gain_slot: u8,
        learning_rate: f32,
        min_gain: f32,
        max_gain: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphBackendDef {
    pub operator: GraphOperator,
    pub inputs: Vec<InputReference>,
    pub coefficients: Vec<f32>,
    pub bias: f32,
    pub state_slot_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmInstruction {
    Noop,
    LoadConst {
        dst: u8,
        const_idx: u8,
    },
    Move {
        dst: u8,
        src: u8,
    },
    Add {
        dst: u8,
        a: u8,
        b: u8,
    },
    Sub {
        dst: u8,
        a: u8,
        b: u8,
    },
    Mul {
        dst: u8,
        a: u8,
        b: u8,
    },
    Div {
        dst: u8,
        a: u8,
        b: u8,
    },
    Min {
        dst: u8,
        a: u8,
        b: u8,
    },
    Max {
        dst: u8,
        a: u8,
        b: u8,
    },
    Abs {
        dst: u8,
        src: u8,
    },
    Neg {
        dst: u8,
        src: u8,
    },
    Clamp01 {
        dst: u8,
        src: u8,
    },
    CmpGt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpLt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpEq {
        dst: u8,
        a: u8,
        b: u8,
        epsilon: f32,
    },
    And {
        dst: u8,
        a: u8,
        b: u8,
    },
    Or {
        dst: u8,
        a: u8,
        b: u8,
    },
    Not {
        dst: u8,
        src: u8,
    },
    ToI32 {
        dst: u8,
        src: u8,
    },
    ToU8 {
        dst: u8,
        src: u8,
    },
    ToBool {
        dst: u8,
        src: u8,
    },
    JumpIfZero {
        cond: u8,
        offset: i16,
    },
    Jump {
        offset: i16,
    },
    ReadInput {
        dst: u8,
        input_index: u8,
    },
    ReadSensorCell {
        dst: u8,
        dx: i16,
        dy: i16,
        field: SensorCellField,
    },
    ReadSensorCreature {
        dst: u8,
        dx: i16,
        dy: i16,
        field: SensorCreatureField,
    },
    ReadSensorSummary {
        dst: u8,
        field: SensorSummaryField,
    },
    ReadNeighborCell {
        dst: u8,
        direction: NeighborDirection,
        field: NeighborCellField,
    },
    ReadNeighborCreature {
        dst: u8,
        direction: NeighborDirection,
        field: NeighborCreatureField,
    },
    WriteInternalPayload {
        output_index: u8,
        payload_field_index: u8,
        src: u8,
    },
    WriteWorldActionMeta {
        output_index: u8,
        metadata_field_index: u8,
        src: u8,
    },
    EmitInternal {
        output_index: u8,
    },
    EmitWorldAction {
        output_index: u8,
    },
    Halt,
    LoadMem8 {
        dst: u8,
        addr_reg: u8,
    },
    StoreMem8 {
        addr_reg: u8,
        src: u8,
    },
    LoadMem8Imm {
        dst: u8,
        addr: u16,
    },
    StoreMem8Imm {
        addr: u16,
        src: u8,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct VmBackendDef {
    pub register_count: u8,
    pub program: Vec<VmInstruction>,
    pub constants: Vec<f32>,
    pub max_input_slots: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BackendDef {
    Graph(GraphBackendDef),
    Vm(VmBackendDef),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeGenome {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub backend_def: BackendDef,
    pub output_definitions: Vec<OutputDefinition>,
    pub local_state_init: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreatureGenome {
    pub entry_node_id: NodeId,
    pub nodes: Vec<NodeGenome>,
    pub evolution_params: Option<EvolutionParams>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshSchemaError {
    EmptyGenome,
    DuplicateNodeId(NodeId),
    MissingEntryNode(NodeId),
    InvalidInternalTarget {
        source: NodeId,
        target: NodeId,
    },
    InvalidWorldActionMetadata(WorldActionKind),
    NodeBackendMismatch(NodeId),
    InvalidBackendConfig(NodeId),
    DuplicatePayloadFieldKey {
        source: NodeId,
        key: String,
    },
    VmInputSlotsExceeded {
        source: NodeId,
        target: NodeId,
        provided: usize,
        max: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
        let mut nodes_by_id = HashMap::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if !seen.insert(node.node_id) {
                return Err(MeshSchemaError::DuplicateNodeId(node.node_id));
            }
            nodes_by_id.insert(node.node_id, node);
        }

        if !seen.contains(&self.entry_node_id) {
            return Err(MeshSchemaError::MissingEntryNode(self.entry_node_id));
        }

        for node in &self.nodes {
            validate_backend(node)?;
            for output in &node.output_definitions {
                match output {
                    OutputDefinition::InternalTarget(target) => {
                        if !seen.contains(&target.target_node_id) {
                            return Err(MeshSchemaError::InvalidInternalTarget {
                                source: node.node_id,
                                target: target.target_node_id,
                            });
                        }
                        validate_payload_keys(node.node_id, &target.payload_fields)?;
                        let target_node = nodes_by_id
                            .get(&target.target_node_id)
                            .expect("target presence already checked");
                        if let BackendDef::Vm(vm) = &target_node.backend_def {
                            let provided = target.input_refs.len();
                            let max = usize::from(vm.max_input_slots);
                            if provided > max {
                                return Err(MeshSchemaError::VmInputSlotsExceeded {
                                    source: node.node_id,
                                    target: target.target_node_id,
                                    provided,
                                    max,
                                });
                            }
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

#[must_use]
pub fn emitted_output_from_definition(definition: &OutputDefinition) -> EmittedOutput {
    match definition {
        OutputDefinition::InternalTarget(target) => EmittedOutput::InternalTarget(target.clone()),
        OutputDefinition::WorldAction(action) => EmittedOutput::WorldAction(action.clone()),
    }
}

#[must_use]
pub fn world_action_metadata_is_valid(action: &WorldActionDef) -> bool {
    if has_duplicate_metadata_kinds(&action.action_metadata_fields) {
        return false;
    }

    if action
        .action_metadata_fields
        .iter()
        .filter_map(|field| match field {
            ActionMetadataField::Direction(value) => Some(*value),
            _ => None,
        })
        .any(|direction| !(0..=7).contains(&direction))
    {
        return false;
    }

    let has_direction = has_metadata_kind(&action.action_metadata_fields, |field| {
        matches!(field, ActionMetadataField::Direction(_))
    });
    let has_amount = has_metadata_kind(&action.action_metadata_fields, |field| {
        matches!(field, ActionMetadataField::Amount(_))
    });
    let has_slot = has_metadata_kind(&action.action_metadata_fields, |field| {
        matches!(field, ActionMetadataField::Slot(_))
    });

    match action.action_kind {
        WorldActionKind::Move => has_direction && !has_amount && !has_slot,
        WorldActionKind::Eat => has_direction && has_amount && !has_slot,
        WorldActionKind::Reproduce => has_amount && !has_slot,
        WorldActionKind::InventoryPickup => has_direction && has_amount && !has_slot,
        WorldActionKind::InventoryPut => has_direction && has_amount && has_slot,
        WorldActionKind::NoOp => action.action_metadata_fields.is_empty(),
    }
}

fn has_metadata_kind<F>(fields: &[ActionMetadataField], mut predicate: F) -> bool
where
    F: FnMut(&ActionMetadataField) -> bool,
{
    fields.iter().any(&mut predicate)
}

fn has_duplicate_metadata_kinds(fields: &[ActionMetadataField]) -> bool {
    let mut seen_direction = false;
    let mut seen_amount = false;
    let mut seen_slot = false;
    for field in fields {
        match field {
            ActionMetadataField::Direction(_) => {
                if seen_direction {
                    return true;
                }
                seen_direction = true;
            }
            ActionMetadataField::Amount(_) => {
                if seen_amount {
                    return true;
                }
                seen_amount = true;
            }
            ActionMetadataField::Slot(_) => {
                if seen_slot {
                    return true;
                }
                seen_slot = true;
            }
        }
    }
    false
}

fn validate_payload_keys(source: NodeId, fields: &[PayloadField]) -> Result<(), MeshSchemaError> {
    let mut keys = HashSet::with_capacity(fields.len());
    for field in fields {
        let key = match field {
            PayloadField::Scalar { key, .. } | PayloadField::Flag { key, .. } => key,
        };
        if !keys.insert(key.clone()) {
            return Err(MeshSchemaError::DuplicatePayloadFieldKey {
                source,
                key: key.clone(),
            });
        }
    }
    Ok(())
}

fn validate_backend(node: &NodeGenome) -> Result<(), MeshSchemaError> {
    match (&node.node_type, &node.backend_def) {
        (NodeType::Graph, BackendDef::Graph(graph)) => {
            if graph.state_slot_count > 8 {
                return Err(MeshSchemaError::InvalidBackendConfig(node.node_id));
            }
            Ok(())
        }
        (NodeType::Vm, BackendDef::Vm(vm)) => {
            if vm.register_count == 0
                || vm.register_count > 32
                || vm.max_input_slots == 0
                || vm.max_input_slots > 64
                || vm.program.is_empty()
                || vm.program.len() > 128
                || vm.constants.len() > 64
            {
                return Err(MeshSchemaError::InvalidBackendConfig(node.node_id));
            }
            Ok(())
        }
        _ => Err(MeshSchemaError::NodeBackendMismatch(node.node_id)),
    }
}

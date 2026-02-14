use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::backends::{evaluate_graph_operator, graph_operator_cost_multiplier};
use crate::energy::{
    charge_action, charge_backend_graph, charge_dispatch_entry, vm_opcode_base_cost,
};
use crate::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, EmittedOutput, InputReference,
    IntrospectionInputKey, NeighborCellField, NeighborCreatureField, NeighborDirection, NodeId,
    OutputDefinition, PayloadField, SensorCellField, SensorCreatureField, SensorSummaryField,
    VmBackendDef, VmInstruction, WorldActionDef, WorldActionKind, WorldInputKey,
    emitted_output_from_definition, world_action_metadata_is_valid,
};

pub const MEMORY_BYTES: usize = 1024;
const ENERGY_CLAMP_ABS: f32 = 1_000_000_000.0;
const SENSOR_ENERGY_NORM_SCALE: f32 = 10.0;
const SENSOR_AGE_NORM_TICKS: f32 = 10_000.0;
const SENSOR_GENERATION_NORM: f32 = 256.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeConfigError {
    SensorRadiusZero,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeActionCosts {
    pub move_cost: f32,
    pub eat_cost: f32,
    pub reproduce_cost: f32,
    pub inventory_pickup_cost: f32,
    pub inventory_put_cost: f32,
    pub noop_cost: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeConfig {
    pub dispatch_entry_cost: f32,
    pub graph_base_tariff: f32,
    pub vm_opcode_cost_multiplier: f32,
    pub sensor_radius: u16,
    pub action_costs: RuntimeActionCosts,
}

impl RuntimeConfig {
    pub fn validate(&self) -> Result<(), RuntimeConfigError> {
        if self.sensor_radius == 0 {
            return Err(RuntimeConfigError::SensorRadiusZero);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeContext {
    pub config: RuntimeConfig,
    pub energy_before_tick: f32,
    pub memory_bytes: [u8; MEMORY_BYTES],
    pub graph_state_slots: Vec<f32>,
    pub sensor_frame: SensorFrame,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeOutcome {
    CommittedAction {
        action: WorldActionDef,
        energy_spent: f32,
        energy_remaining: f32,
        dispatches: usize,
    },
    ImplicitNoOp {
        energy_spent: f32,
        energy_remaining: f32,
        dispatches: usize,
    },
    EnergyExhausted {
        energy_spent: f32,
        dispatches: usize,
    },
    RuntimeError {
        error: RuntimeError,
        dispatches: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    Schema(crate::mesh::MeshSchemaError),
    InvalidTarget(NodeId),
    InvalidActionMetadata(WorldActionKind),
    InsufficientEnergyForDispatch,
    VmFault(VmFaultCode),
    InvalidConfig(RuntimeConfigError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmFaultCode {
    InvalidRegisterIndex,
    InvalidConstIndex,
    InvalidOutputIndex,
    InvalidOutputFieldIndex,
    InvalidSensorField,
    InvalidNeighborField,
    InvalidNeighborDirection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PacketValue {
    Bool(bool),
    I32(i32),
    F32(f32),
    U8(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorCreatureSnapshot {
    pub phenotype_rgb: [u8; 3],
    pub energy: f32,
    pub age_ticks: u64,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorCellSnapshot {
    pub food_density_u8: u8,
    pub barrier_flag: bool,
    pub occupied_flag: bool,
    pub is_self: bool,
    pub creature: Option<SensorCreatureSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SensorFrame {
    radius: u16,
    cells: BTreeMap<(i16, i16), SensorCellSnapshot>,
}

impl SensorFrame {
    #[must_use]
    pub fn empty(radius: u16) -> Self {
        Self {
            radius,
            cells: BTreeMap::new(),
        }
    }

    pub fn insert_cell(&mut self, dx: i16, dy: i16, snapshot: SensorCellSnapshot) {
        self.cells.insert((dx, dy), snapshot);
    }

    #[must_use]
    pub fn read_sensor_cell(&self, dx: i16, dy: i16, field: SensorCellField) -> f32 {
        if dx.unsigned_abs() > self.radius || dy.unsigned_abs() > self.radius {
            return 0.0;
        }
        let Some(cell) = self.cells.get(&(dx, dy)) else {
            return 0.0;
        };
        match field {
            SensorCellField::FoodDensityNorm => f32::from(cell.food_density_u8) / 255.0,
            SensorCellField::BarrierFlag => bool_to_f32(cell.barrier_flag),
            SensorCellField::OccupiedFlag => bool_to_f32(cell.occupied_flag),
            SensorCellField::IsSelfFlag => bool_to_f32(cell.is_self),
        }
    }

    #[must_use]
    pub fn read_sensor_creature(&self, dx: i16, dy: i16, field: SensorCreatureField) -> f32 {
        if dx.unsigned_abs() > self.radius || dy.unsigned_abs() > self.radius {
            return 0.0;
        }
        let Some(cell) = self.cells.get(&(dx, dy)) else {
            return 0.0;
        };
        let Some(creature) = cell.creature else {
            return 0.0;
        };
        match field {
            SensorCreatureField::PresentFlag => 1.0,
            SensorCreatureField::PhenotypeRNorm => f32::from(creature.phenotype_rgb[0]) / 255.0,
            SensorCreatureField::PhenotypeGNorm => f32::from(creature.phenotype_rgb[1]) / 255.0,
            SensorCreatureField::PhenotypeBNorm => f32::from(creature.phenotype_rgb[2]) / 255.0,
            SensorCreatureField::EnergyNorm => {
                (creature.energy / SENSOR_ENERGY_NORM_SCALE).clamp(0.0, 1.0)
            }
            SensorCreatureField::AgeNorm => {
                ((creature.age_ticks as f32) / SENSOR_AGE_NORM_TICKS).clamp(0.0, 1.0)
            }
            SensorCreatureField::GenerationNorm => {
                ((creature.generation as f32) / SENSOR_GENERATION_NORM).clamp(0.0, 1.0)
            }
        }
    }

    #[must_use]
    pub fn read_sensor_summary(&self, field: SensorSummaryField) -> f32 {
        let max_visible_cells = ((u32::from(self.radius) * 2 + 1).pow(2)).max(1) as f32;
        let visible_creature_count = self
            .cells
            .values()
            .filter(|cell| cell.creature.is_some())
            .count();
        let visible_food_total_norm = self
            .cells
            .values()
            .map(|cell| f32::from(cell.food_density_u8) / 255.0)
            .sum::<f32>();
        match field {
            SensorSummaryField::VisibleCreatureCountNorm => {
                (visible_creature_count as f32 / max_visible_cells).clamp(0.0, 1.0)
            }
            SensorSummaryField::VisibleFoodMeanNorm => {
                if self.cells.is_empty() {
                    0.0
                } else {
                    visible_food_total_norm / self.cells.len() as f32
                }
            }
            SensorSummaryField::VisibleFoodTotalNorm => {
                (visible_food_total_norm / max_visible_cells).clamp(0.0, 1.0)
            }
            SensorSummaryField::CrowdingNorm => {
                (visible_creature_count as f32 / max_visible_cells).clamp(0.0, 1.0)
            }
        }
    }

    #[must_use]
    pub fn read_neighbor_cell(
        &self,
        direction: NeighborDirection,
        field: NeighborCellField,
    ) -> f32 {
        let (dx, dy) = direction.to_offset();
        let mapped = match field {
            NeighborCellField::FoodDensityNorm => SensorCellField::FoodDensityNorm,
            NeighborCellField::BarrierFlag => SensorCellField::BarrierFlag,
            NeighborCellField::OccupiedFlag => SensorCellField::OccupiedFlag,
        };
        self.read_sensor_cell(dx, dy, mapped)
    }

    #[must_use]
    pub fn read_neighbor_creature(
        &self,
        direction: NeighborDirection,
        field: NeighborCreatureField,
    ) -> f32 {
        let (dx, dy) = direction.to_offset();
        let mapped = match field {
            NeighborCreatureField::PresentFlag => SensorCreatureField::PresentFlag,
            NeighborCreatureField::PhenotypeRNorm => SensorCreatureField::PhenotypeRNorm,
            NeighborCreatureField::PhenotypeGNorm => SensorCreatureField::PhenotypeGNorm,
            NeighborCreatureField::PhenotypeBNorm => SensorCreatureField::PhenotypeBNorm,
            NeighborCreatureField::EnergyNorm => SensorCreatureField::EnergyNorm,
            NeighborCreatureField::AgeNorm => SensorCreatureField::AgeNorm,
            NeighborCreatureField::GenerationNorm => SensorCreatureField::GenerationNorm,
        };
        self.read_sensor_creature(dx, dy, mapped)
    }
}

#[must_use]
pub fn sanitize_f32(value: f32) -> f32 {
    if value.is_nan() {
        return 0.0;
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            -ENERGY_CLAMP_ABS
        } else {
            ENERGY_CLAMP_ABS
        };
    }
    value.clamp(-ENERGY_CLAMP_ABS, ENERGY_CLAMP_ABS)
}

#[must_use]
pub fn resolve_graph_inputs(input_refs: &[InputReference], sensor_frame: &SensorFrame) -> Vec<f32> {
    resolve_input_slots(input_refs, &[], sensor_frame, 0.0, 0.0, 0.0, 0.0)
}

#[must_use]
pub fn resolve_input_slots(
    input_refs: &[InputReference],
    packet_fields: &[(String, PacketValue)],
    sensor_frame: &SensorFrame,
    energy_current: f32,
    energy_spent_this_tick: f32,
    energy_remaining_this_tick: f32,
    age_ticks: f32,
) -> Vec<f32> {
    let packet_map = packet_fields
        .iter()
        .cloned()
        .collect::<HashMap<String, PacketValue>>();

    input_refs
        .iter()
        .map(|input| {
            let value = match input {
                InputReference::World(key) => match key {
                    WorldInputKey::FoodHere => {
                        sensor_frame.read_sensor_cell(0, 0, SensorCellField::FoodDensityNorm)
                    }
                    WorldInputKey::NearestFoodDistance => 1.0,
                    WorldInputKey::NearestFoodDirection => 0.0,
                    WorldInputKey::NearestCreatureDistance => 1.0,
                    WorldInputKey::NearestCreatureDirection => 0.0,
                    WorldInputKey::OccupiedHere => {
                        sensor_frame.read_sensor_cell(0, 0, SensorCellField::OccupiedFlag)
                    }
                },
                InputReference::Introspection(key) => match key {
                    IntrospectionInputKey::EnergyCurrent => energy_current,
                    IntrospectionInputKey::EnergySpentThisTick => energy_spent_this_tick,
                    IntrospectionInputKey::EnergyRemainingThisTick => energy_remaining_this_tick,
                    IntrospectionInputKey::AgeTicks => age_ticks,
                    IntrospectionInputKey::MemoryBytesTotal => MEMORY_BYTES as f32,
                },
                InputReference::Packet(key) => packet_map
                    .get(key)
                    .map_or(0.0, |value| packet_value_as_f32(*value)),
                InputReference::SensorCell { dx, dy, field } => {
                    sensor_frame.read_sensor_cell(*dx, *dy, *field)
                }
                InputReference::SensorCreature { dx, dy, field } => {
                    sensor_frame.read_sensor_creature(*dx, *dy, *field)
                }
                InputReference::SensorSummary(field) => sensor_frame.read_sensor_summary(*field),
                InputReference::NeighborCell { direction, field } => {
                    sensor_frame.read_neighbor_cell(*direction, *field)
                }
                InputReference::NeighborCreature { direction, field } => {
                    sensor_frame.read_neighbor_creature(*direction, *field)
                }
            };
            sanitize_f32(value)
        })
        .collect()
}

#[must_use]
pub fn run_runtime_tick(genome: &CreatureGenome, context: &mut RuntimeContext) -> RuntimeOutcome {
    if let Err(error) = context.config.validate() {
        return RuntimeOutcome::RuntimeError {
            error: RuntimeError::InvalidConfig(error),
            dispatches: 0,
        };
    }
    if let Err(error) = genome.validate() {
        return RuntimeOutcome::RuntimeError {
            error: RuntimeError::Schema(error),
            dispatches: 0,
        };
    }

    let mut node_map = HashMap::with_capacity(genome.nodes.len());
    for node in &genome.nodes {
        node_map.insert(node.node_id, node);
    }

    let mut queue = VecDeque::from([QueueItem {
        target_node_id: genome.entry_node_id,
        input_refs: Vec::new(),
        payload_fields: Vec::new(),
    }]);

    let mut energy_remaining = context.energy_before_tick.max(0.0);
    let mut energy_spent = 0.0_f32;
    let mut dispatches = 0_usize;

    while let Some(item) = queue.pop_front() {
        let dispatch_charge =
            charge_dispatch_entry(energy_remaining, context.config.dispatch_entry_cost);
        energy_spent += dispatch_charge.charged_energy;
        energy_remaining = dispatch_charge.remaining_energy;
        if dispatch_charge.exhausted {
            return RuntimeOutcome::EnergyExhausted {
                energy_spent,
                dispatches,
            };
        }

        let Some(node) = node_map.get(&item.target_node_id) else {
            return RuntimeOutcome::RuntimeError {
                error: RuntimeError::InvalidTarget(item.target_node_id),
                dispatches,
            };
        };
        dispatches += 1;

        let backend = run_backend(node, &item, context, energy_remaining, energy_spent);
        energy_spent += backend.charged_energy;
        energy_remaining = backend.remaining_energy;
        if let Some(error) = backend.error {
            return RuntimeOutcome::RuntimeError { error, dispatches };
        }
        if backend.exhausted {
            return RuntimeOutcome::EnergyExhausted {
                energy_spent,
                dispatches,
            };
        }

        for emitted in backend.emitted {
            match emitted {
                EmittedOutput::InternalTarget(target) => {
                    if !node_map.contains_key(&target.target_node_id) {
                        return RuntimeOutcome::RuntimeError {
                            error: RuntimeError::InvalidTarget(target.target_node_id),
                            dispatches,
                        };
                    }
                    queue.push_back(QueueItem {
                        target_node_id: target.target_node_id,
                        input_refs: target.input_refs,
                        payload_fields: target.payload_fields,
                    });
                }
                EmittedOutput::WorldAction(action) => {
                    if !world_action_metadata_is_valid(&action) {
                        return RuntimeOutcome::RuntimeError {
                            error: RuntimeError::InvalidActionMetadata(action.action_kind),
                            dispatches,
                        };
                    }
                    let action_cost = action_cost(&context.config.action_costs, action.action_kind);
                    let action_charge = charge_action(energy_remaining, action_cost);
                    energy_spent += action_charge.charged_energy;
                    energy_remaining = action_charge.remaining_energy;
                    return RuntimeOutcome::CommittedAction {
                        action,
                        energy_spent,
                        energy_remaining,
                        dispatches,
                    };
                }
            }
        }
    }

    RuntimeOutcome::ImplicitNoOp {
        energy_spent,
        energy_remaining,
        dispatches,
    }
}

struct QueueItem {
    target_node_id: NodeId,
    input_refs: Vec<InputReference>,
    payload_fields: Vec<PayloadField>,
}

struct BackendRun {
    emitted: Vec<EmittedOutput>,
    charged_energy: f32,
    remaining_energy: f32,
    exhausted: bool,
    error: Option<RuntimeError>,
}

fn run_backend(
    node: &crate::mesh::NodeGenome,
    item: &QueueItem,
    context: &mut RuntimeContext,
    energy_remaining: f32,
    energy_spent_so_far: f32,
) -> BackendRun {
    match &node.backend_def {
        BackendDef::Graph(graph) => {
            let graph_inputs = resolve_graph_inputs(&graph.inputs, &context.sensor_frame);
            let required_slots = usize::from(graph.state_slot_count).max(1);
            if context.graph_state_slots.len() < required_slots {
                context.graph_state_slots.resize(required_slots, 0.0);
            }
            let state_slice = &mut context.graph_state_slots[..required_slots];
            let _ = evaluate_graph_operator(graph, &graph_inputs, state_slice);
            let requested = context.config.graph_base_tariff
                * graph_operator_cost_multiplier(graph.operator.clone());
            let charge = charge_backend_graph(energy_remaining, requested);
            let emitted = node
                .output_definitions
                .iter()
                .map(emitted_output_from_definition)
                .collect::<Vec<_>>();
            BackendRun {
                emitted,
                charged_energy: charge.charged_energy,
                remaining_energy: charge.remaining_energy,
                exhausted: charge.exhausted && requested > charge.charged_energy,
                error: None,
            }
        }
        BackendDef::Vm(vm) => {
            let packet_fields = payload_fields_as_packet_values(&item.payload_fields);
            let input_slots = resolve_input_slots(
                &item.input_refs,
                &packet_fields,
                &context.sensor_frame,
                context.energy_before_tick,
                energy_spent_so_far,
                energy_remaining,
                0.0,
            );
            let result = execute_vm(
                vm,
                &node.output_definitions,
                &input_slots,
                &context.sensor_frame,
                &mut context.memory_bytes,
                energy_remaining,
                context.config.vm_opcode_cost_multiplier,
            );
            BackendRun {
                emitted: result.emitted,
                charged_energy: result.charged_energy,
                remaining_energy: result.remaining_energy,
                exhausted: result.exhausted,
                error: result.fault.map(RuntimeError::VmFault),
            }
        }
    }
}

struct VmExecution {
    emitted: Vec<EmittedOutput>,
    charged_energy: f32,
    remaining_energy: f32,
    exhausted: bool,
    fault: Option<VmFaultCode>,
}

fn execute_vm(
    vm: &VmBackendDef,
    output_definitions: &[OutputDefinition],
    input_slots: &[f32],
    sensor_frame: &SensorFrame,
    memory: &mut [u8; MEMORY_BYTES],
    starting_energy: f32,
    opcode_cost_multiplier: f32,
) -> VmExecution {
    let mut registers = vec![0.0_f32; usize::from(vm.register_count)];
    let mut pc: i32 = 0;
    let mut emitted = Vec::new();
    let mut remaining_energy = starting_energy.max(0.0);
    let mut charged_energy = 0.0_f32;
    let mut payload_overrides = HashMap::<(usize, usize), f32>::new();
    let mut metadata_overrides = HashMap::<(usize, usize), f32>::new();

    while let Some(instruction) = vm.program.get(pc as usize).cloned() {
        let op_cost = vm_opcode_base_cost(&instruction) * opcode_cost_multiplier.max(0.0);
        if remaining_energy + f32::EPSILON < op_cost {
            return VmExecution {
                emitted,
                charged_energy,
                remaining_energy: 0.0,
                exhausted: true,
                fault: None,
            };
        }
        remaining_energy -= op_cost;
        charged_energy += op_cost;
        pc += 1;

        if let Err(fault) = execute_instruction(
            &instruction,
            &mut registers,
            vm,
            output_definitions,
            input_slots,
            sensor_frame,
            memory,
            &mut pc,
            &mut payload_overrides,
            &mut metadata_overrides,
            &mut emitted,
        ) {
            return VmExecution {
                emitted,
                charged_energy,
                remaining_energy,
                exhausted: false,
                fault: Some(fault),
            };
        }

        if matches!(instruction, VmInstruction::Halt) {
            break;
        }
        if pc < 0 {
            break;
        }
    }

    VmExecution {
        emitted,
        charged_energy,
        remaining_energy,
        exhausted: false,
        fault: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_instruction(
    instruction: &VmInstruction,
    registers: &mut [f32],
    vm: &VmBackendDef,
    output_definitions: &[OutputDefinition],
    input_slots: &[f32],
    sensor_frame: &SensorFrame,
    memory: &mut [u8; MEMORY_BYTES],
    pc: &mut i32,
    payload_overrides: &mut HashMap<(usize, usize), f32>,
    metadata_overrides: &mut HashMap<(usize, usize), f32>,
    emitted: &mut Vec<EmittedOutput>,
) -> Result<(), VmFaultCode> {
    match instruction {
        VmInstruction::Noop => {}
        VmInstruction::LoadConst { dst, const_idx } => {
            let value = *vm
                .constants
                .get(usize::from(*const_idx))
                .ok_or(VmFaultCode::InvalidConstIndex)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Move { dst, src } => {
            let value = read_register(registers, *src)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Add { dst, a, b } => {
            let value = read_register(registers, *a)? + read_register(registers, *b)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Sub { dst, a, b } => {
            let value = read_register(registers, *a)? - read_register(registers, *b)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Mul { dst, a, b } => {
            let value = read_register(registers, *a)? * read_register(registers, *b)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Div { dst, a, b } => {
            let denominator = read_register(registers, *b)?;
            let value = if denominator.abs() <= f32::EPSILON {
                0.0
            } else {
                read_register(registers, *a)? / denominator
            };
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Min { dst, a, b } => {
            let value = read_register(registers, *a)?.min(read_register(registers, *b)?);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Max { dst, a, b } => {
            let value = read_register(registers, *a)?.max(read_register(registers, *b)?);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Abs { dst, src } => {
            let value = read_register(registers, *src)?.abs();
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Neg { dst, src } => {
            let value = -read_register(registers, *src)?;
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Clamp01 { dst, src } => {
            let value = read_register(registers, *src)?.clamp(0.0, 1.0);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::CmpGt { dst, a, b } => {
            let value = bool_to_f32(read_register(registers, *a)? > read_register(registers, *b)?);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::CmpLt { dst, a, b } => {
            let value = bool_to_f32(read_register(registers, *a)? < read_register(registers, *b)?);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::CmpEq { dst, a, b, epsilon } => {
            let epsilon = epsilon.clamp(1e-6, 1.0);
            let value = bool_to_f32(
                (read_register(registers, *a)? - read_register(registers, *b)?).abs() <= epsilon,
            );
            set_register(registers, *dst, value)?;
        }
        VmInstruction::And { dst, a, b } => {
            let value = bool_to_f32(
                to_bool(read_register(registers, *a)?) && to_bool(read_register(registers, *b)?),
            );
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Or { dst, a, b } => {
            let value = bool_to_f32(
                to_bool(read_register(registers, *a)?) || to_bool(read_register(registers, *b)?),
            );
            set_register(registers, *dst, value)?;
        }
        VmInstruction::Not { dst, src } => {
            let value = bool_to_f32(!to_bool(read_register(registers, *src)?));
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ToI32 { dst, src } => {
            let value = read_register(registers, *src)?.round();
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ToU8 { dst, src } => {
            let value = read_register(registers, *src)?.clamp(0.0, 255.0).round();
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ToBool { dst, src } => {
            let value = bool_to_f32(to_bool(read_register(registers, *src)?));
            set_register(registers, *dst, value)?;
        }
        VmInstruction::JumpIfZero { cond, offset } => {
            if read_register(registers, *cond)? == 0.0 {
                *pc += i32::from(*offset);
            }
        }
        VmInstruction::Jump { offset } => {
            *pc += i32::from(*offset);
        }
        VmInstruction::ReadInput { dst, input_index } => {
            let value = input_slots
                .get(usize::from(*input_index))
                .copied()
                .unwrap_or(0.0);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ReadSensorCell { dst, dx, dy, field } => {
            let value = sensor_frame.read_sensor_cell(*dx, *dy, *field);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ReadSensorCreature { dst, dx, dy, field } => {
            let value = sensor_frame.read_sensor_creature(*dx, *dy, *field);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ReadSensorSummary { dst, field } => {
            let value = sensor_frame.read_sensor_summary(*field);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ReadNeighborCell {
            dst,
            direction,
            field,
        } => {
            let value = sensor_frame.read_neighbor_cell(*direction, *field);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::ReadNeighborCreature {
            dst,
            direction,
            field,
        } => {
            let value = sensor_frame.read_neighbor_creature(*direction, *field);
            set_register(registers, *dst, value)?;
        }
        VmInstruction::WriteInternalPayload {
            output_index,
            payload_field_index,
            src,
        } => {
            verify_internal_output(
                output_definitions,
                usize::from(*output_index),
                usize::from(*payload_field_index),
            )?;
            payload_overrides.insert(
                (
                    usize::from(*output_index),
                    usize::from(*payload_field_index),
                ),
                read_register(registers, *src)?,
            );
        }
        VmInstruction::WriteWorldActionMeta {
            output_index,
            metadata_field_index,
            src,
        } => {
            verify_world_output(
                output_definitions,
                usize::from(*output_index),
                usize::from(*metadata_field_index),
            )?;
            metadata_overrides.insert(
                (
                    usize::from(*output_index),
                    usize::from(*metadata_field_index),
                ),
                read_register(registers, *src)?,
            );
        }
        VmInstruction::EmitInternal { output_index } => {
            let output_index = usize::from(*output_index);
            let output = output_definitions
                .get(output_index)
                .ok_or(VmFaultCode::InvalidOutputIndex)?;
            let OutputDefinition::InternalTarget(target) = output else {
                return Err(VmFaultCode::InvalidOutputIndex);
            };
            let mut emitted_target = target.clone();
            for (field_index, field) in emitted_target.payload_fields.iter_mut().enumerate() {
                if let Some(value) = payload_overrides.get(&(output_index, field_index)) {
                    apply_payload_override(field, *value);
                }
            }
            payload_overrides.retain(|(index, _), _| *index != output_index);
            emitted.push(EmittedOutput::InternalTarget(emitted_target));
        }
        VmInstruction::EmitWorldAction { output_index } => {
            let output_index = usize::from(*output_index);
            let output = output_definitions
                .get(output_index)
                .ok_or(VmFaultCode::InvalidOutputIndex)?;
            let OutputDefinition::WorldAction(action) = output else {
                return Err(VmFaultCode::InvalidOutputIndex);
            };
            let mut emitted_action = action.clone();
            for (field_index, field) in emitted_action.action_metadata_fields.iter_mut().enumerate()
            {
                if let Some(value) = metadata_overrides.get(&(output_index, field_index)) {
                    apply_metadata_override(field, *value);
                }
            }
            metadata_overrides.retain(|(index, _), _| *index != output_index);
            emitted.push(EmittedOutput::WorldAction(emitted_action));
        }
        VmInstruction::Halt => {}
        VmInstruction::LoadMem8 { dst, addr_reg } => {
            let addr = resolve_memory_addr(read_register(registers, *addr_reg)?);
            set_register(registers, *dst, f32::from(memory[addr]))?;
        }
        VmInstruction::StoreMem8 { addr_reg, src } => {
            let addr = resolve_memory_addr(read_register(registers, *addr_reg)?);
            memory[addr] = to_u8(read_register(registers, *src)?);
        }
        VmInstruction::LoadMem8Imm { dst, addr } => {
            let addr = usize::from(*addr) % MEMORY_BYTES;
            set_register(registers, *dst, f32::from(memory[addr]))?;
        }
        VmInstruction::StoreMem8Imm { addr, src } => {
            let addr = usize::from(*addr) % MEMORY_BYTES;
            memory[addr] = to_u8(read_register(registers, *src)?);
        }
    }
    Ok(())
}

fn verify_internal_output(
    outputs: &[OutputDefinition],
    output_index: usize,
    field_index: usize,
) -> Result<(), VmFaultCode> {
    let Some(output) = outputs.get(output_index) else {
        return Err(VmFaultCode::InvalidOutputIndex);
    };
    let OutputDefinition::InternalTarget(target) = output else {
        return Err(VmFaultCode::InvalidOutputIndex);
    };
    if field_index >= target.payload_fields.len() {
        return Err(VmFaultCode::InvalidOutputFieldIndex);
    }
    Ok(())
}

fn verify_world_output(
    outputs: &[OutputDefinition],
    output_index: usize,
    field_index: usize,
) -> Result<(), VmFaultCode> {
    let Some(output) = outputs.get(output_index) else {
        return Err(VmFaultCode::InvalidOutputIndex);
    };
    let OutputDefinition::WorldAction(action) = output else {
        return Err(VmFaultCode::InvalidOutputIndex);
    };
    if field_index >= action.action_metadata_fields.len() {
        return Err(VmFaultCode::InvalidOutputFieldIndex);
    }
    Ok(())
}

fn apply_payload_override(field: &mut PayloadField, value: f32) {
    match field {
        PayloadField::Scalar {
            value: existing, ..
        } => {
            *existing = value.round() as i32;
        }
        PayloadField::Flag {
            value: existing, ..
        } => {
            *existing = to_bool(value);
        }
    }
}

fn apply_metadata_override(field: &mut ActionMetadataField, value: f32) {
    match field {
        ActionMetadataField::Direction(existing) => {
            *existing = value.round() as i32;
        }
        ActionMetadataField::Amount(existing) => {
            *existing = to_u8(value);
        }
        ActionMetadataField::Slot(existing) => {
            *existing = to_u8(value);
        }
    }
}

fn payload_fields_as_packet_values(fields: &[PayloadField]) -> Vec<(String, PacketValue)> {
    fields
        .iter()
        .map(|field| match field {
            PayloadField::Scalar { key, value } => (key.clone(), PacketValue::I32(*value)),
            PayloadField::Flag { key, value } => (key.clone(), PacketValue::Bool(*value)),
        })
        .collect()
}

fn action_cost(costs: &RuntimeActionCosts, kind: WorldActionKind) -> f32 {
    match kind {
        WorldActionKind::Move => costs.move_cost,
        WorldActionKind::Eat => costs.eat_cost,
        WorldActionKind::Reproduce => costs.reproduce_cost,
        WorldActionKind::InventoryPickup => costs.inventory_pickup_cost,
        WorldActionKind::InventoryPut => costs.inventory_put_cost,
        WorldActionKind::NoOp => costs.noop_cost,
    }
}

fn read_register(registers: &[f32], index: u8) -> Result<f32, VmFaultCode> {
    registers
        .get(usize::from(index))
        .copied()
        .ok_or(VmFaultCode::InvalidRegisterIndex)
}

fn set_register(registers: &mut [f32], index: u8, value: f32) -> Result<(), VmFaultCode> {
    let Some(slot) = registers.get_mut(usize::from(index)) else {
        return Err(VmFaultCode::InvalidRegisterIndex);
    };
    *slot = sanitize_f32(value);
    Ok(())
}

fn resolve_memory_addr(value: f32) -> usize {
    let raw = value.round() as i32;
    raw.rem_euclid(MEMORY_BYTES as i32) as usize
}

fn bool_to_f32(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn to_bool(value: f32) -> bool {
    value >= 0.5
}

fn to_u8(value: f32) -> u8 {
    value.clamp(0.0, 255.0).round() as u8
}

fn packet_value_as_f32(value: PacketValue) -> f32 {
    match value {
        PacketValue::Bool(value) => bool_to_f32(value),
        PacketValue::I32(value) => value as f32,
        PacketValue::F32(value) => sanitize_f32(value),
        PacketValue::U8(value) => f32::from(value),
    }
}

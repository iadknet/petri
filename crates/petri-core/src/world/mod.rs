use std::collections::{HashMap, VecDeque};

use petri_graph::{
    ActionOutputs, ComputationGraph, ControllerPalette, SensorInputs, ACTION_CONFIDENCE_COUNT,
    SLOT_COUNT_MAX, TOUCH_DIRECTION_COUNT,
};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use slotmap::{Key, SlotMap};

use crate::config::WorldConfig;
use crate::types::{
    CognitionDiagnostics, CreatureDetail, CreatureEvent, CreatureEventKind, CreatureId,
    CreatureSnapshot, CreatureStateSnapshot, IllegalActionAttempt, IllegalActionKind,
    IllegalActionReason, InventoryItem, MemoryHeadState, SelectedAction, WorldDiagnostics,
    WorldFrame, WorldSnapshot,
};

mod color;
mod food;
mod helpers;
mod paint;
mod perception;
mod snapshot;
mod spawn;
mod tick;

#[cfg(test)]
mod tests;

const EVENT_LOG_CAPACITY: usize = 8;
const ILLEGAL_LOG_CAPACITY: usize = 8;
const INITIAL_WEIGHT_MUTATION_SCALE: f32 = 0.7;
const INITIAL_STRUCTURAL_MUTATION_SCALE: f32 = 0.35;
const FOUNDER_MEMORY_REGISTER_BITS: usize = 32;
const MEMORY_REGISTER_MIN_BITS: usize = 1;
const MAX_MEMORY_REGISTER_BITS: usize = 1024;
const MEMORY_REGISTER_MUTATION_STEP_MAX_BITS: usize = 32;
const FOUNDER_SLOT_CAPACITY: usize = 1;
const SLOT_CAPACITY_MIN: usize = 1;
const SLOT_CAPACITY_MUTATION_STEP_MAX: usize = 1;
const MAX_BRUSH_HALF_EXTENT: u8 = 2;
const MOVE_LOAD_PENALTY_PER_FILLED_SLOT: f32 = 0.35;
type OffspringRequest = (
    u32,
    u32,
    f32,
    u32,
    u64,
    ComputationGraph,
    u64,
    u64,
    usize,
    Vec<u8>,
    f32,
    f32,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TouchDirection {
    SelfCell = 0,
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct CreatureView {
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaintTool {
    Food,
    Barrier,
    EraseFood,
    EraseBarrier,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PaintPoint {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaintStats {
    pub affected_cells: usize,
    pub food_set_cells: usize,
    pub food_cleared_cells: usize,
    pub barrier_set_cells: usize,
    pub barrier_cleared_cells: usize,
    pub creatures_removed: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintError {
    InvalidBrushHalfExtent { received: u8, max: u8 },
}

#[derive(Clone, Copy, Debug)]
struct Cell {
    food: f32,
    barrier: bool,
}

#[derive(Debug)]
struct Creature {
    x: u32,
    y: u32,
    energy: f32,
    age: u64,
    generation: u32,
    lineage_id: u64,
    parent_id: Option<u64>,
    controller: ComputationGraph,
    phenotype_color: [u8; 3],
    phenotype_hue: f32,
    phenotype_saturation: f32,
    memory_register: Vec<u8>,
    last_memory_head: MemoryHeadState,
    rng: SmallRng,
    events: VecDeque<CreatureEvent>,
    illegal_attempts: VecDeque<IllegalActionAttempt>,
    slot_capacity: usize,
    slots: Vec<Option<InventoryItem>>,
    last_move_blocked: bool,
    last_inputs: SensorInputs,
    last_outputs: ActionOutputs,
    cognition: CognitionDiagnostics,
}

#[derive(Clone, Copy, Debug, Default)]
struct PerceptionScan {
    food_direction: f32,
    food_distance: f32,
    creature_direction: f32,
    creature_distance: f32,
    local_density: f32,
    barrier_direction: f32,
    barrier_distance: f32,
}

fn founder_memory_register() -> Vec<u8> {
    let founder_bits =
        FOUNDER_MEMORY_REGISTER_BITS.clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    vec![0; founder_bits]
}

fn founder_slot_capacity() -> usize {
    normalize_slot_capacity(FOUNDER_SLOT_CAPACITY)
}

fn normalize_slot_capacity(slot_capacity: usize) -> usize {
    slot_capacity.clamp(SLOT_CAPACITY_MIN, SLOT_COUNT_MAX)
}

fn empty_slots(slot_capacity: usize) -> Vec<Option<InventoryItem>> {
    vec![None; normalize_slot_capacity(slot_capacity)]
}

fn normalize_slots(
    mut slots: Vec<Option<InventoryItem>>,
    slot_capacity: usize,
) -> Vec<Option<InventoryItem>> {
    let bounded_capacity = normalize_slot_capacity(slot_capacity);
    if slots.len() > bounded_capacity {
        slots.truncate(bounded_capacity);
    }
    if slots.len() < bounded_capacity {
        slots.resize(bounded_capacity, None);
    }
    slots
}

fn normalize_memory_register(mut memory_register: Vec<u8>) -> Vec<u8> {
    if memory_register.is_empty() {
        return founder_memory_register();
    }
    let bounded_len = memory_register
        .len()
        .clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    memory_register.resize(bounded_len, 0);
    memory_register
}

fn maybe_mutate_memory_register_size(
    memory_register: &mut Vec<u8>,
    mutation_rate: f32,
    rng: &mut SmallRng,
) {
    if rng.gen::<f32>() > mutation_rate.clamp(0.0, 1.0) {
        return;
    }

    let current_len = memory_register
        .len()
        .clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    memory_register.resize(current_len, 0);

    let can_grow = current_len < MAX_MEMORY_REGISTER_BITS;
    let can_shrink = current_len > MEMORY_REGISTER_MIN_BITS;
    let grow = match (can_grow, can_shrink) {
        (true, true) => rng.gen::<bool>(),
        (true, false) => true,
        (false, true) => false,
        (false, false) => return,
    };

    if grow {
        let max_delta =
            (MAX_MEMORY_REGISTER_BITS - current_len).min(MEMORY_REGISTER_MUTATION_STEP_MAX_BITS);
        let delta = rng.gen_range(1..=max_delta);
        memory_register.resize(current_len + delta, 0);
    } else {
        let max_delta =
            (current_len - MEMORY_REGISTER_MIN_BITS).min(MEMORY_REGISTER_MUTATION_STEP_MAX_BITS);
        let delta = rng.gen_range(1..=max_delta);
        memory_register.truncate(current_len - delta);
    }
}

fn maybe_mutate_slot_capacity(
    slot_capacity: usize,
    mutation_rate: f32,
    rng: &mut SmallRng,
) -> usize {
    let slot_capacity = normalize_slot_capacity(slot_capacity);
    if rng.gen::<f32>() > mutation_rate.clamp(0.0, 1.0) {
        return slot_capacity;
    }
    let can_grow = slot_capacity < SLOT_COUNT_MAX;
    let can_shrink = slot_capacity > SLOT_CAPACITY_MIN;
    let grow = match (can_grow, can_shrink) {
        (true, true) => rng.gen::<bool>(),
        (true, false) => true,
        (false, true) => false,
        (false, false) => return slot_capacity,
    };

    if grow {
        let max_delta = (SLOT_COUNT_MAX - slot_capacity).min(SLOT_CAPACITY_MUTATION_STEP_MAX);
        let delta = rng.gen_range(1..=max_delta);
        slot_capacity + delta
    } else {
        let max_delta = (slot_capacity - SLOT_CAPACITY_MIN).min(SLOT_CAPACITY_MUTATION_STEP_MAX);
        let delta = rng.gen_range(1..=max_delta);
        slot_capacity - delta
    }
}

pub struct World {
    pub config: WorldConfig,
    tick: u64,
    cells: Vec<Cell>,
    creatures: SlotMap<CreatureId, Creature>,
    creature_at: Vec<Option<CreatureId>>,
    rng: SmallRng,
    palette: ControllerPalette,
    diagnostics: WorldDiagnostics,
    lineage_tree: HashMap<u64, Vec<u64>>,
    next_lineage_id: u64,
}

impl World {
    pub fn new(config: WorldConfig, seed: u64) -> Self {
        Self::new_with_palette(config, seed, ControllerPalette::Hybrid)
    }

    pub fn new_with_palette(config: WorldConfig, seed: u64, palette: ControllerPalette) -> Self {
        let mut world = Self {
            tick: 0,
            cells: vec![
                Cell {
                    food: 0.0,
                    barrier: false,
                };
                (config.width * config.height) as usize
            ],
            creature_at: vec![None; (config.width * config.height) as usize],
            creatures: SlotMap::with_key(),
            rng: SmallRng::seed_from_u64(seed),
            config,
            palette,
            diagnostics: WorldDiagnostics::default(),
            lineage_tree: HashMap::new(),
            next_lineage_id: 1,
        };

        for _ in 0..world.config.initial_creatures {
            world.spawn_random_creature(0, None, None);
        }
        world
    }

    pub fn frame(&self) -> WorldFrame {
        let creatures = self
            .creatures
            .iter()
            .map(|(id, c)| CreatureSnapshot {
                id: id.data().as_ffi(),
                lineage_id: c.lineage_id,
                parent_id: c.parent_id,
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
                node_count: c.controller.compute_node_count() as u32,
                phenotype_color: c.phenotype_color,
            })
            .collect::<Vec<_>>();

        let total_energy: f32 = creatures.iter().map(|c| c.energy).sum();
        let average_energy = if creatures.is_empty() {
            0.0
        } else {
            total_energy / creatures.len() as f32
        };

        WorldFrame {
            tick: self.tick,
            width: self.config.width,
            height: self.config.height,
            food: self
                .cells
                .iter()
                .map(|c| helpers::quantize_food(c.food, self.config.food_max_density))
                .collect(),
            barrier_bits: helpers::pack_barrier_bits(&self.cells),
            population: creatures.len(),
            average_energy,
            creatures,
        }
    }

    pub fn diagnostics(&self) -> WorldDiagnostics {
        self.diagnostics
    }

    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    pub fn average_energy(&self) -> f32 {
        if self.creatures.is_empty() {
            return 0.0;
        }
        let total_energy: f32 = self.creatures.values().map(|c| c.energy).sum();
        total_energy / self.creatures.len() as f32
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }
    pub fn creature_views(&self) -> Vec<CreatureView> {
        self.creatures
            .iter()
            .map(|(id, c)| CreatureView {
                id: id.data().as_ffi(),
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
            })
            .collect()
    }

    pub fn creature_detail(&self, creature_id: u64) -> Option<CreatureDetail> {
        self.creatures.iter().find_map(|(id, c)| {
            let id_u64 = id.data().as_ffi();
            if id_u64 != creature_id {
                return None;
            }

            Some(CreatureDetail {
                id: id_u64,
                lineage_id: c.lineage_id,
                parent_id: c.parent_id,
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
                node_count: c.controller.compute_node_count() as u32,
                phenotype_color: c.phenotype_color,
                last_move_blocked: c.last_move_blocked,
                last_inputs: c.last_inputs,
                last_outputs: c.last_outputs,
                last_memory_head: c.last_memory_head,
                cognition: c.cognition,
                events: c.events.iter().copied().collect(),
                slot_capacity: c.slot_capacity as u8,
                slots: c.slots.clone(),
                illegal_attempts: c.illegal_attempts.iter().copied().collect(),
            })
        })
    }

    pub(super) fn selector_to_bin(value: f32, bins: usize) -> usize {
        if bins <= 1 {
            return 0;
        }
        let clamped = value.clamp(-1.0, 1.0);
        let normalized = (clamped + 1.0) * 0.5;
        let scaled = (normalized * bins as f32).floor() as usize;
        scaled.min(bins - 1)
    }

    pub(super) fn direction_from_selector(value: f32) -> TouchDirection {
        match Self::selector_to_bin(value, TOUCH_DIRECTION_COUNT) {
            0 => TouchDirection::SelfCell,
            1 => TouchDirection::North,
            2 => TouchDirection::East,
            3 => TouchDirection::South,
            _ => TouchDirection::West,
        }
    }

    pub(super) fn slot_index_from_selector(value: f32) -> usize {
        Self::selector_to_bin(value, SLOT_COUNT_MAX)
    }

    pub(super) fn touch_target(
        &self,
        x: u32,
        y: u32,
        direction: TouchDirection,
    ) -> Option<(u32, u32)> {
        match direction {
            TouchDirection::SelfCell => Some((x, y)),
            TouchDirection::North => self.offset_target(x, y, 0, -1),
            TouchDirection::East => self.offset_target(x, y, 1, 0),
            TouchDirection::South => self.offset_target(x, y, 0, 1),
            TouchDirection::West => self.offset_target(x, y, -1, 0),
        }
    }

    fn offset_target(&self, x: u32, y: u32, dx: i32, dy: i32) -> Option<(u32, u32)> {
        let raw_x = x as i32 + dx;
        let raw_y = y as i32 + dy;
        if self.config.world_wrap {
            Some((
                helpers::wrap_axis(raw_x, self.config.width),
                helpers::wrap_axis(raw_y, self.config.height),
            ))
        } else if raw_x < 0
            || raw_y < 0
            || raw_x >= self.config.width as i32
            || raw_y >= self.config.height as i32
        {
            None
        } else {
            Some((raw_x as u32, raw_y as u32))
        }
    }

    pub(super) fn touch_direction_index(direction: TouchDirection) -> usize {
        direction as usize
    }

    fn filled_slot_count(creature: &Creature) -> usize {
        creature.slots.iter().filter(|slot| slot.is_some()).count()
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.config.width + x) as usize
    }
}

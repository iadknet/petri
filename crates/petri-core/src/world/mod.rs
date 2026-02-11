use std::collections::{HashMap, VecDeque};

use petri_graph::{ActionOutputs, ComputationGraph, ControllerPalette, SensorInputs};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::{Key, SlotMap};

use crate::config::WorldConfig;
use crate::types::{
    CreatureDetail, CreatureEvent, CreatureEventKind, CreatureId, CreatureSnapshot,
    CreatureStateSnapshot, WorldDiagnostics, WorldFrame, WorldSnapshot,
};

mod food;
mod helpers;
mod perception;
mod snapshot;
mod spawn;
mod tick;

#[cfg(test)]
mod tests;

const EVENT_LOG_CAPACITY: usize = 8;
const FOOD_SENSOR_RADIUS: i32 = 12;
const INITIAL_WEIGHT_MUTATION_SCALE: f32 = 0.7;
const INITIAL_STRUCTURAL_MUTATION_SCALE: f32 = 0.35;
const FOUNDER_MEMORY_REGISTER_BITS: usize = 32;
const MEMORY_REGISTER_MIN_BITS: usize = 1;
const MAX_MEMORY_REGISTER_BITS: usize = 1024;
const MEMORY_REGISTER_MUTATION_STEP_MAX_BITS: usize = 32;
type OffspringRequest = (
    u32,
    u32,
    f32,
    u32,
    u64,
    ComputationGraph,
    u64,
    u64,
    Vec<bool>,
);

#[derive(Clone, Copy, Debug)]
pub struct CreatureView {
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
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
    memory_register: Vec<bool>,
    rng: SmallRng,
    events: VecDeque<CreatureEvent>,
    last_move_blocked: bool,
    last_inputs: SensorInputs,
    last_outputs: ActionOutputs,
}

#[derive(Clone, Copy, Debug, Default)]
struct PerceptionScan {
    food_direction: f32,
    food_distance: f32,
    creature_direction: f32,
    creature_distance: f32,
    local_density: f32,
}

fn founder_memory_register() -> Vec<bool> {
    let founder_bits =
        FOUNDER_MEMORY_REGISTER_BITS.clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    vec![false; founder_bits]
}

fn normalize_memory_register(mut memory_register: Vec<bool>) -> Vec<bool> {
    if memory_register.is_empty() {
        return founder_memory_register();
    }
    let bounded_len = memory_register
        .len()
        .clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    memory_register.resize(bounded_len, false);
    memory_register
}

fn maybe_mutate_memory_register_size(
    memory_register: &mut Vec<bool>,
    mutation_rate: f32,
    rng: &mut SmallRng,
) {
    if rng.gen::<f32>() > mutation_rate.clamp(0.0, 1.0) {
        return;
    }

    let current_len = memory_register
        .len()
        .clamp(MEMORY_REGISTER_MIN_BITS, MAX_MEMORY_REGISTER_BITS);
    memory_register.resize(current_len, false);

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
        memory_register.resize(current_len + delta, false);
    } else {
        let max_delta =
            (current_len - MEMORY_REGISTER_MIN_BITS).min(MEMORY_REGISTER_MUTATION_STEP_MAX_BITS);
        let delta = rng.gen_range(1..=max_delta);
        memory_register.truncate(current_len - delta);
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
                last_move_blocked: c.last_move_blocked,
                last_inputs: c.last_inputs,
                last_outputs: c.last_outputs,
                events: c.events.iter().copied().collect(),
            })
        })
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.config.width + x) as usize
    }
}

use std::collections::VecDeque;

use petri_graph::{ComputationGraph, ControllerPalette, MutationConfig, SensorInputs};
use rand::rngs::SmallRng;
use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use slotmap::{Key, SlotMap};

use crate::config::WorldConfig;
use crate::types::{
    CreatureEvent, CreatureEventKind, CreatureId, CreatureSnapshot, WorldDiagnostics, WorldFrame,
};

const EVENT_LOG_CAPACITY: usize = 8;
const FOOD_SENSOR_RADIUS: i32 = 12;
const INITIAL_WEIGHT_MUTATION_SCALE: f32 = 0.7;
const INITIAL_STRUCTURAL_MUTATION_SCALE: f32 = 0.35;

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
}

#[derive(Debug)]
struct Creature {
    x: u32,
    y: u32,
    energy: f32,
    age: u64,
    generation: u32,
    controller: ComputationGraph,
    rng: SmallRng,
    events: VecDeque<CreatureEvent>,
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
}

impl World {
    pub fn new(config: WorldConfig, seed: u64) -> Self {
        Self::new_with_palette(config, seed, ControllerPalette::Hybrid)
    }

    pub fn new_with_palette(config: WorldConfig, seed: u64, palette: ControllerPalette) -> Self {
        let mut world = Self {
            tick: 0,
            cells: vec![Cell { food: 0.0 }; (config.width * config.height) as usize],
            creature_at: vec![None; (config.width * config.height) as usize],
            creatures: SlotMap::with_key(),
            rng: SmallRng::seed_from_u64(seed),
            config,
            palette,
            diagnostics: WorldDiagnostics::default(),
        };

        for _ in 0..world.config.initial_creatures {
            world.spawn_random_creature(0);
        }
        world
    }

    pub fn tick(&mut self) {
        if self.config.paused {
            return;
        }

        self.tick += 1;
        self.update_food();

        let ids = self.creatures.keys().collect::<Vec<_>>();
        let mut to_remove = Vec::new();
        let mut offspring = Vec::new();

        for id in ids {
            if !self.creatures.contains_key(id) {
                continue;
            }

            let width = self.config.width;
            let height = self.config.height;
            let can_spawn_more = self.creatures.len() < self.config.max_creatures;

            let mut dead = false;
            let mut child_request: Option<(u32, u32, f32, u32, u64, ComputationGraph)> = None;
            let mut reproduce_from: Option<(u32, u32)> = None;
            let mut reproduce_intent = false;
            let (sensor_x, sensor_y) = {
                let creature = self
                    .creatures
                    .get(id)
                    .expect("id list should only contain live creatures");
                (creature.x, creature.y)
            };
            let (food_direction, food_distance) = self.nearest_food_sensor(sensor_x, sensor_y);
            let offspring_mutation_cfg = self.offspring_mutation_config();

            {
                let creature = self
                    .creatures
                    .get_mut(id)
                    .expect("id list should only contain live creatures");

                creature.age += 1;

                let compute_cost = self.config.energy_per_compute_node
                    * creature.controller.compute_node_count() as f32;
                creature.energy -= self.config.energy_per_tick_decay + compute_cost;

                let current_idx = (creature.y * width + creature.x) as usize;
                let outputs = creature.controller.evaluate(SensorInputs {
                    food_here: self.cells[current_idx].food,
                    energy: (creature.energy / self.config.energy_max).clamp(0.0, 1.0),
                    random: creature.rng.gen_range(-1.0_f32..=1.0_f32),
                    food_direction,
                    food_distance,
                });

                if outputs.eat > 0.5 {
                    let available_food = self.cells[current_idx].food;
                    if available_food > 0.0 {
                        let consumed = available_food.min(0.5);
                        self.cells[current_idx].food -= consumed;
                        creature.energy += consumed * self.config.food_energy_value;
                        creature.energy = creature.energy.min(self.config.energy_max);
                        self.diagnostics.eats += 1;
                        push_event(creature, CreatureEventKind::AteFood, self.tick);
                    }
                }

                let dx = axis_step(outputs.move_x);
                let dy = axis_step(outputs.move_y);
                if dx != 0 || dy != 0 {
                    let nx = wrap_axis(creature.x as i32 + dx, width);
                    let ny = wrap_axis(creature.y as i32 + dy, height);
                    let next_idx = (ny * width + nx) as usize;

                    if self.creature_at[next_idx].is_none() {
                        self.creature_at[current_idx] = None;
                        self.creature_at[next_idx] = Some(id);
                        creature.x = nx;
                        creature.y = ny;
                        creature.energy -= self.config.energy_per_move;
                        self.diagnostics.moves += 1;
                        push_event(creature, CreatureEventKind::Moved, self.tick);
                    }
                }

                if outputs.reproduce > 0.5
                    && can_spawn_more
                    && creature.energy >= self.config.min_reproduce_energy
                {
                    reproduce_from = Some((creature.x, creature.y));
                    reproduce_intent = true;
                }

                if creature.energy <= 0.0 {
                    dead = true;
                    self.diagnostics.deaths += 1;
                    push_event(creature, CreatureEventKind::Starved, self.tick);
                }
            }

            if !dead && reproduce_intent {
                if let Some((px, py)) = reproduce_from {
                    if let Some((cx, cy)) = self.find_empty_neighbor(px, py) {
                        if let Some(creature) = self.creatures.get_mut(id) {
                            if creature.energy >= self.config.min_reproduce_energy {
                                let inherited = (creature.energy
                                    * self.config.offspring_energy_fraction)
                                    .max(0.0);
                                creature.energy -= inherited + self.config.energy_per_reproduce;
                                let seed = creature.rng.gen::<u64>();
                                let mut child_controller = creature.controller.clone();
                                child_controller
                                    .mutate_with_config(&mut creature.rng, offspring_mutation_cfg);

                                child_request = Some((
                                    cx,
                                    cy,
                                    inherited,
                                    creature.generation + 1,
                                    seed,
                                    child_controller,
                                ));
                                self.diagnostics.reproductions += 1;
                                push_event(creature, CreatureEventKind::Reproduced, self.tick);
                                if creature.energy <= 0.0 {
                                    dead = true;
                                    self.diagnostics.deaths += 1;
                                    push_event(creature, CreatureEventKind::Starved, self.tick);
                                }
                            }
                        }
                    }
                }
            }

            if dead {
                to_remove.push(id);
            }

            if let Some(req) = child_request {
                offspring.push(req);
            }
        }

        for id in to_remove {
            if let Some(creature) = self.creatures.remove(id) {
                let idx = self.idx(creature.x, creature.y);
                self.creature_at[idx] = None;
            }
        }

        for (x, y, energy, generation, seed, controller) in offspring {
            if self.creatures.len() >= self.config.max_creatures {
                break;
            }
            let idx = self.idx(x, y);
            if self.creature_at[idx].is_some() {
                continue;
            }

            let child = Creature {
                x,
                y,
                energy: energy.min(self.config.energy_max),
                age: 0,
                generation,
                controller,
                rng: SmallRng::seed_from_u64(seed),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
            };
            let child_id = self.creatures.insert(child);
            self.creature_at[idx] = Some(child_id);
        }
    }

    pub fn frame(&self) -> WorldFrame {
        let creatures = self
            .creatures
            .iter()
            .map(|(id, c)| CreatureSnapshot {
                id: id.data().as_ffi(),
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
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
                .map(|c| quantize_food(c.food, self.config.food_max_density))
                .collect(),
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

    pub fn seed_food_density(&mut self, density: f32) {
        let total_cells = self.cells.len();
        if total_cells == 0 {
            return;
        }

        for cell in &mut self.cells {
            cell.food = 0.0;
        }

        let clamped = density.clamp(0.0, 1.0);
        let target = ((clamped * total_cells as f32).round() as usize).min(total_cells);
        if target == 0 {
            return;
        }

        let sampled = sample(&mut self.rng, total_cells, target);
        for idx in sampled.iter() {
            self.cells[idx].food = self.config.food_max_density;
        }
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

    fn spawn_random_creature(&mut self, generation: u32) -> Option<CreatureId> {
        let total_cells = self.cells.len();
        if total_cells == 0 {
            return None;
        }
        let start_idx = self.rng.gen_range(0..total_cells);

        for offset in 0..total_cells {
            let idx = (start_idx + offset) % total_cells;
            if self.creature_at[idx].is_none() {
                let x = (idx as u32) % self.config.width;
                let y = (idx as u32) / self.config.width;
                let seed = self.rng.gen::<u64>();
                let mut controller = ComputationGraph::founder(self.palette);
                let initial_mutation_cfg = self.initial_mutation_config();
                // Add slight startup diversity so founders are viable but not identical clones.
                controller.mutate_with_config(&mut self.rng, initial_mutation_cfg);
                let creature = Creature {
                    x,
                    y,
                    energy: self.config.energy_initial,
                    age: 0,
                    generation,
                    controller,
                    rng: SmallRng::seed_from_u64(seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                };
                let id = self.creatures.insert(creature);
                self.creature_at[idx] = Some(id);
                return Some(id);
            }
        }
        None
    }

    fn update_food(&mut self) {
        let total_cells = self.cells.len();
        if total_cells == 0 {
            return;
        }

        let spawn_attempts = (total_cells as f32 * self.config.food_spawn_rate).round() as usize;
        let growth_per_spawn = self.config.food_growth_rate.max(0.0);
        let max_density = self.config.food_max_density.max(0.0);

        for _ in 0..spawn_attempts {
            let idx = self.rng.gen_range(0..total_cells);
            let cell = &mut self.cells[idx];
            cell.food = (cell.food + growth_per_spawn).min(max_density);
        }
    }

    fn nearest_food_sensor(&self, x: u32, y: u32) -> (f32, f32) {
        let mut best: Option<(i32, i32, i32)> = None;

        for dy in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
            for dx in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
                let nx = wrap_axis(x as i32 + dx, self.config.width);
                let ny = wrap_axis(y as i32 + dy, self.config.height);
                let idx = self.idx(nx, ny);
                if self.cells[idx].food <= 0.0 {
                    continue;
                }

                let dist_sq = dx * dx + dy * dy;
                match best {
                    Some((best_dist_sq, _, _)) if dist_sq >= best_dist_sq => {}
                    _ => best = Some((dist_sq, dx, dy)),
                }
            }
        }

        let Some((dist_sq, dx, dy)) = best else {
            return (0.0, 1.0);
        };
        if dist_sq == 0 {
            return (0.0, 0.0);
        }

        let distance = (dist_sq as f32).sqrt() / FOOD_SENSOR_RADIUS as f32;
        let direction = (dy as f32).atan2(dx as f32) / std::f32::consts::PI;
        (direction.clamp(-1.0, 1.0), distance.clamp(0.0, 1.0))
    }

    fn initial_mutation_config(&self) -> MutationConfig {
        MutationConfig {
            weight_mutation_rate: (self.config.weight_mutation_rate
                * INITIAL_WEIGHT_MUTATION_SCALE)
                .clamp(0.0, 1.0),
            weight_mutation_magnitude: self.config.weight_mutation_magnitude.max(0.0),
            logic_node_mutation_rate: (self.config.logic_node_mutation_rate
                * INITIAL_STRUCTURAL_MUTATION_SCALE)
                .clamp(0.0, 1.0),
            structural_mutation_rate: (self.config.structural_mutation_rate
                * INITIAL_STRUCTURAL_MUTATION_SCALE)
                .clamp(0.0, 1.0),
        }
    }

    fn offspring_mutation_config(&self) -> MutationConfig {
        MutationConfig {
            weight_mutation_rate: self.config.weight_mutation_rate.clamp(0.0, 1.0),
            weight_mutation_magnitude: self.config.weight_mutation_magnitude.max(0.0),
            logic_node_mutation_rate: self.config.logic_node_mutation_rate.clamp(0.0, 1.0),
            structural_mutation_rate: self.config.structural_mutation_rate.clamp(0.0, 1.0),
        }
    }

    fn find_empty_neighbor(&self, x: u32, y: u32) -> Option<(u32, u32)> {
        for dx in -1_i32..=1 {
            for dy in -1_i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = wrap_axis(x as i32 + dx, self.config.width);
                let ny = wrap_axis(y as i32 + dy, self.config.height);
                let idx = self.idx(nx, ny);
                if self.creature_at[idx].is_none() {
                    return Some((nx, ny));
                }
            }
        }
        None
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.config.width + x) as usize
    }
}

fn axis_step(value: f32) -> i32 {
    if value > 0.25 {
        1
    } else if value < -0.25 {
        -1
    } else {
        0
    }
}

fn wrap_axis(v: i32, max: u32) -> u32 {
    let m = max as i32;
    (((v % m) + m) % m) as u32
}

fn quantize_food(food: f32, max_density: f32) -> u8 {
    let max_density = max_density.max(f32::EPSILON);
    let normalized = (food.clamp(0.0, max_density) / max_density).clamp(0.0, 1.0);
    (normalized * 255.0).round() as u8
}

fn push_event(creature: &mut Creature, kind: CreatureEventKind, tick: u64) {
    if creature.events.len() == EVENT_LOG_CAPACITY {
        creature.events.pop_front();
    }
    creature.events.push_back(CreatureEvent { kind, tick });
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use crate::ControllerPalette;

    use super::*;

    #[test]
    fn world_initializes_with_creatures_in_bounds() {
        let cfg = WorldConfig {
            width: 40,
            height: 30,
            initial_creatures: 80,
            max_creatures: 500,
            ..WorldConfig::default()
        };

        let world = World::new(cfg, 7);
        assert_eq!(world.creature_count(), 80);
        for c in world.creature_views() {
            assert!(c.x < 40);
            assert!(c.y < 30);
        }
    }

    #[test]
    fn tick_changes_creature_positions_over_time() {
        let cfg = WorldConfig {
            width: 32,
            height: 32,
            initial_creatures: 30,
            max_creatures: 300,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 3);
        let before = world
            .creature_views()
            .into_iter()
            .map(|c| (c.id, (c.x, c.y)))
            .collect::<std::collections::BTreeMap<_, _>>();

        world.tick();

        let after = world
            .creature_views()
            .into_iter()
            .map(|c| (c.id, (c.x, c.y)))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_ne!(before, after);
    }

    #[test]
    fn creatures_lose_energy_and_die() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            energy_initial: 0.05,
            energy_per_tick_decay: 0.02,
            energy_per_move: 0.02,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 9);
        for _ in 0..10 {
            world.tick();
        }
        assert_eq!(world.creature_count(), 0);
    }

    #[test]
    fn creatures_eat_food_and_gain_energy() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            food_energy_value: 1.0,
            min_reproduce_energy: 10.0,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 11);

        let (id, mut creature) = world.creatures.iter().next().unwrap();
        let idx = world.idx(creature.x, creature.y);
        world.cells[idx].food = 1.0;
        let before = creature.energy;

        world.tick();

        creature = world.creatures.get(id).unwrap();
        assert!(creature.energy > before);
    }

    #[test]
    fn creatures_can_reproduce_when_energy_is_high() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 1,
            max_creatures: 50,
            min_reproduce_energy: 0.6,
            energy_initial: 1.5,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 21);
        let (_id, parent) = world.creatures.iter().next().unwrap();
        let idx = world.idx(parent.x, parent.y);
        world.cells[idx].food = 1.0;
        assert_eq!(world.creature_count(), 1);
        world.tick();
        assert!(world.creature_count() > 1);
    }

    #[test]
    fn world_can_be_created_with_each_controller_palette() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 20,
            max_creatures: 200,
            ..WorldConfig::default()
        };

        for palette in [
            ControllerPalette::NeuralOnly,
            ControllerPalette::LogicOnly,
            ControllerPalette::Hybrid,
        ] {
            let mut world = World::new_with_palette(cfg.clone(), 99, palette);
            world.tick();
            assert!(world.creature_count() > 0);
        }
    }

    #[test]
    fn world_tracks_diagnostics_and_logic_eat_gate() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            food_energy_value: 1.0,
            min_reproduce_energy: 10.0,
            ..WorldConfig::default()
        };

        let mut world = World::new_with_palette(cfg, 123, ControllerPalette::LogicOnly);
        let initial_diag = world.diagnostics();
        assert_eq!(initial_diag.moves, 0);
        assert_eq!(initial_diag.eats, 0);
        assert_eq!(initial_diag.reproductions, 0);
        assert_eq!(initial_diag.deaths, 0);

        let (id, creature) = world.creatures.iter().next().unwrap();
        let idx = world.idx(creature.x, creature.y);
        world.cells[idx].food = 0.0;
        let before = creature.energy;

        world.tick();
        let after_low_food = world.creatures.get(id).unwrap().energy;
        assert!(after_low_food <= before);

        let creature_after = world.creatures.get(id).unwrap();
        let idx2 = world.idx(creature_after.x, creature_after.y);
        world.cells[idx2].food = 1.0;

        world.tick();
        let diag = world.diagnostics();
        assert!(diag.eats > 0);
    }

    fn controller_checksum(controller: &ComputationGraph) -> i64 {
        let node_sum = controller
            .nodes
            .iter()
            .map(|node| match node {
                petri_graph::NodeKind::Constant(v) => (*v * 1000.0) as i64,
                petri_graph::NodeKind::Threshold(t) => (*t * 1000.0) as i64,
                _ => 0,
            })
            .sum::<i64>();
        let edge_sum = controller
            .edges
            .iter()
            .map(|e| (e.weight * 1000.0) as i64)
            .sum::<i64>();
        node_sum + edge_sum
    }

    #[test]
    fn initial_population_has_founder_variation() {
        let cfg = WorldConfig {
            width: 30,
            height: 30,
            initial_creatures: 80,
            max_creatures: 200,
            ..WorldConfig::default()
        };
        let world = World::new_with_palette(cfg, 4242, ControllerPalette::Hybrid);

        let signatures = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();

        assert!(signatures.len() > 1);
    }

    #[test]
    fn food_growth_rate_zero_prevents_spawn_growth() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            max_creatures: 10,
            food_spawn_rate: 1.0,
            food_growth_rate: 0.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 1234);
        world.tick();

        let total_food: f32 = world.cells.iter().map(|c| c.food).sum();
        assert_eq!(total_food, 0.0);
    }

    #[test]
    fn nearest_food_sensor_reports_direction_and_distance() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 0,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 101);

        let source_x = 10_u32;
        let source_y = 10_u32;
        let food_x = 13_u32;
        let food_y = 10_u32;
        let food_idx = world.idx(food_x, food_y);
        world.cells[food_idx].food = world.config.food_max_density;

        let (direction, distance) = world.nearest_food_sensor(source_x, source_y);
        assert!(
            direction.abs() < 0.01,
            "expected east-facing direction, got {direction}"
        );
        assert!(distance > 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nearest_food_sensor_defaults_when_no_food_is_visible() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 0,
            ..WorldConfig::default()
        };
        let world = World::new(cfg, 202);

        let (direction, distance) = world.nearest_food_sensor(6, 6);
        assert_eq!(direction, 0.0);
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn spawn_random_creature_finds_free_cell_beyond_random_attempt_window() {
        let seed = 9001_u64;
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 0,
            max_creatures: 400,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, seed, ControllerPalette::Hybrid);

        let mut probe_rng = SmallRng::seed_from_u64(seed);
        let first_window = (0..64)
            .map(|_| {
                (
                    probe_rng.gen_range(0..world.config.width),
                    probe_rng.gen_range(0..world.config.height),
                )
            })
            .collect::<HashSet<_>>();

        let free_cell = (0..world.config.height)
            .flat_map(|y| (0..world.config.width).map(move |x| (x, y)))
            .find(|coord| !first_window.contains(coord))
            .expect("grid should contain at least one unsampled cell");

        let mut creature_seed = 1_u64;
        for y in 0..world.config.height {
            for x in 0..world.config.width {
                if (x, y) == free_cell {
                    continue;
                }
                let idx = world.idx(x, y);
                let creature = Creature {
                    x,
                    y,
                    energy: 1.0,
                    age: 0,
                    generation: 0,
                    controller: ComputationGraph::founder(ControllerPalette::Hybrid),
                    rng: SmallRng::seed_from_u64(creature_seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                };
                creature_seed += 1;
                let id = world.creatures.insert(creature);
                world.creature_at[idx] = Some(id);
            }
        }

        let spawned = world.spawn_random_creature(0);
        assert!(spawned.is_some());
        assert!(world.creature_at[world.idx(free_cell.0, free_cell.1)].is_some());
    }

    #[test]
    fn offspring_controller_is_mutated_from_parent() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 1,
            max_creatures: 8,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 52, ControllerPalette::Hybrid);

        let (parent_id, px, py) = {
            let (id, parent) = world.creatures.iter().next().unwrap();
            (id, parent.x, parent.y)
        };
        if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
            parent_mut.controller = ComputationGraph::founder(ControllerPalette::Hybrid);
        }
        let idx = world.idx(px, py);
        world.cells[idx].food = 1.0;

        world.tick();
        assert!(world.creatures.len() >= 2);

        let checksums = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();
        assert!(checksums.len() > 1);
    }

    #[test]
    fn offspring_mutation_respects_zeroed_mutation_config() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 1,
            max_creatures: 12,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            weight_mutation_rate: 0.0,
            weight_mutation_magnitude: 0.0,
            structural_mutation_rate: 0.0,
            logic_node_mutation_rate: 0.0,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 222, ControllerPalette::Hybrid);
        world.seed_food_density(1.0);

        let parent_id = world
            .creatures
            .iter()
            .next()
            .map(|(id, _)| id)
            .expect("world should have one founder");
        if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
            parent_mut.controller = ComputationGraph::founder(ControllerPalette::Hybrid);
        }

        for _ in 0..20 {
            if world.creature_count() > 1 {
                break;
            }
            world.tick();
        }
        assert!(
            world.creature_count() > 1,
            "expected reproduction with abundant food"
        );

        let checksums = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();
        assert_eq!(checksums.len(), 1);
    }

    #[test]
    fn founder_seeded_hybrid_population_survives_short_horizon() {
        let cfg = WorldConfig {
            width: 40,
            height: 40,
            initial_creatures: 80,
            max_creatures: 1000,
            food_spawn_rate: 0.1,
            food_growth_rate: 0.2,
            energy_per_compute_node: 0.002,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 77, ControllerPalette::Hybrid);
        for _ in 0..100 {
            world.tick();
        }
        assert!(world.creature_count() > 0);
    }

    #[test]
    fn seed_food_density_sets_expected_occupied_cells() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 99);
        world.seed_food_density(0.25);

        let occupied = world
            .cells
            .iter()
            .filter(|cell| (cell.food - world.config.food_max_density).abs() < f32::EPSILON)
            .count();
        assert_eq!(occupied, 25);
        assert_eq!(
            world.cells.len() - occupied,
            world.cells.iter().filter(|cell| cell.food == 0.0).count()
        );
    }
}

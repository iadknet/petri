use std::collections::{HashMap, VecDeque};

use petri_graph::{ComputationGraph, ControllerPalette, MutationConfig, SensorInputs};
use rand::rngs::SmallRng;
use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use slotmap::{Key, SlotMap};

use crate::config::WorldConfig;
use crate::types::{
    CreatureEvent, CreatureEventKind, CreatureId, CreatureSnapshot, CreatureStateSnapshot,
    WorldDiagnostics, WorldFrame, WorldSnapshot,
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
    lineage_id: u64,
    parent_id: Option<u64>,
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
            cells: vec![Cell { food: 0.0 }; (config.width * config.height) as usize],
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
            let mut child_request: Option<(u32, u32, f32, u32, u64, ComputationGraph, u64, u64)> =
                None;
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
                    let nx = map_axis(creature.x as i32 + dx, width, self.config.world_wrap);
                    let ny = map_axis(creature.y as i32 + dy, height, self.config.world_wrap);
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
                                    creature.lineage_id,
                                    id.data().as_ffi(),
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

        for (x, y, energy, generation, seed, controller, lineage_id, parent_id) in offspring {
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
                lineage_id,
                parent_id: Some(parent_id),
                controller,
                rng: SmallRng::seed_from_u64(seed),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
            };
            let child_id = self.creatures.insert(child);
            self.creature_at[idx] = Some(child_id);
            let child_id_u64 = child_id.data().as_ffi();
            self.lineage_tree
                .entry(parent_id)
                .or_default()
                .push(child_id_u64);
            self.lineage_tree.entry(child_id_u64).or_default();
        }
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

    pub fn snapshot(&self) -> WorldSnapshot {
        let creatures = self
            .creatures
            .iter()
            .map(|(id, c)| CreatureStateSnapshot {
                id: id.data().as_ffi(),
                lineage_id: c.lineage_id,
                parent_id: c.parent_id,
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
                controller: c.controller.clone(),
            })
            .collect::<Vec<_>>();

        WorldSnapshot {
            tick: self.tick,
            config: self.config.clone(),
            palette: self.palette,
            cells_food: self.cells.iter().map(|cell| cell.food).collect(),
            creatures,
            diagnostics: self.diagnostics,
            lineage_tree: self.lineage_tree.clone(),
            next_lineage_id: self.next_lineage_id,
        }
    }

    pub fn from_snapshot(snapshot: WorldSnapshot) -> Self {
        let total_cells = (snapshot.config.width * snapshot.config.height) as usize;
        let mut world = Self {
            config: snapshot.config,
            tick: snapshot.tick,
            cells: vec![Cell { food: 0.0 }; total_cells],
            creature_at: vec![None; total_cells],
            creatures: SlotMap::with_key(),
            rng: SmallRng::seed_from_u64(snapshot.tick ^ 0xA11C_E5EED_u64),
            palette: snapshot.palette,
            diagnostics: snapshot.diagnostics,
            lineage_tree: HashMap::new(),
            next_lineage_id: snapshot.next_lineage_id.max(1),
        };

        for (idx, food) in snapshot.cells_food.into_iter().enumerate().take(total_cells) {
            world.cells[idx].food = food.max(0.0);
        }

        let mut id_map: HashMap<u64, CreatureId> = HashMap::new();
        let mut pending_parent: Vec<(CreatureId, Option<u64>)> = Vec::new();

        for creature in snapshot.creatures {
            let old_id = creature.id;
            let x = creature.x.min(world.config.width.saturating_sub(1));
            let y = creature.y.min(world.config.height.saturating_sub(1));
            let new_creature = Creature {
                x,
                y,
                energy: creature.energy,
                age: creature.age,
                generation: creature.generation,
                lineage_id: creature.lineage_id,
                parent_id: creature.parent_id,
                controller: creature.controller,
                rng: SmallRng::seed_from_u64(old_id ^ snapshot.tick.rotate_left(13)),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
            };

            let new_id = world.creatures.insert(new_creature);
            id_map.insert(old_id, new_id);
            pending_parent.push((new_id, creature.parent_id));
            let idx = world.idx(x, y);
            if world.creature_at[idx].is_none() {
                world.creature_at[idx] = Some(new_id);
            }
        }

        for (new_id, parent_old) in pending_parent {
            if let Some(creature) = world.creatures.get_mut(new_id) {
                creature.parent_id =
                    parent_old.and_then(|old| id_map.get(&old).map(|mapped| mapped.data().as_ffi()));
            }
        }

        for (old_parent, old_children) in snapshot.lineage_tree {
            let Some(parent) = id_map.get(&old_parent) else {
                continue;
            };
            let parent_id = parent.data().as_ffi();
            let mapped_children = old_children
                .iter()
                .filter_map(|child| id_map.get(child).map(|mapped| mapped.data().as_ffi()))
                .collect::<Vec<_>>();
            world.lineage_tree.insert(parent_id, mapped_children);
        }

        for (id, creature) in world.creatures.iter() {
            let id_u64 = id.data().as_ffi();
            world.lineage_tree.entry(id_u64).or_default();
            if let Some(parent_id) = creature.parent_id {
                let children = world.lineage_tree.entry(parent_id).or_default();
                if !children.contains(&id_u64) {
                    children.push(id_u64);
                }
            }
        }

        let max_lineage = world
            .creatures
            .values()
            .map(|creature| creature.lineage_id)
            .max()
            .unwrap_or(0);
        world.next_lineage_id = world
            .next_lineage_id
            .max(max_lineage.saturating_add(1));
        world
    }

    fn spawn_random_creature(
        &mut self,
        generation: u32,
        lineage_id: Option<u64>,
        parent_id: Option<u64>,
    ) -> Option<CreatureId> {
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
                let lineage_id = lineage_id.unwrap_or_else(|| {
                    let next = self.next_lineage_id;
                    self.next_lineage_id += 1;
                    next
                });
                let creature = Creature {
                    x,
                    y,
                    energy: self.config.energy_initial,
                    age: 0,
                    generation,
                    lineage_id,
                    parent_id,
                    controller,
                    rng: SmallRng::seed_from_u64(seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                };
                let id = self.creatures.insert(creature);
                self.creature_at[idx] = Some(id);
                let id_u64 = id.data().as_ffi();
                if let Some(parent_id) = parent_id {
                    self.lineage_tree.entry(parent_id).or_default().push(id_u64);
                }
                self.lineage_tree.entry(id_u64).or_default();
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
        let width = self.config.width as i32;
        let height = self.config.height as i32;

        for dy in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
            for dx in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
                let raw_x = x as i32 + dx;
                let raw_y = y as i32 + dy;
                let Some((nx, ny)) = (if self.config.world_wrap {
                    Some((
                        wrap_axis(raw_x, self.config.width),
                        wrap_axis(raw_y, self.config.height),
                    ))
                } else if raw_x < 0 || raw_x >= width || raw_y < 0 || raw_y >= height {
                    None
                } else {
                    Some((raw_x as u32, raw_y as u32))
                }) else {
                    continue;
                };
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
        let width = self.config.width as i32;
        let height = self.config.height as i32;
        for dx in -1_i32..=1 {
            for dy in -1_i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let raw_x = x as i32 + dx;
                let raw_y = y as i32 + dy;
                let Some((nx, ny)) = (if self.config.world_wrap {
                    Some((
                        wrap_axis(raw_x, self.config.width),
                        wrap_axis(raw_y, self.config.height),
                    ))
                } else if raw_x < 0 || raw_x >= width || raw_y < 0 || raw_y >= height {
                    None
                } else {
                    Some((raw_x as u32, raw_y as u32))
                }) else {
                    continue;
                };
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

fn map_axis(v: i32, max: u32, wrap: bool) -> u32 {
    if max == 0 {
        return 0;
    }
    if wrap {
        wrap_axis(v, max)
    } else {
        v.clamp(0, max as i32 - 1) as u32
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
    use petri_graph::{Edge, NodeKind};

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

    fn move_right_controller() -> ComputationGraph {
        ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![
                NodeKind::Constant(1.0), // 0
                NodeKind::OutputMoveX,   // 1
                NodeKind::OutputMoveY,   // 2
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            }],
        }
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
                    lineage_id: creature_seed,
                    parent_id: None,
                    controller: ComputationGraph::founder(ControllerPalette::Hybrid),
                    rng: SmallRng::seed_from_u64(creature_seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                };
                creature_seed += 1;
                let id = world.creatures.insert(creature);
                world.creature_at[idx] = Some(id);
            }
        }

        let spawned = world.spawn_random_creature(0, None, None);
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
    fn founders_have_unique_lineage_ids() {
        let cfg = WorldConfig {
            width: 30,
            height: 30,
            initial_creatures: 40,
            max_creatures: 200,
            ..WorldConfig::default()
        };
        let world = World::new(cfg, 5150);

        let lineage_ids = world
            .creatures
            .values()
            .map(|creature| creature.lineage_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(lineage_ids.len(), world.creature_count());
    }

    #[test]
    fn offspring_inherits_lineage_and_records_parent_link() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 1,
            max_creatures: 12,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 818, ControllerPalette::Hybrid);
        world.seed_food_density(1.0);

        let parent_id = world
            .creatures
            .iter()
            .next()
            .map(|(id, _)| id)
            .expect("expected one founder");
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
            "expected at least one offspring"
        );

        let frame = world.frame();
        let child = frame
            .creatures
            .iter()
            .find(|c| c.parent_id.is_some())
            .expect("expected offspring snapshot with parent id");
        let parent = frame
            .creatures
            .iter()
            .find(|c| c.id == child.parent_id.expect("parent id should be present"))
            .expect("parent should still be visible in frame");
        assert_eq!(child.lineage_id, parent.lineage_id);
        assert_eq!(child.generation, parent.generation + 1);

        let children = world
            .lineage_tree
            .get(&parent.id)
            .expect("lineage tree should track parent-child relation");
        assert!(children.contains(&child.id));
    }

    #[test]
    fn world_wrap_true_wraps_movement_across_edge() {
        let cfg = WorldConfig {
            width: 6,
            height: 6,
            initial_creatures: 1,
            world_wrap: true,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 404, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);

        world.tick();

        let creature = world.creatures.get(id).expect("creature should remain alive");
        assert_eq!(creature.y, edge_y);
        assert_eq!(creature.x, 0);
    }

    #[test]
    fn world_wrap_false_keeps_movement_in_bounds() {
        let cfg = WorldConfig {
            width: 6,
            height: 6,
            initial_creatures: 1,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 505, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);

        world.tick();

        let creature = world.creatures.get(id).expect("creature should remain alive");
        assert_eq!(creature.y, edge_y);
        assert_eq!(creature.x, edge_x);
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
    fn frame_creatures_include_controller_node_count() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 1,
            ..WorldConfig::default()
        };
        let world = World::new_with_palette(cfg, 919, ControllerPalette::Hybrid);

        let frame = world.frame();
        assert_eq!(frame.creatures.len(), 1);
        assert!(frame.creatures[0].node_count > 0);
    }

    #[test]
    fn world_snapshot_round_trip_preserves_core_state() {
        let cfg = WorldConfig {
            width: 32,
            height: 24,
            initial_creatures: 30,
            max_creatures: 300,
            food_spawn_rate: 0.12,
            food_growth_rate: 0.18,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 2026, ControllerPalette::Hybrid);
        world.seed_food_density(0.2);
        for _ in 0..12 {
            world.tick();
        }

        let snapshot = world.snapshot();
        let restored = World::from_snapshot(snapshot);

        assert_eq!(restored.tick_count(), world.tick_count());
        assert_eq!(restored.config.width, world.config.width);
        assert_eq!(restored.config.height, world.config.height);
        assert_eq!(restored.config.initial_creatures, world.config.initial_creatures);
        assert_eq!(restored.config.max_creatures, world.config.max_creatures);
        assert_eq!(restored.config.food_spawn_rate, world.config.food_spawn_rate);
        assert_eq!(restored.config.food_growth_rate, world.config.food_growth_rate);
        assert_eq!(restored.frame().food, world.frame().food);
        assert_eq!(restored.creature_count(), world.creature_count());
        assert_eq!(restored.diagnostics().moves, world.diagnostics().moves);
        assert_eq!(restored.diagnostics().eats, world.diagnostics().eats);
        assert_eq!(restored.diagnostics().reproductions, world.diagnostics().reproductions);
        assert_eq!(restored.diagnostics().deaths, world.diagnostics().deaths);

        let before_links = world.lineage_tree.values().map(Vec::len).sum::<usize>();
        let after_links = restored.lineage_tree.values().map(Vec::len).sum::<usize>();
        assert_eq!(after_links, before_links);
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

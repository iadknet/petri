use std::collections::VecDeque;

use petri_graph::{ComputationGraph, ControllerPalette, SensorInputs};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::{Key, SlotMap};

use crate::config::WorldConfig;
use crate::types::{
    CreatureEvent, CreatureEventKind, CreatureId, CreatureSnapshot, WorldDiagnostics, WorldFrame,
};

const EVENT_LOG_CAPACITY: usize = 8;

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

            {
                let creature = self
                    .creatures
                    .get_mut(id)
                    .expect("id list should only contain live creatures");

                creature.age += 1;

                let compute_cost =
                    self.config.energy_per_compute_node * creature.controller.nodes.len() as f32;
                creature.energy -= self.config.energy_per_tick_decay + compute_cost;

                let current_idx = (creature.y * width + creature.x) as usize;
                let outputs = creature.controller.evaluate(SensorInputs {
                    food_here: self.cells[current_idx].food,
                    energy: (creature.energy / self.config.energy_max).clamp(0.0, 1.0),
                    random: creature.rng.gen_range(-1.0_f32..=1.0_f32),
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
                                child_request = Some((
                                    cx,
                                    cy,
                                    inherited,
                                    creature.generation + 1,
                                    seed,
                                    creature.controller.clone(),
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
                id: id.data().as_ffi() as u64,
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
            food: self.cells.iter().map(|c| c.food).collect(),
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

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn creature_views(&self) -> Vec<CreatureView> {
        self.creatures
            .iter()
            .map(|(id, c)| CreatureView {
                id: id.data().as_ffi() as u64,
                x: c.x,
                y: c.y,
                energy: c.energy,
                age: c.age,
                generation: c.generation,
            })
            .collect()
    }

    fn spawn_random_creature(&mut self, generation: u32) -> Option<CreatureId> {
        for _ in 0..64 {
            let x = self.rng.gen_range(0..self.config.width);
            let y = self.rng.gen_range(0..self.config.height);
            let idx = self.idx(x, y);
            if self.creature_at[idx].is_none() {
                let seed = self.rng.gen::<u64>();
                let creature = Creature {
                    x,
                    y,
                    energy: self.config.energy_initial,
                    age: 0,
                    generation,
                    controller: ComputationGraph::from_palette(self.palette),
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
        let spawn_attempts = (total_cells as f32 * self.config.food_spawn_rate).round() as usize;

        for _ in 0..spawn_attempts {
            let idx = self.rng.gen_range(0..total_cells);
            let cell = &mut self.cells[idx];
            cell.food = (cell.food + self.config.food_growth_rate.max(0.05))
                .min(self.config.food_max_density);
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

fn push_event(creature: &mut Creature, kind: CreatureEventKind, tick: u64) {
    if creature.events.len() == EVENT_LOG_CAPACITY {
        creature.events.pop_front();
    }
    creature.events.push_back(CreatureEvent { kind, tick });
}

#[cfg(test)]
mod tests {
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
            energy_initial: 1.2,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 21);
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
        world.cells[idx].food = 0.1;
        let before = creature.energy;

        world.tick();
        let after_low_food = world.creatures.get(id).unwrap().energy;
        assert!(after_low_food <= before);

        let creature_after = world.creatures.get(id).unwrap();
        let idx2 = world.idx(creature_after.x, creature_after.y);
        world.cells[idx2].food = 1.0;

        world.tick();
        let diag = world.diagnostics();
        assert!(diag.moves > 0);
        assert!(diag.eats > 0);
    }
}

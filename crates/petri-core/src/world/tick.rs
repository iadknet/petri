use std::collections::VecDeque;

use rand::{Rng, SeedableRng};
use slotmap::Key;

use super::helpers::{axis_step, map_axis, push_event};
use super::*;

impl World {
    pub fn tick(&mut self) {
        if self.config.paused {
            return;
        }

        self.tick += 1;
        self.update_food();

        let ids = self.creatures.keys().collect::<Vec<_>>();
        let mut to_remove = Vec::new();
        let mut offspring = Vec::new();
        let memory_mutation_rate = self.config.structural_mutation_rate.clamp(0.0, 1.0);

        for id in ids {
            if !self.creatures.contains_key(id) {
                continue;
            }

            let width = self.config.width;
            let height = self.config.height;
            let can_spawn_more = self.creatures.len() < self.config.max_creatures;

            let mut dead = false;
            let mut child_request: Option<OffspringRequest> = None;
            let mut reproduce_from: Option<(u32, u32)> = None;
            let mut reproduce_intent = false;
            let (sensor_x, sensor_y, move_blocked_last_tick) = {
                let creature = self
                    .creatures
                    .get(id)
                    .expect("id list should only contain live creatures");
                (creature.x, creature.y, creature.last_move_blocked)
            };
            let perception = self.scan_perception(sensor_x, sensor_y, Some(id));
            let offspring_mutation_cfg = self.offspring_mutation_config();

            {
                let creature = self
                    .creatures
                    .get_mut(id)
                    .expect("id list should only contain live creatures");

                creature.age += 1;

                if creature.memory_register.is_empty() {
                    creature.memory_register.push(false);
                }
                let memory_idx =
                    creature.age.saturating_sub(1) as usize % creature.memory_register.len();
                let memory_read = if creature.memory_register[memory_idx] {
                    1.0
                } else {
                    0.0
                };

                let compute_cost = self.config.energy_per_compute_node
                    * creature.controller.compute_node_count() as f32;
                creature.energy -= self.config.energy_per_tick_decay + compute_cost;

                let current_idx = (creature.y * width + creature.x) as usize;
                let inputs = SensorInputs {
                    food_here: self.cells[current_idx].food,
                    energy: (creature.energy / self.config.energy_max).clamp(0.0, 1.0),
                    random: creature.rng.gen_range(-1.0_f32..=1.0_f32),
                    food_direction: perception.food_direction,
                    food_distance: perception.food_distance,
                    creature_direction: perception.creature_direction,
                    creature_distance: perception.creature_distance,
                    local_density: perception.local_density,
                    move_blocked_last_tick: if move_blocked_last_tick { 1.0 } else { 0.0 },
                    memory_read,
                };
                creature.last_inputs = inputs;
                let outputs = creature.controller.evaluate(inputs);
                creature.last_outputs = outputs;
                creature.memory_register[memory_idx] = outputs.memory_write > 0.5;

                if outputs.eat > 0.5 {
                    let available_food = self.cells[current_idx].food;
                    if available_food > 0.0 {
                        let consumed = available_food;
                        self.cells[current_idx].food -= consumed;
                        creature.energy += consumed * self.config.food_energy_value;
                        creature.energy = creature.energy.min(self.config.energy_max);
                        self.diagnostics.eats += 1;
                        push_event(creature, CreatureEventKind::AteFood, self.tick);
                    }
                }

                let dx = axis_step(outputs.move_x);
                let dy = axis_step(outputs.move_y);
                let mut move_attempted = false;
                let mut move_succeeded = false;
                if dx != 0 || dy != 0 {
                    move_attempted = true;
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
                        move_succeeded = true;
                    }
                }
                creature.last_move_blocked = move_attempted && !move_succeeded;

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
                                let mut child_memory_register = creature.memory_register.clone();
                                maybe_mutate_memory_register_size(
                                    &mut child_memory_register,
                                    memory_mutation_rate,
                                    &mut creature.rng,
                                );
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
                                    child_memory_register,
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

        for (x, y, energy, generation, seed, controller, lineage_id, parent_id, memory_register) in
            offspring
        {
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
                memory_register,
                rng: SmallRng::seed_from_u64(seed),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                last_move_blocked: false,
                last_inputs: SensorInputs::default(),
                last_outputs: ActionOutputs::default(),
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
}

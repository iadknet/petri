use std::collections::VecDeque;

use rand::{Rng, SeedableRng};
use slotmap::Key;

use super::helpers::{axis_step, map_axis, push_event, push_illegal_action, wrap_axis};
use super::*;

fn apply_illegal_action_penalty(
    creature: &mut Creature,
    diagnostics: &mut WorldDiagnostics,
    illegal_action_energy_penalty: f32,
    action: IllegalActionKind,
    reason: IllegalActionReason,
    tick: u64,
) {
    creature.energy -= illegal_action_energy_penalty;
    diagnostics.illegal_actions += 1;
    push_illegal_action(creature, action, reason, tick);
}

fn direction_target(
    x: u32,
    y: u32,
    direction: TouchDirection,
    width: u32,
    height: u32,
    world_wrap: bool,
) -> Option<(u32, u32)> {
    let (dx, dy) = match direction {
        TouchDirection::SelfCell => return Some((x, y)),
        TouchDirection::North => (0, -1),
        TouchDirection::East => (1, 0),
        TouchDirection::South => (0, 1),
        TouchDirection::West => (-1, 0),
    };
    let raw_x = x as i32 + dx;
    let raw_y = y as i32 + dy;
    if world_wrap {
        Some((wrap_axis(raw_x, width), wrap_axis(raw_y, height)))
    } else if raw_x < 0 || raw_y < 0 || raw_x >= width as i32 || raw_y >= height as i32 {
        None
    } else {
        Some((raw_x as u32, raw_y as u32))
    }
}

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
        let structural_mutation_rate = self.config.structural_mutation_rate.clamp(0.0, 1.0);

        for id in ids {
            if !self.creatures.contains_key(id) {
                continue;
            }

            let width = self.config.width;
            let height = self.config.height;
            let world_wrap = self.config.world_wrap;
            let can_spawn_more = self.creatures.len() < self.config.max_creatures;

            let mut dead = false;
            let mut child_request: Option<OffspringRequest> = None;
            let mut reproduce_from: Option<(u32, u32)> = None;
            let mut reproduce_intent = false;

            let (sensor_x, sensor_y, move_blocked_last_tick, slot_capacity, slots_snapshot) = {
                let creature = self
                    .creatures
                    .get(id)
                    .expect("id list should only contain live creatures");
                (
                    creature.x,
                    creature.y,
                    creature.last_move_blocked,
                    creature.slot_capacity,
                    creature.slots.clone(),
                )
            };

            let perception = self.scan_perception(sensor_x, sensor_y, Some(id));
            let offspring_mutation_cfg = self.offspring_mutation_config();

            let mut touch_exists = [0.0; TOUCH_DIRECTION_COUNT];
            let mut touch_food_value = [0.0; TOUCH_DIRECTION_COUNT];
            let mut touch_has_barrier = [0.0; TOUCH_DIRECTION_COUNT];
            let mut touch_occupied = [0.0; TOUCH_DIRECTION_COUNT];
            for direction in [
                TouchDirection::SelfCell,
                TouchDirection::North,
                TouchDirection::East,
                TouchDirection::South,
                TouchDirection::West,
            ] {
                let dir_idx = Self::touch_direction_index(direction);
                if let Some((tx, ty)) =
                    direction_target(sensor_x, sensor_y, direction, width, height, world_wrap)
                {
                    let idx = self.idx(tx, ty);
                    touch_exists[dir_idx] = 1.0;
                    touch_food_value[dir_idx] = self.cells[idx].food;
                    touch_has_barrier[dir_idx] = if self.cells[idx].barrier { 1.0 } else { 0.0 };
                    let occupied = if direction == TouchDirection::SelfCell {
                        true
                    } else {
                        self.creature_at[idx].is_some()
                    };
                    touch_occupied[dir_idx] = if occupied { 1.0 } else { 0.0 };
                }
            }

            let mut slot_exists = [0.0; SLOT_COUNT_MAX];
            let mut slot_is_empty = [0.0; SLOT_COUNT_MAX];
            let mut slot_is_barrier = [0.0; SLOT_COUNT_MAX];
            let mut slot_food_value = [0.0; SLOT_COUNT_MAX];
            for slot_idx in 0..SLOT_COUNT_MAX {
                if slot_idx < slot_capacity {
                    slot_exists[slot_idx] = 1.0;
                    match slots_snapshot.get(slot_idx).and_then(|slot| slot.as_ref()) {
                        None => slot_is_empty[slot_idx] = 1.0,
                        Some(InventoryItem::Barrier) => slot_is_barrier[slot_idx] = 1.0,
                        Some(InventoryItem::Food(value)) => slot_food_value[slot_idx] = *value,
                    }
                }
            }

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
                    barrier_direction: perception.barrier_direction,
                    barrier_distance: perception.barrier_distance,
                    move_blocked_last_tick: if move_blocked_last_tick { 1.0 } else { 0.0 },
                    memory_read,
                    touch_exists,
                    touch_food_value,
                    touch_has_barrier,
                    touch_occupied,
                    slot_exists,
                    slot_is_empty,
                    slot_is_barrier,
                    slot_food_value,
                };
                creature.last_inputs = inputs;

                let outputs = creature.controller.evaluate(inputs);
                creature.last_outputs = outputs;
                creature.memory_register[memory_idx] = outputs.memory_write > 0.5;

                let selected_inventory_action =
                    if outputs.inventory_pickup > 0.5 || outputs.inventory_put > 0.5 {
                        if outputs.inventory_pickup >= outputs.inventory_put
                            && outputs.inventory_pickup > 0.5
                        {
                            Some(IllegalActionKind::InventoryPickup)
                        } else if outputs.inventory_put > 0.5 {
                            Some(IllegalActionKind::InventoryPut)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                let selected_slot_idx =
                    Self::slot_index_from_selector(outputs.inventory_slot_select);
                let selected_direction =
                    Self::direction_from_selector(outputs.inventory_direction_select);

                if selected_inventory_action == Some(IllegalActionKind::InventoryPickup) {
                    creature.energy -= self.config.energy_per_inventory_attempt;

                    let failure = if selected_slot_idx >= creature.slot_capacity
                        || selected_slot_idx >= creature.slots.len()
                    {
                        Some(IllegalActionReason::SlotMissing)
                    } else if creature.slots[selected_slot_idx].is_some() {
                        Some(IllegalActionReason::SlotFull)
                    } else {
                        match direction_target(
                            creature.x,
                            creature.y,
                            selected_direction,
                            width,
                            height,
                            world_wrap,
                        ) {
                            None => Some(IllegalActionReason::TargetOutOfBounds),
                            Some((target_x, target_y)) => {
                                if selected_direction != TouchDirection::SelfCell {
                                    let target_idx = (target_y * width + target_x) as usize;
                                    if self.creature_at[target_idx].is_some() {
                                        Some(IllegalActionReason::TargetOccupied)
                                    } else {
                                        let target_idx = (target_y * width + target_x) as usize;
                                        if self.cells[target_idx].barrier {
                                            self.cells[target_idx].barrier = false;
                                            creature.slots[selected_slot_idx] =
                                                Some(InventoryItem::Barrier);
                                            None
                                        } else if self.cells[target_idx].food > 0.0 {
                                            let food_value = self.cells[target_idx].food;
                                            self.cells[target_idx].food = 0.0;
                                            creature.slots[selected_slot_idx] =
                                                Some(InventoryItem::Food(food_value));
                                            None
                                        } else {
                                            Some(IllegalActionReason::NoPickupableMaterial)
                                        }
                                    }
                                } else {
                                    let target_idx = (target_y * width + target_x) as usize;
                                    if self.cells[target_idx].barrier {
                                        self.cells[target_idx].barrier = false;
                                        creature.slots[selected_slot_idx] =
                                            Some(InventoryItem::Barrier);
                                        None
                                    } else if self.cells[target_idx].food > 0.0 {
                                        let food_value = self.cells[target_idx].food;
                                        self.cells[target_idx].food = 0.0;
                                        creature.slots[selected_slot_idx] =
                                            Some(InventoryItem::Food(food_value));
                                        None
                                    } else {
                                        Some(IllegalActionReason::NoPickupableMaterial)
                                    }
                                }
                            }
                        }
                    };

                    if let Some(reason) = failure {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::InventoryPickup,
                            reason,
                            self.tick,
                        );
                    }
                }

                if outputs.eat > 0.5 {
                    let available_food = self.cells[current_idx].food;
                    if available_food > 0.0 {
                        let consumed = available_food;
                        self.cells[current_idx].food =
                            (self.cells[current_idx].food - consumed).max(0.0);
                        creature.energy += consumed * self.config.food_energy_value;
                        creature.energy = creature.energy.min(self.config.energy_max);
                        self.diagnostics.eats += 1;
                        push_event(creature, CreatureEventKind::AteFood, self.tick);
                    } else {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::Eat,
                            IllegalActionReason::EatNoFood,
                            self.tick,
                        );
                    }
                }

                let dx = axis_step(outputs.move_x);
                let dy = axis_step(outputs.move_y);
                let mut move_attempted = false;
                let mut move_succeeded = false;
                if dx != 0 || dy != 0 {
                    move_attempted = true;
                    creature.energy -= self.config.energy_per_move
                        * (1.0
                            + Self::filled_slot_count(creature) as f32
                                * MOVE_LOAD_PENALTY_PER_FILLED_SLOT);

                    let raw_x = creature.x as i32 + dx;
                    let raw_y = creature.y as i32 + dy;
                    if !world_wrap
                        && (raw_x < 0
                            || raw_y < 0
                            || raw_x >= width as i32
                            || raw_y >= height as i32)
                    {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::Move,
                            IllegalActionReason::MoveOutOfBounds,
                            self.tick,
                        );
                    } else {
                        let nx = map_axis(raw_x, width, world_wrap);
                        let ny = map_axis(raw_y, height, world_wrap);
                        let next_idx = (ny * width + nx) as usize;

                        if self.creature_at[next_idx].is_none() && !self.cells[next_idx].barrier {
                            self.creature_at[current_idx] = None;
                            self.creature_at[next_idx] = Some(id);
                            creature.x = nx;
                            creature.y = ny;
                            self.diagnostics.moves += 1;
                            push_event(creature, CreatureEventKind::Moved, self.tick);
                            move_succeeded = true;
                        } else {
                            apply_illegal_action_penalty(
                                creature,
                                &mut self.diagnostics,
                                self.config.illegal_action_energy_penalty,
                                IllegalActionKind::Move,
                                IllegalActionReason::MoveBlocked,
                                self.tick,
                            );
                        }
                    }
                }
                creature.last_move_blocked = move_attempted && !move_succeeded;

                if selected_inventory_action == Some(IllegalActionKind::InventoryPut) {
                    creature.energy -= self.config.energy_per_inventory_attempt;

                    let failure = if selected_slot_idx >= creature.slot_capacity
                        || selected_slot_idx >= creature.slots.len()
                    {
                        Some(IllegalActionReason::SlotMissing)
                    } else if creature.slots[selected_slot_idx].is_none() {
                        Some(IllegalActionReason::SlotEmpty)
                    } else {
                        match direction_target(
                            creature.x,
                            creature.y,
                            selected_direction,
                            width,
                            height,
                            world_wrap,
                        ) {
                            None => Some(IllegalActionReason::TargetOutOfBounds),
                            Some((target_x, target_y)) => {
                                let target_idx = (target_y * width + target_x) as usize;
                                let item = creature.slots[selected_slot_idx]
                                    .clone()
                                    .expect("slot presence checked above");

                                match item {
                                    InventoryItem::Food(food_value) => {
                                        if self.cells[target_idx].barrier {
                                            Some(IllegalActionReason::TargetHasBarrier)
                                        } else if self.cells[target_idx].food + food_value
                                            > self.config.food_max_density + f32::EPSILON
                                        {
                                            Some(IllegalActionReason::FoodOverflow)
                                        } else {
                                            self.cells[target_idx].food += food_value;
                                            creature.slots[selected_slot_idx] = None;
                                            None
                                        }
                                    }
                                    InventoryItem::Barrier => {
                                        if self.creature_at[target_idx].is_some() {
                                            Some(IllegalActionReason::TargetOccupied)
                                        } else if self.cells[target_idx].barrier {
                                            Some(IllegalActionReason::TargetHasBarrier)
                                        } else {
                                            self.cells[target_idx].food = 0.0;
                                            self.cells[target_idx].barrier = true;
                                            creature.slots[selected_slot_idx] = None;
                                            None
                                        }
                                    }
                                }
                            }
                        }
                    };

                    if let Some(reason) = failure {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::InventoryPut,
                            reason,
                            self.tick,
                        );
                    }
                }

                if outputs.reproduce > 0.5 {
                    creature.energy -= self.config.energy_per_reproduce;
                    if !can_spawn_more {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::Reproduce,
                            IllegalActionReason::ReproduceMaxCreatures,
                            self.tick,
                        );
                    } else if creature.energy < self.config.min_reproduce_energy {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::Reproduce,
                            IllegalActionReason::ReproduceLowEnergy,
                            self.tick,
                        );
                    } else {
                        reproduce_from = Some((creature.x, creature.y));
                        reproduce_intent = true;
                    }
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
                                creature.energy -= inherited;
                                let seed = creature.rng.gen::<u64>();
                                let mut child_controller = creature.controller.clone();
                                let mut child_memory_register = creature.memory_register.clone();
                                maybe_mutate_memory_register_size(
                                    &mut child_memory_register,
                                    structural_mutation_rate,
                                    &mut creature.rng,
                                );
                                let child_slot_capacity = maybe_mutate_slot_capacity(
                                    creature.slot_capacity,
                                    structural_mutation_rate,
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
                                    child_slot_capacity,
                                    child_memory_register,
                                ));
                                self.diagnostics.reproductions += 1;
                                push_event(creature, CreatureEventKind::Reproduced, self.tick);
                                if creature.energy <= 0.0 {
                                    dead = true;
                                    self.diagnostics.deaths += 1;
                                    push_event(creature, CreatureEventKind::Starved, self.tick);
                                }
                            } else {
                                apply_illegal_action_penalty(
                                    creature,
                                    &mut self.diagnostics,
                                    self.config.illegal_action_energy_penalty,
                                    IllegalActionKind::Reproduce,
                                    IllegalActionReason::ReproduceLowEnergy,
                                    self.tick,
                                );
                                if creature.energy <= 0.0 {
                                    dead = true;
                                    self.diagnostics.deaths += 1;
                                    push_event(creature, CreatureEventKind::Starved, self.tick);
                                }
                            }
                        }
                    } else if let Some(creature) = self.creatures.get_mut(id) {
                        apply_illegal_action_penalty(
                            creature,
                            &mut self.diagnostics,
                            self.config.illegal_action_energy_penalty,
                            IllegalActionKind::Reproduce,
                            IllegalActionReason::ReproduceNoSpace,
                            self.tick,
                        );
                        if creature.energy <= 0.0 {
                            dead = true;
                            self.diagnostics.deaths += 1;
                            push_event(creature, CreatureEventKind::Starved, self.tick);
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
                self.drop_inventory_on_death(creature.x, creature.y, &creature.slots);
            }
        }

        for (
            x,
            y,
            energy,
            generation,
            seed,
            controller,
            lineage_id,
            parent_id,
            slot_capacity,
            memory_register,
        ) in offspring
        {
            if self.creatures.len() >= self.config.max_creatures {
                break;
            }
            let idx = self.idx(x, y);
            if self.creature_at[idx].is_some() || self.cells[idx].barrier {
                continue;
            }

            let phenotype_color = controller.phenotype_color();
            let child = Creature {
                x,
                y,
                energy: energy.min(self.config.energy_max),
                age: 0,
                generation,
                lineage_id,
                parent_id: Some(parent_id),
                controller,
                phenotype_color,
                memory_register,
                rng: SmallRng::seed_from_u64(seed),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
                slot_capacity,
                slots: empty_slots(slot_capacity),
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

    fn drop_inventory_on_death(&mut self, x: u32, y: u32, slots: &[Option<InventoryItem>]) {
        for item in slots.iter().flatten() {
            for direction in [
                TouchDirection::SelfCell,
                TouchDirection::North,
                TouchDirection::East,
                TouchDirection::South,
                TouchDirection::West,
            ] {
                let Some((tx, ty)) = self.touch_target(x, y, direction) else {
                    continue;
                };
                let idx = self.idx(tx, ty);
                let placed = match item {
                    InventoryItem::Food(food_value) => {
                        if self.cells[idx].barrier
                            || self.cells[idx].food + *food_value
                                > self.config.food_max_density + f32::EPSILON
                        {
                            false
                        } else {
                            self.cells[idx].food += *food_value;
                            true
                        }
                    }
                    InventoryItem::Barrier => {
                        if self.creature_at[idx].is_some() || self.cells[idx].barrier {
                            false
                        } else {
                            self.cells[idx].food = 0.0;
                            self.cells[idx].barrier = true;
                            true
                        }
                    }
                };

                if placed {
                    break;
                }
            }
        }
    }
}

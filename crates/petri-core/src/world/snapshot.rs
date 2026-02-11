use std::collections::{HashMap, VecDeque};

use rand::SeedableRng;
use slotmap::Key;

use super::*;

impl World {
    pub fn snapshot(&self) -> WorldSnapshot {
        let has_barriers = self.cells.iter().any(|cell| cell.barrier);
        let cells_barrier = if has_barriers {
            self.cells.iter().map(|cell| cell.barrier).collect()
        } else {
            Vec::new()
        };
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
                memory_register: c.memory_register.clone(),
                last_move_blocked: c.last_move_blocked,
                last_inputs: c.last_inputs,
                last_outputs: c.last_outputs,
            })
            .collect::<Vec<_>>();

        WorldSnapshot {
            tick: self.tick,
            config: self.config.clone(),
            palette: self.palette,
            cells_food: self.cells.iter().map(|cell| cell.food).collect(),
            cells_barrier,
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
            cells: vec![
                Cell {
                    food: 0.0,
                    barrier: false,
                };
                total_cells
            ],
            creature_at: vec![None; total_cells],
            creatures: SlotMap::with_key(),
            rng: SmallRng::seed_from_u64(snapshot.tick ^ 0xA11C_E5EED_u64),
            palette: snapshot.palette,
            diagnostics: snapshot.diagnostics,
            lineage_tree: HashMap::new(),
            next_lineage_id: snapshot.next_lineage_id.max(1),
        };

        for (idx, food) in snapshot
            .cells_food
            .into_iter()
            .enumerate()
            .take(total_cells)
        {
            world.cells[idx].food = food.max(0.0);
        }
        for (idx, barrier) in snapshot
            .cells_barrier
            .into_iter()
            .enumerate()
            .take(total_cells)
        {
            world.cells[idx].barrier = barrier;
        }

        let mut id_map: HashMap<u64, CreatureId> = HashMap::new();
        let mut pending_parent: Vec<(CreatureId, Option<u64>)> = Vec::new();

        for creature in snapshot.creatures {
            let old_id = creature.id;
            let x = creature.x.min(world.config.width.saturating_sub(1));
            let y = creature.y.min(world.config.height.saturating_sub(1));
            let idx = world.idx(x, y);
            if world.cells[idx].barrier || world.creature_at[idx].is_some() {
                continue;
            }
            let new_creature = Creature {
                x,
                y,
                energy: creature.energy,
                age: creature.age,
                generation: creature.generation,
                lineage_id: creature.lineage_id,
                parent_id: creature.parent_id,
                controller: creature.controller,
                memory_register: normalize_memory_register(creature.memory_register),
                rng: SmallRng::seed_from_u64(old_id ^ snapshot.tick.rotate_left(13)),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                last_move_blocked: creature.last_move_blocked,
                last_inputs: creature.last_inputs,
                last_outputs: creature.last_outputs,
            };

            let new_id = world.creatures.insert(new_creature);
            id_map.insert(old_id, new_id);
            pending_parent.push((new_id, creature.parent_id));
            world.creature_at[idx] = Some(new_id);
        }

        for (new_id, parent_old) in pending_parent {
            if let Some(creature) = world.creatures.get_mut(new_id) {
                creature.parent_id = parent_old
                    .and_then(|old| id_map.get(&old).map(|mapped| mapped.data().as_ffi()));
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
        world.next_lineage_id = world.next_lineage_id.max(max_lineage.saturating_add(1));
        world
    }
}

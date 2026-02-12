use std::collections::VecDeque;

use petri_graph::MutationConfig;
use rand::{Rng, SeedableRng};
use slotmap::Key;

use super::helpers::wrap_axis;
use super::*;

impl World {
    pub(super) fn spawn_random_creature(
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
            if self.creature_at[idx].is_none() && !self.cells[idx].barrier {
                let x = (idx as u32) % self.config.width;
                let y = (idx as u32) / self.config.width;
                let seed = self.rng.gen::<u64>();
                let mut controller = ComputationGraph::founder(self.palette);
                let initial_mutation_cfg = self.initial_mutation_config();
                // Add slight startup diversity so founders are viable but not identical clones.
                controller.mutate_with_config(&mut self.rng, initial_mutation_cfg);
                let phenotype_color = controller.phenotype_color();
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
                    phenotype_color,
                    memory_register: founder_memory_register(),
                    rng: SmallRng::seed_from_u64(seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                    illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
                    slot_capacity: founder_slot_capacity(),
                    slots: empty_slots(founder_slot_capacity()),
                    last_move_blocked: false,
                    last_inputs: SensorInputs::default(),
                    last_outputs: ActionOutputs::default(),
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
    pub(super) fn initial_mutation_config(&self) -> MutationConfig {
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

    pub(super) fn offspring_mutation_config(&self) -> MutationConfig {
        MutationConfig {
            weight_mutation_rate: self.config.weight_mutation_rate.clamp(0.0, 1.0),
            weight_mutation_magnitude: self.config.weight_mutation_magnitude.max(0.0),
            logic_node_mutation_rate: self.config.logic_node_mutation_rate.clamp(0.0, 1.0),
            structural_mutation_rate: self.config.structural_mutation_rate.clamp(0.0, 1.0),
        }
    }

    pub(super) fn find_empty_neighbor(&self, x: u32, y: u32) -> Option<(u32, u32)> {
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
                if self.creature_at[idx].is_none() && !self.cells[idx].barrier {
                    return Some((nx, ny));
                }
            }
        }
        None
    }
}

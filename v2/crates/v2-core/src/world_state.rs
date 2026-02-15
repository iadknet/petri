use std::collections::HashSet;

use crate::world_seed::WorldSeedConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldActionCounts {
    pub r#move: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub inventory_pickup: u32,
    pub inventory_put: u32,
    pub noop: u32,
}

impl WorldActionCounts {
    pub fn accumulate(&mut self, other: Self) {
        self.r#move = self.r#move.saturating_add(other.r#move);
        self.eat = self.eat.saturating_add(other.eat);
        self.reproduce = self.reproduce.saturating_add(other.reproduce);
        self.inventory_pickup = self.inventory_pickup.saturating_add(other.inventory_pickup);
        self.inventory_put = self.inventory_put.saturating_add(other.inventory_put);
        self.noop = self.noop.saturating_add(other.noop);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTickConfig {
    pub energy_decay_per_tick: f32,
    pub move_cost: f32,
    pub food_energy_gain: f32,
    pub energy_max: f32,
}

impl Default for WorldTickConfig {
    fn default() -> Self {
        Self {
            energy_decay_per_tick: 0.08,
            move_cost: 0.02,
            food_energy_gain: 0.25,
            energy_max: 20.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTickOutcome {
    pub action_counts: WorldActionCounts,
    pub births: u32,
    pub deaths: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldCreature {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub phenotype_rgb: [u8; 3],
}

pub fn tick_world(
    width: u16,
    height: u16,
    wrap: bool,
    tick: u64,
    seed: u64,
    seed_config: WorldSeedConfig,
    tick_config: WorldTickConfig,
    creatures: &mut Vec<WorldCreature>,
    food: &mut HashSet<WorldCell>,
    barriers: &HashSet<WorldCell>,
) -> WorldTickOutcome {
    grow_food_for_tick(
        width,
        height,
        creatures,
        food,
        barriers,
        tick,
        seed,
        seed_config,
    );

    let mut outcome = WorldTickOutcome::default();
    let energy_decay = tick_config.energy_decay_per_tick.max(0.0);
    let move_cost = tick_config.move_cost.max(0.0);
    let food_energy_gain = tick_config.food_energy_gain.max(0.0);
    let energy_max = tick_config.energy_max.max(0.01);

    let mut ordered = std::mem::take(creatures);
    ordered.sort_by_key(|creature| creature.id);
    let mut occupied = ordered
        .iter()
        .map(|creature| WorldCell {
            x: creature.x,
            y: creature.y,
        })
        .collect::<HashSet<_>>();

    let mut survivors = Vec::with_capacity(ordered.len());
    let mut rng = Lcg64::new(seed ^ tick ^ 0xD00D_F00D_5150_1ACE);
    const DIRECTIONS: [(i16, i16); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

    for mut creature in ordered {
        let current = WorldCell {
            x: creature.x,
            y: creature.y,
        };
        occupied.remove(&current);

        creature.energy -= energy_decay;
        if creature.energy <= 0.0 {
            outcome.deaths = outcome.deaths.saturating_add(1);
            continue;
        }

        if food.remove(&current) {
            creature.energy = (creature.energy + food_energy_gain).min(energy_max);
            outcome.action_counts.eat = outcome.action_counts.eat.saturating_add(1);
        } else {
            let mut moved = false;
            let base_dir = rng.next_usize(DIRECTIONS.len());
            for offset in 0..DIRECTIONS.len() {
                let (dx, dy) = DIRECTIONS[(base_dir + offset) % DIRECTIONS.len()];
                let Some(target) = normalize_cell(
                    i32::from(creature.x) + i32::from(dx),
                    i32::from(creature.y) + i32::from(dy),
                    width,
                    height,
                    wrap,
                ) else {
                    continue;
                };
                if occupied.contains(&target) || barriers.contains(&target) {
                    continue;
                }
                creature.x = target.x;
                creature.y = target.y;
                moved = true;
                break;
            }
            if moved {
                creature.energy -= move_cost;
                outcome.action_counts.r#move = outcome.action_counts.r#move.saturating_add(1);
            } else {
                outcome.action_counts.noop = outcome.action_counts.noop.saturating_add(1);
            }
        }

        if creature.energy <= 0.0 {
            outcome.deaths = outcome.deaths.saturating_add(1);
            continue;
        }

        occupied.insert(WorldCell {
            x: creature.x,
            y: creature.y,
        });
        survivors.push(creature);
    }

    *creatures = survivors;
    outcome
}

fn grow_food_for_tick(
    width: u16,
    height: u16,
    creatures: &[WorldCreature],
    food: &mut HashSet<WorldCell>,
    barriers: &HashSet<WorldCell>,
    tick: u64,
    seed: u64,
    config: WorldSeedConfig,
) {
    let width_usize = usize::from(width);
    let height_usize = usize::from(height);
    let total_cells = width_usize.saturating_mul(height_usize);
    if total_cells == 0 {
        return;
    }
    if config.food_growth_rate <= 0.0 && config.food_spawn_rate <= 0.0 {
        return;
    }

    let current_density = food.len() as f32 / total_cells as f32;
    let mut attempts =
        ((total_cells as f32 * config.food_spawn_rate.clamp(0.0, 1.0)).round() as usize).max(1);
    if current_density < config.food_spawn_floor_density.clamp(0.0, 1.0) {
        attempts = attempts.saturating_mul(2);
    }
    if current_density >= config.food_spread_threshold.clamp(0.0, 1.0) {
        attempts = attempts.saturating_add(attempts / 2);
    }
    let growth_bonus = (config.food_growth_rate.clamp(0.0, 1.0) * 8.0).round() as usize;
    attempts = attempts.saturating_add(growth_bonus);

    let creature_cells = creatures
        .iter()
        .map(|creature| WorldCell {
            x: creature.x,
            y: creature.y,
        })
        .collect::<HashSet<_>>();

    let mut rng = Lcg64::new(seed ^ tick ^ 0xA5A5_5A5A_1122_3344);
    for _ in 0..attempts {
        let index = rng.next_usize(total_cells);
        let x = (index % width_usize) as u16;
        let y = (index / width_usize) as u16;
        let cell = WorldCell { x, y };
        if barriers.contains(&cell) || creature_cells.contains(&cell) {
            continue;
        }
        food.insert(cell);
    }
}

fn normalize_cell(x: i32, y: i32, width: u16, height: u16, wrap: bool) -> Option<WorldCell> {
    if width == 0 || height == 0 {
        return None;
    }
    if wrap {
        return Some(WorldCell {
            x: x.rem_euclid(i32::from(width)) as u16,
            y: y.rem_euclid(i32::from(height)) as u16,
        });
    }
    if x < 0 || y < 0 {
        return None;
    }
    let ux = x as u16;
    let uy = y as u16;
    if ux < width && uy < height {
        Some(WorldCell { x: ux, y: uy })
    } else {
        None
    }
}

#[derive(Clone, Debug)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_usize(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() % upper_exclusive as u64) as usize
    }
}

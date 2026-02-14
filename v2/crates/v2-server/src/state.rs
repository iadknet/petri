use std::collections::HashSet;

use crate::api::{ActionCounts, PaintPoint, PaintStrokeTool, StartupRequest};
use v2_core::ecology::{EcologyConfig, run_noncollapse_baseline};
use v2_core::phenotype::FOUNDER_PHENOTYPE_RGB;
use v2_core::viability::{StartupViabilityGate, run_startup_viability_gate};
use v2_core::world_seed::{WorldSeedConfig, seed_creature_cells, seed_food_cells};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationPhase {
    Idle,
    Running,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug)]
pub struct WorldCreature {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub phenotype_rgb: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct WorldGrid {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,
    pub food: HashSet<WorldCell>,
    pub barriers: HashSet<WorldCell>,
    pub creatures: Vec<WorldCreature>,
}

impl WorldGrid {
    #[must_use]
    pub fn new(width: u16, height: u16, wrap: bool) -> Self {
        Self {
            width,
            height,
            wrap,
            food: HashSet::new(),
            barriers: HashSet::new(),
            creatures: Vec::new(),
        }
    }

    #[must_use]
    pub fn in_bounds(&self, x: u16, y: u16) -> bool {
        x < self.width && y < self.height
    }

    #[must_use]
    pub fn apply_stroke(
        &mut self,
        tool: PaintStrokeTool,
        brush_half_extent: u8,
        points: &[PaintPoint],
    ) -> u32 {
        let mut changed = HashSet::new();
        let radius = i32::from(brush_half_extent);
        for point in points {
            let px = i32::from(point.x);
            let py = i32::from(point.y);
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if let Some(cell) = self.normalize_cell(px + dx, py + dy) {
                        let was_changed = apply_tool(self, tool, cell);
                        if was_changed {
                            changed.insert(cell);
                        }
                    }
                }
            }
        }
        changed.len() as u32
    }

    #[must_use]
    pub fn clear_all(&mut self) -> u32 {
        let touched = self.food.len().saturating_add(self.barriers.len()) as u32;
        self.food.clear();
        self.barriers.clear();
        touched
    }

    fn normalize_cell(&self, x: i32, y: i32) -> Option<WorldCell> {
        if self.wrap {
            if self.width == 0 || self.height == 0 {
                return None;
            }
            let wrapped_x = x.rem_euclid(i32::from(self.width)) as u16;
            let wrapped_y = y.rem_euclid(i32::from(self.height)) as u16;
            return Some(WorldCell {
                x: wrapped_x,
                y: wrapped_y,
            });
        }

        if x < 0 || y < 0 {
            return None;
        }
        let x = x as u16;
        let y = y as u16;
        self.in_bounds(x, y).then_some(WorldCell { x, y })
    }
}

fn apply_tool(grid: &mut WorldGrid, tool: PaintStrokeTool, cell: WorldCell) -> bool {
    match tool {
        PaintStrokeTool::Food => {
            let mut changed = false;
            if grid.barriers.remove(&cell) {
                changed = true;
            }
            if grid.food.insert(cell) {
                changed = true;
            }
            changed
        }
        PaintStrokeTool::Barrier => {
            let mut changed = false;
            if grid.food.remove(&cell) {
                changed = true;
            }
            if grid.barriers.insert(cell) {
                changed = true;
            }
            changed
        }
        PaintStrokeTool::EraseFood => grid.food.remove(&cell),
        PaintStrokeTool::EraseBarrier => grid.barriers.remove(&cell),
    }
}

#[derive(Clone, Debug)]
pub struct SimulationState {
    pub phase: SimulationPhase,
    pub tick: u64,
    pub config_digest: String,
    pub startup: StartupRequest,
    pub world: WorldGrid,
    pub health_window_ticks: u16,
    pub population: u32,
    pub mean_energy: f32,
    pub births_last_window: u32,
    pub deaths_last_window: u32,
    pub last_action_counts: ActionCounts,
    pub world_seed_config: WorldSeedConfig,
}

impl SimulationState {
    #[must_use]
    pub fn new_default() -> Self {
        let startup = StartupRequest::default();
        let world = WorldGrid::new(
            startup.world.width,
            startup.world.height,
            startup.world.wrap,
        );
        Self {
            phase: SimulationPhase::Idle,
            tick: 0,
            config_digest: String::new(),
            startup,
            world,
            health_window_ticks: EcologyConfig::default().health_window_ticks as u16,
            population: 0,
            mean_energy: 0.0,
            births_last_window: 0,
            deaths_last_window: 0,
            last_action_counts: ActionCounts::default(),
            world_seed_config: WorldSeedConfig::default(),
        }
    }

    pub fn reset(&mut self, startup: StartupRequest, digest: String) {
        let startup = sanitize_startup_request(startup);
        let mut world = WorldGrid::new(
            startup.world.width,
            startup.world.height,
            startup.world.wrap,
        );
        let occupancy = u32::from(startup.world.width) * u32::from(startup.world.height);
        let seeded = startup
            .population
            .initial_creatures
            .min(startup.population.max_creatures)
            .min(occupancy);

        seed_world(&mut world, seeded, startup.seed, self.world_seed_config);
        enforce_startup_viability(&mut world, startup.seed);

        self.phase = SimulationPhase::Idle;
        self.tick = 0;
        self.config_digest = digest;
        self.startup = startup;
        self.world = world;
        self.population = self.world.creatures.len() as u32;
        self.mean_energy = if self.population == 0 {
            0.0
        } else {
            self.world
                .creatures
                .iter()
                .map(|creature| creature.energy)
                .sum::<f32>()
                / self.population as f32
        };
        self.births_last_window = 0;
        self.deaths_last_window = 0;
        self.last_action_counts = ActionCounts::default();
        self.refresh_health_from_core();
    }

    pub fn advance_ticks(&mut self, steps: u16) {
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            self.grow_food_for_tick();
        }
        self.last_action_counts = ActionCounts::default();
        self.refresh_health_from_core();
    }

    pub fn refresh_health_from_core(&mut self) {
        let config = EcologyConfig::default();
        self.health_window_ticks = config.health_window_ticks as u16;

        let ticks = u32::try_from(self.tick.max(1)).unwrap_or(u32::MAX);
        let run = run_noncollapse_baseline(
            self.startup.seed,
            ticks,
            (self.world.creatures.len() as u32).max(1),
            &config,
        );

        if let Some(latest) = run.snapshots.last() {
            self.mean_energy = latest.mean_energy;
            self.births_last_window = latest.births_last_window;
            self.deaths_last_window = latest.deaths_last_window;
        }

        self.population = self.world.creatures.len() as u32;
    }

    fn grow_food_for_tick(&mut self) {
        let width = usize::from(self.world.width);
        let height = usize::from(self.world.height);
        let total_cells = width.saturating_mul(height);
        if total_cells == 0 {
            return;
        }

        let config = self.world_seed_config;
        if config.food_growth_rate <= 0.0 && config.food_spawn_rate <= 0.0 {
            return;
        }

        let current_density = self.world.food.len() as f32 / total_cells as f32;
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

        let creature_cells = self
            .world
            .creatures
            .iter()
            .map(|creature| WorldCell {
                x: creature.x,
                y: creature.y,
            })
            .collect::<HashSet<_>>();

        let mut rng = Lcg64::new(self.startup.seed ^ self.tick ^ 0xA5A5_5A5A_1122_3344);
        for _ in 0..attempts {
            let index = rng.next_usize(total_cells);
            let x = (index % width) as u16;
            let y = (index / width) as u16;
            let cell = WorldCell { x, y };

            if self.world.barriers.contains(&cell) || creature_cells.contains(&cell) {
                continue;
            }

            self.world.food.insert(cell);
        }
    }

    #[must_use]
    pub fn phase_label(&self) -> &'static str {
        match self.phase {
            SimulationPhase::Idle => "idle",
            SimulationPhase::Running => "running",
            SimulationPhase::Paused => "paused",
        }
    }
}

fn seed_world(world: &mut WorldGrid, creature_count: u32, seed: u64, config: WorldSeedConfig) {
    for (x, y) in seed_food_cells(world.width, world.height, config.initial_food_density, seed) {
        world.food.insert(WorldCell { x, y });
    }

    world.creatures = seed_creature_cells(world.width, world.height, creature_count as usize, seed)
        .into_iter()
        .enumerate()
        .map(|(index, (x, y))| WorldCreature {
            id: index as u64 + 1,
            x,
            y,
            energy: 20.0,
            phenotype_rgb: FOUNDER_PHENOTYPE_RGB,
        })
        .collect();
}

fn enforce_startup_viability(world: &mut WorldGrid, seed: u64) {
    let ecology = EcologyConfig::default();
    let gate = StartupViabilityGate::default();
    let viability = run_startup_viability_gate(seed, world.creatures.len() as u32, &ecology, &gate);

    if !viability.viable || world.creatures.is_empty() {
        world.creatures.clear();
        world.creatures.push(WorldCreature {
            id: 1,
            x: 0,
            y: 0,
            energy: 20.0,
            phenotype_rgb: FOUNDER_PHENOTYPE_RGB,
        });
    }

    if world.food.is_empty() {
        let fallback = world
            .creatures
            .first()
            .map(|creature| WorldCell {
                x: creature.x,
                y: creature.y,
            })
            .unwrap_or(WorldCell { x: 0, y: 0 });
        world.food.insert(fallback);
    }
}

fn sanitize_startup_request(mut startup: StartupRequest) -> StartupRequest {
    startup.world.width = startup.world.width.max(1);
    startup.world.height = startup.world.height.max(1);
    startup.population.max_creatures = startup.population.max_creatures.max(1);
    startup.population.initial_creatures = startup
        .population
        .initial_creatures
        .max(1)
        .min(startup.population.max_creatures);
    startup
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

use std::collections::{HashSet, VecDeque};

use crate::api::{ActionCounts, PaintPoint, PaintStrokeTool, StartupRequest};
use v2_core::ecology::EcologyConfig;
use v2_core::phenotype::FOUNDER_PHENOTYPE_RGB;
use v2_core::viability::{run_startup_viability_gate, StartupViabilityGate};
use v2_core::world_seed::{seed_creature_cells, seed_food_cells, WorldSeedConfig};
use v2_core::world_state::{
    tick_world, WorldActionCounts, WorldCell, WorldCreature, WorldTickConfig,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationPhase {
    Idle,
    Running,
    Paused,
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
    pub world_tick_config: WorldTickConfig,
    recent_population_events: VecDeque<(u32, u32)>,
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
            world_tick_config: WorldTickConfig::default(),
            recent_population_events: VecDeque::new(),
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
        self.health_window_ticks = EcologyConfig::default().health_window_ticks as u16;
        self.recent_population_events.clear();
        self.births_last_window = 0;
        self.deaths_last_window = 0;
        self.last_action_counts = ActionCounts::default();
        self.refresh_metrics_from_world();
    }

    pub fn advance_ticks(&mut self, steps: u16) {
        let mut combined_action_counts = WorldActionCounts::default();
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            let outcome = tick_world(
                self.world.width,
                self.world.height,
                self.world.wrap,
                self.tick,
                self.startup.seed,
                self.world_seed_config,
                self.world_tick_config,
                &mut self.world.creatures,
                &mut self.world.food,
                &self.world.barriers,
            );
            combined_action_counts.accumulate(outcome.action_counts);
            self.record_population_window(outcome.births, outcome.deaths);
        }
        self.last_action_counts = map_action_counts(combined_action_counts);
        self.refresh_metrics_from_world();
    }

    fn record_population_window(&mut self, births: u32, deaths: u32) {
        let max_window = usize::from(self.health_window_ticks.max(1));
        self.recent_population_events.push_back((births, deaths));
        while self.recent_population_events.len() > max_window {
            self.recent_population_events.pop_front();
        }
    }

    fn refresh_metrics_from_world(&mut self) {
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
        self.births_last_window = self
            .recent_population_events
            .iter()
            .map(|(births, _)| *births)
            .sum();
        self.deaths_last_window = self
            .recent_population_events
            .iter()
            .map(|(_, deaths)| *deaths)
            .sum();
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

fn map_action_counts(counts: WorldActionCounts) -> ActionCounts {
    ActionCounts {
        r#move: counts.r#move,
        eat: counts.eat,
        reproduce: counts.reproduce,
        inventory_pickup: counts.inventory_pickup,
        inventory_put: counts.inventory_put,
        noop: counts.noop,
    }
}

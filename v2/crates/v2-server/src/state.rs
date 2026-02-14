use std::collections::HashSet;

use crate::api::{ActionCounts, PaintPoint, PaintStrokeTool, StartupRequest};
use v2_core::ecology::{EcologyConfig, run_noncollapse_baseline};

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
pub struct WorldGrid {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,
    pub food: HashSet<WorldCell>,
    pub barriers: HashSet<WorldCell>,
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
        }
    }

    pub fn reset(&mut self, startup: StartupRequest, digest: String) {
        let world = WorldGrid::new(
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
        self.phase = SimulationPhase::Idle;
        self.tick = 0;
        self.config_digest = digest;
        self.startup = startup;
        self.world = world;
        self.population = seeded;
        self.mean_energy = if seeded == 0 { 0.0 } else { 20.0 };
        self.births_last_window = 0;
        self.deaths_last_window = 0;
        self.last_action_counts = ActionCounts::default();
        self.refresh_health_from_core();
    }

    pub fn advance_ticks(&mut self, steps: u16) {
        self.tick = self.tick.saturating_add(u64::from(steps));
        self.last_action_counts = action_counts_for_tick(self.tick);
        self.refresh_health_from_core();
    }

    pub fn refresh_health_from_core(&mut self) {
        let config = EcologyConfig::default();
        self.health_window_ticks = config.health_window_ticks as u16;

        let ticks = u32::try_from(self.tick.max(1)).unwrap_or(u32::MAX);
        let run =
            run_noncollapse_baseline(self.startup.seed, ticks, self.population.max(1), &config);

        if let Some(latest) = run.snapshots.last() {
            self.population = latest.population.max(1);
            self.mean_energy = latest.mean_energy;
            self.births_last_window = latest.births_last_window;
            self.deaths_last_window = latest.deaths_last_window;
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

fn action_counts_for_tick(tick: u64) -> ActionCounts {
    ActionCounts {
        r#move: ((tick + 1) % 9) as u32,
        eat: ((tick + 2) % 7) as u32,
        reproduce: ((tick + 3) % 5) as u32,
        inventory_pickup: ((tick + 4) % 4) as u32,
        inventory_put: ((tick + 5) % 4) as u32,
        noop: ((tick + 6) % 6) as u32,
    }
}

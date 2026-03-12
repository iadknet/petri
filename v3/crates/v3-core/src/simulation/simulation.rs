use rand::rngs::SmallRng;
use slotmap::{SecondaryMap, SlotMap};

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::action_log::ActionLog;
use crate::creature::state::CreatureState;
use crate::kernel::paint::{PaintPoint, PaintStats, PaintTool};
use crate::kernel::WorldState;
use crate::simulation::stats::SimStats;

/// Central simulation container.
///
/// Holds world state, creature slotmap, tick counter, config, and the simulation RNG.
/// The RNG is private to maintain determinism — only `seeding` and `tick` modules
/// drive it directly.
pub struct Simulation {
    pub world: WorldState,
    pub creatures: SlotMap<CreatureId, CreatureState>,
    /// Per-creature action logs, parallel to `creatures`. Observational only.
    pub action_logs: SecondaryMap<CreatureId, ActionLog>,
    pub tick: u64,
    pub config: SimulationConfig,
    pub stats: SimStats,
    /// Private RNG seeded at startup; used for food growth, turn-queue shuffling, etc.
    pub(crate) rng: SmallRng,
}

impl Simulation {
    /// Create a `Simulation` from pre-built components, seeding the internal RNG from `seed`.
    ///
    /// Prefer [`crate::simulation::seed_simulation`] for normal simulation startup.
    /// This constructor exists for tests that need fine-grained control over world state.
    pub fn new(
        world: WorldState,
        creatures: SlotMap<CreatureId, CreatureState>,
        tick: u64,
        config: SimulationConfig,
        seed: u64,
    ) -> Self {
        use rand::SeedableRng;
        Self {
            world,
            creatures,
            action_logs: SecondaryMap::new(),
            tick,
            config,
            stats: SimStats::default(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Number of living creatures in this simulation.
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    /// Current tick number (incremented by `run_tick`).
    pub fn tick_number(&self) -> u64 {
        self.tick
    }

    /// Apply a paint stroke, mutating world grids and evicting any creatures under barriers.
    pub fn apply_paint(
        &mut self,
        tool: PaintTool,
        brush_half_extent: u8,
        points: &[PaintPoint],
    ) -> PaintStats {
        let max_density = self.config.world.food.max_density;
        let (stats, evicted) =
            self.world
                .apply_paint_stroke(tool, brush_half_extent, points, max_density);
        for id in evicted {
            self.remove_creature(id);
        }
        stats
    }

    /// Remove one creature and record exit-time mutation value aggregates.
    pub fn remove_creature(&mut self, id: CreatureId) {
        if let Some(creature) = self.creatures.remove(id) {
            for operator in creature.birth_mutation_operators.iter().copied() {
                let totals = self
                    .stats
                    .mutation_value_totals_by_operator
                    .entry(operator)
                    .or_default();
                totals.carriers_observed_total += 1;
                totals.survival_ticks_sum += creature.age;
                totals.offspring_spawned_sum += creature.offspring_spawned_count;
                totals.final_energy_sum += f64::from(creature.energy.max(0.0));
            }
            self.world.remove_creature(creature.position);
        }
        self.action_logs.remove(id);
    }

    /// Mean energy across all living creatures. Returns `0.0` for an empty population.
    pub fn mean_energy(&self) -> f32 {
        if self.creatures.is_empty() {
            0.0
        } else {
            self.creatures.values().map(|c| c.energy).sum::<f32>() / self.creatures.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::seeding::seed_simulation;

    fn small_config() -> SimulationConfig {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 20;
        cfg.world.height = 20;
        cfg.population.initial_creatures = 5;
        cfg
    }

    #[test]
    fn mean_energy_returns_zero_for_empty_simulation() {
        let mut sim = seed_simulation(small_config(), 42);
        sim.creatures.clear();
        assert_eq!(sim.mean_energy(), 0.0);
    }

    #[test]
    fn mean_energy_returns_correct_mean_for_multiple_creatures() {
        let sim = seed_simulation(small_config(), 42);
        let expected =
            sim.creatures.values().map(|c| c.energy).sum::<f32>() / sim.creatures.len() as f32;
        assert!((sim.mean_energy() - expected).abs() < f32::EPSILON);
    }
}

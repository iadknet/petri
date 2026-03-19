use rand::rngs::SmallRng;
use slotmap::{SecondaryMap, SlotMap};

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::action_log::ActionLog;
use crate::creature::state::CreatureState;
use crate::kernel::paint::{PaintPoint, PaintStats, PaintTool};
use crate::kernel::WorldState;
use crate::simulation::stats::{MutationOutcomeObservation, SimStats};

const SHORT_SURVIVAL_HORIZON_TICKS: u64 = 32;
const LONG_SURVIVAL_HORIZON_TICKS: u64 = 128;

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
        mut world: WorldState,
        creatures: SlotMap<CreatureId, CreatureState>,
        tick: u64,
        config: SimulationConfig,
        seed: u64,
    ) -> Self {
        use rand::SeedableRng;
        // Ensure the world's FoodResource uses the simulation's food config.
        world.apply_food_config(config.world.food.clone());
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
            let observation = build_mutation_outcome_observation(&creature, &self.config);
            let evaluation = self
                .stats
                .classify_mutation_outcome(creature.generation, observation.viability_score);

            if !creature.birth_mutation_operators.is_empty() {
                self.stats
                    .mutation_outcome_summary
                    .record_outcome(observation, evaluation);
            }

            for operator in creature.birth_mutation_operators.iter().copied() {
                self.stats
                    .mutation_value_totals_by_operator
                    .entry(operator)
                    .or_default()
                    .record_outcome(observation, evaluation);
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

fn build_mutation_outcome_observation(
    creature: &CreatureState,
    config: &SimulationConfig,
) -> MutationOutcomeObservation {
    let survived_short_horizon = creature.age >= SHORT_SURVIVAL_HORIZON_TICKS;
    let survived_long_horizon = creature.age >= LONG_SURVIVAL_HORIZON_TICKS;
    let reproduced_once = creature.offspring_spawned_count > 0;
    let mean_lifetime_energy = if creature.lifetime_energy_sample_count > 0 {
        creature.lifetime_energy_sum / creature.lifetime_energy_sample_count as f64
    } else {
        f64::from(creature.energy.max(0.0))
    };
    let blocked_move_total = creature.lifetime_blocked_move_count;
    let invalid_reproduce_total = creature.lifetime_invalid_reproduce_count;
    let invalid_action_total = blocked_move_total + invalid_reproduce_total;
    let action_attempted_total = creature.lifetime_action_attempted_count.max(1);
    let invalid_action_rate = invalid_action_total as f64 / action_attempted_total as f64;

    // Composite score tuned for observability: survival + reproduction + energy stability,
    // with a penalty for recurrent invalid/blocked actions.
    let min_reproduce_energy = f64::from(config.energy.lifecycle.min_reproduce_energy.max(0.001));
    let energy_term = (mean_lifetime_energy / min_reproduce_energy).clamp(0.0, 1.0);
    let viability_score = 0.30 * survived_short_horizon as u8 as f64
        + 0.35 * survived_long_horizon as u8 as f64
        + 0.25 * reproduced_once as u8 as f64
        + 0.10 * energy_term
        - 0.20 * invalid_action_rate;

    MutationOutcomeObservation {
        survival_ticks: creature.age,
        offspring_spawned_total: creature.offspring_spawned_count,
        final_energy: f64::from(creature.energy.max(0.0)),
        viability_score,
        mean_lifetime_energy,
        action_attempted_total: creature.lifetime_action_attempted_count,
        blocked_move_total,
        invalid_reproduce_total,
        invalid_action_total,
        survived_short_horizon,
        survived_long_horizon,
        reproduced_once,
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

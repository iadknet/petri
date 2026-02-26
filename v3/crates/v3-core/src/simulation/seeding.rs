use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use slotmap::SlotMap;

use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Position};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::simulation::simulation::Simulation;

/// Founder phenotype baseline per v3-phenotype-spec.md.
const FOUNDER_RGB: [u8; 3] = [204, 61, 61];
const FOUNDER_ACTIVE_CHANNEL: usize = 0; // all founders start on red channel
const FOUNDER_POLARITY: [bool; 3] = [true, true, true];

/// Seed a new simulation from a `SimulationConfig` and a deterministic seed.
///
/// Steps per v3-startup-seeding-spec.md Section 4:
/// 1. Seed RNG from `seed`.
/// 2. Create an empty world with dimensions and edge mode from config.
/// 3. Seed food into the world.
/// 4. Collect all non-barrier cells, shuffle, take the first `initial_creatures` positions.
/// 5. Place a founder creature at each selected position.
/// 6. Return the assembled `Simulation`.
pub fn seed_simulation(config: SimulationConfig, seed: u64) -> Simulation {
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut world = WorldState::new(
        config.world.width,
        config.world.height,
        config.world.edge_mode,
    );
    world.seed_food(&mut rng, &config);

    // Collect all non-barrier, in-bounds positions.
    let mut positions: Vec<Position> =
        Vec::with_capacity(config.world.width as usize * config.world.height as usize);
    for y in 0..config.world.height {
        for x in 0..config.world.width {
            let pos = Position::new(x, y);
            if !world.is_barrier(pos) {
                positions.push(pos);
            }
        }
    }

    positions.shuffle(&mut rng);

    let desired = config.population.initial_creatures as usize;
    let spawn_count = desired.min(positions.len());

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

    for &pos in positions.iter().take(spawn_count) {
        let genome = v3alpha1_founder_genome();
        let energy = config.energy.lifecycle.initial_energy;
        // Insert into slotmap to get an id, then fill with actual state.
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                genome,
                pos,
                energy,
                0,
                FOUNDER_RGB,
                FOUNDER_ACTIVE_CHANNEL,
                FOUNDER_POLARITY,
            )
        });
        world.place_creature(pos, id);
    }

    Simulation {
        world,
        creatures,
        tick: 0,
        config,
        stats: crate::simulation::stats::SimStats::default(),
        rng,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use std::collections::HashSet;

    fn small_config() -> SimulationConfig {
        let mut cfg = SimulationConfig::default();
        // Use a small world so tests run quickly.
        cfg.world.width = 20;
        cfg.world.height = 20;
        cfg.population.initial_creatures = 5;
        cfg
    }

    #[test]
    fn seeded_simulation_has_creatures() {
        let sim = seed_simulation(small_config(), 42);
        assert!(sim.creature_count() > 0);
    }

    #[test]
    fn founders_placed_at_distinct_positions() {
        let sim = seed_simulation(small_config(), 42);
        let mut seen = HashSet::new();
        for (_, creature) in &sim.creatures {
            assert!(
                seen.insert(creature.position),
                "duplicate position {:?}",
                creature.position
            );
        }
    }

    #[test]
    fn world_has_food_after_seeding() {
        let sim = seed_simulation(small_config(), 42);
        assert!(sim.world.total_food() > 0.0);
    }

    #[test]
    fn seeding_is_deterministic() {
        let cfg = small_config();
        let s1 = seed_simulation(cfg.clone(), 42);
        let s2 = seed_simulation(cfg, 42);
        assert_eq!(s1.creature_count(), s2.creature_count());
        assert_eq!(s1.world.total_food(), s2.world.total_food());
    }

    #[test]
    fn founder_phenotype_baseline() {
        let sim = seed_simulation(small_config(), 42);
        // All founders should have the canonical baseline phenotype.
        for (_, creature) in &sim.creatures {
            assert_eq!(creature.phenotype_rgb, FOUNDER_RGB, "rgb mismatch");
            assert_eq!(
                creature.phenotype_active_channel, FOUNDER_ACTIVE_CHANNEL,
                "active channel mismatch"
            );
            assert_eq!(
                creature.phenotype_channel_polarity, FOUNDER_POLARITY,
                "polarity mismatch"
            );
        }
    }

    #[test]
    fn founder_energy_matches_config() {
        let cfg = small_config();
        let expected_energy = cfg.energy.lifecycle.initial_energy;
        let sim = seed_simulation(cfg, 42);
        for (_, creature) in &sim.creatures {
            assert!(
                (creature.energy - expected_energy).abs() < f32::EPSILON,
                "energy mismatch: {} vs {}",
                creature.energy,
                expected_energy
            );
        }
    }
}

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use slotmap::{SecondaryMap, SlotMap};

use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{CreatureId, InputReference, Position, WorldInputKey};
use crate::creature::action_log::ActionLog;
use crate::creature::founder::founder_genome_with_min_reproduce_age;
use crate::creature::genome::CreatureGenome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::patterns::{generate_pattern_seeded, PatternBounds};
use crate::simulation::simulation::Simulation;

/// Founder phenotype baseline — 6 HSL-mapped channels that produce RGB [204, 61, 61].
const FOUNDER_CHANNELS: [u8; 6] = [0, 0, 92, 92, 138, 138];
const FOUNDER_ACTIVE_CHANNEL: usize = 0;
const FOUNDER_POLARITY: [bool; 6] = [true; 6];

/// Seed a new simulation from a `SimulationConfig` and a deterministic seed.
///
/// Steps per v3-startup-seeding-spec.md Section 4:
/// 1. Seed RNG from `seed`.
/// 2. Create an empty world with dimensions and edge mode from config.
/// 3. Apply terrain and seed fertility from independent map/layer seeds.
/// 4. Seed food from the run RNG.
/// 5. Collect all non-barrier cells, shuffle, take the first `initial_creatures` positions.
/// 6. Place a founder creature at each selected position.
/// 7. Return the assembled `Simulation`.
pub fn seed_simulation(config: SimulationConfig, seed: u64) -> Simulation {
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut world = WorldState::new(
        config.world.width,
        config.world.height,
        config.world.edge_mode,
    );
    let map_seed = config.world.world_seed.unwrap_or(seed);
    for (index, layer) in config.world.terrain.iter().enumerate() {
        let bounds = layer.bounds.unwrap_or(PatternBounds {
            x: 0,
            y: 0,
            width: world.width,
            height: world.height,
        });
        // Intersect without wrapping or adding potentially overflowing u16 endpoints.
        let bounds = PatternBounds {
            width: bounds.width.min(world.width.saturating_sub(bounds.x)),
            height: bounds.height.min(world.height.saturating_sub(bounds.y)),
            ..bounds
        };
        if bounds.width == 0 || bounds.height == 0 {
            continue;
        }
        let layer_seed = layer
            .seed
            .unwrap_or_else(|| map_seed.wrapping_add(index as u64));
        for point in generate_pattern_seeded(bounds, &layer.params, layer_seed) {
            world.set_barrier(Position::new(point.x, point.y), true);
        }
    }
    world.reconfigure_food(config.world.food.clone());
    world.seed_fertility(map_seed);
    world.seed_food(&mut rng);

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
    let mut action_logs: SecondaryMap<CreatureId, ActionLog> = SecondaryMap::new();
    let log_capacity = config.action_log.capacity;

    for (founder_index, &pos) in positions.iter().take(spawn_count).enumerate() {
        let genome = founder_genome_with_min_reproduce_age(
            config.population.founder_profile,
            config.energy.lifecycle.min_reproduce_age,
        );
        let energy = config.energy.lifecycle.initial_energy;
        let identity = CreatureIdentityState::founder(founder_index, seed);
        // Insert into slotmap to get an id, then fill with actual state.
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                genome,
                pos,
                energy,
                0,
                FOUNDER_CHANNELS,
                FOUNDER_ACTIVE_CHANNEL,
                FOUNDER_POLARITY,
                identity,
                [0.0; crate::creature::state::SHARED_MEMORY_SLOTS],
            )
        });
        action_logs.insert(id, ActionLog::new(log_capacity));
        world.place_creature(pos, id);
    }

    let mut stats = crate::simulation::stats::SimStats::default();
    stats.energy_flows.food_intake_by_type = vec![0.0; config.world.food.types.len()];
    Simulation {
        world,
        creatures,
        action_logs,
        tick: 0,
        config,
        stats,
        rng,
    }
}

fn inject_extended_perception_input(genome: &mut CreatureGenome) {
    let marker = InputReference::World(WorldInputKey::AreaFoodSummary {
        type_idx: OrdinaryFoodTypeId::default(),
    });
    if genome
        .nodes
        .iter()
        .any(|node| node.input_refs.iter().any(|input_ref| input_ref == &marker))
    {
        return;
    }

    if let Some(node) = genome.nodes.first_mut() {
        if let Some(first_input_ref) = node.input_refs.first_mut() {
            *first_input_ref = marker;
        } else {
            node.input_refs.push(marker);
        }
    }
}

/// Seed a deterministic simulation and force the first `perception_creatures`
/// founders to use extended perception inputs.
///
/// This exists for tests and benchmarks that need a stable perception-heavy
/// workload from tick 0, instead of relying on evolution to discover those
/// inputs during the benchmark horizon.
pub fn seed_simulation_with_perception_mix(
    config: SimulationConfig,
    seed: u64,
    perception_creatures: usize,
) -> Simulation {
    let mut sim = seed_simulation(config, seed);
    for (_, creature) in sim.creatures.iter_mut().take(perception_creatures) {
        inject_extended_perception_input(&mut creature.genome);
    }
    sim
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FounderProfile, SimulationConfig};
    use crate::creature::genome::{BackendDef, VmInstruction};
    use crate::sensors::perception::genome_uses_extended_perception;
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
            assert_eq!(
                creature.phenotype_channels, FOUNDER_CHANNELS,
                "channels mismatch"
            );
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

    #[test]
    fn founder_lineage_ids_are_sequential() {
        let sim = seed_simulation(small_config(), 42);
        let mut lineage_ids: Vec<u32> = sim
            .creatures
            .values()
            .map(|c| c.identity.lineage_id)
            .collect();
        lineage_ids.sort();
        let expected: Vec<u32> = (0..lineage_ids.len() as u32).collect();
        assert_eq!(lineage_ids, expected);
    }

    #[test]
    fn founder_kin_tags_are_unique() {
        let sim = seed_simulation(small_config(), 42);
        let kin_tags: HashSet<u32> = sim.creatures.values().map(|c| c.identity.kin_tag).collect();
        assert_eq!(
            kin_tags.len(),
            sim.creature_count(),
            "kin_tags should be unique across founders"
        );
    }

    #[test]
    fn founder_identity_deterministic_across_runs() {
        let cfg = small_config();
        let s1 = seed_simulation(cfg.clone(), 42);
        let s2 = seed_simulation(cfg, 42);
        // Collect identities in lineage_id order for stable comparison.
        let mut ids1: Vec<_> = s1.creatures.values().map(|c| c.identity).collect();
        let mut ids2: Vec<_> = s2.creatures.values().map(|c| c.identity).collect();
        ids1.sort_by_key(|i| i.lineage_id);
        ids2.sort_by_key(|i| i.lineage_id);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn founder_baseline_does_not_use_extended_perception() {
        let sim = seed_simulation(small_config(), 42);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| genome_uses_extended_perception(&creature.genome))
                .count(),
            0
        );
    }

    #[test]
    fn perception_mix_marks_requested_number_of_founders() {
        let sim = seed_simulation_with_perception_mix(small_config(), 42, 2);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| genome_uses_extended_perception(&creature.genome))
                .count(),
            2
        );
    }

    #[test]
    fn perception_mix_rewrites_a_live_input_ref_to_extended_perception() {
        let sim = seed_simulation_with_perception_mix(small_config(), 42, 1);
        let creature = sim.creatures.values().next().expect("seeded creature");
        assert_eq!(
            creature.genome.nodes[0].input_refs[0],
            InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: OrdinaryFoodTypeId::default(),
            })
        );
    }

    #[test]
    fn perception_mix_clamps_to_population_size() {
        let sim = seed_simulation_with_perception_mix(small_config(), 42, 99);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| genome_uses_extended_perception(&creature.genome))
                .count(),
            sim.creature_count()
        );
    }

    #[test]
    fn perception_mix_scales_to_benchmark_population() {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 240;
        cfg.world.height = 240;
        cfg.population.initial_creatures = 400;
        let sim = seed_simulation_with_perception_mix(cfg, 42, 200);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| genome_uses_extended_perception(&creature.genome))
                .count(),
            200
        );
    }

    #[test]
    fn seeding_uses_configured_founder_profile() {
        let mut cfg = small_config();
        cfg.population.founder_profile = FounderProfile::ForageFirstSparse;
        let sim = seed_simulation(cfg, 42);
        let founder = sim.creatures.values().next().expect("seeded founder");
        let BackendDef::Vm(vm) = &founder.genome.nodes[1].backend_def else {
            panic!("node 1 should be VM backend");
        };

        let first_eat = vm
            .program
            .iter()
            .position(|instr| matches!(instr, VmInstruction::PushAction { action_type: 1 }))
            .expect("eat action should exist");
        let first_reproduce = vm
            .program
            .iter()
            .position(|instr| matches!(instr, VmInstruction::PushAction { action_type: 3 }))
            .expect("reproduce action should exist");

        assert!(
            first_eat < first_reproduce,
            "forage-first founder should prioritize eat before reproduce"
        );
    }

    #[test]
    fn seeding_threads_min_reproduce_age_into_founder_gate() {
        let mut cfg = small_config();
        cfg.energy.lifecycle.min_reproduce_age = 25;
        let sim = seed_simulation(cfg, 42);
        let founder = sim.creatures.values().next().expect("seeded founder");
        let BackendDef::Graph(graph) = &founder.genome.nodes[0].backend_def else {
            panic!("node 0 should be graph backend");
        };
        assert_eq!(
            graph.compute_nodes[1].kind,
            crate::creature::genome::cgp::ComputeNodeKind::Threshold(24.5)
        );
    }
}

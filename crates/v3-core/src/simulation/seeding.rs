use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use slotmap::{SecondaryMap, SlotMap};

use crate::config::{OrdinaryFoodTypeId, PatternBounds, SimulationConfig};
use crate::contracts::{CreatureId, Direction, InputReference, Position, WorldInputKey};
use crate::creature::action_log::ActionLog;
use crate::creature::body::HeritableBody;
use crate::creature::founder::founder_genome_with_age_gate;
use crate::creature::genome::CreatureGenome;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::patterns::generate_pattern_seeded;
use crate::simulation::simulation::Simulation;

/// Seed a new simulation from a `SimulationConfig` and a deterministic seed.
///
/// Steps per v3-startup-seeding-spec.md Section 4:
/// 1. Seed RNG from `seed`.
/// 2. Create an empty world with dimensions and edge mode from config.
/// 3. Apply terrain and seed fertility from independent map/layer seeds.
/// 4. Seed food from the run RNG.
/// 5. Collect all non-barrier cells, shuffle, take the first `initial_creatures` positions.
/// 6. Place a founder creature at each selected position, drawing its
///    heading uniformly over `Direction::ALL` from the run RNG (T11.F30).
/// 7. Return the assembled `Simulation`.
pub fn seed_simulation(config: SimulationConfig, seed: u64) -> Simulation {
    seed_simulation_with_founder_genomes(config, seed, |_, _| {})
}

/// [`seed_simulation`] with `edit` applied to founder `i`'s genome (in
/// placement order) before that founder is constructed, so every birth cache
/// derives from the edited genome. `edit` draws nothing: positions, headings
/// and bodies are those of [`seed_simulation`]. For fixtures and benchmarks.
pub fn seed_simulation_with_founder_genomes(
    config: SimulationConfig,
    seed: u64,
    mut edit: impl FnMut(usize, &mut CreatureGenome),
) -> Simulation {
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut world = WorldState::new(
        config.world.width,
        config.world.height,
        config.world.edge_mode,
    );
    let map_seed = config.world.world_seed.unwrap_or(seed);
    for (index, layer) in config.world.terrain.iter().enumerate() {
        let bounds = layer
            .bounds
            .unwrap_or(PatternBounds {
                x: 0,
                y: 0,
                width: world.width,
                height: world.height,
            })
            .clipped_to(world.width, world.height);
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
        let mut genome = founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        edit(founder_index, &mut genome);
        let energy = config.energy.lifecycle.initial_energy;
        // One draw per founder, in placement order (T11.F30).
        let heading = Direction::ALL[rng.gen_range(0..Direction::ALL.len())];
        // Insert into slotmap to get an id, then fill with actual state.
        let id = creatures.insert_with_key(|id| {
            let mut founder = CreatureState::new(
                id,
                genome,
                pos,
                energy,
                0,
                HeritableBody::founder(
                    founder_index,
                    seed,
                    config.energy.capacity.founder,
                    config.exchange.founder_share,
                ),
                [0.0; crate::contracts::SHARED_MEMORY_SLOTS],
            );
            founder.heading = heading;
            founder
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
        food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(seed),
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
    seed_simulation_with_founder_genomes(config, seed, |founder_index, genome| {
        if founder_index < perception_creatures {
            inject_extended_perception_input(genome);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FounderProfile, SimulationConfig};
    use crate::creature::genome::sensor_analysis::perception_requirements;
    use crate::creature::genome::BackendDef;
    use crate::creature::phenotype::Phenotype;
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
    fn founder_headings_are_drawn_over_every_direction_and_reproducible() {
        let mut cfg = small_config();
        cfg.population.initial_creatures = 200;
        let headings = |seed| {
            let sim = seed_simulation(cfg.clone(), seed);
            let mut by_position: Vec<_> = sim
                .creatures
                .values()
                .map(|c| (c.position, c.heading))
                .collect();
            by_position.sort_by_key(|(p, _)| (p.y, p.x));
            by_position
        };
        let first = headings(42);
        assert_eq!(first, headings(42));
        let seen: HashSet<Direction> = first.iter().map(|(_, h)| *h).collect();
        assert_eq!(seen.len(), Direction::ALL.len());
    }

    #[test]
    fn founder_phenotype_baseline() {
        let sim = seed_simulation(small_config(), 42);
        // All founders should have the canonical baseline phenotype.
        for (_, creature) in &sim.creatures {
            assert_eq!(creature.body.phenotype, Phenotype::FOUNDER);
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
            .map(|c| c.body.identity.lineage_id)
            .collect();
        lineage_ids.sort();
        let expected: Vec<u32> = (0..lineage_ids.len() as u32).collect();
        assert_eq!(lineage_ids, expected);
    }

    #[test]
    fn founder_kin_tags_are_unique() {
        let sim = seed_simulation(small_config(), 42);
        let kin_tags: HashSet<u32> = sim
            .creatures
            .values()
            .map(|c| c.body.identity.kin_tag)
            .collect();
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
        let mut ids1: Vec<_> = s1.creatures.values().map(|c| c.body.identity).collect();
        let mut ids2: Vec<_> = s2.creatures.values().map(|c| c.body.identity).collect();
        ids1.sort_by_key(|i| i.lineage_id);
        ids2.sort_by_key(|i| i.lineage_id);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn founder_baseline_does_not_use_extended_perception() {
        let founder = founder_genome_with_age_gate(
            small_config().population.founder_profile,
            &small_config().energy.lifecycle,
        );
        assert!(!perception_requirements(&founder).extended_perception);
        let sim = seed_simulation(small_config(), 42);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| creature.perception_requirements().extended_perception)
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
                .filter(|creature| creature.perception_requirements().extended_perception)
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

    /// A mixed founder is built from its edited genome: every genome-derived
    /// cache equals the analysis of the genome it carries.
    #[test]
    fn perception_mix_caches_derive_from_the_edited_genome() {
        use crate::creature::genome::analysis::mesh_reachable_nodes;
        use crate::creature::genome::sensor_analysis::{
            declared_world_keys, has_barrier_reader, live_vm_world_reads, sensor_census,
        };
        use crate::runtime::state::GraphRuntimeState;
        use crate::sensors::reads::WorldKeySet;
        let sim = seed_simulation_with_perception_mix(small_config(), 42, 2);
        for creature in sim.creatures.values() {
            let genome = &creature.genome;
            let reachable = mesh_reachable_nodes(genome);
            assert_eq!(*creature.cached_reachable_nodes, *reachable);
            assert_eq!(creature.cached_complexity, genome.complexity());
            assert_eq!(creature.cached_genome_size, genome.genome_size());
            assert_eq!(
                creature.cached_has_barrier_reader,
                has_barrier_reader(genome, &reachable)
            );
            assert_eq!(
                creature.cached_live_vm_world_inputs,
                live_vm_world_reads(genome, &reachable)
            );
            assert_eq!(
                creature.perception_requirements(),
                perception_requirements(genome)
            );
            let declared: WorldKeySet = declared_world_keys(genome, &reachable)
                .into_iter()
                .collect();
            let wired: WorldKeySet = sensor_census(genome, &reachable)
                .world_inputs
                .into_iter()
                .collect();
            assert_eq!(creature.cached_declared_world_inputs, declared);
            assert_eq!(creature.cached_wired_world_inputs, wired);
            assert_eq!(
                creature.graph_runtime.sink_weights,
                GraphRuntimeState::for_genome(genome, Vec::new()).sink_weights
            );
        }
    }

    /// The hook draws nothing: positions, headings and bodies are those of
    /// the unedited seeding.
    #[test]
    fn the_founder_genome_hook_leaves_every_draw_unchanged() {
        let plain = seed_simulation(small_config(), 42);
        let mixed = seed_simulation_with_perception_mix(small_config(), 42, 3);
        let draws = |sim: &Simulation| {
            sim.creatures
                .values()
                .map(|c| (c.position, c.heading, c.body.identity, c.energy))
                .collect::<Vec<_>>()
        };
        assert_eq!(draws(&plain), draws(&mixed));
        assert_eq!(plain.world.total_food(), mixed.world.total_food());
    }

    #[test]
    fn perception_mix_clamps_to_population_size() {
        let sim = seed_simulation_with_perception_mix(small_config(), 42, 99);
        assert_eq!(
            sim.creatures
                .values()
                .filter(|creature| creature.perception_requirements().extended_perception)
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
                .filter(|creature| creature.perception_requirements().extended_perception)
                .count(),
            200
        );
    }

    #[test]
    fn seeding_uses_configured_founder_profile() {
        let mut cfg = small_config();
        cfg.population.founder_profile = FounderProfile::ForageFirstSparse;
        let expected = crate::creature::founder::founder_genome_with_age_gate(
            FounderProfile::ForageFirstSparse,
            &cfg.energy.lifecycle,
        );
        let sim = seed_simulation(cfg, 42);
        let founder = sim.creatures.values().next().expect("seeded founder");
        assert_eq!(founder.genome, expected);
        assert_ne!(
            founder.genome,
            crate::creature::founder::v3alpha1_founder_genome(),
            "the forage-first decision graph differs from V3Alpha1's"
        );
    }

    /// The founder's age gate is `(min_reproduce_age - 0.5) /
    /// age_reference_ticks`, both from the seeded config (T17.F02 invariant 5).
    #[test]
    fn seeding_threads_min_reproduce_age_and_reference_span_into_founder_gate() {
        let mut cfg = small_config();
        cfg.energy.lifecycle.min_reproduce_age = 25;
        cfg.energy.lifecycle.age_reference_ticks = 1_000;
        let sim = seed_simulation(cfg, 42);
        let founder = sim.creatures.values().next().expect("seeded founder");
        let BackendDef::Graph(graph) = &founder.genome.nodes[0].backend_def else {
            panic!("node 0 should be graph backend");
        };
        assert_eq!(
            graph.compute_nodes()[1].kind,
            crate::creature::genome::cgp::ComputeNodeKind::Threshold(24.5 / 1_000.0)
        );
    }
}

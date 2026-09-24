use serde_json::json;
use v3_core::config::{FoodTypeConfig, SimulationConfig};
use v3_core::patterns::{generate_pattern_seeded, PatternBounds, PatternParams};

#[test]
fn food_overrides_and_fertile_placement_roundtrip() {
    let value = json!({"name":"fruit", "color":"#ff0000", "initial_density":1.0,
        "initial_coverage":0.1, "energy_per_unit":12.0, "growth_rate":0.01,
        "recovery_spawn_rate":0.0, "initial_fertility_only":true});
    let food: FoodTypeConfig = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(food.energy_per_unit, Some(12.0));
    assert_eq!(food.growth_rate, Some(0.01));
    assert_eq!(food.recovery_spawn_rate, Some(0.0));
    assert!(food.initial_fertility_only);
    let decoded: FoodTypeConfig =
        serde_json::from_value(serde_json::to_value(&food).unwrap()).unwrap();
    assert_eq!(decoded, food);
}

#[test]
fn fbm_threshold_extremes_and_translation() {
    let bounds = PatternBounds {
        x: 12,
        y: 31,
        width: 16,
        height: 17,
    };
    let params: PatternParams = serde_json::from_value(json!({"pattern_type":"FbmThreshold",
        "octaves":4,"frequency":0.02,"lacunarity":2.0,"persistence":0.5,"threshold":1.0}))
    .unwrap();
    assert!(generate_pattern_seeded(bounds, &params, 42).is_empty());
}

#[test]
fn fbm_threshold_is_strict_at_zero_and_clips_coordinate_edges() {
    let mut params = PatternParams::FbmThreshold {
        octaves: 4,
        frequency: 0.02,
        lacunarity: 2.0,
        persistence: 0.5,
        threshold: 0.0,
    };
    // Perlin fBm is exactly zero at world (0, 0), which a strict `>` excludes.
    let origin = PatternBounds {
        x: 0,
        y: 0,
        width: 4,
        height: 5,
    };
    let cells = |params: &PatternParams| {
        generate_pattern_seeded(origin, params, 42)
            .into_iter()
            .map(|point| (point.x, point.y))
            .collect::<std::collections::BTreeSet<_>>()
    };
    let at_zero = cells(&params);
    assert!(
        !at_zero.contains(&(0, 0)),
        "the world origin samples exactly zero, which threshold 0.0 excludes"
    );
    let bounds = PatternBounds {
        x: u16::MAX,
        y: u16::MAX,
        width: 4,
        height: 5,
    };
    if let PatternParams::FbmThreshold { threshold, .. } = &mut params {
        *threshold = -1.0;
    }
    // The same cell is included once the threshold drops below its value, so
    // the exclusion above is strictness at zero and not an empty layer.
    let below_zero = cells(&params);
    assert!(below_zero.contains(&(0, 0)));
    assert!(below_zero.is_superset(&at_zero));
    let points = generate_pattern_seeded(bounds, &params, 42);
    assert_eq!(points.len(), 1);
    assert_eq!((points[0].x, points[0].y), (u16::MAX, u16::MAX));
    let bounds = PatternBounds {
        x: u16::MAX - 1,
        y: u16::MAX - 2,
        width: 4,
        height: 5,
    };
    let points = generate_pattern_seeded(bounds, &params, 42);
    let actual: std::collections::BTreeSet<_> =
        points.iter().map(|point| (point.x, point.y)).collect();
    let expected = (u16::MAX - 1..=u16::MAX)
        .flat_map(|x| (u16::MAX - 2..=u16::MAX).map(move |y| (x, y)))
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(points.len(), 6);
}

/// The hash below pins the short production-default trajectory after two runs
/// agree. Intentional changes to production defaults must update it only after
/// the new trajectory has been reproduced. Re-pinned by T19.F05 (the input
/// reference draw grows from 22 to 27 entries, remapping mutated births) and
/// by T11.F25 (the `AddGraphEdge` surface draw skips the undecoded parameter
/// sinks; restoring that draw alone restores the old hash).
#[test]
fn legacy_default_short_run_identity() {
    use std::hash::{Hash, Hasher};
    use v3_core::contracts::Position;
    use v3_core::simulation::{run_tick, seed_simulation};
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 48;
    cfg.world.height = 48;
    cfg.population.initial_creatures = 100;
    let mut sim = seed_simulation(cfg, 11);
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for tick in 0..=30 {
        for y in 0..48 {
            for x in 0..48 {
                sim.world
                    .food_at(Position::new(x, y))
                    .to_bits()
                    .hash(&mut hash);
            }
        }
        for c in sim.creatures.values() {
            format!("{:?}", (c.position, c.energy.to_bits(), c.age, &c.genome)).hash(&mut hash);
        }
        if tick < 30 {
            run_tick(&mut sim, &mut None);
        }
    }
    assert_eq!(hash.finish(), 7552184245034446484);
}

use proptest::prelude::*;
use v3_core::config::{FertilityAlgorithm, FertilityLayer, OrdinaryFoodTypeId};
use v3_core::contracts::Position;
use v3_core::simulation::seed_simulation;

/// Side of the small world the fBm translation property samples.
const WORLD: u16 = 32;

proptest! {
    #[test]
    fn override_normalization_and_roundtrip(value in any::<f32>(), shared in 0.0f32..1.0) {
        let mut config = SimulationConfig::default();
        config.world.food.types[0].energy_per_unit = Some(value);
        config.world.food.types[0].growth_rate = Some(value);
        config.world.food.types[0].recovery_spawn_rate = Some(value);
        config.normalize();
        let food = &config.world.food.types[0];
        let reward = value.is_finite().then(|| value.max(0.0));
        let rate = value.is_finite().then(|| value.clamp(0.0,1.0));
        prop_assert_eq!(food.energy_per_unit, reward);
        prop_assert_eq!(food.growth_rate, rate);
        prop_assert_eq!(food.recovery_spawn_rate, rate);
        let decoded: FoodTypeConfig = serde_json::from_value(serde_json::to_value(food).unwrap()).unwrap();
        prop_assert_eq!(&decoded, food);
        prop_assert_eq!(food.growth_rate.unwrap_or(shared), rate.unwrap_or(shared));
    }

    #[test]
    fn fertile_coverage_is_exact(coverage in 0.0f32..=1.0, seed in any::<u64>(), enabled in any::<bool>()) {
        let mut config = SimulationConfig::default();
        config.world.width = 12;
        config.world.height = 11;
        config.population.initial_creatures = 0;
        config.world.food.types[0].initial_fertility_only = true;
        config.world.food.types[0].initial_coverage = coverage;
        config.world.food.fertility.enabled = enabled;
        config.world.food.fertility.layers = vec![FertilityLayer { algorithm: FertilityAlgorithm::Uniform { value: -1.0 }, ..FertilityLayer::default() }];
        let sim = seed_simulation(config, seed);
        let occupied = (0..11).flat_map(|y| (0..12).map(move |x| Position::new(x,y))).filter(|p| sim.world.food_at(*p) > 0.0).count();
        prop_assert_eq!(occupied, if enabled {0} else {(coverage * 132.0).round() as usize});
    }

    #[test]
    fn fbm_unique_in_bounds_and_threshold_monotone(seed in any::<u64>(), a in -1.0f32..1.0, b in -1.0f32..1.0, x in any::<u16>(), y in any::<u16>()) {
        let bounds = PatternBounds { x, y, width: 13, height: 12 };
        let make = |threshold| PatternParams::FbmThreshold {octaves:4, frequency:0.1,lacunarity:2.0,persistence:0.5,threshold};
        let low = generate_pattern_seeded(bounds, &make(a.min(b)), seed);
        let high = generate_pattern_seeded(bounds, &make(a.max(b)), seed);
        let set: std::collections::BTreeSet<_> = low.iter().map(|p|(p.x,p.y)).collect();
        prop_assert_eq!(set.len(), low.len());
        for p in &low { prop_assert!(p.x >= x && p.y >= y && u32::from(p.x) < u32::from(x)+13 && u32::from(p.y) < u32::from(y)+12); }
        for p in high { prop_assert!(set.contains(&(p.x,p.y))); }
        prop_assert_eq!(low, generate_pattern_seeded(bounds, &make(a.min(b)), seed));
    }

    /// The noise field is a property of the world, not of a layer's bounds: a
    /// bounded layer must equal the whole-world layer at the same seed cut to
    /// that rectangle, so nested same-seed layers grade one region.
    #[test]
    fn fbm_bounded_layer_equals_the_whole_world_layer_inside_its_bounds(
        seed in any::<u64>(), threshold in -0.5f32..0.5,
        x in 0..WORLD, y in 0..WORLD, width in 1..2 * WORLD, height in 1..2 * WORLD,
    ) {
        let params = PatternParams::FbmThreshold {octaves:4, frequency:0.1, lacunarity:2.0, persistence:0.5, threshold};
        // Clip to the world exactly as `seed_simulation` clips a layer's bounds.
        let bounds = PatternBounds { x, y, width: width.min(WORLD - x), height: height.min(WORLD - y) };
        let whole = PatternBounds { x: 0, y: 0, width: WORLD, height: WORLD };
        let cells = |bounds| generate_pattern_seeded(bounds, &params, seed)
            .into_iter().map(|p| (p.x, p.y)).collect::<std::collections::BTreeSet<_>>();
        let inside = cells(whole).into_iter()
            .filter(|(px, py)| (bounds.x..bounds.x + bounds.width).contains(px)
                && (bounds.y..bounds.y + bounds.height).contains(py))
            .collect::<std::collections::BTreeSet<_>>();
        prop_assert_eq!(cells(bounds), inside);
    }
}

#[test]
fn typed_rewards_inherit_live_values_and_apply_cap_before_cost() {
    use v3_core::simulation::actions::apply_typed_eat;
    let mut config = SimulationConfig::default();
    config.world.width = 4;
    config.world.height = 4;
    config.population.initial_creatures = 1;
    config.world.food.types.push(FoodTypeConfig::default());
    config.energy.costs.eat_cost = 2.0;
    config.energy.complexity_cost.enabled = false;
    let mut sim = seed_simulation(config, 11);
    let id = sim.creatures.keys().next().unwrap();
    let pos = sim.creatures[id].position;
    for (reward, shared, expected) in [
        (None, 4.0, 10.0),
        (None, 8.0, 12.0),
        (Some(20.0), 8.0, 18.0),
        (Some(0.0), 8.0, 8.0),
    ] {
        sim.config.world.food.types[1].energy_per_unit = reward;
        sim.config.energy.costs.eat_reward_per_food = shared;
        sim.creatures[id].energy = 10.0;
        sim.world
            .set_food_type(pos, OrdinaryFoodTypeId::new(1), 0.5);
        assert!(apply_typed_eat(
            &mut sim.creatures[id],
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::new(1),
            &mut sim.stats.energy_flows
        ));
        assert_eq!(sim.creatures[id].energy, expected);
    }
    sim.config.world.food.types[1].energy_per_unit = Some(1000.0);
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(1), 1.0);
    assert!(apply_typed_eat(
        &mut sim.creatures[id],
        &mut sim.world,
        &sim.config,
        OrdinaryFoodTypeId::new(1),
        &mut sim.stats.energy_flows
    ));
    assert_eq!(
        sim.creatures[id].energy,
        sim.config.energy.lifecycle.max_energy - 2.0
    );
    let before = sim.creatures[id].energy;
    assert!(!apply_typed_eat(
        &mut sim.creatures[id],
        &mut sim.world,
        &sim.config,
        OrdinaryFoodTypeId::new(99),
        &mut sim.stats.energy_flows
    ));
    assert_eq!(sim.creatures[id].energy, before - 2.0);
}

#[test]
fn per_type_growth_and_recovery_keep_inherited_updates_live() {
    use rand::{rngs::SmallRng, SeedableRng};
    use v3_core::{
        config::{FoodConfig, WorldEdgeMode},
        kernel::WorldState,
    };
    let mut food = FoodConfig::default();
    food.fertility.enabled = false;
    food.shared.occupancy_depletion.enabled = false;
    food.shared.growth_rate = 0.1;
    food.shared.recovery_spawn_rate = 0.0;
    food.types = vec![
        FoodTypeConfig {
            growth_inhibitor: 0.0,
            ..FoodTypeConfig::default()
        },
        FoodTypeConfig {
            growth_rate: Some(0.2),
            growth_inhibitor: 0.0,
            ..FoodTypeConfig::default()
        },
    ];
    let mut world = WorldState::new(1, 1, WorldEdgeMode::Bounded);
    world.reconfigure_food(food.clone());
    let pos = Position::new(0, 0);
    let mut rng = SmallRng::seed_from_u64(4);
    for shared in [0.1, 0.3] {
        food.shared.growth_rate = shared;
        world.reconfigure_food(food.clone());
        for idx in 0..2 {
            world.set_food_type(pos, OrdinaryFoodTypeId::new(idx), 0.5);
        }
        let summary = world.grow_food(0, &mut rng);
        assert_eq!(
            world.food_at_type(pos, OrdinaryFoodTypeId::new(0)),
            0.5 + 0.5 * shared
        );
        assert_eq!(world.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 0.6);
        assert_eq!(summary.per_type[1].total_density, 0.6);
    }
    // A one-cell world makes each recovery attempt observable and deterministic.
    food.shared.recovery_floor_ratio = 1.0;
    food.shared.recovery_spawn_rate = 1.0;
    food.types[0].growth_rate = Some(0.0);
    food.types[1].recovery_spawn_rate = Some(0.0);
    world.reconfigure_food(food.clone());
    world.grow_food(0, &mut rng);
    assert_eq!(world.food_at(pos), 0.0);
    food.types[0].growth_rate = Some(0.25);
    food.types[1].recovery_spawn_rate = Some(1.0);
    world.reconfigure_food(food);
    world.grow_food(0, &mut rng);
    assert_eq!(world.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 0.25);
    assert_eq!(world.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 0.2);
}

#[test]
fn fertile_placement_respects_annealing_and_barriers() {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 8;
    cfg.world.height = 8;
    cfg.population.initial_creatures = 0;
    cfg.world.food.types[0].initial_fertility_only = true;
    cfg.world.food.types[0].initial_coverage = 1.0;
    cfg.world.food.fertility.layers = vec![FertilityLayer {
        algorithm: FertilityAlgorithm::Uniform { value: -1.0 },
        ..FertilityLayer::default()
    }];
    cfg.world.food.annealing.enabled = true;
    cfg.world.food.annealing.initial_min_fertility = 0.3;
    cfg.world.terrain.push(v3_core::config::TerrainLayer {
        params: PatternParams::Noise {
            density: 1.0,
            cluster_size: 1,
        },
        bounds: Some(PatternBounds {
            x: 0,
            y: 0,
            width: 4,
            height: 8,
        }),
        seed: Some(9),
    });
    let sim = seed_simulation(cfg, 11);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(
                sim.world.food_at(Position::new(x, y)) > 0.0,
                !sim.world.is_barrier(Position::new(x, y))
            );
        }
    }
}

#[test]
fn absent_and_null_overrides_inherit_but_zero_is_retained() {
    for extra in [
        json!({}),
        json!({"energy_per_unit":null,"growth_rate":null,"recovery_spawn_rate":null}),
    ] {
        let mut value =
            json!({"name":"grass","color":"#00ff00","initial_density":1.0,"initial_coverage":0.5});
        value
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let food: FoodTypeConfig = serde_json::from_value(value).unwrap();
        assert_eq!(food.energy_per_unit, None);
        assert_eq!(food.growth_rate, None);
        assert_eq!(food.recovery_spawn_rate, None);
        assert!(!food.initial_fertility_only);
    }
}

#[test]
fn connectivity_uses_diagonals_and_world_edges() {
    use v3_core::{config::WorldEdgeMode, kernel::WorldState};
    for (mode, expected) in [(WorldEdgeMode::Bounded, 1), (WorldEdgeMode::Wrap, 2)] {
        let mut world = WorldState::new(5, 1, mode);
        for x in 1..4 {
            world.set_barrier(Position::new(x, 0), true);
        }
        let reading = world.passable_connectivity();
        assert_eq!(reading.total_cells, 5);
        assert_eq!(reading.passable_cells, 2);
        assert_eq!(reading.largest_component_cells, expected);
        assert_eq!(reading.passable_fraction, 0.4);
        assert_eq!(
            reading.largest_component_fraction_of_passable,
            expected as f64 / 2.0
        );
    }
    let mut world = WorldState::new(2, 2, WorldEdgeMode::Bounded);
    world.set_barrier(Position::new(1, 0), true);
    world.set_barrier(Position::new(0, 1), true);
    assert_eq!(world.passable_connectivity().largest_component_cells, 2);
    world.set_barrier(Position::new(0, 0), true);
    world.set_barrier(Position::new(1, 1), true);
    assert_eq!(world.passable_connectivity().largest_component_cells, 0);
    assert_eq!(
        world
            .passable_connectivity()
            .largest_component_fraction_of_passable,
        0.0
    );
}

proptest! {
    #[test]
    fn connectivity_bounds_ignore_occupancy(barriers in prop::collection::vec(any::<bool>(),64), wrapped in any::<bool>()) {
        use v3_core::config::WorldEdgeMode;
        let mut cfg=SimulationConfig::default(); cfg.world.width=8;cfg.world.height=8;cfg.population.initial_creatures=10;
        cfg.world.edge_mode=if wrapped {WorldEdgeMode::Wrap} else {WorldEdgeMode::Bounded};
        let mut sim=seed_simulation(cfg,5);
        for (index, barrier) in barriers.iter().enumerate() {sim.world.set_barrier(Position::new((index%8) as u16,(index/8) as u16),*barrier);}
        let reading=sim.world.passable_connectivity();
        prop_assert_eq!(reading.passable_cells,barriers.iter().filter(|v| !**v).count() as u64);
        prop_assert!(reading.largest_component_cells<=reading.passable_cells);
        prop_assert!((0.0..=1.0).contains(&reading.passable_fraction));
        prop_assert!((0.0..=1.0).contains(&reading.largest_component_fraction_of_passable));
        for c in sim.creatures.values() {sim.world.remove_creature(c.position);}
        prop_assert_eq!(sim.world.passable_connectivity(),reading);
    }
    #[test]
    fn fbm_normalization_is_finite_and_idempotent(octaves in any::<u32>(), f in any::<f32>(), l in any::<f32>(), p in any::<f32>(), t in any::<f32>()) {
        let mut params=PatternParams::FbmThreshold {octaves,frequency:f,lacunarity:l,persistence:p,threshold:t};
        params.normalize();
        let PatternParams::FbmThreshold {octaves,frequency,lacunarity,persistence,threshold}=&params else {unreachable!()};
        prop_assert!((1..=32).contains(octaves));
        for (actual,input,min,max,fallback) in [(*frequency,f,0.000001,1.0,0.02),(*lacunarity,l,1.0,4.0,2.0),(*persistence,p,0.0,1.0,0.5),(*threshold,t,-1.0,1.0,0.0)] {
            prop_assert_eq!(actual,if input.is_finite(){input.clamp(min,max)}else{fallback});
        }
        let first=serde_json::to_value(&params).unwrap();params.normalize();prop_assert_eq!(serde_json::to_value(params).unwrap(),first);
    }
}

#[test]
fn saved_world_recipes_are_partial_configs_that_pin_their_maps() {
    use v3_core::config::resolve_config;
    for (name, source) in RECIPES {
        let patch: serde_json::Value = serde_json::from_str(source).unwrap();
        let cfg = resolve_config(&SimulationConfig::default(), patch.clone())
            .unwrap_or_else(|error| panic!("{name} must load: {error}"));
        // The goal profile applies its own world size and founder count, so a
        // recipe that set either would be silently overridden.
        assert_eq!(patch.pointer("/world/width"), None, "{name} sets a width");
        assert_eq!(patch.pointer("/world/height"), None, "{name} sets a height");
        assert_eq!(
            patch.pointer("/population"),
            None,
            "{name} sets a population"
        );
        assert_eq!(cfg.population.initial_creatures, 10_000, "{name}");
        assert_eq!(cfg.world.width, 1600, "{name}");
        if name == "plains" {
            assert!(cfg.world.terrain.is_empty(), "plains carries no terrain");
            continue;
        }
        assert!(
            cfg.world.world_seed.is_some(),
            "{name} must pin its map so every closure runs the same world"
        );
    }
}

/// Orchards' fruit is the latent niche: richer per unit, confined to patches
/// of the world rather than spread across it, and quicker to come back inside
/// those patches than the grass the founders live on. Grass itself must reach
/// every passable cell, so no lineage is starved by where it happened to land.
#[test]
fn orchards_fruit_is_richer_patchier_and_faster_regrowing_than_grass() {
    let cfg = baseline_config("orchards-in-grassland");
    let grass = &cfg.world.food.types[0];
    let fruit = &cfg.world.food.types[1];
    assert!(
        fruit.energy_per_unit > grass.energy_per_unit,
        "fruit {:?} must be richer than grass {:?}",
        fruit.energy_per_unit,
        grass.energy_per_unit
    );
    assert!(
        fruit.growth_rate > grass.growth_rate,
        "fruit {:?} must regrow faster than grass {:?}",
        fruit.growth_rate,
        grass.growth_rate
    );
    assert!(
        fruit.initial_fertility_only,
        "fruit must be confined to its own fertile patches"
    );

    // Patchiness is a property of the applied map, not of `initial_coverage`:
    // a fertility-confined type's coverage applies only inside its patches.
    let sim = seed_baseline_map(cfg);
    let fertility: Vec<_> = (0..2)
        .map(|index| {
            sim.world
                .food()
                .effective_fertility_grid(OrdinaryFoodTypeId::new(index), 0)
        })
        .collect();
    let mut fertile = [0u64; 2];
    let mut passable = 0u64;
    for y in 0..sim.world.height {
        for x in 0..sim.world.width {
            if sim.world.is_barrier(Position::new(x, y)) {
                continue;
            }
            passable += 1;
            for (index, count) in fertile.iter_mut().enumerate() {
                *count += u64::from(*fertility[index].get(x, y) > 0.0);
            }
        }
    }
    assert_eq!(
        fertile[0], passable,
        "grass fertility must be positive on every one of the {passable} passable cells"
    );
    assert!(
        fertile[1] < passable,
        "fruit reaches {} of {passable} passable cells, so it is not patchy",
        fertile[1]
    );
}

/// Canyon country is a looped landscape, not a maze: nearly every passable
/// cell stays reachable from every other, while barriers are frequent enough
/// to block moves often.
#[test]
fn canyon_country_is_well_looped_and_substantially_walled() {
    let sim = seed_baseline_map(baseline_config("canyon-country"));
    let connectivity = sim.world.passable_connectivity();
    assert!(
        connectivity.largest_component_fraction_of_passable >= 0.95,
        "largest component holds {} of passable cells",
        connectivity.largest_component_fraction_of_passable
    );
    assert!(
        1.0 - connectivity.passable_fraction >= 0.15,
        "barriers cover {} of the world",
        1.0 - connectivity.passable_fraction
    );
    assert_eq!(
        sim.config.world.food.types.len(),
        1,
        "canyon country keeps the production one-food substrate"
    );
}

/// The composite carries both pressures at once.
#[test]
fn confluence_carries_both_foods_bounded_edges_and_terrain() {
    let cfg = baseline_config("confluence");
    assert_eq!(cfg.world.food.types.len(), 2);
    assert_eq!(cfg.world.edge_mode, v3_core::config::WorldEdgeMode::Bounded);
    assert!(!cfg.world.terrain.is_empty());
}

/// Explicit one-time feature diagnostic; not a production indicator or a score gate.
#[test]
#[ignore = "bounded feature diagnostic, run explicitly and store its readings"]
fn food_choice_mutation_diagnostic() {
    use rand::{rngs::SmallRng, SeedableRng};
    use v3_core::{
        contracts::WorldAction,
        creature::{founder::founder_genome, genome::analysis::mesh_reachable_nodes},
        mutation::{reachability::ParentExecuted, MutationEngine},
        neighborhood::Battery,
    };
    let mut cfg = SimulationConfig::default();
    cfg.world.food.types.push(FoodTypeConfig::default());
    let parent = founder_genome(cfg.population.founder_profile);
    let battery = Battery::generate(2);
    let base = battery.signature(&parent, &cfg.runtime, cfg.shared_memory.decay_rate);
    let executed = battery.executed_indices(&parent, &cfg.runtime, cfg.shared_memory.decay_rate);
    let reachable = mesh_reachable_nodes(&parent);
    let mut zero = 0;
    let mut changed_eat = 0;
    let mut nonprimary = 0;
    for birth in 0..1000u64 {
        let mut child = parent.clone();
        let mut rng = SmallRng::seed_from_u64(9000 + birth);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut child,
            &cfg.mutation,
            &reachable,
            ParentExecuted::Indices(&executed),
            &mut rng,
            2,
        );
        zero += usize::from(summary.applied_events == 0);
        let signature = battery.signature(&child, &cfg.runtime, cfg.shared_memory.decay_rate);
        let parent_executions = base.snapshots.iter().chain(base.sequences.iter().flatten());
        let child_executions = signature
            .snapshots
            .iter()
            .chain(signature.sequences.iter().flatten());
        changed_eat+=usize::from(parent_executions.zip(child_executions).any(|(p,c)|p.iter().zip(c).any(|(p,c)|matches!((p,c),(WorldAction::Eat{type_idx:a},WorldAction::Eat{type_idx:b}) if a!=b))));
        nonprimary += usize::from(
            signature
                .snapshots
                .iter()
                .chain(signature.sequences.iter().flatten())
                .flatten()
                .any(|a| matches!(a,WorldAction::Eat{type_idx} if type_idx.get()!=0)),
        );
    }
    println!("food-choice diagnostic: births=1000 seeds=9000..9999 food_types=2 executions_per_genome={} executed_parent_nodes={executed:?} zero_event_births={zero} offspring_with_corresponding_eat_index_change={changed_eat} offspring_selecting_nonprimary_anywhere={nonprimary}",base.execution_count());
}

/// The checked-in recipes, by file stem.
const RECIPES: [(&str, &str); 4] = [
    (
        "plains",
        include_str!("../../../experiments/worlds/plains.json"),
    ),
    (
        "orchards-in-grassland",
        include_str!("../../../experiments/worlds/orchards-in-grassland.json"),
    ),
    (
        "canyon-country",
        include_str!("../../../experiments/worlds/canyon-country.json"),
    ),
    (
        "confluence",
        include_str!("../../../experiments/worlds/confluence.json"),
    ),
];

/// One checked-in recipe resolved over the production defaults.
fn baseline_config(name: &str) -> SimulationConfig {
    let (_, source) = RECIPES
        .iter()
        .find(|(stem, _)| *stem == name)
        .unwrap_or_else(|| panic!("{name} must be a checked-in recipe"));
    v3_core::config::resolve_config(
        &SimulationConfig::default(),
        serde_json::from_str(source).unwrap(),
    )
    .unwrap()
}

/// Seed a baseline world's map at production size. The founder count is cut to
/// one: terrain, fertility, and connectivity come from the pinned world seed
/// alone, and placing ten thousand founders would only slow the assertion down.
fn seed_baseline_map(mut cfg: SimulationConfig) -> v3_core::simulation::Simulation {
    cfg.population.initial_creatures = 1;
    seed_simulation(cfg, 11)
}

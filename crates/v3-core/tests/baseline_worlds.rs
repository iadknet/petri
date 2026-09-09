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
    // Perlin fBm is exactly zero at the local origin, which must be excluded.
    let bounds = PatternBounds {
        x: u16::MAX,
        y: u16::MAX,
        width: 4,
        height: 5,
    };
    assert!(generate_pattern_seeded(bounds, &params, 42).is_empty());
    if let PatternParams::FbmThreshold { threshold, .. } = &mut params {
        *threshold = -1.0;
    }
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
    assert_eq!(hash.finish(), 13138541837675773035);
}

use proptest::prelude::*;
use v3_core::config::{FertilityAlgorithm, FertilityLayer, OrdinaryFoodTypeId};
use v3_core::contracts::Position;
use v3_core::simulation::seed_simulation;

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
            OrdinaryFoodTypeId::new(1)
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
        OrdinaryFoodTypeId::new(1)
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
        OrdinaryFoodTypeId::new(99)
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
fn saved_world_recipes_have_distinct_applied_pressures() {
    use v3_core::config::resolve_config;
    let sources = [
        include_str!("../../../experiments/worlds/plains.json"),
        include_str!("../../../experiments/worlds/orchards-in-grassland.json"),
        include_str!("../../../experiments/worlds/canyon-country.json"),
        include_str!("../../../experiments/worlds/confluence.json"),
    ];
    for (index, source) in sources.iter().enumerate() {
        let cfg = resolve_config(
            &SimulationConfig::default(),
            serde_json::from_str(source).unwrap(),
        )
        .unwrap();
        assert_eq!(cfg.population.initial_creatures, 10_000);
        assert_eq!(cfg.world.world_seed, None);
        if index == 1 || index == 3 {
            let grass = &cfg.world.food.types[0];
            let fruit = &cfg.world.food.types[1];
            assert!(fruit.energy_per_unit > grass.energy_per_unit);
            assert!(fruit.initial_coverage < grass.initial_coverage);
            assert!(fruit.growth_rate < grass.growth_rate);
            assert!(fruit.recovery_spawn_rate < grass.recovery_spawn_rate);
            assert!(fruit.initial_fertility_only);
            assert_eq!(cfg.world.food.fertility.min_fertility, 0.0);
        }
        if index == 0 {
            assert!(cfg.world.terrain.is_empty());
        }
        if index == 2 {
            assert!(matches!(
                cfg.world.terrain[0].params,
                PatternParams::Maze {
                    corridor_width: 5,
                    ..
                }
            ));
        }
        if index == 3 {
            assert_eq!(cfg.world.edge_mode, v3_core::config::WorldEdgeMode::Bounded);
        }
    }
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

/// Full-size tick-zero visual inspection, deliberately outside the ordinary test suite.
#[test]
#[ignore = "full-size layout inspection; run explicitly before the single goal measurement"]
fn inspect_saved_world_layouts() {
    use std::io::Write;
    use v3_core::config::resolve_config;
    for (name, seed, source) in [
        (
            "orchards-in-grassland",
            11,
            include_str!("../../../experiments/worlds/orchards-in-grassland.json"),
        ),
        (
            "canyon-country",
            22,
            include_str!("../../../experiments/worlds/canyon-country.json"),
        ),
        (
            "confluence",
            33,
            include_str!("../../../experiments/worlds/confluence.json"),
        ),
    ] {
        let cfg = resolve_config(
            &SimulationConfig::default(),
            serde_json::from_str(source).unwrap(),
        )
        .unwrap();
        let sim = seed_simulation(cfg, seed);
        let mut habitat = [0u64; 4];
        let mut food_cells = [0u64; 2];
        let mut barrier_habitat = 0;
        let mut pixels = Vec::new();
        for y in 0..sim.world.height {
            for x in 0..sim.world.width {
                let pos = Position::new(x, y);
                let grass = *sim
                    .world
                    .food()
                    .fertility_for_type(OrdinaryFoodTypeId::new(0))
                    .unwrap()
                    .get(x, y)
                    > -1.0;
                let fruit = sim
                    .world
                    .food()
                    .fertility_for_type(OrdinaryFoodTypeId::new(1))
                    .is_some_and(|grid| *grid.get(x, y) > -1.0);
                let barrier = sim.world.is_barrier(pos);
                if !barrier {
                    habitat[usize::from(grass) + 2 * usize::from(fruit)] += 1;
                }
                barrier_habitat += u64::from(barrier && (grass || fruit));
                for (idx, count) in food_cells
                    .iter_mut()
                    .enumerate()
                    .take(sim.config.world.food.types.len())
                {
                    *count += u64::from(
                        sim.world
                            .food_at_type(pos, OrdinaryFoodTypeId::new(idx as u16))
                            > 0.0,
                    );
                }
                if x % 4 == 2 && y % 4 == 2 {
                    pixels.extend_from_slice(if barrier {
                        &[180u8, 184, 195]
                    } else {
                        match (grass, fruit) {
                            (true, true) => &[226, 190, 69],
                            (true, false) => &[49, 132, 74],
                            (false, true) => &[231, 108, 42],
                            (false, false) => &[21, 29, 42],
                        }
                    });
                }
            }
        }
        if sim.config.world.food.types.len() == 2 {
            assert!(habitat[3] > 0 && habitat[1] > 0 && habitat[2] > 0);
        }
        let path = format!("/private/tmp/t12-f04-{name}.ppm");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(
            file,
            "P6\n{} {}\n255\n",
            sim.world.width / 4,
            sim.world.height / 4
        )
        .unwrap();
        file.write_all(&pixels).unwrap();
        println!("{name} seed={seed} connectivity={} habitats_neither_grass_fruit_overlap={habitat:?} food_cells={food_cells:?} barriers_overlapping_habitat={barrier_habitat} map={path}",serde_json::to_string(&sim.world.passable_connectivity()).unwrap());
    }
}

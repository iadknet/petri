//! Startup terrain configuration and applied seeding contract.
use serde_json::json;
use v3_core::config::SimulationConfig;
use v3_core::contracts::Position;
use v3_core::simulation::seed_simulation;

#[test]
fn terrain_precedes_food_and_clamps_founders() {
    let mut value = serde_json::to_value(SimulationConfig::default()).unwrap();
    value["world"]["width"] = json!(10);
    value["world"]["height"] = json!(10);
    value["world"]["terrain"] = json!([{
        "params": {"pattern_type":"Noise", "density":1.0, "cluster_size":1},
        "bounds": {"x":0,"y":0,"width":9,"height":10}
    }]);
    let config = serde_json::from_value(value).unwrap();
    let sim = seed_simulation(config, 42);
    assert_eq!(sim.creature_count(), 10);
    for y in 0..10 {
        for x in 0..10 {
            let pos = Position::new(x, y);
            assert_eq!(sim.world.is_barrier(pos), x < 9);
            if x < 9 {
                assert_eq!(sim.world.food_at(pos), 0.0);
            }
        }
    }
    assert!(sim.creatures.values().all(|c| c.position.x == 9));
}

#[test]
fn clustered_noise_trimming_has_reproducible_set() {
    use v3_core::patterns::{generate_pattern_seeded, PatternBounds, PatternParams};
    // A tiny target with a large central cluster forces excess-cell trimming.
    let bounds = PatternBounds {
        x: 0,
        y: 0,
        width: 40,
        height: 40,
    };
    let params = PatternParams::Noise {
        density: 0.001,
        cluster_size: 20,
    };
    let cells = || {
        let mut points: Vec<_> = generate_pattern_seeded(bounds, &params, 42)
            .into_iter()
            .map(|p| (p.x, p.y))
            .collect();
        points.sort_unstable();
        points
    };
    let first = cells();
    assert_eq!(first.len(), 2);
    for _ in 0..8 {
        assert_eq!(first, cells());
    }
}

use proptest::prelude::*;
use std::collections::BTreeSet;
use v3_core::config::TerrainLayer;
use v3_core::patterns::{generate_pattern_seeded, PatternBounds, PatternParams};

fn config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 40;
    cfg.world.height = 40;
    cfg.population.initial_creatures = 12;
    cfg.world.food.fertility.enabled = false;
    cfg
}
fn layer(params: PatternParams, bounds: Option<PatternBounds>, seed: Option<u64>) -> TerrainLayer {
    TerrainLayer {
        params,
        bounds,
        seed,
    }
}
fn noise(density: f32) -> PatternParams {
    PatternParams::Noise {
        density,
        cluster_size: 20,
    }
}
fn barriers(sim: &v3_core::simulation::Simulation) -> BTreeSet<(u16, u16)> {
    (0..sim.world.height)
        .flat_map(|y| (0..sim.world.width).map(move |x| (x, y)))
        .filter(|&(x, y)| sim.world.is_barrier(Position::new(x, y)))
        .collect()
}
fn patterns() -> [PatternParams; 5] {
    [
        PatternParams::Maze {
            corridor_width: 2,
            wall_thickness: 1,
            open_center_radius: 0,
        },
        PatternParams::Spiral {
            arm_count: 3,
            arm_thickness: 2,
            gap_width: 4,
            clockwise: true,
            open_center_radius: 0,
        },
        noise(0.001),
        PatternParams::ParallelLines {
            spacing: 6,
            thickness: 2,
            jaggedness: 0.5,
            angle_degrees: 30.0,
        },
        PatternParams::Star {
            point_count: 2,
            ray_count: 8,
            ray_length: 20,
            ray_thickness: 2,
        },
    ]
}

#[test]
fn absent_fields_preserve_defaults_and_unknown_layers_are_rejected() {
    let mut value = serde_json::to_value(config()).unwrap();
    value["world"].as_object_mut().unwrap().remove("terrain");
    value["world"].as_object_mut().unwrap().remove("world_seed");
    let cfg: SimulationConfig = serde_json::from_value(value.clone()).unwrap();
    assert!(cfg.world.terrain.is_empty());
    assert_eq!(cfg.world.world_seed, None);
    value["world"]["terrain"] =
        json!([{"params":{"pattern_type":"Noise","density":0.1,"cluster_size":2},"typo":1}]);
    assert!(serde_json::from_value::<SimulationConfig>(value).is_err());
    let plain = seed_simulation(config(), 42);
    let absent = seed_simulation(cfg, 42);
    assert_eq!(
        plain
            .creatures
            .values()
            .map(|c| c.position)
            .collect::<Vec<_>>(),
        absent
            .creatures
            .values()
            .map(|c| c.position)
            .collect::<Vec<_>>()
    );
}

#[test]
fn empty_outside_and_all_barrier_terrain_are_valid() {
    for bounds in [
        PatternBounds {
            x: u16::MAX,
            y: 0,
            width: u16::MAX,
            height: 40,
        },
        PatternBounds {
            x: 0,
            y: u16::MAX,
            width: 40,
            height: u16::MAX,
        },
        PatternBounds {
            x: 2,
            y: 2,
            width: 0,
            height: 20,
        },
    ] {
        let mut cfg = config();
        cfg.world.terrain = vec![layer(noise(1.0), Some(bounds), None)];
        assert!(barriers(&seed_simulation(cfg, 0)).is_empty());
    }
    let mut cfg = config();
    cfg.world.terrain = vec![layer(noise(1.0), None, None)];
    let sim = seed_simulation(cfg, 0);
    assert_eq!(barriers(&sim).len(), 1600);
    assert_eq!(sim.creature_count(), 0);
}

#[test]
fn exact_food_coverage_counts_only_passable_cells() {
    let mut cfg = config();
    cfg.world.food.types[0].initial_coverage = 0.5;
    cfg.world.terrain = vec![layer(
        noise(1.0),
        Some(PatternBounds {
            x: 0,
            y: 0,
            width: 20,
            height: 40,
        }),
        None,
    )];
    let sim = seed_simulation(cfg, 8);
    let count = (0..40)
        .flat_map(|y| (0..40).map(move |x| Position::new(x, y)))
        .filter(|&p| sim.world.food_at(p) > 0.0)
        .count();
    assert_eq!(count, 400);
}

#[test]
fn production_sized_startup_has_no_runtime_pattern_area_cap() {
    let mut cfg = config();
    cfg.world.width = 1600;
    cfg.world.height = 1600;
    cfg.population.initial_creatures = 0;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.terrain = vec![layer(
        PatternParams::ParallelLines {
            spacing: 255,
            thickness: 1,
            jaggedness: 0.0,
            angle_degrees: 0.0,
        },
        None,
        Some(1),
    )];
    assert!(!barriers(&seed_simulation(cfg, 0)).is_empty());
}

proptest! {
    #[test]
    fn all_patterns_are_bounded_idempotent_seeded_unions(seed in any::<u64>(), x in 0u16..45, y in 0u16..45, width in any::<u16>(), height in any::<u16>()) {
        for params in patterns() {
            let bounds=PatternBounds{x,y,width,height};
            let mut cfg=config();
            cfg.population.initial_creatures=0;
            cfg.world.world_seed=Some(seed);
            cfg.world.terrain=vec![layer(params.clone(),Some(bounds),Some(seed))];
            let one=barriers(&seed_simulation(cfg.clone(),1));
            let clipped=PatternBounds{x,y,width:((u32::from(x)+u32::from(width)).min(40).saturating_sub(u32::from(x))) as u16,height:((u32::from(y)+u32::from(height)).min(40).saturating_sub(u32::from(y))) as u16};
            let expected:BTreeSet<_>=generate_pattern_seeded(clipped,&params,seed).into_iter().map(|p|(p.x,p.y)).collect();
            prop_assert_eq!(&one,&expected);
            cfg.world.terrain.push(cfg.world.terrain[0].clone());
            let two=barriers(&seed_simulation(cfg,2));
            prop_assert_eq!(&one,&two);
            prop_assert!(one.iter().all(|&(px,py)|px<40 && py<40 && px>=x && py>=y));
            let encoded=serde_json::to_value(layer(params,Some(bounds),Some(seed))).unwrap();
            let decoded:TerrainLayer=serde_json::from_value(encoded.clone()).unwrap();
            prop_assert_eq!(serde_json::to_value(decoded).unwrap(),encoded);
        }
    }
    #[test]
    fn implicit_layer_seed_wraps_and_explicit_seed_wins(seed in any::<u64>()) {
        for map_seed in [seed,u64::MAX] {
            let mut cfg=config();
            cfg.world.world_seed=Some(map_seed);
            cfg.world.terrain=vec![layer(noise(0.0),None,None),layer(noise(0.01),None,None)];
            let actual=barriers(&seed_simulation(cfg.clone(),seed));
            let expected:BTreeSet<_>=generate_pattern_seeded(PatternBounds{x:0,y:0,width:40,height:40},&noise(0.01),map_seed.wrapping_add(1)).into_iter().map(|p|(p.x,p.y)).collect();
            prop_assert_eq!(actual,expected);
            cfg.world.terrain[1].seed=Some(seed);
            let explicit=barriers(&seed_simulation(cfg.clone(),1));
            cfg.world.world_seed=Some(map_seed.wrapping_add(13));
            prop_assert_eq!(explicit,barriers(&seed_simulation(cfg,2)));
        }
    }
    #[test]
    fn clustered_noise_trimming_always_has_exact_reproducible_target(seed in any::<u64>()) {
        let bounds=PatternBounds{x:0,y:0,width:40,height:40};
        let points=||generate_pattern_seeded(bounds,&noise(0.001),seed).into_iter().map(|p|(p.x,p.y)).collect::<BTreeSet<_>>();
        let first=points();
        prop_assert_eq!(first.len(),2);
        prop_assert_eq!(first,points());
    }
}

#[test]
fn overlapping_layers_union_and_empty_terrain_does_not_draw_from_run_rng() {
    let mut cfg = config();
    cfg.world.terrain = vec![
        layer(
            noise(1.0),
            Some(PatternBounds {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            }),
            None,
        ),
        layer(
            noise(1.0),
            Some(PatternBounds {
                x: 5,
                y: 0,
                width: 10,
                height: 10,
            }),
            None,
        ),
    ];
    assert_eq!(barriers(&seed_simulation(cfg, 42)).len(), 150);
    let plain = seed_simulation(config(), 42);
    let mut cfg = config();
    cfg.world.terrain = vec![layer(noise(0.0), None, None)];
    let empty = seed_simulation(cfg, 42);
    assert_eq!(
        plain
            .creatures
            .values()
            .map(|c| c.position)
            .collect::<Vec<_>>(),
        empty
            .creatures
            .values()
            .map(|c| c.position)
            .collect::<Vec<_>>()
    );
    for y in 0..40 {
        for x in 0..40 {
            assert_eq!(
                plain.world.food_at(Position::new(x, y)),
                empty.world.food_at(Position::new(x, y))
            );
        }
    }
    let value = json!({"params":{"pattern_type":"Noise","density":0.0,"cluster_size":1},"bounds":null,"seed":null});
    let decoded: TerrainLayer = serde_json::from_value(value).unwrap();
    assert!(decoded.bounds.is_none() && decoded.seed.is_none());
}

    use std::collections::{BTreeSet, HashSet};

    use crate::ControllerPalette;
    use petri_graph::{Edge, NodeKind};
    use rand::Rng;

    use super::*;

    #[test]
    fn world_initializes_with_creatures_in_bounds() {
        let cfg = WorldConfig {
            width: 40,
            height: 30,
            initial_creatures: 80,
            max_creatures: 500,
            ..WorldConfig::default()
        };

        let world = World::new(cfg, 7);
        assert_eq!(world.creature_count(), 80);
        for c in world.creature_views() {
            assert!(c.x < 40);
            assert!(c.y < 30);
        }
    }

    #[test]
    fn tick_changes_creature_positions_over_time() {
        let cfg = WorldConfig {
            width: 32,
            height: 32,
            initial_creatures: 30,
            max_creatures: 300,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 3);
        let before = world
            .creature_views()
            .into_iter()
            .map(|c| (c.id, (c.x, c.y)))
            .collect::<std::collections::BTreeMap<_, _>>();

        world.tick();

        let after = world
            .creature_views()
            .into_iter()
            .map(|c| (c.id, (c.x, c.y)))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_ne!(before, after);
    }

    #[test]
    fn creatures_lose_energy_and_die() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            energy_initial: 0.05,
            energy_per_tick_decay: 0.02,
            energy_per_move: 0.02,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 9);
        for _ in 0..10 {
            world.tick();
        }
        assert_eq!(world.creature_count(), 0);
    }

    #[test]
    fn creatures_eat_food_and_gain_energy() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            food_energy_value: 1.0,
            min_reproduce_energy: 10.0,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 11);

        let (id, mut creature) = world.creatures.iter().next().unwrap();
        let idx = world.idx(creature.x, creature.y);
        world.cells[idx].food = 1.0;
        let before = creature.energy;

        world.tick();

        creature = world.creatures.get(id).unwrap();
        assert!(creature.energy > before);
    }

    #[test]
    fn creature_eat_action_consumes_all_food_in_cell() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            food_spawn_rate: 0.0,
            min_reproduce_energy: 10.0,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 77, ControllerPalette::LogicOnly);

        let (_id, creature) = world.creatures.iter().next().unwrap();
        let idx = world.idx(creature.x, creature.y);
        world.cells[idx].food = 1.0;

        world.tick();

        assert!(world.cells[idx].food <= f32::EPSILON);
    }

    #[test]
    fn creatures_can_reproduce_when_energy_is_high() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 1,
            max_creatures: 50,
            min_reproduce_energy: 0.6,
            energy_initial: 1.5,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 21);
        let (_id, parent) = world.creatures.iter().next().unwrap();
        let idx = world.idx(parent.x, parent.y);
        world.cells[idx].food = 1.0;
        assert_eq!(world.creature_count(), 1);
        world.tick();
        assert!(world.creature_count() > 1);
    }

    #[test]
    fn world_can_be_created_with_each_controller_palette() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 20,
            max_creatures: 200,
            ..WorldConfig::default()
        };

        for palette in [
            ControllerPalette::NeuralOnly,
            ControllerPalette::LogicOnly,
            ControllerPalette::Hybrid,
        ] {
            let mut world = World::new_with_palette(cfg.clone(), 99, palette);
            world.tick();
            assert!(world.creature_count() > 0);
        }
    }

    #[test]
    fn world_tracks_diagnostics_and_logic_eat_gate() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            food_energy_value: 1.0,
            min_reproduce_energy: 10.0,
            ..WorldConfig::default()
        };

        let mut world = World::new_with_palette(cfg, 123, ControllerPalette::LogicOnly);
        let initial_diag = world.diagnostics();
        assert_eq!(initial_diag.moves, 0);
        assert_eq!(initial_diag.eats, 0);
        assert_eq!(initial_diag.reproductions, 0);
        assert_eq!(initial_diag.deaths, 0);

        let (id, creature) = world.creatures.iter().next().unwrap();
        let idx = world.idx(creature.x, creature.y);
        world.cells[idx].food = 0.0;
        let before = creature.energy;

        world.tick();
        let after_low_food = world.creatures.get(id).unwrap().energy;
        assert!(after_low_food <= before);

        let creature_after = world.creatures.get(id).unwrap();
        let idx2 = world.idx(creature_after.x, creature_after.y);
        world.cells[idx2].food = 1.0;

        world.tick();
        let diag = world.diagnostics();
        assert!(diag.eats > 0);
    }

    fn controller_checksum(controller: &ComputationGraph) -> i64 {
        let node_sum = controller
            .nodes
            .iter()
            .map(|node| match node {
                petri_graph::NodeKind::Constant(v) => (*v * 1000.0) as i64,
                petri_graph::NodeKind::Threshold(t) => (*t * 1000.0) as i64,
                _ => 0,
            })
            .sum::<i64>();
        let edge_sum = controller
            .edges
            .iter()
            .map(|e| (e.weight * 1000.0) as i64)
            .sum::<i64>();
        node_sum + edge_sum
    }

    fn move_right_controller() -> ComputationGraph {
        ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![
                NodeKind::Constant(1.0), // 0
                NodeKind::OutputMoveX,   // 1
                NodeKind::OutputMoveY,   // 2
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            }],
        }
    }

    fn idle_controller() -> ComputationGraph {
        ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![
                NodeKind::OutputMoveX,
                NodeKind::OutputMoveY,
                NodeKind::OutputEat,
                NodeKind::OutputReproduce,
            ],
            edges: vec![],
        }
    }

    #[test]
    fn initial_population_has_founder_variation() {
        let cfg = WorldConfig {
            width: 30,
            height: 30,
            initial_creatures: 80,
            max_creatures: 200,
            ..WorldConfig::default()
        };
        let world = World::new_with_palette(cfg, 4242, ControllerPalette::Hybrid);

        let signatures = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();

        assert!(signatures.len() > 1);
    }

    #[test]
    fn food_growth_rate_zero_prevents_spawn_growth() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            max_creatures: 10,
            food_spawn_rate: 1.0,
            food_growth_rate: 0.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 1234);
        world.tick();

        let total_food: f32 = world.cells.iter().map(|c| c.food).sum();
        assert_eq!(total_food, 0.0);
    }

    #[test]
    fn food_growth_is_density_proportional_within_cells() {
        let cfg = WorldConfig {
            width: 2,
            height: 1,
            initial_creatures: 0,
            max_creatures: 10,
            food_spawn_rate: 0.0,
            food_growth_rate: 0.5,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 5150);
        world.cells[0].food = 0.2;
        world.cells[1].food = 0.4;

        world.tick();

        assert!((world.cells[0].food - 0.3).abs() < 1e-6);
        assert!((world.cells[1].food - 0.6).abs() < 1e-6);
    }

    #[test]
    fn food_below_spread_threshold_does_not_spread_even_if_local_growth_reaches_cap() {
        let cfg = WorldConfig {
            width: 2,
            height: 1,
            initial_creatures: 0,
            max_creatures: 10,
            world_wrap: false,
            food_spawn_rate: 0.0,
            food_growth_rate: 0.5,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 5151);
        let source_idx = world.idx(0, 0);
        let neighbor_idx = world.idx(1, 0);
        world.cells[source_idx].food = 0.74;
        world.cells[neighbor_idx].food = 0.0;

        world.tick();

        assert!((world.cells[source_idx].food - 1.0).abs() < 1e-6);
        assert!((world.cells[neighbor_idx].food - 0.0).abs() < 1e-6);
    }

    #[test]
    fn food_at_spread_threshold_spreads_to_single_neighbor() {
        let cfg = WorldConfig {
            width: 2,
            height: 1,
            initial_creatures: 0,
            max_creatures: 10,
            world_wrap: false,
            food_spawn_rate: 0.0,
            food_growth_rate: 0.2,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 5152);
        let source_idx = world.idx(0, 0);
        let neighbor_idx = world.idx(1, 0);
        world.cells[source_idx].food = 0.75;
        world.cells[neighbor_idx].food = 0.0;

        world.tick();

        assert!((world.cells[source_idx].food - 0.9).abs() < 1e-6);
        assert!((world.cells[neighbor_idx].food - 0.15).abs() < 1e-6);
    }

    #[test]
    fn fallback_spawn_does_not_run_when_average_density_is_above_floor() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            max_creatures: 10,
            food_spawn_rate: 1.0,
            food_growth_rate: 0.2,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 5153);
        for cell in &mut world.cells {
            cell.food = 0.1;
        }

        world.tick();

        let total_food: f32 = world.cells.iter().map(|c| c.food).sum();
        assert!((total_food - 12.0).abs() < 1e-4);
    }

    #[test]
    fn fallback_spawn_runs_when_average_density_is_below_floor() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            max_creatures: 10,
            food_spawn_rate: 1.0,
            food_growth_rate: 0.2,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 5154);
        let idx = world.idx(0, 0);
        world.cells[idx].food = 0.01;

        world.tick();

        let total_food: f32 = world.cells.iter().map(|c| c.food).sum();
        assert!(total_food > 0.012);
    }

    #[test]
    fn nearest_food_sensor_reports_direction_and_distance() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 0,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 101);

        let source_x = 10_u32;
        let source_y = 10_u32;
        let food_x = 13_u32;
        let food_y = 10_u32;
        let food_idx = world.idx(food_x, food_y);
        world.cells[food_idx].food = world.config.food_max_density;

        let (direction, distance) = world.nearest_food_sensor(source_x, source_y);
        assert!(
            direction.abs() < 0.01,
            "expected east-facing direction, got {direction}"
        );
        assert!(distance > 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nearest_food_sensor_defaults_when_no_food_is_visible() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 0,
            ..WorldConfig::default()
        };
        let world = World::new(cfg, 202);

        let (direction, distance) = world.nearest_food_sensor(6, 6);
        assert_eq!(direction, 0.0);
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn perception_reports_nearest_creature_direction_distance_and_density() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 424, ControllerPalette::Hybrid);

        let a = Creature {
            x: 5,
            y: 5,
            energy: 1.0,
            age: 0,
            generation: 0,
            lineage_id: 1,
            parent_id: None,
            controller: idle_controller(),
            rng: SmallRng::seed_from_u64(1),
            events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
            last_move_blocked: false,
            last_inputs: SensorInputs::default(),
            last_outputs: petri_graph::ActionOutputs::default(),
        };
        let b = Creature {
            x: 7,
            y: 5,
            energy: 1.0,
            age: 0,
            generation: 0,
            lineage_id: 2,
            parent_id: None,
            controller: idle_controller(),
            rng: SmallRng::seed_from_u64(2),
            events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
            last_move_blocked: false,
            last_inputs: SensorInputs::default(),
            last_outputs: petri_graph::ActionOutputs::default(),
        };
        let id_a = world.creatures.insert(a);
        let id_b = world.creatures.insert(b);
        let idx_a = world.idx(5, 5);
        let idx_b = world.idx(7, 5);
        world.creature_at[idx_a] = Some(id_a);
        world.creature_at[idx_b] = Some(id_b);

        world.tick();

        let a_after = world.creatures.get(id_a).expect("a should survive");
        assert!(
            a_after.last_inputs.creature_direction.abs() < 0.01,
            "expected eastward direction, got {}",
            a_after.last_inputs.creature_direction
        );
        assert!(
            (a_after.last_inputs.creature_distance - (2.0 / FOOD_SENSOR_RADIUS as f32)).abs()
                < 0.02
        );
        assert!(a_after.last_inputs.local_density > 0.0);
        assert!(a_after.last_inputs.local_density < 0.2);
    }

    #[test]
    fn move_blocked_feedback_toggles_for_failed_and_successful_moves() {
        let cfg = WorldConfig {
            width: 5,
            height: 5,
            initial_creatures: 1,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 5155, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);

        world.tick();
        let first = world
            .creatures
            .get(id)
            .expect("creature should remain alive after first blocked move");
        assert!(first.last_move_blocked);
        assert_eq!(first.last_inputs.move_blocked_last_tick, 0.0);

        world.tick();
        let second = world
            .creatures
            .get(id)
            .expect("creature should remain alive after second blocked move");
        assert!(second.last_move_blocked);
        assert!(second.last_inputs.move_blocked_last_tick > 0.5);

        let current_idx = world.idx(second.x, second.y);
        let open_idx = world.idx(2, second.y);
        if let Some(creature) = world.creatures.get_mut(id) {
            creature.x = 2;
        }
        world.creature_at[current_idx] = None;
        world.creature_at[open_idx] = Some(id);

        world.tick();
        let third = world
            .creatures
            .get(id)
            .expect("creature should remain alive after successful move");
        assert!(!third.last_move_blocked);

        world.tick();
        let fourth = world
            .creatures
            .get(id)
            .expect("creature should remain alive after follow-up move");
        assert_eq!(fourth.last_inputs.move_blocked_last_tick, 0.0);
    }

    #[test]
    fn spawn_random_creature_finds_free_cell_beyond_random_attempt_window() {
        let seed = 9001_u64;
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 0,
            max_creatures: 400,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, seed, ControllerPalette::Hybrid);

        let mut probe_rng = SmallRng::seed_from_u64(seed);
        let first_window = (0..64)
            .map(|_| {
                (
                    probe_rng.gen_range(0..world.config.width),
                    probe_rng.gen_range(0..world.config.height),
                )
            })
            .collect::<HashSet<_>>();

        let free_cell = (0..world.config.height)
            .flat_map(|y| (0..world.config.width).map(move |x| (x, y)))
            .find(|coord| !first_window.contains(coord))
            .expect("grid should contain at least one unsampled cell");

        let mut creature_seed = 1_u64;
        for y in 0..world.config.height {
            for x in 0..world.config.width {
                if (x, y) == free_cell {
                    continue;
                }
                let idx = world.idx(x, y);
                let creature = Creature {
                    x,
                    y,
                    energy: 1.0,
                    age: 0,
                    generation: 0,
                    lineage_id: creature_seed,
                    parent_id: None,
                    controller: ComputationGraph::founder(ControllerPalette::Hybrid),
                    rng: SmallRng::seed_from_u64(creature_seed),
                    events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                    last_move_blocked: false,
                    last_inputs: SensorInputs::default(),
                    last_outputs: ActionOutputs::default(),
                };
                creature_seed += 1;
                let id = world.creatures.insert(creature);
                world.creature_at[idx] = Some(id);
            }
        }

        let spawned = world.spawn_random_creature(0, None, None);
        assert!(spawned.is_some());
        assert!(world.creature_at[world.idx(free_cell.0, free_cell.1)].is_some());
    }

    #[test]
    fn offspring_controller_is_mutated_from_parent() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 1,
            max_creatures: 8,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 52, ControllerPalette::Hybrid);

        let (parent_id, px, py) = {
            let (id, parent) = world.creatures.iter().next().unwrap();
            (id, parent.x, parent.y)
        };
        if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
            parent_mut.controller = ComputationGraph::founder(ControllerPalette::Hybrid);
        }
        let idx = world.idx(px, py);
        world.cells[idx].food = 1.0;

        world.tick();
        assert!(world.creatures.len() >= 2);

        let checksums = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();
        assert!(checksums.len() > 1);
    }

    #[test]
    fn offspring_mutation_respects_zeroed_mutation_config() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 1,
            max_creatures: 12,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            weight_mutation_rate: 0.0,
            weight_mutation_magnitude: 0.0,
            structural_mutation_rate: 0.0,
            logic_node_mutation_rate: 0.0,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 222, ControllerPalette::Hybrid);
        world.seed_food_density(1.0);

        let parent_id = world
            .creatures
            .iter()
            .next()
            .map(|(id, _)| id)
            .expect("world should have one founder");
        if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
            parent_mut.controller = ComputationGraph::founder(ControllerPalette::Hybrid);
        }

        for _ in 0..20 {
            if world.creature_count() > 1 {
                break;
            }
            world.tick();
        }
        assert!(
            world.creature_count() > 1,
            "expected reproduction with abundant food"
        );

        let checksums = world
            .creatures
            .iter()
            .map(|(_, creature)| controller_checksum(&creature.controller))
            .collect::<BTreeSet<_>>();
        assert_eq!(checksums.len(), 1);
    }

    #[test]
    fn founders_have_unique_lineage_ids() {
        let cfg = WorldConfig {
            width: 30,
            height: 30,
            initial_creatures: 40,
            max_creatures: 200,
            ..WorldConfig::default()
        };
        let world = World::new(cfg, 5150);

        let lineage_ids = world
            .creatures
            .values()
            .map(|creature| creature.lineage_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(lineage_ids.len(), world.creature_count());
    }

    #[test]
    fn offspring_inherits_lineage_and_records_parent_link() {
        let cfg = WorldConfig {
            width: 16,
            height: 16,
            initial_creatures: 1,
            max_creatures: 12,
            energy_initial: 1.5,
            min_reproduce_energy: 0.8,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 818, ControllerPalette::Hybrid);
        world.seed_food_density(1.0);

        let parent_id = world
            .creatures
            .iter()
            .next()
            .map(|(id, _)| id)
            .expect("expected one founder");
        if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
            parent_mut.controller = ComputationGraph::founder(ControllerPalette::Hybrid);
        }

        for _ in 0..20 {
            if world.creature_count() > 1 {
                break;
            }
            world.tick();
        }
        assert!(
            world.creature_count() > 1,
            "expected at least one offspring"
        );

        let frame = world.frame();
        let child = frame
            .creatures
            .iter()
            .find(|c| c.parent_id.is_some())
            .expect("expected offspring snapshot with parent id");
        let parent = frame
            .creatures
            .iter()
            .find(|c| c.id == child.parent_id.expect("parent id should be present"))
            .expect("parent should still be visible in frame");
        assert_eq!(child.lineage_id, parent.lineage_id);
        assert_eq!(child.generation, parent.generation + 1);

        let children = world
            .lineage_tree
            .get(&parent.id)
            .expect("lineage tree should track parent-child relation");
        assert!(children.contains(&child.id));
    }

    #[test]
    fn world_wrap_true_wraps_movement_across_edge() {
        let cfg = WorldConfig {
            width: 6,
            height: 6,
            initial_creatures: 1,
            world_wrap: true,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 404, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);

        world.tick();

        let creature = world
            .creatures
            .get(id)
            .expect("creature should remain alive");
        assert_eq!(creature.y, edge_y);
        assert_eq!(creature.x, 0);
    }

    #[test]
    fn world_wrap_false_keeps_movement_in_bounds() {
        let cfg = WorldConfig {
            width: 6,
            height: 6,
            initial_creatures: 1,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 505, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);

        world.tick();

        let creature = world
            .creatures
            .get(id)
            .expect("creature should remain alive");
        assert_eq!(creature.y, edge_y);
        assert_eq!(creature.x, edge_x);
    }

    #[test]
    fn founder_seeded_hybrid_population_survives_short_horizon() {
        let cfg = WorldConfig {
            width: 40,
            height: 40,
            initial_creatures: 80,
            max_creatures: 1000,
            food_spawn_rate: 0.1,
            food_growth_rate: 0.2,
            energy_per_compute_node: 0.002,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 77, ControllerPalette::Hybrid);
        for _ in 0..100 {
            world.tick();
        }
        assert!(world.creature_count() > 0);
    }

    #[test]
    fn frame_creatures_include_controller_node_count() {
        let cfg = WorldConfig {
            width: 12,
            height: 12,
            initial_creatures: 1,
            ..WorldConfig::default()
        };
        let world = World::new_with_palette(cfg, 919, ControllerPalette::Hybrid);

        let frame = world.frame();
        assert_eq!(frame.creatures.len(), 1);
        assert!(frame.creatures[0].node_count > 0);
    }

    #[test]
    fn world_snapshot_round_trip_preserves_core_state() {
        let cfg = WorldConfig {
            width: 32,
            height: 24,
            initial_creatures: 30,
            max_creatures: 300,
            food_spawn_rate: 0.12,
            food_growth_rate: 0.18,
            food_spread_threshold: 0.62,
            food_spawn_floor_density: 0.09,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 2026, ControllerPalette::Hybrid);
        world.seed_food_density(0.2);
        for _ in 0..12 {
            world.tick();
        }

        let snapshot = world.snapshot();
        let restored = World::from_snapshot(snapshot);

        assert_eq!(restored.tick_count(), world.tick_count());
        assert_eq!(restored.config.width, world.config.width);
        assert_eq!(restored.config.height, world.config.height);
        assert_eq!(
            restored.config.initial_creatures,
            world.config.initial_creatures
        );
        assert_eq!(restored.config.max_creatures, world.config.max_creatures);
        assert_eq!(
            restored.config.food_spawn_rate,
            world.config.food_spawn_rate
        );
        assert_eq!(
            restored.config.food_growth_rate,
            world.config.food_growth_rate
        );
        assert_eq!(
            restored.config.food_spread_threshold,
            world.config.food_spread_threshold
        );
        assert_eq!(
            restored.config.food_spawn_floor_density,
            world.config.food_spawn_floor_density
        );
        assert_eq!(restored.frame().food, world.frame().food);
        assert_eq!(restored.creature_count(), world.creature_count());
        assert_eq!(restored.diagnostics().moves, world.diagnostics().moves);
        assert_eq!(restored.diagnostics().eats, world.diagnostics().eats);
        assert_eq!(
            restored.diagnostics().reproductions,
            world.diagnostics().reproductions
        );
        assert_eq!(restored.diagnostics().deaths, world.diagnostics().deaths);

        let before_links = world.lineage_tree.values().map(Vec::len).sum::<usize>();
        let after_links = restored.lineage_tree.values().map(Vec::len).sum::<usize>();
        assert_eq!(after_links, before_links);
    }

    #[test]
    fn world_snapshot_round_trip_preserves_last_inputs_outputs_and_blocked_feedback() {
        let cfg = WorldConfig {
            width: 6,
            height: 6,
            initial_creatures: 1,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 2027, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);
        let edge_x = world.config.width - 1;
        let edge_y = old_y;
        let edge_idx = world.idx(edge_x, edge_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = edge_x;
            creature.y = edge_y;
        }
        world.creature_at[old_idx] = None;
        world.creature_at[edge_idx] = Some(id);
        world.tick();
        world.tick();

        let snapshot = world.snapshot();
        let restored = World::from_snapshot(snapshot);
        let restored_creature = restored
            .creatures
            .values()
            .next()
            .expect("expected restored creature");
        assert!(restored_creature.last_move_blocked);
        assert!(restored_creature.last_inputs.move_blocked_last_tick > 0.5);
        assert!(restored_creature.last_outputs.move_x > 0.9);
    }

    #[test]
    fn creature_detail_exposes_last_inputs_outputs_and_events() {
        let cfg = WorldConfig {
            width: 8,
            height: 8,
            initial_creatures: 1,
            world_wrap: false,
            ..WorldConfig::default()
        };
        let mut world = World::new_with_palette(cfg, 616, ControllerPalette::Hybrid);
        let (id, old_x, old_y) = world
            .creatures
            .iter()
            .next()
            .map(|(id, c)| (id, c.x, c.y))
            .expect("expected one creature");
        let old_idx = world.idx(old_x, old_y);

        if let Some(creature) = world.creatures.get_mut(id) {
            creature.controller = move_right_controller();
            creature.x = 2;
            creature.y = 2;
        }
        let placed_idx = world.idx(2, 2);
        world.creature_at[old_idx] = None;
        world.creature_at[placed_idx] = Some(id);

        world.tick();

        let detail = world
            .creature_detail(id.data().as_ffi())
            .expect("detail should exist for live creature");
        assert_eq!(detail.id, id.data().as_ffi());
        assert!(detail.last_outputs.move_x > 0.9);
        assert!(detail
            .events
            .iter()
            .any(|event| event.kind == CreatureEventKind::Moved));
    }

    #[test]
    fn seed_food_density_sets_expected_occupied_cells() {
        let cfg = WorldConfig {
            width: 10,
            height: 10,
            initial_creatures: 0,
            food_max_density: 1.0,
            ..WorldConfig::default()
        };

        let mut world = World::new(cfg, 99);
        world.seed_food_density(0.25);

        let occupied = world
            .cells
            .iter()
            .filter(|cell| (cell.food - world.config.food_max_density).abs() < f32::EPSILON)
            .count();
        assert_eq!(occupied, 25);
        assert_eq!(
            world.cells.len() - occupied,
            world.cells.iter().filter(|cell| cell.food == 0.0).count()
        );
    }

use std::collections::{BTreeSet, HashSet};

use crate::ControllerPalette;
use petri_graph::{Edge, NodeKind};
use rand::Rng;
use serde_json::Value;

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

fn barrier_bit_is_set(bytes: &[u8], cell_idx: usize) -> bool {
    let byte_idx = cell_idx >> 3;
    let bit_idx = cell_idx & 7;
    bytes
        .get(byte_idx)
        .map(|byte| (byte & (1 << bit_idx)) != 0)
        .unwrap_or(false)
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

fn insert_idle_creature(world: &mut World, x: u32, y: u32, seed: u64) -> CreatureId {
    let idx = world.idx(x, y);
    assert!(
        world.creature_at[idx].is_none(),
        "expected target cell to be empty"
    );

    let controller = idle_controller();
    let phenotype_color = color::phenotype_rgb(0.0, 0.7);
    let creature = Creature {
        x,
        y,
        energy: 1.0,
        age: 0,
        generation: 0,
        lineage_id: seed,
        parent_id: None,
        controller,
        phenotype_color,
        phenotype_hue: 0.0,
        phenotype_saturation: 0.7,
        memory_register: founder_memory_register(),
        rng: SmallRng::seed_from_u64(seed),
        events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
        illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
        slot_capacity: founder_slot_capacity(),
        slots: empty_slots(founder_slot_capacity()),
        last_move_blocked: false,
        last_inputs: SensorInputs::default(),
        last_outputs: ActionOutputs::default(),
        last_memory_head: MemoryHeadState::default(),
    };

    let id = world.creatures.insert(creature);
    world.creature_at[idx] = Some(id);
    id
}

fn always_reproduce_and_eat_controller() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(1.0),   // 0
            NodeKind::OutputEat,       // 1
            NodeKind::OutputReproduce, // 2
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 2,
                weight: 1.0,
            },
        ],
    }
}

fn invert_memory_controller() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputMemoryRead,           // 0
            NodeKind::Negate,                    // 1
            NodeKind::OutputMemoryWriteValue,    // 2
            NodeKind::Constant(1.0),             // 3
            NodeKind::OutputMemoryWriteEnable,   // 4
            NodeKind::Constant(1.0),             // 5
            NodeKind::OutputMemoryAddressSelect, // 6
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 5,
                to: 6,
                weight: 1.0,
            },
        ],
    }
}

fn reproduce_only_controller() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(1.0),   // 0
            NodeKind::OutputReproduce, // 1
        ],
        edges: vec![Edge {
            from: 0,
            to: 1,
            weight: 1.0,
        }],
    }
}

fn eat_only_controller() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(1.0), // 0
            NodeKind::OutputEat,     // 1
        ],
        edges: vec![Edge {
            from: 0,
            to: 1,
            weight: 1.0,
        }],
    }
}

fn inventory_controller(
    pickup: f32,
    put: f32,
    slot_selector: f32,
    direction_selector: f32,
) -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(pickup),               // 0
            NodeKind::OutputInventoryPickup,          // 1
            NodeKind::Constant(put),                  // 2
            NodeKind::OutputInventoryPut,             // 3
            NodeKind::Constant(slot_selector),        // 4
            NodeKind::OutputInventorySlotSelect,      // 5
            NodeKind::Constant(direction_selector),   // 6
            NodeKind::OutputInventoryDirectionSelect, // 7
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
        ],
    }
}

fn slot_selector(slot_id: usize) -> f32 {
    assert!((1..=SLOT_COUNT_MAX).contains(&slot_id));
    let width = 2.0 / SLOT_COUNT_MAX as f32;
    -1.0 + ((slot_id - 1) as f32 + 0.5) * width
}

fn direction_selector(direction: TouchDirection) -> f32 {
    match direction {
        TouchDirection::SelfCell => -0.8,
        TouchDirection::North => -0.4,
        TouchDirection::East => 0.0,
        TouchDirection::South => 0.4,
        TouchDirection::West => 0.8,
    }
}

#[test]
fn selector_binning_is_deterministic_at_boundaries() {
    assert_eq!(World::selector_to_bin(-1.5, SLOT_COUNT_MAX), 0);
    assert_eq!(World::selector_to_bin(-1.0, SLOT_COUNT_MAX), 0);
    assert_eq!(
        World::selector_to_bin(1.0, SLOT_COUNT_MAX),
        SLOT_COUNT_MAX - 1
    );
    assert_eq!(
        World::selector_to_bin(1.5, SLOT_COUNT_MAX),
        SLOT_COUNT_MAX - 1
    );

    assert_eq!(
        World::direction_from_selector(-0.6001),
        TouchDirection::SelfCell
    );
    assert_eq!(
        World::direction_from_selector(-0.5999),
        TouchDirection::North
    );
    assert_eq!(
        World::direction_from_selector(-0.1999),
        TouchDirection::East
    );
    assert_eq!(
        World::direction_from_selector(0.2001),
        TouchDirection::South
    );
    assert_eq!(World::direction_from_selector(0.6), TouchDirection::West);
}

#[test]
fn inventory_actions_use_explicit_slot_addressing_for_pickup_and_put() {
    let cfg = WorldConfig {
        width: 5,
        height: 5,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        min_reproduce_energy: 10.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1313, ControllerPalette::Hybrid);
    let (id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);
    let center_idx = world.idx(2, 2);
    let north_idx = world.idx(2, 1);
    let south_idx = world.idx(2, 3);

    world.cells[north_idx].food = 0.4;

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.x = 2;
        creature.y = 2;
        creature.slot_capacity = 2;
        creature.slots = vec![None, None];
        creature.controller = inventory_controller(
            1.0,
            0.0,
            slot_selector(2),
            direction_selector(TouchDirection::North),
        );
    }
    world.creature_at[old_idx] = None;
    world.creature_at[center_idx] = Some(id);

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!(creature.slots[0].is_none());
    match &creature.slots[1] {
        Some(InventoryItem::Food(food_value)) => assert!((*food_value - 0.4).abs() < 1e-6),
        other => panic!("expected slot 2 food pickup, got {other:?}"),
    }
    assert!(world.cells[north_idx].food.abs() < 1e-6);

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = inventory_controller(
            0.0,
            1.0,
            slot_selector(2),
            direction_selector(TouchDirection::South),
        );
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!(creature.slots[1].is_none());
    assert!((world.cells[south_idx].food - 0.4).abs() < 1e-6);
}

#[test]
fn illegal_inventory_attempts_stack_penalties_and_record_reasons() {
    let cfg = WorldConfig {
        width: 5,
        height: 5,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_move: 0.0,
        energy_per_reproduce: 0.0,
        energy_per_inventory_attempt: 0.05,
        illegal_action_energy_penalty: 0.1,
        min_reproduce_energy: 10.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1414, ControllerPalette::Hybrid);
    let (id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);
    let center_idx = world.idx(2, 2);

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.x = 2;
        creature.y = 2;
        creature.energy = 1.0;
        creature.slot_capacity = 1;
        creature.slots = vec![Some(InventoryItem::Barrier)];
        creature.controller = inventory_controller(
            1.0,
            0.0,
            slot_selector(1),
            direction_selector(TouchDirection::North),
        );
    }
    world.creature_at[old_idx] = None;
    world.creature_at[center_idx] = Some(id);

    world.tick();

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.slots[0] = None;
        creature.controller = inventory_controller(
            0.0,
            1.0,
            slot_selector(1),
            direction_selector(TouchDirection::South),
        );
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!((creature.energy - 0.7).abs() < 1e-6);
    assert_eq!(world.diagnostics().illegal_actions, 2);
    let attempts = creature
        .illegal_attempts
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].action, IllegalActionKind::InventoryPickup);
    assert_eq!(attempts[0].reason, IllegalActionReason::SlotFull);
    assert_eq!(attempts[1].action, IllegalActionKind::InventoryPut);
    assert_eq!(attempts[1].reason, IllegalActionReason::SlotEmpty);
}

#[test]
fn illegal_move_cost_includes_load_scaling_and_penalty() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_move: 0.1,
        illegal_action_energy_penalty: 0.05,
        min_reproduce_energy: 10.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1515, ControllerPalette::Hybrid);
    let (id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);
    let edge_x = world.config.width - 1;
    let edge_idx = world.idx(edge_x, old_y);

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = move_right_controller();
        creature.x = edge_x;
        creature.energy = 1.0;
        creature.slot_capacity = 2;
        creature.slots = vec![
            Some(InventoryItem::Barrier),
            Some(InventoryItem::Food(0.25)),
        ];
    }
    world.creature_at[old_idx] = None;
    world.creature_at[edge_idx] = Some(id);

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!((creature.energy - 0.78).abs() < 1e-6);
    assert_eq!(world.diagnostics().illegal_actions, 1);
    let last = creature
        .illegal_attempts
        .back()
        .copied()
        .expect("illegal attempt should be logged");
    assert_eq!(last.action, IllegalActionKind::Move);
    assert_eq!(last.reason, IllegalActionReason::MoveOutOfBounds);
}

#[test]
fn reproduce_failure_charges_attempt_cost_plus_illegal_penalty() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        max_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_reproduce: 0.2,
        illegal_action_energy_penalty: 0.1,
        min_reproduce_energy: 0.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1616, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = reproduce_only_controller();
        creature.energy = 1.0;
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!((creature.energy - 0.7).abs() < 1e-6);
    assert_eq!(world.diagnostics().illegal_actions, 1);
    let last = creature
        .illegal_attempts
        .back()
        .copied()
        .expect("illegal attempt should be logged");
    assert_eq!(last.action, IllegalActionKind::Reproduce);
    assert_eq!(last.reason, IllegalActionReason::ReproduceMaxCreatures);
}

#[test]
fn eat_failure_charges_only_illegal_penalty() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        illegal_action_energy_penalty: 0.07,
        min_reproduce_energy: 10.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1717, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = eat_only_controller();
        creature.energy = 1.0;
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should remain alive");
    assert!((creature.energy - 0.93).abs() < 1e-6);
    assert_eq!(world.diagnostics().illegal_actions, 1);
    let last = creature
        .illegal_attempts
        .back()
        .copied()
        .expect("illegal attempt should be logged");
    assert_eq!(last.action, IllegalActionKind::Eat);
    assert_eq!(last.reason, IllegalActionReason::EatNoFood);
}

#[test]
fn offspring_slot_capacity_mutates_within_bounds_and_slots_start_empty() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        max_creatures: 2,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_reproduce: 0.0,
        min_reproduce_energy: 0.2,
        offspring_energy_fraction: 0.5,
        structural_mutation_rate: 1.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1818, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    let parent_id = id.data().as_ffi();
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = reproduce_only_controller();
        creature.energy = 1.0;
        creature.slot_capacity = 1;
        creature.slots = vec![Some(InventoryItem::Food(0.5))];
    }

    world.tick();

    let child = world
        .creatures
        .values()
        .find(|creature| creature.parent_id == Some(parent_id))
        .expect("expected offspring");
    assert_eq!(child.slot_capacity, 2);
    assert_eq!(child.slots.len(), 2);
    assert!(child.slots.iter().all(|slot| slot.is_none()));
}

#[test]
fn offspring_slot_capacity_shrinks_from_max_boundary() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        max_creatures: 2,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_reproduce: 0.0,
        min_reproduce_energy: 0.2,
        offspring_energy_fraction: 0.5,
        structural_mutation_rate: 1.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 1919, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    let parent_id = id.data().as_ffi();
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = reproduce_only_controller();
        creature.energy = 1.0;
        creature.slot_capacity = SLOT_COUNT_MAX;
        creature.slots = vec![None; SLOT_COUNT_MAX];
    }

    world.tick();

    let child = world
        .creatures
        .values()
        .find(|creature| creature.parent_id == Some(parent_id))
        .expect("expected offspring");
    assert_eq!(child.slot_capacity, SLOT_COUNT_MAX - 1);
    assert_eq!(child.slots.len(), SLOT_COUNT_MAX - 1);
}

#[test]
fn death_drop_falls_back_in_self_n_e_s_w_order() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.2,
        energy_per_compute_node: 0.0,
        min_reproduce_energy: 10.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 2020, ControllerPalette::Hybrid);
    let (dying_id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);
    let center_idx = world.idx(2, 2);
    let north_idx = world.idx(2, 1);
    let east_idx = world.idx(3, 2);

    if let Some(creature) = world.creatures.get_mut(dying_id) {
        creature.controller = idle_controller();
        creature.x = 2;
        creature.y = 2;
        creature.energy = 0.1;
        creature.slot_capacity = 1;
        creature.slots = vec![Some(InventoryItem::Barrier)];
    }
    world.creature_at[old_idx] = None;
    world.creature_at[center_idx] = Some(dying_id);
    world.cells[center_idx].barrier = true;

    let blocker_id = insert_idle_creature(&mut world, 2, 1, 4242);
    assert_eq!(world.creature_at[north_idx], Some(blocker_id));

    world.tick();

    assert!(!world.creatures.contains_key(dying_id));
    assert!(world.cells[east_idx].barrier);
}

#[test]
fn snapshot_round_trip_preserves_slots_illegal_attempts_and_illegal_metrics() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 2121, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.slot_capacity = 3;
        creature.slots = vec![
            Some(InventoryItem::Food(0.45)),
            Some(InventoryItem::Barrier),
            None,
        ];
        creature.illegal_attempts.push_back(IllegalActionAttempt {
            action: IllegalActionKind::Move,
            reason: IllegalActionReason::MoveBlocked,
            tick: 3,
        });
        creature.illegal_attempts.push_back(IllegalActionAttempt {
            action: IllegalActionKind::InventoryPut,
            reason: IllegalActionReason::TargetHasBarrier,
            tick: 4,
        });
    }
    world.diagnostics.illegal_actions = 2;

    let restored = World::from_snapshot(world.snapshot());
    let restored_creature = restored
        .creatures
        .values()
        .next()
        .expect("expected restored creature");
    assert_eq!(restored.diagnostics().illegal_actions, 2);
    assert_eq!(restored_creature.slot_capacity, 3);
    assert_eq!(restored_creature.slots.len(), 3);
    match &restored_creature.slots[0] {
        Some(InventoryItem::Food(value)) => assert!((*value - 0.45).abs() < 1e-6),
        other => panic!("expected restored food item in slot 1, got {other:?}"),
    }
    assert_eq!(restored_creature.slots[1], Some(InventoryItem::Barrier));
    assert!(restored_creature.slots[2].is_none());
    let attempts = restored_creature
        .illegal_attempts
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].action, IllegalActionKind::Move);
    assert_eq!(attempts[1].reason, IllegalActionReason::TargetHasBarrier);
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
fn founders_start_with_32_byte_memory_register() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 25,
        max_creatures: 200,
        ..WorldConfig::default()
    };

    let world = World::new(cfg, 3030);
    assert!(!world.creatures.is_empty());
    assert!(world
        .creatures
        .values()
        .all(|creature| creature.memory_register.len() == FOUNDER_MEMORY_REGISTER_BITS));
}

#[test]
fn offspring_memory_register_size_evolves_within_bounds() {
    let cfg = WorldConfig {
        width: 14,
        height: 14,
        initial_creatures: 1,
        max_creatures: 80,
        energy_initial: 1.4,
        min_reproduce_energy: 0.8,
        structural_mutation_rate: 1.0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 4040, ControllerPalette::Hybrid);
    world.seed_food_density(1.0);

    let parent_id = world
        .creatures
        .iter()
        .next()
        .map(|(id, _)| id)
        .expect("expected one founder");
    if let Some(parent_mut) = world.creatures.get_mut(parent_id) {
        parent_mut.controller = always_reproduce_and_eat_controller();
    }

    for _ in 0..30 {
        if world.creature_count() > 1 {
            break;
        }
        world.tick();
    }

    assert!(
        world.creature_count() > 1,
        "expected at least one offspring"
    );

    let sizes = world
        .creatures
        .values()
        .map(|creature| creature.memory_register.len())
        .collect::<Vec<_>>();
    assert!(sizes
        .iter()
        .all(|size| *size >= MEMORY_REGISTER_MIN_BITS && *size <= MAX_MEMORY_REGISTER_BITS));
    assert!(sizes
        .iter()
        .any(|size| *size != FOUNDER_MEMORY_REGISTER_BITS));
}

#[test]
fn memory_write_output_updates_targeted_creature_register_byte() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        min_reproduce_energy: 10.0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 711, ControllerPalette::Hybrid);

    let id = world
        .creatures
        .iter()
        .next()
        .map(|(id, _)| id)
        .expect("expected one creature");

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = invert_memory_controller();
        creature.memory_register = vec![64, 255];
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should still exist after one tick");
    assert_eq!(creature.last_inputs.memory_read, 1.0);
    assert_eq!(creature.last_outputs.memory_write_value, 0.0);
    assert_eq!(creature.last_outputs.memory_write_enable, 1.0);
    assert_eq!(creature.last_outputs.memory_address_select, 1.0);
    assert_eq!(creature.memory_register, vec![64, 0]);
}

#[test]
fn compute_cost_is_charged_once_per_tick_with_two_stage_memory_evaluation() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        energy_initial: 1.0,
        energy_max: 1.0,
        energy_per_tick_decay: 0.0,
        energy_per_move: 0.0,
        energy_per_compute_node: 0.1,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        min_reproduce_energy: 10.0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 9811, ControllerPalette::Hybrid);
    let id = world
        .creatures
        .iter()
        .next()
        .map(|(id, _)| id)
        .expect("expected one creature");

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![
                NodeKind::Constant(1.0),             // 0 hidden
                NodeKind::Add,                       // 1 hidden
                NodeKind::OutputMemoryAddressSelect, // 2 output
                NodeKind::OutputMemoryWriteEnable,   // 3 output
            ],
            edges: vec![
                Edge {
                    from: 0,
                    to: 1,
                    weight: 1.0,
                },
                Edge {
                    from: 1,
                    to: 2,
                    weight: 1.0,
                },
                Edge {
                    from: 0,
                    to: 3,
                    weight: 1.0,
                },
            ],
        };
        creature.energy = 1.0;
    }

    world.tick();

    let creature = world
        .creatures
        .get(id)
        .expect("creature should still exist after one tick");
    assert!((creature.energy - 0.8).abs() < 1e-6);
}

#[test]
fn offspring_inherits_parent_memory_bytes() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        max_creatures: 2,
        food_spawn_rate: 0.0,
        food_growth_rate: 0.0,
        energy_per_tick_decay: 0.0,
        energy_per_compute_node: 0.0,
        energy_per_reproduce: 0.0,
        min_reproduce_energy: 0.2,
        offspring_energy_fraction: 0.5,
        structural_mutation_rate: 0.0,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 9912, ControllerPalette::Hybrid);
    let (id, _) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x))
        .expect("expected one creature");
    let parent_id = id.data().as_ffi();
    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = reproduce_only_controller();
        creature.energy = 1.0;
        creature.memory_register = vec![5, 200, 13];
    }

    world.tick();

    let child = world
        .creatures
        .values()
        .find(|creature| creature.parent_id == Some(parent_id))
        .expect("expected offspring");
    assert_eq!(child.memory_register, vec![5, 200, 13]);
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
fn food_spread_skips_barrier_neighbor_cells() {
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

    let mut world = World::new(cfg, 6151);
    let source_idx = world.idx(0, 0);
    let blocked_idx = world.idx(1, 0);
    world.cells[source_idx].food = 0.75;
    world.cells[blocked_idx].barrier = true;

    world.tick();

    assert!((world.cells[source_idx].food - 0.9).abs() < 1e-6);
    assert_eq!(world.cells[blocked_idx].food, 0.0);
}

#[test]
fn food_spread_skips_occupied_neighbor_cells() {
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

    let mut world = World::new(cfg, 6152);
    let source_idx = world.idx(0, 0);
    let occupied_idx = world.idx(1, 0);
    world.cells[source_idx].food = 0.75;
    insert_idle_creature(&mut world, 1, 0, 77);
    assert!(world.creature_at[occupied_idx].is_some());

    world.tick();

    assert!((world.cells[source_idx].food - 0.9).abs() < 1e-6);
    assert_eq!(world.cells[occupied_idx].food, 0.0);
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
fn fallback_spawn_skips_barrier_cells() {
    let cfg = WorldConfig {
        width: 1,
        height: 1,
        initial_creatures: 0,
        max_creatures: 10,
        food_spawn_rate: 1.0,
        food_growth_rate: 0.2,
        food_max_density: 1.0,
        ..WorldConfig::default()
    };

    let mut world = World::new(cfg, 6153);
    let blocked_idx = world.idx(0, 0);
    world.cells[blocked_idx].barrier = true;

    world.tick();

    assert_eq!(world.cells[blocked_idx].food, 0.0);
}

#[test]
fn fallback_spawn_skips_occupied_cells() {
    let cfg = WorldConfig {
        width: 1,
        height: 1,
        initial_creatures: 0,
        max_creatures: 10,
        food_spawn_rate: 1.0,
        food_growth_rate: 0.2,
        food_max_density: 1.0,
        ..WorldConfig::default()
    };

    let mut world = World::new(cfg, 6154);
    insert_idle_creature(&mut world, 0, 0, 88);

    world.tick();

    assert_eq!(world.cells[world.idx(0, 0)].food, 0.0);
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
fn nearest_food_sensor_respects_configured_radius_cutoff() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 0,
        sensor_radius: 2,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 203);
    let source = (10_u32, 10_u32);
    let far_food = (13_u32, 10_u32);
    let far_food_idx = world.idx(far_food.0, far_food.1);
    world.cells[far_food_idx].food = world.config.food_max_density;

    let (direction, distance) = world.nearest_food_sensor(source.0, source.1);
    assert_eq!(direction, 0.0);
    assert_eq!(distance, 1.0);
}

#[test]
fn nearest_food_sensor_distance_normalizes_by_configured_radius() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 0,
        sensor_radius: 4,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 204);
    let source = (10_u32, 10_u32);
    let food = (12_u32, 10_u32);
    let food_idx = world.idx(food.0, food.1);
    world.cells[food_idx].food = world.config.food_max_density;

    let (direction, distance) = world.nearest_food_sensor(source.0, source.1);
    assert!(direction.abs() < 0.01);
    assert!((distance - 0.5).abs() < 0.02, "distance was {distance}");
}

#[test]
fn nearest_barrier_sensor_reports_direction_and_distance() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 0,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 301);

    let source_x = 10_u32;
    let source_y = 10_u32;
    let barrier_x = 13_u32;
    let barrier_y = 10_u32;
    let barrier_idx = world.idx(barrier_x, barrier_y);
    world.cells[barrier_idx].barrier = true;

    let (direction, distance) = world.nearest_barrier_sensor(source_x, source_y);
    assert!(
        direction.abs() < 0.01,
        "expected east-facing direction, got {direction}"
    );
    assert!(distance > 0.0);
    assert!(distance < 1.0);
}

#[test]
fn nearest_barrier_sensor_defaults_when_no_barrier_is_visible() {
    let cfg = WorldConfig {
        width: 12,
        height: 12,
        initial_creatures: 0,
        ..WorldConfig::default()
    };
    let world = World::new(cfg, 302);

    let (direction, distance) = world.nearest_barrier_sensor(6, 6);
    assert_eq!(direction, 0.0);
    assert_eq!(distance, 1.0);
}

#[test]
fn nearest_barrier_sensor_respects_configured_radius_cutoff() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 0,
        sensor_radius: 2,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 303);
    let source = (10_u32, 10_u32);
    let far_barrier = (13_u32, 10_u32);
    let far_barrier_idx = world.idx(far_barrier.0, far_barrier.1);
    world.cells[far_barrier_idx].barrier = true;

    let (direction, distance) = world.nearest_barrier_sensor(source.0, source.1);
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

    let a_controller = idle_controller();
    let a = Creature {
        x: 5,
        y: 5,
        energy: 1.0,
        age: 0,
        generation: 0,
        lineage_id: 1,
        parent_id: None,
        phenotype_color: color::phenotype_rgb(0.0, 0.7),
        phenotype_hue: 0.0,
        phenotype_saturation: 0.7,
        controller: a_controller,
        memory_register: founder_memory_register(),
        rng: SmallRng::seed_from_u64(1),
        events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
        illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
        slot_capacity: founder_slot_capacity(),
        slots: empty_slots(founder_slot_capacity()),
        last_move_blocked: false,
        last_inputs: SensorInputs::default(),
        last_outputs: petri_graph::ActionOutputs::default(),
        last_memory_head: MemoryHeadState::default(),
    };
    let b_controller = idle_controller();
    let b = Creature {
        x: 7,
        y: 5,
        energy: 1.0,
        age: 0,
        generation: 0,
        lineage_id: 2,
        parent_id: None,
        phenotype_color: color::phenotype_rgb(0.0, 0.7),
        phenotype_hue: 0.0,
        phenotype_saturation: 0.7,
        controller: b_controller,
        memory_register: founder_memory_register(),
        rng: SmallRng::seed_from_u64(2),
        events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
        illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
        slot_capacity: founder_slot_capacity(),
        slots: empty_slots(founder_slot_capacity()),
        last_move_blocked: false,
        last_inputs: SensorInputs::default(),
        last_outputs: petri_graph::ActionOutputs::default(),
        last_memory_head: MemoryHeadState::default(),
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
        (a_after.last_inputs.creature_distance - (2.0 / world.config.sensor_radius as f32)).abs()
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
fn movement_into_barrier_cell_is_blocked_and_sets_feedback() {
    let cfg = WorldConfig {
        width: 6,
        height: 6,
        initial_creatures: 1,
        world_wrap: false,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 31337, ControllerPalette::Hybrid);
    let (id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);

    if let Some(creature) = world.creatures.get_mut(id) {
        creature.controller = move_right_controller();
        creature.phenotype_hue = 0.0;
        creature.phenotype_saturation = 0.7;
        creature.phenotype_color = color::phenotype_rgb(0.0, 0.7);
        creature.x = 2;
        creature.y = 2;
    }
    let placed_idx = world.idx(2, 2);
    let blocked_idx = world.idx(3, 2);
    world.cells[blocked_idx].barrier = true;
    world.creature_at[old_idx] = None;
    world.creature_at[placed_idx] = Some(id);

    world.tick();
    let first = world
        .creatures
        .get(id)
        .expect("creature should remain alive after blocked move");
    assert_eq!(first.x, 2);
    assert_eq!(first.y, 2);
    assert!(first.last_move_blocked);

    world.tick();
    let second = world
        .creatures
        .get(id)
        .expect("creature should remain alive after second blocked move");
    assert!(second.last_inputs.move_blocked_last_tick > 0.5);
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
            let controller = ComputationGraph::founder(ControllerPalette::Hybrid);
            let creature = Creature {
                x,
                y,
                energy: 1.0,
                age: 0,
                generation: 0,
                lineage_id: creature_seed,
                parent_id: None,
                phenotype_color: color::phenotype_rgb(0.0, 0.7),
                phenotype_hue: 0.0,
                phenotype_saturation: 0.7,
                controller,
                memory_register: founder_memory_register(),
                rng: SmallRng::seed_from_u64(creature_seed),
                events: VecDeque::with_capacity(EVENT_LOG_CAPACITY),
                illegal_attempts: VecDeque::with_capacity(ILLEGAL_LOG_CAPACITY),
                slot_capacity: founder_slot_capacity(),
                slots: empty_slots(founder_slot_capacity()),
                last_move_blocked: false,
                last_inputs: SensorInputs::default(),
                last_outputs: ActionOutputs::default(),
                last_memory_head: MemoryHeadState::default(),
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
fn spawn_random_creature_avoids_barrier_cells() {
    let cfg = WorldConfig {
        width: 4,
        height: 4,
        initial_creatures: 0,
        max_creatures: 16,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 4242, ControllerPalette::Hybrid);
    for idx in 0..world.cells.len() {
        world.cells[idx].barrier = true;
    }
    let open = (1, 2);
    let open_idx = world.idx(open.0, open.1);
    world.cells[open_idx].barrier = false;

    let spawned = world.spawn_random_creature(0, None, None);
    assert!(
        spawned.is_some(),
        "expected a creature to spawn on the only open cell"
    );
    assert_eq!(world.creature_count(), 1);
    let creature = world
        .creatures
        .values()
        .next()
        .expect("spawned creature should exist");
    assert_eq!((creature.x, creature.y), open);
}

#[test]
fn reproduction_placement_avoids_barrier_neighbors() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        max_creatures: 4,
        world_wrap: false,
        energy_initial: 1.6,
        min_reproduce_energy: 0.8,
        energy_per_reproduce: 0.0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 6161, ControllerPalette::Hybrid);
    let (id, old_x, old_y) = world
        .creatures
        .iter()
        .next()
        .map(|(id, c)| (id, c.x, c.y))
        .expect("expected one creature");
    let old_idx = world.idx(old_x, old_y);
    let parent_pos = (3, 3);
    if let Some(parent) = world.creatures.get_mut(id) {
        parent.x = parent_pos.0;
        parent.y = parent_pos.1;
        parent.controller = always_reproduce_and_eat_controller();
    }
    let parent_idx = world.idx(parent_pos.0, parent_pos.1);
    world.creature_at[old_idx] = None;
    world.creature_at[parent_idx] = Some(id);

    for dx in -1_i32..=1 {
        for dy in -1_i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (parent_pos.0 as i32 + dx) as u32;
            let ny = (parent_pos.1 as i32 + dy) as u32;
            let idx = world.idx(nx, ny);
            world.cells[idx].barrier = true;
        }
    }
    let open_child_cell = (4, 3);
    let open_child_idx = world.idx(open_child_cell.0, open_child_cell.1);
    world.cells[open_child_idx].barrier = false;

    world.tick();

    assert_eq!(world.creature_count(), 2);
    let occupied_child = world.creature_at[world.idx(open_child_cell.0, open_child_cell.1)];
    assert!(
        occupied_child.is_some(),
        "expected offspring on only non-barrier neighbor"
    );
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
    assert_eq!(frame.creatures[0].phenotype_color.len(), 3);
}

#[test]
fn frame_barrier_bits_are_lsb_packed() {
    let cfg = WorldConfig {
        width: 5,
        height: 2,
        initial_creatures: 0,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 7171, ControllerPalette::Hybrid);
    world.cells[0].barrier = true;
    world.cells[3].barrier = true;
    world.cells[8].barrier = true;

    let frame = world.frame();
    assert_eq!(frame.barrier_bits, vec![0b0000_1001, 0b0000_0001]);
    assert!(barrier_bit_is_set(&frame.barrier_bits, 0));
    assert!(barrier_bit_is_set(&frame.barrier_bits, 3));
    assert!(barrier_bit_is_set(&frame.barrier_bits, 8));
    assert!(!barrier_bit_is_set(&frame.barrier_bits, 4));
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
    world.cells[0].barrier = true;
    let barrier_a = world.idx(2, 1);
    let barrier_b = world.idx(31, 23);
    world.cells[barrier_a].barrier = true;
    world.cells[barrier_b].barrier = true;
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
    assert_eq!(restored.frame().barrier_bits, world.frame().barrier_bits);
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
fn world_snapshot_import_defaults_missing_barriers_to_empty() {
    let cfg = WorldConfig {
        width: 10,
        height: 10,
        initial_creatures: 0,
        ..WorldConfig::default()
    };
    let world = World::new_with_palette(cfg, 2028, ControllerPalette::Hybrid);
    let snapshot = world.snapshot();
    let mut json = serde_json::to_value(snapshot).expect("snapshot should serialize");
    if let Value::Object(map) = &mut json {
        map.remove("cells_barrier");
    } else {
        panic!("snapshot must serialize to object");
    }

    let legacy_snapshot: WorldSnapshot =
        serde_json::from_value(json).expect("legacy snapshot should deserialize");
    let restored = World::from_snapshot(legacy_snapshot);
    assert!(restored.cells.iter().all(|cell| !cell.barrier));
    assert!(restored.frame().barrier_bits.iter().all(|byte| *byte == 0));
}

#[test]
fn world_snapshot_import_skips_creatures_placed_on_barriers() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        ..WorldConfig::default()
    };
    let world = World::new_with_palette(cfg, 2029, ControllerPalette::Hybrid);
    let mut snapshot = world.snapshot();
    let creature = snapshot
        .creatures
        .first()
        .cloned()
        .expect("expected one creature in snapshot");
    let creature_idx = (creature.y * snapshot.config.width + creature.x) as usize;
    snapshot.cells_barrier = vec![false; (snapshot.config.width * snapshot.config.height) as usize];
    snapshot.cells_barrier[creature_idx] = true;

    let restored = World::from_snapshot(snapshot);
    assert_eq!(restored.creature_count(), 0);
    assert!(barrier_bit_is_set(
        &restored.frame().barrier_bits,
        creature_idx
    ));
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
    assert!(restored_creature.last_memory_head.address_index < FOUNDER_MEMORY_REGISTER_BITS as u16);
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
        creature.phenotype_hue = 0.0;
        creature.phenotype_saturation = 0.7;
        creature.phenotype_color = color::phenotype_rgb(0.0, 0.7);
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
    assert_eq!(detail.phenotype_color.len(), 3);
    assert!((-1.0..=1.0).contains(&detail.last_inputs.barrier_direction));
    assert!((0.0..=1.0).contains(&detail.last_inputs.barrier_distance));
    assert!((0.0..=1.0).contains(&detail.last_inputs.memory_address_norm));
    assert!(detail.last_memory_head.address_index < FOUNDER_MEMORY_REGISTER_BITS as u16);
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

#[test]
fn paint_square_brush_half_extents_cover_expected_cell_counts() {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 0,
        food_max_density: 1.0,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 9001);
    let point = PaintPoint { x: 10, y: 10 };

    let one = world
        .apply_paint_stroke(PaintTool::Food, 0, &[point])
        .expect("extent 0 should be valid");
    assert_eq!(one.affected_cells, 1);

    world.clear_painted_cells();
    let three = world
        .apply_paint_stroke(PaintTool::Food, 1, &[point])
        .expect("extent 1 should be valid");
    assert_eq!(three.affected_cells, 9);

    world.clear_painted_cells();
    let five = world
        .apply_paint_stroke(PaintTool::Food, 2, &[point])
        .expect("extent 2 should be valid");
    assert_eq!(five.affected_cells, 25);
}

#[test]
fn paint_food_sets_cells_to_food_max_density() {
    let cfg = WorldConfig {
        width: 12,
        height: 12,
        initial_creatures: 0,
        food_max_density: 1.5,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 9002);
    let point = PaintPoint { x: 5, y: 6 };
    let stats = world
        .apply_paint_stroke(PaintTool::Food, 1, &[point])
        .expect("food paint should succeed");

    assert_eq!(stats.food_set_cells, 9);
    assert_eq!(stats.food_cleared_cells, 0);
    for x in 4..=6 {
        for y in 5..=7 {
            let idx = world.idx(x, y);
            assert!((world.cells[idx].food - world.config.food_max_density).abs() < f32::EPSILON);
        }
    }
}

#[test]
fn paint_erase_food_zeros_targeted_cells() {
    let cfg = WorldConfig {
        width: 10,
        height: 10,
        initial_creatures: 0,
        food_max_density: 1.0,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 9003);
    let idx = world.idx(3, 4);
    world.cells[idx].food = 0.8;

    let stats = world
        .apply_paint_stroke(PaintTool::EraseFood, 0, &[PaintPoint { x: 3, y: 4 }])
        .expect("food erase should succeed");

    assert_eq!(stats.food_cleared_cells, 1);
    assert_eq!(world.cells[idx].food, 0.0);
}

#[test]
fn paint_barrier_removes_creatures_without_counting_starvation_deaths() {
    let cfg = WorldConfig {
        width: 8,
        height: 8,
        initial_creatures: 1,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 9004);
    let (_, creature) = world
        .creatures
        .iter()
        .next()
        .expect("expected one creature");
    let point = PaintPoint {
        x: creature.x,
        y: creature.y,
    };
    let idx = world.idx(point.x, point.y);
    let deaths_before = world.diagnostics().deaths;

    let stats = world
        .apply_paint_stroke(PaintTool::Barrier, 0, &[point])
        .expect("barrier paint should succeed");

    assert_eq!(stats.creatures_removed, 1);
    assert_eq!(world.creature_count(), 0);
    assert!(world.cells[idx].barrier);
    assert_eq!(world.diagnostics().deaths, deaths_before);
}

#[test]
fn paint_clear_all_resets_food_and_barriers() {
    let cfg = WorldConfig {
        width: 10,
        height: 10,
        initial_creatures: 0,
        food_max_density: 1.0,
        ..WorldConfig::default()
    };
    let mut world = World::new(cfg, 9005);
    world
        .apply_paint_stroke(PaintTool::Food, 1, &[PaintPoint { x: 4, y: 4 }])
        .expect("food paint should succeed");
    world
        .apply_paint_stroke(PaintTool::Barrier, 1, &[PaintPoint { x: 6, y: 6 }])
        .expect("barrier paint should succeed");

    let stats = world.clear_painted_cells();

    assert!(stats.food_cleared_cells > 0);
    assert!(stats.barrier_cleared_cells > 0);
    assert!(world.cells.iter().all(|cell| !cell.barrier));
    assert!(world.cells.iter().all(|cell| cell.food == 0.0));
}

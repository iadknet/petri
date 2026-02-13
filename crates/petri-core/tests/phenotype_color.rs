use petri_core::{ControllerPalette, World, WorldConfig};

fn small_world(seed: u64) -> World {
    let cfg = WorldConfig {
        width: 20,
        height: 20,
        initial_creatures: 4,
        min_reproduce_energy: 3.0,
        energy_initial: 10.0,
        food_spawn_rate: 0.05,
        food_growth_rate: 0.1,
        food_energy_value: 5.0,
        ..WorldConfig::default()
    };
    World::new_with_palette(cfg, seed, ControllerPalette::Hybrid)
}

#[test]
fn founder_creatures_have_identical_colors() {
    let world = small_world(42);
    let frame = world.frame();
    assert!(frame.creatures.len() >= 2);
    let colors: Vec<_> = frame.creatures.iter().map(|c| c.phenotype_color).collect();
    let all_same = colors.windows(2).all(|w| w[0] == w[1]);
    assert!(all_same, "founders should all start with identical colors");
    // Founder RGB is intentionally a stable red-ish baseline.
    assert_eq!(colors[0], [204, 61, 61], "founder color should be red");
}

#[test]
fn snapshot_round_trip_preserves_phenotype_color() {
    let world = small_world(99);
    let frame_before = world.frame();

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot);
    let frame_after = restored.frame();

    assert_eq!(frame_before.creatures.len(), frame_after.creatures.len());
    for (before, after) in frame_before
        .creatures
        .iter()
        .zip(frame_after.creatures.iter())
    {
        assert_eq!(
            before.phenotype_color, after.phenotype_color,
            "phenotype color should survive snapshot round-trip"
        );
    }
}

#[test]
fn offspring_color_is_close_to_parent_color() {
    let cfg = WorldConfig {
        width: 10,
        height: 10,
        initial_creatures: 2,
        min_reproduce_energy: 3.0,
        energy_initial: 20.0,
        food_spawn_rate: 0.3,
        food_growth_rate: 0.5,
        food_energy_value: 10.0,
        energy_per_tick_decay: 0.01,
        energy_per_move: 0.01,
        energy_per_think_step: 0.0,
        offspring_energy_fraction: 0.4,
        ..WorldConfig::default()
    };
    let mut world = World::new_with_palette(cfg, 777, ControllerPalette::Hybrid);

    let founders: Vec<_> = world
        .frame()
        .creatures
        .iter()
        .map(|c| (c.id, c.phenotype_color))
        .collect();

    for _ in 0..500 {
        world.tick();
        if world.creature_count() > founders.len() {
            break;
        }
    }

    if world.creature_count() > founders.len() {
        let frame = world.frame();
        let founder_ids: Vec<_> = founders.iter().map(|(id, _)| *id).collect();
        let offspring: Vec<_> = frame
            .creatures
            .iter()
            .filter(|c| !founder_ids.contains(&c.id))
            .collect();

        for child in &offspring {
            if let Some(parent_id) = child.parent_id {
                if let Some(parent) = frame.creatures.iter().find(|c| c.id == parent_id) {
                    let max_channel_diff = child
                        .phenotype_color
                        .iter()
                        .zip(parent.phenotype_color.iter())
                        .map(|(a, b)| (*a as i32 - *b as i32).unsigned_abs())
                        .max()
                        .unwrap_or(0);
                    assert!(
                        max_channel_diff < 50,
                        "offspring color should be close to parent: child={:?} parent={:?} max_diff={}",
                        child.phenotype_color, parent.phenotype_color, max_channel_diff
                    );
                }
            }
        }
    }
}

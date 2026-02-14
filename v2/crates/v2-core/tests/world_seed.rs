use std::collections::HashSet;

use v2_core::world_seed::{seed_creature_cells, seed_food_cells};

fn as_set(cells: &[(u16, u16)]) -> HashSet<(u16, u16)> {
    cells.iter().copied().collect::<HashSet<_>>()
}

#[test]
fn food_seeding_is_deterministic_for_same_seed() {
    let first = seed_food_cells(20, 10, 0.15, 7);
    let second = seed_food_cells(20, 10, 0.15, 7);
    assert_eq!(first, second);
}

#[test]
fn food_seeding_varies_when_seed_changes() {
    let first = seed_food_cells(20, 10, 0.15, 7);
    let second = seed_food_cells(20, 10, 0.15, 8);
    assert_ne!(first, second);
}

#[test]
fn food_seeding_respects_density_target() {
    let cells = seed_food_cells(10, 10, 0.15, 9);
    assert_eq!(cells.len(), 15);
}

#[test]
fn creature_seeding_is_unique_and_bounded() {
    let creatures = seed_creature_cells(12, 8, 20, 99);
    let unique = as_set(&creatures);

    assert_eq!(creatures.len(), 20);
    assert_eq!(unique.len(), creatures.len());
    assert!(creatures.iter().all(|(x, y)| *x < 12 && *y < 8));
}

#[test]
fn creature_seeding_is_capped_by_occupancy() {
    let creatures = seed_creature_cells(4, 3, 100, 5);
    assert_eq!(creatures.len(), 12);
}

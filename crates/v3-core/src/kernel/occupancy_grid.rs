//! Coarse spatial occupancy — where the living population stands, by clade.
//!
//! A fixed 16x16 grid is laid over the world's extent, so a cell is a fraction
//! of the world rather than a fixed distance and the artifact is the same size
//! in every world. Per cell the grid carries the living population and the
//! number of distinct founder clades among them, and nothing else.
//!
//! Pure counting: positions and lineage ids the tick already wrote go in,
//! integer counts come out. No RNG is consumed and no float is accumulated.
//! The distinct-clade count comes from a sorted `Vec` of
//! `(cell_index, lineage_id)` pairs, so no hash iteration order can reach a
//! stored report.

use crate::contracts::Position;

/// Cells along each axis of the occupancy grid. A constant, not a config
/// value: the grid is a fraction of the world, whatever the world's size.
pub const OCCUPANCY_CELLS_PER_AXIS: u16 = 16;

/// Cells in the whole grid, row-major.
pub const OCCUPANCY_CELL_COUNT: usize =
    (OCCUPANCY_CELLS_PER_AXIS as usize) * (OCCUPANCY_CELLS_PER_AXIS as usize);

/// One checkpoint's occupancy grid: two parallel row-major arrays, always
/// full. An unoccupied cell is a `0`, never an absence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccupancyGrid {
    /// Living creatures standing in each cell.
    pub population: [u64; OCCUPANCY_CELL_COUNT],
    /// Distinct founder clades (`lineage_id`) among them. A clade counts once
    /// per cell however many of its creatures stand there, and once again in
    /// every other cell it occupies.
    pub distinct_clades: [u64; OCCUPANCY_CELL_COUNT],
}

impl Default for OccupancyGrid {
    /// The empty grid: both arrays at full length, every entry `0`.
    fn default() -> Self {
        Self {
            population: [0; OCCUPANCY_CELL_COUNT],
            distinct_clades: [0; OCCUPANCY_CELL_COUNT],
        }
    }
}

/// The row-major cell index of a position in a `width` x `height` world.
///
/// Binning is by proportion: `(coord * 16) / extent`, multiplied in `u32`
/// because `u16` overflows at `4096` and the goal worlds are `1600` wide. An
/// in-world position never yields `16`, since `coord <= extent - 1`, so no
/// divisibility precondition and no clamp is needed.
///
/// `position` is in-world and `width` and `height` are nonzero: the only
/// caller bins one living creature per call, and a creature stands in a world
/// with an extent.
#[must_use]
pub fn occupancy_cell_index(position: Position, width: u16, height: u16) -> usize {
    let axis = u32::from(OCCUPANCY_CELLS_PER_AXIS);
    let cell = |coord: u16, extent: u16| -> usize {
        (u32::from(coord) * axis / u32::from(extent)) as usize
    };
    cell(position.y, height) * usize::from(OCCUPANCY_CELLS_PER_AXIS) + cell(position.x, width)
}

/// Aggregate one checkpoint's living population into the occupancy grid.
///
/// `creatures` yields one `(position, lineage_id)` pair per living creature,
/// in any order: the pairs are sorted before counting, so the result does not
/// depend on the caller's iteration order.
#[must_use]
pub fn occupancy_grid(
    width: u16,
    height: u16,
    creatures: impl IntoIterator<Item = (Position, u32)>,
) -> OccupancyGrid {
    let mut pairs: Vec<(usize, u32)> = creatures
        .into_iter()
        .map(|(position, lineage_id)| (occupancy_cell_index(position, width, height), lineage_id))
        .collect();
    pairs.sort_unstable();

    let mut grid = OccupancyGrid::default();
    let mut previous: Option<(usize, u32)> = None;
    for pair in pairs {
        let (cell, _) = pair;
        grid.population[cell] += 1;
        if previous != Some(pair) {
            grid.distinct_clades[cell] += 1;
        }
        previous = Some(pair);
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn at(x: u16, y: u16) -> Position {
        Position::new(x, y)
    }

    #[test]
    fn bins_a_width_divisible_by_sixteen_on_its_exact_cell_edges() {
        // 1600 / 16 == 100: the edges land on multiples of 100.
        assert_eq!(occupancy_cell_index(at(0, 0), 1600, 1600), 0);
        assert_eq!(occupancy_cell_index(at(99, 0), 1600, 1600), 0);
        assert_eq!(occupancy_cell_index(at(100, 0), 1600, 1600), 1);
        assert_eq!(occupancy_cell_index(at(199, 0), 1600, 1600), 1);
        assert_eq!(occupancy_cell_index(at(1500, 0), 1600, 1600), 15);
    }

    #[test]
    fn bins_a_width_not_divisible_by_sixteen_without_a_precondition() {
        // 100 positions over 16 cells: 6.25 positions per cell, no divisor.
        assert_eq!(occupancy_cell_index(at(6, 0), 100, 100), 0);
        assert_eq!(occupancy_cell_index(at(7, 0), 100, 100), 1);
        assert_eq!(occupancy_cell_index(at(12, 0), 100, 100), 1);
        assert_eq!(occupancy_cell_index(at(13, 0), 100, 100), 2);
        assert_eq!(occupancy_cell_index(at(99, 0), 100, 100), 15);
    }

    #[test]
    fn bins_rows_major_so_y_strides_by_sixteen() {
        assert_eq!(occupancy_cell_index(at(0, 100), 1600, 1600), 16);
        assert_eq!(occupancy_cell_index(at(300, 200), 1600, 1600), 2 * 16 + 3);
    }

    #[test]
    fn bins_the_far_corner_of_a_world_into_the_last_cell() {
        assert_eq!(
            occupancy_cell_index(at(1599, 1599), 1600, 1600),
            OCCUPANCY_CELL_COUNT - 1
        );
        assert_eq!(
            occupancy_cell_index(at(127, 127), 128, 128),
            OCCUPANCY_CELL_COUNT - 1
        );
        assert_eq!(occupancy_cell_index(at(0, 0), 1, 1), 0);
    }

    #[test]
    fn counts_a_clade_once_per_cell_however_many_of_its_creatures_stand_there() {
        let grid = occupancy_grid(
            1600,
            1600,
            [(at(10, 10), 7), (at(20, 20), 7), (at(30, 30), 7)],
        );
        assert_eq!(grid.population[0], 3);
        assert_eq!(grid.distinct_clades[0], 1);
    }

    #[test]
    fn counts_one_clade_separately_in_each_cell_it_occupies() {
        let grid = occupancy_grid(1600, 1600, [(at(10, 10), 7), (at(110, 10), 7)]);
        assert_eq!(grid.population[0], 1);
        assert_eq!(grid.distinct_clades[0], 1);
        assert_eq!(grid.population[1], 1);
        assert_eq!(grid.distinct_clades[1], 1);
    }

    #[test]
    fn distinguishes_two_clades_sharing_a_cell_from_two_clades_split_across_cells() {
        let shared = occupancy_grid(1600, 1600, [(at(10, 10), 1), (at(20, 20), 2)]);
        assert_eq!(shared.population[0], 2);
        assert_eq!(shared.distinct_clades[0], 2);
        assert_eq!(shared.population[1], 0);
        assert_eq!(shared.distinct_clades[1], 0);

        let split = occupancy_grid(1600, 1600, [(at(10, 10), 1), (at(110, 10), 2)]);
        assert_eq!(split.population[0], 1);
        assert_eq!(split.distinct_clades[0], 1);
        assert_eq!(split.population[1], 1);
        assert_eq!(split.distinct_clades[1], 1);
    }

    #[test]
    fn reports_a_full_zero_grid_for_an_empty_population() {
        let grid = occupancy_grid(1600, 1600, []);
        assert_eq!(grid.population.len(), OCCUPANCY_CELL_COUNT);
        assert_eq!(grid.distinct_clades.len(), OCCUPANCY_CELL_COUNT);
        assert!(grid.population.iter().all(|&count| count == 0));
        assert!(grid.distinct_clades.iter().all(|&count| count == 0));
        assert_eq!(grid, OccupancyGrid::default());
    }

    /// In-world `(position, lineage_id)` pairs for a world of the drawn size.
    fn population_in_world() -> impl Strategy<Value = (u16, u16, Vec<(Position, u32)>)> {
        (1u16..=2000, 1u16..=2000).prop_flat_map(|(width, height)| {
            (
                Just(width),
                Just(height),
                prop::collection::vec((0..width, 0..height, 0u32..6), 0..60).prop_map(
                    |creatures| {
                        creatures
                            .into_iter()
                            .map(|(x, y, lineage_id)| (Position::new(x, y), lineage_id))
                            .collect::<Vec<_>>()
                    },
                ),
            )
        })
    }

    proptest! {
        #[test]
        fn every_in_world_position_bins_below_the_cell_count(
            (width, height, creatures) in population_in_world()
        ) {
            for (position, _) in creatures {
                prop_assert!(occupancy_cell_index(position, width, height) < OCCUPANCY_CELL_COUNT);
            }
        }

        #[test]
        fn the_population_sum_equals_the_creature_count(
            (width, height, creatures) in population_in_world()
        ) {
            let expected = creatures.len() as u64;
            let grid = occupancy_grid(width, height, creatures);
            prop_assert_eq!(grid.population.iter().sum::<u64>(), expected);
        }

        #[test]
        fn each_cell_counts_at_most_its_population_and_at_most_the_clades_present(
            (width, height, creatures) in population_in_world()
        ) {
            let distinct_present = creatures
                .iter()
                .map(|&(_, lineage_id)| lineage_id)
                .collect::<BTreeSet<_>>()
                .len() as u64;
            let grid = occupancy_grid(width, height, creatures);
            for cell in 0..OCCUPANCY_CELL_COUNT {
                prop_assert!(grid.distinct_clades[cell] <= grid.population[cell]);
                prop_assert!(grid.distinct_clades[cell] <= distinct_present);
                prop_assert_eq!(
                    grid.distinct_clades[cell] == 0,
                    grid.population[cell] == 0
                );
            }
        }

        #[test]
        fn the_grid_does_not_depend_on_the_order_creatures_are_visited(
            (width, height, creatures) in population_in_world()
        ) {
            let forward = occupancy_grid(width, height, creatures.clone());
            let reversed = occupancy_grid(width, height, creatures.into_iter().rev());
            prop_assert_eq!(forward, reversed);
        }
    }
}

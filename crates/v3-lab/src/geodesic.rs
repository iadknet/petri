//! Shortest passable paths on a wrap arena. A king step is passable when its
//! destination is not a barrier (`apply_move` checks only the destination),
//! so a diagonal passes between two corner-adjacent barriers. On a
//! barrier-free torus the geodesic equals the toroidal Chebyshev distance.

use std::collections::VecDeque;

use v3_core::contracts::{Direction, Position};
use v3_core::kernel::WorldState;

/// The barrier cells of a `size × size` wrap arena.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    size: u16,
    blocked: Vec<bool>,
}

impl Terrain {
    /// A terrain whose barriers are `cells`.
    #[must_use]
    pub fn from_cells(size: u16, cells: &[Position]) -> Self {
        let mut terrain = Self {
            size,
            blocked: vec![false; usize::from(size) * usize::from(size)],
        };
        for &cell in cells {
            let index = terrain.index(cell);
            terrain.blocked[index] = true;
        }
        terrain
    }

    /// The barriers of a square `world`.
    #[must_use]
    pub fn from_world(world: &WorldState) -> Self {
        let size = world.width;
        let cells: Vec<Position> = (0..size)
            .flat_map(|y| (0..size).map(move |x| Position::new(x, y)))
            .filter(|&cell| world.is_barrier(cell))
            .collect();
        Self::from_cells(size, &cells)
    }

    #[must_use]
    pub fn size(&self) -> u16 {
        self.size
    }

    fn index(&self, cell: Position) -> usize {
        cell_index(cell, self.size)
    }

    #[must_use]
    pub fn is_blocked(&self, cell: Position) -> bool {
        self.blocked[self.index(cell)]
    }

    /// The wrapped neighbour of `cell` in `direction`.
    #[must_use]
    pub fn neighbor(&self, cell: Position, direction: Direction) -> Position {
        offset(cell, direction.delta(), self.size)
    }
}

/// The row-major index of `cell` on a `size × size` arena.
fn cell_index(cell: Position, size: u16) -> usize {
    usize::from(cell.y) * usize::from(size) + usize::from(cell.x)
}

/// `cell + (dx, dy)`, wrapped on a `size × size` torus.
#[must_use]
pub fn offset(cell: Position, (dx, dy): (i32, i32), size: u16) -> Position {
    let axis = |p: u16, d: i32| {
        let wrapped = (i32::from(p) + d).rem_euclid(i32::from(size));
        u16::try_from(wrapped).expect("a wrapped coordinate fits the arena")
    };
    Position::new(axis(cell.x, dx), axis(cell.y, dy))
}

fn wrapped_sign(from: u16, to: u16, size: u16) -> i32 {
    let forward = (i32::from(to) - i32::from(from)).rem_euclid(i32::from(size));
    if forward == 0 {
        0
    } else if forward <= i32::from(size) / 2 {
        1
    } else {
        -1
    }
}

/// The king step that shortens the toroidal Chebyshev distance to `to`
/// (`N` when `from == to`).
#[must_use]
pub fn step_toward(from: Position, to: Position, size: u16) -> Direction {
    let delta = (
        wrapped_sign(from.x, to.x, size),
        wrapped_sign(from.y, to.y, size),
    );
    Direction::ALL
        .into_iter()
        .find(|direction| direction.delta() == delta)
        .unwrap_or(Direction::N)
}

/// Geodesic distance from every cell to the nearest of a set of sources:
/// one multi-source BFS over passable cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    size: u16,
    dist: Vec<Option<u32>>,
}

impl Field {
    /// The field of `sources` over `terrain`; blocked sources are ignored.
    #[must_use]
    pub fn new(terrain: &Terrain, sources: &[Position]) -> Self {
        let mut dist = vec![None; terrain.blocked.len()];
        let mut queue = VecDeque::new();
        for &source in sources {
            let index = terrain.index(source);
            if !terrain.blocked[index] && dist[index].is_none() {
                dist[index] = Some(0);
                queue.push_back(source);
            }
        }
        while let Some(cell) = queue.pop_front() {
            let next = dist[terrain.index(cell)].expect("queued cells have a distance") + 1;
            for direction in Direction::ALL {
                let neighbor = terrain.neighbor(cell, direction);
                let index = terrain.index(neighbor);
                if !terrain.blocked[index] && dist[index].is_none() {
                    dist[index] = Some(next);
                    queue.push_back(neighbor);
                }
            }
        }
        Self {
            size: terrain.size,
            dist,
        }
    }

    /// The distance at `cell`; `None` when no source is reachable.
    #[must_use]
    pub fn at(&self, cell: Position) -> Option<u32> {
        self.dist[cell_index(cell, self.size)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::torus_distance;
    use proptest::prelude::*;

    #[test]
    fn a_wall_forces_a_detour_around_its_end() {
        // A vertical wall x = 10, y in 5..=15, between (8, 10) and (12, 10).
        let wall: Vec<Position> = (5..=15).map(|y| Position::new(10, y)).collect();
        let terrain = Terrain::from_cells(32, &wall);
        let field = Field::new(&terrain, &[Position::new(12, 10)]);
        // Around the end at y = 4: 6 steps to (10, 4), then 6 down to (12, 10)
        // along a king path: max(|dx|, |dy|) per leg = 6 + 6.
        assert_eq!(field.at(Position::new(8, 10)), Some(12));
        assert_eq!(
            torus_distance(Position::new(8, 10), Position::new(12, 10), 32),
            4
        );
        assert_eq!(field.at(Position::new(10, 10)), None, "a barrier cell");
    }

    #[test]
    fn a_path_wraps_across_the_edge() {
        let terrain = Terrain::from_cells(16, &[]);
        let field = Field::new(&terrain, &[Position::new(0, 0)]);
        assert_eq!(field.at(Position::new(15, 15)), Some(1));
        assert_eq!(field.at(Position::new(13, 2)), Some(3));
        // A full-height wall at x = 1 leaves only the wrapped way west.
        let wall: Vec<Position> = (0..16).map(|y| Position::new(1, y)).collect();
        let field = Field::new(&Terrain::from_cells(16, &wall), &[Position::new(0, 0)]);
        assert_eq!(field.at(Position::new(2, 0)), Some(14));
    }

    #[test]
    fn a_diagonal_passes_between_corner_adjacent_barriers() {
        // Barriers at (5, 4) and (4, 5): the step (4, 4) -> (5, 5) is
        // passable because only its destination is checked.
        let terrain = Terrain::from_cells(16, &[Position::new(5, 4), Position::new(4, 5)]);
        let field = Field::new(&terrain, &[Position::new(5, 5)]);
        assert_eq!(field.at(Position::new(4, 4)), Some(1));
    }

    #[test]
    fn an_enclosed_source_is_unreachable_and_a_blocked_one_ignored() {
        let source = Position::new(8, 8);
        let ring: Vec<Position> = Direction::ALL
            .into_iter()
            .map(|d| offset(source, d.delta(), 16))
            .collect();
        let field = Field::new(&Terrain::from_cells(16, &ring), &[source]);
        assert_eq!(field.at(source), Some(0));
        assert_eq!(field.at(Position::new(0, 0)), None);
        let field = Field::new(&Terrain::from_cells(16, &[source]), &[source]);
        assert_eq!(field.at(Position::new(0, 0)), None);
    }

    proptest! {
        #[test]
        fn a_barrier_free_field_is_the_chebyshev_distance(
            size in 3u16..40,
            sources in prop::collection::vec((0u16..40, 0u16..40), 1..4),
            cell in (0u16..40, 0u16..40),
        ) {
            let sources: Vec<Position> = sources
                .into_iter()
                .map(|(x, y)| Position::new(x % size, y % size))
                .collect();
            let cell = Position::new(cell.0 % size, cell.1 % size);
            let field = Field::new(&Terrain::from_cells(size, &[]), &sources);
            let nearest = sources
                .iter()
                .map(|&s| u32::from(torus_distance(s, cell, size)))
                .min();
            prop_assert_eq!(field.at(cell), nearest);
        }
    }
}

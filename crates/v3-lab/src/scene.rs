//! Arena `sparse-food-v1` scenes: one seeded food placement around the arena
//! centre, drawn so the exposure predicate holds by construction.

use rand::rngs::SmallRng;
use rand::Rng;
use v3_core::contracts::Position;

/// Consecutive failed draws that make a density infeasible.
pub const REDRAW_LIMIT: u32 = 100;
/// Food cells are drawn at toroidal Chebyshev distance at least this far from
/// the start cell.
pub const MIN_START_DISTANCE: u16 = 2;

/// Toroidal Chebyshev distance on a `size × size` wrap world.
#[must_use]
pub fn torus_distance(a: Position, b: Position, size: u16) -> u16 {
    let axis = |p: u16, q: u16| {
        let d = p.abs_diff(q);
        d.min(size - d)
    };
    axis(a.x, b.x).max(axis(a.y, b.y))
}

/// The arena centre, where every scene starts.
#[must_use]
pub fn centre(size: u16) -> Position {
    Position::new(size / 2, size / 2)
}

/// One seeded placement.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Scene {
    /// `Simulation` seed (one `scenes` draw).
    pub seed: u64,
    /// Food cells of type 0 at `max_density`, in draw order.
    pub food: Vec<Position>,
    /// Draws rejected by the exposure predicate before this one.
    pub redraws: u32,
}

/// A density whose draws failed the exposure predicate `REDRAW_LIMIT` times
/// in a row (or cannot hold the requested cell count at all).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Infeasible {
    pub redraws: u32,
}

/// Food cells requested at `fraction`: `round(fraction × size²)`.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn food_count(size: u16, fraction: f64) -> usize {
    let cells = f64::from(size) * f64::from(size);
    (fraction * cells).round().max(0.0) as usize
}

/// Whether `food` meets the exposure predicate from `start`: at least one
/// cell within `vision_radius`, none within distance 1.
#[must_use]
pub fn exposed(food: &[Position], start: Position, size: u16, vision_radius: u8) -> bool {
    let mut seen = false;
    for &cell in food {
        let d = torus_distance(cell, start, size);
        if d <= 1 {
            return false;
        }
        seen |= d <= u16::from(vision_radius);
    }
    seen
}

/// Draw one scene from `rng`: a `Simulation` seed, then food cells uniformly
/// without replacement among cells at distance ≥ 2 from the centre,
/// redrawing from the same stream until the exposure predicate holds.
///
/// # Errors
///
/// [`Infeasible`] after `REDRAW_LIMIT` consecutive failures, or when the
/// eligible cells cannot hold the requested count.
pub fn draw_scene(
    rng: &mut SmallRng,
    size: u16,
    fraction: f64,
    vision_radius: u8,
) -> Result<Scene, Infeasible> {
    let seed = rng.gen::<u64>();
    let start = centre(size);
    let eligible: Vec<Position> = (0..size)
        .flat_map(|y| (0..size).map(move |x| Position::new(x, y)))
        .filter(|&p| torus_distance(p, start, size) >= MIN_START_DISTANCE)
        .collect();
    let count = food_count(size, fraction);
    if count == 0 || count > eligible.len() {
        return Err(Infeasible { redraws: 0 });
    }
    for redraws in 0..REDRAW_LIMIT {
        let food: Vec<Position> = rand::seq::index::sample(rng, eligible.len(), count)
            .into_iter()
            .map(|i| eligible[i])
            .collect();
        if exposed(&food, start, size, vision_radius) {
            return Ok(Scene {
                seed,
                food,
                redraws,
            });
        }
    }
    Err(Infeasible {
        redraws: REDRAW_LIMIT,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::SeedableRng;

    #[test]
    fn distance_wraps_on_both_axes() {
        let size = 10;
        assert_eq!(
            torus_distance(Position::new(0, 0), Position::new(9, 9), size),
            1
        );
        assert_eq!(
            torus_distance(Position::new(0, 0), Position::new(5, 2), size),
            5
        );
        assert_eq!(
            torus_distance(Position::new(3, 3), Position::new(3, 3), size),
            0
        );
    }

    #[test]
    fn exposure_predicate_rejects_adjacent_food_and_requires_visible_food() {
        let size = 32;
        let start = centre(size);
        let near = Position::new(start.x + 1, start.y);
        let visible = Position::new(start.x + 4, start.y);
        let far = Position::new(start.x + 10, start.y);
        assert!(!exposed(&[near, visible], start, size, 5));
        assert!(exposed(&[visible, far], start, size, 5));
        assert!(!exposed(&[far], start, size, 5));
    }

    #[test]
    fn sparse_draws_record_their_redraws_and_dense_impossible_draws_fail() {
        let mut rng = SmallRng::seed_from_u64(3);
        // One food cell on a 48² arena is almost never within radius 5:
        // either the redraw record is non-zero or the density is infeasible.
        match draw_scene(&mut rng, 48, 1.0 / (48.0 * 48.0), 5) {
            Ok(scene) => {
                assert!(scene.redraws > 0);
                assert!(exposed(&scene.food, centre(48), 48, 5));
            }
            Err(infeasible) => assert_eq!(infeasible.redraws, REDRAW_LIMIT),
        }
        // Radius 1 can never be met: every draw fails the predicate.
        let mut rng = SmallRng::seed_from_u64(3);
        assert_eq!(
            draw_scene(&mut rng, 16, 0.05, 1),
            Err(Infeasible {
                redraws: REDRAW_LIMIT
            })
        );
    }

    #[test]
    fn a_count_of_zero_or_above_the_eligible_cells_fails_without_drawing() {
        // An 8² arena has 64 − 9 = 55 cells at distance ≥ 2 from the centre.
        let cells = 64.0;
        let draw = |fraction: f64| draw_scene(&mut SmallRng::seed_from_u64(1), 8, fraction, 5);
        assert_eq!(draw(0.0), Err(Infeasible { redraws: 0 }));
        assert_eq!(draw(56.0 / cells), Err(Infeasible { redraws: 0 }));
        let full = draw(55.0 / cells).expect("every eligible cell holds food");
        assert_eq!(full.food.len(), 55);
        assert_eq!(full.redraws, 0);
    }

    proptest! {
        #[test]
        fn distance_is_a_bounded_symmetric_metric(
            size in 3u16..80,
            ax in 0u16..80, ay in 0u16..80, bx in 0u16..80, by in 0u16..80,
        ) {
            let a = Position::new(ax % size, ay % size);
            let b = Position::new(bx % size, by % size);
            let d = torus_distance(a, b, size);
            prop_assert_eq!(d, torus_distance(b, a, size));
            prop_assert!(d <= size / 2);
            prop_assert_eq!(d == 0, a == b);
        }

        #[test]
        fn every_drawn_scene_meets_the_exposure_predicate(
            seed in any::<u64>(),
            size in 24u16..48,
            fraction in 0.02f64..0.1,
        ) {
            let mut rng = SmallRng::seed_from_u64(seed);
            if let Ok(scene) = draw_scene(&mut rng, size, fraction, 5) {
                prop_assert!(exposed(&scene.food, centre(size), size, 5));
                prop_assert_eq!(scene.food.len(), food_count(size, fraction));
                let mut unique = scene.food.clone();
                unique.sort_by_key(|p| (p.y, p.x));
                unique.dedup();
                prop_assert_eq!(unique.len(), scene.food.len());
            }
        }
    }
}

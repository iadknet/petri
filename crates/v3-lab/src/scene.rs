//! Scenes: one seeded placement of start, food and barriers on an arena,
//! drawn so the assay's exposure predicate holds.
//!
//! Arenas: `sparse-food-v1` (food uniform around the centre), `wall-v1`
//! (a food block behind a barrier wall), `ring-v1` (a food block inside a
//! barrier ring with one gap) and a JSON layout file.

use rand::rngs::SmallRng;
use rand::Rng;
use v3_core::contracts::{Direction, Position};

use crate::eval::Scoring;
use crate::geodesic::{offset, step_toward, Field, Terrain};
use crate::layout::Layout;

/// Consecutive failed draws that make a point infeasible.
pub const REDRAW_LIMIT: u32 = 100;
/// Food cells are drawn at toroidal Chebyshev distance at least this far from
/// the start cell.
pub const MIN_START_DISTANCE: u16 = 2;
/// Built-in barrier arenas: the food block's centre lies this far along `D`.
pub const FOOD_BLOCK_DISTANCE: i32 = 5;
/// `wall-v1`: the wall's forward distance from the start.
pub const WALL_DISTANCE: i32 = 3;
/// The scale axis of `wall-v1` and `ring-v1` (at 4 the ring holds the start).
pub const SCALES: std::ops::RangeInclusive<u8> = 1..=3;
/// `D` is drawn uniformly over these, in this order.
const CARDINALS: [Direction; 4] = [Direction::N, Direction::E, Direction::S, Direction::W];

/// The capability an assay scores.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Assay {
    FoodSeeking,
    BarrierNavigation,
}

impl Assay {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::FoodSeeking => "food-seeking",
            Self::BarrierNavigation => "barrier-navigation",
        }
    }

    /// The built-in arena `--arena` defaults to.
    #[must_use]
    pub fn default_arena(self) -> ArenaId {
        match self {
            Self::FoodSeeking => ArenaId::SparseFoodV1,
            Self::BarrierNavigation => ArenaId::WallV1,
        }
    }

    /// The default scene score.
    #[must_use]
    pub fn scoring(self) -> Scoring {
        match self {
            Self::FoodSeeking => Scoring::BITES_AND_PROGRESS,
            Self::BarrierNavigation => Scoring::BARRIER_NAVIGATION,
        }
    }
}

/// A built-in arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ArenaId {
    #[value(name = "sparse-food-v1")]
    SparseFoodV1,
    #[value(name = "wall-v1")]
    WallV1,
    #[value(name = "ring-v1")]
    RingV1,
}

impl ArenaId {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SparseFoodV1 => "sparse-food-v1",
            Self::WallV1 => "wall-v1",
            Self::RingV1 => "ring-v1",
        }
    }
}

/// One point on an arena's axis: what a scene draw needs besides the size.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    SparseFood { fraction: f64 },
    Wall { scale: u8 },
    Ring { scale: u8 },
    Layout(Layout),
}

impl Geometry {
    /// `(food_fraction, scale)`: the point's axis value, the other null.
    #[must_use]
    pub fn axis(&self) -> (Option<f64>, Option<u8>) {
        match self {
            Self::SparseFood { fraction } => (Some(*fraction), None),
            Self::Wall { scale } | Self::Ring { scale } => (None, Some(*scale)),
            Self::Layout(_) => (None, None),
        }
    }

    /// The arena identifier recorded in provenance.
    #[must_use]
    pub fn arena_id(&self) -> &'static str {
        match self {
            Self::SparseFood { .. } => ArenaId::SparseFoodV1.name(),
            Self::Wall { .. } => ArenaId::WallV1.name(),
            Self::Ring { .. } => ArenaId::RingV1.name(),
            Self::Layout(_) => "layout",
        }
    }

    /// The canonical arena descriptor (`arena_version: 1`) at `size`: id,
    /// start, food type, the axis value (null when `with_axis` is false)
    /// and the geometry and sampling rules; a layout records its format,
    /// path and rows.
    #[must_use]
    pub fn descriptor(&self, size: u16, with_axis: bool) -> serde_json::Value {
        use serde_json::json;
        const REDRAW: &str =
            "a draw failing the exposure predicate redraws the placement from the scenes stream, the Simulation seed kept, up to 100 draws";
        const DIRECTION: &str =
            "D uniform over N, E, S, W from the scenes stream after the Simulation seed";
        const FOOD_BLOCK: &str = "the 3 x 3 cells centred at start + 5 D, row-major";
        let (fraction, scale) = if with_axis { self.axis() } else { (None, None) };
        let mut spec = json!({
            "arena_version": 1,
            "id": self.arena_id(),
            "size": size,
            "food_type": 0,
            "start": "centre",
            "food_fraction": fraction,
            "scale": scale,
        });
        let rules = match self {
            Self::SparseFood { .. } => json!({
                "food": "round(food_fraction x size^2) cells uniform without replacement at Chebyshev distance >= 2 from start",
                "redraw": REDRAW,
            }),
            Self::Wall { .. } => json!({
                "direction": DIRECTION,
                "food": FOOD_BLOCK,
                "barriers": "the 2 scale + 1 cells at forward 3, lateral -scale..scale",
                "redraw": REDRAW,
            }),
            Self::Ring { .. } => json!({
                "direction": DIRECTION,
                "food": FOOD_BLOCK,
                "barriers": "the cells at Chebyshev distance scale + 1 from start + 5 D minus one gap",
                "gap": "one draw uniform over the ring's non-corner cells in row-major order; D and the gap redraw together",
                "redraw": REDRAW,
            }),
            Self::Layout(layout) => {
                spec["start"] = json!([layout.start.x, layout.start.y]);
                spec["arena_format"] = json!(crate::layout::ARENA_FORMAT);
                spec["path"] = json!(layout.path);
                spec["rows"] = json!(layout.rows);
                json!({"scenes": "the file's cells with a fresh Simulation seed; no redraw"})
            }
        };
        spec["rules"] = rules;
        spec
    }
}

/// Everything a scene draw needs.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSpec {
    pub geometry: Geometry,
    pub size: u16,
    pub vision_radius: u8,
    pub assay: Assay,
}

impl SceneSpec {
    /// A food-seeking `sparse-food-v1` point (F01's draw).
    #[must_use]
    pub fn sparse(size: u16, fraction: f64, vision_radius: u8) -> Self {
        Self {
            geometry: Geometry::SparseFood { fraction },
            size,
            vision_radius,
            assay: Assay::FoodSeeking,
        }
    }

    /// Draw one scene from `rng`: a `Simulation` seed, then the arena's
    /// placement, redrawing (seed kept) until the exposure predicate holds.
    ///
    /// # Errors
    ///
    /// [`Infeasible`] after `REDRAW_LIMIT` consecutive failures, or when no
    /// draw can succeed (too many sparse cells; a layout failing the
    /// predicate).
    pub fn draw(&self, rng: &mut SmallRng) -> Result<Scene, Infeasible> {
        match &self.geometry {
            Geometry::SparseFood { fraction } => self.draw_sparse(rng, *fraction),
            Geometry::Wall { scale } => self.draw_barrier(rng, *scale, false),
            Geometry::Ring { scale } => self.draw_barrier(rng, *scale, true),
            Geometry::Layout(layout) => {
                let seed = rng.gen::<u64>();
                let scene = Scene {
                    seed,
                    start: layout.start,
                    food: layout.food.clone(),
                    barriers: layout.barriers.clone(),
                    redraws: 0,
                };
                if self.admits(&scene) {
                    Ok(scene)
                } else {
                    Err(Infeasible { redraws: 0 })
                }
            }
        }
    }

    /// The assay's exposure predicate on `scene`.
    #[must_use]
    pub fn admits(&self, scene: &Scene) -> bool {
        admits(self.assay, scene, self.size, self.vision_radius)
    }

    fn draw_sparse(&self, rng: &mut SmallRng, fraction: f64) -> Result<Scene, Infeasible> {
        let size = self.size;
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
            let scene = Scene {
                seed,
                start,
                food,
                barriers: Vec::new(),
                redraws,
            };
            if self.admits(&scene) {
                return Ok(scene);
            }
        }
        Err(Infeasible {
            redraws: REDRAW_LIMIT,
        })
    }

    /// `wall-v1` and `ring-v1`: `D`, then (ring) the gap, per draw.
    fn draw_barrier(&self, rng: &mut SmallRng, scale: u8, ring: bool) -> Result<Scene, Infeasible> {
        let size = self.size;
        let seed = rng.gen::<u64>();
        let start = centre(size);
        let k = i32::from(scale);
        for redraws in 0..REDRAW_LIMIT {
            let direction = CARDINALS[rng.gen_range(0..CARDINALS.len())];
            let (fx, fy) = direction.delta();
            // Lateral is `D` turned a quarter.
            let (lx, ly) = (-fy, fx);
            let at = |forward: i32, lateral: i32| {
                offset(
                    start,
                    (forward * fx + lateral * lx, forward * fy + lateral * ly),
                    size,
                )
            };
            let block = at(FOOD_BLOCK_DISTANCE, 0);
            let food = row_major(square(block, 1, size, |_, _| true));
            let barriers = if ring {
                let r = k + 1;
                let sides = row_major(square(block, r, size, |dx, dy| {
                    (dx.abs() == r) != (dy.abs() == r)
                }));
                let gap = sides[rng.gen_range(0..sides.len())];
                let mut cells = square(block, r, size, |dx, dy| dx.abs() == r || dy.abs() == r);
                cells.retain(|&cell| cell != gap);
                row_major(cells)
            } else {
                row_major((-k..=k).map(|lateral| at(WALL_DISTANCE, lateral)).collect())
            };
            let scene = Scene {
                seed,
                start,
                food,
                barriers,
                redraws,
            };
            if self.admits(&scene) {
                return Ok(scene);
            }
        }
        Err(Infeasible {
            redraws: REDRAW_LIMIT,
        })
    }
}

/// The cells at offsets `(dx, dy)` in `-r..=r` around `centre` that `keep`.
fn square(centre: Position, r: i32, size: u16, keep: impl Fn(i32, i32) -> bool) -> Vec<Position> {
    (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| keep(dx, dy))
        .map(|delta| offset(centre, delta, size))
        .collect()
}

/// Sort cells in row-major world order (y, then x).
fn row_major(mut cells: Vec<Position>) -> Vec<Position> {
    cells.sort_by_key(|cell| (cell.y, cell.x));
    cells
}

/// Toroidal Chebyshev distance on a `size × size` wrap world.
#[must_use]
pub fn torus_distance(a: Position, b: Position, size: u16) -> u16 {
    let axis = |p: u16, q: u16| {
        let d = p.abs_diff(q);
        d.min(size - d)
    };
    axis(a.x, b.x).max(axis(a.y, b.y))
}

/// The arena centre, where every built-in scene starts.
#[must_use]
pub fn centre(size: u16) -> Position {
    Position::new(size / 2, size / 2)
}

/// The Chebyshev-nearest of `food` from `at`, first in order on ties (F01's
/// target).
#[must_use]
pub fn nearest(food: &[Position], at: Position, size: u16) -> Option<Position> {
    food.iter()
        .copied()
        .min_by_key(|&cell| torus_distance(cell, at, size))
}

/// One seeded placement.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Scene {
    /// `Simulation` seed (one `scenes` draw).
    pub seed: u64,
    pub start: Position,
    /// Food cells of type 0 at `max_density`, in draw (or row-major) order.
    pub food: Vec<Position>,
    /// Barrier cells, set before the creature is placed.
    pub barriers: Vec<Position>,
    /// Draws rejected by the exposure predicate before this one.
    pub redraws: u32,
}

impl Scene {
    /// A barrier-free scene starting at the centre of a `size` arena.
    #[must_use]
    pub fn open(seed: u64, size: u16, food: Vec<Position>) -> Self {
        Self {
            seed,
            start: centre(size),
            food,
            barriers: Vec::new(),
            redraws: 0,
        }
    }
}

/// A point whose draws failed the exposure predicate `REDRAW_LIMIT` times
/// in a row (or cannot succeed at all).
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

/// Predicate (a): at least one food cell within `vision_radius` of `start`,
/// none within distance 1.
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

/// Predicate (c): the barrier-blind greedy walk — F01's `step_toward` its
/// target, the Chebyshev-nearest food (first in order on ties) — meets a
/// barrier cell before it stands on food.
#[must_use]
pub fn greedy_meets_barrier(start: Position, food: &[Position], terrain: &Terrain) -> bool {
    let size = terrain.size();
    let mut at = start;
    // Each step shortens the distance to the nearest food by one.
    for _ in 0..=size {
        let Some(target) = nearest(food, at, size) else {
            return false;
        };
        if target == at {
            return false;
        }
        let next = terrain.neighbor(at, step_toward(at, target, size));
        if terrain.is_blocked(next) {
            return true;
        }
        at = next;
    }
    false
}

/// The assay's exposure predicate: (a) for food seeking; (a), (b) a finite
/// geodesic from the start to food, and (c) for barrier navigation.
#[must_use]
pub fn admits(assay: Assay, scene: &Scene, size: u16, vision_radius: u8) -> bool {
    if !exposed(&scene.food, scene.start, size, vision_radius) {
        return false;
    }
    match assay {
        Assay::FoodSeeking => true,
        Assay::BarrierNavigation => {
            let terrain = Terrain::from_cells(size, &scene.barriers);
            Field::new(&terrain, &scene.food).at(scene.start).is_some()
                && greedy_meets_barrier(scene.start, &scene.food, &terrain)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{tests::rows, LayoutFile};
    use proptest::prelude::*;
    use rand::SeedableRng;

    fn draw_scene(
        rng: &mut SmallRng,
        size: u16,
        fraction: f64,
        vision_radius: u8,
    ) -> Result<Scene, Infeasible> {
        SceneSpec::sparse(size, fraction, vision_radius).draw(rng)
    }

    fn barrier_spec(geometry: Geometry) -> SceneSpec {
        SceneSpec {
            geometry,
            size: 64,
            vision_radius: 5,
            assay: Assay::BarrierNavigation,
        }
    }

    #[test]
    fn assay_name_is_its_serialized_kebab_case_value() {
        for (assay, name) in [
            (Assay::FoodSeeking, "food-seeking"),
            (Assay::BarrierNavigation, "barrier-navigation"),
        ] {
            assert_eq!(assay.name(), name);
            assert_eq!(serde_json::to_value(assay).unwrap(), name);
        }
    }

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

    #[test]
    fn a_barrier_free_sparse_arena_never_admits_barrier_navigation() {
        let spec = SceneSpec {
            assay: Assay::BarrierNavigation,
            ..SceneSpec::sparse(48, 0.05, 5)
        };
        assert_eq!(
            spec.draw(&mut SmallRng::seed_from_u64(1)),
            Err(Infeasible {
                redraws: REDRAW_LIMIT
            })
        );
    }

    #[test]
    fn a_wall_scene_places_the_food_block_behind_its_wall() {
        for scale in SCALES {
            for seed in 0..8 {
                let spec = barrier_spec(Geometry::Wall { scale });
                let scene = spec.draw(&mut SmallRng::seed_from_u64(seed)).unwrap();
                assert_eq!(scene.redraws, 0, "exposure holds by construction");
                assert_eq!(scene.start, centre(64));
                assert_eq!(scene.food.len(), 9);
                assert_eq!(scene.barriers.len(), 2 * usize::from(scale) + 1);
                let nearest_food = scene
                    .food
                    .iter()
                    .map(|&f| torus_distance(f, scene.start, 64))
                    .min();
                assert_eq!(nearest_food, Some(4));
                assert!(scene
                    .barriers
                    .iter()
                    .all(|&b| torus_distance(b, scene.start, 64) == 3));
                assert!(spec.admits(&scene));
                let mut sorted = scene.food.clone();
                sorted.sort_by_key(|p| (p.y, p.x));
                assert_eq!(sorted, scene.food, "row-major food");
            }
        }
    }

    #[test]
    fn every_wall_direction_and_scale_meets_the_predicate() {
        let start = centre(64);
        for scale in SCALES {
            for direction in CARDINALS {
                let (fx, fy) = direction.delta();
                let k = i32::from(scale);
                let block = offset(start, (5 * fx, 5 * fy), 64);
                let food = row_major(square(block, 1, 64, |_, _| true));
                let barriers = (-k..=k)
                    .map(|l| offset(start, (3 * fx - l * fy, 3 * fy + l * fx), 64))
                    .collect();
                let scene = Scene {
                    seed: 0,
                    start,
                    food,
                    barriers,
                    redraws: 0,
                };
                assert!(
                    admits(Assay::BarrierNavigation, &scene, 64, 5),
                    "{direction:?} {scale}"
                );
            }
        }
    }

    /// Seeded fixtures: one accepted ring draw with no redraw, and one draw
    /// whose first gap let the greedy walk through, redrawn with the
    /// `Simulation` seed kept.
    #[test]
    fn ring_draws_redraw_a_gap_the_greedy_walk_passes() {
        let spec = barrier_spec(Geometry::Ring { scale: 2 });
        let draws: Vec<Scene> = (0..64)
            .map(|seed| spec.draw(&mut SmallRng::seed_from_u64(seed)).unwrap())
            .collect();
        let accepted = draws.iter().position(|s| s.redraws == 0).unwrap();
        let rejected = draws.iter().position(|s| s.redraws > 0).unwrap();
        for (seed, scene) in [(accepted, &draws[accepted]), (rejected, &draws[rejected])] {
            assert!(spec.admits(scene));
            // 5 × 5 food-free square ring, side 2k + 3 = 7: 24 cells − gap.
            assert_eq!(scene.barriers.len(), 23);
            let mut rng = SmallRng::seed_from_u64(seed as u64);
            assert_eq!(scene.seed, rng.gen::<u64>(), "the seed is the first draw");
        }
        // The first draw of the rejected seed, replayed: its gap is passable
        // by the greedy walk (predicate (c) fails).
        let mut rng = SmallRng::seed_from_u64(rejected as u64);
        let _seed: u64 = rng.gen();
        let direction = CARDINALS[rng.gen_range(0..4)];
        let (fx, fy) = direction.delta();
        let block = offset(centre(64), (5 * fx, 5 * fy), 64);
        let sides = row_major(square(block, 3, 64, |dx, dy| {
            (dx.abs() == 3) != (dy.abs() == 3)
        }));
        assert_eq!(sides.len(), 20);
        let gap = sides[rng.gen_range(0..sides.len())];
        let mut ring = square(block, 3, 64, |dx, dy| dx.abs() == 3 || dy.abs() == 3);
        ring.retain(|&c| c != gap);
        let food = row_major(square(block, 1, 64, |_, _| true));
        assert!(!greedy_meets_barrier(
            centre(64),
            &food,
            &Terrain::from_cells(64, &ring)
        ));
        assert_eq!(accepted, 0, "pinned: seed 0 is accepted outright");
    }

    #[test]
    fn layouts_draw_their_cells_and_fail_without_redraws() {
        let layout = |cells: &[(usize, usize, char)]| {
            Layout::parse(
                "l.json".into(),
                LayoutFile {
                    arena_format: 1,
                    rows: rows(16, cells),
                },
            )
            .unwrap()
        };
        // A wall between the start (4, 8) and food (8, 8).
        let wall: Vec<(usize, usize, char)> = (6..=10)
            .map(|y| (6, y, '#'))
            .chain([(4, 8, 'S'), (8, 8, 'F')])
            .collect();
        let spec = SceneSpec {
            geometry: Geometry::Layout(layout(&wall)),
            size: 16,
            vision_radius: 5,
            assay: Assay::BarrierNavigation,
        };
        let scene = spec.draw(&mut SmallRng::seed_from_u64(2)).unwrap();
        assert_eq!(scene.start, Position::new(4, 8));
        assert_eq!(scene.food, [Position::new(8, 8)]);
        assert_eq!(scene.barriers.len(), 5);
        // The food enclosed: exposed but unreachable, so infeasible.
        let enclosed: Vec<(usize, usize, char)> =
            square(Position::new(8, 8), 1, 16, |dx, dy| dx != 0 || dy != 0)
                .into_iter()
                .map(|p| (usize::from(p.x), usize::from(p.y), '#'))
                .chain([(4, 8, 'S'), (8, 8, 'F')])
                .collect();
        let spec = SceneSpec {
            geometry: Geometry::Layout(layout(&enclosed)),
            ..spec
        };
        assert_eq!(
            spec.draw(&mut SmallRng::seed_from_u64(2)),
            Err(Infeasible { redraws: 0 })
        );
        // Food seeking keeps predicate (a) only.
        let seeking = SceneSpec {
            assay: Assay::FoodSeeking,
            ..spec
        };
        assert!(seeking.draw(&mut SmallRng::seed_from_u64(2)).is_ok());
    }

    #[test]
    fn distinct_arenas_and_axis_values_never_share_a_descriptor_hash() {
        let layout = Layout::parse(
            "l.json".into(),
            LayoutFile {
                arena_format: 1,
                rows: rows(16, &[(1, 1, 'S'), (5, 5, 'F')]),
            },
        )
        .unwrap();
        let hash = |geometry: &Geometry, size: u16| {
            crate::sha256_hex(&serde_json::to_vec(&geometry.descriptor(size, true)).unwrap())
        };
        let points = [
            Geometry::SparseFood { fraction: 0.04 },
            Geometry::SparseFood { fraction: 0.08 },
            Geometry::Wall { scale: 1 },
            Geometry::Wall { scale: 2 },
            Geometry::Ring { scale: 1 },
            Geometry::Ring { scale: 2 },
            Geometry::Layout(layout),
        ];
        let mut hashes: Vec<String> = points.iter().map(|g| hash(g, 64)).collect();
        hashes.push(hash(&points[2], 48));
        let count = hashes.len();
        hashes.sort();
        hashes.dedup();
        assert_eq!(hashes.len(), count, "two descriptors share a hash");
        // Serialized with sorted keys: the bytes do not depend on insertion.
        let text = serde_json::to_string(&points[4].descriptor(64, true)).unwrap();
        assert!(
            text.starts_with(
                r#"{"arena_version":1,"food_fraction":null,"food_type":0,"id":"ring-v1""#
            ),
            "{text}"
        );
        assert_eq!(
            points[3].descriptor(64, false)["scale"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn the_greedy_walk_reaches_open_food_and_stops_at_a_wall() {
        let open = Terrain::from_cells(32, &[]);
        let start = Position::new(10, 10);
        let food = [Position::new(15, 12)];
        assert!(!greedy_meets_barrier(start, &food, &open));
        let wall = Terrain::from_cells(32, &[Position::new(13, 12)]);
        // Steps: (11, 11), (12, 12), then E into (13, 12).
        assert!(greedy_meets_barrier(start, &food, &wall));
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

        #[test]
        fn every_ring_scene_is_disjoint_and_admitted(
            seed in any::<u64>(),
            scale in SCALES,
            size in 48u16..=64,
        ) {
            let spec = SceneSpec { size, ..barrier_spec(Geometry::Ring { scale }) };
            let scene = spec.draw(&mut SmallRng::seed_from_u64(seed));
            let scene = scene.expect("a ring always has an admissible gap");
            prop_assert!(spec.admits(&scene));
            prop_assert!(!scene.barriers.contains(&scene.start));
            prop_assert!(scene.food.iter().all(|f| !scene.barriers.contains(f)));
            let ring = 8 * (usize::from(scale) + 1);
            prop_assert_eq!(scene.barriers.len(), ring - 1);
        }
    }
}

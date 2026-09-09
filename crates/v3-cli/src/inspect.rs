//! `v3-cli world inspect`: read a saved world recipe's tick-zero map (T12.F04).
//!
//! Everything here observes one seeded, unrun simulation. No tick runs, so the
//! readings are the map the goal profile starts each baseline world from.

use serde::Serialize;

use crate::{fraction_or_undefined, mean_or_undefined, six};
use v3_core::config::{OrdinaryFoodTypeId, SimulationConfig};
use v3_core::contracts::Position;
use v3_core::kernel::PassableConnectivity;
use v3_core::simulation::Simulation;

/// The longest side of a written preview, in pixels, per panel.
pub const PREVIEW_MAX_SIDE: u32 = 512;

/// One food type's tick-zero habitat and standing crop.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoodTypeReading {
    pub type_idx: u16,
    pub name: String,
    /// `energy_per_unit`, or the shared Eat reward when the type inherits it —
    /// the same fallback [`v3_core::simulation::actions::apply_typed_eat`] uses.
    pub effective_energy_per_unit: f32,
    /// Passable cells whose effective tick-zero fertility is above zero.
    pub fertile_cells: u64,
    /// `fertile_cells` against every cell in the world, barriers included.
    pub fertile_fraction_of_world: String,
    /// Mean effective fertility over `fertile_cells` alone.
    pub mean_fertility: String,
    /// Cells holding any of this type's food at tick zero.
    pub food_cells: u64,
    /// Total seeded density times this type's effective energy per unit.
    pub standing_energy: String,
}

/// One recipe's complete tick-zero reading.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldInspection {
    pub recipe_path: String,
    pub seed: u64,
    pub world_width: u16,
    pub world_height: u16,
    pub edge_mode: String,
    /// The pinned map seed, or the run seed when the recipe pins none.
    pub world_seed: u64,
    /// Whether `world_seed` came from the recipe rather than the run seed.
    pub world_seed_pinned: bool,
    pub passable_connectivity: PassableConnectivity,
    pub founders_placed: u64,
    pub food_types: Vec<FoodTypeReading>,
}

/// Read a seeded, unrun world. `recipe_path` and `seed` are recorded as given
/// so the JSON identifies what produced it.
#[must_use]
pub fn read_world(sim: &Simulation, recipe_path: &str, seed: u64) -> WorldInspection {
    let (width, height) = (sim.world.width, sim.world.height);
    let total_cells = u64::from(width) * u64::from(height);
    let food_types = sim
        .config
        .world
        .food
        .types
        .iter()
        .enumerate()
        .map(|(index, food_type)| {
            let type_idx = OrdinaryFoodTypeId::new(index as u16);
            let fertility = sim.world.food().effective_fertility_grid(type_idx, 0);
            let effective_energy_per_unit = food_type
                .energy_per_unit
                .unwrap_or(sim.config.energy.costs.eat_reward_per_food);
            let mut fertile_cells = 0u64;
            let mut fertility_sum = 0.0f64;
            let mut food_cells = 0u64;
            let mut density_sum = 0.0f64;
            for y in 0..height {
                for x in 0..width {
                    let pos = Position::new(x, y);
                    let value = *fertility.get(x, y);
                    if value > 0.0 && !sim.world.is_barrier(pos) {
                        fertile_cells += 1;
                        fertility_sum += f64::from(value);
                    }
                    let density = sim.world.food_at_type(pos, type_idx);
                    if density > 0.0 {
                        food_cells += 1;
                        density_sum += f64::from(density);
                    }
                }
            }
            FoodTypeReading {
                type_idx: index as u16,
                name: food_type.name.clone(),
                effective_energy_per_unit,
                fertile_cells,
                fertile_fraction_of_world: fraction_or_undefined(fertile_cells, total_cells),
                mean_fertility: mean_or_undefined(fertility_sum, fertile_cells),
                food_cells,
                standing_energy: six(density_sum * f64::from(effective_energy_per_unit)),
            }
        })
        .collect();

    WorldInspection {
        recipe_path: recipe_path.to_string(),
        seed,
        world_width: width,
        world_height: height,
        edge_mode: format!("{:?}", sim.world.edge_mode),
        world_seed: sim.config.world.world_seed.unwrap_or(seed),
        world_seed_pinned: sim.config.world.world_seed.is_some(),
        passable_connectivity: sim.world.passable_connectivity(),
        founders_placed: sim.creatures.len() as u64,
        food_types,
    }
}

/// The integer downsample factor that keeps a panel's longer side within
/// [`PREVIEW_MAX_SIDE`].
#[must_use]
pub fn preview_downsample_factor(width: u16, height: u16) -> u32 {
    let longest = u32::from(width.max(height));
    longest.div_ceil(PREVIEW_MAX_SIDE).max(1)
}

/// The pixel size of the two-panel preview for a world of this size: the
/// habitat panel and the food panel side by side, each downsampled.
#[must_use]
pub fn preview_dimensions(width: u16, height: u16) -> (u32, u32) {
    let factor = preview_downsample_factor(width, height);
    let panel_width = u32::from(width).div_ceil(factor);
    let panel_height = u32::from(height).div_ceil(factor);
    (panel_width * 2, panel_height)
}

/// Barrier gray, and the two food-type hues blended over an empty-ground base.
const BARRIER_RGB: [f32; 3] = [110.0, 116.0, 128.0];
const GROUND_RGB: [f32; 3] = [21.0, 29.0, 42.0];
const TYPE_RGB: [[f32; 3]; 2] = [[49.0, 200.0, 100.0], [231.0, 130.0, 42.0]];

/// A pooled value as a share of the most this world can hold. A nonpositive
/// ceiling leaves no share to report and reads as empty rather than as an
/// infinity the color blend would turn into black.
fn intensity(value: f32, ceiling: f32) -> f32 {
    if ceiling > 0.0 {
        value / ceiling
    } else {
        0.0
    }
}

/// One downsampled cell's color: each of the first two food types adds its hue
/// over bare ground in proportion to the pooled value, and the result fades to
/// barrier gray in proportion to how much of the block is walled — a block of
/// nothing but barrier is solid gray, a block with none is untouched.
fn blend(barrier_fraction: f32, intensities: [f32; 2]) -> [u8; 3] {
    let mut rgb = GROUND_RGB;
    for (hue, intensity) in TYPE_RGB.iter().zip(intensities) {
        let scaled = intensity.clamp(0.0, 1.0);
        for (channel, hue_channel) in rgb.iter_mut().zip(hue) {
            *channel += hue_channel * scaled;
        }
    }
    let gray = barrier_fraction.clamp(0.0, 1.0);
    std::array::from_fn(|channel| {
        let color = rgb[channel].clamp(0.0, 255.0);
        (color * (1.0 - gray) + BARRIER_RGB[channel] * gray) as u8
    })
}

/// Render the two-panel PNG preview: habitat (barriers and per-type effective
/// tick-zero fertility) on the left, tick-zero food density on the right.
///
/// Every layer, barriers included, is mean-pooled over each downsample block,
/// so a block reads gray in proportion to how much of it is walled and a sparse
/// rubble field cannot render as solid rock. Only the first two food types are
/// drawn.
#[must_use]
pub fn render_preview(sim: &Simulation) -> Vec<u8> {
    let (width, height) = (sim.world.width, sim.world.height);
    let factor = preview_downsample_factor(width, height);
    let (image_width, image_height) = preview_dimensions(width, height);
    let panel_width = image_width / 2;

    let drawn_types = sim.config.world.food.types.len().min(TYPE_RGB.len());
    let fertility: Vec<_> = (0..drawn_types)
        .map(|index| {
            sim.world
                .food()
                .effective_fertility_grid(OrdinaryFoodTypeId::new(index as u16), 0)
        })
        .collect();
    // Normalizers: fertility against its own effective ceiling, density against
    // the shared per-cell maximum, so both panels read as "share of the most
    // this world can hold".
    let fertility_ceiling = if sim.config.world.food.fertility.enabled {
        sim.config.world.food.fertility.max_fertility
    } else {
        1.0
    };
    let density_ceiling = sim.config.world.food.shared.max_density;

    let mut pixels = vec![0u8; (image_width * image_height * 3) as usize];
    for out_y in 0..image_height {
        for out_x in 0..panel_width {
            let x0 = out_x * factor;
            let y0 = out_y * factor;
            let x1 = (x0 + factor).min(u32::from(width));
            let y1 = (y0 + factor).min(u32::from(height));
            let cells = u64::from(x1 - x0) * u64::from(y1 - y0);
            let mut barrier_sum = 0.0f32;
            let mut fertility_sums = [0.0f32; 2];
            let mut density_sums = [0.0f32; 2];
            for y in y0..y1 {
                for x in x0..x1 {
                    let pos = Position::new(x as u16, y as u16);
                    if sim.world.is_barrier(pos) {
                        barrier_sum += 1.0;
                    }
                    for index in 0..drawn_types {
                        fertility_sums[index] += *fertility[index].get(x as u16, y as u16);
                        density_sums[index] += sim
                            .world
                            .food_at_type(pos, OrdinaryFoodTypeId::new(index as u16));
                    }
                }
            }
            let mean = |sum: f32| {
                if cells == 0 {
                    0.0
                } else {
                    sum / cells as f32
                }
            };
            let barrier_fraction = mean(barrier_sum);
            let habitat = blend(
                barrier_fraction,
                fertility_sums.map(|sum| intensity(mean(sum), fertility_ceiling)),
            );
            let food = blend(
                barrier_fraction,
                density_sums.map(|sum| intensity(mean(sum), density_ceiling)),
            );
            for (panel_x, rgb) in [(out_x, habitat), (out_x + panel_width, food)] {
                let offset = ((out_y * image_width + panel_x) * 3) as usize;
                pixels[offset..offset + 3].copy_from_slice(&rgb);
            }
        }
    }

    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image_width, image_height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("an in-memory PNG header always writes");
    writer
        .write_image_data(&pixels)
        .expect("an in-memory PNG body always writes");
    drop(writer);
    out
}

/// Resolve a recipe patch over the production defaults into its applied
/// config, exactly as the goal profile resolves a baseline world's recipe.
///
/// [`v3_core::config::resolve_config`] is the whole chain: deep-merge over the
/// defaults, `normalize`, then `apply_startup_overrides`. The goal profile then
/// forces its own world size and founder count onto the result; the checked-in
/// baseline recipes set neither, so what this returns is the map that profile
/// runs. A recipe that did set them would be inspected as written.
pub fn resolve_baseline_world(
    patch: serde_json::Value,
    recipe_path: &str,
) -> Result<SimulationConfig, String> {
    v3_core::config::resolve_config(&SimulationConfig::default(), patch)
        .map_err(|error| format!("invalid recipe {recipe_path}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::simulation::seed_simulation;

    /// A world small enough to count by hand, with two food types and a
    /// barrier column, seeded from a pinned world seed.
    fn small_sim() -> Simulation {
        let config = v3_core::config::resolve_config(
            &SimulationConfig::default(),
            serde_json::json!({
                "world": {
                    "width": 16,
                    "height": 12,
                    "world_seed": 7,
                    "food": {
                        "types": [
                            {"name": "Grass", "color": "#22c55e", "initial_density": 1.0,
                             "initial_coverage": 0.5, "energy_per_unit": 2.0},
                            {"name": "Fruit", "color": "#f97316", "initial_density": 1.0,
                             "initial_coverage": 0.1, "energy_per_unit": 8.0}
                        ]
                    }
                },
                "population": {"initial_creatures": 5}
            }),
        )
        .unwrap();
        let mut sim = seed_simulation(config, 11);
        for y in 0..12u16 {
            sim.world.set_barrier(Position::new(4, y), true);
        }
        sim
    }

    #[test]
    fn readings_equal_direct_grid_counts_on_a_small_recipe() {
        let sim = small_sim();
        let reading = read_world(&sim, "small.json", 11);

        assert_eq!(reading.recipe_path, "small.json");
        assert_eq!(reading.seed, 11);
        assert_eq!(reading.world_width, 16);
        assert_eq!(reading.world_height, 12);
        assert_eq!(reading.world_seed, 7);
        assert!(reading.world_seed_pinned);
        assert_eq!(reading.founders_placed, sim.creatures.len() as u64);
        assert_eq!(
            reading.passable_connectivity,
            sim.world.passable_connectivity()
        );
        assert_eq!(reading.food_types.len(), 2);

        for (index, food) in reading.food_types.iter().enumerate() {
            let type_idx = OrdinaryFoodTypeId::new(index as u16);
            let fertility = sim.world.food().effective_fertility_grid(type_idx, 0);
            let mut fertile = 0u64;
            let mut fertility_sum = 0.0f64;
            let mut cells = 0u64;
            let mut density_sum = 0.0f64;
            for y in 0..12u16 {
                for x in 0..16u16 {
                    let pos = Position::new(x, y);
                    if *fertility.get(x, y) > 0.0 && !sim.world.is_barrier(pos) {
                        fertile += 1;
                        fertility_sum += f64::from(*fertility.get(x, y));
                    }
                    let density = sim.world.food_at_type(pos, type_idx);
                    if density > 0.0 {
                        cells += 1;
                        density_sum += f64::from(density);
                    }
                }
            }
            assert_eq!(food.fertile_cells, fertile, "type {index} fertile cells");
            assert_eq!(food.food_cells, cells, "type {index} food cells");
            assert_eq!(
                food.fertile_fraction_of_world,
                six(fertile as f64 / (16.0 * 12.0))
            );
            assert_eq!(
                food.mean_fertility,
                mean_or_undefined(fertility_sum, fertile),
                "type {index} mean fertility is over fertile cells alone"
            );
            assert_eq!(
                food.standing_energy,
                six(density_sum * f64::from(food.effective_energy_per_unit))
            );
            assert!(cells > 0, "type {index} must be seeded somewhere");
        }
        assert_eq!(reading.food_types[0].effective_energy_per_unit, 2.0);
        assert_eq!(reading.food_types[1].effective_energy_per_unit, 8.0);
    }

    #[test]
    fn a_food_type_with_no_energy_override_inherits_the_shared_eat_reward() {
        let config = v3_core::config::resolve_config(
            &SimulationConfig::default(),
            serde_json::json!({"world": {"width": 8, "height": 8}}),
        )
        .unwrap();
        let shared = config.energy.costs.eat_reward_per_food;
        let sim = seed_simulation(config, 3);
        let reading = read_world(&sim, "default.json", 3);
        assert_eq!(reading.food_types[0].effective_energy_per_unit, shared);
        assert_eq!(
            (reading.world_seed, reading.world_seed_pinned),
            (3, false),
            "an unpinned map falls back to the run seed, as seeding does"
        );
    }

    #[test]
    fn downsampling_keeps_each_panel_within_the_pixel_budget() {
        assert_eq!(preview_downsample_factor(16, 12), 1);
        assert_eq!(preview_downsample_factor(512, 512), 1);
        assert_eq!(preview_downsample_factor(513, 512), 2);
        assert_eq!(preview_downsample_factor(1600, 1600), 4);
        assert_eq!(preview_dimensions(1600, 1600), (800, 400));
        assert_eq!(
            preview_dimensions(1023, 500),
            (2 * 512, 250),
            "an odd side rounds up rather than dropping its last strip"
        );
    }

    #[test]
    fn the_preview_decodes_to_the_downsampled_two_panel_size() {
        let sim = small_sim();
        let bytes = render_preview(&sim);
        let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
        let mut reader = decoder.read_info().expect("the preview must be a PNG");
        let info = reader.info();
        let (expected_width, expected_height) = preview_dimensions(16, 12);
        assert_eq!((info.width, info.height), (expected_width, expected_height));
        assert_eq!(info.color_type, png::ColorType::Rgb);
        let mut pixels = vec![0u8; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut pixels).expect("the preview decodes");
        assert_eq!(
            frame.buffer_size(),
            (expected_width * expected_height * 3) as usize
        );

        // The barrier column is drawn gray in both panels at factor 1.
        let pixel_at = |x: u32, y: u32| {
            let offset = ((y * expected_width + x) * 3) as usize;
            [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
        };
        let barrier = BARRIER_RGB.map(|channel| channel as u8);
        assert_eq!(pixel_at(4, 0), barrier, "habitat panel draws the barrier");
        assert_eq!(
            pixel_at(4 + expected_width / 2, 0),
            barrier,
            "food panel draws the same barrier"
        );
    }

    /// A world wide enough to need downsampling, with one food type, no
    /// fertility layer (so habitat intensity is a known constant) and a
    /// per-cell maximum that is not 1.0 (so normalizing by it is visible).
    fn wide_sim() -> Simulation {
        let config = v3_core::config::resolve_config(
            &SimulationConfig::default(),
            serde_json::json!({
                "world": {
                    "width": 2048,
                    "height": 16,
                    "food": {
                        "shared": {"max_density": 4.0},
                        "fertility": {"enabled": false},
                        "types": [{"name": "Grass", "color": "#22c55e",
                                   "initial_density": 1.0, "initial_coverage": 0.0}]
                    }
                },
                "population": {"initial_creatures": 1}
            }),
        )
        .unwrap();
        seed_simulation(config, 5)
    }

    fn decode(bytes: &[u8]) -> Vec<u8> {
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = decoder.read_info().expect("the preview must be a PNG");
        let mut pixels = vec![0u8; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).expect("the preview decodes");
        pixels
    }

    /// Barriers are mean-pooled: a block reads gray in proportion to how much
    /// of it is walled, so a sparse rubble field cannot render as solid rock.
    /// Fertility is disabled in `wide_sim`, so every cell — barrier or not —
    /// carries full habitat intensity and the gray is the only variable.
    #[test]
    fn a_downsampled_block_grays_in_proportion_to_its_barrier_fraction() {
        let mut sim = wide_sim();
        assert_eq!(preview_downsample_factor(2048, 16), 4);
        // Block (5, 0) covers x 20..24, y 0..4: wall all sixteen of its cells.
        for y in 0..4u16 {
            for x in 20..24u16 {
                sim.world.set_barrier(Position::new(x, y), true);
            }
        }
        // Block (6, 0) covers x 24..28: wall the eight cells in its top half.
        for y in 0..2u16 {
            for x in 24..28u16 {
                sim.world.set_barrier(Position::new(x, y), true);
            }
        }

        let pixels = decode(&render_preview(&sim));
        let (image_width, image_height) = preview_dimensions(2048, 16);
        assert_eq!(pixels.len(), (image_width * image_height * 3) as usize);
        let at = |x: u32, y: u32| {
            let offset = ((y * image_width + x) * 3) as usize;
            [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
        };
        let gray = BARRIER_RGB.map(|channel| channel as u8);
        let full = blend(0.0, [1.0, 0.0]);
        let bare = blend(0.0, [0.0, 0.0]);
        assert_ne!(full, bare, "the hue must be visible");

        // Habitat panel: solid gray, half gray, and untouched.
        assert_eq!(at(5, 0), gray, "an all-barrier block is solid gray");
        assert_eq!(at(6, 0), blend(0.5, [1.0, 0.0]), "half-walled reads half");
        assert_eq!(at(7, 0), full, "a block with no barrier is unchanged");
        assert!(
            (0..3)
                .all(|c| at(6, 0)[c] > full[c].min(gray[c]) && at(6, 0)[c] < full[c].max(gray[c])),
            "the half-walled block lies strictly between {full:?} and {gray:?}"
        );
        // Barriers are pooled inside their own block, not smeared into the next.
        assert_eq!(at(4, 0), full, "the block before the walled one is clear");
        assert_eq!(at(5, 1), full, "the row below the walled block is clear");

        // Food panel: nothing was seeded, so the same grays sit over bare ground.
        let food = image_width / 2;
        assert_eq!(at(food + 5, 0), gray);
        assert_eq!(at(food + 6, 0), blend(0.5, [0.0, 0.0]));
        assert_eq!(at(food + 7, 0), bare);
    }

    #[test]
    fn a_food_block_is_the_mean_over_its_own_cells_normalized_by_the_maximum() {
        let mut sim = wide_sim();
        let max = sim.config.world.food.shared.max_density;
        let grass = OrdinaryFoodTypeId::default();
        // Block (2, 0) covers x 8..12, y 0..4: fill it, so its mean is `max`.
        for y in 0..4u16 {
            for x in 8..12u16 {
                sim.world.set_food_type(Position::new(x, y), grass, max);
            }
        }
        // Block (5, 0) covers x 20..24: four of its sixteen cells at half the
        // maximum, so its mean is an eighth of the maximum.
        for x in 20..24u16 {
            sim.world
                .set_food_type(Position::new(x, 0), grass, max / 2.0);
        }

        let pixels = decode(&render_preview(&sim));
        let (image_width, _) = preview_dimensions(2048, 16);
        let food = |block_x: u32, block_y: u32| {
            let offset = ((block_y * image_width + image_width / 2 + block_x) * 3) as usize;
            [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
        };
        assert_eq!(food(2, 0), blend(0.0, [1.0, 0.0]), "a full block is full");
        assert_eq!(
            food(5, 0),
            blend(0.0, [0.125, 0.0]),
            "four of sixteen cells at half the maximum is an eighth of it"
        );
        assert_ne!(
            blend(0.0, [0.125, 0.0]),
            blend(0.0, [1.0, 0.0]),
            "the two intensities must be distinguishable"
        );
        let bare = blend(0.0, [0.0, 0.0]);
        assert_eq!(food(1, 0), bare, "the block before the filled one is bare");
        assert_eq!(food(3, 0), bare, "and so is the one after it");
        assert_eq!(food(2, 1), bare, "the row below the filled block is bare");
    }

    /// `fertile_cells` counts a cell only when its effective fertility is
    /// strictly above zero, which a blob layer over a zero floor makes visible.
    #[test]
    fn fertile_cells_exclude_passable_cells_whose_fertility_is_exactly_zero() {
        let config = v3_core::config::resolve_config(
            &SimulationConfig::default(),
            serde_json::json!({
                "world": {
                    "width": 64,
                    "height": 64,
                    "world_seed": 9,
                    "food": {
                        "types": [{"name": "Grass", "color": "#22c55e",
                                   "initial_density": 1.0, "initial_coverage": 0.1}],
                        "fertility": {
                            "min_fertility": 0.0,
                            "max_fertility": 2.0,
                            "layers": [{
                                "target": "AllFoods",
                                "weight": 1.0,
                                "algorithm": {"PoissonBlobs": {"blob_count": 3, "min_radius": 4,
                                                               "max_radius": 8, "falloff": 0.5,
                                                               "seed": null}}
                            }]
                        }
                    }
                },
                "population": {"initial_creatures": 1}
            }),
        )
        .unwrap();
        let sim = seed_simulation(config, 3);
        let grid = sim
            .world
            .food()
            .effective_fertility_grid(OrdinaryFoodTypeId::default(), 0);
        let barren = (0..64u16)
            .flat_map(|y| (0..64u16).map(move |x| (x, y)))
            .filter(|(x, y)| *grid.get(*x, *y) == 0.0)
            .count();
        assert!(barren > 0, "the fixture must contain infertile ground");

        let reading = read_world(&sim, "blobs.json", 3);
        let total = 64 * 64;
        assert_eq!(reading.food_types[0].fertile_cells, total - barren as u64);
        assert!(
            reading.food_types[0].fertile_cells < total,
            "cells at exactly zero fertility are not fertile"
        );
    }

    #[test]
    fn a_baseline_recipe_that_sets_no_size_resolves_to_the_goal_profile_world() {
        let config = resolve_baseline_world(
            serde_json::json!({"world": {"world_seed": 4242}}),
            "experiments/worlds/example.json",
        )
        .unwrap();
        let goal = crate::bench::goal_profile_params();
        assert_eq!(config.world.width, goal.width);
        assert_eq!(config.world.height, goal.height);
        assert_eq!(config.population.initial_creatures, goal.founders);
        assert_eq!(config.world.world_seed, Some(4242));
        assert_eq!(
            v3_core::config::config_digest(&config),
            v3_core::config::config_digest(&crate::bench::build_config(
                &crate::bench::ProfileParams {
                    recipe: Some(crate::bench::Recipe {
                        path: "experiments/worlds/example.json".to_string(),
                        config: config.clone(),
                    }),
                    ..goal
                }
            )),
            "the goal profile's own resolution of the same recipe is byte-identical"
        );
    }

    #[test]
    fn an_unreadable_recipe_is_a_validation_error() {
        let error = resolve_baseline_world(serde_json::json!([]), "broken.json").unwrap_err();
        assert!(error.contains("broken.json"), "{error}");
    }

    /// The exact colors the preview draws, pinned as literals: an expectation
    /// computed by `blend` itself would move with any change to `blend`.
    #[test]
    fn blend_adds_each_type_hue_over_bare_ground_and_fades_to_the_barrier_gray() {
        assert_eq!(blend(0.0, [0.0, 0.0]), [21, 29, 42]);
        assert_eq!(blend(0.0, [1.0, 0.0]), [70, 229, 142]);
        assert_eq!(blend(0.0, [0.0, 1.0]), [252, 159, 84]);
        assert_eq!(blend(0.0, [0.5, 0.0]), [45, 129, 92]);
        assert_eq!(
            blend(0.0, [1.0, 1.0]),
            [255, 255, 184],
            "two hues add and clamp per channel"
        );
        assert_eq!(
            blend(0.0, [2.0, 0.0]),
            [70, 229, 142],
            "an intensity above one clamps rather than overflowing the hue"
        );
        assert_eq!(
            blend(1.0, [1.0, 1.0]),
            [110, 116, 128],
            "an entirely walled block is solid gray whatever grows under it"
        );
        assert_eq!(
            blend(0.5, [0.0, 0.0]),
            [65, 72, 85],
            "half a block walled is halfway from its color to the gray"
        );
        assert_eq!(blend(0.5, [1.0, 0.0]), [90, 172, 135]);
        assert_eq!(blend(0.25, [1.0, 0.0]), [80, 200, 138]);
        assert_eq!(
            blend(2.0, [1.0, 0.0]),
            [110, 116, 128],
            "a fraction above one clamps rather than overshooting the gray"
        );
    }

    #[test]
    fn intensity_is_a_share_of_the_ceiling_and_empty_without_one() {
        assert_eq!(intensity(2.0, 4.0), 0.5);
        assert_eq!(intensity(4.0, 4.0), 1.0);
        assert_eq!(
            intensity(1.0, 0.0),
            0.0,
            "a zero ceiling has no share to report"
        );
        assert_eq!(intensity(1.0, -2.0), 0.0);
    }
}

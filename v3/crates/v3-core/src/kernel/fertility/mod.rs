//! Fertility map generation: algorithms, mixing, and annealing.
//!
//! The fertility layer assigns each cell a raw value in [-1, 1] that is later
//! mapped to an effective fertility multiplier controlling food growth rate.

mod annealing;
mod fbm;
mod mixing;
mod poisson_blobs;
mod uniform;

pub use annealing::{effective_fertility_range, map_fertility};
pub use fbm::generate_fbm;
pub use mixing::mix_layers;
pub use poisson_blobs::generate_poisson_blobs;
pub use uniform::generate_uniform;

use crate::config::{FertilityAlgorithm, FertilityConfig};
use crate::kernel::Grid;

/// Generate a raw fertility grid from the configured layers.
///
/// If fertility is disabled, returns a grid filled with 0.0.
/// If no layers are configured, uses a default PoissonBlobs layer.
/// Values are in [-1, 1] (pre-annealing/mapping).
///
/// Each algorithm that requires randomness creates its own RNG seeded from
/// `world_seed` plus a per-layer index for isolation.
pub fn generate_fertility(
    width: u16,
    height: u16,
    config: &FertilityConfig,
    world_seed: u64,
) -> Grid<f32> {
    if !config.enabled {
        return Grid::new(width, height, 0.0);
    }

    let layers = if config.layers.is_empty() {
        // Use the default PoissonBlobs algorithm.
        let default_algo = FertilityAlgorithm::default();
        vec![crate::config::FertilityLayer {
            algorithm: default_algo,
            weight: 1.0,
        }]
    } else {
        config.layers.clone()
    };

    let generated: Vec<(Grid<f32>, f32)> = layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let grid = generate_layer(width, height, &layer.algorithm, world_seed, i as u64);
            (grid, layer.weight)
        })
        .collect();

    mix_layers(&generated)
}

/// Generate a single layer grid from a `FertilityAlgorithm` variant.
fn generate_layer(
    width: u16,
    height: u16,
    algorithm: &FertilityAlgorithm,
    world_seed: u64,
    layer_index: u64,
) -> Grid<f32> {
    match algorithm {
        FertilityAlgorithm::Uniform { value } => generate_uniform(width, height, *value),
        FertilityAlgorithm::Fbm {
            octaves,
            frequency,
            lacunarity,
            persistence,
            seed,
        } => {
            let effective_seed = seed.unwrap_or(world_seed.wrapping_add(layer_index));
            generate_fbm(
                width,
                height,
                *octaves,
                *frequency,
                *lacunarity,
                *persistence,
                effective_seed,
            )
        }
        FertilityAlgorithm::PoissonBlobs {
            blob_count,
            min_radius,
            max_radius,
            falloff,
            seed,
        } => {
            // If a seed is specified, create a dedicated rng from it for determinism.
            // Otherwise derive from world_seed + layer_index.
            let effective_seed = seed.unwrap_or(world_seed.wrapping_add(layer_index));
            let mut blob_rng = rand::rngs::SmallRng::seed_from_u64(effective_seed);
            generate_poisson_blobs(
                width,
                height,
                *blob_count,
                *min_radius,
                *max_radius,
                *falloff,
                &mut blob_rng,
            )
        }
    }
}

// Need SeedableRng for SmallRng::seed_from_u64
use rand::SeedableRng;

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_config() -> FertilityConfig {
        FertilityConfig {
            enabled: false,
            ..FertilityConfig::default()
        }
    }

    fn enabled_config() -> FertilityConfig {
        FertilityConfig {
            enabled: true,
            ..FertilityConfig::default()
        }
    }

    #[test]
    fn disabled_returns_all_zeros() {
        let grid = generate_fertility(32, 32, &disabled_config(), 1);
        for (_, _, v) in grid.iter() {
            assert!(v.abs() < f32::EPSILON, "expected 0.0, got {v}");
        }
    }

    #[test]
    fn enabled_with_default_config_produces_variation() {
        let grid = generate_fertility(64, 64, &enabled_config(), 1);
        let mut has_positive = false;
        let mut has_negative = false;
        for (_, _, v) in grid.iter() {
            if *v > 0.1 {
                has_positive = true;
            }
            if *v < -0.1 {
                has_negative = true;
            }
        }
        assert!(has_positive, "expected some positive values");
        assert!(has_negative, "expected some negative values");
    }

    #[test]
    fn empty_layers_uses_default() {
        let config = FertilityConfig {
            enabled: true,
            layers: vec![],
            ..FertilityConfig::default()
        };
        let grid = generate_fertility(32, 32, &config, 1);
        // Should produce a non-trivial grid (not all zeros) because it falls
        // back to PoissonBlobs.
        let any_nonzero = grid.iter().any(|(_, _, v)| v.abs() > 0.01);
        assert!(
            any_nonzero,
            "expected non-trivial output from default layer"
        );
    }
}

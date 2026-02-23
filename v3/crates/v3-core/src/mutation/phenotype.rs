use rand::Rng;

use crate::config::PhenotypeConfig;

/// Apply phenotype mutation to one channel, following v3-phenotype-spec.md Section 5.
///
/// Returns updated `(rgb, channel_weights, channel_polarity)`.
pub fn mutate_phenotype(
    rgb: [u8; 3],
    weights: [f32; 3],
    polarity: [bool; 3],
    config: &PhenotypeConfig,
    rng: &mut impl Rng,
) -> ([u8; 3], [f32; 3], [bool; 3]) {
    // Step 1: Sanitize weights — non-finite or negative → 0.0.
    let sanitized: [f32; 3] = [
        sanitize_weight(weights[0]),
        sanitize_weight(weights[1]),
        sanitize_weight(weights[2]),
    ];

    // Step 2: Weighted channel selection.
    let channel = select_channel(&sanitized, rng);

    // Step 3: Polarity flip.
    let mut child_polarity = polarity;
    if rng.gen::<f32>() < config.polarity_flip_chance {
        child_polarity[channel] = !child_polarity[channel];
    }

    // Step 4: Weight re-randomize for selected channel.
    let mut child_weights = weights;
    let lo = config.channel_weight_min;
    let hi = config.channel_weight_max;
    child_weights[channel] = rng.gen_range(lo..=hi);

    // Step 5: RGB step with wrapping u8 arithmetic.
    let mut child_rgb = rgb;
    let step = config.channel_step.max(1);
    if child_polarity[channel] {
        child_rgb[channel] = child_rgb[channel].wrapping_add(step);
    } else {
        child_rgb[channel] = child_rgb[channel].wrapping_sub(step);
    }

    (child_rgb, child_weights, child_polarity)
}

/// Sanitize a single weight: non-finite or negative → 0.0.
fn sanitize_weight(w: f32) -> f32 {
    if w.is_finite() && w >= 0.0 {
        w
    } else {
        0.0
    }
}

/// Select a channel (0=R, 1=G, 2=B) using weighted random sampling.
/// Falls back to uniform if all sanitized weights are effectively zero.
fn select_channel(sanitized: &[f32; 3], rng: &mut impl Rng) -> usize {
    let total: f32 = sanitized.iter().sum();
    if total <= f32::EPSILON {
        // All-zero fallback: uniform random channel selection.
        rng.gen_range(0..3)
    } else {
        let mut pick = rng.gen_range(0.0..total);
        for (i, &w) in sanitized.iter().enumerate() {
            pick -= w;
            if pick <= 0.0 {
                return i;
            }
        }
        // Floating-point rounding safety fallback.
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhenotypeConfig;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn default_config() -> PhenotypeConfig {
        PhenotypeConfig::default()
    }

    #[test]
    fn mutate_phenotype_applies_step_to_one_channel() {
        let rgb = [100u8, 100, 100];
        let weights = [1.0f32, 1.0, 1.0];
        let polarity = [true; 3];
        let mut rng = SmallRng::seed_from_u64(42);
        let (new_rgb, _, _) = mutate_phenotype(rgb, weights, polarity, &default_config(), &mut rng);
        // Exactly one channel should change (step=2, adding to 100 gives 102).
        let changed: Vec<usize> = (0..3).filter(|&i| new_rgb[i] != rgb[i]).collect();
        assert_eq!(
            changed.len(),
            1,
            "exactly one channel changes: got {:?}",
            new_rgb
        );
    }

    #[test]
    fn mutate_phenotype_wraps_u8_arithmetic() {
        let cfg = PhenotypeConfig {
            channel_step: 10,
            ..PhenotypeConfig::default()
        };
        // Force polarity=false (decrement) on all channels, start at 5 → 5 - 10 wraps.
        let rgb = [5u8, 5, 5];
        let weights = [1.0f32, 0.0, 0.0]; // force channel 0 selection
        let polarity = [false; 3];
        let mut rng = SmallRng::seed_from_u64(0);
        let (new_rgb, _, _) = mutate_phenotype(rgb, weights, polarity, &cfg, &mut rng);
        // Channel 0 should wrap: 5u8.wrapping_sub(10) = 251
        assert_eq!(new_rgb[0], 5u8.wrapping_sub(10), "expected wrapping sub");
        assert_eq!(new_rgb[1], 5, "channel 1 unchanged");
        assert_eq!(new_rgb[2], 5, "channel 2 unchanged");
    }

    #[test]
    fn mutate_phenotype_all_zero_weights_uses_uniform() {
        let cfg = default_config();
        let rgb = [100u8, 100, 100];
        let weights = [0.0f32, 0.0, 0.0];
        let polarity = [true; 3];
        // Run many times and check all channels can be selected.
        let mut seen = [false; 3];
        for seed in 0u64..200 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let (new_rgb, _, _) = mutate_phenotype(rgb, weights, polarity, &cfg, &mut rng);
            for i in 0..3 {
                if new_rgb[i] != rgb[i] {
                    seen[i] = true;
                }
            }
        }
        assert!(
            seen[0],
            "channel 0 must be selectable with all-zero weights"
        );
        assert!(
            seen[1],
            "channel 1 must be selectable with all-zero weights"
        );
        assert!(
            seen[2],
            "channel 2 must be selectable with all-zero weights"
        );
    }

    #[test]
    fn mutate_phenotype_negative_weight_treated_as_zero() {
        let cfg = default_config();
        let rgb = [100u8, 100, 100];
        let weights = [-1.0f32, 0.0, 0.0]; // all effectively zero after sanitization
        let polarity = [true; 3];
        // Should not panic; uniform fallback applies.
        let mut rng = SmallRng::seed_from_u64(7);
        let result = mutate_phenotype(rgb, weights, polarity, &cfg, &mut rng);
        // One channel changes (wrapping add step=2).
        let changed = (0..3).filter(|&i| result.0[i] != rgb[i]).count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn mutate_phenotype_polarity_flip_changes_direction() {
        // With polarity_flip_chance=1.0 and polarity=true, after flip polarity becomes false → sub.
        let cfg = PhenotypeConfig {
            polarity_flip_chance: 1.0,
            channel_step: 10,
            ..PhenotypeConfig::default()
        };
        let rgb = [100u8, 100, 100];
        let weights = [1.0f32, 0.0, 0.0]; // force channel 0
        let polarity = [true; 3]; // initial polarity true (add)
        let mut rng = SmallRng::seed_from_u64(1);
        let (new_rgb, _, new_pol) = mutate_phenotype(rgb, weights, polarity, &cfg, &mut rng);
        // After flip, polarity should be false → sub direction → 100 - 10 = 90
        assert!(!new_pol[0], "polarity should be flipped to false");
        assert_eq!(new_rgb[0], 90, "rgb should decrease with flipped polarity");
    }
}

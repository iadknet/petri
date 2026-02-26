use rand::Rng;

use crate::config::PhenotypeConfig;

/// Apply phenotype mutation to active channel, following v3-phenotype-spec.md Section 5.
///
/// Returns updated `(rgb, active_channel, channel_polarity)`.
///
/// # Panics
///
/// Debug-asserts that `active_channel < 3`.
pub fn mutate_phenotype(
    rgb: [u8; 3],
    active_channel: usize,
    polarity: [bool; 3],
    config: &PhenotypeConfig,
    rng: &mut impl Rng,
) -> ([u8; 3], usize, [bool; 3]) {
    debug_assert!(active_channel < 3, "active_channel must be 0, 1, or 2");

    // Step 1: Channel switch — with prob `channel_change_chance`, pick a different channel.
    let mut child_active_channel = active_channel;
    if rng.gen::<f32>() < config.channel_change_chance {
        // Pick one of the other 2 channels uniformly at random.
        let other_channels = if active_channel == 0 {
            [1, 2]
        } else if active_channel == 1 {
            [0, 2]
        } else {
            [0, 1]
        };
        child_active_channel = other_channels[rng.gen_range(0..2)];
    }

    // Step 2: Polarity flip.
    let mut child_polarity = polarity;
    if rng.gen::<f32>() < config.polarity_flip_chance {
        child_polarity[child_active_channel] = !child_polarity[child_active_channel];
    }

    // Step 3: RGB step with wrapping u8 arithmetic.
    let mut child_rgb = rgb;
    let step = config.channel_step.max(1);
    if child_polarity[child_active_channel] {
        child_rgb[child_active_channel] = child_rgb[child_active_channel].wrapping_add(step);
    } else {
        child_rgb[child_active_channel] = child_rgb[child_active_channel].wrapping_sub(step);
    }

    (child_rgb, child_active_channel, child_polarity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhenotypeConfig;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn phenotype_active_channel_unchanged_when_no_switch() {
        // With channel_change_chance=0.0, active channel should never change.
        let cfg = PhenotypeConfig {
            channel_change_chance: 0.0,
            ..PhenotypeConfig::default()
        };
        let rgb = [100u8, 100, 100];
        let active_channel = 1; // green
        let polarity = [true; 3];
        let mut rng = SmallRng::seed_from_u64(42);
        let (_, new_channel, _) = mutate_phenotype(rgb, active_channel, polarity, &cfg, &mut rng);
        assert_eq!(new_channel, active_channel, "channel should not change");
    }

    #[test]
    fn phenotype_active_channel_switches_to_different_channel() {
        // With channel_change_chance=1.0, active channel should always change to a different one.
        let cfg = PhenotypeConfig {
            channel_change_chance: 1.0,
            ..PhenotypeConfig::default()
        };
        let rgb = [100u8, 100, 100];
        let active_channel = 0; // red
        let polarity = [true; 3];
        let mut rng = SmallRng::seed_from_u64(42);
        let (_, new_channel, _) = mutate_phenotype(rgb, active_channel, polarity, &cfg, &mut rng);
        assert_ne!(
            new_channel, active_channel,
            "channel should change to a different one"
        );
        assert!(new_channel < 3, "channel must be in range [0, 2]");
    }

    #[test]
    fn phenotype_channel_step_applies_to_active_channel_only() {
        // Only the active channel's RGB should change.
        let cfg = PhenotypeConfig {
            channel_step: 5,
            channel_change_chance: 0.0, // don't switch
            polarity_flip_chance: 0.0,  // don't flip
        };
        let rgb = [100u8, 100, 100];
        let active_channel = 1; // green
        let polarity = [true, true, true]; // all add direction
        let mut rng = SmallRng::seed_from_u64(42);
        let (new_rgb, _, _) = mutate_phenotype(rgb, active_channel, polarity, &cfg, &mut rng);
        // Only channel 1 should change.
        assert_eq!(new_rgb[0], 100, "channel 0 unchanged");
        assert_eq!(new_rgb[1], 105, "channel 1 should add 5");
        assert_eq!(new_rgb[2], 100, "channel 2 unchanged");
    }

    #[test]
    fn phenotype_wraps_u8_arithmetic() {
        let cfg = PhenotypeConfig {
            channel_step: 10,
            channel_change_chance: 0.0,
            polarity_flip_chance: 0.0,
        };
        // Start at 5, subtract 10 → wraps to 251.
        let rgb = [5u8, 5, 5];
        let active_channel = 0;
        let polarity = [false, false, false]; // all subtract direction
        let mut rng = SmallRng::seed_from_u64(0);
        let (new_rgb, _, _) = mutate_phenotype(rgb, active_channel, polarity, &cfg, &mut rng);
        assert_eq!(new_rgb[0], 5u8.wrapping_sub(10), "expected wrapping sub");
        assert_eq!(new_rgb[1], 5, "channel 1 unchanged");
        assert_eq!(new_rgb[2], 5, "channel 2 unchanged");
    }

    #[test]
    fn phenotype_polarity_flip_changes_direction() {
        // With polarity_flip_chance=1.0, polarity should flip → direction changes.
        let cfg = PhenotypeConfig {
            channel_step: 10,
            channel_change_chance: 0.0,
            polarity_flip_chance: 1.0,
        };
        let rgb = [100u8, 100, 100];
        let active_channel = 0;
        let polarity = [true, true, true]; // all add initially
        let mut rng = SmallRng::seed_from_u64(1);
        let (new_rgb, _, new_pol) = mutate_phenotype(rgb, active_channel, polarity, &cfg, &mut rng);
        assert!(!new_pol[0], "polarity 0 should be flipped to false");
        assert_eq!(new_rgb[0], 90, "rgb 0 should subtract 10 after flip");
    }
}

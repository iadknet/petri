use rand::Rng;

use crate::config::PhenotypeConfig;
use crate::creature::phenotype::Phenotype;

/// Apply phenotype mutation to the active channel of `phenotype`'s six
/// internal HSL-mapped channels and return the child's phenotype.
///
/// # Panics
///
/// Debug-asserts that `active_channel < 6`.
#[must_use]
pub fn mutate_phenotype(
    phenotype: Phenotype,
    config: &PhenotypeConfig,
    rng: &mut impl Rng,
) -> Phenotype {
    let Phenotype {
        channels,
        active_channel,
        polarity,
    } = phenotype;
    debug_assert!(active_channel < 6, "active_channel must be in 0..6");

    // Step 1: Channel switch — with prob `channel_change_chance`, pick a different channel.
    let mut child_active_channel = active_channel;
    if rng.gen::<f32>() < config.channel_change_chance {
        // Build stack-allocated list of the other 5 channels.
        let mut others = [0usize; 5];
        let mut idx = 0;
        for ch in 0..6 {
            if ch != active_channel {
                others[idx] = ch;
                idx += 1;
            }
        }
        child_active_channel = others[rng.gen_range(0..5)];
    }

    // Step 2: Polarity flip.
    let mut child_polarity = polarity;
    if rng.gen::<f32>() < config.polarity_flip_chance {
        child_polarity[child_active_channel] = !child_polarity[child_active_channel];
    }

    // Step 3: Channel step with wrapping u8 arithmetic.
    let mut child_channels = channels;
    let step = config.channel_step.max(1);
    if child_polarity[child_active_channel] {
        child_channels[child_active_channel] =
            child_channels[child_active_channel].wrapping_add(step);
    } else {
        child_channels[child_active_channel] =
            child_channels[child_active_channel].wrapping_sub(step);
    }

    Phenotype {
        channels: child_channels,
        active_channel: child_active_channel,
        polarity: child_polarity,
    }
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
        let channels = [100u8; 6];
        let active_channel = 1;
        let polarity = [true; 6];
        let mut rng = SmallRng::seed_from_u64(42);
        let child = mutate_phenotype(
            Phenotype {
                channels,
                active_channel,
                polarity,
            },
            &cfg,
            &mut rng,
        );
        let new_channel = child.active_channel;
        assert_eq!(new_channel, active_channel, "channel should not change");
    }

    #[test]
    fn phenotype_active_channel_switches_to_different_channel() {
        // With channel_change_chance=1.0, active channel should always change to a different one.
        let cfg = PhenotypeConfig {
            channel_change_chance: 1.0,
            ..PhenotypeConfig::default()
        };
        let channels = [100u8; 6];
        let active_channel = 0;
        let polarity = [true; 6];
        let mut rng = SmallRng::seed_from_u64(42);
        let child = mutate_phenotype(
            Phenotype {
                channels,
                active_channel,
                polarity,
            },
            &cfg,
            &mut rng,
        );
        let new_channel = child.active_channel;
        assert_ne!(
            new_channel, active_channel,
            "channel should change to a different one"
        );
        assert!(new_channel < 6, "channel must be in range [0, 5]");
    }

    #[test]
    fn phenotype_channel_step_applies_to_active_channel_only() {
        // Only the active channel should change.
        let cfg = PhenotypeConfig {
            channel_step: 5,
            channel_change_chance: 0.0, // don't switch
            polarity_flip_chance: 0.0,  // don't flip
        };
        let channels = [100u8; 6];
        let active_channel = 1;
        let polarity = [true; 6]; // all add direction
        let mut rng = SmallRng::seed_from_u64(42);
        let child = mutate_phenotype(
            Phenotype {
                channels,
                active_channel,
                polarity,
            },
            &cfg,
            &mut rng,
        );
        let new_ch = child.channels;
        // Only channel 1 should change.
        assert_eq!(new_ch[0], 100, "channel 0 unchanged");
        assert_eq!(new_ch[1], 105, "channel 1 should add 5");
        for (i, &val) in new_ch.iter().enumerate().skip(2) {
            assert_eq!(val, 100, "channel {i} unchanged");
        }
    }

    #[test]
    fn phenotype_wraps_u8_arithmetic() {
        let cfg = PhenotypeConfig {
            channel_step: 10,
            channel_change_chance: 0.0,
            polarity_flip_chance: 0.0,
        };
        // Start at 5, subtract 10 → wraps to 251.
        let channels = [5u8; 6];
        let active_channel = 0;
        let polarity = [false; 6]; // all subtract direction
        let mut rng = SmallRng::seed_from_u64(0);
        let child = mutate_phenotype(
            Phenotype {
                channels,
                active_channel,
                polarity,
            },
            &cfg,
            &mut rng,
        );
        let new_ch = child.channels;
        assert_eq!(new_ch[0], 5u8.wrapping_sub(10), "expected wrapping sub");
        for (i, &val) in new_ch.iter().enumerate().skip(1) {
            assert_eq!(val, 5, "channel {i} unchanged");
        }
    }

    #[test]
    fn phenotype_polarity_flip_changes_direction() {
        // With polarity_flip_chance=1.0, polarity should flip → direction changes.
        let cfg = PhenotypeConfig {
            channel_step: 10,
            channel_change_chance: 0.0,
            polarity_flip_chance: 1.0,
        };
        let channels = [100u8; 6];
        let active_channel = 0;
        let polarity = [true; 6]; // all add initially
        let mut rng = SmallRng::seed_from_u64(1);
        let child = mutate_phenotype(
            Phenotype {
                channels,
                active_channel,
                polarity,
            },
            &cfg,
            &mut rng,
        );
        let new_ch = child.channels;
        let new_pol = child.polarity;
        assert!(!new_pol[0], "polarity 0 should be flipped to false");
        assert_eq!(new_ch[0], 90, "channel 0 should subtract 10 after flip");
    }
}

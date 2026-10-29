//! The colour phenotype record (T26.F01) and the deterministic application
//! of a sampled phenotype step (`mutation::phenotype`, T29.F01).

use crate::mutation::phenotype::PhenotypeDraw;

/// The six-channel internal colour phenotype (HSL-mapped; converted to RGB by
/// `creature::colour::channels_to_rgb` for the wire format) and the state
/// that steers its mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phenotype {
    /// The six channels: hue pair, saturation pair, lightness pair.
    pub channels: [u8; 6],
    /// The channel a mutation steps (0..6).
    pub active_channel: usize,
    /// Per-channel step direction: `true` adds, `false` subtracts.
    pub polarity: [bool; 6],
}

impl Phenotype {
    /// The founder phenotype: channels that render as RGB `[204, 61, 61]`,
    /// channel 0 active, every polarity adding.
    pub const FOUNDER: Self = Self {
        channels: [0, 0, 92, 92, 138, 138],
        active_channel: 0,
        polarity: [true; 6],
    };

    /// This phenotype after the sampled step `draw`: the active channel
    /// switched to the drawn other channel, its polarity flipped when drawn,
    /// then the active channel stepped by `draw.step` in its polarity's
    /// direction with wrapping `u8` arithmetic.
    ///
    /// # Panics
    ///
    /// Debug-asserts that `active_channel < 6`.
    #[must_use]
    pub fn mutated(self, draw: &PhenotypeDraw) -> Self {
        let Self {
            mut channels,
            active_channel,
            mut polarity,
        } = self;
        debug_assert!(active_channel < 6, "active_channel must be in 0..6");
        let active = draw.switch_to_other.map_or(active_channel, |k| {
            (0..6)
                .filter(|&channel| channel != active_channel)
                .nth(k)
                .expect("an index among the other five channels")
        });
        if draw.flip_polarity {
            polarity[active] = !polarity[active];
        }
        channels[active] = if polarity[active] {
            channels[active].wrapping_add(draw.step)
        } else {
            channels[active].wrapping_sub(draw.step)
        };
        Self {
            channels,
            active_channel: active,
            polarity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhenotypeConfig;
    use crate::mutation::phenotype::sample_phenotype_draw;
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
        let child = Phenotype {
            channels,
            active_channel,
            polarity,
        }
        .mutated(&sample_phenotype_draw(&cfg, &mut rng));
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
        let child = Phenotype {
            channels,
            active_channel,
            polarity,
        }
        .mutated(&sample_phenotype_draw(&cfg, &mut rng));
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
        let child = Phenotype {
            channels,
            active_channel,
            polarity,
        }
        .mutated(&sample_phenotype_draw(&cfg, &mut rng));
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
        let child = Phenotype {
            channels,
            active_channel,
            polarity,
        }
        .mutated(&sample_phenotype_draw(&cfg, &mut rng));
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
        let child = Phenotype {
            channels,
            active_channel,
            polarity,
        }
        .mutated(&sample_phenotype_draw(&cfg, &mut rng));
        let new_ch = child.channels;
        let new_pol = child.polarity;
        assert!(!new_pol[0], "polarity 0 should be flipped to false");
        assert_eq!(new_ch[0], 90, "channel 0 should subtract 10 after flip");
    }
}

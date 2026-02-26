use rand::Rng;

use crate::config::PhenotypeConfig;

// ─── HSL-to-RGB mapping constants ────────────────────────────────────────────

const MIN_SAT: f32 = 0.35;
const SAT_RANGE: f32 = 0.65;
const MIN_LIT: f32 = 0.25;
const LIT_RANGE: f32 = 0.50;

/// Convert HSL to RGB. `h` is hue in [0.0, 1.0), `s` is saturation in [0.0, 1.0],
/// `l` is lightness in [0.0, 1.0]. Returns `[R, G, B]` each in [0, 255].
#[inline]
#[must_use]
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return [v, v, v];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_channel(p, q, h + 1.0 / 3.0);
    let g = hue_to_channel(p, q, h);
    let b = hue_to_channel(p, q, h - 1.0 / 3.0);

    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    ]
}

#[inline]
fn hue_to_channel(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Map 6 internal phenotype channels to an RGB color via HSL.
///
/// Channels layout:
/// - `[0] + [1]` → hue (wrapping u8 add, mapped to [0.0, 1.0))
/// - `([2] + [3]) / 2` → saturation (linearly mapped to [0.35, 1.0])
/// - `([4] + [5]) / 2` → lightness (linearly mapped to [0.25, 0.75])
#[inline]
#[must_use]
pub fn channels_to_rgb(channels: [u8; 6]) -> [u8; 3] {
    let h_raw = channels[0].wrapping_add(channels[1]);
    let h = h_raw as f32 / 255.0;

    let s_avg = (channels[2] as f32 + channels[3] as f32) / 2.0;
    let s = MIN_SAT + (s_avg / 255.0) * SAT_RANGE;

    let l_avg = (channels[4] as f32 + channels[5] as f32) / 2.0;
    let l = MIN_LIT + (l_avg / 255.0) * LIT_RANGE;

    hsl_to_rgb(h, s, l)
}

/// Apply phenotype mutation to active channel.
///
/// Operates on 6 internal HSL-mapped channels. Returns updated
/// `(channels, active_channel, channel_polarity)`.
///
/// # Panics
///
/// Debug-asserts that `active_channel < 6`.
pub fn mutate_phenotype(
    channels: [u8; 6],
    active_channel: usize,
    polarity: [bool; 6],
    config: &PhenotypeConfig,
    rng: &mut impl Rng,
) -> ([u8; 6], usize, [bool; 6]) {
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

    (child_channels, child_active_channel, child_polarity)
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
        let (_, new_channel, _) =
            mutate_phenotype(channels, active_channel, polarity, &cfg, &mut rng);
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
        let (_, new_channel, _) =
            mutate_phenotype(channels, active_channel, polarity, &cfg, &mut rng);
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
        let (new_ch, _, _) = mutate_phenotype(channels, active_channel, polarity, &cfg, &mut rng);
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
        let (new_ch, _, _) = mutate_phenotype(channels, active_channel, polarity, &cfg, &mut rng);
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
        let (new_ch, _, new_pol) =
            mutate_phenotype(channels, active_channel, polarity, &cfg, &mut rng);
        assert!(!new_pol[0], "polarity 0 should be flipped to false");
        assert_eq!(new_ch[0], 90, "channel 0 should subtract 10 after flip");
    }

    // ── HSL conversion tests ─────────────────────────────────────────────────

    #[test]
    fn hsl_to_rgb_known_red() {
        // HSL(0.0, 0.584, 0.520) should produce a red-dominant color.
        let rgb = hsl_to_rgb(0.0, 0.584, 0.520);
        assert_eq!(rgb, [204, 61, 61]);
    }

    #[test]
    fn hsl_to_rgb_achromatic() {
        // Saturation=0 → grayscale regardless of hue.
        let rgb = hsl_to_rgb(0.5, 0.0, 0.5);
        assert_eq!(rgb[0], rgb[1]);
        assert_eq!(rgb[1], rgb[2]);
        assert_eq!(rgb[0], 128);
    }

    #[test]
    fn channels_to_rgb_founder_values() {
        // Founder channels [0, 0, 92, 92, 138, 138] should map to founder RGB [204, 61, 61].
        let rgb = channels_to_rgb([0, 0, 92, 92, 138, 138]);
        assert_eq!(rgb, [204, 61, 61]);
    }

    #[test]
    fn channels_to_rgb_saturation_floor() {
        // Channels [0,0] for sat → minimum saturation of 0.35.
        let rgb = channels_to_rgb([0, 0, 0, 0, 128, 128]);
        // With s=0.35, l≈0.5, hue=0 → should still produce color, not grayscale.
        assert_ne!(rgb[0], rgb[1], "saturation floor should prevent grayscale");
    }

    #[test]
    fn channels_to_rgb_lightness_ceiling() {
        // Channels [255,255] for lit → maximum lightness of 0.75.
        let rgb = channels_to_rgb([0, 0, 128, 128, 255, 255]);
        // L=0.75 means the max component should be well below 255 (not washed out to white).
        assert!(
            rgb[0] < 255 || rgb[1] < 255 || rgb[2] < 255,
            "lightness ceiling should prevent pure white"
        );
    }
}

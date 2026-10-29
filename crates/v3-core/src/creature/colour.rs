//! A creature's seen colour (T27.F01): the mapping from the six heritable
//! phenotype channels to the RGB a viewer sees, shared by perception, the
//! own-colour inputs and the server's display.

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

/// The seen colour on the unit scale: `channels_to_rgb(channels)[c] / 255`,
/// each in `[0, 1]`. What a neighbour's slot reads as `RED`, `GREEN`, `BLUE`.
#[inline]
#[must_use]
pub fn unit_rgb(channels: [u8; 6]) -> [f32; 3] {
    channels_to_rgb(channels).map(|c| f32::from(c) / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let rgb = channels_to_rgb(crate::genome::body::Phenotype::FOUNDER.channels());
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

    /// CSS Color 4 `hslToRgb`, in f64: an independent formulation of the same
    /// HSL mapping (no piecewise hue ramp), returning channels on [0, 255].
    fn reference_hsl_to_rgb(h: f64, s: f64, l: f64) -> [f64; 3] {
        let a = s * l.min(1.0 - l);
        let f = |n: f64| {
            let k = (n + h * 12.0) % 12.0;
            (l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)) * 255.0
        };
        [f(0.0), f(8.0), f(4.0)]
    }

    #[test]
    fn channels_to_rgb_matches_reference_across_every_hue() {
        // Every hue byte, at the founder's saturation and lightness and at the
        // full-saturation, mid-lightness and the floor/ceiling extremes: each
        // hue sector (the red channel's wrap above 2/3 and the blue channel's
        // shift below 1/3 included) agrees with the reference.
        for (sat, lit) in [(92u8, 138u8), (255, 127), (0, 0), (255, 255)] {
            let s = f64::from(MIN_SAT) + f64::from(sat) / 255.0 * f64::from(SAT_RANGE);
            let l = f64::from(MIN_LIT) + f64::from(lit) / 255.0 * f64::from(LIT_RANGE);
            for hue in 0..=255u8 {
                let rgb = channels_to_rgb([hue, 0, sat, sat, lit, lit]);
                let expected = reference_hsl_to_rgb(f64::from(hue) / 255.0, s, l);
                for c in 0..3 {
                    assert!(
                        (f64::from(rgb[c]) - expected[c]).abs() <= 0.5 + 1e-3,
                        "hue {hue} sat {sat} lit {lit} channel {c}: {rgb:?} vs {expected:?}"
                    );
                }
            }
        }
    }

    proptest::proptest! {
        /// Every channel array reads as three unit values equal to its RGB
        /// over 255.
        #[test]
        fn unit_rgb_is_rgb_over_255(channels in proptest::array::uniform6(0u8..)) {
            let rgb = channels_to_rgb(channels);
            let unit = unit_rgb(channels);
            for c in 0..3 {
                proptest::prop_assert!((0.0..=1.0).contains(&unit[c]));
                proptest::prop_assert_eq!(unit[c], f32::from(rgb[c]) / 255.0);
            }
        }
    }
}

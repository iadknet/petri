use rand::Rng;

pub(super) const PHENOTYPE_HUE_DRIFT: f32 = 2.0;
pub(super) const PHENOTYPE_SATURATION_DRIFT: f32 = 0.05;
pub(super) const PHENOTYPE_SATURATION_MIN: f32 = 0.4;
pub(super) const PHENOTYPE_SATURATION_MAX: f32 = 1.0;
pub(super) const PHENOTYPE_VALUE: f32 = 0.80;

/// Standard HSV to RGB conversion.
///
/// Hue wraps via `rem_euclid(360.0)`, non-finite hue is treated as 0.0,
/// saturation and value are clamped to \[0, 1\].
pub(super) fn hsv_to_rgb(hue_degrees: f32, saturation: f32, value: f32) -> [u8; 3] {
    let hue = if hue_degrees.is_finite() {
        hue_degrees.rem_euclid(360.0)
    } else {
        0.0
    };
    let saturation = saturation.clamp(0.0, 1.0);
    let value = value.clamp(0.0, 1.0);

    let chroma = value * saturation;
    let hue_section = hue / 60.0;
    let x = chroma * (1.0 - ((hue_section.rem_euclid(2.0)) - 1.0).abs());
    let (r1, g1, b1) = if hue_section < 1.0 {
        (chroma, x, 0.0)
    } else if hue_section < 2.0 {
        (x, chroma, 0.0)
    } else if hue_section < 3.0 {
        (0.0, chroma, x)
    } else if hue_section < 4.0 {
        (0.0, x, chroma)
    } else if hue_section < 5.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };
    let m = value - chroma;

    [to_rgb_u8(r1 + m), to_rgb_u8(g1 + m), to_rgb_u8(b1 + m)]
}

/// Convert HSV to RGB using the fixed `PHENOTYPE_VALUE` brightness.
pub(super) fn phenotype_rgb(hue: f32, saturation: f32) -> [u8; 3] {
    hsv_to_rgb(hue, saturation, PHENOTYPE_VALUE)
}

/// Generate a random founder hue in \[0.0, 360.0).
pub(super) fn random_founder_hue<R: Rng>(rng: &mut R) -> f32 {
    rng.gen_range(0.0..360.0)
}

/// Generate a random founder saturation in \[PHENOTYPE_SATURATION_MIN, PHENOTYPE_SATURATION_MAX\].
pub(super) fn random_founder_saturation<R: Rng>(rng: &mut R) -> f32 {
    rng.gen_range(PHENOTYPE_SATURATION_MIN..=PHENOTYPE_SATURATION_MAX)
}

/// Inherit hue from parent. If `mutated` is false, return `parent_hue` exactly.
/// If mutated, nudge by +/- `PHENOTYPE_HUE_DRIFT` and wrap via `rem_euclid(360.0)`.
pub(super) fn inherit_hue<R: Rng>(parent_hue: f32, mutated: bool, rng: &mut R) -> f32 {
    if !mutated {
        return parent_hue;
    }
    let drift = rng.gen_range(-PHENOTYPE_HUE_DRIFT..=PHENOTYPE_HUE_DRIFT);
    (parent_hue + drift).rem_euclid(360.0)
}

/// Inherit saturation from parent, nudging by +/- `PHENOTYPE_SATURATION_DRIFT`
/// and clamping to \[PHENOTYPE_SATURATION_MIN, PHENOTYPE_SATURATION_MAX\].
pub(super) fn inherit_saturation<R: Rng>(parent_saturation: f32, rng: &mut R) -> f32 {
    let drift = rng.gen_range(-PHENOTYPE_SATURATION_DRIFT..=PHENOTYPE_SATURATION_DRIFT);
    (parent_saturation + drift).clamp(PHENOTYPE_SATURATION_MIN, PHENOTYPE_SATURATION_MAX)
}

fn to_rgb_u8(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn hsv_to_rgb_pure_red() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [255, 0, 0]);
    }

    #[test]
    fn hsv_to_rgb_pure_green() {
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), [0, 255, 0]);
    }

    #[test]
    fn hsv_to_rgb_pure_blue() {
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), [0, 0, 255]);
    }

    #[test]
    fn inherit_hue_no_mutation_returns_exact_parent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let parent_hue = 123.456;
        let child_hue = inherit_hue(parent_hue, false, &mut rng);
        assert_eq!(child_hue, parent_hue);
    }

    #[test]
    fn inherit_hue_with_mutation_drifts_within_range() {
        let mut rng = SmallRng::seed_from_u64(42);
        let parent_hue = 180.0;
        for _ in 0..100 {
            let child_hue = inherit_hue(parent_hue, true, &mut rng);
            let diff = (child_hue - parent_hue).abs();
            // Account for wraparound: shortest angular distance
            let angular_diff = diff.min(360.0 - diff);
            assert!(
                angular_diff <= PHENOTYPE_HUE_DRIFT,
                "angular diff {angular_diff} exceeded drift {PHENOTYPE_HUE_DRIFT}"
            );
        }
    }

    #[test]
    fn inherit_hue_wraps_around_360() {
        let mut rng = SmallRng::seed_from_u64(42);
        // Test near upper boundary
        for _ in 0..100 {
            let child_hue = inherit_hue(359.5, true, &mut rng);
            assert!(
                (0.0..360.0).contains(&child_hue),
                "child_hue {child_hue} not in [0, 360)"
            );
        }
        // Test near lower boundary
        for _ in 0..100 {
            let child_hue = inherit_hue(0.5, true, &mut rng);
            assert!(
                (0.0..360.0).contains(&child_hue),
                "child_hue {child_hue} not in [0, 360)"
            );
        }
    }

    #[test]
    fn inherit_saturation_clamps_to_bounds() {
        let mut rng = SmallRng::seed_from_u64(42);
        // Test at minimum boundary
        for _ in 0..100 {
            let child_sat = inherit_saturation(PHENOTYPE_SATURATION_MIN, &mut rng);
            assert!(
                (PHENOTYPE_SATURATION_MIN..=PHENOTYPE_SATURATION_MAX).contains(&child_sat),
                "child_sat {child_sat} out of bounds [{PHENOTYPE_SATURATION_MIN}, {PHENOTYPE_SATURATION_MAX}]"
            );
        }
        // Test at maximum boundary
        for _ in 0..100 {
            let child_sat = inherit_saturation(PHENOTYPE_SATURATION_MAX, &mut rng);
            assert!(
                (PHENOTYPE_SATURATION_MIN..=PHENOTYPE_SATURATION_MAX).contains(&child_sat),
                "child_sat {child_sat} out of bounds [{PHENOTYPE_SATURATION_MIN}, {PHENOTYPE_SATURATION_MAX}]"
            );
        }
    }

    #[test]
    fn founder_hue_in_range() {
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..100 {
            let hue = random_founder_hue(&mut rng);
            assert!(
                (0.0..360.0).contains(&hue),
                "founder hue {hue} not in [0, 360)"
            );
        }
    }

    #[test]
    fn founder_saturation_in_range() {
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..100 {
            let sat = random_founder_saturation(&mut rng);
            assert!(
                (PHENOTYPE_SATURATION_MIN..=PHENOTYPE_SATURATION_MAX).contains(&sat),
                "founder saturation {sat} not in [{PHENOTYPE_SATURATION_MIN}, {PHENOTYPE_SATURATION_MAX}]"
            );
        }
    }
}

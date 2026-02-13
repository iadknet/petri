use rand::Rng;

pub(super) const FOUNDER_PHENOTYPE_RGB: [u8; 3] = [204, 61, 61];
pub(super) const FOUNDER_PHENOTYPE_POSITIVE_INCREMENT: bool = true;
pub(super) const PHENOTYPE_RGB_CHANNEL_STEP: u8 = 2;
pub(super) const PHENOTYPE_POLARITY_FLIP_CHANCE: f32 = 0.002;

pub(super) fn founder_rgb() -> [u8; 3] {
    FOUNDER_PHENOTYPE_RGB
}

pub(super) fn founder_positive_increment() -> bool {
    FOUNDER_PHENOTYPE_POSITIVE_INCREMENT
}

/// Inherit RGB phenotype from the parent.
///
/// If `mutated` is false, both RGB channels and polarity are preserved exactly.
/// If `mutated` is true:
/// - polarity may flip with a rare probability
/// - one random RGB channel may change by `PHENOTYPE_RGB_CHANNEL_STEP`
///   using the (possibly flipped) polarity as sign.
pub(super) fn inherit_rgb<R: Rng>(
    parent_rgb: [u8; 3],
    parent_positive_increment: bool,
    mutated: bool,
    rng: &mut R,
) -> ([u8; 3], bool) {
    if !mutated {
        return (parent_rgb, parent_positive_increment);
    }

    let mut child_positive_increment = parent_positive_increment;
    if rng.gen::<f32>() < PHENOTYPE_POLARITY_FLIP_CHANCE {
        child_positive_increment = !child_positive_increment;
    }

    let mut child_rgb = parent_rgb;
    let channel = rng.gen_range(0..child_rgb.len());
    let signed_delta = if child_positive_increment {
        i16::from(PHENOTYPE_RGB_CHANNEL_STEP)
    } else {
        -i16::from(PHENOTYPE_RGB_CHANNEL_STEP)
    };
    let next_value = i16::from(child_rgb[channel]) + signed_delta;
    child_rgb[channel] = next_value.clamp(0, 255) as u8;

    (child_rgb, child_positive_increment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn founder_phenotype_defaults_are_stable() {
        assert_eq!(founder_rgb(), FOUNDER_PHENOTYPE_RGB);
        assert_eq!(
            founder_positive_increment(),
            FOUNDER_PHENOTYPE_POSITIVE_INCREMENT
        );
    }

    #[test]
    fn inherit_rgb_without_mutation_keeps_channels_and_polarity() {
        let mut rng = SmallRng::seed_from_u64(99);
        let parent_rgb = [120, 130, 140];
        let parent_positive = true;

        let (child_rgb, child_positive) = inherit_rgb(parent_rgb, parent_positive, false, &mut rng);
        assert_eq!(child_rgb, parent_rgb);
        assert_eq!(child_positive, parent_positive);
    }

    #[test]
    fn inherit_rgb_mutation_changes_exactly_one_channel_by_step() {
        let mut rng = SmallRng::seed_from_u64(7);
        let parent_rgb = [128, 128, 128];

        for _ in 0..500 {
            let (child_rgb, _) = inherit_rgb(parent_rgb, true, true, &mut rng);
            let changed_channels = (0..3)
                .filter(|channel| child_rgb[*channel] != parent_rgb[*channel])
                .count();
            assert_eq!(changed_channels, 1, "changed {changed_channels} channels");
            let delta = child_rgb
                .iter()
                .zip(parent_rgb.iter())
                .map(|(child, parent)| i16::from(*child) - i16::from(*parent))
                .find(|delta| *delta != 0)
                .expect("expected non-zero channel delta");
            assert_eq!(delta.abs(), i16::from(PHENOTYPE_RGB_CHANNEL_STEP));
        }
    }

    #[test]
    fn inherit_rgb_can_flip_polarity() {
        let mut rng = SmallRng::seed_from_u64(11);
        let parent_rgb = [50, 70, 90];
        let mut polarity = true;
        let mut flipped = false;

        for _ in 0..50_000 {
            let (_, next_polarity) = inherit_rgb(parent_rgb, polarity, true, &mut rng);
            if next_polarity != polarity {
                flipped = true;
                break;
            }
            polarity = next_polarity;
        }

        assert!(
            flipped,
            "expected deterministic run to include at least one polarity flip"
        );
    }

    #[test]
    fn inherit_rgb_negative_polarity_can_reduce_channel_value() {
        let mut rng = SmallRng::seed_from_u64(17);
        let parent_rgb = [100, 100, 100];
        let mut observed_reduction = false;

        for _ in 0..1_000 {
            let (child_rgb, _) = inherit_rgb(parent_rgb, false, true, &mut rng);
            if child_rgb != parent_rgb {
                observed_reduction = child_rgb.iter().any(|channel| *channel < 100);
                if observed_reduction {
                    break;
                }
            }
        }

        assert!(
            observed_reduction,
            "expected at least one negative channel mutation"
        );
    }
}

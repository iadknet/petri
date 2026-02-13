use rand::Rng;

pub(super) const FOUNDER_PHENOTYPE_RGB: [u8; 3] = [204, 61, 61];
pub(super) const FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS: [f32; 3] = [1.0, 1.0, 1.0];
pub(super) const FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT: [bool; 3] = [true, true, true];
pub(super) const PHENOTYPE_RGB_CHANNEL_STEP: u8 = 2;
pub(super) const PHENOTYPE_POLARITY_FLIP_CHANCE: f32 = 0.002;
pub(super) const PHENOTYPE_CHANNEL_WEIGHT_MIN: f32 = 0.05;
pub(super) const PHENOTYPE_CHANNEL_WEIGHT_MAX: f32 = 1.0;

pub(super) fn founder_rgb() -> [u8; 3] {
    FOUNDER_PHENOTYPE_RGB
}

pub(super) fn founder_channel_weights() -> [f32; 3] {
    FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS
}

pub(super) fn founder_channel_positive_increment() -> [bool; 3] {
    FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT
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
    parent_channel_weights: [f32; 3],
    parent_channel_positive_increment: [bool; 3],
    mutated: bool,
    rng: &mut R,
) -> ([u8; 3], [f32; 3], [bool; 3]) {
    if !mutated {
        return (
            parent_rgb,
            parent_channel_weights,
            parent_channel_positive_increment,
        );
    }

    let channel = select_weighted_channel(parent_channel_weights, rng);
    let mut child_positive_increment = parent_channel_positive_increment;
    let mut child_weights = parent_channel_weights;
    if rng.gen::<f32>() < PHENOTYPE_POLARITY_FLIP_CHANCE {
        child_positive_increment[channel] = !child_positive_increment[channel];
    }

    let mut child_rgb = parent_rgb;
    child_weights[channel] = random_channel_weight(rng);
    let step = PHENOTYPE_RGB_CHANNEL_STEP.max(1);
    if child_positive_increment[channel] {
        child_rgb[channel] = child_rgb[channel].wrapping_add(step);
    } else {
        child_rgb[channel] = child_rgb[channel].wrapping_sub(step);
    }

    (child_rgb, child_weights, child_positive_increment)
}

fn select_weighted_channel<R: Rng>(weights: [f32; 3], rng: &mut R) -> usize {
    let sanitized = sanitize_weights(weights);
    let total = sanitized.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return rng.gen_range(0..sanitized.len());
    }
    let target = rng.gen_range(0.0..total);
    let mut cumulative = 0.0;
    for (idx, weight) in sanitized.iter().enumerate() {
        cumulative += *weight;
        if target < cumulative {
            return idx;
        }
    }
    sanitized.len() - 1
}

fn sanitize_weights(weights: [f32; 3]) -> [f32; 3] {
    let mut sanitized = [0.0; 3];
    for (idx, weight) in weights.iter().enumerate() {
        sanitized[idx] = if weight.is_finite() {
            weight.max(0.0)
        } else {
            0.0
        };
    }
    sanitized
}

fn random_channel_weight<R: Rng>(rng: &mut R) -> f32 {
    rng.gen_range(PHENOTYPE_CHANNEL_WEIGHT_MIN..=PHENOTYPE_CHANNEL_WEIGHT_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn founder_phenotype_defaults_are_stable() {
        assert_eq!(founder_rgb(), FOUNDER_PHENOTYPE_RGB);
        assert_eq!(founder_channel_weights(), FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS);
        assert_eq!(
            founder_channel_positive_increment(),
            FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT
        );
    }

    #[test]
    fn inherit_rgb_without_mutation_keeps_channels_weights_and_polarity() {
        let mut rng = SmallRng::seed_from_u64(99);
        let parent_rgb = [120, 130, 140];
        let parent_weights = [0.2, 0.3, 0.5];
        let parent_positive = [true, false, true];

        let (child_rgb, child_weights, child_positive) =
            inherit_rgb(parent_rgb, parent_weights, parent_positive, false, &mut rng);
        assert_eq!(child_rgb, parent_rgb);
        assert_eq!(child_weights, parent_weights);
        assert_eq!(child_positive, parent_positive);
    }

    #[test]
    fn inherit_rgb_mutation_changes_exactly_one_channel_by_step() {
        let mut rng = SmallRng::seed_from_u64(7);
        let parent_rgb = [128, 128, 128];
        let mut parent_weights = [1.0 / 3.0; 3];
        let mut parent_positive = [true; 3];

        for _ in 0..500 {
            let (child_rgb, child_weights, child_positive) =
                inherit_rgb(parent_rgb, parent_weights, parent_positive, true, &mut rng);
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
            parent_weights = child_weights;
            parent_positive = child_positive;
        }
    }

    #[test]
    fn inherit_rgb_uses_channel_weights_to_select_channel() {
        let mut rng = SmallRng::seed_from_u64(1234);
        let parent_rgb = [10, 20, 30];
        let parent_weights = [1.0, 0.0, 0.0];
        let parent_positive = [true, true, true];

        let (child_rgb, child_weights, child_positive) =
            inherit_rgb(parent_rgb, parent_weights, parent_positive, true, &mut rng);

        assert_ne!(child_rgb[0], parent_rgb[0], "red channel should mutate");
        assert_eq!(
            child_rgb[1], parent_rgb[1],
            "green channel should not mutate"
        );
        assert_eq!(
            child_rgb[2], parent_rgb[2],
            "blue channel should not mutate"
        );
        assert_eq!(
            child_weights[1], parent_weights[1],
            "unselected channel weight should remain unchanged"
        );
        assert_eq!(
            child_weights[2], parent_weights[2],
            "unselected channel weight should remain unchanged"
        );
        assert_eq!(
            child_positive[1], parent_positive[1],
            "unselected channel polarity should remain unchanged"
        );
        assert_eq!(
            child_positive[2], parent_positive[2],
            "unselected channel polarity should remain unchanged"
        );
    }

    #[test]
    fn inherit_rgb_re_randomizes_selected_channel_weight() {
        let mut rng = SmallRng::seed_from_u64(7_777);
        let parent_rgb = [100, 120, 140];
        let parent_weights = [1.0, 0.0, 0.0];
        let parent_positive = [true, true, true];

        let (_, child_weights, _) =
            inherit_rgb(parent_rgb, parent_weights, parent_positive, true, &mut rng);

        assert!(
            (PHENOTYPE_CHANNEL_WEIGHT_MIN..=PHENOTYPE_CHANNEL_WEIGHT_MAX)
                .contains(&child_weights[0]),
            "selected channel weight should be randomized to configured range"
        );
        assert_eq!(
            child_weights[1], parent_weights[1],
            "unselected channel weight should remain unchanged"
        );
        assert_eq!(
            child_weights[2], parent_weights[2],
            "unselected channel weight should remain unchanged"
        );
    }

    #[test]
    fn inherit_rgb_can_flip_only_selected_channel_polarity() {
        let mut rng = SmallRng::seed_from_u64(11);
        let parent_rgb = [50, 70, 90];
        let parent_weights = [1.0, 0.0, 0.0];
        let parent_positive = [true, false, true];
        let mut flipped = false;

        for _ in 0..200_000 {
            let (_, _, next_positive) =
                inherit_rgb(parent_rgb, parent_weights, parent_positive, true, &mut rng);
            assert_eq!(next_positive[1], parent_positive[1]);
            assert_eq!(next_positive[2], parent_positive[2]);
            if next_positive[0] != parent_positive[0] {
                flipped = true;
                break;
            }
        }

        assert!(
            flipped,
            "expected selected channel polarity to eventually flip"
        );
    }

    #[test]
    fn inherit_rgb_negative_polarity_can_reduce_selected_channel_value() {
        let mut rng = SmallRng::seed_from_u64(17);
        let parent_rgb = [100, 100, 100];
        let parent_weights = [1.0, 0.0, 0.0];
        let parent_positive = [false, true, true];
        let mut observed_reduction = false;

        for _ in 0..1_000 {
            let (child_rgb, _, _) =
                inherit_rgb(parent_rgb, parent_weights, parent_positive, true, &mut rng);
            if child_rgb != parent_rgb {
                observed_reduction = child_rgb[0] < 100;
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

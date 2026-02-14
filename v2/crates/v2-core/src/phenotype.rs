pub const FOUNDER_PHENOTYPE_RGB: [u8; 3] = [204, 61, 61];
pub const FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS: [f32; 3] = [1.0, 1.0, 1.0];
pub const FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT: [bool; 3] = [true, true, true];
pub const PHENOTYPE_RGB_CHANNEL_STEP: u8 = 2;
pub const PHENOTYPE_POLARITY_FLIP_CHANCE: f32 = 0.002;
pub const PHENOTYPE_CHANNEL_WEIGHT_MIN: f32 = 0.05;
pub const PHENOTYPE_CHANNEL_WEIGHT_MAX: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phenotype {
    pub rgb: [u8; 3],
    pub channel_weights: [f32; 3],
    pub channel_positive_increment: [bool; 3],
}

impl Phenotype {
    #[must_use]
    pub const fn founder() -> Self {
        Self {
            rgb: FOUNDER_PHENOTYPE_RGB,
            channel_weights: FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS,
            channel_positive_increment: FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT,
        }
    }
}

#[must_use]
pub fn inherit_phenotype(parent: Phenotype, mutated: bool, seed: u64) -> Phenotype {
    if !mutated {
        return parent;
    }

    let mut rng = Lcg64::new(seed);
    let channel = select_weighted_channel(parent.channel_weights, &mut rng);

    let mut child = parent;
    if rng.next_f32() < PHENOTYPE_POLARITY_FLIP_CHANCE {
        child.channel_positive_increment[channel] = !child.channel_positive_increment[channel];
    }

    child.channel_weights[channel] = random_channel_weight(&mut rng);
    let step = PHENOTYPE_RGB_CHANNEL_STEP.max(1);
    if child.channel_positive_increment[channel] {
        child.rgb[channel] = child.rgb[channel].wrapping_add(step);
    } else {
        child.rgb[channel] = child.rgb[channel].wrapping_sub(step);
    }

    child
}

fn select_weighted_channel(weights: [f32; 3], rng: &mut Lcg64) -> usize {
    let sanitized = sanitize_weights(weights);
    let total = sanitized.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return rng.next_usize(sanitized.len());
    }

    let target = rng.next_f32() * total;
    let mut cumulative = 0.0_f32;
    for (index, weight) in sanitized.iter().enumerate() {
        cumulative += *weight;
        if target < cumulative {
            return index;
        }
    }
    sanitized.len() - 1
}

fn sanitize_weights(weights: [f32; 3]) -> [f32; 3] {
    let mut sanitized = [0.0_f32; 3];
    for (index, weight) in weights.iter().enumerate() {
        sanitized[index] = if weight.is_finite() {
            weight.max(0.0)
        } else {
            0.0
        };
    }
    sanitized
}

fn random_channel_weight(rng: &mut Lcg64) -> f32 {
    let span = PHENOTYPE_CHANNEL_WEIGHT_MAX - PHENOTYPE_CHANNEL_WEIGHT_MIN;
    PHENOTYPE_CHANNEL_WEIGHT_MIN + span * rng.next_f32()
}

#[derive(Clone, Debug)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_u32(&mut self) -> u32 {
        let bytes = self.next_u64().to_le_bytes();
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn next_usize(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u32() as usize) % upper_exclusive
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT, FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS,
        FOUNDER_PHENOTYPE_RGB, PHENOTYPE_CHANNEL_WEIGHT_MAX, PHENOTYPE_CHANNEL_WEIGHT_MIN,
        PHENOTYPE_RGB_CHANNEL_STEP, Phenotype, inherit_phenotype,
    };

    #[test]
    fn founder_phenotype_defaults_are_stable() {
        let founder = Phenotype::founder();
        assert_eq!(founder.rgb, FOUNDER_PHENOTYPE_RGB);
        assert_eq!(founder.channel_weights, FOUNDER_PHENOTYPE_CHANNEL_WEIGHTS);
        assert_eq!(
            founder.channel_positive_increment,
            FOUNDER_PHENOTYPE_CHANNEL_POSITIVE_INCREMENT
        );
    }

    #[test]
    fn inherit_without_mutation_preserves_parent() {
        let parent = Phenotype {
            rgb: [120, 130, 140],
            channel_weights: [0.2, 0.3, 0.5],
            channel_positive_increment: [true, false, true],
        };
        assert_eq!(inherit_phenotype(parent, false, 12), parent);
    }

    #[test]
    fn inherit_mutation_changes_exactly_one_channel_by_step() {
        let parent = Phenotype {
            rgb: [128, 128, 128],
            channel_weights: [0.4, 0.3, 0.3],
            channel_positive_increment: [true, true, true],
        };

        for seed in 1_u64..=128 {
            let child = inherit_phenotype(parent, true, seed);
            let changed_channels = child
                .rgb
                .iter()
                .zip(parent.rgb.iter())
                .filter(|(child_channel, parent_channel)| child_channel != parent_channel)
                .count();
            assert_eq!(changed_channels, 1);

            let delta = child
                .rgb
                .iter()
                .zip(parent.rgb.iter())
                .map(|(child_channel, parent_channel)| {
                    i16::from(*child_channel) - i16::from(*parent_channel)
                })
                .find(|difference| *difference != 0)
                .expect("one channel should change");
            assert_eq!(delta.abs(), i16::from(PHENOTYPE_RGB_CHANNEL_STEP));
        }
    }

    #[test]
    fn inherit_respects_weighted_channel_selection() {
        let parent = Phenotype {
            rgb: [10, 20, 30],
            channel_weights: [1.0, 0.0, 0.0],
            channel_positive_increment: [true, true, true],
        };
        let child = inherit_phenotype(parent, true, 99);
        assert_ne!(child.rgb[0], parent.rgb[0]);
        assert_eq!(child.rgb[1], parent.rgb[1]);
        assert_eq!(child.rgb[2], parent.rgb[2]);
    }

    #[test]
    fn inherit_randomizes_only_selected_channel_weight() {
        let parent = Phenotype {
            rgb: [100, 110, 120],
            channel_weights: [1.0, 0.0, 0.0],
            channel_positive_increment: [true, true, true],
        };
        let child = inherit_phenotype(parent, true, 123);
        assert!(
            (PHENOTYPE_CHANNEL_WEIGHT_MIN..=PHENOTYPE_CHANNEL_WEIGHT_MAX)
                .contains(&child.channel_weights[0])
        );
        assert_eq!(child.channel_weights[1], parent.channel_weights[1]);
        assert_eq!(child.channel_weights[2], parent.channel_weights[2]);
    }
}

//! The phenotype step's draws (T29.F01 invariant 8): sampled from config
//! alone; `Phenotype::mutated` (`creature::phenotype`) applies them.

use rand::Rng;

use crate::config::PhenotypeConfig;

/// One phenotype step's sampled choices: a switch to another channel (its
/// index among the five channels other than the active one), whether the new
/// active channel's polarity flips, and the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhenotypeDraw {
    /// `Some(k)`: the active channel becomes the `k`-th of the other five,
    /// in channel order.
    pub switch_to_other: Option<usize>,
    /// Whether the (new) active channel's polarity flips.
    pub flip_polarity: bool,
    /// The step applied to the active channel, at least 1.
    pub step: u8,
}

/// Sample a phenotype step under `config`: the switch draw, its index among
/// the other five channels when it passes, then the polarity draw.
pub fn sample_phenotype_draw(config: &PhenotypeConfig, rng: &mut impl Rng) -> PhenotypeDraw {
    let switch_to_other =
        (rng.gen::<f32>() < config.channel_change_chance).then(|| rng.gen_range(0..5));
    let flip_polarity = rng.gen::<f32>() < config.polarity_flip_chance;
    PhenotypeDraw {
        switch_to_other,
        flip_polarity,
        step: config.channel_step.max(1),
    }
}

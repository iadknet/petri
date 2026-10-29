//! The body policy (T29.F02 invariant 3): one body event's draws, sampled
//! from config alone, and the operator each gene's event reports under.
//! `BodyGenome::apply` (`genome::body`) applies the sampled edit.

use std::fmt;

use rand::Rng;

use crate::config::MutationConfig;
use crate::genome::body::{BodyEdit, BodyGene};
use crate::mutation::event::ComplexityEffect;
use crate::mutation::phenotype::sample_phenotype_draw;

/// The mutation operator of one body gene's events: one per gene, keyed
/// `Body.<Gene>`, complexity-neutral.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyOperator(BodyGene);

impl BodyOperator {
    /// Every body operator, in chromosome order.
    pub const ALL: [Self; BodyGene::ALL.len()] = {
        let mut all = [Self(BodyGene::Colour); BodyGene::ALL.len()];
        let mut i = 0;
        while i < all.len() {
            all[i] = Self(BodyGene::ALL[i]);
            i += 1;
        }
        all
    };

    /// The gene this operator edits.
    #[must_use]
    pub const fn gene(self) -> BodyGene {
        self.0
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self.0 {
            BodyGene::Colour => "Body.Colour",
            BodyGene::KinTag => "Body.KinTag",
            BodyGene::Capacity => "Body.Capacity",
            BodyGene::StealShare => "Body.StealShare",
            BodyGene::GiveShare => "Body.GiveShare",
        }
    }

    /// A body edit changes no genome size.
    #[must_use]
    pub const fn complexity_effect(self) -> ComplexityEffect {
        ComplexityEffect::Neutral
    }
}

impl From<BodyGene> for BodyOperator {
    fn from(gene: BodyGene) -> Self {
        Self(gene)
    }
}

/// The gene's name (`Colour`), so the flat operator name reads `BodyColour`.
impl fmt::Debug for BodyOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

/// Sample one body event under `config`: its gene uniformly over
/// [`BodyGene::ALL`], then that gene's draws: the colour step's (switch,
/// index, polarity), the kin-tag bit in `0..32`, or a scalar's log-space step
/// uniform in `[-log_step, log_step]` (`capacity.log_step` for the capacity,
/// `share.log_step` for each share).
pub fn sample_body_edit(config: &MutationConfig, rng: &mut impl Rng) -> BodyEdit {
    match BodyGene::ALL[rng.gen_range(0..BodyGene::ALL.len())] {
        BodyGene::Colour => BodyEdit::Colour(sample_phenotype_draw(&config.phenotype, rng)),
        BodyGene::KinTag => BodyEdit::KinTag(rng.gen_range(0..u32::BITS)),
        BodyGene::Capacity => BodyEdit::Capacity(log_step(config.capacity.log_step, rng)),
        BodyGene::StealShare => BodyEdit::StealShare(log_step(config.share.log_step, rng)),
        BodyGene::GiveShare => BodyEdit::GiveShare(log_step(config.share.log_step, rng)),
    }
}

/// A log-space step uniform in `[-step, step]`.
fn log_step(step: f32, rng: &mut impl Rng) -> f32 {
    rng.gen_range(-step..=step)
}

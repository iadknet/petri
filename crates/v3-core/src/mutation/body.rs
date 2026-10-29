//! The body policy (T29.F01 invariant 8): every draw of a child's body,
//! sampled from config alone in today's order. `HeritableBody::inherit`
//! applies the sampled choices deterministically.

use rand::Rng;

use crate::config::{LogStepMutationConfig, MutationConfig};
use crate::mutation::phenotype::{sample_phenotype_draw, PhenotypeDraw};

/// Every body draw of one birth, in draw order: the phenotype step and the
/// kin bit (drawn only when the birth mutated), then the capacity, the steal
/// share and the give share log-steps (drawn on every birth).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDraws {
    /// The phenotype step, when the birth mutated.
    pub phenotype: Option<PhenotypeDraw>,
    /// The kin-tag bit to flip, in `0..32`, when the birth mutated.
    pub kin_bit: Option<u32>,
    /// The capacity's log-step, when its chance draw passed.
    pub capacity: Option<f32>,
    /// The steal share's log-step, when its chance draw passed.
    pub steal_share: Option<f32>,
    /// The give share's log-step, when its chance draw passed.
    pub give_share: Option<f32>,
}

/// Sample a birth's body draws under `config`. `mutated` is whether at
/// least one genome mutation event applied.
pub fn sample_body_draws(mutated: bool, config: &MutationConfig, rng: &mut impl Rng) -> BodyDraws {
    // WORKAROUND(G5): colour and kin tag are drawn only when a genome event applied; T29.F02 gives the body its own genes and supply and deletes this gate
    let (phenotype, kin_bit) = if mutated {
        let phenotype = sample_phenotype_draw(&config.phenotype, rng);
        (Some(phenotype), Some(rng.gen_range(0..32u32)))
    } else {
        (None, None)
    };
    BodyDraws {
        phenotype,
        kin_bit,
        capacity: sample_log_step(&config.capacity, rng),
        steal_share: sample_log_step(&config.share, rng),
        give_share: sample_log_step(&config.share, rng),
    }
}

/// One scalar's log-step: the chance draw, then, when it passes, the step
/// drawn uniformly in `[-log_step, log_step]`.
fn sample_log_step(config: &LogStepMutationConfig, rng: &mut impl Rng) -> Option<f32> {
    (rng.gen::<f32>() < config.chance).then(|| {
        let step = config.log_step;
        rng.gen_range(-step..=step)
    })
}

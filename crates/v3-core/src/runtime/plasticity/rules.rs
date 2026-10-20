//! The four Hebbian laws and the three-factor update (T11.F06, T11.F07),
//! applied per plastic edge (T25.F06).
//!
//! Learning parameters are read as stored: I1 keeps them in range by
//! construction and the load boundary checks it, so no law clamps them.

use crate::creature::genome::HebbianRule;

/// A pure-Hebbian edge's weight change for one committed visit, from the
/// edge's recorded `pre` and `post` samples and its visit-start weight `w`.
#[inline]
pub(crate) fn hebbian_delta(rule: HebbianRule, eta: f32, pre: f32, post: f32, w: f32) -> f32 {
    match rule {
        HebbianRule::Classic => eta * pre * post,
        HebbianRule::Oja => eta * post * (pre - w * post),
        HebbianRule::AntiHebb => -eta * pre * post,
        HebbianRule::Covariance => eta * (pre - 0.5) * (post - 0.5),
    }
}

/// A reward-modulated edge's trace contribution for one committed visit:
/// its rule's activity without the learning rate, which the reward applies.
#[inline]
pub(crate) fn activity(rule: HebbianRule, pre: f32, post: f32, w: f32) -> f32 {
    match rule {
        HebbianRule::Classic => pre * post,
        HebbianRule::Oja => post * (pre - w * post),
        HebbianRule::AntiHebb => -pre * post,
        HebbianRule::Covariance => (pre - 0.5) * (post - 0.5),
    }
}

/// The three-factor update: `w + eta * signal * trace`, bounded by the
/// edge's clamp.
#[inline]
pub(crate) fn three_factor(w: f32, eta: f32, signal: f32, trace: f32, clamp: f32) -> f32 {
    (w + eta * signal * trace).clamp(-clamp, clamp)
}

/// The learned-weight decay (T25.F09): `value` relaxed once toward its
/// birth value, `birth + retention · (value − birth)`.
#[inline]
pub(crate) fn relax(value: f32, birth: f32, retention: f32) -> f32 {
    birth + retention * (value - birth)
}

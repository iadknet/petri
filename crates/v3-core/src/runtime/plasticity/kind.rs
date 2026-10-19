//! The plastic-edge kind descriptor (T25.F06): an edge's rule, learning
//! pathway and inheritance, read by the learned-state view and the census.

use crate::creature::genome::{HebbianRule, PlasticityConfig};

/// One plastic edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlasticityKind {
    pub rule: HebbianRule,
    pub reward_modulated: bool,
    pub lamarckian: bool,
}

/// The four rules, in census order.
pub const RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

impl PlasticityKind {
    /// The kind of an edge with learning attributes `config`.
    #[must_use]
    pub fn of(config: &PlasticityConfig) -> Self {
        Self {
            rule: config.rule,
            reward_modulated: config.modulation.is_some(),
            lamarckian: config.lamarckian,
        }
    }

    /// Position of the rule in [`RULES`].
    #[must_use]
    pub const fn rule_index(self) -> usize {
        match self.rule {
            HebbianRule::Classic => 0,
            HebbianRule::Oja => 1,
            HebbianRule::AntiHebb => 2,
            HebbianRule::Covariance => 3,
        }
    }
}

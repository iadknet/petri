//! The heritable body: what a creature inherits beside its brain (T26.F01).
//!
//! [`HeritableBody`] carries the six-channel colour [`Phenotype`] and the
//! kin [`CreatureIdentityState`]. Seeding builds it with
//! [`HeritableBody::founder`], reproduction with [`HeritableBody::inherit`],
//! and the creature constructors take it as one argument. The record owns the
//! order of the two inheritance draws; `mutation::phenotype` and
//! `creature::identity` own how each part mutates.

use rand::Rng;

use crate::config::PhenotypeConfig;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::phenotype::Phenotype;
use crate::mutation::phenotype::mutate_phenotype;

/// Everything a creature inherits beside its genome and shared memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeritableBody {
    pub phenotype: Phenotype,
    pub identity: CreatureIdentityState,
}

impl HeritableBody {
    /// The body of founder `founder_index` of a run seeded with
    /// `startup_seed`: [`Phenotype::FOUNDER`] and
    /// [`CreatureIdentityState::founder`].
    #[must_use]
    pub fn founder(founder_index: usize, startup_seed: u64) -> Self {
        Self {
            phenotype: Phenotype::FOUNDER,
            identity: CreatureIdentityState::founder(founder_index, startup_seed),
        }
    }

    /// The child's body: a copy of `parent`'s and, when the birth `mutated`
    /// (at least one genome mutation event applied), a phenotype mutation
    /// followed by a kin-tag mutation, in that draw order. An unmutated birth
    /// draws nothing from `rng`.
    #[must_use]
    pub fn inherit(
        parent: &Self,
        mutated: bool,
        config: &PhenotypeConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let mut child = Self {
            phenotype: parent.phenotype,
            identity: CreatureIdentityState::inherit(&parent.identity),
        };
        if mutated {
            child.phenotype = mutate_phenotype(child.phenotype, config, rng);
            child.identity.mutate_kin_tag(rng);
        }
        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn config() -> PhenotypeConfig {
        PhenotypeConfig {
            channel_step: 3,
            channel_change_chance: 0.5,
            polarity_flip_chance: 0.5,
        }
    }

    fn phenotype() -> impl Strategy<Value = Phenotype> {
        (any::<[u8; 6]>(), 0usize..6, any::<[bool; 6]>()).prop_map(
            |(channels, active_channel, polarity)| Phenotype {
                channels,
                active_channel,
                polarity,
            },
        )
    }

    fn body() -> impl Strategy<Value = HeritableBody> {
        (phenotype(), any::<u32>(), any::<u32>()).prop_map(|(phenotype, lineage_id, kin_tag)| {
            HeritableBody {
                phenotype,
                identity: CreatureIdentityState {
                    lineage_id,
                    kin_tag,
                },
            }
        })
    }

    #[test]
    fn founder_pairs_the_founder_phenotype_with_the_founder_identity() {
        let body = HeritableBody::founder(4, 99);
        assert_eq!(body.phenotype, Phenotype::FOUNDER);
        assert_eq!(body.identity, CreatureIdentityState::founder(4, 99));
    }

    proptest! {
        /// A mutated inheritance is the phenotype mutation followed by the
        /// kin-tag mutation on one stream, and leaves the stream where those
        /// two calls leave it.
        #[test]
        fn a_mutated_inheritance_draws_phenotype_then_kin_tag(parent in body(), seed in any::<u64>()) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(&parent, true, &config(), &mut rng);

            let mut expected_rng = SmallRng::seed_from_u64(seed);
            let phenotype = mutate_phenotype(parent.phenotype, &config(), &mut expected_rng);
            let mut identity = parent.identity;
            identity.mutate_kin_tag(&mut expected_rng);

            prop_assert_eq!(child, HeritableBody { phenotype, identity });
            prop_assert_eq!(rng.gen::<u64>(), expected_rng.gen::<u64>());
        }

        /// An unmutated inheritance copies the parent and draws nothing.
        #[test]
        fn an_unmutated_inheritance_copies_the_parent_and_draws_nothing(parent in body(), seed in any::<u64>()) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(&parent, false, &config(), &mut rng);
            prop_assert_eq!(child, parent);
            prop_assert_eq!(rng.gen::<u64>(), SmallRng::seed_from_u64(seed).gen::<u64>());
        }

        /// A mutated inheritance keeps the lineage and flips one kin-tag bit.
        #[test]
        fn a_mutated_inheritance_keeps_lineage_and_flips_one_kin_bit(parent in body(), seed in any::<u64>()) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(&parent, true, &config(), &mut rng);
            prop_assert_eq!(child.identity.lineage_id, parent.identity.lineage_id);
            prop_assert_eq!((child.identity.kin_tag ^ parent.identity.kin_tag).count_ones(), 1);
        }
    }
}

//! Fixed lifetime totals for removed creatures, classified by carried structure.

use crate::creature::genome::StructuralCompanions;

/// Exhaustive accounting classes in precedence order, not an ability ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveClass {
    Plasticity,
    Stateful,
    SharedMemory,
    None,
}

impl CognitiveClass {
    pub const ALL: [Self; 4] = [
        Self::Plasticity,
        Self::Stateful,
        Self::SharedMemory,
        Self::None,
    ];

    #[must_use]
    pub fn from_companions(companions: &StructuralCompanions) -> Self {
        if companions.has_plasticity {
            Self::Plasticity
        } else if companions.has_stateful_compute_node {
            Self::Stateful
        } else if companions.reads_shared_memory || companions.writes_shared_memory {
            Self::SharedMemory
        } else {
            Self::None
        }
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Plasticity => "plasticity",
            Self::Stateful => "stateful",
            Self::SharedMemory => "shared_memory",
            Self::None => "none",
        }
    }
}

/// Raw exit-time observations; survivors have not entered these cohorts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReproductiveSuccessTotals {
    pub creatures_observed_total: u64,
    pub offspring_spawned_sum: u64,
    pub survival_ticks_sum: u64,
}

/// Exactly four rows, independent of population size or number of removals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReproductiveSuccessByCognitiveClass {
    pub by_class: [ReproductiveSuccessTotals; CognitiveClass::ALL.len()],
}

impl ReproductiveSuccessByCognitiveClass {
    /// Record one successful removal, using its existing offspring count and age.
    pub fn record(&mut self, class: CognitiveClass, offspring_spawned: u64, age: u64) {
        let totals = &mut self.by_class[class as usize];
        totals.creatures_observed_total += 1;
        totals.offspring_spawned_sum += offspring_spawned;
        totals.survival_ticks_sum += age;
    }

    #[must_use]
    pub fn totals(&self, class: CognitiveClass) -> ReproductiveSuccessTotals {
        self.by_class[class as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn companions(flags: [bool; 4]) -> StructuralCompanions {
        StructuralCompanions {
            functional_complexity: 17,
            reachable_node_count: 9,
            reads_shared_memory: flags[0],
            writes_shared_memory: flags[1],
            has_stateful_compute_node: flags[2],
            has_plasticity: flags[3],
        }
    }

    #[test]
    fn all_sixteen_structural_combinations_follow_class_precedence() {
        use CognitiveClass::{None, Plasticity, SharedMemory, Stateful};
        let expected = [
            None,
            SharedMemory,
            SharedMemory,
            SharedMemory,
            Stateful,
            Stateful,
            Stateful,
            Stateful,
            Plasticity,
            Plasticity,
            Plasticity,
            Plasticity,
            Plasticity,
            Plasticity,
            Plasticity,
            Plasticity,
        ];
        for (bits, expected) in expected.into_iter().enumerate() {
            let flags = std::array::from_fn(|bit| bits & (1 << bit) != 0);
            assert_eq!(
                CognitiveClass::from_companions(&companions(flags)),
                expected
            );
        }
    }

    #[test]
    fn runtime_storage_is_exactly_twelve_u64_totals() {
        assert_eq!(
            std::mem::size_of::<ReproductiveSuccessByCognitiveClass>(),
            12 * 8
        );
        assert_eq!(CognitiveClass::ALL.len(), 4);
    }

    #[test]
    fn cognitive_class_keys_match_the_report_contract() {
        assert_eq!(
            CognitiveClass::ALL.map(CognitiveClass::as_key),
            ["plasticity", "stateful", "shared_memory", "none"]
        );
    }

    proptest! {
        #[test]
        fn cognitive_classes_partition_every_structural_flag_combination(
            flags in any::<[bool; 4]>(),
            complexity in any::<u32>(),
            reachable in any::<usize>(),
        ) {
            let mut structure = companions(flags);
            structure.functional_complexity = complexity;
            structure.reachable_node_count = reachable;
            let predicates = [
                flags[3],
                !flags[3] && flags[2],
                !flags[3] && !flags[2] && (flags[0] || flags[1]),
                !flags.iter().any(|flag| *flag),
            ];
            prop_assert_eq!(predicates.iter().filter(|matched| **matched).count(), 1);
            let class = CognitiveClass::from_companions(&structure);
            for (candidate, matched) in CognitiveClass::ALL.into_iter().zip(predicates) {
                prop_assert_eq!(class == candidate, matched);
            }
        }

        #[test]
        fn reproductive_success_totals_are_additive_and_permutation_invariant(
            observations in prop::collection::vec((0usize..4, any::<u32>(), any::<u32>()), 0..64),
        ) {
            let mut forward = ReproductiveSuccessByCognitiveClass::default();
            let mut reverse = ReproductiveSuccessByCognitiveClass::default();
            for &(class, offspring, age) in &observations {
                forward.record(CognitiveClass::ALL[class], u64::from(offspring), u64::from(age));
            }
            for &(class, offspring, age) in observations.iter().rev() {
                reverse.record(CognitiveClass::ALL[class], u64::from(offspring), u64::from(age));
            }
            prop_assert_eq!(forward, reverse);
            for (index, class) in CognitiveClass::ALL.into_iter().enumerate() {
                let expected = observations.iter().filter(|(observed, _, _)| *observed == index)
                    .fold(ReproductiveSuccessTotals::default(), |mut sum, (_, offspring, age)| {
                        sum.creatures_observed_total += 1;
                        sum.offspring_spawned_sum += u64::from(*offspring);
                        sum.survival_ticks_sum += u64::from(*age);
                        sum
                    });
                prop_assert_eq!(forward.totals(class), expected);
            }
            prop_assert_eq!(
                CognitiveClass::ALL.into_iter().map(|class| forward.totals(class).creatures_observed_total).sum::<u64>(),
                observations.len() as u64,
            );
        }
    }
}

use rand::Rng;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::runtime::OUTPUT_SLOT_COUNT;

/// Generate a random input reference from the full set of 27 possible values.
///
/// Distribution: FoodHere (1) + Ring sensors (3) + StaticIntrospection (1) +
/// DynamicIntrospection (2) + ActionQueue (1) + Area summaries (3) +
/// Nearby creature (3) + decision state (5, T19.F05) + UpstreamSlot (8
/// weighted slots) = 27 total.
pub(crate) fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    random_input_reference_for_food_types(rng, 1)
}

/// Generate a random input reference while sampling typed food sensors from
/// `[0, food_type_count)`.
///
/// `food_type_count <= 1` soft-defaults to type 0 only.
pub(crate) fn random_input_reference_for_food_types(
    rng: &mut impl Rng,
    food_type_count: usize,
) -> InputReference {
    let idx = rng.gen_range(0u8..27);
    match idx {
        0 => InputReference::World(WorldInputKey::FoodHere {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        1 => InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        2 => InputReference::World(WorldInputKey::NeighborBarrierRing),
        3 => InputReference::World(WorldInputKey::NeighborOccupiedRing),
        4 => InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        5 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        6 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        7 => InputReference::ActionQueue,
        8 => InputReference::World(WorldInputKey::AreaFoodSummary {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        9 => InputReference::World(WorldInputKey::AreaBarrierSummary),
        10 => InputReference::World(WorldInputKey::NearbyCreatureCore),
        11 => InputReference::World(WorldInputKey::NearbyCreatureVitals),
        12 => InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        13 => InputReference::World(WorldInputKey::AreaOccupancySummary),
        14 => InputReference::ActionVotes,
        15 => InputReference::PreviousPassVotes,
        16 => InputReference::CommitCounts,
        17 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
        18 => InputReference::PreviousOutcome,
        _ => InputReference::UpstreamSlot(rng.gen_range(0..OUTPUT_SLOT_COUNT)),
    }
}

fn sample_food_type_id(food_type_count: usize, rng: &mut impl Rng) -> OrdinaryFoodTypeId {
    let capped = food_type_count.clamp(1, usize::from(u16::MAX) + 1);
    if capped <= 1 {
        return OrdinaryFoodTypeId::default();
    }
    OrdinaryFoodTypeId::new(rng.gen_range(0..capped) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[derive(Default)]
    struct ZeroCountingRng {
        draws: usize,
    }

    impl RngCore for ZeroCountingRng {
        fn next_u32(&mut self) -> u32 {
            self.draws += 1;
            0
        }

        fn next_u64(&mut self) -> u64 {
            self.draws += 1;
            0
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.draws += 1;
            dest.fill(0);
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn typed_sampler_single_food_mode_does_not_draw_extra_food_type_rng() {
        let mut rng = ZeroCountingRng::default();
        let sampled = random_input_reference_for_food_types(&mut rng, 1);
        assert_eq!(
            sampled,
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default()
            })
        );
        assert_eq!(
            rng.draws, 1,
            "single-food mode should only draw for input-ref key selection"
        );
    }

    #[test]
    fn typed_sampler_multi_food_mode_draws_for_food_type_idx() {
        let mut rng = ZeroCountingRng::default();
        let sampled = random_input_reference_for_food_types(&mut rng, 3);
        assert_eq!(
            sampled,
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default()
            })
        );
        assert_eq!(
            rng.draws, 2,
            "multi-food mode should draw once for key selection and once for food type_idx"
        );
    }
    #[test]
    fn typed_sampler_retains_both_energy_introspection_keys() {
        for (index, key) in [
            (5_u64, DynamicIntrospectionKey::EnergyCurrent),
            (6, DynamicIntrospectionKey::EnergyConsumedThisTick),
        ] {
            // The midpoint of this u32 bucket selects the exact catalog index
            // under rand's multiply-high uniform sampler, with no rejection.
            let draw = ((index << 32) + (1_u64 << 31)) / 27;
            let mut rng = rand::rngs::mock::StepRng::new(draw, 0);
            assert_eq!(
                random_input_reference_for_food_types(&mut rng, 1),
                InputReference::DynamicIntrospection(key)
            );
        }
    }

    /// The five decision-state entries sit at catalog indices 14 to 18 and
    /// each costs the key draw alone.
    #[test]
    fn typed_sampler_draws_each_decision_state_input_with_one_draw() {
        for (index, expected) in [
            (14_u64, InputReference::ActionVotes),
            (15, InputReference::PreviousPassVotes),
            (16, InputReference::CommitCounts),
            (
                17,
                InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
            ),
            (18, InputReference::PreviousOutcome),
        ] {
            for food_type_count in [1, 3] {
                let draw = ((index << 32) + (1_u64 << 31)) / 27;
                let mut rng = CountingStepRng {
                    inner: rand::rngs::mock::StepRng::new(draw, 0),
                    draws: 0,
                };
                assert_eq!(
                    random_input_reference_for_food_types(&mut rng, food_type_count),
                    expected
                );
                assert_eq!(rng.draws, 1, "index {index}");
            }
        }
    }

    /// Indices 19 to 26 are the upstream slot, which draws its slot.
    #[test]
    fn typed_sampler_upstream_range_starts_after_the_decision_state() {
        let draw = ((19_u64 << 32) + (1_u64 << 31)) / 27;
        let mut rng = CountingStepRng {
            inner: rand::rngs::mock::StepRng::new(draw, 0),
            draws: 0,
        };
        assert!(matches!(
            random_input_reference_for_food_types(&mut rng, 1),
            InputReference::UpstreamSlot(_)
        ));
        assert_eq!(rng.draws, 2);
    }

    struct CountingStepRng {
        inner: rand::rngs::mock::StepRng,
        draws: usize,
    }

    impl RngCore for CountingStepRng {
        fn next_u32(&mut self) -> u32 {
            self.draws += 1;
            self.inner.next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.draws += 1;
            self.inner.next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.draws += 1;
            self.inner.fill_bytes(dest);
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }
}

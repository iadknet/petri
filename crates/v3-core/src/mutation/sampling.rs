use rand::Rng;

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::mutation::compound::sub_value_count;
use crate::runtime::OUTPUT_SLOT_COUNT;

/// Generate a random input reference from the full set of 24 possible values.
///
/// Distribution: FoodHere (1) + Ring sensors (3) + StaticIntrospection (2) +
/// DynamicIntrospection (3) + ActionQueue (1) + Area summaries (3) +
/// Nearby creature (3) + UpstreamSlot (8 weighted slots) = 24 total.
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
    let idx = rng.gen_range(0u8..24);
    match idx {
        0 => InputReference::World(WorldInputKey::FoodHere {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        1 => InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        2 => InputReference::World(WorldInputKey::NeighborBarrierRing),
        3 => InputReference::World(WorldInputKey::NeighborOccupiedRing),
        4 => InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
        5 => InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        6 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        7 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        8 => InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::ReproductiveReserveCurrent,
        ),
        9 => InputReference::ActionQueue,
        10 => InputReference::World(WorldInputKey::AreaFoodSummary {
            type_idx: sample_food_type_id(food_type_count, rng),
        }),
        11 => InputReference::World(WorldInputKey::AreaBarrierSummary),
        12 => InputReference::World(WorldInputKey::NearbyCreatureCore),
        13 => InputReference::World(WorldInputKey::NearbyCreatureVitals),
        14 => InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        15 => InputReference::World(WorldInputKey::AreaOccupancySummary),
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

/// Sample a valid sub-index for an input reference based on its compound width.
pub(crate) fn sample_sub_idx_for_input_ref(
    reference: &InputReference,
    config: &MutationConfig,
    rng: &mut impl Rng,
) -> u16 {
    let width = sub_value_count(reference, config);
    if width <= 1 {
        return 0;
    }
    rng.gen_range(0..width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, RngCore, SeedableRng};

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
    fn typed_sampler_can_sample_live_reproductive_reserve() {
        let reserve = InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::ReproductiveReserveCurrent,
        );
        assert!((0..256).any(|seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            random_input_reference_for_food_types(&mut rng, 2) == reserve
        }));
    }
}

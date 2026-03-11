use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::mutation::compound::sub_value_count;

/// Generate a random input reference from the full set of 23 possible values.
///
/// Distribution: FoodHere (1) + Ring sensors (3) + StaticIntrospection (2) +
/// DynamicIntrospection (2) + ActionQueue (1) + Area summaries (3) +
/// Nearby creature (3) + UpstreamSlot (8 weighted slots) = 23 total.
pub(crate) fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    let idx = rng.gen_range(0u8..23);
    match idx {
        0 => InputReference::World(WorldInputKey::FoodHere),
        1 => InputReference::World(WorldInputKey::NeighborFoodRing),
        2 => InputReference::World(WorldInputKey::NeighborBarrierRing),
        3 => InputReference::World(WorldInputKey::NeighborOccupiedRing),
        4 => InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
        5 => InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        6 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        7 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        8 => InputReference::ActionQueue,
        9 => InputReference::World(WorldInputKey::AreaFoodSummary),
        10 => InputReference::World(WorldInputKey::AreaBarrierSummary),
        11 => InputReference::World(WorldInputKey::AreaOccupancySummary),
        12 => InputReference::World(WorldInputKey::NearbyCreatureCore),
        13 => InputReference::World(WorldInputKey::NearbyCreatureVitals),
        14 => InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        _ => InputReference::UpstreamSlot(rng.gen_range(0..12_usize)),
    }
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

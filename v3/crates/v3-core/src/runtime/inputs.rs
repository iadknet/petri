use crate::contracts::{DynamicIntrospectionKey, InputReference};
use crate::sensors::static_inputs::StaticInputs;

/// Resolve an `InputReference` to its current f32 value.
///
/// - World and static introspection keys are read from the pre-assembled snapshot.
/// - Dynamic introspection is resolved live from the `energy` and `energy_consumed` args.
/// - UpstreamSlot: reads `upstream_slots[idx]`; idx >= 12 yields 0.0.
/// - Missing or out-of-range index: 0.0 (soft default).
#[inline]
#[must_use]
pub fn resolve_input(
    reference: &InputReference,
    static_inputs: &StaticInputs,
    upstream_slots: &[f32; 12],
    energy: f32,
    energy_consumed: f32,
) -> f32 {
    match reference {
        InputReference::World(key) => static_inputs.resolve_world(key),
        InputReference::StaticIntrospection(key) => static_inputs.resolve_static(key),
        InputReference::DynamicIntrospection(key) => match key {
            DynamicIntrospectionKey::EnergyCurrent => energy,
            DynamicIntrospectionKey::EnergyConsumedThisTick => energy_consumed,
        },
        InputReference::UpstreamSlot(idx) => {
            if *idx < 12 {
                upstream_slots[*idx]
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
    };
    use crate::sensors::static_inputs::StaticInputs;

    fn make_static_inputs(food_here: f32) -> StaticInputs {
        StaticInputs {
            food_here,
            neighbor_food: [0.5; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 3.0,
            age_ticks: 10.0,
        }
    }

    #[test]
    fn world_food_here() {
        let si = make_static_inputs(0.75);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::World(WorldInputKey::FoodHere),
            &si,
            &upstream,
            50.0,
            0.0,
        );
        assert!((v - 0.75).abs() < 1e-6);
    }

    #[test]
    fn world_neighbor_food() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::World(WorldInputKey::NeighborCellFood(Direction::N)),
            &si,
            &upstream,
            50.0,
            0.0,
        );
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_generation() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            &si,
            &upstream,
            20.0,
            0.0,
        );
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_age_ticks() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            &si,
            &upstream,
            20.0,
            0.0,
        );
        assert!((v - 10.0).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_current_live() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            &si,
            &upstream,
            42.5,
            0.0,
        );
        assert!((v - 42.5).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_consumed_this_tick() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            &si,
            &upstream,
            20.0,
            5.5,
        );
        assert!((v - 5.5).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_in_range() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[7] = 99.0;
        let v = resolve_input(&InputReference::UpstreamSlot(7), &si, &upstream, 20.0, 0.0);
        assert!((v - 99.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_at_boundary_11() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[11] = 3.0;
        let v = resolve_input(&InputReference::UpstreamSlot(11), &si, &upstream, 20.0, 0.0);
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_out_of_range_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let v = resolve_input(&InputReference::UpstreamSlot(12), &si, &upstream, 20.0, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn upstream_slot_large_index_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let v = resolve_input(
            &InputReference::UpstreamSlot(999),
            &si,
            &upstream,
            20.0,
            0.0,
        );
        assert_eq!(v, 0.0);
    }
}

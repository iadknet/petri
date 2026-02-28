use crate::contracts::{DynamicIntrospectionKey, InputReference};
use crate::sensors::static_inputs::StaticInputs;

/// Shared resolution context for input references.
///
/// Extracted from graph's `EvalCtx` to be shared between graph evaluation,
/// VM execution, and any future resolution paths. Holds the immutable
/// snapshot and live energy values needed to resolve any `InputReference`.
#[derive(Debug, Clone)]
pub struct ResolveCtx<'a> {
    pub static_inputs: &'a StaticInputs,
    pub upstream_slots: &'a [f32; 12],
    pub energy: f32,
    pub energy_consumed: f32,
}

/// Resolve an `InputReference` to its current f32 value.
///
/// - `sub_idx`: sub-value index for compound inputs. For scalar inputs
///   (all current variants), `sub_idx > 0` returns `0.0`.
/// - World and static introspection keys are read from the pre-assembled snapshot.
/// - Dynamic introspection is resolved live from `ctx.energy` and `ctx.energy_consumed`.
/// - UpstreamSlot: reads `upstream_slots[idx]`; idx >= 12 yields 0.0.
/// - Missing or out-of-range index: 0.0 (soft default).
#[inline]
#[must_use]
pub fn resolve_input(reference: &InputReference, sub_idx: u16, ctx: &ResolveCtx<'_>) -> f32 {
    // All current input types are scalar: sub_idx > 0 returns 0.0.
    if sub_idx > 0 {
        return 0.0;
    }
    match reference {
        InputReference::World(key) => ctx.static_inputs.resolve_world(key),
        InputReference::StaticIntrospection(key) => ctx.static_inputs.resolve_static(key),
        InputReference::DynamicIntrospection(key) => match key {
            DynamicIntrospectionKey::EnergyCurrent => ctx.energy,
            DynamicIntrospectionKey::EnergyConsumedThisTick => ctx.energy_consumed,
        },
        InputReference::UpstreamSlot(idx) => {
            if *idx < 12 {
                ctx.upstream_slots[*idx]
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

    fn make_ctx<'a>(
        si: &'a StaticInputs,
        upstream: &'a [f32; 12],
        energy: f32,
        energy_consumed: f32,
    ) -> ResolveCtx<'a> {
        ResolveCtx {
            static_inputs: si,
            upstream_slots: upstream,
            energy,
            energy_consumed,
        }
    }

    #[test]
    fn world_food_here() {
        let si = make_static_inputs(0.75);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 50.0, 0.0);
        let v = resolve_input(&InputReference::World(WorldInputKey::FoodHere), 0, &ctx);
        assert!((v - 0.75).abs() < 1e-6);
    }

    #[test]
    fn world_neighbor_food() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 50.0, 0.0);
        let v = resolve_input(
            &InputReference::World(WorldInputKey::NeighborCellFood(Direction::N)),
            0,
            &ctx,
        );
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_generation() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            0,
            &ctx,
        );
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_age_ticks() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            0,
            &ctx,
        );
        assert!((v - 10.0).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_current_live() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 42.5, 0.0);
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            0,
            &ctx,
        );
        assert!((v - 42.5).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_consumed_this_tick() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 20.0, 5.5);
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            0,
            &ctx,
        );
        assert!((v - 5.5).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_in_range() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[7] = 99.0;
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(7), 0, &ctx);
        assert!((v - 99.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_at_boundary_11() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[11] = 3.0;
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(11), 0, &ctx);
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_out_of_range_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(12), 0, &ctx);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn upstream_slot_large_index_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let ctx = make_ctx(&si, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(999), 0, &ctx);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn sub_idx_nonzero_on_scalar_returns_zero() {
        let si = make_static_inputs(0.75);
        let mut upstream = [0.0f32; 12];
        upstream[0] = 5.0;
        let ctx = make_ctx(&si, &upstream, 50.0, 3.0);

        let refs = vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::UpstreamSlot(0),
        ];

        for r in &refs {
            // sub_idx=0 should return non-zero for these inputs
            let v0 = resolve_input(r, 0, &ctx);
            assert!(v0 != 0.0, "sub_idx=0 should return non-zero for {:?}", r);

            // sub_idx > 0 should always return 0.0 for scalar inputs
            for sub in [1u16, 2, 100, u16::MAX] {
                let v = resolve_input(r, sub, &ctx);
                assert_eq!(v, 0.0, "sub_idx={sub} should return 0.0 for scalar {:?}", r);
            }
        }
    }
}

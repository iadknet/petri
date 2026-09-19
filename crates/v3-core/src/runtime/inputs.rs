use crate::contracts::{ActionQueue, DynamicIntrospectionKey, InputReference};
use crate::runtime::OUTPUT_SLOT_COUNT;
use crate::sensors::perception::SensorSnapshot;

/// Shared resolution context for input references.
///
/// Extracted from graph's `EvalCtx` to be shared between graph evaluation,
/// VM execution, and any future resolution paths. Holds the immutable
/// snapshot and live energy values needed to resolve any `InputReference`.
#[derive(Debug, Clone)]
pub struct ResolveCtx<'a> {
    pub sensors: &'a SensorSnapshot,
    pub upstream_slots: &'a [f32; OUTPUT_SLOT_COUNT],
    pub energy: f32,
    pub energy_consumed: f32,
    pub action_queue: &'a ActionQueue,
}

/// Resolve an `InputReference` to its current f32 value.
///
/// - `sub_idx`: sub-value index for compound inputs (ActionQueue, compound
///   world keys). Scalar inputs ignore sub_idx. Compound inputs wrap via
///   `compound_width()`.
/// - World and static introspection keys are read from the pre-assembled snapshot.
/// - Dynamic introspection is resolved live from `ctx.energy` and `ctx.energy_consumed`.
/// - UpstreamSlot: reads `upstream_slots[idx]`; idx >= `OUTPUT_SLOT_COUNT` yields 0.0.
/// - Missing or out-of-range index: 0.0 (soft default).
#[inline]
#[must_use]
pub fn resolve_input(reference: &InputReference, sub_idx: u16, ctx: &ResolveCtx<'_>) -> f32 {
    match reference {
        // Compound input: ActionQueue uses sub_idx for two-level addressing.
        InputReference::ActionQueue => {
            let slot = (sub_idx / 3) as usize;
            match sub_idx % 3 {
                0 => ctx.action_queue.action_type_at(slot),
                1 => ctx.action_queue.param_at(slot, 0),
                _ => ctx.action_queue.param_at(slot, 1),
            }
        }
        // Compound world keys: use sub_idx for multi-field addressing.
        InputReference::World(key) if key.compound_width() > 1 => {
            ctx.sensors.resolve_compound(key, sub_idx)
        }
        // Scalar world keys: sub_idx is ignored.
        InputReference::World(key) => ctx.sensors.resolve_world(key),
        InputReference::StaticIntrospection(key) => ctx.sensors.local.resolve_static(key),
        InputReference::DynamicIntrospection(key) => match key {
            DynamicIntrospectionKey::EnergyCurrent => ctx.energy,
            DynamicIntrospectionKey::EnergyConsumedThisTick => ctx.energy_consumed,
        },
        InputReference::UpstreamSlot(idx) => {
            if *idx < OUTPUT_SLOT_COUNT {
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
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::{
        ActionQueue, Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey,
        WorldAction, WorldInputKey,
    };
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;
    use crate::sensors::typed_food::TypedFoodLocalSnapshot;

    fn make_sensor_snapshot(food_here: f32) -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here,
                neighbor_food: [0.5; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 3.0,
                age_ticks: 10.0,
            },
            typed_local_food: TypedFoodLocalSnapshot {
                food_here_by_type: vec![food_here],
                neighbor_food_by_type: vec![[0.5; 8]],
            },
            perception: PerceptionSnapshot::zeroed(1),
        }
    }

    fn make_ctx<'a>(
        ss: &'a SensorSnapshot,
        upstream: &'a [f32; OUTPUT_SLOT_COUNT],
        energy: f32,
        energy_consumed: f32,
    ) -> ResolveCtx<'a> {
        // Leak a default ActionQueue for test convenience (tests don't need to
        // read action queue via this helper).
        static EMPTY_AQ: std::sync::LazyLock<ActionQueue> =
            std::sync::LazyLock::new(|| ActionQueue::new(4));
        ResolveCtx {
            sensors: ss,
            upstream_slots: upstream,
            energy,
            energy_consumed,
            action_queue: &EMPTY_AQ,
        }
    }

    #[test]
    fn world_food_here() {
        let ss = make_sensor_snapshot(0.75);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);
        let v = resolve_input(
            &InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            0,
            &ctx,
        );
        assert!((v - 0.75).abs() < 1e-6);
    }

    #[test]
    fn world_neighbor_food_ring() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);
        // NeighborFoodRing is compound, sub_idx=0 → Direction::N
        let v = resolve_input(
            &InputReference::World(WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::default(),
            )),
            0,
            &ctx,
        );
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_generation() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            0,
            &ctx,
        );
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_age_ticks() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            0,
            &ctx,
        );
        assert!((v - 10.0).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_current_live() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 42.5, 0.0);
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            0,
            &ctx,
        );
        assert!((v - 42.5).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_consumed_this_tick() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 20.0, 5.5);
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            0,
            &ctx,
        );
        assert!((v - 5.5).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_in_range() {
        let ss = make_sensor_snapshot(0.0);
        let mut upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        upstream[7] = 99.0;
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(7), 0, &ctx);
        assert!((v - 99.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_at_last_valid_boundary() {
        let ss = make_sensor_snapshot(0.0);
        let mut upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let last_slot = OUTPUT_SLOT_COUNT - 1;
        upstream[last_slot] = 3.0;
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(last_slot), 0, &ctx);
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_out_of_range_yields_zero() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [1.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(OUTPUT_SLOT_COUNT), 0, &ctx);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn upstream_slot_large_index_yields_zero() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [1.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 20.0, 0.0);
        let v = resolve_input(&InputReference::UpstreamSlot(999), 0, &ctx);
        assert_eq!(v, 0.0);
    }

    // ── ActionQueue compound input tests ────────────────────────────────

    #[test]
    fn action_queue_sub_idx_0_returns_action_type() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let mut aq = ActionQueue::new(4);
        aq.push(WorldAction::eat(OrdinaryFoodTypeId::default())); // type 1
        let ctx = ResolveCtx {
            sensors: &ss,
            upstream_slots: &upstream,
            energy: 50.0,
            energy_consumed: 0.0,
            action_queue: &aq,
        };
        // sub_idx=0 → slot 0, field 0 (action_type)
        let v = resolve_input(&InputReference::ActionQueue, 0, &ctx);
        assert!((v - 1.0).abs() < f32::EPSILON, "Eat action type = 1.0");
    }

    #[test]
    fn action_queue_sub_idx_maps_slot_and_field() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let mut aq = ActionQueue::new(4);
        aq.push(WorldAction::NoOp); // slot 0: type=0
        aq.push(WorldAction::Move(Direction::E)); // slot 1: type=2, param0=2.0 (E direction index)
        let ctx = ResolveCtx {
            sensors: &ss,
            upstream_slots: &upstream,
            energy: 50.0,
            energy_consumed: 0.0,
            action_queue: &aq,
        };
        // sub_idx=3 → slot 1 (3/3=1), field 0 (3%3=0) = action_type = 2.0 (Move)
        let v = resolve_input(&InputReference::ActionQueue, 3, &ctx);
        assert!((v - 2.0).abs() < f32::EPSILON, "Move action type = 2.0");
        // sub_idx=4 → slot 1, field 1 = param0 = direction = 2.0 (E)
        let v = resolve_input(&InputReference::ActionQueue, 4, &ctx);
        assert!(
            (v - 2.0).abs() < f32::EPSILON,
            "Move param0 = E direction = 2.0"
        );
    }

    #[test]
    fn action_queue_param1_field_coverage() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let mut aq = ActionQueue::new(4);
        aq.push(WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 0.42,
        });
        let ctx = ResolveCtx {
            sensors: &ss,
            upstream_slots: &upstream,
            energy: 50.0,
            energy_consumed: 0.0,
            action_queue: &aq,
        };
        // sub_idx=2 → slot 0, field 2 = param1 = energy_transfer_fraction = 0.42
        let v = resolve_input(&InputReference::ActionQueue, 2, &ctx);
        assert!(
            (v - 0.42).abs() < f32::EPSILON,
            "Reproduce param1 = energy_transfer_fraction = 0.42, got {v}"
        );
    }

    #[test]
    fn action_queue_oob_returns_zero() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let aq = ActionQueue::new(4); // empty queue
        let ctx = ResolveCtx {
            sensors: &ss,
            upstream_slots: &upstream,
            energy: 50.0,
            energy_consumed: 0.0,
            action_queue: &aq,
        };
        // Any sub_idx on empty queue returns 0.0
        assert_eq!(resolve_input(&InputReference::ActionQueue, 0, &ctx), 0.0);
        assert_eq!(resolve_input(&InputReference::ActionQueue, 5, &ctx), 0.0);
        assert_eq!(resolve_input(&InputReference::ActionQueue, 100, &ctx), 0.0);
    }

    #[test]
    fn sub_idx_ignored_on_scalar() {
        let ss = make_sensor_snapshot(0.75);
        let mut upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        upstream[0] = 5.0;
        let ctx = make_ctx(&ss, &upstream, 50.0, 3.0);

        let refs = vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::UpstreamSlot(0),
        ];

        for r in &refs {
            let v0 = resolve_input(r, 0, &ctx);
            assert!(v0 != 0.0, "sub_idx=0 should return non-zero for {:?}", r);

            // sub_idx > 0 returns the SAME value as sub_idx == 0 for scalar inputs
            for sub in [1u16, 2, 100, u16::MAX] {
                let v = resolve_input(r, sub, &ctx);
                assert_eq!(
                    v, v0,
                    "sub_idx={sub} should return same value as sub_idx=0 for scalar {:?}",
                    r
                );
            }
        }
    }

    // ── Extended perception routing tests ─────────────────────────────────

    #[test]
    fn extended_key_routes_through_perception_snapshot() {
        let mut ss = make_sensor_snapshot(0.0);
        ss.perception.typed_area_food[0][0] = 0.42;
        ss.perception.typed_area_food[0][6] = 0.88;
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);

        let v0 = resolve_input(
            &InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            }),
            0,
            &ctx,
        );
        assert!((v0 - 0.42).abs() < f32::EPSILON);

        let v6 = resolve_input(
            &InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            }),
            6,
            &ctx,
        );
        assert!((v6 - 0.88).abs() < f32::EPSILON);
    }

    #[test]
    fn compound_oob_sub_idx_wraps() {
        let mut ss = make_sensor_snapshot(0.0);
        ss.perception.typed_area_food[0][0] = 0.42;
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);

        // area_food has 7 sub-values; sub_idx=7 wraps to index 0
        let v = resolve_input(
            &InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            }),
            7,
            &ctx,
        );
        assert!(
            (v - 0.42).abs() < f32::EPSILON,
            "sub_idx=7 should wrap to index 0 (0.42), got {v}"
        );

        // Large sub_idx also wraps
        let v_large = resolve_input(
            &InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            }),
            14,
            &ctx,
        );
        assert!(
            (v_large - 0.42).abs() < f32::EPSILON,
            "sub_idx=14 should wrap to index 0 (0.42), got {v_large}"
        );
    }

    #[test]
    fn extended_key_nearby_core_routes_correctly() {
        let mut ss = make_sensor_snapshot(0.0);
        ss.perception.nearby_core[0] = 1.0; // present
        ss.perception.nearby_core[3] = 0.75; // dist
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);

        let present = resolve_input(
            &InputReference::World(WorldInputKey::NearbyCreatureCore),
            0,
            &ctx,
        );
        assert!((present - 1.0).abs() < f32::EPSILON);

        let dist = resolve_input(
            &InputReference::World(WorldInputKey::NearbyCreatureCore),
            3,
            &ctx,
        );
        assert!((dist - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn extended_key_identity_routes_correctly() {
        let mut ss = make_sensor_snapshot(0.0);
        ss.perception.nearby_identity[0] = 0.9; // kin_affinity slot 0
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);

        let v = resolve_input(
            &InputReference::World(WorldInputKey::NearbyCreatureIdentity),
            0,
            &ctx,
        );
        assert!((v - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn zero_perception_returns_zeros_for_all_extended_keys() {
        let ss = make_sensor_snapshot(0.0);
        let upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        let ctx = make_ctx(&ss, &upstream, 50.0, 0.0);

        let keys = [
            WorldInputKey::AreaFoodSummary {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            },
            WorldInputKey::AreaBarrierSummary,
            WorldInputKey::AreaOccupancySummary,
            WorldInputKey::NearbyCreatureCore,
            WorldInputKey::NearbyCreatureVitals,
            WorldInputKey::NearbyCreatureIdentity,
        ];

        for key in &keys {
            for sub in 0..20u16 {
                let v = resolve_input(&InputReference::World(*key), sub, &ctx);
                assert_eq!(
                    v, 0.0,
                    "zero perception should return 0.0 for {:?} sub_idx={sub}",
                    key
                );
            }
        }
    }
}

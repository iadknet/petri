use crate::config::{EnergyLifecycleConfig, OrdinaryFoodTypeId};
use crate::contracts::{Direction, StaticIntrospectionKey, WorldInputKey};
use crate::creature::genome::{OutcomeChannel, OUTCOME_CHANNEL_COUNT};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;

/// Snapshot of world and static-introspection sensor values for one creature's turn.
///
/// Assembled once at turn start; dynamic introspection (energy, consumed) is
/// resolved live during mesh evaluation as a fraction of `max_energy`, which
/// the snapshot carries from the lifecycle config for that purpose.
///
/// All food values are normalized to [0.0, 1.0] by clamping raw food density;
/// introspection is on the unit scale (T17.F02).
#[derive(Debug, Clone, PartialEq)]
pub struct StaticInputs {
    /// Food density at the creature's current cell, normalized to [0.0, 1.0].
    pub food_here: f32,
    /// Food density of each neighboring cell, indexed by Direction::to_index().
    /// 0.0 for out-of-bounds (bounded edge mode) neighbors.
    pub neighbor_food: [f32; 8],
    /// Whether each neighboring cell has a barrier (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_barrier: [f32; 8],
    /// Whether each neighboring cell is occupied by a creature (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_occupied: [f32; 8],
    /// Creature's age as a saturating fraction of
    /// `EnergyLifecycleConfig::age_reference_ticks`, in [0.0, 1.0].
    pub age_ticks: f32,
    /// `EnergyLifecycleConfig::max_energy`: the denominator for the live
    /// energy introspection reads.
    pub max_energy: f32,
    /// The previous tick's outcome channels (`PreviousOutcome`, T19.F05),
    /// indexed by `OutcomeChannel` discriminant: the energy channels as
    /// fractions of `max_energy`, the other two as stored.
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

/// The `PreviousOutcome` read of a stored outcome bank: `EnergyDelta` over
/// `max_energy` clamped to [-1, 1], `DamageDelta` over `max_energy` clamped
/// to [-1, 0], `ActionSuccess` and `OffspringSuccess` as stored.
#[must_use]
pub fn previous_outcome_read(
    stored: &[f32; OUTCOME_CHANNEL_COUNT],
    max_energy: f32,
) -> [f32; OUTCOME_CHANNEL_COUNT] {
    let mut read = *stored;
    let energy = OutcomeChannel::EnergyDelta as usize;
    let damage = OutcomeChannel::DamageDelta as usize;
    read[energy] = (stored[energy] / max_energy).clamp(-1.0, 1.0);
    read[damage] = (stored[damage] / max_energy).clamp(-1.0, 0.0);
    read
}

/// `value / max` clamped to [0, 1]: the unit-scale read of an energy quantity
/// (`EnergyCurrent`, `EnergyConsumedThisTick`). A negative `value` (the VM's
/// mid-dispatch `effective` energy can be) reads 0.
#[inline]
#[must_use]
pub fn energy_fraction(value: f32, max_energy: f32) -> f32 {
    (value / max_energy).clamp(0.0, 1.0)
}

/// `min(age / reference, 1)` as f32 division: the unit-scale `AgeTicks` read.
#[inline]
#[must_use]
pub fn age_fraction(age: u64, age_reference_ticks: u64) -> f32 {
    (age as f32 / age_reference_ticks as f32).min(1.0)
}

impl StaticInputs {
    /// Look up a resolved f32 value by WorldInputKey.
    ///
    /// Only handles `FoodHere` (scalar). Ring and extended perception compound
    /// keys are resolved through `SensorSnapshot::resolve_compound()`.
    pub fn resolve_world(&self, key: &WorldInputKey) -> f32 {
        match key {
            WorldInputKey::FoodHere { type_idx } if *type_idx == OrdinaryFoodTypeId::default() => {
                self.food_here
            }
            WorldInputKey::FoodHere { .. } => 0.0,
            // Ring + compound keys are resolved via SensorSnapshot::resolve_compound(), not here.
            _ => 0.0,
        }
    }

    /// Look up a resolved f32 value by StaticIntrospectionKey.
    pub fn resolve_static(&self, key: &StaticIntrospectionKey) -> f32 {
        match key {
            StaticIntrospectionKey::AgeTicks => self.age_ticks,
        }
    }
}

/// Assemble all static sensor values for one creature at turn start.
///
/// Edge-mode behaviour for neighbor cells:
/// - `WorldEdgeMode::Wrap`: neighbor coordinates are wrapped toroidally, so every
///   direction always resolves to a valid cell and is sampled normally.
/// - `WorldEdgeMode::Bounded`: a neighbor that falls outside the grid bounds resolves
///   to `None`. All sensor slots for such out-of-bounds neighbors are set to 0.0
///   (no food, no barrier signal, not occupied).
pub fn assemble_static_inputs(
    world: &WorldState,
    creature: &CreatureState,
    lifecycle: &EnergyLifecycleConfig,
) -> StaticInputs {
    let pos = creature.position;
    let food_here = world.food_at(pos).clamp(0.0, 1.0);

    let mut neighbor_food = [0.0f32; 8];
    let mut neighbor_barrier = [0.0f32; 8];
    let mut neighbor_occupied = [0.0f32; 8];

    for dir in Direction::ALL {
        let idx = dir.to_index();
        match world.resolve_neighbor(pos, dir) {
            Some(npos) => {
                neighbor_food[idx] = world.food_at(npos).clamp(0.0, 1.0);
                neighbor_barrier[idx] = if world.is_barrier(npos) { 1.0 } else { 0.0 };
                neighbor_occupied[idx] = if world.creature_at(npos).is_some() {
                    1.0
                } else {
                    0.0
                };
            }
            None => {
                // Bounded-edge out-of-bounds: soft default 0.0 for all fields.
            }
        }
    }

    StaticInputs {
        food_here,
        neighbor_food,
        neighbor_barrier,
        neighbor_occupied,
        age_ticks: age_fraction(creature.age, lifecycle.age_reference_ticks),
        max_energy: lifecycle.max_energy,
        previous_outcome: previous_outcome_read(&creature.previous_outcome, lifecycle.max_energy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SimulationConfig, WorldEdgeMode};
    use crate::contracts::{CreatureId, NodeId, Position};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use proptest::prelude::*;
    use slotmap::SlotMap;

    fn make_world(w: u16, h: u16) -> WorldState {
        WorldState::new(w, h, WorldEdgeMode::Wrap)
    }

    fn make_creature(id: CreatureId, pos: Position) -> CreatureState {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        CreatureState::new(
            id,
            genome,
            pos,
            20.0,
            0,
            [128, 64, 32, 10, 20, 30],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    }

    fn get_id() -> CreatureId {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        sm.insert(())
    }

    #[test]
    fn food_here_zero_when_no_food() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        assert_eq!(si.food_here, 0.0);
    }

    #[test]
    fn food_here_normalized_to_one_when_max() {
        let mut world = make_world(4, 4);
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut food_cfg = SimulationConfig::default().world.food;
        food_cfg.initial_coverage = 1.0;
        food_cfg.initial_density = 1.0;
        food_cfg.types[0].initial_coverage = 1.0;
        food_cfg.types[0].initial_density = 1.0;
        world.reconfigure_food(food_cfg);
        let mut rng = SmallRng::seed_from_u64(42);
        world.seed_food(&mut rng);
        let id = get_id();
        let creature = make_creature(id, Position::new(1, 1));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        assert!((si.food_here - 1.0).abs() < 1e-6);
    }

    #[test]
    fn neighbor_food_all_one_when_all_cells_max_food() {
        let mut world = make_world(4, 4);
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut food_cfg = SimulationConfig::default().world.food;
        food_cfg.initial_coverage = 1.0;
        food_cfg.initial_density = 1.0;
        food_cfg.types[0].initial_coverage = 1.0;
        food_cfg.types[0].initial_density = 1.0;
        world.reconfigure_food(food_cfg);
        let mut rng = SmallRng::seed_from_u64(0);
        world.seed_food(&mut rng);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        for i in 0..8 {
            assert!(
                (si.neighbor_food[i] - 1.0).abs() < 1e-6,
                "neighbor_food[{i}]"
            );
        }
    }

    #[test]
    fn neighbor_barrier_detected_north() {
        let mut world = make_world(5, 5);
        world.set_barrier(Position::new(2, 1), true);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        assert_eq!(si.neighbor_barrier[Direction::N.to_index()], 1.0);
        assert_eq!(si.neighbor_barrier[Direction::S.to_index()], 0.0);
    }

    #[test]
    fn neighbor_occupied_detected_east() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id1 = sm.insert(());
        let id2 = sm.insert(());
        let mut world = make_world(5, 5);
        world.place_creature(Position::new(3, 2), id2);
        let creature = make_creature(id1, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        assert_eq!(si.neighbor_occupied[Direction::E.to_index()], 1.0);
        assert_eq!(si.neighbor_occupied[Direction::W.to_index()], 0.0);
    }

    #[test]
    fn static_introspection_age_is_a_fraction_of_the_reference_span() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let world = make_world(4, 4);
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let mut creature = CreatureState::new(
            id,
            genome,
            Position::new(1, 1),
            20.0,
            5,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        );
        creature.age = 42;
        let lifecycle = EnergyLifecycleConfig {
            max_energy: 150.0,
            age_reference_ticks: 400,
            ..EnergyLifecycleConfig::default()
        };
        let si = assemble_static_inputs(&world, &creature, &lifecycle);
        assert!((si.age_ticks - 0.105).abs() < 1e-6);
        assert_eq!(si.max_energy, 150.0);
        assert_eq!(
            si.resolve_static(&StaticIntrospectionKey::AgeTicks),
            si.age_ticks
        );
    }

    #[test]
    fn age_fraction_saturates_at_the_reference_span() {
        assert_eq!(age_fraction(0, 500), 0.0);
        assert_eq!(age_fraction(500, 500), 1.0);
        assert_eq!(age_fraction(10_000, 500), 1.0);
        assert!((age_fraction(20, 500) - 0.04).abs() < 1e-7);
    }

    #[test]
    fn energy_fraction_clamps_negative_and_overfull_reads() {
        assert_eq!(energy_fraction(-5.0, 200.0), 0.0);
        assert_eq!(energy_fraction(0.0, 200.0), 0.0);
        assert_eq!(energy_fraction(200.0, 200.0), 1.0);
        assert_eq!(energy_fraction(260.0, 200.0), 1.0);
        assert!((energy_fraction(32.0, 200.0) - 0.16).abs() < 1e-7);
    }

    proptest! {
        /// T17.F02 invariant 8: every unit-scale read lies in [0, 1] and is
        /// non-decreasing in its numerator.
        #[test]
        fn energy_fraction_is_bounded_and_monotone(
            lo in -1.0e6f32..1.0e6,
            step in 0.0f32..1.0e6,
            max_energy in 1.0f32..1.0e6,
        ) {
            let a = energy_fraction(lo, max_energy);
            let b = energy_fraction(lo + step, max_energy);
            prop_assert!((0.0..=1.0).contains(&a), "{a}");
            prop_assert!((0.0..=1.0).contains(&b), "{b}");
            prop_assert!(a <= b, "{lo} -> {a}, {} -> {b}", lo + step);
        }

        #[test]
        fn age_fraction_is_bounded_and_monotone(
            age in 0u64..1_000_000,
            step in 0u64..1_000_000,
            reference in 1u64..1_000_000,
        ) {
            let a = age_fraction(age, reference);
            let b = age_fraction(age + step, reference);
            prop_assert!((0.0..=1.0).contains(&a), "{a}");
            prop_assert!((0.0..=1.0).contains(&b), "{b}");
            prop_assert!(a <= b);
        }
    }

    #[test]
    fn resolve_world_food_here_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(1, 1));
        let si = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
        let result = si.resolve_world(&WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        });
        assert_eq!(result, 0.0);
    }

    /// The `PreviousOutcome` read scales the two energy channels by
    /// `max_energy` into their clamps and passes the other two through.
    #[test]
    fn previous_outcome_read_scales_and_clamps_the_energy_channels() {
        assert_eq!(
            previous_outcome_read(&[50.0, 0.5, -20.0, 2.0], 200.0),
            [0.25, 0.5, -0.1, 2.0]
        );
        assert_eq!(
            previous_outcome_read(&[-900.0, 1.0, -900.0, 0.0], 200.0),
            [-1.0, 1.0, -1.0, 0.0]
        );
        assert_eq!(
            previous_outcome_read(&[900.0, 0.0, 5.0, 0.0], 200.0),
            [1.0, 0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn assembly_reads_the_creature_outcome_store() {
        let world = make_world(8, 8);
        let mut ids: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let mut creature = make_creature(ids.insert(()), Position::new(2, 2));
        let lifecycle = SimulationConfig::default().energy.lifecycle;
        assert_eq!(
            assemble_static_inputs(&world, &creature, &lifecycle).previous_outcome,
            [0.0; OUTCOME_CHANNEL_COUNT]
        );
        creature.previous_outcome = [lifecycle.max_energy / 4.0, 1.0, 0.0, 1.0];
        assert_eq!(
            assemble_static_inputs(&world, &creature, &lifecycle).previous_outcome,
            [0.25, 1.0, 0.0, 1.0]
        );
    }
}

use slotmap::SecondaryMap;

use crate::config::EnergyConfig;
use crate::contracts::{CreatureId, Direction, Position, StaticIntrospectionKey};
// WORKAROUND(G2): OutcomeChannel moves to contracts (T25.F11)
use crate::genome::{OutcomeChannel, OUTCOME_CHANNEL_COUNT};
use crate::kernel::WorldState;

/// What sensors read of one body: the creature record builds it
/// (`CreatureState::sensed_body`) with every body rule already applied, so
/// sensors compute none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyView {
    pub position: Position,
    pub age: u64,
    pub energy: f32,
    /// The body's energy sensor denominator (`HeritableBody::energy_scale`).
    pub energy_scale: f32,
    /// The seen colour on the unit scale (`creature::colour::unit_rgb`).
    pub colour: [f32; 3],
    pub kin_tag: u32,
    /// The stored outcome bank, indexed by `OutcomeChannel` discriminant.
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

/// One tick's body views of every living creature, which the scheduler
/// fills once per assembly call.
pub type BodyViews = SecondaryMap<CreatureId, BodyView>;

/// Snapshot of world and static-introspection sensor values for one creature's turn.
///
/// Assembled once at turn start; dynamic introspection (energy, consumed) is
/// resolved live during mesh evaluation as a fraction of `energy_scale`, the
/// creature's sensor denominator the snapshot carries for that purpose.
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
    /// The creature's own colour on the unit scale
    /// (`creature::colour::unit_rgb`): `OwnRed`, `OwnGreen`, `OwnBlue`.
    pub own_colour: [f32; 3],
    /// The creature's energy sensor denominator
    /// ([`HeritableBody::energy_scale`](crate::creature::body::HeritableBody::energy_scale),
    /// T26.F02): the reference capacity or the creature's own, by
    /// `energy.capacity.sensor_scale`. Divides the live energy introspection
    /// reads and the `PreviousOutcome` energy channels.
    pub energy_scale: f32,
    /// The previous tick's outcome channels (`PreviousOutcome`, T19.F05),
    /// indexed by `OutcomeChannel` discriminant: the energy channels as
    /// fractions of `energy_scale`, the other two as stored.
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

/// The `PreviousOutcome` read of a stored outcome bank: `EnergyDelta` over
/// `energy_scale` clamped to [-1, 1], `DamageDelta` over `energy_scale`
/// clamped to [-1, 0], `ActionSuccess` and `OffspringSuccess` as stored.
#[must_use]
pub fn previous_outcome_read(
    stored: &[f32; OUTCOME_CHANNEL_COUNT],
    energy_scale: f32,
) -> [f32; OUTCOME_CHANNEL_COUNT] {
    let mut read = *stored;
    let energy = OutcomeChannel::EnergyDelta as usize;
    let damage = OutcomeChannel::DamageDelta as usize;
    read[energy] = (stored[energy] / energy_scale).clamp(-1.0, 1.0);
    read[damage] = (stored[damage] / energy_scale).clamp(-1.0, 0.0);
    read
}

/// `value / max` clamped to [0, 1]: the unit-scale read of an energy quantity
/// (`EnergyCurrent`, `EnergyConsumedThisTick`). A negative `value` (the VM's
/// mid-dispatch `effective` energy can be) reads 0.
#[inline]
#[must_use]
pub fn energy_fraction(value: f32, energy_scale: f32) -> f32 {
    (value / energy_scale).clamp(0.0, 1.0)
}

/// `min(age / reference, 1)` as f32 division: the unit-scale `AgeTicks` read.
#[inline]
#[must_use]
pub fn age_fraction(age: u64, age_reference_ticks: u64) -> f32 {
    (age as f32 / age_reference_ticks as f32).min(1.0)
}

impl StaticInputs {
    /// Look up a resolved f32 value by StaticIntrospectionKey.
    pub fn resolve_static(&self, key: &StaticIntrospectionKey) -> f32 {
        match key {
            StaticIntrospectionKey::AgeTicks => self.age_ticks,
            StaticIntrospectionKey::OwnRed => self.own_colour[0],
            StaticIntrospectionKey::OwnGreen => self.own_colour[1],
            StaticIntrospectionKey::OwnBlue => self.own_colour[2],
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
    body: &BodyView,
    energy: &EnergyConfig,
) -> StaticInputs {
    let pos = body.position;
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
        age_ticks: age_fraction(body.age, energy.lifecycle.age_reference_ticks),
        own_colour: body.colour,
        energy_scale: body.energy_scale,
        previous_outcome: previous_outcome_read(&body.previous_outcome, body.energy_scale),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SimulationConfig, WorldEdgeMode};
    use crate::contracts::CreatureId;
    use crate::kernel::WorldState;
    use proptest::prelude::*;
    use slotmap::SlotMap;

    fn make_world(w: u16, h: u16) -> WorldState {
        WorldState::new(w, h, WorldEdgeMode::Wrap)
    }

    fn view_at(position: Position) -> BodyView {
        BodyView {
            position,
            age: 0,
            energy: 20.0,
            energy_scale: 200.0,
            colour: [0.5, 0.25, 0.125],
            kin_tag: 0,
            previous_outcome: [0.0; OUTCOME_CHANNEL_COUNT],
        }
    }

    fn assemble(world: &WorldState, view: &BodyView) -> StaticInputs {
        assemble_static_inputs(world, view, &EnergyConfig::default())
    }

    #[test]
    fn food_here_zero_when_no_food() {
        let world = make_world(4, 4);
        let si = assemble(&world, &view_at(Position::new(2, 2)));
        assert_eq!(si.food_here, 0.0);
    }

    fn full_food_world(seed: u64) -> WorldState {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut world = make_world(4, 4);
        let mut food_cfg = SimulationConfig::default().world.food;
        food_cfg.types[0].initial_coverage = 1.0;
        food_cfg.types[0].initial_density = 1.0;
        world.reconfigure_food(food_cfg);
        world.seed_food(&mut SmallRng::seed_from_u64(seed));
        world
    }

    #[test]
    fn food_here_normalized_to_one_when_max() {
        let world = full_food_world(42);
        let si = assemble(&world, &view_at(Position::new(1, 1)));
        assert!((si.food_here - 1.0).abs() < 1e-6);
    }

    #[test]
    fn neighbor_food_all_one_when_all_cells_max_food() {
        let world = full_food_world(0);
        let si = assemble(&world, &view_at(Position::new(2, 2)));
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
        let si = assemble(&world, &view_at(Position::new(2, 2)));
        assert_eq!(si.neighbor_barrier[Direction::N.to_index()], 1.0);
        assert_eq!(si.neighbor_barrier[Direction::S.to_index()], 0.0);
    }

    #[test]
    fn neighbor_occupied_detected_east() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let neighbour = sm.insert(());
        let mut world = make_world(5, 5);
        world.place_creature(Position::new(3, 2), neighbour);
        let si = assemble(&world, &view_at(Position::new(2, 2)));
        assert_eq!(si.neighbor_occupied[Direction::E.to_index()], 1.0);
        assert_eq!(si.neighbor_occupied[Direction::W.to_index()], 0.0);
    }

    /// The view's age, colour and energy scale reach the snapshot through
    /// the sensor rules alone: age as a fraction of the reference span, the
    /// colour and the scale as resolved by the creature record.
    #[test]
    fn static_introspection_reads_the_body_view() {
        let world = make_world(4, 4);
        let view = BodyView {
            age: 42,
            energy_scale: 150.0,
            ..view_at(Position::new(1, 1))
        };
        let mut energy = EnergyConfig::default();
        energy.lifecycle.age_reference_ticks = 400;
        let si = assemble_static_inputs(&world, &view, &energy);
        assert!((si.age_ticks - 0.105).abs() < 1e-6);
        assert_eq!(si.energy_scale, 150.0);
        assert_eq!(si.own_colour, view.colour);
        assert_eq!(
            si.resolve_static(&StaticIntrospectionKey::AgeTicks),
            si.age_ticks
        );
        assert_eq!(
            si.resolve_static(&StaticIntrospectionKey::OwnGreen),
            view.colour[1]
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

    /// The `PreviousOutcome` read scales the two energy channels by
    /// `energy_scale` into their clamps and passes the other two through.
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
    fn assembly_reads_the_viewed_outcome_store() {
        let world = make_world(8, 8);
        let mut view = view_at(Position::new(2, 2));
        assert_eq!(
            assemble(&world, &view).previous_outcome,
            [0.0; OUTCOME_CHANNEL_COUNT]
        );
        view.previous_outcome = [view.energy_scale / 4.0, 1.0, 0.0, 1.0];
        assert_eq!(
            assemble(&world, &view).previous_outcome,
            [0.25, 1.0, 0.0, 1.0]
        );
    }

    /// The energy and outcome reads divide by the view's scale: the same 32
    /// reads 0.16 at a scale of 200 and 0.64 at a scale of 50.
    #[test]
    fn the_outcome_reads_divide_by_the_viewed_energy_scale() {
        let world = make_world(8, 8);
        for scale in [50.0, 200.0, 800.0] {
            let view = BodyView {
                energy_scale: scale,
                previous_outcome: [32.0, 0.0, -32.0, 0.0],
                ..view_at(Position::new(2, 2))
            };
            let si = assemble(&world, &view);
            assert_eq!(si.energy_scale, scale);
            assert_eq!(si.previous_outcome[0], (32.0 / scale).min(1.0));
            assert_eq!(si.previous_outcome[2], (-32.0 / scale).max(-1.0));
        }
    }
}

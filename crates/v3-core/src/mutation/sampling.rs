use rand::Rng;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::OUTPUT_SLOT_COUNT;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, SensorContext, StaticIntrospectionKey, WorldInputKey,
};

/// A food-typed world family: its food type is a second draw when the
/// world has more than one food type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FoodFamily {
    FoodHere,
    NeighborFoodRing,
    AreaFoodSummary,
}

impl FoodFamily {
    const fn key(self, type_idx: OrdinaryFoodTypeId) -> WorldInputKey {
        match self {
            Self::FoodHere => WorldInputKey::food_here(type_idx),
            Self::NeighborFoodRing => WorldInputKey::neighbor_food_ring(type_idx),
            Self::AreaFoodSummary => WorldInputKey::area_food_summary(type_idx),
        }
    }
}

/// One draw bucket of the input-reference sampler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Bucket {
    /// A food-typed world key, one reference per food type.
    Food(FoodFamily),
    /// One reference, no further draw.
    Fixed(InputReference),
    /// `UpstreamSlot`, its slot a second draw over `0..OUTPUT_SLOT_COUNT`.
    Upstream,
}

const fn world(key: WorldInputKey) -> Bucket {
    Bucket::Fixed(InputReference::World(key))
}
const fn own(key: StaticIntrospectionKey) -> Bucket {
    Bucket::Fixed(InputReference::StaticIntrospection(key))
}
const fn dynamic(key: DynamicIntrospectionKey) -> Bucket {
    Bucket::Fixed(InputReference::DynamicIntrospection(key))
}

/// Number of input buckets: the sampler's key draw range.
const BUCKET_COUNT: u8 = 30;

/// The one declaration of the input-reference draw (T27.F01): the sampler
/// is one `gen_range(0..30)` into this table, and
/// `input_ref::input_reference_universe` is the table expanded. A
/// reference's bucket count is its draw weight: `NearbyCreature` keeps the
/// three buckets its three predecessors held (T25.F05), `UpstreamSlot`
/// seven, whatever its slot count, and `UpstreamBus` one (T25.F11).
pub(crate) static INPUT_BUCKETS: [Bucket; BUCKET_COUNT as usize] = [
    Bucket::Food(FoodFamily::FoodHere),
    Bucket::Food(FoodFamily::NeighborFoodRing),
    world(WorldInputKey::NeighborBarrierRing),
    world(WorldInputKey::NeighborOccupiedRing),
    own(StaticIntrospectionKey::AgeTicks),
    own(StaticIntrospectionKey::OwnRed),
    own(StaticIntrospectionKey::OwnGreen),
    own(StaticIntrospectionKey::OwnBlue),
    dynamic(DynamicIntrospectionKey::EnergyCurrent),
    dynamic(DynamicIntrospectionKey::EnergyConsumedThisTick),
    Bucket::Fixed(InputReference::ActionQueue),
    Bucket::Food(FoodFamily::AreaFoodSummary),
    world(WorldInputKey::AreaBarrierSummary),
    world(WorldInputKey::NearbyCreature),
    world(WorldInputKey::NearbyCreature),
    world(WorldInputKey::NearbyCreature),
    world(WorldInputKey::AreaOccupancySummary),
    Bucket::Fixed(InputReference::ActionVotes),
    Bucket::Fixed(InputReference::PreviousPassVotes),
    Bucket::Fixed(InputReference::CommitCounts),
    dynamic(DynamicIntrospectionKey::HopsThisTick),
    Bucket::Fixed(InputReference::PreviousOutcome),
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Upstream,
    Bucket::Fixed(InputReference::UpstreamBus),
];

/// Every reference [`INPUT_BUCKETS`] can produce with `food_type_count`
/// food types, in table order: a bucket repeating the one before it adds
/// nothing. Repeats are adjacent runs (`NearbyCreature`, `Upstream`); the
/// agreement test's no-duplicate assertion pins that.
pub(crate) fn bucket_references(food_type_count: usize) -> Vec<InputReference> {
    let food_types = capped_food_types(food_type_count);
    let mut references =
        Vec::with_capacity(INPUT_BUCKETS.len() + 3 * food_types + OUTPUT_SLOT_COUNT);
    for (index, bucket) in INPUT_BUCKETS.iter().enumerate() {
        if index > 0 && INPUT_BUCKETS[index - 1] == *bucket {
            continue;
        }
        match bucket {
            Bucket::Food(family) => {
                references.extend((0..food_types).map(|idx| {
                    InputReference::World(family.key(OrdinaryFoodTypeId::new(idx as u16)))
                }))
            }
            Bucket::Fixed(reference) => references.push(reference.clone()),
            Bucket::Upstream => {
                references.extend((0..OUTPUT_SLOT_COUNT).map(InputReference::UpstreamSlot));
            }
        }
    }
    references
}

/// Generate a random input reference under `context`: the food-typed draw
/// over its food types.
pub(crate) fn random_input_reference(
    rng: &mut impl Rng,
    context: &SensorContext,
) -> InputReference {
    random_input_reference_for_food_types(rng, context.food_type_count)
}

/// Generate a random input reference: one key draw into [`INPUT_BUCKETS`],
/// then a food type for a food bucket (with more than one food type) or a
/// slot for an upstream bucket.
///
/// `food_type_count <= 1` soft-defaults to type 0 only.
pub(crate) fn random_input_reference_for_food_types(
    rng: &mut impl Rng,
    food_type_count: usize,
) -> InputReference {
    match &INPUT_BUCKETS[usize::from(rng.gen_range(0..BUCKET_COUNT))] {
        Bucket::Food(family) => {
            InputReference::World(family.key(sample_food_type_id(food_type_count, rng)))
        }
        Bucket::Fixed(reference) => reference.clone(),
        Bucket::Upstream => InputReference::UpstreamSlot(rng.gen_range(0..OUTPUT_SLOT_COUNT)),
    }
}

fn capped_food_types(food_type_count: usize) -> usize {
    food_type_count.clamp(1, usize::from(u16::MAX) + 1)
}

fn sample_food_type_id(food_type_count: usize, rng: &mut impl Rng) -> OrdinaryFoodTypeId {
    let capped = capped_food_types(food_type_count);
    if capped <= 1 {
        return OrdinaryFoodTypeId::default();
    }
    OrdinaryFoodTypeId::new(rng.gen_range(0..capped) as u16)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::mutation::input_ref::input_reference_universe;
    use crate::neighborhood::input_use::catalog::Family;
    use proptest::prelude::*;
    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    /// An RNG that returns `values` in order, repeating the last, and counts
    /// the draws taken from it. `next_u32` truncates, as `StepRng` does.
    struct ScriptedRng {
        values: Vec<u64>,
        draws: usize,
    }

    impl ScriptedRng {
        fn new(values: Vec<u64>) -> Self {
            Self { values, draws: 0 }
        }
    }

    impl RngCore for ScriptedRng {
        fn next_u32(&mut self) -> u32 {
            self.next_u64() as u32
        }

        fn next_u64(&mut self) -> u64 {
            let value = self.values[self.draws.min(self.values.len() - 1)];
            self.draws += 1;
            value
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            for chunk in dest.chunks_mut(8) {
                let bytes = self.next_u64().to_le_bytes();
                chunk.copy_from_slice(&bytes[..chunk.len()]);
            }
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    /// The midpoint of table index `index`'s u32 bucket: selects that
    /// index under rand's multiply-high uniform sampler, with no rejection.
    fn catalog_draw(index: u64) -> u64 {
        ((index << 32) + (1_u64 << 31)) / INPUT_BUCKETS.len() as u64
    }

    /// The midpoint of sub-index `index`'s u64 bucket in a `usize` range of
    /// `len`: selects that index with no rejection, like [`catalog_draw`].
    fn sub_draw(index: usize, len: usize) -> u64 {
        ((((index as u128) << 64) + (1_u128 << 63)) / len as u128) as u64
    }

    /// The reference table index `index` yields and the draws it took.
    fn draw_at(index: u64, food_type_count: usize) -> (InputReference, usize) {
        let mut rng = ScriptedRng::new(vec![catalog_draw(index)]);
        let reference = random_input_reference_for_food_types(&mut rng, food_type_count);
        (reference, rng.draws)
    }

    #[test]
    fn typed_sampler_single_food_mode_does_not_draw_extra_food_type_rng() {
        let mut rng = ScriptedRng::new(vec![0]);
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
        let mut rng = ScriptedRng::new(vec![0]);
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

    /// Every fixed bucket sits at its table index and costs the key draw
    /// alone, at one food type and at three (T27.F01 invariant 7).
    #[test]
    fn typed_sampler_draws_each_fixed_bucket_at_its_index_with_one_draw() {
        use DynamicIntrospectionKey as D;
        use InputReference as R;
        use StaticIntrospectionKey as S;
        let nearby = R::World(WorldInputKey::NearbyCreature);
        for (index, expected) in [
            (2_u64, R::World(WorldInputKey::NeighborBarrierRing)),
            (3, R::World(WorldInputKey::NeighborOccupiedRing)),
            (4, R::StaticIntrospection(S::AgeTicks)),
            (5, R::StaticIntrospection(S::OwnRed)),
            (6, R::StaticIntrospection(S::OwnGreen)),
            (7, R::StaticIntrospection(S::OwnBlue)),
            (8, R::DynamicIntrospection(D::EnergyCurrent)),
            (9, R::DynamicIntrospection(D::EnergyConsumedThisTick)),
            (10, R::ActionQueue),
            (12, R::World(WorldInputKey::AreaBarrierSummary)),
            (13, nearby.clone()),
            (14, nearby.clone()),
            (15, nearby),
            (16, R::World(WorldInputKey::AreaOccupancySummary)),
            (17, R::ActionVotes),
            (18, R::PreviousPassVotes),
            (19, R::CommitCounts),
            (20, R::DynamicIntrospection(D::HopsThisTick)),
            (21, R::PreviousOutcome),
        ] {
            for food_type_count in [1, 3] {
                assert_eq!(
                    draw_at(index, food_type_count),
                    (expected.clone(), 1),
                    "index {index}"
                );
            }
        }
    }

    /// Indices 0, 1 and 11 are the food-typed buckets: one draw at one food
    /// type, a second for the type at three.
    #[test]
    fn typed_sampler_food_buckets_draw_their_type_with_more_than_one_food_type() {
        for (index, key) in [
            (0_u64, "FoodHere"),
            (1, "NeighborFoodRing"),
            (11, "AreaFoodSummary"),
        ] {
            for (food_type_count, draws) in [(1, 1), (3, 2)] {
                let (reference, taken) = draw_at(index, food_type_count);
                let InputReference::World(world) = reference else {
                    panic!("index {index}: {reference:?}");
                };
                assert_eq!(world.as_key(), key);
                assert_eq!(taken, draws, "index {index}");
            }
        }
    }

    /// Indices 22 to 28 are the upstream slot, which draws its slot; 29 is
    /// the upstream bus, one draw (T25.F11 invariant 1).
    #[test]
    fn typed_sampler_upstream_range_is_seven_slots_then_the_bus() {
        for index in 22_u64..29 {
            let (reference, draws) = draw_at(index, 1);
            assert!(
                matches!(reference, InputReference::UpstreamSlot(_)),
                "{index}"
            );
            assert_eq!(draws, 2);
        }
        assert_eq!(draw_at(29, 1), (InputReference::UpstreamBus, 1));
    }

    /// Every reference the sampler can produce at `food_type_count` food
    /// types, by driving each table index and each of its sub-draws: the
    /// script length is the draw count the sampler must take.
    fn every_sampler_output(food_type_count: usize) -> Vec<InputReference> {
        let mut outputs = Vec::new();
        for (index, bucket) in INPUT_BUCKETS.iter().enumerate() {
            let key = catalog_draw(index as u64);
            let scripts: Vec<(Vec<u64>, Option<InputReference>)> = match bucket {
                Bucket::Fixed(_) => vec![(vec![key], None)],
                Bucket::Food(family) if food_type_count > 1 => (0..food_type_count)
                    .map(|t| {
                        let expected =
                            InputReference::World(family.key(OrdinaryFoodTypeId::new(t as u16)));
                        (vec![key, sub_draw(t, food_type_count)], Some(expected))
                    })
                    .collect(),
                Bucket::Food(family) => vec![(
                    vec![key],
                    Some(InputReference::World(
                        family.key(OrdinaryFoodTypeId::default()),
                    )),
                )],
                Bucket::Upstream => (0..OUTPUT_SLOT_COUNT)
                    .map(|slot| {
                        let script = vec![key, sub_draw(slot, OUTPUT_SLOT_COUNT)];
                        (script, Some(InputReference::UpstreamSlot(slot)))
                    })
                    .collect(),
            };
            for (script, expected) in scripts {
                let mut rng = ScriptedRng::new(script.clone());
                let reference = random_input_reference_for_food_types(&mut rng, food_type_count);
                assert_eq!(rng.draws, script.len(), "index {index}: {reference:?}");
                if let Some(expected) = expected {
                    assert_eq!(reference, expected, "index {index}");
                }
                outputs.push(reference);
            }
        }
        outputs
    }

    /// The agreement test (T27.F01 invariant 7): the references the sampler
    /// can draw, enumerated exhaustively, are exactly
    /// `input_reference_universe`, every world key is in it, and
    /// `Family::of` gives every named member its own family.
    #[test]
    fn the_sampler_and_the_universe_agree() {
        for food_type_count in [1_usize, 3] {
            let universe =
                input_reference_universe(&crate::config::test_sensor_context(food_type_count));
            let as_set: BTreeSet<_> = universe.iter().map(|r| format!("{r:?}")).collect();
            assert_eq!(as_set.len(), universe.len(), "no duplicate");

            let drawn: BTreeSet<_> = every_sampler_output(food_type_count)
                .iter()
                .map(|r| format!("{r:?}"))
                .collect();
            assert_eq!(drawn, as_set, "food types {food_type_count}");
            assert!(universe.contains(&InputReference::UpstreamSlot(OUTPUT_SLOT_COUNT - 1)));

            let food = (0..food_type_count).map(|t| OrdinaryFoodTypeId::new(t as u16));
            for key in WorldInputKey::universe(food) {
                assert!(universe.contains(&InputReference::World(key)), "{key:?}");
            }
            let named: BTreeSet<_> = universe
                .iter()
                .filter(|r| !matches!(r, InputReference::UpstreamSlot(_)))
                .map(Family::of)
                .collect();
            assert_eq!(
                named.len(),
                universe.len() - OUTPUT_SLOT_COUNT,
                "one family per named reference"
            );
            if food_type_count == 1 {
                assert_eq!(named.len(), 21, "21 named families (UpstreamBus, T25.F11)");
            }
        }
    }

    proptest! {
        /// Whatever the RNG state, a sampled reference is a universe member.
        #[test]
        fn every_sampled_reference_is_in_the_universe(
            seed in any::<u64>(),
            food_type_count in 0_usize..=4,
        ) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let reference = random_input_reference_for_food_types(&mut rng, food_type_count);
            prop_assert!(
                input_reference_universe(&crate::config::test_sensor_context(food_type_count)).contains(&reference),
                "{reference:?}"
            );
        }
    }

    /// The universe follows the table order: `AgeTicks`, then the three
    /// own-colour keys, then the energy reads.
    #[test]
    fn the_universe_is_the_table_in_order() {
        use StaticIntrospectionKey as S;
        let universe = input_reference_universe(&crate::config::test_sensor_context(1));
        let at = |reference: &InputReference| universe.iter().position(|r| r == reference);
        let order: Vec<_> = [
            InputReference::StaticIntrospection(S::AgeTicks),
            InputReference::StaticIntrospection(S::OwnRed),
            InputReference::StaticIntrospection(S::OwnGreen),
            InputReference::StaticIntrospection(S::OwnBlue),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        ]
        .iter()
        .map(|r| at(r).expect("in universe"))
        .collect();
        assert_eq!(order, [4, 5, 6, 7, 8]);
        assert_eq!(universe.len(), 21 + OUTPUT_SLOT_COUNT);
    }
}

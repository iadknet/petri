use rand::Rng;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::runtime::OUTPUT_SLOT_COUNT;

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
/// three buckets its three predecessors held (T25.F05) and `UpstreamSlot`
/// eight, whatever its slot count. Recruitment's family weights are a
/// separate declaration (`graph::recruitment::universe_entries`).
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
    Bucket::Upstream,
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

/// Generate a random input reference from the one-food-type table.
pub(crate) fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    random_input_reference_for_food_types(rng, 1)
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
    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    /// A `StepRng` that counts the draws taken from it.
    struct CountingStepRng {
        inner: rand::rngs::mock::StepRng,
        draws: usize,
    }

    impl CountingStepRng {
        fn new(value: u64) -> Self {
            Self {
                inner: rand::rngs::mock::StepRng::new(value, 0),
                draws: 0,
            }
        }
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

    /// The midpoint of table index `index`'s u32 bucket: selects that
    /// index under rand's multiply-high uniform sampler, with no rejection.
    fn catalog_draw(index: u64) -> u64 {
        ((index << 32) + (1_u64 << 31)) / INPUT_BUCKETS.len() as u64
    }

    /// The reference table index `index` yields and the draws it took.
    fn draw_at(index: u64, food_type_count: usize) -> (InputReference, usize) {
        let mut rng = CountingStepRng::new(catalog_draw(index));
        let reference = random_input_reference_for_food_types(&mut rng, food_type_count);
        (reference, rng.draws)
    }

    #[test]
    fn typed_sampler_single_food_mode_does_not_draw_extra_food_type_rng() {
        let mut rng = CountingStepRng::new(0);
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
        let mut rng = CountingStepRng::new(0);
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

    /// Indices 22 to 29 are the upstream slot, which draws its slot.
    #[test]
    fn typed_sampler_upstream_range_is_the_last_eight_indices() {
        for index in 22_u64..30 {
            let (reference, draws) = draw_at(index, 1);
            assert!(
                matches!(reference, InputReference::UpstreamSlot(_)),
                "{index}"
            );
            assert_eq!(draws, 2);
        }
    }

    /// The agreement test (T27.F01 invariant 7): the references the sampler
    /// draws are exactly `input_reference_universe`, every world key is in
    /// it, and `Family::of` gives every named member its own family.
    #[test]
    fn the_sampler_and_the_universe_agree() {
        for food_type_count in [1_usize, 3] {
            let universe = input_reference_universe(food_type_count);
            let as_set: BTreeSet<_> = universe.iter().map(|r| format!("{r:?}")).collect();
            assert_eq!(as_set.len(), universe.len(), "no duplicate");

            let mut rng = SmallRng::seed_from_u64(27_001);
            let drawn: BTreeSet<_> = (0..40_000)
                .map(|_| {
                    format!(
                        "{:?}",
                        random_input_reference_for_food_types(&mut rng, food_type_count)
                    )
                })
                .collect();
            assert_eq!(drawn, as_set, "food types {food_type_count}");
            assert!(universe.contains(&InputReference::UpstreamSlot(23)));

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
                assert_eq!(named.len(), 20, "20 named families");
            }
        }
    }

    /// The universe follows the table order: `AgeTicks`, then the three
    /// own-colour keys, then the energy reads.
    #[test]
    fn the_universe_is_the_table_in_order() {
        use StaticIntrospectionKey as S;
        let universe = input_reference_universe(1);
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
        assert_eq!(universe.len(), 20 + OUTPUT_SLOT_COUNT);
    }
}

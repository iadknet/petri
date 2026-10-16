use crate::contracts::OrdinaryFoodTypeId;

// ── Compound sub-field index constants ──────────────────────────────────────
// Per v3-sensor-spec.md Section 5. Prevents magic-number indexing in assembly.

/// `AreaFoodSummary` sub-field indices (width 7).
pub mod food_idx {
    pub const TOTAL_RATIO: usize = 0;
    pub const GRADIENT_X: usize = 1;
    pub const GRADIENT_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const MAX_VALUE: usize = 6;
}

/// `AreaBarrierSummary` sub-field indices (width 7).
pub mod barrier_idx {
    pub const DENSITY_RATIO: usize = 0;
    pub const BLOCKED_ADJACENT_RATIO: usize = 1;
    pub const GRADIENT_X: usize = 2;
    pub const GRADIENT_Y: usize = 3;
    pub const NEAREST_DX: usize = 4;
    pub const NEAREST_DY: usize = 5;
    pub const NEAREST_DIST: usize = 6;
}

/// `AreaOccupancySummary` sub-field indices (width 7).
pub mod occupancy_idx {
    pub const COUNT_RATIO: usize = 0;
    pub const CENTER_X: usize = 1;
    pub const CENTER_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const CROWDING_RATIO: usize = 6;
}

/// `NearbyCreatureCore` per-slot field count and offsets.
pub mod core_idx {
    pub const FIELDS_PER_SLOT: usize = 4;
    pub const PRESENT: usize = 0;
    pub const REL_X: usize = 1;
    pub const REL_Y: usize = 2;
    pub const DIST: usize = 3;
}

/// `NearbyCreatureVitals` per-slot field count and offsets.
pub mod vitals_idx {
    pub const FIELDS_PER_SLOT: usize = 2;
    pub const ENERGY_RATIO: usize = 0;
    pub const REPRODUCE_READY: usize = 1;
}

/// `NearbyCreatureIdentity` per-slot field count and offsets.
pub mod identity_idx {
    pub const FIELDS_PER_SLOT: usize = 3;
    pub const KIN_AFFINITY: usize = 0;
    pub const LINEAGE_MATCH: usize = 1;
    pub const PHENOTYPE_SIMILARITY: usize = 2;
}

/// One `(x, y)` offset a world family senses toward a thing, and the thing's
/// plain facts (amount, distance, presence): what a `DirectionPair` edge
/// group reads and recruitment's pair case wires (T25.F04). Field indices of
/// the family's compound layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DirectionPair {
    /// The `x` offset field (right positive).
    pub x: u16,
    /// The `y` offset field (down positive).
    pub y: u16,
    /// The same thing's plain facts, in wiring order.
    pub facts: &'static [u16],
}

// Field indices are `usize` for array indexing; every one is below 16.
const fn pair(x: usize, y: usize, facts: &'static [u16]) -> DirectionPair {
    DirectionPair {
        x: x as u16,
        y: y as u16,
        facts,
    }
}

const FOOD_FACTS: &[u16] = &[
    food_idx::TOTAL_RATIO as u16,
    food_idx::NEAREST_DIST as u16,
    food_idx::MAX_VALUE as u16,
];
const FOOD_PAIRS: &[DirectionPair] = &[
    pair(food_idx::GRADIENT_X, food_idx::GRADIENT_Y, FOOD_FACTS),
    pair(food_idx::NEAREST_DX, food_idx::NEAREST_DY, FOOD_FACTS),
];
const BARRIER_FACTS: &[u16] = &[
    barrier_idx::DENSITY_RATIO as u16,
    barrier_idx::BLOCKED_ADJACENT_RATIO as u16,
    barrier_idx::NEAREST_DIST as u16,
];
const BARRIER_PAIRS: &[DirectionPair] = &[
    pair(
        barrier_idx::GRADIENT_X,
        barrier_idx::GRADIENT_Y,
        BARRIER_FACTS,
    ),
    pair(
        barrier_idx::NEAREST_DX,
        barrier_idx::NEAREST_DY,
        BARRIER_FACTS,
    ),
];
const OCCUPANCY_FACTS: &[u16] = &[
    occupancy_idx::COUNT_RATIO as u16,
    occupancy_idx::NEAREST_DIST as u16,
    occupancy_idx::CROWDING_RATIO as u16,
];
const OCCUPANCY_PAIRS: &[DirectionPair] = &[
    pair(
        occupancy_idx::CENTER_X,
        occupancy_idx::CENTER_Y,
        OCCUPANCY_FACTS,
    ),
    pair(
        occupancy_idx::NEAREST_DX,
        occupancy_idx::NEAREST_DY,
        OCCUPANCY_FACTS,
    ),
];

/// Slot `k` of `NearbyCreatureCore`'s fields at `offset`.
const fn core_field(k: usize, offset: usize) -> u16 {
    (k * core_idx::FIELDS_PER_SLOT + offset) as u16
}
const fn core_facts(k: usize) -> [u16; 2] {
    [
        core_field(k, core_idx::PRESENT),
        core_field(k, core_idx::DIST),
    ]
}
const CORE_FACTS: [[u16; 2]; 4] = [core_facts(0), core_facts(1), core_facts(2), core_facts(3)];
/// Slot `k`: `(rel_x, rel_y)`, facts presence and distance.
const fn core_slot(k: usize) -> DirectionPair {
    DirectionPair {
        x: core_field(k, core_idx::REL_X),
        y: core_field(k, core_idx::REL_Y),
        facts: &CORE_FACTS[k],
    }
}
const CORE_PAIRS: &[DirectionPair] = &[core_slot(0), core_slot(1), core_slot(2), core_slot(3)];

/// Identifies a world-state spatial sensor input.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum WorldInputKey {
    /// Food density on current cell, normalized to [0.0, 1.0].
    FoodHere { type_idx: OrdinaryFoodTypeId },
    /// Compound: neighbor food ring (8 sub-values, indexed by Direction::to_index()).
    NeighborFoodRing { type_idx: OrdinaryFoodTypeId },
    /// Compound: neighbor barrier ring (8 sub-values, indexed by Direction::to_index()).
    NeighborBarrierRing,
    /// Compound: neighbor occupied ring (8 sub-values, indexed by Direction::to_index()).
    NeighborOccupiedRing,
    /// Compound: area food summary (7 sub-values). See v3-sensor-spec.md Section 5.1.
    AreaFoodSummary { type_idx: OrdinaryFoodTypeId },
    /// Compound: area barrier summary (7 sub-values). See v3-sensor-spec.md Section 5.2.
    AreaBarrierSummary,
    /// Compound: area occupancy summary (7 sub-values). See v3-sensor-spec.md Section 5.3.
    AreaOccupancySummary,
    /// Compound: nearby creature core (4 slots × 4 fields = 16 sub-values). See v3-sensor-spec.md Section 5.4.
    NearbyCreatureCore,
    /// Compound: nearby creature vitals (4 slots × 2 fields = 8 sub-values). See v3-sensor-spec.md Section 5.5.
    NearbyCreatureVitals,
    /// Compound: nearby creature identity (4 slots × 3 fields = 12 sub-values). See v3-sensor-spec.md Section 5.6.
    NearbyCreatureIdentity,
}

impl WorldInputKey {
    /// Construct a typed food-here key for the given food type.
    #[must_use]
    pub const fn food_here(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::FoodHere { type_idx }
    }

    /// Construct a typed neighbor-food-ring key for the given food type.
    #[must_use]
    pub const fn neighbor_food_ring(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::NeighborFoodRing { type_idx }
    }

    /// Construct a typed area-food-summary key for the given food type.
    #[must_use]
    pub const fn area_food_summary(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::AreaFoodSummary { type_idx }
    }

    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::FoodHere { .. } => "FoodHere",
            Self::NeighborFoodRing { .. } => "NeighborFoodRing",
            Self::NeighborBarrierRing => "NeighborBarrierRing",
            Self::NeighborOccupiedRing => "NeighborOccupiedRing",
            Self::AreaFoodSummary { .. } => "AreaFoodSummary",
            Self::AreaBarrierSummary => "AreaBarrierSummary",
            Self::AreaOccupancySummary => "AreaOccupancySummary",
            Self::NearbyCreatureCore => "NearbyCreatureCore",
            Self::NearbyCreatureVitals => "NearbyCreatureVitals",
            Self::NearbyCreatureIdentity => "NearbyCreatureIdentity",
        }
    }

    /// Whether this is one of the three neighbor rings: width 8, sub-value `c`
    /// the neighbor in `Direction::ALL[c]` (the shape `Ring` edge groups read).
    #[must_use]
    pub const fn is_neighbor_ring(self) -> bool {
        matches!(
            self,
            Self::NeighborFoodRing { .. } | Self::NeighborBarrierRing | Self::NeighborOccupiedRing
        )
    }

    /// The `(x, y)` offset pairs this family declares (T25.F04): two for each
    /// area summary, one per ranked slot of `NearbyCreatureCore`, none for
    /// every other key. Depends on the variant only, never on a food type.
    #[must_use]
    pub const fn direction_pairs(self) -> &'static [DirectionPair] {
        match self {
            Self::AreaFoodSummary { .. } => FOOD_PAIRS,
            Self::AreaBarrierSummary => BARRIER_PAIRS,
            Self::AreaOccupancySummary => OCCUPANCY_PAIRS,
            Self::NearbyCreatureCore => CORE_PAIRS,
            _ => &[],
        }
    }

    /// Returns the typed food index for food-specific world keys, or `None` otherwise.
    #[must_use]
    pub const fn food_type_idx(self) -> Option<OrdinaryFoodTypeId> {
        match self {
            Self::FoodHere { type_idx }
            | Self::NeighborFoodRing { type_idx }
            | Self::AreaFoodSummary { type_idx } => Some(type_idx),
            _ => None,
        }
    }

    /// Number of sub-values for compound access. Returns 1 for scalar keys.
    ///
    /// `opt-inline-small`: Hot path — called per instruction per tick.
    /// `api-must-use`: Pure getter.
    #[inline]
    #[must_use]
    pub fn compound_width(&self) -> u16 {
        match self {
            Self::AreaFoodSummary { .. }
            | Self::AreaBarrierSummary
            | Self::AreaOccupancySummary => 7,
            Self::NeighborFoodRing { .. }
            | Self::NeighborBarrierRing
            | Self::NeighborOccupiedRing => 8,
            Self::NearbyCreatureCore => 16,
            Self::NearbyCreatureVitals => 8,
            Self::NearbyCreatureIdentity => 12,
            _ => 1,
        }
    }
}

/// Static introspection values assembled at tick start (snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StaticIntrospectionKey {
    /// Age as a saturating fraction of `age_reference_ticks`, in [0, 1].
    AgeTicks,
}

/// Dynamic introspection values resolved live at read time (not snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicIntrospectionKey {
    /// Current energy level.
    EnergyCurrent,
    /// Energy spent since this tick's mesh run began (earlier dispatches and
    /// hop ramp charges), as a fraction of max energy. A VM read also counts
    /// its own in-flight step costs; a graph node reads the pre-dispatch value.
    EnergyConsumedThisTick,
    /// Mesh hops dispatched this tick, the one in flight included; a raw
    /// count (T19.F05).
    HopsThisTick,
}

/// A reference to a specific input source for a node input slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InputReference {
    /// World state spatial sensor input.
    World(WorldInputKey),
    /// Static introspection value (snapshot at tick start).
    StaticIntrospection(StaticIntrospectionKey),
    /// Dynamic introspection value (resolved live).
    DynamicIntrospection(DynamicIntrospectionKey),
    /// Output slot from the upstream node in the mesh chain.
    UpstreamSlot(usize),
    /// Compound: the creature's action queue. sub_idx maps to queue slot values:
    /// `sub_idx / 3` = queue slot index, `sub_idx % 3`: 0 = action_type, 1 = param0, 2 = param1.
    ActionQueue,
    /// Compound (T19.F05): the current pass's vote vector, one sub-value per
    /// `VoteSink` index, resolved live.
    ActionVotes,
    /// Compound (T19.F05): the vote vector at the previous pass's end; zeros
    /// in the tick's first pass.
    PreviousPassVotes,
    /// Compound (T19.F05): the per-kind bars, one raw commit count per
    /// `VoteKind` index, resolved live.
    CommitCounts,
    /// Compound (T19.F05): the previous tick's outcome channels, one per
    /// `OutcomeChannel` discriminant, frozen at tick start.
    PreviousOutcome,
}

impl InputReference {
    /// Whether this reference is a neighbor ring a `Ring` edge group can read
    /// (T25.F03): `World` of one of [`WorldInputKey::is_neighbor_ring`].
    #[must_use]
    pub const fn is_ring(&self) -> bool {
        matches!(self, Self::World(key) if key.is_neighbor_ring())
    }

    /// The `(x, y)` pairs this reference declares (T25.F04): its world
    /// key's [`WorldInputKey::direction_pairs`], none for a non-world
    /// reference.
    #[must_use]
    pub const fn direction_pairs(&self) -> &'static [DirectionPair] {
        match self {
            Self::World(key) => key.direction_pairs(),
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::OrdinaryFoodTypeId;

    fn every_world_key() -> Vec<WorldInputKey> {
        let food = (0..3).map(OrdinaryFoodTypeId::new);
        crate::creature::sensor_census::world_input_key_universe(food)
            .into_iter()
            .collect()
    }

    #[test]
    fn every_declared_pair_has_distinct_fields_below_the_width() {
        for key in every_world_key() {
            for pair in key.direction_pairs() {
                let mut fields = vec![pair.x, pair.y];
                fields.extend_from_slice(pair.facts);
                assert!(fields.iter().all(|&f| f < key.compound_width()), "{key:?}");
                let mut sorted = fields.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(sorted.len(), fields.len(), "{key:?}: {pair:?}");
            }
        }
    }

    #[test]
    fn the_four_layouts_declare_their_pairs_and_facts() {
        let food = WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(2));
        let layouts = [
            (food, vec![(1, 2, vec![0, 5, 6]), (3, 4, vec![0, 5, 6])]),
            (
                WorldInputKey::AreaBarrierSummary,
                vec![(2, 3, vec![0, 1, 6]), (4, 5, vec![0, 1, 6])],
            ),
            (
                WorldInputKey::AreaOccupancySummary,
                vec![(1, 2, vec![0, 5, 6]), (3, 4, vec![0, 5, 6])],
            ),
            (
                WorldInputKey::NearbyCreatureCore,
                (0..4u16)
                    .map(|k| (4 * k + 1, 4 * k + 2, vec![4 * k, 4 * k + 3]))
                    .collect(),
            ),
        ];
        for (key, want) in layouts {
            let got: Vec<_> = key
                .direction_pairs()
                .iter()
                .map(|pair| (pair.x, pair.y, pair.facts.to_vec()))
                .collect();
            assert_eq!(got, want, "{key:?}");
        }
        let declaring = every_world_key()
            .into_iter()
            .filter(|key| !key.direction_pairs().is_empty())
            .map(WorldInputKey::as_key)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            declaring,
            [
                "AreaBarrierSummary",
                "AreaFoodSummary",
                "AreaOccupancySummary",
                "NearbyCreatureCore"
            ]
            .into()
        );
        assert!(InputReference::ActionVotes.direction_pairs().is_empty());
        assert_eq!(
            InputReference::World(food).direction_pairs(),
            food.direction_pairs()
        );
    }

    #[test]
    fn the_moved_index_modules_match_the_compound_widths() {
        let area = [
            food_idx::MAX_VALUE,
            barrier_idx::NEAREST_DIST,
            occupancy_idx::CROWDING_RATIO,
        ];
        assert!(area.iter().all(|&last| last + 1 == 7));
        assert_eq!(
            core_idx::FIELDS_PER_SLOT * 4,
            usize::from(WorldInputKey::NearbyCreatureCore.compound_width())
        );
        assert_eq!(
            vitals_idx::FIELDS_PER_SLOT * 4,
            usize::from(WorldInputKey::NearbyCreatureVitals.compound_width())
        );
        assert_eq!(
            identity_idx::FIELDS_PER_SLOT * 4,
            usize::from(WorldInputKey::NearbyCreatureIdentity.compound_width())
        );
        assert_eq!(
            crate::sensors::perception::food_idx::NEAREST_DY,
            food_idx::NEAREST_DY
        );
    }

    #[test]
    fn every_swap_alternative_keeps_the_pairs() {
        for food_types in 1..=3 {
            for key in every_world_key() {
                let reference = InputReference::World(key);
                for alternative in
                    crate::mutation::input_ref::swap_alternatives(&reference, food_types)
                {
                    assert_eq!(
                        alternative.direction_pairs(),
                        reference.direction_pairs(),
                        "{reference:?} -> {alternative:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_frontend_fixture_equals_the_declared_pairs() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../frontend/src/components/inspector/directionPairs.json"
        ))
        .expect("fixture parses");
        let mut declared = serde_json::Map::new();
        for key in every_world_key() {
            if !key.direction_pairs().is_empty() {
                declared.insert(
                    key.as_key().to_owned(),
                    serde_json::to_value(key.direction_pairs()).expect("serializes"),
                );
            }
        }
        assert_eq!(fixture, serde_json::Value::Object(declared));
    }

    #[test]
    fn world_input_keys_constructible() {
        let _food = WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let _nfood = WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let _barrier = WorldInputKey::NeighborBarrierRing;
        let _occ = WorldInputKey::NeighborOccupiedRing;
    }

    #[test]
    fn input_reference_upstream_slot() {
        let r = InputReference::UpstreamSlot(3);
        if let InputReference::UpstreamSlot(idx) = r {
            assert_eq!(idx, 3);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn input_reference_serde_roundtrip() {
        let refs = vec![
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::AreaBarrierSummary),
            InputReference::World(WorldInputKey::AreaOccupancySummary),
            InputReference::World(WorldInputKey::NearbyCreatureCore),
            InputReference::World(WorldInputKey::NearbyCreatureVitals),
            InputReference::World(WorldInputKey::NearbyCreatureIdentity),
            InputReference::World(WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
            InputReference::UpstreamSlot(0),
            InputReference::ActionQueue,
            InputReference::ActionVotes,
            InputReference::PreviousPassVotes,
            InputReference::CommitCounts,
            InputReference::PreviousOutcome,
        ];
        for r in refs {
            let json = serde_json::to_string(&r).unwrap();
            let r2: InputReference = serde_json::from_str(&json).unwrap();
            assert_eq!(r, r2);
        }
    }

    #[test]
    fn ring_sensor_compound_widths() {
        assert_eq!(
            WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width(),
            8
        );
        assert_eq!(WorldInputKey::NeighborBarrierRing.compound_width(), 8);
        assert_eq!(WorldInputKey::NeighborOccupiedRing.compound_width(), 8);
        assert_eq!(
            WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width(),
            1
        );
    }

    #[test]
    fn typed_food_keys_have_stable_keys_and_widths() {
        let here = WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let ring = WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let area = WorldInputKey::AreaFoodSummary {
            type_idx: OrdinaryFoodTypeId::default(),
        };

        assert_eq!(here.as_key(), "FoodHere");
        assert_eq!(ring.as_key(), "NeighborFoodRing");
        assert_eq!(area.as_key(), "AreaFoodSummary");
        assert_eq!(here.compound_width(), 1);
        assert_eq!(ring.compound_width(), 8);
        assert_eq!(area.compound_width(), 7);
    }
}

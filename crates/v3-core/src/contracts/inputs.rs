use crate::config::OrdinaryFoodTypeId;

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
    /// Number of generations from the founder.
    Generation,
    /// Age in ticks.
    AgeTicks,
}

/// Dynamic introspection values resolved live at read time (not snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicIntrospectionKey {
    /// Current energy level.
    EnergyCurrent,
    /// Total energy consumed by Eat actions this tick so far.
    EnergyConsumedThisTick,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;

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
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::UpstreamSlot(0),
            InputReference::ActionQueue,
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

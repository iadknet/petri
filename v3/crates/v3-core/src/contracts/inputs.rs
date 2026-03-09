/// Identifies a world-state spatial sensor input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorldInputKey {
    /// Food density on current cell, normalized to [0.0, 1.0].
    FoodHere,
    /// Compound: neighbor food ring (8 sub-values, indexed by Direction::to_index()).
    NeighborFoodRing,
    /// Compound: neighbor barrier ring (8 sub-values, indexed by Direction::to_index()).
    NeighborBarrierRing,
    /// Compound: neighbor occupied ring (8 sub-values, indexed by Direction::to_index()).
    NeighborOccupiedRing,
    /// Compound: area food summary (7 sub-values). See v3-sensor-spec.md Section 5.1.
    AreaFoodSummary,
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
    /// Number of sub-values for compound access. Returns 1 for scalar keys.
    ///
    /// [`opt-inline-small`] Hot path — called per instruction per tick.
    /// [`api-must-use`] Pure getter.
    #[inline]
    #[must_use]
    pub fn compound_width(&self) -> u16 {
        match self {
            Self::AreaFoodSummary | Self::AreaBarrierSummary | Self::AreaOccupancySummary => 7,
            Self::NeighborFoodRing | Self::NeighborBarrierRing | Self::NeighborOccupiedRing => 8,
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

    #[test]
    fn world_input_keys_constructible() {
        let _food = WorldInputKey::FoodHere;
        let _nfood = WorldInputKey::NeighborFoodRing;
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
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::World(WorldInputKey::AreaFoodSummary),
            InputReference::World(WorldInputKey::AreaBarrierSummary),
            InputReference::World(WorldInputKey::AreaOccupancySummary),
            InputReference::World(WorldInputKey::NearbyCreatureCore),
            InputReference::World(WorldInputKey::NearbyCreatureVitals),
            InputReference::World(WorldInputKey::NearbyCreatureIdentity),
            InputReference::World(WorldInputKey::NeighborFoodRing),
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
        assert_eq!(WorldInputKey::NeighborFoodRing.compound_width(), 8);
        assert_eq!(WorldInputKey::NeighborBarrierRing.compound_width(), 8);
        assert_eq!(WorldInputKey::NeighborOccupiedRing.compound_width(), 8);
        assert_eq!(WorldInputKey::FoodHere.compound_width(), 1);
    }
}

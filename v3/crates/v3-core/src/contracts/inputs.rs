use crate::contracts::Direction;

/// Identifies a world-state spatial sensor input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorldInputKey {
    /// Food density on current cell, normalized to [0.0, 1.0].
    FoodHere,
    /// Food density on neighbor cell in given direction, normalized to [0.0, 1.0].
    NeighborCellFood(Direction),
    /// Whether neighbor cell has a barrier: 1.0 = barrier, 0.0 = clear.
    NeighborCellBarrier(Direction),
    /// Whether neighbor cell is occupied by another creature: 1.0 = occupied, 0.0 = empty.
    NeighborCellOccupied(Direction),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_input_keys_constructible() {
        let _food = WorldInputKey::FoodHere;
        let _nfood = WorldInputKey::NeighborCellFood(Direction::N);
        let _barrier = WorldInputKey::NeighborCellBarrier(Direction::SE);
        let _occ = WorldInputKey::NeighborCellOccupied(Direction::W);
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
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::UpstreamSlot(0),
        ];
        for r in refs {
            let json = serde_json::to_string(&r).unwrap();
            let r2: InputReference = serde_json::from_str(&json).unwrap();
            assert_eq!(r, r2);
        }
    }

    #[test]
    fn all_eight_directions_covered_for_neighbor_inputs() {
        for dir in Direction::ALL {
            let _food = InputReference::World(WorldInputKey::NeighborCellFood(dir));
            let _barrier = InputReference::World(WorldInputKey::NeighborCellBarrier(dir));
            let _occ = InputReference::World(WorldInputKey::NeighborCellOccupied(dir));
        }
    }
}

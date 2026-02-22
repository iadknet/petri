use slotmap::new_key_type;

new_key_type! {
    /// Opaque identifier for a living creature in the simulation.
    pub struct CreatureId;
}

/// Opaque identifier for a genome node.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    #[test]
    fn creature_id_slotmap_insert_remove() {
        let mut map: SlotMap<CreatureId, u32> = SlotMap::with_key();
        let id = map.insert(42);
        assert_eq!(map[id], 42);
        map.remove(id);
        assert!(!map.contains_key(id));
    }

    #[test]
    fn node_id_ordering() {
        assert!(NodeId(0) < NodeId(1));
        assert_eq!(NodeId(5), NodeId(5));
    }

    #[test]
    fn node_id_serde_roundtrip() {
        let id = NodeId(7);
        let json = serde_json::to_string(&id).unwrap();
        let id2: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, id2);
    }
}

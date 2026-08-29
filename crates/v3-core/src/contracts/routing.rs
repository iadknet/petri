use super::NodeId;

/// Maximum distinct gate slots per node.
pub const MAX_GATE_SLOTS: usize = 8;

/// A single routing target with stable identity and evolvable bias.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteTarget {
    pub target_id: NodeId,
    /// Stable slot identifier (0..MAX_GATE_SLOTS).
    pub slot: u8,
    /// Genome-level evolvable bias. Clamped to [-4.0, 4.0].
    pub gate_bias: f32,
}

// Prevent accidental size regression — RouteTarget is stored in Vec per node.
const _: () = assert!(std::mem::size_of::<RouteTarget>() <= 16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_target_serde_roundtrip() {
        let rt = RouteTarget {
            target_id: NodeId::new(42),
            slot: 3,
            gate_bias: -1.5,
        };
        let json = serde_json::to_string(&rt).unwrap();
        let back: RouteTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, back);
    }
}

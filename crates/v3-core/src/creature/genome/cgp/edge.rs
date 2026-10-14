//! Edge addressing and the weighted edge (T25.F01).
//!
//! A [`GraphEdge`] is built only through [`GraphEdge::new`] and read through
//! its getters; its fields are writable only inside the `cgp` module, where
//! the container keeps the birth payload aligned with every write.

/// Tagged source for edge addressing. Replaces bare `source_idx: u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GraphSource {
    /// Read from `input_refs[ref_idx]` with sub-value index `sub_idx`.
    InputLeaf { ref_idx: u16, sub_idx: u16 },
    /// Read shared memory slot (current or previous tick).
    SharedMemory { slot: u8, previous: bool },
    /// Read output of `compute_nodes[idx]`.
    ComputeNode(u16),
}

/// Weighted edge in the graph.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub(super) source: GraphSource,
    pub(super) weight: f32,
}

// The edge keeps the layout it had before T25.F01: a 6-byte source and a
// 4-byte weight, 12 bytes at 4-byte alignment.
const _: () = assert!(std::mem::size_of::<GraphEdge>() == 12);

impl GraphEdge {
    #[inline]
    #[must_use]
    pub const fn new(source: GraphSource, weight: f32) -> Self {
        Self { source, weight }
    }

    /// Where the edge reads from.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> GraphSource {
        self.source
    }

    /// The edge's genomic weight.
    #[inline]
    #[must_use]
    pub const fn weight(&self) -> f32 {
        self.weight
    }
}

//! Edge addressing and the weighted edge (T25.F01, T25.F03).
//!
//! A [`GraphEdge`] is built only through [`GraphEdge::new`] (a static,
//! ungrouped edge) or by the container's group primitives, and read through
//! its getters; its fields are writable only inside the `cgp` module, where
//! the container keeps the birth payload and the group invariants aligned
//! with every write.

use super::group::GroupId;

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

/// A sink edge's membership in an edge group (T25.F03): the group it
/// belongs to. Only sink edges carry one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Membership {
    pub group: GroupId,
}

/// Weighted edge in the graph. Inside a group, `own_weight` is a correction
/// on top of the group term, born 0; outside one it is the genomic weight.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    pub(super) source: GraphSource,
    pub(super) own_weight: f32,
    /// Omitted from JSON when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) membership: Option<Membership>,
}

// A 6-byte source, a 4-byte weight and a 4-byte membership, 16 bytes at
// 4-byte alignment (T25.F03).
const _: () = assert!(std::mem::size_of::<GraphEdge>() == 16);

impl GraphEdge {
    /// A static, ungrouped edge.
    #[inline]
    #[must_use]
    pub const fn new(source: GraphSource, own_weight: f32) -> Self {
        Self {
            source,
            own_weight,
            membership: None,
        }
    }

    /// Where the edge reads from.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> GraphSource {
        self.source
    }

    /// The edge's own weight: its genomic weight when ungrouped, its
    /// correction on top of the group term when a member.
    #[inline]
    #[must_use]
    pub const fn own_weight(&self) -> f32 {
        self.own_weight
    }

    /// The group this edge belongs to, if any.
    #[inline]
    #[must_use]
    pub const fn membership(&self) -> Option<Membership> {
        self.membership
    }
}

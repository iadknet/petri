//! Edge addressing and the weighted edge (T25.F01, T25.F03).
//!
//! A [`GraphEdge`] is built only through [`GraphEdge::new`] (a static,
//! ungrouped edge) or by the container's group primitives, and read through
//! its getters; its fields are writable only inside the `cgp` module, where
//! the container keeps the birth payload and the group invariants aligned
//! with every write.

use super::group::GroupId;
use crate::creature::genome::PlasticityConfig;

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

/// How a plastic member learns (T25.F07): for itself (`Edge`, its own
/// correction) or for its group (`Group`, the group's one shared scale).
/// Every member carries one; a static member's scope is inert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Scope {
    Edge,
    Group,
}

impl Scope {
    /// The other scope.
    #[inline]
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Edge => Self::Group,
            Self::Group => Self::Edge,
        }
    }
}

/// A sink edge's membership in an edge group (T25.F03): the group it
/// belongs to and its learning scope (T25.F07). Only sink edges carry one.
/// A group is identified by `group` alone: scope never splits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Membership {
    pub group: GroupId,
    pub scope: Scope,
}

/// Weighted edge in the graph. Inside a group, `own_weight` is a correction
/// on top of the group term, born 0; outside one it is the genomic weight.
/// `plasticity` is the edge's learning attributes (T25.F06): `None` for a
/// static edge.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    pub(super) source: GraphSource,
    pub(super) own_weight: f32,
    /// Omitted from JSON when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) plasticity: Option<PlasticityConfig>,
    /// Omitted from JSON when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) membership: Option<Membership>,
}

// A 6-byte source, a 4-byte weight, a 20-byte plasticity and a 4-byte
// membership (a 2-byte group and a 1-byte scope), 36 bytes at 4-byte
// alignment (T25.F03, T25.F06, T25.F07).
const _: () = assert!(std::mem::size_of::<GraphEdge>() == 36);

impl GraphEdge {
    /// A static, ungrouped edge.
    #[inline]
    #[must_use]
    pub const fn new(source: GraphSource, own_weight: f32) -> Self {
        Self {
            source,
            own_weight,
            plasticity: None,
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

    /// The edge's learning attributes, `None` for a static edge.
    #[inline]
    #[must_use]
    pub const fn plasticity(&self) -> Option<PlasticityConfig> {
        self.plasticity
    }

    /// The group this edge belongs to, if any.
    #[inline]
    #[must_use]
    pub const fn membership(&self) -> Option<Membership> {
        self.membership
    }

    /// Whether this edge is a member of `group`, whatever its scope.
    #[inline]
    #[must_use]
    pub fn is_member_of(&self, group: GroupId) -> bool {
        self.membership
            .is_some_and(|membership| membership.group == group)
    }
}

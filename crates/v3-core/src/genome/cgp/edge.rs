//! Edge addressing and the weighted edge (T25.F01, T25.F03, T25.F12).
//!
//! A [`GraphEdge`] is built only through [`GraphEdge::new`] (a static,
//! lone edge) or by the container's bundle primitives, and read through its
//! getters; its fields are writable only inside the `cgp` module, where the
//! container keeps the birth payload and the bundle invariants aligned with
//! every write.

use super::bundle::BundleId;
use crate::genome::learning::{Learning, MemberPolicy};

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

/// A sink edge's membership in an edge bundle (T25.F03, T25.F12): the
/// bundle it belongs to, how its modifier learns (`policy`) and whether its
/// activity feeds the bundle's scale (`credits`). Only sink edges carry one.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Membership {
    pub bundle: BundleId,
    pub policy: MemberPolicy,
    pub credits: bool,
}

/// What an edge is (T25.F12 invariant 2): lone with its own optional
/// learning, or a member of a bundle. A lone edge cannot hold a policy or
/// `credits`; a member's learning lives in its policy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeRole {
    Lone(Option<Learning>),
    Member(Membership),
}

/// Weighted edge in the graph. Inside a bundle, `own_weight` is a modifier
/// on top of the bundle term, born 0; outside one it is the genomic weight.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "EdgeWire", into = "EdgeWire")]
pub struct GraphEdge {
    pub(super) source: GraphSource,
    pub(super) own_weight: f32,
    pub(super) role: EdgeRole,
}

// A 6-byte source, a 4-byte weight and the role: a 24-byte learning beside
// a 2-byte bundle and a 1-byte `credits`, 40 bytes at 4-byte alignment
// (T25.F03, T25.F06, T25.F12).
const _: () = assert!(std::mem::size_of::<GraphEdge>() == 40);

/// The edge's JSON shape: `learning` on a lone edge, `membership` on a
/// member, never both.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeWire {
    source: GraphSource,
    own_weight: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    learning: Option<Learning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    membership: Option<Membership>,
}

impl TryFrom<EdgeWire> for GraphEdge {
    type Error = &'static str;

    fn try_from(wire: EdgeWire) -> Result<Self, Self::Error> {
        let role = match (wire.learning, wire.membership) {
            (learning, None) => EdgeRole::Lone(learning),
            (None, Some(membership)) => EdgeRole::Member(membership),
            (Some(_), Some(_)) => {
                return Err("a member's learning lives in its policy, not `learning`")
            }
        };
        Ok(Self {
            source: wire.source,
            own_weight: wire.own_weight,
            role,
        })
    }
}

impl From<GraphEdge> for EdgeWire {
    fn from(edge: GraphEdge) -> Self {
        let (learning, membership) = match edge.role {
            EdgeRole::Lone(learning) => (learning, None),
            EdgeRole::Member(membership) => (None, Some(membership)),
        };
        Self {
            source: edge.source,
            own_weight: edge.own_weight,
            learning,
            membership,
        }
    }
}

impl GraphEdge {
    /// A static, lone edge.
    #[inline]
    #[must_use]
    pub const fn new(source: GraphSource, own_weight: f32) -> Self {
        Self {
            source,
            own_weight,
            role: EdgeRole::Lone(None),
        }
    }

    /// Where the edge reads from.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> GraphSource {
        self.source
    }

    /// The edge's own weight: its genomic weight when lone, its modifier on
    /// top of the bundle term when a member.
    #[inline]
    #[must_use]
    pub const fn own_weight(&self) -> f32 {
        self.own_weight
    }

    /// Whether the edge is lone or a member.
    #[inline]
    #[must_use]
    pub const fn role(&self) -> EdgeRole {
        self.role
    }

    /// The learning the edge stores: a lone edge's, an `Own` member's
    /// override; `None` otherwise. Not what the edge runs: the learning
    /// layout resolves that.
    #[inline]
    #[must_use]
    pub const fn stored_learning(&self) -> Option<Learning> {
        match self.role {
            EdgeRole::Lone(learning) => learning,
            EdgeRole::Member(membership) => membership.policy.stored(),
        }
    }

    /// The bundle this edge belongs to, if any.
    #[inline]
    #[must_use]
    pub const fn membership(&self) -> Option<Membership> {
        match self.role {
            EdgeRole::Lone(_) => None,
            EdgeRole::Member(membership) => Some(membership),
        }
    }

    /// Whether this edge is a member of `bundle`.
    #[inline]
    #[must_use]
    pub fn is_member_of(&self, bundle: BundleId) -> bool {
        self.membership()
            .is_some_and(|membership| membership.bundle == bundle)
    }
}

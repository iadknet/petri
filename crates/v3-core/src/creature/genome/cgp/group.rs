//! Edge groups (T25.F03): one developmental program that wires a whole map.
//!
//! A [`Group`] is a record `{shape, kind, offset, scale}`; its members are
//! real sink edges whose [`super::Membership`] names it. A member's genomic
//! weight is [`compose`]`(own_weight, scale, pattern)`.

use crate::creature::genome::vote::DirectedVoteKind;

/// Index of a group in its graph's `groups` list.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct GroupId(pub(super) u16);

impl GroupId {
    #[inline]
    #[must_use]
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// The group's index in `groups`.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The geometry a group's members fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Members: ring cell `c` of `input_refs[ring_ref]` into sink
    /// `K[(c + offset) mod 8]`; pattern 1.
    Ring { ring_ref: u16 },
}

/// One edge group's record. Fields are written only through the container's
/// group primitives.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub(super) shape: Shape,
    pub(super) kind: DirectedVoteKind,
    /// Rotation, always `< 8`.
    pub(super) offset: u8,
    /// One shared weight, born 0.
    pub(super) scale: f32,
}

impl Group {
    #[inline]
    #[must_use]
    pub const fn shape(&self) -> Shape {
        self.shape
    }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> DirectedVoteKind {
        self.kind
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> u8 {
        self.offset
    }

    #[inline]
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// The only composition formula: a member's genomic weight from its own
/// correction, its group's scale and its pattern.
#[inline]
#[must_use]
pub fn compose(own: f32, scale: f32, pattern: f32) -> f32 {
    scale * pattern + own
}

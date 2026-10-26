//! Edge groups (T25.F03): one developmental program that wires a whole map.
//!
//! A [`Group`] is a record `{shape, kind, offset, scale, inherit_scale,
//! retention}`; its members are
//! real sink edges whose [`super::Membership`] names it. A member's genomic
//! weight is [`compose`]`(own_weight, scale, pattern)`.

use super::{CgpGraphBackendDef, EdgeOccurrence, OutputSinkKind};
use crate::contracts::{compass_x, compass_y, DirectionPair, InputReference};
use crate::creature::genome::vote::{DirectedVoteKind, VOTE_DIRECTION_COUNT};

/// Where a member sits relative to its group's kind `K` (T25.F10): a
/// **direction member** on a direction sink `K(d)`, or a **fact member** of
/// a `DirectionPair` gene on `K`'s drive sink. The role is read from the
/// sink, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemberSite {
    /// The direction sink `K(d)`, `d < 8`.
    Direction(u8),
    /// The drive sink `Drive K`.
    Drive,
}

impl MemberSite {
    /// The site a sink of kind `sink` offers a member of a kind-`kind`
    /// group; `None` for any other sink, another kind's included.
    #[must_use]
    pub fn of(sink: OutputSinkKind, kind: DirectedVoteKind) -> Option<Self> {
        if sink == Self::Drive.sink(kind) {
            return Some(Self::Drive);
        }
        sink.direction()
            .filter(|&(on, _)| on == kind)
            .map(|(_, d)| Self::Direction(d))
    }

    /// The sink this site names for a kind-`kind` group.
    #[must_use]
    pub const fn sink(self, kind: DirectedVoteKind) -> OutputSinkKind {
        match self {
            Self::Direction(d) => OutputSinkKind::ActionVote(kind.sink(d)),
            Self::Drive => OutputSinkKind::ActionVote(kind.drive()),
        }
    }
}

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
    /// Members (T25.F04): the `x` and `y` fields of pair `pair` of
    /// `input_refs[ref_idx]` (`WorldInputKey::direction_pairs`) into sink
    /// `K[d]` at pattern `compass_x((d − offset) mod 8)` and
    /// `compass_y(...)`: at a positive scale the group votes toward the
    /// sensed offset.
    DirectionPair { ref_idx: u16, pair: u8 },
}

impl Shape {
    /// The `input_refs` index the shape reads.
    #[inline]
    #[must_use]
    pub const fn ref_idx(self) -> u16 {
        match self {
            Self::Ring { ring_ref } => ring_ref,
            Self::DirectionPair { ref_idx, .. } => ref_idx,
        }
    }

    /// The pair a `DirectionPair` shape names, when its reference declares
    /// it; `None` for a `Ring` or an unresolvable pair.
    #[must_use]
    pub fn resolve_pair(self, input_refs: &[InputReference]) -> Option<DirectionPair> {
        let Self::DirectionPair { ref_idx, pair } = self else {
            return None;
        };
        input_refs
            .get(usize::from(ref_idx))?
            .direction_pairs()
            .get(usize::from(pair))
            .copied()
    }
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
    /// Whether a parent's learned scale seeds the child's (T25.F07); born
    /// false.
    pub(super) inherit_scale: bool,
    /// The learned scale's retention per world tick, in `[0, 1]` (T25.F09);
    /// born 1, an exact no-op.
    pub(super) retention: f32,
}

impl Group {
    /// The silent record: scale 0, `inherit_scale` false, retention 1.
    #[must_use]
    pub(super) const fn new(shape: Shape, kind: DirectedVoteKind, offset: u8) -> Self {
        Self {
            shape,
            kind,
            offset,
            scale: 0.0,
            inherit_scale: false,
            retention: 1.0,
        }
    }

    /// The group's pattern at the member reading field `sub_idx` on a sink
    /// of direction `direction`: 1 for a `Ring` group whose `ring_ref`
    /// resolves to a ring of `input_refs` (the coordinate ignored); for a
    /// `DirectionPair` whose pair resolves, `compass_x((direction − offset)
    /// mod 8)` at the pair's `x` field and `compass_y(...)` at its `y`
    /// field; 0 otherwise, a `direction >= 8` included. Resolved from the
    /// shape reference, never from the member's own source.
    #[must_use]
    pub fn pattern(&self, input_refs: &[InputReference], sub_idx: u16, direction: u8) -> f32 {
        match self.shape {
            Shape::Ring { ring_ref } => {
                if input_refs
                    .get(usize::from(ring_ref))
                    .is_some_and(InputReference::is_ring)
                {
                    1.0
                } else {
                    0.0
                }
            }
            Shape::DirectionPair { .. } => {
                let Some(pair) = self.shape.resolve_pair(input_refs) else {
                    return 0.0;
                };
                if direction >= VOTE_DIRECTION_COUNT {
                    return 0.0;
                }
                let relative = pair_relative(direction, self.offset);
                if sub_idx == pair.x {
                    compass_x(relative)
                } else if sub_idx == pair.y {
                    compass_y(relative)
                } else {
                    0.0
                }
            }
        }
    }

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

    /// Whether a parent's learned scale seeds the child's.
    #[inline]
    #[must_use]
    pub const fn inherit_scale(&self) -> bool {
        self.inherit_scale
    }

    /// The learned scale's retention per world tick (T25.F09).
    #[inline]
    #[must_use]
    pub const fn retention(&self) -> f32 {
        self.retention
    }
}

/// Detach the member at `occurrence` (an edit that changes its source outside
/// a shape operator): its own weight becomes its genomic weight, bitwise, and
/// it leaves its group. Returns the edge's new occurrence, on which the
/// calling operator continues its edit, and whether the group was removed
/// (it was the last member).
pub fn detach(
    def: &mut CgpGraphBackendDef,
    occurrence: EdgeOccurrence,
    input_refs: &[InputReference],
) -> (EdgeOccurrence, bool) {
    let pattern = def.member_pattern(occurrence, input_refs);
    def.detach_at(occurrence, pattern)
}

/// The `Ring` offset whose member on channel `channel` sits on direction
/// `d`'s sink: `(d − channel) mod 8`, for every `u8` pair.
#[must_use]
pub const fn ring_offset(d: u8, channel: u8) -> u8 {
    let n = VOTE_DIRECTION_COUNT;
    (d % n + n - channel % n) % n
}

/// A `DirectionPair` member's relative direction on a sink of direction `d`
/// in a group at `offset`: `(d − offset) mod 8`.
#[must_use]
pub const fn pair_relative(d: u8, offset: u8) -> u8 {
    ring_offset(d, offset)
}

/// The `(channel, direction)` members a new `Ring` group at `offset`
/// materializes: channel `c` on direction `(c + offset) mod 8` — eight.
pub fn ring_birth_members(offset: u8) -> impl Iterator<Item = (u16, u8)> {
    let n = VOTE_DIRECTION_COUNT;
    (0..n).map(move |channel| (u16::from(channel), (channel + offset % n) % n))
}

/// One birth coordinate of a group (T30.F01): the field a member born with
/// the group reads, the site it is born on, and whether a member of the
/// group reading that field of the shape's reference sits there now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BirthCoordinate {
    pub field: u16,
    pub site: MemberSite,
    pub present: bool,
}

/// The `(field, direction)` members a new `DirectionPair` group on `pair` at
/// `offset` materializes: every one whose pattern is non-zero, `x` at the
/// relative directions NE, E, SE, SW, W, NW and `y` at N, NE, SE, S, SW, NW —
/// twelve.
pub fn pair_birth_members(pair: DirectionPair, offset: u8) -> impl Iterator<Item = (u16, u8)> {
    let n = VOTE_DIRECTION_COUNT;
    (0..n).flat_map(move |relative| {
        let d = (relative + offset % n) % n;
        let x = (compass_x(relative) != 0.0).then_some((pair.x, d));
        let y = (compass_y(relative) != 0.0).then_some((pair.y, d));
        x.into_iter().chain(y)
    })
}

/// The only composition formula: a member's genomic weight from its own
/// correction, its group's scale and its pattern.
#[inline]
#[must_use]
pub fn compose(own: f32, scale: f32, pattern: f32) -> f32 {
    scale * pattern + own
}

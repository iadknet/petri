//! Edge bundles (T25.F03): one developmental program that wires a whole map.
//!
//! A [`Bundle`] is a record `{shape, kind, offset, scale, inherit_scale,
//! retention}`; its members are
//! real sink edges whose [`super::Membership`] names it. A member's genomic
//! weight is [`compose`]`(own_weight, scale, pattern)`.

use super::{CgpGraphBackendDef, EdgeOccurrence, OutputSinkKind};
use crate::contracts::votes::{DirectedVoteKind, VoteSink, VOTE_DIRECTION_COUNT};
use crate::contracts::{compass_x, compass_y, DirectionPair, FieldSet, InputReference};

/// Where a member sits relative to its bundle's kind `K` (T25.F10): a
/// **direction member** on a direction sink `K(d)`, or a **fact member** of
/// a `DirectionPair` gene on `K`'s drive sink. The role is read from the
/// sink, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Site {
    /// The direction sink `K(d)`, `d < 8`.
    Direction(u8),
    /// The drive sink `Drive K`.
    Drive,
}

impl Site {
    /// The site a sink of kind `sink` offers a member of a kind-`kind`
    /// bundle; `None` for any other sink, another kind's included.
    #[must_use]
    pub fn of(sink: OutputSinkKind, kind: DirectedVoteKind) -> Option<Self> {
        if sink == Self::Drive.sink(kind) {
            return Some(Self::Drive);
        }
        sink.direction()
            .filter(|&(on, _)| on == kind)
            .map(|(_, d)| Self::Direction(d))
    }

    /// The sink this site names for a kind-`kind` bundle.
    #[must_use]
    pub const fn sink(self, kind: DirectedVoteKind) -> OutputSinkKind {
        match self {
            Self::Direction(d) => OutputSinkKind::ActionVote(kind.sink(d)),
            Self::Drive => OutputSinkKind::ActionVote(kind.drive()),
        }
    }
}

/// Index of a bundle in its graph's `bundles` list.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct BundleId(pub(super) u16);

impl BundleId {
    #[inline]
    #[must_use]
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// The bundle's index in `bundles`.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The geometry a bundle's members fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Members: ring cell `c` of `input_refs[ring_ref]` into sink
    /// `K[(c + offset) mod 8]`; pattern 1.
    Ring { ring_ref: u16 },
    /// Members (T25.F04): the `x` and `y` fields of pair `pair` of
    /// `input_refs[ref_idx]` (`WorldInputKey::direction_pairs`) into sink
    /// `K[d]` at pattern `compass_x((d − offset) mod 8)` and
    /// `compass_y(...)`: at a positive scale the bundle votes toward the
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

/// One edge bundle's record. Fields are written only through the container's
/// bundle primitives.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
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

impl Bundle {
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

    /// The bundle's pattern at the member reading field `sub_idx` on a sink
    /// of direction `direction`: 1 for a `Ring` bundle whose `ring_ref`
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
/// it leaves its bundle. Returns the edge's new occurrence, on which the
/// calling operator continues its edit, and whether the bundle was removed
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
/// in a bundle at `offset`: `(d − offset) mod 8`.
#[must_use]
pub const fn pair_relative(d: u8, offset: u8) -> u8 {
    ring_offset(d, offset)
}

/// The `(channel, direction)` members a new `Ring` bundle at `offset`
/// materializes: channel `c` on direction `(c + offset) mod 8` — eight.
pub fn ring_birth_members(offset: u8) -> impl Iterator<Item = (u16, u8)> {
    let n = VOTE_DIRECTION_COUNT;
    (0..n).map(move |channel| (u16::from(channel), (channel + offset % n) % n))
}

/// One birth coordinate of a bundle (T30.F01): the field a member born with
/// the bundle reads, the site it is born on, and whether a member of the
/// bundle reading that field of the shape's reference sits there now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BirthCoordinate {
    pub field: u16,
    pub site: Site,
    pub present: bool,
}

/// The `(field, direction)` members a new `DirectionPair` bundle on `pair` at
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
/// correction, its bundle's scale and its pattern.
#[inline]
#[must_use]
pub fn compose(own: f32, scale: f32, pattern: f32) -> f32 {
    scale * pattern + own
}

/// What a bundle reads (T25.F11): one field set of a declared sensor, or a
/// shared-memory bank (no declaration; its one field set is index 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BundleSource {
    Sensor { ref_idx: u16, field_set: u8 },
    Memory { previous: bool },
}

impl BundleSource {
    /// The field set index the source names.
    #[inline]
    #[must_use]
    pub const fn field_set(self) -> u8 {
        match self {
            Self::Sensor { field_set, .. } => field_set,
            Self::Memory { .. } => 0,
        }
    }
}

/// How a bundle's members sit on the sinks (T25.F11): every field on one
/// vote sink (`Plain`), a ring cell per direction (`Ring`), or an offset
/// pair across the compass with its facts on the drive (`DirectionPair`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Wiring {
    Plain { sink: VoteSink },
    Ring { kind: DirectedVoteKind, offset: u8 },
    DirectionPair { kind: DirectedVoteKind, offset: u8 },
}

/// One sampled plan with its birth law (T25.F11): the field set it wires,
/// the wiring, and the record's scale (0 for a silent bundle).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BundleBirth {
    pub field_set: u8,
    pub wiring: Wiring,
    pub scale: f32,
}

/// Whether `def` carries every sink `kind`'s directed wirings need: its
/// eight direction sinks, and its drive when `drive` is set.
fn hosts_kind(def: &CgpGraphBackendDef, kind: DirectedVoteKind, drive: bool) -> bool {
    let present = |vote: VoteSink| def.sink(OutputSinkKind::ActionVote(vote)).is_some();
    (0..VOTE_DIRECTION_COUNT).all(|d| present(kind.sink(d))) && (!drive || present(kind.drive()))
}

/// Every admissible wiring of `set` on `def`, in deterministic order (T25.F11
/// invariant 5): a set without geometry one `Plain` per present vote sink in
/// catalog order; a `Ring` or `Pair` set `(kind, offset)` for every kind
/// whose required sinks are present, kinds in `DirectedVoteKind::ALL` order,
/// offsets `0..8`. Depends only on the set's geometry and the sinks present.
#[must_use]
pub fn plans(def: &CgpGraphBackendDef, set: &FieldSet) -> Vec<Wiring> {
    let directed = |drive: bool, wiring: fn(DirectedVoteKind, u8) -> Wiring| {
        DirectedVoteKind::ALL
            .into_iter()
            .filter(|&kind| hosts_kind(def, kind, drive))
            .flat_map(move |kind| (0..VOTE_DIRECTION_COUNT).map(move |offset| wiring(kind, offset)))
            .collect()
    };
    match set {
        FieldSet::Plain { .. } => def
            .output_sinks()
            .iter()
            .filter_map(|sink| match sink.kind {
                OutputSinkKind::ActionVote(vote) => Some(Wiring::Plain { sink: vote }),
                _ => None,
            })
            .collect(),
        FieldSet::Ring => directed(false, |kind, offset| Wiring::Ring { kind, offset }),
        FieldSet::Pair(_) => directed(true, |kind, offset| Wiring::DirectionPair { kind, offset }),
    }
}

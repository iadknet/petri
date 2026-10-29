//! Bundles (T25.F03, T25.F11): one developmental program that wires a whole
//! field set.
//!
//! A [`Bundle`] is a record `{source, wiring, scale, scale_learning,
//! member_default}`; its members are real sink edges whose [`super::Membership`]
//! names it. Every member, whatever its wiring, weighs
//! [`compose`]`(own_weight, scale, pattern)`, the pattern read from the
//! wiring at the member's coordinate `(field, site)`.

use super::{CgpGraphBackendDef, EdgeOccurrence, OutputSinkKind};
use crate::contracts::votes::{DirectedVoteKind, VoteSink, VOTE_DIRECTION_COUNT};
use crate::contracts::{
    compass_x, compass_y, FieldSet, InputReference, SensorContext, SensorDescriptor, MEMORY_BANK,
};
use crate::genome::learning::Learning;

/// Where a member sits relative to its bundle's wiring (T25.F11): the one
/// vote sink of a `Plain` wiring, a direction sink `K(d)` of a geometric
/// wiring's kind `K`, or `K`'s drive. Read from the sink, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Site {
    /// The vote sink a `Plain` wiring names.
    Sink,
    /// The direction sink `K(d)`, `d < 8`.
    Direction(u8),
    /// The drive sink `Drive K`.
    Drive,
}

impl Site {
    /// The site a sink of kind `sink` offers a member of a bundle wired
    /// `wiring`; `None` for any other sink.
    #[must_use]
    pub fn of(sink: OutputSinkKind, wiring: &Wiring) -> Option<Self> {
        match *wiring {
            Wiring::Plain { sink: vote } => {
                (sink == OutputSinkKind::ActionVote(vote)).then_some(Self::Sink)
            }
            Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. } => {
                if sink == OutputSinkKind::ActionVote(kind.drive()) {
                    return Some(Self::Drive);
                }
                sink.direction()
                    .filter(|&(on, _)| on == kind)
                    .map(|(_, d)| Self::Direction(d))
            }
        }
    }

    /// The sink this site names under `wiring`; `None` for a site the
    /// wiring has no sink for (`Sink` of a geometric wiring, a direction or
    /// the drive of a `Plain` one).
    #[must_use]
    pub const fn sink(self, wiring: &Wiring) -> Option<OutputSinkKind> {
        match (self, *wiring) {
            (Self::Sink, Wiring::Plain { sink }) => Some(OutputSinkKind::ActionVote(sink)),
            (
                Self::Direction(d),
                Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. },
            ) => Some(OutputSinkKind::ActionVote(kind.sink(d))),
            (Self::Drive, Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. }) => {
                Some(OutputSinkKind::ActionVote(kind.drive()))
            }
            _ => None,
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

    /// The declaration index a sensor source reads; `None` for a memory
    /// bank.
    #[inline]
    #[must_use]
    pub const fn ref_idx(self) -> Option<u16> {
        match self {
            Self::Sensor { ref_idx, .. } => Some(ref_idx),
            Self::Memory { .. } => None,
        }
    }

    /// The field set this source names on a node declaring `input_refs`,
    /// resolved under [`SensorContext::WIDEST`]: the genome's geometry and
    /// validity never depend on the configured queue.
    #[must_use]
    pub fn resolve(self, input_refs: &[InputReference]) -> Option<FieldSet> {
        self.resolve_in(input_refs, &SensorContext::WIDEST)
    }

    /// The field set this source names on a node declaring `input_refs`
    /// under `context`: a reader's view of the run (an `ActionQueue` set
    /// spans the configured queue's width).
    #[must_use]
    pub fn resolve_in(
        self,
        input_refs: &[InputReference],
        context: &SensorContext,
    ) -> Option<FieldSet> {
        match self {
            Self::Sensor { ref_idx, field_set } => {
                let reference = input_refs.get(usize::from(ref_idx))?;
                SensorDescriptor::of(reference, context).field_set(field_set)
            }
            Self::Memory { .. } => Some(MEMORY_BANK),
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

impl Wiring {
    /// The kind a geometric wiring votes for; `None` for `Plain`.
    #[inline]
    #[must_use]
    pub const fn kind(self) -> Option<DirectedVoteKind> {
        match self {
            Self::Ring { kind, .. } | Self::DirectionPair { kind, .. } => Some(kind),
            Self::Plain { .. } => None,
        }
    }

    /// A geometric wiring's rotation; `None` for `Plain`.
    #[inline]
    #[must_use]
    pub const fn offset(self) -> Option<u8> {
        match self {
            Self::Ring { offset, .. } | Self::DirectionPair { offset, .. } => Some(offset),
            Self::Plain { .. } => None,
        }
    }

    /// This wiring with a geometric wiring's kind and offset replaced; a
    /// `Plain` wiring is returned unchanged.
    #[must_use]
    pub const fn rotated(self, kind: DirectedVoteKind, offset: u8) -> Self {
        match self {
            Self::Ring { .. } => Self::Ring { kind, offset },
            Self::DirectionPair { .. } => Self::DirectionPair { kind, offset },
            Self::Plain { .. } => self,
        }
    }

    /// Whether this wiring fits `set`'s geometry: `Plain` a set without
    /// geometry, `Ring` a ring, `DirectionPair` a pair.
    #[must_use]
    pub const fn fits(self, set: &FieldSet) -> bool {
        matches!(
            (self, set),
            (Self::Plain { .. }, FieldSet::Plain { .. })
                | (Self::Ring { .. }, FieldSet::Ring)
                | (Self::DirectionPair { .. }, FieldSet::Pair(_))
        )
    }

    /// Whether the wiring alone, without the field set, can map `field` to
    /// `site` (T25.F11 invariant 4, the record-and-edge half of
    /// `MemberShape`): a `Plain` member on `Sink`; a `Ring` cell `c < 8` on
    /// `Direction((c + offset) mod 8)`, never the drive; a `DirectionPair`
    /// member on a direction or the drive, whose field role needs the set.
    #[must_use]
    pub const fn admits_site(self, field: u16, site: Site) -> bool {
        let n = VOTE_DIRECTION_COUNT as u16;
        match (self, site) {
            (Self::Plain { .. }, Site::Sink) => true,
            (Self::Ring { offset, .. }, Site::Direction(d)) => {
                field < n && (field + offset as u16 % n) % n == d as u16
            }
            (Self::DirectionPair { .. }, Site::Direction(_) | Site::Drive) => true,
            _ => false,
        }
    }

    /// The pattern at coordinate `(field, site)` of a bundle on `set` with
    /// this wiring (T25.F11 invariant 3), `None` when the wiring maps
    /// `field` to no such site (an off-site or wrong-field member): 1 for
    /// every field of a `Plain` set on `Sink`; 1 for ring cell `c` on
    /// `Direction((c + offset) mod 8)`; for a pair, `compass_x` /
    /// `compass_y` of `(d − offset) mod 8` at `x` / `y` on a direction where
    /// it is non-zero, and 1 at each fact on `Drive`.
    #[must_use]
    pub fn pattern(self, set: &FieldSet, field: u16, site: Site) -> Option<f32> {
        if !self.fits(set) || !set.contains(field) {
            return None;
        }
        let n = VOTE_DIRECTION_COUNT;
        match (self, site, set) {
            (Self::Plain { .. }, Site::Sink, _) => Some(1.0),
            (Self::Ring { offset, .. }, Site::Direction(d), _) => {
                let on = (u16::from(offset % n) + field) % u16::from(n);
                (u16::from(d) == on).then_some(1.0)
            }
            (Self::DirectionPair { offset, .. }, Site::Direction(d), FieldSet::Pair(pair))
                if d < n =>
            {
                let relative = pair_relative(d, offset);
                let pattern = if field == pair.x {
                    compass_x(relative)
                } else if field == pair.y {
                    compass_y(relative)
                } else {
                    0.0
                };
                (pattern != 0.0).then_some(pattern)
            }
            (Self::DirectionPair { .. }, Site::Drive, FieldSet::Pair(pair)) => {
                pair.facts.contains(&field).then_some(1.0)
            }
            _ => None,
        }
    }

    /// Every coordinate a new bundle on `set` with this wiring materializes,
    /// in canonical order: a `Plain` set's fields on `Sink`; a ring's eight
    /// cells; a pair's twelve non-zero compass coordinates
    /// (x then y per relative direction) then its facts on `Drive`. Empty when the
    /// wiring does not fit the set.
    #[must_use]
    pub fn coordinates(self, set: &FieldSet) -> Vec<(u16, Site)> {
        let n = VOTE_DIRECTION_COUNT;
        match (self, set) {
            (Self::Plain { .. }, FieldSet::Plain { .. }) => {
                set.fields().into_iter().map(|f| (f, Site::Sink)).collect()
            }
            (Self::Ring { offset, .. }, FieldSet::Ring) => (0..n)
                .map(|c| (u16::from(c), Site::Direction((c + offset % n) % n)))
                .collect(),
            (Self::DirectionPair { offset, .. }, FieldSet::Pair(pair)) => {
                let mut coordinates: Vec<(u16, Site)> = (0..n)
                    .flat_map(|relative| {
                        let d = (relative + offset % n) % n;
                        let x =
                            (compass_x(relative) != 0.0).then_some((pair.x, Site::Direction(d)));
                        let y =
                            (compass_y(relative) != 0.0).then_some((pair.y, Site::Direction(d)));
                        x.into_iter().chain(y)
                    })
                    .collect();
                coordinates.extend(pair.facts.iter().map(|&fact| (fact, Site::Drive)));
                coordinates
            }
            _ => Vec::new(),
        }
    }
}

/// One sampled plan with its birth law (T25.F11): the field set it wires,
/// the wiring, and the record's scale (0 for a silent bundle).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BundleBirth {
    pub field_set: u8,
    pub wiring: Wiring,
    pub scale: f32,
}

/// One bundle's record. Fields are written only through the container's
/// bundle primitives.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub(super) source: BundleSource,
    pub(super) wiring: Wiring,
    /// One shared weight.
    pub(super) scale: f32,
    /// How the scale learns ("learns as a group", T25.F12); born absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) scale_learning: Option<Learning>,
    /// The learning every `Inherit` member runs ("learns per edge",
    /// T25.F12); born absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) member_default: Option<Learning>,
}

/// One of a bundle's two learning slots (T25.F12 invariant 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearningSlot {
    /// `scale_learning`.
    Scale,
    /// `member_default`.
    MemberDefault,
}

impl LearningSlot {
    /// Both slots in target order: `scale_learning` first.
    pub const ALL: [Self; 2] = [Self::Scale, Self::MemberDefault];
}

impl Bundle {
    /// A silent record: scale 0, both learning slots absent.
    #[must_use]
    pub(super) const fn new(source: BundleSource, wiring: Wiring) -> Self {
        Self {
            source,
            wiring,
            scale: 0.0,
            scale_learning: None,
            member_default: None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn source(&self) -> BundleSource {
        self.source
    }

    #[inline]
    #[must_use]
    pub const fn wiring(&self) -> Wiring {
        self.wiring
    }

    #[inline]
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// How the scale learns, `None` for a scale that does not.
    #[inline]
    #[must_use]
    pub const fn scale_learning(&self) -> Option<Learning> {
        self.scale_learning
    }

    /// The learning `Inherit` members run, `None` for none.
    #[inline]
    #[must_use]
    pub const fn member_default(&self) -> Option<Learning> {
        self.member_default
    }

    /// The learning stored in `slot`.
    #[inline]
    #[must_use]
    pub const fn slot(&self, slot: LearningSlot) -> Option<Learning> {
        match slot {
            LearningSlot::Scale => self.scale_learning,
            LearningSlot::MemberDefault => self.member_default,
        }
    }

    /// The storage of `slot`, for the container's one slot writer.
    pub(super) fn slot_mut(&mut self, slot: LearningSlot) -> &mut Option<Learning> {
        match slot {
            LearningSlot::Scale => &mut self.scale_learning,
            LearningSlot::MemberDefault => &mut self.member_default,
        }
    }
}

/// Detach the member at `occurrence` (an edit that changes its source
/// outside a bundle operator): its own weight becomes its genomic weight,
/// bitwise, and it leaves its bundle. Returns the edge's new occurrence, on
/// which the calling operator continues its edit, and whether the bundle
/// was removed (it was the last member).
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

/// One birth coordinate of a bundle (T30.F01): the field a member born with
/// the bundle reads, the site it is born on, and the location of the member
/// of the bundle reading that field that sits there now (T30.F14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BirthCoordinate {
    pub field: u16,
    pub site: Site,
    /// The first such member in `members` order (sink order, then edge
    /// order); `None` for a hole. A coordinate is present exactly when it
    /// carries a member.
    pub member: Option<EdgeOccurrence>,
}

/// The only composition formula: a member's genomic weight from its own
/// correction, its bundle's scale and its pattern.
#[inline]
#[must_use]
pub fn compose(own: f32, scale: f32, pattern: f32) -> f32 {
    scale * pattern + own
}

/// Whether `def` carries every sink `kind`'s directed wirings need: its
/// eight direction sinks, and its drive when `drive` is set.
fn hosts_kind(def: &CgpGraphBackendDef, kind: DirectedVoteKind, drive: bool) -> bool {
    let present = |vote: VoteSink| def.sink(OutputSinkKind::ActionVote(vote)).is_some();
    (0..VOTE_DIRECTION_COUNT).all(|d| present(kind.sink(d))) && (!drive || present(kind.drive()))
}

/// Whether `def` carries every sink `wiring` needs: a `Plain` wiring's vote
/// sink; a `Ring`'s eight direction sinks; a `DirectionPair`'s eight and the
/// drive.
#[must_use]
pub fn hosts(def: &CgpGraphBackendDef, wiring: &Wiring) -> bool {
    match *wiring {
        Wiring::Plain { sink } => def.sink(OutputSinkKind::ActionVote(sink)).is_some(),
        Wiring::Ring { kind, .. } => hosts_kind(def, kind, false),
        Wiring::DirectionPair { kind, .. } => hosts_kind(def, kind, true),
    }
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

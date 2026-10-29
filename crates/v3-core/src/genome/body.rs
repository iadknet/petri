//! The body chromosome (T29.F02): five genes, one genome unit each, owned
//! here with their deterministic edits. The body policy in `mutation` samples
//! an edit and the range it applies under; the body rules in
//! `creature::body` read the values and resolve the ranges from config.

use std::fmt;

/// How many colour channels the colour gene carries.
const CHANNELS: usize = 6;

/// One body gene, in chromosome order.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum BodyGene {
    Colour,
    KinTag,
    Capacity,
    StealShare,
    GiveShare,
}

impl BodyGene {
    /// Every body gene, in chromosome order.
    pub const ALL: [Self; 5] = [
        Self::Colour,
        Self::KinTag,
        Self::Capacity,
        Self::StealShare,
        Self::GiveShare,
    ];
}

/// The colour gene: six internal channels (HSL-mapped; rendered by
/// `creature::colour`), the channel a step changes and each channel's step
/// direction. The active channel is always in `0..6`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "PhenotypeRecord")]
pub struct Phenotype {
    channels: [u8; CHANNELS],
    active_channel: usize,
    polarity: [bool; CHANNELS],
}

/// A decoded colour gene before its active channel is checked.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhenotypeRecord {
    channels: [u8; CHANNELS],
    active_channel: usize,
    polarity: [bool; CHANNELS],
}

impl TryFrom<PhenotypeRecord> for Phenotype {
    type Error = ActiveChannelOutOfRange;

    fn try_from(record: PhenotypeRecord) -> Result<Self, Self::Error> {
        Self::new(record.channels, record.active_channel, record.polarity)
    }
}

/// A colour gene whose active channel is not in `0..6`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveChannelOutOfRange(pub usize);

impl fmt::Display for ActiveChannelOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "active channel {} is not in 0..{CHANNELS}", self.0)
    }
}

impl std::error::Error for ActiveChannelOutOfRange {}

/// One colour step's sampled choices: a switch to another channel (its index
/// among the five channels other than the active one), whether the new
/// active channel's polarity flips, and the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhenotypeDraw {
    /// `Some(k)`: the active channel becomes the `k`-th of the other five,
    /// in channel order.
    pub switch_to_other: Option<usize>,
    /// Whether the (new) active channel's polarity flips.
    pub flip_polarity: bool,
    /// The step applied to the active channel, at least 1.
    pub step: u8,
}

impl Phenotype {
    /// The founder colour: channels that render as RGB `[204, 61, 61]`,
    /// channel 0 active, every polarity adding.
    pub const FOUNDER: Self = Self {
        channels: [0, 0, 92, 92, 138, 138],
        active_channel: 0,
        polarity: [true; CHANNELS],
    };

    /// A colour gene, rejecting an active channel outside `0..6`.
    pub fn new(
        channels: [u8; CHANNELS],
        active_channel: usize,
        polarity: [bool; CHANNELS],
    ) -> Result<Self, ActiveChannelOutOfRange> {
        if active_channel < CHANNELS {
            Ok(Self {
                channels,
                active_channel,
                polarity,
            })
        } else {
            Err(ActiveChannelOutOfRange(active_channel))
        }
    }

    /// The six channels: hue pair, saturation pair, lightness pair.
    #[must_use]
    pub const fn channels(&self) -> [u8; CHANNELS] {
        self.channels
    }

    /// The channel a step changes, in `0..6`.
    #[must_use]
    pub const fn active_channel(&self) -> usize {
        self.active_channel
    }

    /// Each channel's step direction: `true` adds, `false` subtracts.
    #[must_use]
    pub const fn polarity(&self) -> [bool; CHANNELS] {
        self.polarity
    }

    /// This colour after the sampled step `draw`: the active channel
    /// switched to the drawn other channel, its polarity flipped when drawn,
    /// then the active channel stepped by `draw.step` in its polarity's
    /// direction with wrapping `u8` arithmetic.
    ///
    /// # Panics
    ///
    /// When `draw.switch_to_other` is not in `0..5`.
    #[must_use]
    pub fn mutated(self, draw: &PhenotypeDraw) -> Self {
        let Self {
            mut channels,
            active_channel,
            mut polarity,
        } = self;
        let active = draw.switch_to_other.map_or(active_channel, |k| {
            (0..CHANNELS)
                .filter(|&channel| channel != active_channel)
                .nth(k)
                .expect("an index among the other five channels")
        });
        if draw.flip_polarity {
            polarity[active] = !polarity[active];
        }
        channels[active] = if polarity[active] {
            channels[active].wrapping_add(draw.step)
        } else {
            channels[active].wrapping_sub(draw.step)
        };
        Self {
            channels,
            active_channel: active,
            polarity,
        }
    }
}

/// The ranges the three scalar genes are held in at an edit: plain numbers,
/// resolved from config by the body rules (`creature::body`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyRanges {
    /// The capacity's `(min, max)`.
    pub capacity: (f32, f32),
    /// Each share's `(min, max)`.
    pub share: (f32, f32),
}

/// One sampled edit of one body gene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyEdit {
    /// A colour step.
    Colour(PhenotypeDraw),
    /// Flip this bit, in `0..32`, of the kin tag.
    KinTag(u32),
    /// A log-space step of the capacity.
    Capacity(f32),
    /// A log-space step of the steal share.
    StealShare(f32),
    /// A log-space step of the give share.
    GiveShare(f32),
}

impl BodyEdit {
    /// The gene this edit changes.
    #[must_use]
    pub const fn gene(&self) -> BodyGene {
        match self {
            Self::Colour(_) => BodyGene::Colour,
            Self::KinTag(_) => BodyGene::KinTag,
            Self::Capacity(_) => BodyGene::Capacity,
            Self::StealShare(_) => BodyGene::StealShare,
            Self::GiveShare(_) => BodyGene::GiveShare,
        }
    }
}

/// The body chromosome: colour, kin tag, capacity, steal share and give
/// share, one genome unit each. No `Default`: a body's values come from
/// config (founders) or a parent.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyGenome {
    colour: Phenotype,
    kin_tag: u32,
    capacity: f32,
    steal_share: f32,
    give_share: f32,
}

impl BodyGenome {
    /// The body chromosome's genome units: one per gene.
    pub const UNITS: u32 = BodyGene::ALL.len() as u32;

    /// A body of these gene values.
    #[must_use]
    pub const fn new(
        colour: Phenotype,
        kin_tag: u32,
        capacity: f32,
        steal_share: f32,
        give_share: f32,
    ) -> Self {
        Self {
            colour,
            kin_tag,
            capacity,
            steal_share,
            give_share,
        }
    }

    /// The colour gene.
    #[must_use]
    pub const fn colour(&self) -> &Phenotype {
        &self.colour
    }

    /// The kin tag.
    #[must_use]
    pub const fn kin_tag(&self) -> u32 {
        self.kin_tag
    }

    /// The most energy the body can store, in energy units.
    #[must_use]
    pub const fn capacity(&self) -> f32 {
        self.capacity
    }

    /// The share of a victim's store a `StealEnergy` at modifier 1 takes.
    #[must_use]
    pub const fn steal_share(&self) -> f32 {
        self.steal_share
    }

    /// The share of its own store a `GiveEnergy` at modifier 1 passes.
    #[must_use]
    pub const fn give_share(&self) -> f32 {
        self.give_share
    }

    /// Apply `edit` deterministically: the colour step, the kin-tag bit flip,
    /// or a scalar's `(value × e^δ).clamp(min, max)` over its range in
    /// `ranges`, so every step lands in range.
    ///
    /// # Panics
    ///
    /// When a kin-tag bit is not in `0..32`.
    pub fn apply(&mut self, edit: BodyEdit, ranges: &BodyRanges) {
        match edit {
            BodyEdit::Colour(draw) => self.colour = self.colour.mutated(&draw),
            BodyEdit::KinTag(bit) => {
                assert!(bit < u32::BITS, "kin-tag bit {bit} is not in 0..32");
                self.kin_tag ^= 1 << bit;
            }
            BodyEdit::Capacity(delta) => {
                self.capacity = stepped(self.capacity, delta, ranges.capacity);
            }
            BodyEdit::StealShare(delta) => {
                self.steal_share = stepped(self.steal_share, delta, ranges.share);
            }
            BodyEdit::GiveShare(delta) => {
                self.give_share = stepped(self.give_share, delta, ranges.share);
            }
        }
    }

    /// Clamp the three scalars into `ranges`: idempotent on an in-range
    /// body, and how a changed bound reaches a body without a step.
    pub fn clamp(&mut self, ranges: &BodyRanges) {
        let (min, max) = ranges.capacity;
        self.capacity = self.capacity.clamp(min, max);
        let (min, max) = ranges.share;
        self.steal_share = self.steal_share.clamp(min, max);
        self.give_share = self.give_share.clamp(min, max);
    }
}

/// A fixture's body differing in one gene.
#[cfg(test)]
impl BodyGenome {
    pub(crate) const fn with_capacity(self, capacity: f32) -> Self {
        Self { capacity, ..self }
    }

    pub(crate) const fn with_steal_share(self, steal_share: f32) -> Self {
        Self {
            steal_share,
            ..self
        }
    }

    pub(crate) const fn with_give_share(self, give_share: f32) -> Self {
        Self { give_share, ..self }
    }
}

/// One scalar's log-space step, clamped to its range.
fn stepped(value: f32, delta: f32, (min, max): (f32, f32)) -> f32 {
    (value * delta.exp()).clamp(min, max)
}

#[cfg(test)]
#[path = "tests/body_tests.rs"]
mod tests;

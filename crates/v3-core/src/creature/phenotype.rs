//! The colour phenotype record (T26.F01): a leaf both `creature::body`
//! (inheritance) and `mutation::phenotype` (how it mutates) import.

/// The six-channel internal colour phenotype (HSL-mapped; converted to RGB by
/// `creature::colour::channels_to_rgb` for the wire format) and the state
/// that steers its mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phenotype {
    /// The six channels: hue pair, saturation pair, lightness pair.
    pub channels: [u8; 6],
    /// The channel a mutation steps (0..6).
    pub active_channel: usize,
    /// Per-channel step direction: `true` adds, `false` subtracts.
    pub polarity: [bool; 6],
}

impl Phenotype {
    /// The founder phenotype: channels that render as RGB `[204, 61, 61]`,
    /// channel 0 active, every polarity adding.
    pub const FOUNDER: Self = Self {
        channels: [0, 0, 92, 92, 138, 138],
        active_channel: 0,
        polarity: [true; 6],
    };
}

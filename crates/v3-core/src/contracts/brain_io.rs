//! The brain's non-vote I/O vocabulary: the upstream output slots a mesh
//! node writes and the outcome channels reward-modulated learning reads.

/// The custom output slots a node writes for its downstream node: what an
/// `UpstreamSlot` reads one of and `UpstreamBus` reads all of.
pub const OUTPUT_SLOT_COUNT: usize = 24;

/// Outcome channel for reward-modulated learning.
///
/// Describes **what happened** (not what should matter). Evolution discovers
/// which circuits listen to which signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutcomeChannel {
    EnergyDelta = 0,
    ActionSuccess = 1,
    DamageDelta = 2,
    OffspringSuccess = 3,
}

/// Number of outcome channels.
pub const OUTCOME_CHANNEL_COUNT: usize = 4;
const _: () = {
    assert!(OutcomeChannel::OffspringSuccess as usize + 1 == OUTCOME_CHANNEL_COUNT);
    // `ALL[i]` has discriminant `i`, so the list is complete and ordered.
    let mut i = 0;
    while i < OUTCOME_CHANNEL_COUNT {
        assert!(OutcomeChannel::ALL[i] as usize == i);
        i += 1;
    }
};

impl OutcomeChannel {
    /// Every channel, in discriminant order.
    pub const ALL: [Self; OUTCOME_CHANNEL_COUNT] = [
        Self::EnergyDelta,
        Self::ActionSuccess,
        Self::DamageDelta,
        Self::OffspringSuccess,
    ];
}

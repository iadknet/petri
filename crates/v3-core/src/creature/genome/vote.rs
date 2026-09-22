//! The action-vote catalog (T19.F03).
//!
//! A motor pool wired before it is ever driven. The catalog fixes the identity
//! and index order of every vote sink and of the per-kind parameter slots; the
//! graph sink catalog, the `AddVote` opcode, the mesh vote vector, and the
//! traces all read their positions from here. Nothing in this module carries
//! runtime state, and nothing reads the surface until T19.F04.

/// The four world-action kinds a creature can vote for.
pub const VOTE_KIND_COUNT: usize = 4;
/// Vote sinks in the catalog: one `Eat`, eight each for the three directed
/// kinds, `Terminate`, and `Decide`.
pub const VOTE_SINK_COUNT: usize = 27;
/// Directions a directed vote sink covers — the `Direction::ALL` index range.
pub const VOTE_DIRECTION_COUNT: u8 = 8;
/// Parameter slots per kind, mirroring the shared `[f32; 2]` decode buffer.
pub const VOTE_PARAM_SLOTS: u8 = 2;

const _: () = assert!(VOTE_DIRECTION_COUNT as usize == crate::contracts::Direction::ALL.len());
const _: () = assert!(VOTE_SINK_COUNT == 2 + 1 + 3 * VOTE_DIRECTION_COUNT as usize);

/// One node's contribution, or the mesh's accumulated vote, over the catalog.
pub type VoteVector = [f32; VOTE_SINK_COUNT];

/// A world-action kind that carries votes and parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VoteKind {
    Eat,
    Move,
    Reproduce,
    StealEnergy,
}

impl VoteKind {
    /// Catalog order, which is also the `ActionParam` kind-major order.
    pub const ALL: [Self; VOTE_KIND_COUNT] =
        [Self::Eat, Self::Move, Self::Reproduce, Self::StealEnergy];

    /// Index in `0..VOTE_KIND_COUNT`, the position in the per-kind counters.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Eat => 0,
            Self::Move => 1,
            Self::Reproduce => 2,
            Self::StealEnergy => 3,
        }
    }

    /// The kind at `index`; `None` at or above `VOTE_KIND_COUNT`.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

/// One addressable vote sink. `Terminate` and `Decide` carry no kind: they
/// vote on ending the tick's action pass rather than on a world action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VoteSink {
    Eat,
    /// Directed sink; `u8` is a `Direction::ALL` index in `0..8`.
    Move(u8),
    /// Directed sink; `u8` is a `Direction::ALL` index in `0..8`.
    Reproduce(u8),
    /// Directed sink; `u8` is a `Direction::ALL` index in `0..8`.
    StealEnergy(u8),
    Terminate,
    Decide,
}

impl VoteSink {
    /// Catalog index: `Eat` 0, `Move(d)` `1 + d`, `Reproduce(d)` `9 + d`,
    /// `StealEnergy(d)` `17 + d`, `Terminate` 25, `Decide` 26. A directed sink
    /// built with an out-of-range direction indexes past the catalog and is
    /// never constructed by the catalog itself.
    #[must_use]
    pub fn index(self) -> usize {
        let directions = VOTE_DIRECTION_COUNT as usize;
        match self {
            Self::Eat => 0,
            Self::Move(d) => 1 + d as usize,
            Self::Reproduce(d) => 1 + directions + d as usize,
            Self::StealEnergy(d) => 1 + 2 * directions + d as usize,
            Self::Terminate => VOTE_SINK_COUNT - 2,
            Self::Decide => VOTE_SINK_COUNT - 1,
        }
    }

    /// The sink at `index`; `None` at or above `VOTE_SINK_COUNT`, which is the
    /// soft default of every index read of the catalog.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        let directions = VOTE_DIRECTION_COUNT as usize;
        match index {
            0 => Some(Self::Eat),
            _ if index < 1 + directions => Some(Self::Move((index - 1) as u8)),
            _ if index < 1 + 2 * directions => {
                Some(Self::Reproduce((index - 1 - directions) as u8))
            }
            _ if index < 1 + 3 * directions => {
                Some(Self::StealEnergy((index - 1 - 2 * directions) as u8))
            }
            _ if index == VOTE_SINK_COUNT - 2 => Some(Self::Terminate),
            _ if index == VOTE_SINK_COUNT - 1 => Some(Self::Decide),
            _ => None,
        }
    }

    /// Every sink in catalog index order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..VOTE_SINK_COUNT).filter_map(Self::from_index)
    }

    /// The world-action kind this sink votes for; `None` for the two
    /// pass-control sinks.
    #[must_use]
    pub fn kind(self) -> Option<VoteKind> {
        match self {
            Self::Eat => Some(VoteKind::Eat),
            Self::Move(_) => Some(VoteKind::Move),
            Self::Reproduce(_) => Some(VoteKind::Reproduce),
            Self::StealEnergy(_) => Some(VoteKind::StealEnergy),
            Self::Terminate | Self::Decide => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_indices_are_the_pinned_order() {
        assert_eq!(VoteSink::Eat.index(), 0);
        for d in 0..VOTE_DIRECTION_COUNT {
            assert_eq!(VoteSink::Move(d).index(), 1 + d as usize);
            assert_eq!(VoteSink::Reproduce(d).index(), 9 + d as usize);
            assert_eq!(VoteSink::StealEnergy(d).index(), 17 + d as usize);
        }
        assert_eq!(VoteSink::Terminate.index(), 25);
        assert_eq!(VoteSink::Decide.index(), 26);
    }

    #[test]
    fn from_index_round_trips_the_whole_catalog_and_stops_at_the_count() {
        let all: Vec<VoteSink> = VoteSink::all().collect();
        assert_eq!(all.len(), VOTE_SINK_COUNT);
        for (index, sink) in all.into_iter().enumerate() {
            assert_eq!(sink.index(), index);
            assert_eq!(VoteSink::from_index(index), Some(sink));
        }
        assert_eq!(VoteSink::from_index(VOTE_SINK_COUNT), None);
        assert_eq!(VoteSink::from_index(usize::MAX), None);
    }

    #[test]
    fn kind_is_absent_for_exactly_the_two_pass_control_sinks() {
        let without_kind: Vec<VoteSink> = VoteSink::all()
            .filter(|sink| sink.kind().is_none())
            .collect();
        assert_eq!(without_kind, vec![VoteSink::Terminate, VoteSink::Decide]);
        assert_eq!(VoteSink::Eat.kind(), Some(VoteKind::Eat));
        assert_eq!(VoteSink::Move(7).kind(), Some(VoteKind::Move));
        assert_eq!(VoteSink::Reproduce(0).kind(), Some(VoteKind::Reproduce));
        assert_eq!(VoteSink::StealEnergy(3).kind(), Some(VoteKind::StealEnergy));
    }

    #[test]
    fn vote_kind_indices_match_the_counter_order() {
        for (index, kind) in VoteKind::ALL.into_iter().enumerate() {
            assert_eq!(kind.index(), index);
            assert_eq!(VoteKind::from_index(index), Some(kind));
        }
        assert_eq!(VoteKind::from_index(VOTE_KIND_COUNT), None);
    }

    #[test]
    fn serde_round_trips_every_sink_and_kind() {
        for sink in VoteSink::all() {
            let json = serde_json::to_string(&sink).unwrap();
            assert_eq!(serde_json::from_str::<VoteSink>(&json).unwrap(), sink);
        }
        assert_eq!(serde_json::to_string(&VoteSink::Eat).unwrap(), "\"Eat\"");
        assert_eq!(
            serde_json::to_string(&VoteSink::Move(3)).unwrap(),
            "{\"Move\":3}"
        );
        for kind in VoteKind::ALL {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(serde_json::from_str::<VoteKind>(&json).unwrap(), kind);
        }
    }
}

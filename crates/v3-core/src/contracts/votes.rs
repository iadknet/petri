//! The action-vote catalog (T19.F03).
//!
//! A motor pool wired before it is ever driven. The catalog fixes the identity
//! and index order of every vote sink and of the action-parameter fields; the
//! graph sink catalog, the `AddVote` opcode, the mesh vote vector, and the
//! traces all read their positions from here. Nothing in this module carries
//! runtime state; the pass end commits from the surface (T19.F04).

/// The world-action kinds a creature can vote for: `Eat` and the directed
/// kinds.
pub const VOTE_KIND_COUNT: usize = 1 + DIRECTED_VOTE_KIND_COUNT;
/// Directed kinds: the kinds with eight direction sinks and a drive.
pub const DIRECTED_VOTE_KIND_COUNT: usize = DirectedVoteKind::ALL.len();
/// Vote sinks in the catalog: one `Eat`, eight per directed kind, one drive
/// per directed kind (T11.F30), `Terminate`, and `Decide`.
pub const VOTE_SINK_COUNT: usize =
    3 + (VOTE_DIRECTION_COUNT as usize + 1) * DIRECTED_VOTE_KIND_COUNT;
/// Directions a directed vote sink covers — the `Direction::ALL` index range.
pub const VOTE_DIRECTION_COUNT: u8 = 8;
/// Action-parameter fields: the fields `decode_commit` reads (T11.F27).
pub const ACTION_PARAM_FIELD_COUNT: usize = 5;

/// Catalog index of the first drive sink: the direction sinks end here.
const FIRST_DRIVE_SINK: usize = 1 + VOTE_DIRECTION_COUNT as usize * DIRECTED_VOTE_KIND_COUNT;

const _: () = assert!(VOTE_DIRECTION_COUNT as usize == crate::contracts::Direction::ALL.len());
const _: () = assert!(FIRST_DRIVE_SINK + DIRECTED_VOTE_KIND_COUNT == VOTE_SINK_COUNT - 2);

/// One node's contribution, or the mesh's accumulated vote, over the catalog.
pub type VoteVector = [f32; VOTE_SINK_COUNT];

/// A world-action kind that carries votes and parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VoteKind {
    Eat,
    Move,
    Reproduce,
    StealEnergy,
    GiveEnergy,
}

impl VoteKind {
    /// Catalog order, which is also the per-kind counter order: `Eat`, then
    /// the directed kinds' vote kinds in [`DirectedVoteKind::ALL`] order.
    pub const ALL: [Self; VOTE_KIND_COUNT] = [
        Self::Eat,
        Self::Move,
        Self::Reproduce,
        Self::StealEnergy,
        Self::GiveEnergy,
    ];

    /// Index in `0..VOTE_KIND_COUNT`, the position in the per-kind counters.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Eat => 0,
            Self::Move => 1,
            Self::Reproduce => 2,
            Self::StealEnergy => 3,
            Self::GiveEnergy => 4,
        }
    }

    /// The kind at `index`; `None` at or above `VOTE_KIND_COUNT`.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

/// One action-parameter field, a value the commit decoder (`decode_commit`)
/// reads (T11.F27). The catalog holds exactly the decoded fields, in
/// kind-major order, so no Graph sink, VM `WriteActionParam` field index or
/// surface cell names a field the body never reads. A feature that decodes
/// another field extends this catalog and the decoder together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ActionParamField {
    /// The food type an `Eat` commit targets.
    EatFoodType,
    /// The share of the cell's food an `Eat` commit leaves on the cell, in
    /// `[0, 1]` (T02.F08); 0.0 eats it all.
    EatLeaveShare,
    /// The energy fraction a `Reproduce` commit transfers to the child.
    ReproduceTransferFraction,
    /// A `StealEnergy` commit's effort modifier (T27.F03): a field value
    /// `x` scales the attacker's `steal_share` by `2^clamp(x, −2, 2)`.
    StealEffort,
    /// A `GiveEnergy` commit's effort modifier (T27.F03): a field value `x`
    /// scales the giver's `give_share` by `2^clamp(x, −2, 2)`.
    GiveEffort,
}

impl ActionParamField {
    /// Catalog order: the Graph sink order, the VM `field_idx` addressing,
    /// and the parameter-surface index.
    pub const ALL: [Self; ACTION_PARAM_FIELD_COUNT] = [
        Self::EatFoodType,
        Self::EatLeaveShare,
        Self::ReproduceTransferFraction,
        Self::StealEffort,
        Self::GiveEffort,
    ];

    /// Position in [`Self::ALL`], in `0..ACTION_PARAM_FIELD_COUNT`.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::EatFoodType => 0,
            Self::EatLeaveShare => 1,
            Self::ReproduceTransferFraction => 2,
            Self::StealEffort => 3,
            Self::GiveEffort => 4,
        }
    }

    /// The world-action kind whose commit reads this field.
    #[must_use]
    pub const fn kind(self) -> VoteKind {
        match self {
            Self::EatFoodType | Self::EatLeaveShare => VoteKind::Eat,
            Self::ReproduceTransferFraction => VoteKind::Reproduce,
            Self::StealEffort => VoteKind::StealEnergy,
            Self::GiveEffort => VoteKind::GiveEnergy,
        }
    }
}

/// One addressable vote sink. `Terminate` and `Decide` carry no kind: they
/// vote on ending the tick's action pass rather than on a world action. A
/// `Drive` (T11.F30) votes for its kind without naming a direction: the pass
/// end adds it to the kind's best direction vote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VoteSink {
    Eat,
    /// A directed kind's sink; `u8` is a `Direction::ALL` index in `0..8`.
    Directed(DirectedVoteKind, u8),
    /// The kind's drive: whether to act, apart from where (T11.F30).
    Drive(DirectedVoteKind),
    Terminate,
    Decide,
}

impl VoteSink {
    /// Catalog index: `Eat` 0, `Directed(K, d)` `1 + 8·K.index() + d`,
    /// `Drive(K)` `1 + 8·N + K.index()` for `N` directed kinds, `Terminate`
    /// `VOTE_SINK_COUNT − 2`, `Decide` `VOTE_SINK_COUNT − 1`. A directed sink
    /// built with an out-of-range direction indexes past its kind and is
    /// never constructed by the catalog itself.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Eat => 0,
            Self::Directed(kind, d) => {
                1 + VOTE_DIRECTION_COUNT as usize * kind.index() + d as usize
            }
            Self::Drive(kind) => FIRST_DRIVE_SINK + kind.index(),
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
            _ if index < FIRST_DRIVE_SINK => Some(Self::Directed(
                DirectedVoteKind::ALL[(index - 1) / directions],
                ((index - 1) % directions) as u8,
            )),
            _ if index < FIRST_DRIVE_SINK + DIRECTED_VOTE_KIND_COUNT => {
                Some(Self::Drive(DirectedVoteKind::ALL[index - FIRST_DRIVE_SINK]))
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
            Self::Directed(kind, _) | Self::Drive(kind) => Some(kind.vote_kind()),
            Self::Terminate | Self::Decide => None,
        }
    }
}

/// A directed world-action kind: the kinds with eight direction sinks and a
/// drive, which a `Ring` edge bundle (T25.F03) votes for. A new kind is
/// appended to [`Self::ALL`], and the index rule of [`VoteSink::index`]
/// places its sinks.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DirectedVoteKind {
    Move,
    Reproduce,
    StealEnergy,
    /// Pass part of the giver's own store to the neighbour (T27.F03).
    GiveEnergy,
}

impl DirectedVoteKind {
    /// Catalog order, the order `AddRingBundle` and `RetargetBundleKind` draw
    /// over.
    pub const ALL: [Self; 4] = [
        Self::Move,
        Self::Reproduce,
        Self::StealEnergy,
        Self::GiveEnergy,
    ];

    /// Position in [`Self::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Move => 0,
            Self::Reproduce => 1,
            Self::StealEnergy => 2,
            Self::GiveEnergy => 3,
        }
    }

    /// The direction-`d` sink of this kind; `d` is a `Direction::ALL` index.
    #[must_use]
    pub const fn sink(self, d: u8) -> VoteSink {
        VoteSink::Directed(self, d)
    }

    /// The kind and direction of a directed sink; `None` for any other sink,
    /// a drive included (it names no direction).
    #[must_use]
    pub const fn of_sink(sink: VoteSink) -> Option<(Self, u8)> {
        match sink {
            VoteSink::Directed(kind, d) => Some((kind, d)),
            VoteSink::Eat | VoteSink::Drive(_) | VoteSink::Terminate | VoteSink::Decide => None,
        }
    }

    /// The kind's drive sink (T11.F30).
    #[must_use]
    pub const fn drive(self) -> VoteSink {
        VoteSink::Drive(self)
    }

    /// The world-action kind.
    #[must_use]
    pub const fn vote_kind(self) -> VoteKind {
        match self {
            Self::Move => VoteKind::Move,
            Self::Reproduce => VoteKind::Reproduce,
            Self::StealEnergy => VoteKind::StealEnergy,
            Self::GiveEnergy => VoteKind::GiveEnergy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directed_kinds_map_onto_their_sinks() {
        for (index, kind) in DirectedVoteKind::ALL.into_iter().enumerate() {
            assert_eq!(kind.index(), index);
            for d in 0..VOTE_DIRECTION_COUNT {
                let sink = kind.sink(d);
                assert_eq!(sink.index(), 1 + index * 8 + usize::from(d));
                assert_eq!(DirectedVoteKind::of_sink(sink), Some((kind, d)));
                assert_eq!(sink.kind(), Some(kind.vote_kind()));
            }
        }
        for sink in [VoteSink::Eat, VoteSink::Terminate, VoteSink::Decide] {
            assert_eq!(DirectedVoteKind::of_sink(sink), None);
        }
        for kind in DirectedVoteKind::ALL {
            assert_eq!(DirectedVoteKind::of_sink(kind.drive()), None);
            assert_eq!(kind.drive().kind(), Some(kind.vote_kind()));
        }
    }

    #[test]
    fn action_param_fields_are_the_decoded_catalog_in_kind_major_order() {
        assert_eq!(
            ActionParamField::ALL,
            [
                ActionParamField::EatFoodType,
                ActionParamField::EatLeaveShare,
                ActionParamField::ReproduceTransferFraction,
                ActionParamField::StealEffort,
                ActionParamField::GiveEffort,
            ]
        );
        for (index, field) in ActionParamField::ALL.into_iter().enumerate() {
            assert_eq!(field.index(), index);
        }
        let kinds = ActionParamField::ALL.map(ActionParamField::kind);
        assert_eq!(
            kinds,
            [
                VoteKind::Eat,
                VoteKind::Eat,
                VoteKind::Reproduce,
                VoteKind::StealEnergy,
                VoteKind::GiveEnergy
            ]
        );
        for field in ActionParamField::ALL {
            let json = serde_json::to_string(&field).unwrap();
            assert_eq!(json, format!("\"{field:?}\""));
            assert_eq!(
                serde_json::from_str::<ActionParamField>(&json).unwrap(),
                field
            );
        }
    }

    #[test]
    fn catalog_indices_are_the_pinned_order() {
        assert_eq!(VoteSink::Eat.index(), 0);
        for d in 0..VOTE_DIRECTION_COUNT {
            assert_eq!(
                VoteSink::Directed(DirectedVoteKind::Move, d).index(),
                1 + d as usize
            );
            assert_eq!(
                VoteSink::Directed(DirectedVoteKind::Reproduce, d).index(),
                9 + d as usize
            );
            assert_eq!(
                VoteSink::Directed(DirectedVoteKind::StealEnergy, d).index(),
                17 + d as usize
            );
            assert_eq!(
                VoteSink::Directed(DirectedVoteKind::GiveEnergy, d).index(),
                25 + d as usize
            );
        }
        assert_eq!(VoteSink::Drive(DirectedVoteKind::Move).index(), 33);
        assert_eq!(VoteSink::Drive(DirectedVoteKind::Reproduce).index(), 34);
        assert_eq!(VoteSink::Drive(DirectedVoteKind::StealEnergy).index(), 35);
        assert_eq!(VoteSink::Drive(DirectedVoteKind::GiveEnergy).index(), 36);
        assert_eq!(VoteSink::Terminate.index(), 37);
        assert_eq!(VoteSink::Decide.index(), 38);
        assert_eq!(VOTE_SINK_COUNT, 39);
        assert_eq!(VOTE_KIND_COUNT, 5);
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
        assert_eq!(
            VoteSink::Directed(DirectedVoteKind::Move, 7).kind(),
            Some(VoteKind::Move)
        );
        assert_eq!(
            VoteSink::Directed(DirectedVoteKind::Reproduce, 0).kind(),
            Some(VoteKind::Reproduce)
        );
        assert_eq!(
            VoteSink::Directed(DirectedVoteKind::StealEnergy, 3).kind(),
            Some(VoteKind::StealEnergy)
        );
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
    fn vote_kinds_are_eat_then_the_directed_kinds_in_catalog_order() {
        let mut expected = vec![VoteKind::Eat];
        expected.extend(DirectedVoteKind::ALL.map(DirectedVoteKind::vote_kind));
        assert_eq!(VoteKind::ALL.to_vec(), expected);
        for kind in DirectedVoteKind::ALL {
            assert_eq!(kind.vote_kind().index(), 1 + kind.index());
        }
    }

    /// The index rule of invariant 1 holds for every directed kind, so a new
    /// kind appended to `DirectedVoteKind::ALL` needs no index arithmetic.
    #[test]
    fn the_catalog_follows_the_directed_index_rule() {
        let n = DirectedVoteKind::ALL.len();
        assert_eq!(VOTE_SINK_COUNT, 3 + 9 * n);
        for kind in DirectedVoteKind::ALL {
            for d in 0..VOTE_DIRECTION_COUNT {
                assert_eq!(kind.sink(d).index(), 1 + 8 * kind.index() + usize::from(d));
            }
            assert_eq!(kind.drive().index(), 1 + 8 * n + kind.index());
        }
        assert_eq!(VoteSink::Terminate.index(), VOTE_SINK_COUNT - 2);
        assert_eq!(VoteSink::Decide.index(), VOTE_SINK_COUNT - 1);
    }

    /// The frontend's `vote-sinks.json` is the declared catalog (T27.F03
    /// invariant 2), as `nearbyFields.json` is the nearby fields.
    #[test]
    fn the_frontend_fixture_equals_the_declared_catalog() {
        use crate::contracts::ActionType;
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../frontend/src/types/vote-sinks.json"
        ))
        .expect("fixture parses");
        let action_types: Vec<_> = ActionType::ALL
            .iter()
            .map(|&t| serde_json::json!({ "name": t.as_key(), "code": t as u8 }))
            .collect();
        let fields: Vec<_> = ActionParamField::ALL
            .iter()
            .map(|&f| serde_json::json!({ "field": f, "kind": f.kind() }))
            .collect();
        let declared = serde_json::json!({
            "voteKinds": VoteKind::ALL,
            "directedKinds": DirectedVoteKind::ALL,
            "voteSinks": VoteSink::all().collect::<Vec<_>>(),
            "actionParamFields": fields,
            "actionTypes": action_types,
        });
        assert_eq!(fixture, declared);
    }

    #[test]
    fn serde_round_trips_every_sink_and_kind() {
        for sink in VoteSink::all() {
            let json = serde_json::to_string(&sink).unwrap();
            assert_eq!(serde_json::from_str::<VoteSink>(&json).unwrap(), sink);
        }
        assert_eq!(serde_json::to_string(&VoteSink::Eat).unwrap(), "\"Eat\"");
        assert_eq!(
            serde_json::to_string(&VoteSink::Directed(DirectedVoteKind::Move, 3)).unwrap(),
            "{\"Directed\":[\"Move\",3]}"
        );
        assert_eq!(
            serde_json::to_string(&VoteSink::Drive(DirectedVoteKind::Move)).unwrap(),
            "{\"Drive\":\"Move\"}"
        );
        for kind in VoteKind::ALL {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(serde_json::from_str::<VoteKind>(&json).unwrap(), kind);
        }
    }
}

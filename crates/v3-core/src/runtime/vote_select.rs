//! The pass-end commit rule (T19.F04): within-kind argmax with the heading
//! tie (T11.F30), the kind's drive, per-kind rising bar, and the kind argmax
//! with the incumbent tie-break.
//!
//! Pure and RNG-free: the mesh loop calls [`select`] at every node boundary
//! for the `Decide` guard and once at every pass end for the commit.

use crate::contracts::Direction;
use crate::creature::genome::vote::{
    DirectedVoteKind, VoteKind, VoteSink, VoteVector, VOTE_DIRECTION_COUNT, VOTE_KIND_COUNT,
};

/// The bar one commit adds to its kind, equal to the vote scale.
pub const VOTE_UNIT: f32 = 1.0;

/// The pass-end reading of one vote vector against the per-kind bars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    /// Argmax sink of each kind, in `VoteKind` index order: `Eat` itself,
    /// and for a directed kind the first maximal direction sink visited
    /// clockwise from the heading, the heading first. Never a drive.
    pub best: [VoteSink; VOTE_KIND_COUNT],
    /// Effective vote per kind: `(votes[Drive(K)] + votes[best[K]]) -
    /// bars[K] * VOTE_UNIT` for a directed kind, `votes[Eat] - bars[Eat] *
    /// VOTE_UNIT` for `Eat`.
    pub effective: [f32; VOTE_KIND_COUNT],
    /// The kind with the largest effective vote when that vote is positive;
    /// ties go to `previous`, then to the lowest kind index.
    pub winner: Option<VoteKind>,
}

impl Selection {
    /// The winning sink and its effective vote, when some kind is positive.
    #[must_use]
    pub fn committed(&self) -> Option<(VoteSink, f32)> {
        self.winner
            .map(|kind| (self.best[kind.index()], self.effective[kind.index()]))
    }
}

/// The first maximal direction sink of `kind`, visiting the eight in the
/// order `(h + k) mod 8` for `k` in `0..8`, where `h` is the heading's
/// `Direction::ALL` index (T11.F30). Under heading `N` this is the
/// lowest-index argmax.
fn best_direction(votes: &VoteVector, kind: DirectedVoteKind, heading: Direction) -> VoteSink {
    let h = heading.to_index();
    let mut top = kind.sink(h as u8);
    for k in 1..usize::from(VOTE_DIRECTION_COUNT) {
        let sink = kind.sink(((h + k) % usize::from(VOTE_DIRECTION_COUNT)) as u8);
        if votes[sink.index()] > votes[top.index()] {
            top = sink;
        }
    }
    top
}

/// Read `votes` (already sanitized) against `bars` (commits per kind this
/// tick). `previous` is the kind committed in the previous pass; `heading`
/// breaks ties within a directed kind.
#[must_use]
pub fn select(
    votes: &VoteVector,
    bars: &[u32; VOTE_KIND_COUNT],
    previous: Option<VoteKind>,
    heading: Direction,
) -> Selection {
    let mut best = [VoteSink::Eat; VOTE_KIND_COUNT];
    let mut effective = [0.0f32; VOTE_KIND_COUNT];
    let bar = |kind: VoteKind| bars[kind.index()] as f32 * VOTE_UNIT;
    effective[VoteKind::Eat.index()] = votes[VoteSink::Eat.index()] - bar(VoteKind::Eat);
    for kind in DirectedVoteKind::ALL {
        let top = best_direction(votes, kind, heading);
        let vote_kind = kind.vote_kind();
        best[vote_kind.index()] = top;
        // With no drive written, `0.0 + x` is `x` exactly (T11.F30).
        effective[vote_kind.index()] =
            (votes[kind.drive().index()] + votes[top.index()]) - bar(vote_kind);
    }
    let mut leader = VoteKind::Eat;
    for kind in VoteKind::ALL {
        if effective[kind.index()] > effective[leader.index()] {
            leader = kind;
        }
    }
    if let Some(incumbent) = previous {
        if effective[incumbent.index()] == effective[leader.index()] {
            leader = incumbent;
        }
    }
    let winner = (effective[leader.index()] > 0.0).then_some(leader);
    Selection {
        best,
        effective,
        winner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::vote::VOTE_SINK_COUNT;
    use proptest::prelude::*;

    const N: Direction = Direction::N;

    fn votes_with(entries: &[(VoteSink, f32)]) -> VoteVector {
        let mut votes = [0.0; VOTE_SINK_COUNT];
        for (sink, value) in entries {
            votes[sink.index()] = *value;
        }
        votes
    }

    #[test]
    fn founder_forage_pass_one_commits_eat_over_the_best_move() {
        let votes = votes_with(&[
            (VoteSink::Eat, 1.0),
            (VoteSink::Directed(DirectedVoteKind::Move, 0), 0.58),
            (VoteSink::Directed(DirectedVoteKind::Move, 2), 0.86),
            (VoteSink::Directed(DirectedVoteKind::Move, 4), 0.54),
            (VoteSink::Directed(DirectedVoteKind::Move, 6), 0.66),
        ]);
        let selection = select(&votes, &[0; VOTE_KIND_COUNT], None, N);
        assert_eq!(selection.committed(), Some((VoteSink::Eat, 1.0)));
        assert_eq!(
            selection.best[VoteKind::Move.index()],
            VoteSink::Directed(DirectedVoteKind::Move, 2)
        );
        let second = select(&votes, &[1, 0, 0, 0, 0], Some(VoteKind::Eat), N);
        assert_eq!(second.winner, Some(VoteKind::Move));
        assert_eq!(
            second.best[VoteKind::Move.index()],
            VoteSink::Directed(DirectedVoteKind::Move, 2)
        );
    }

    #[test]
    fn a_kind_tie_goes_to_the_previous_kind_then_the_lowest_index() {
        let votes = votes_with(&[
            (VoteSink::Eat, 1.0),
            (VoteSink::Directed(DirectedVoteKind::Move, 2), 1.0),
        ]);
        assert_eq!(
            select(&votes, &[0; VOTE_KIND_COUNT], None, N).winner,
            Some(VoteKind::Eat)
        );
        assert_eq!(
            select(&votes, &[0; VOTE_KIND_COUNT], Some(VoteKind::Move), N).winner,
            Some(VoteKind::Move)
        );
        // An incumbent that is not tied at the top does not win.
        assert_eq!(
            select(&votes, &[0, 1, 0, 0, 0], Some(VoteKind::Move), N).winner,
            Some(VoteKind::Eat)
        );
    }

    #[test]
    fn nothing_positive_selects_nothing() {
        let votes = votes_with(&[(VoteSink::Terminate, 5.0), (VoteSink::Decide, 5.0)]);
        assert_eq!(select(&votes, &[0; VOTE_KIND_COUNT], None, N).winner, None);
        let votes = votes_with(&[(VoteSink::Directed(DirectedVoteKind::Move, 1), 1.0)]);
        assert_eq!(select(&votes, &[0, 1, 0, 0, 0], None, N).winner, None);
    }

    #[test]
    fn a_flat_channel_with_a_positive_drive_commits_the_heading() {
        for kind in DirectedVoteKind::ALL {
            let votes = votes_with(&[(kind.drive(), 0.5)]);
            for (h, heading) in Direction::ALL.into_iter().enumerate() {
                let selection = select(&votes, &[0; VOTE_KIND_COUNT], None, heading);
                assert_eq!(selection.committed(), Some((kind.sink(h as u8), 0.5)));
            }
        }
    }

    #[test]
    fn a_tie_goes_to_the_first_tied_sink_clockwise_from_the_heading() {
        // Tied at N (0) and SE (3): from NE the walk meets SE first; from SW
        // it wraps to N.
        let votes = votes_with(&[
            (VoteSink::Directed(DirectedVoteKind::Move, 0), 1.0),
            (VoteSink::Directed(DirectedVoteKind::Move, 3), 1.0),
        ]);
        let best = |heading| {
            select(&votes, &[0; VOTE_KIND_COUNT], None, heading).best[VoteKind::Move.index()]
        };
        assert_eq!(
            best(Direction::NE),
            VoteSink::Directed(DirectedVoteKind::Move, 3)
        );
        assert_eq!(
            best(Direction::SE),
            VoteSink::Directed(DirectedVoteKind::Move, 3)
        );
        assert_eq!(
            best(Direction::SW),
            VoteSink::Directed(DirectedVoteKind::Move, 0)
        );
        assert_eq!(
            best(Direction::N),
            VoteSink::Directed(DirectedVoteKind::Move, 0)
        );
        // A strict maximum wins whatever the heading.
        let votes = votes_with(&[
            (VoteSink::Directed(DirectedVoteKind::Move, 0), 1.0),
            (VoteSink::Directed(DirectedVoteKind::Move, 3), 2.0),
        ]);
        assert_eq!(
            select(&votes, &[0; VOTE_KIND_COUNT], None, Direction::NW).best[VoteKind::Move.index()],
            VoteSink::Directed(DirectedVoteKind::Move, 3)
        );
    }

    #[test]
    fn the_drive_adds_to_the_best_direction_and_is_never_best() {
        let votes = votes_with(&[
            (VoteSink::Directed(DirectedVoteKind::Move, 5), 0.4),
            (VoteSink::Drive(DirectedVoteKind::Move), 0.7),
            (VoteSink::Eat, 1.0),
        ]);
        let selection = select(&votes, &[0; VOTE_KIND_COUNT], None, N);
        assert_eq!(
            selection.best[VoteKind::Move.index()],
            VoteSink::Directed(DirectedVoteKind::Move, 5)
        );
        assert_eq!(selection.effective[VoteKind::Move.index()], 0.7 + 0.4);
        assert_eq!(selection.winner, Some(VoteKind::Move));
        // A negative drive suppresses the kind.
        let votes = votes_with(&[
            (VoteSink::Directed(DirectedVoteKind::Reproduce, 2), 1.0),
            (VoteSink::Drive(DirectedVoteKind::Reproduce), -1.5),
        ]);
        assert_eq!(select(&votes, &[0; VOTE_KIND_COUNT], None, N).winner, None);
        // `Eat` has no drive: another kind's drive never reaches it.
        let votes = votes_with(&[(VoteSink::Drive(DirectedVoteKind::StealEnergy), 3.0)]);
        let selection = select(&votes, &[0; VOTE_KIND_COUNT], None, N);
        assert_eq!(selection.effective[VoteKind::Eat.index()], 0.0);
        assert_eq!(selection.winner, Some(VoteKind::StealEnergy));
    }

    fn any_vote() -> impl Strategy<Value = f32> {
        prop_oneof![
            4 => -3.0f32..3.0,
            1 => (-3i8..=3).prop_map(f32::from),
        ]
    }

    proptest! {
        /// The selection is exactly the definition: `Eat`'s best sink is
        /// itself; a directed kind's is the first maximal direction sink
        /// clockwise from the heading, the heading first, never its drive;
        /// the effective vote is the drive plus that vote minus the bar
        /// (`Eat`: its vote minus its bar); and a winner exists iff the
        /// largest effective vote is positive, is that largest, and is the
        /// incumbent or the lowest index among the tied.
        #[test]
        fn selection_matches_the_commit_rule_definition(
            votes in prop::collection::vec(any_vote(), VOTE_SINK_COUNT),
            bars in prop::array::uniform5(0u32..4),
            previous in prop::option::of(0usize..VOTE_KIND_COUNT),
            heading in 0usize..8,
        ) {
            let votes: VoteVector = votes.try_into().unwrap();
            let previous = previous.and_then(VoteKind::from_index);
            let heading = Direction::ALL[heading];
            let selection = select(&votes, &bars, previous, heading);
            prop_assert_eq!(selection.best[VoteKind::Eat.index()], VoteSink::Eat);
            prop_assert_eq!(
                selection.effective[VoteKind::Eat.index()],
                votes[VoteSink::Eat.index()] - bars[VoteKind::Eat.index()] as f32
            );
            for kind in DirectedVoteKind::ALL {
                let order: Vec<VoteSink> = (0..8)
                    .map(|k| kind.sink(((heading.to_index() + k) % 8) as u8))
                    .collect();
                let max = order.iter().map(|s| votes[s.index()]).fold(f32::NEG_INFINITY, f32::max);
                let first = order.iter().find(|s| votes[s.index()] == max).copied().unwrap();
                let k = kind.vote_kind().index();
                prop_assert_eq!(selection.best[k], first);
                prop_assert_eq!(
                    selection.effective[k],
                    (votes[kind.drive().index()] + max) - bars[k] as f32
                );
            }
            let top = selection.effective.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            match selection.winner {
                None => prop_assert!(top <= 0.0),
                Some(kind) => {
                    prop_assert!(top > 0.0);
                    prop_assert_eq!(selection.effective[kind.index()], top);
                    let lowest = VoteKind::ALL.into_iter().find(|k| selection.effective[k.index()] == top).unwrap();
                    let expected = previous.filter(|p| selection.effective[p.index()] == top).unwrap_or(lowest);
                    prop_assert_eq!(kind, expected);
                }
            }
        }

        /// With no drive written and heading `N`, the selection is the
        /// pre-drive rule: the lowest-index argmax of each kind, and its vote
        /// minus its bar.
        #[test]
        fn an_unwired_drive_under_north_is_the_lowest_index_rule(
            votes in prop::collection::vec(any_vote(), VOTE_SINK_COUNT),
            bars in prop::array::uniform5(0u32..4),
            previous in prop::option::of(0usize..VOTE_KIND_COUNT),
        ) {
            let mut votes: VoteVector = votes.try_into().unwrap();
            for kind in DirectedVoteKind::ALL {
                votes[kind.drive().index()] = 0.0;
            }
            let previous = previous.and_then(VoteKind::from_index);
            let selection = select(&votes, &bars, previous, N);
            for kind in VoteKind::ALL {
                let sinks: Vec<VoteSink> = VoteSink::all()
                    .filter(|s| s.kind() == Some(kind) && !matches!(s, VoteSink::Drive(_)))
                    .collect();
                let max = sinks.iter().map(|s| votes[s.index()]).fold(f32::NEG_INFINITY, f32::max);
                let first = sinks.iter().find(|s| votes[s.index()] == max).copied().unwrap();
                prop_assert_eq!(selection.best[kind.index()], first);
                prop_assert_eq!(
                    selection.effective[kind.index()],
                    votes[first.index()] - bars[kind.index()] as f32
                );
            }
        }
    }
}

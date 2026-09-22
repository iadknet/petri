//! The pass-end commit rule (T19.F04): within-kind argmax, per-kind rising
//! bar, and the kind argmax with the incumbent tie-break.
//!
//! Pure and RNG-free: the mesh loop calls [`select`] at every node boundary
//! for the `Decide` guard and once at every pass end for the commit.

use crate::creature::genome::vote::{VoteKind, VoteSink, VoteVector, VOTE_KIND_COUNT};

/// The bar one commit adds to its kind, equal to the vote scale.
pub const VOTE_UNIT: f32 = 1.0;

/// The pass-end reading of one vote vector against the per-kind bars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    /// Lowest-index argmax sink of each kind, in `VoteKind` index order.
    pub best: [VoteSink; VOTE_KIND_COUNT],
    /// Effective vote per kind: `votes[best[K]] - bars[K] * VOTE_UNIT`.
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

/// The sinks of `kind` in catalog index order.
fn kind_sinks(kind: VoteKind) -> impl Iterator<Item = VoteSink> {
    VoteSink::all().filter(move |sink| sink.kind() == Some(kind))
}

/// Read `votes` (already sanitized) against `bars` (commits per kind this
/// tick). `previous` is the kind committed in the previous pass.
#[must_use]
pub fn select(
    votes: &VoteVector,
    bars: &[u32; VOTE_KIND_COUNT],
    previous: Option<VoteKind>,
) -> Selection {
    let mut best = [VoteSink::Eat; VOTE_KIND_COUNT];
    let mut effective = [0.0f32; VOTE_KIND_COUNT];
    for kind in VoteKind::ALL {
        let mut sinks = kind_sinks(kind);
        let mut top = sinks.next().expect("every kind has a sink");
        for sink in sinks {
            if votes[sink.index()] > votes[top.index()] {
                top = sink;
            }
        }
        best[kind.index()] = top;
        effective[kind.index()] = votes[top.index()] - bars[kind.index()] as f32 * VOTE_UNIT;
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
            (VoteSink::Move(0), 0.58),
            (VoteSink::Move(2), 0.86),
            (VoteSink::Move(4), 0.54),
            (VoteSink::Move(6), 0.66),
        ]);
        let selection = select(&votes, &[0; 4], None);
        assert_eq!(selection.committed(), Some((VoteSink::Eat, 1.0)));
        assert_eq!(selection.best[VoteKind::Move.index()], VoteSink::Move(2));
        let second = select(&votes, &[1, 0, 0, 0], Some(VoteKind::Eat));
        assert_eq!(second.winner, Some(VoteKind::Move));
        assert_eq!(second.best[VoteKind::Move.index()], VoteSink::Move(2));
    }

    #[test]
    fn a_kind_tie_goes_to_the_previous_kind_then_the_lowest_index() {
        let votes = votes_with(&[(VoteSink::Eat, 1.0), (VoteSink::Move(2), 1.0)]);
        assert_eq!(select(&votes, &[0; 4], None).winner, Some(VoteKind::Eat));
        assert_eq!(
            select(&votes, &[0; 4], Some(VoteKind::Move)).winner,
            Some(VoteKind::Move)
        );
        // An incumbent that is not tied at the top does not win.
        assert_eq!(
            select(&votes, &[0, 1, 0, 0], Some(VoteKind::Move)).winner,
            Some(VoteKind::Eat)
        );
    }

    #[test]
    fn nothing_positive_selects_nothing() {
        let votes = votes_with(&[(VoteSink::Terminate, 5.0), (VoteSink::Decide, 5.0)]);
        assert_eq!(select(&votes, &[0; 4], None).winner, None);
        let votes = votes_with(&[(VoteSink::Move(1), 1.0)]);
        assert_eq!(select(&votes, &[0, 1, 0, 0], None).winner, None);
    }

    fn any_vote() -> impl Strategy<Value = f32> {
        prop_oneof![
            4 => -3.0f32..3.0,
            1 => (-3i8..=3).prop_map(f32::from),
        ]
    }

    proptest! {
        /// The selection is exactly the definition: each kind's best sink is
        /// its lowest-index argmax, its effective vote is that vote minus its
        /// bar, and a winner exists iff the largest effective vote is
        /// positive, is that largest, and is the incumbent or the lowest
        /// index among the tied.
        #[test]
        fn selection_matches_the_commit_rule_definition(
            votes in prop::array::uniform27(any_vote()),
            bars in prop::array::uniform4(0u32..4),
            previous in prop::option::of(0usize..VOTE_KIND_COUNT),
        ) {
            let previous = previous.and_then(VoteKind::from_index);
            let selection = select(&votes, &bars, previous);
            for kind in VoteKind::ALL {
                let sinks: Vec<VoteSink> = VoteSink::all().filter(|s| s.kind() == Some(kind)).collect();
                let max = sinks.iter().map(|s| votes[s.index()]).fold(f32::NEG_INFINITY, f32::max);
                let first = sinks.iter().find(|s| votes[s.index()] == max).copied().unwrap();
                prop_assert_eq!(selection.best[kind.index()], first);
                prop_assert_eq!(selection.effective[kind.index()], max - bars[kind.index()] as f32);
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
    }
}

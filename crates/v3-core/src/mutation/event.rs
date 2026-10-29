//! Event-level result types shared by the mutation domains and the engine
//! (T30.F10): the leaf the domain modules import, so `types` can name their
//! operator enums without an import cycle.

use crate::contracts::votes::VoteSink;
use crate::genome::cgp::Wiring;
use crate::genome::learning::InheritanceMapping;

/// How far a learning edit reached (T25.F12 invariant 9): its one picked
/// target, or every member of the picked member's bundle the operator's
/// predicate admits (the broadcast coin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Broadcast {
    One,
    Members,
}

impl Broadcast {
    /// Both reaches, in index order.
    pub const ALL: [Self; 2] = [Self::One, Self::Members];

    /// This reach's position in [`Self::ALL`].
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Members => 1,
        }
    }

    /// Stable string key used by telemetry and bench boundaries.
    #[inline]
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Members => "members",
        }
    }
}

/// Whether a mutation operator increases, decreases, or preserves genome complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityEffect {
    Increasing,
    Decreasing,
    Neutral,
}

impl ComplexityEffect {
    /// Returns true if this effect is `Decreasing`.
    #[must_use]
    pub const fn is_decreasing(self) -> bool {
        matches!(self, Self::Decreasing)
    }
}

/// What one committed event did to bundles (T25.F11), reported by the
/// operator from the container's return values: the bundles it created by
/// wiring (`plain`, `ring`, `pair`) and creation form (`uniform`,
/// `randomized`), the members it detached and the records it removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BundleEdits {
    pub created: [[u8; 2]; 3],
    pub detached: u8,
    pub removed: u8,
}

impl BundleEdits {
    /// One member detach, which removed its record when it was the last.
    #[must_use]
    pub const fn detach(removed: bool) -> Self {
        Self {
            created: [[0; 2]; 3],
            detached: 1,
            removed: removed as u8,
        }
    }

    /// A removal that took a record when `removed`.
    #[must_use]
    pub const fn removal(removed: bool) -> Self {
        Self {
            created: [[0; 2]; 3],
            detached: 0,
            removed: removed as u8,
        }
    }

    /// One bundle created with `wiring` in `form` (a test expectation).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn creation(wiring: &Wiring, randomized: bool) -> Self {
        let mut edits = Self::default();
        edits.created[wiring_index(wiring)][usize::from(randomized)] = 1;
        edits
    }
}

/// The report keys of [`BundleEdits::created`]'s rows, by [`wiring_index`].
pub const WIRING_KEYS: [&str; 3] = ["plain", "ring", "pair"];

/// The report keys of [`BundleEdits::created`]'s columns: the creation
/// form, uniform (one learning draw for the bundle) or randomized (one per
/// member).
pub const FORM_KEYS: [&str; 2] = ["uniform", "randomized"];

/// A wiring's row in [`BundleEdits::created`]: `plain`, `ring`, `pair`.
#[must_use]
pub const fn wiring_index(wiring: &Wiring) -> usize {
    match wiring {
        Wiring::Plain { .. } => 0,
        Wiring::Ring { .. } => 1,
        Wiring::DirectionPair { .. } => 2,
    }
}

/// Reason a mutation event was skipped.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MutationSkipReason {
    ParseabilityViolation,
    NumericProposalRejected,
    NoApplicableTarget,
    /// The event left a Graph backend breaking a graph invariant
    /// (`validate_graph`, T25.F03); the draft was rolled back.
    GraphInvariantViolation,
}

impl MutationSkipReason {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::ParseabilityViolation => "ParseabilityViolation",
            Self::NumericProposalRejected => "NumericProposalRejected",
            Self::NoApplicableTarget => "NoApplicableTarget",
            Self::GraphInvariantViolation => "GraphInvariantViolation",
        }
    }
}

/// The class of the vote sink a born edge lands on (T11.F30): a kind's
/// `Drive` or one of its direction sinks. `AddGraphEdge` and `InputRef.Add`
/// report it at the push; the `drive_births` reading keys carrier outcomes
/// by it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum BirthEdgeClass {
    Drive,
    Direction,
}

impl BirthEdgeClass {
    pub const ALL: [Self; 2] = [Self::Drive, Self::Direction];

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Direction => "direction",
        }
    }

    /// The class of `sink`: `Drive` for a drive, `Direction` for a directed
    /// kind's direction sink, `None` for `Eat`, `Terminate` and `Decide`.
    #[must_use]
    pub const fn of_sink(sink: VoteSink) -> Option<Self> {
        match sink {
            VoteSink::Drive(_) => Some(Self::Drive),
            VoteSink::Directed(..) => Some(Self::Direction),
            VoteSink::Eat | VoteSink::Terminate | VoteSink::Decide => None,
        }
    }
}

/// What one committed event reports beside its reachability (T30.F10): the
/// class of the vote-sink edge an `AddGraphEdge` or a creation pushed
/// (T11.F30, T25.F11), how far a learning edit reached (T25.F12) and what
/// the event did to bundles (T25.F11).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventReport {
    /// The class of the vote-sink edge the event pushed.
    pub class: Option<BirthEdgeClass>,
    /// How far a learning edit (the fourteen operators of
    /// `mutation::graph::hebbian`) reached.
    pub broadcast: Option<Broadcast>,
    /// What the event did to bundles, from the container's return values.
    pub bundle_edits: BundleEdits,
    /// Where every learnable of the genome after the event inherits from
    /// (T29.F01 invariant 5).
    pub inheritance: InheritanceMapping,
}

/// Whether a mutation target node is reachable from the mesh entry node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetReachability {
    Reachable,
    Unreachable,
    NotApplicable,
}

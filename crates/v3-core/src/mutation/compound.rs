use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::vote::{VOTE_KIND_COUNT, VOTE_SINK_COUNT};
use crate::creature::genome::OUTCOME_CHANNEL_COUNT;

/// Number of sub-values for a compound input. Returns 1 for scalar inputs.
///
/// World key widths delegate to `WorldInputKey::compound_width()`.
/// ActionQueue width is config-dependent (`action_queue_cap * 3`); the
/// decision-state widths are constants of the vote catalog (T19.F05).
#[must_use]
pub fn sub_value_count(reference: &InputReference, config: &MutationConfig) -> u16 {
    match reference {
        InputReference::ActionQueue => (config.action_queue_cap as u16) * 3,
        InputReference::World(key) => key.compound_width(),
        InputReference::ActionVotes | InputReference::PreviousPassVotes => VOTE_SINK_COUNT as u16,
        InputReference::CommitCounts => VOTE_KIND_COUNT as u16,
        InputReference::PreviousOutcome => OUTCOME_CHANNEL_COUNT as u16,
        InputReference::StaticIntrospection(_)
        | InputReference::DynamicIntrospection(_)
        | InputReference::UpstreamSlot(_) => 1,
    }
}

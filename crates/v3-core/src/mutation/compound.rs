use crate::contracts::InputReference;
use crate::creature::genome::vote::{VOTE_KIND_COUNT, VOTE_SINK_COUNT};
use crate::creature::genome::OUTCOME_CHANNEL_COUNT;

/// Input slots the `ActionQueue` reference exposes: the first four queued
/// actions, three sub-values each.
const ACTION_QUEUE_INPUT_SLOTS: u16 = 4;

/// Sub-values per `ActionQueue` input slot.
const ACTION_QUEUE_SUB_VALUES_PER_SLOT: u16 = 3;

/// Number of sub-values for a compound input. Returns 1 for scalar inputs.
///
/// World key widths delegate to `WorldInputKey::compound_width()`. Every
/// other width is a constant: `ActionQueue` is four slots of three
/// sub-values, and the decision-state widths come from the vote catalog
/// (T19.F05).
#[must_use]
pub fn sub_value_count(reference: &InputReference) -> u16 {
    match reference {
        InputReference::ActionQueue => ACTION_QUEUE_INPUT_SLOTS * ACTION_QUEUE_SUB_VALUES_PER_SLOT,
        InputReference::World(key) => key.compound_width(),
        InputReference::ActionVotes | InputReference::PreviousPassVotes => VOTE_SINK_COUNT as u16,
        InputReference::CommitCounts => VOTE_KIND_COUNT as u16,
        InputReference::PreviousOutcome => OUTCOME_CHANNEL_COUNT as u16,
        InputReference::StaticIntrospection(_)
        | InputReference::DynamicIntrospection(_)
        | InputReference::UpstreamSlot(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;

    /// The `ActionQueue` width follows no config value: it is 12 whatever
    /// `max_actions_per_turn` the normalized config carries.
    #[test]
    fn action_queue_width_is_twelve_at_every_max_actions_per_turn() {
        for max_actions_per_turn in [1, 4, 10, 20] {
            let mut config = SimulationConfig::default();
            config.runtime.max_actions_per_turn = max_actions_per_turn;
            config.normalize();
            assert_eq!(config.runtime.max_actions_per_turn, max_actions_per_turn);
            assert_eq!(sub_value_count(&InputReference::ActionQueue), 12);
        }
    }
}

use crate::config::MutationConfig;
use crate::contracts::InputReference;

/// Number of sub-values for a compound input. Returns 1 for scalar inputs.
///
/// World key widths delegate to `WorldInputKey::compound_width()`.
/// ActionQueue width is config-dependent (`action_queue_cap * 3`).
#[must_use]
pub fn sub_value_count(reference: &InputReference, config: &MutationConfig) -> u16 {
    match reference {
        InputReference::ActionQueue => (config.action_queue_cap as u16) * 3,
        InputReference::World(key) => key.compound_width(),
        _ => 1,
    }
}

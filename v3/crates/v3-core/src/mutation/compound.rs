use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{BackendDef, NodeGenome};

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

/// Create InputRef leaf nodes for a new input reference and optionally wire each
/// into the graph. No-op for CGP Graph backends and VM backends.
pub fn create_and_connect_input_leaves(
    node: &mut NodeGenome,
    _ref_idx: u16,
    _count: u16,
    _connect_chance: f32,
    _rng: &mut impl rand::Rng,
) {
    match &node.backend_def {
        BackendDef::Graph(_) | BackendDef::Vm(_) => { /* no-op */ }
    }
}

// Old compound fan-out tests deleted. CGP Graph backend does not use
// InputRef leaf nodes — inputs are addressed via GraphSource::InputLeaf edges.

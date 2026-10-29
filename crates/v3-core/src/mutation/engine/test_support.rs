//! The engine's test entry: a birth under the default config's sensor
//! context and body ranges.

use rand::Rng;

use super::MutationEngine;
use crate::config::{MutationConfig, SimulationConfig};
use crate::creature::body::body_ranges;
use crate::genome::learning::InheritanceMapping;
use crate::genome::CreatureGenome;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::types::MutationSummary;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// `parent_reachable_nodes` is the parent's cached reachable set (sorted ascending),
    /// used to bias mutation target selection toward functional structure.
    /// The parent carries no executed set here, so the executed layer never fires.
    /// The body's ranges are the default config's.
    ///
    /// Accounting invariant: `summary.attempted_events == summary.applied_events + summary.skipped_events`.
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        rng: &mut impl Rng,
    ) -> (MutationSummary, InheritanceMapping) {
        let defaults = SimulationConfig::default();
        Self::apply_mutations_in(
            genome,
            config,
            &body_ranges(&defaults),
            parent_reachable_nodes,
            ParentExecuted::NONE,
            rng,
            &defaults.sensor_context(1),
        )
    }
}

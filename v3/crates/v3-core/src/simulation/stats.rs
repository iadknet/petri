use std::collections::HashMap;

use crate::mutation::{MutationDomain, MutationOperator, MutationSkipReason};
use crate::simulation::actions::{PredationActionResult, ReproductionActionResult};

/// Observability counters for the simulation.
///
/// Cumulative counters never reset. Per-tick counters are reset at the start
/// of each call to `run_tick`.
#[derive(Debug, Clone, Default)]
pub struct SimStats {
    // ── Cumulative (never reset) ─────────────────────────────────────────────
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    /// Per-domain attempted mutation event breakdown (cumulative).
    pub mutation_events_attempted_total_by_domain: HashMap<MutationDomain, u64>,
    /// Per-domain applied mutation event breakdown (cumulative).
    pub mutation_events_applied_total_by_domain: HashMap<MutationDomain, u64>,
    /// Per-operator attempted mutation event breakdown (cumulative).
    pub mutation_events_attempted_total_by_operator: HashMap<MutationOperator, u64>,
    /// Per-operator applied mutation event breakdown (cumulative).
    pub mutation_events_applied_total_by_operator: HashMap<MutationOperator, u64>,
    /// Applied events classified as semantic-noop.
    pub mutation_events_applied_total_semantic_noop: u64,
    /// Applied events classified as semantic-change.
    pub mutation_events_applied_total_semantic_change: u64,
    /// Per-reason mutation skip breakdown (cumulative).
    pub mutation_events_skipped_by_reason: HashMap<MutationSkipReason, u64>,
    /// Per-reason rejection breakdown (cumulative).
    pub reproduction_actions_rejected_by_reason: HashMap<ReproductionActionResult, u64>,

    // ── Predation cumulative ─────────────────────────────────────────────────
    pub predation_actions_attempted_total: u64,
    pub predation_actions_transferred_total: u64,
    pub predation_actions_rejected_total: u64,
    pub predation_kills_total: u64,
    /// Per-result predation outcome breakdown (cumulative).
    pub predation_actions_by_result: HashMap<PredationActionResult, u64>,

    // ── Per-tick (reset at start of each tick) ───────────────────────────────
    pub last_tick_move: u32,
    pub last_tick_eat: u32,
    pub last_tick_noop: u32,
    pub last_tick_reproduce: u32,
    pub last_tick_steal: u32,

    // ── Per-tick compute cost (reset at start of each tick) ──────────────────
    /// Mean total (vm + graph) compute energy cost across all creatures that ran the mesh.
    pub last_tick_compute_total_mean: f32,
    /// Minimum total compute cost across creatures that ran the mesh.
    pub last_tick_compute_total_min: f32,
    /// Maximum total compute cost across creatures that ran the mesh.
    pub last_tick_compute_total_max: f32,
    /// Mean VM-node compute cost across creatures that executed at least one VM node.
    pub last_tick_compute_vm_mean: f32,
    /// Mean graph-node compute cost across creatures that executed at least one graph node.
    pub last_tick_compute_graph_mean: f32,
}

use std::collections::HashMap;

use crate::mutation::{
    MutationDomain, MutationOperator, MutationOperatorFunnel, MutationSkipReason,
};
use crate::simulation::actions::{
    BarrierReaderState, MoveBlockedCause, PredationActionResult, PredationEventRecord,
    ReproductionActionResult, ReproductionInvalidTargetCause,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct MutationValueTotals {
    pub carriers_observed_total: u64,
    pub survival_ticks_sum: u64,
    pub offspring_spawned_sum: u64,
    pub final_energy_sum: f64,
}

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
    /// Per-operator mutation skip breakdown (cumulative).
    pub mutation_events_skipped_total_by_operator: HashMap<MutationOperator, u64>,
    /// Per-operator mutation event funnel counters (cumulative).
    pub mutation_operator_funnel_total_by_operator:
        HashMap<MutationOperator, MutationOperatorFunnel>,
    /// Per-operator skip reason breakdown (cumulative).
    pub mutation_skip_reasons_total_by_operator:
        HashMap<MutationOperator, HashMap<MutationSkipReason, u64>>,
    /// Per-reason rejection breakdown (cumulative).
    pub reproduction_actions_rejected_by_reason: HashMap<ReproductionActionResult, u64>,
    /// Fine-grained invalid-target rejection breakdown (cumulative).
    pub reproduction_actions_rejected_invalid_target_total_by_cause:
        HashMap<ReproductionInvalidTargetCause, u64>,
    /// Invalid-target reproduction outcomes where at least one adjacent alternative target was
    /// valid.
    pub reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
        HashMap<BarrierReaderState, u64>,
    /// Fine-grained move blocked breakdown (cumulative).
    pub move_actions_blocked_total_by_cause: HashMap<MoveBlockedCause, u64>,
    /// Move blocked outcomes where at least one adjacent alternative target was valid.
    pub move_actions_blocked_avoidable_total_by_reader_state: HashMap<BarrierReaderState, u64>,
    /// Move attempts made while at least one neighboring barrier was present.
    pub move_attempts_with_barrier_neighbor_total_by_reader_state: HashMap<BarrierReaderState, u64>,
    /// Move actions blocked by barriers while at least one neighboring barrier was present.
    pub move_blocked_barrier_with_barrier_neighbor_total_by_reader_state:
        HashMap<BarrierReaderState, u64>,
    /// Reproduction attempts made while at least one neighboring barrier was present.
    pub reproduction_attempts_with_barrier_neighbor_total_by_reader_state:
        HashMap<BarrierReaderState, u64>,
    /// Reproduction invalid-target(barrier) outcomes while at least one neighboring barrier
    /// was present.
    pub reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
        HashMap<BarrierReaderState, u64>,

    // ── Reachability telemetry (cumulative) ───────────────────────────────────
    /// Mutation events where the selected target was a reachable node.
    pub mutation_reachable_target_total: u64,
    /// Mutation events where the selected target was an unreachable node.
    pub mutation_unreachable_target_total: u64,
    /// Mutation events where target reachability was not applicable (exempt operators).
    pub mutation_not_applicable_target_total: u64,
    /// Lifecycle value aggregates keyed by mutation operator on carrier creatures.
    pub mutation_value_totals_by_operator: HashMap<MutationOperator, MutationValueTotals>,

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
    /// Per-tick predation event buffer for frontend visualization.
    pub last_tick_predation_events: Vec<PredationEventRecord>,
    /// Per-tick kill count for time-series charting.
    pub last_tick_predation_kills: u32,

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

    // ── Per-tick priority bid stats (reset at start of each tick) ─────────────
    /// Mean priority bid energy across all creatures (including 0-bidders).
    pub last_tick_priority_bid_mean: f32,
    /// Number of creatures that bid > 0 this tick.
    pub last_tick_priority_bidders_count: u32,
}

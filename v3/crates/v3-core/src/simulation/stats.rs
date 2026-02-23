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
    /// Per-reason rejection breakdown (cumulative).
    pub reproduction_actions_rejected_by_reason: std::collections::HashMap<String, u64>,

    // ── Per-tick (reset at start of each tick) ───────────────────────────────
    pub last_tick_move: u32,
    pub last_tick_eat: u32,
    pub last_tick_noop: u32,
    pub last_tick_reproduce: u32,
}

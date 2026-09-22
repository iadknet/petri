use crate::contracts::{ActionQueue, WorldAction};
use crate::creature::genome::cgp::CUSTOM_OUTPUT_COUNT;
use crate::creature::genome::vote::{
    VoteVector, VOTE_KIND_COUNT, VOTE_PARAM_SLOTS, VOTE_SINK_COUNT,
};
use crate::runtime::routing::RouteGateMap;
use crate::runtime::trace::domain::TerminationReason;

/// Number of output slots produced by an evaluated cognition node.
pub const OUTPUT_SLOT_COUNT: usize = CUSTOM_OUTPUT_COUNT as usize;

/// Result returned by a single node evaluation.
/// The mesh executor uses this to decide routing. Action queue lives in the
/// mesh executor, not inside NodeResult.
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeResult {
    /// Output slots for downstream nodes. Initialized from incoming upstream_slots;
    /// only slots written by the node are overwritten.
    pub output_slots: [f32; OUTPUT_SLOT_COUNT],
    /// Per-target gate scores for routing decisions.
    pub route_gates: RouteGateMap,
    /// True when execution should stop (ExecuteActionQueue or Halt).
    pub terminal: bool,
    /// True when the node was halted due to energy exhaustion.
    /// When true, the mesh executor MUST discard the action queue and return `[NoOp]`.
    pub energy_exhausted: bool,
}

impl NodeResult {
    /// Create a non-terminal result (halt or step cap reached — node finished but mesh continues).
    pub fn halted(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self {
        Self {
            output_slots,
            route_gates,
            terminal: false,
            energy_exhausted: false,
        }
    }

    /// Create a result indicating energy exhaustion.
    pub fn exhausted() -> Self {
        Self {
            output_slots: [0.0; OUTPUT_SLOT_COUNT],
            route_gates: RouteGateMap::default(),
            terminal: true,
            energy_exhausted: true,
        }
    }

    /// Create a terminal result (action emitted or ExecuteActionQueue).
    pub fn terminal(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self {
        Self {
            output_slots,
            route_gates,
            terminal: true,
            energy_exhausted: false,
        }
    }
}

/// Energy cost attributed to VM and graph node execution during one mesh evaluation.
///
/// Returned alongside queued [`WorldAction`] values by the mesh executor.
/// Does not include lifecycle decay, move, eat, noop, or reproduce costs.
#[derive(Debug, Clone, Default)]
pub struct ComputeCostReport {
    /// Total energy deducted from executing VM nodes this tick.
    pub vm_cost: f32,
    /// Total energy deducted from executing Graph nodes this tick.
    pub graph_cost: f32,
    /// Total energy deducted by the per-tick hop ramp this tick (T19.F01).
    pub mesh_ramp_cost: f32,
}

/// Accumulated side outputs from VM/graph node execution within a single mesh evaluation.
/// Passed as `&mut` through the mesh hop chain; consumed by the mesh executor on return.
#[derive(Debug)]
pub struct MeshSideOutputs {
    /// Action queue that persists across mesh hops.
    pub action_queue: ActionQueue,
    /// Energy bid for turn-order priority. 0.0 = no bid (default).
    pub priority_bid: f32,
    /// Deterministic integer work counters accumulated during this mesh evaluation.
    pub work_counters: WorkCounters,
    /// Applied cognition debits and exhausting sink; never used to control execution.
    pub energy_observation: crate::simulation::energy_accounting::CognitionEnergyObservation,
    /// Inert vote surface (T19.F03): the sanitized sum of the latest committed
    /// contribution of every node visited this evaluation. Nothing reads it.
    pub votes: VoteVector,
    /// Per-kind commit counters (T19.F03). Zero throughout: nothing commits a
    /// vote to a world action until T19.F04.
    pub commit_counts: [u32; VOTE_KIND_COUNT],
    /// Inert parameter surface (T19.F03): a wired `ActionParam(kind, i)` sink
    /// overwrites `action_params[kind][i]`, last visit wins. Nothing reads it.
    pub action_params: [[f32; VOTE_PARAM_SLOTS as usize]; VOTE_KIND_COUNT],
    /// Latest committed contribution per genome node index, in first-commit
    /// order. A revisit replaces the node's entry rather than adding one.
    node_contributions: Vec<(usize, VoteVector)>,
    /// Contribution staged by the dispatch in flight, taken by the mesh loop
    /// once the dispatch returns. `None` when the dispatch did not commit.
    staged_contribution: Option<VoteVector>,
}

impl MeshSideOutputs {
    /// Create a new `MeshSideOutputs` with an empty action queue.
    pub fn new(max_actions: usize) -> Self {
        Self {
            action_queue: ActionQueue::new(max_actions),
            priority_bid: 0.0,
            work_counters: WorkCounters::default(),
            energy_observation: Default::default(),
            votes: [0.0; VOTE_SINK_COUNT],
            commit_counts: [0; VOTE_KIND_COUNT],
            action_params: [[0.0; VOTE_PARAM_SLOTS as usize]; VOTE_KIND_COUNT],
            node_contributions: Vec::new(),
            staged_contribution: None,
        }
    }

    /// Stage this dispatch's vote contribution (T19.F03). Called at the
    /// boundary where a dispatch commits its effects, which is every exit
    /// except energy exhaustion; entries are sanitized here.
    pub(crate) fn stage_vote_contribution(&mut self, contribution: &VoteVector) {
        self.staged_contribution = Some(contribution.map(sanitize_f32));
    }

    /// Take the staged contribution as `node_idx`'s latest, replacing that
    /// node's previous one, and re-sum the mesh vote vector. Returns what the
    /// hop committed: zeros when the dispatch staged nothing.
    pub(crate) fn commit_vote_contribution(&mut self, node_idx: usize) -> VoteVector {
        let Some(contribution) = self.staged_contribution.take() else {
            return [0.0; VOTE_SINK_COUNT];
        };
        match self
            .node_contributions
            .iter_mut()
            .find(|(idx, _)| *idx == node_idx)
        {
            Some((_, latest)) => *latest = contribution,
            None => self.node_contributions.push((node_idx, contribution)),
        }
        let mut votes = [0.0f32; VOTE_SINK_COUNT];
        for (_, latest) in &self.node_contributions {
            for (sum, value) in votes.iter_mut().zip(latest) {
                *sum += *value;
            }
        }
        self.votes = votes.map(sanitize_f32);
        contribution
    }
}

/// Deterministic integer work counters for one creature's mesh evaluation.
///
/// Each field is incremented exactly where that unit of work executes, in
/// both the plain and traced mesh execution modes (both share the same
/// underlying loops). These counters are integers, so summing them after the
/// parallel mesh phase is deterministic regardless of thread scheduling.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkCounters {
    /// Mesh hops walked in `execute_creature_mesh_impl`'s routing loop.
    pub mesh_hops: u32,
    /// VM opcodes successfully executed across all VM node dispatches.
    pub vm_steps: u32,
    /// Entered single-evaluation graph visits, including unaffordable visits.
    /// Wire name retained; before T11.F06 this counted relaxation passes.
    pub graph_relax_iters: u32,
    /// Hebbian weight updates applied (reward-modulated updates are counted
    /// separately in `SimStats`, since they run after the mesh phase).
    pub plasticity_updates: u32,
    /// Hebbian assignments whose final stored weight differs from its prior value.
    pub plasticity_changes: u32,
    /// Executed shared-memory store/clear events exceeding the trace epsilon.
    /// Includes VM writes executed before a later exhaustion discards its copy.
    pub shared_memory_writes_changed: u32,
    /// Passes that reached the per-pass hop cap (`max_mesh_hops`) and ended
    /// with their queue kept (T19.F02). Equals the `MaxHopsReached`
    /// termination count while a tick is one pass.
    pub pass_cap_hits: u32,
}

/// Complete output of one creature's mesh evaluation for a single tick.
#[derive(Debug, Clone)]
#[must_use]
pub struct MeshOutput {
    /// Actions queued for Phase 2 execution.
    pub actions: Vec<WorldAction>,
    /// Energy cost attribution (VM vs graph).
    pub cost_report: ComputeCostReport,
    /// Energy bid for turn-order priority. 0.0 = no bid (default).
    pub priority_bid: f32,
    /// Deterministic integer work counters accumulated during this evaluation.
    pub work_counters: WorkCounters,
    /// Dispatch-local applied energy observations, committed sequentially by Phase 2.
    pub energy_observation: crate::simulation::energy_accounting::CognitionEnergyObservation,
    /// Why the mesh chain stopped. Carried on every execution mode's output so
    /// the untraced production path can count terminations without a trace.
    pub termination_reason: TerminationReason,
    /// The evaluation's accumulated vote vector (T19.F03), carried for the
    /// trace. Nothing else reads it.
    pub votes: VoteVector,
    /// Per-kind commit counters (T19.F03), carried for the trace. Zero until
    /// T19.F04 commits a vote.
    pub commit_counts: [u32; VOTE_KIND_COUNT],
}

/// Sanitize an f32 value per v3-vm-isa-spec.md Section 5:
/// - NaN → 0.0
/// - +Inf → +1_000_000_000.0
/// - -Inf → -1_000_000_000.0
/// - Finite values clamped to [-1e9, 1e9]
#[inline]
#[must_use]
pub fn sanitize_f32(v: f32) -> f32 {
    const CLAMP: f32 = 1_000_000_000.0;
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-CLAMP, CLAMP)
    }
}

/// The shared changed-write predicate for production memory events and VM traces.
/// Call with the sanitized written value and the slot's immediately prior value.
#[inline]
pub(crate) fn shared_memory_write_changed(old_value: f32, new_value: f32) -> bool {
    (old_value - new_value).abs() > f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_nan_becomes_zero() {
        assert_eq!(sanitize_f32(f32::NAN), 0.0);
    }

    #[test]
    fn sanitize_pos_inf_becomes_clamp() {
        assert_eq!(sanitize_f32(f32::INFINITY), 1_000_000_000.0);
    }

    #[test]
    fn sanitize_neg_inf_becomes_neg_clamp() {
        assert_eq!(sanitize_f32(f32::NEG_INFINITY), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_large_finite_clamps() {
        assert_eq!(sanitize_f32(2e9), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-2e9), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_normal_value_unchanged() {
        assert!((sanitize_f32(1.5) - 1.5).abs() < 1e-6);
        assert!((sanitize_f32(-0.5) - (-0.5)).abs() < 1e-6);
        assert_eq!(sanitize_f32(0.0), 0.0);
    }

    #[test]
    fn sanitize_exactly_at_boundary_unchanged() {
        assert_eq!(sanitize_f32(1_000_000_000.0), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-1_000_000_000.0), -1_000_000_000.0);
    }

    #[test]
    fn node_result_halted_is_not_terminal() {
        let r = NodeResult::halted([0.0; OUTPUT_SLOT_COUNT], RouteGateMap::default());
        assert!(!r.terminal);
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn node_result_exhausted_has_flag() {
        let r = NodeResult::exhausted();
        assert!(r.energy_exhausted);
        assert!(r.terminal);
        assert_eq!(r.route_gates, RouteGateMap::default());
    }

    #[test]
    fn node_result_terminal_is_terminal() {
        let r = NodeResult::terminal([0.0; OUTPUT_SLOT_COUNT], RouteGateMap::default());
        assert!(r.terminal);
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn output_slot_count_matches_bus_experiment_target() {
        assert_eq!(OUTPUT_SLOT_COUNT, 24);
    }
}

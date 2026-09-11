use std::collections::HashMap;
use std::time::Duration;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::WorldInputKey;
use crate::kernel::FoodGrowthSummary;
use crate::mutation::{
    MutationAddedNodeInputClass, MutationDomain, MutationOperator, MutationOperatorFunnel,
    MutationSkipReason,
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
    pub helpful_total: u64,
    pub neutral_total: u64,
    pub detrimental_total: u64,
    pub confidence_low_total: u64,
    pub confidence_medium_total: u64,
    pub confidence_high_total: u64,
    pub viability_score_sum: f64,
    pub viability_score_delta_sum: f64,
    pub survived_short_horizon_total: u64,
    pub survived_long_horizon_total: u64,
    pub reproduced_once_total: u64,
    pub mean_lifetime_energy_sum: f64,
    pub action_attempted_total: u64,
    pub blocked_move_total: u64,
    pub invalid_reproduce_total: u64,
    pub invalid_action_total: u64,
}

impl MutationValueTotals {
    pub fn record_outcome(
        &mut self,
        observation: MutationOutcomeObservation,
        evaluation: MutationOutcomeEvaluation,
    ) {
        self.carriers_observed_total += 1;
        self.survival_ticks_sum += observation.survival_ticks;
        self.offspring_spawned_sum += observation.offspring_spawned_total;
        self.final_energy_sum += observation.final_energy;
        self.viability_score_sum += observation.viability_score;
        self.viability_score_delta_sum += evaluation.score_delta;
        self.mean_lifetime_energy_sum += observation.mean_lifetime_energy;
        self.action_attempted_total += observation.action_attempted_total;
        self.blocked_move_total += observation.blocked_move_total;
        self.invalid_reproduce_total += observation.invalid_reproduce_total;
        self.invalid_action_total += observation.invalid_action_total;

        if observation.survived_short_horizon {
            self.survived_short_horizon_total += 1;
        }
        if observation.survived_long_horizon {
            self.survived_long_horizon_total += 1;
        }
        if observation.reproduced_once {
            self.reproduced_once_total += 1;
        }

        match evaluation.class {
            MutationOutcomeClass::Helpful => self.helpful_total += 1,
            MutationOutcomeClass::Neutral => self.neutral_total += 1,
            MutationOutcomeClass::Detrimental => self.detrimental_total += 1,
        }
        match evaluation.confidence {
            MutationOutcomeConfidence::Low => self.confidence_low_total += 1,
            MutationOutcomeConfidence::Medium => self.confidence_medium_total += 1,
            MutationOutcomeConfidence::High => self.confidence_high_total += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationOutcomeClass {
    Helpful,
    Neutral,
    Detrimental,
}

impl MutationOutcomeClass {
    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Helpful => "helpful",
            Self::Neutral => "neutral",
            Self::Detrimental => "detrimental",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationOutcomeConfidence {
    Low,
    Medium,
    High,
}

impl MutationOutcomeConfidence {
    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MutationOutcomeEvaluation {
    pub class: MutationOutcomeClass,
    pub confidence: MutationOutcomeConfidence,
    pub score_delta: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MutationOutcomeObservation {
    pub survival_ticks: u64,
    pub offspring_spawned_total: u64,
    pub final_energy: f64,
    pub viability_score: f64,
    pub mean_lifetime_energy: f64,
    pub action_attempted_total: u64,
    pub blocked_move_total: u64,
    pub invalid_reproduce_total: u64,
    pub invalid_action_total: u64,
    pub survived_short_horizon: bool,
    pub survived_long_horizon: bool,
    pub reproduced_once: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RunningStats {
    pub count: u64,
    pub mean: f64,
    pub m2: f64,
}

impl RunningStats {
    pub fn push(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    #[must_use]
    pub fn std_dev(self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        (self.m2 / (self.count as f64 - 1.0)).sqrt()
    }
}

/// Cumulative wall-clock spent inside each timed tick phase (T10.F09).
///
/// Wall-clock is host-dependent and noisy: these fields are observational
/// only, are never reset by [`SimStats::reset_tick_counters`], and no test
/// asserts their magnitude. The turn-queue build and the priority sort are
/// deliberately untimed — they are the remainder against a tick's total
/// wall-clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct PhaseWallClock {
    /// Phase 0: food growth, aging, energy decay, death removal.
    pub world_update: Duration,
    /// Phase 1a: sequential sensor-input assembly.
    pub sensor_assembly: Duration,
    /// Phase 1b: batch cognition (parallel on the rayon pool in effect).
    pub cognition: Duration,
    /// Phase 2: sequential action execution.
    pub actions: Duration,
    /// Phase 2.5: reward-modulated learning.
    pub reward_learning: Duration,
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
    /// Input classes attached or wired when a mutation added a new node, grouped by operator.
    pub mutation_added_node_input_classes_total_by_operator:
        HashMap<MutationOperator, HashMap<MutationAddedNodeInputClass, u64>>,
    /// Exact world inputs attached or wired when a mutation added a new node, grouped by operator.
    pub mutation_added_node_world_inputs_total_by_operator:
        HashMap<MutationOperator, HashMap<WorldInputKey, u64>>,
    /// Per-reason rejection breakdown (cumulative).
    pub reproduction_actions_rejected_by_reason: HashMap<ReproductionActionResult, u64>,
    /// Fine-grained invalid-target rejection breakdown (cumulative).
    pub reproduction_actions_rejected_invalid_target_total_by_cause:
        HashMap<ReproductionInvalidTargetCause, u64>,
    /// Invalid-target reproduction outcomes where at least one adjacent alternative target was
    /// valid.
    pub reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
        HashMap<BarrierReaderState, u64>,
    /// Move actions the action phase executed, whether or not they moved the
    /// creature (cumulative). The denominator for every blocked-move fraction.
    pub move_actions_attempted_total: u64,
    /// Eat actions that consumed food, keyed by the food type the action named
    /// (cumulative). A typed Eat that found no food is not counted, so this
    /// reads which food types the living population actually harvests.
    pub eat_actions_applied_total_by_type: HashMap<OrdinaryFoodTypeId, u64>,
    /// Eat actions that found no food, keyed by the food type the action named
    /// (cumulative). The failure twin of
    /// [`SimStats::eat_actions_applied_total_by_type`]: together they are every
    /// typed Eat the population executed.
    pub eat_actions_failed_total_by_type: HashMap<OrdinaryFoodTypeId, u64>,
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
    /// Mutation events whose target was a node the parent executed recently
    /// (T11.F17). A subset of the reachable total in production.
    pub mutation_executed_target_total: u64,
    /// Mutation events where target reachability was not applicable (exempt operators).
    pub mutation_not_applicable_target_total: u64,
    /// Lifecycle value aggregates keyed by mutation operator on carrier creatures.
    pub mutation_value_totals_by_operator: HashMap<MutationOperator, MutationValueTotals>,
    /// Mutation lifecycle outcome summary across all creatures with at least one
    /// applied birth mutation operator.
    pub mutation_outcome_summary: MutationValueTotals,
    /// Running cohort baselines keyed by generation bucket.
    pub mutation_outcome_baseline_by_generation_bucket: HashMap<u64, RunningStats>,
    /// Global fallback baseline used when cohort-local sample sizes are small.
    pub mutation_outcome_baseline_global: RunningStats,

    // ── Deterministic work counters (cumulative) ──────────────────────────────
    /// Mesh hops walked across all creatures, all ticks.
    pub mesh_hops_total: u64,
    /// VM opcodes executed across all creatures, all ticks.
    pub vm_steps_total: u64,
    /// Entered nonempty graph visits across all creatures/ticks (single evaluation
    /// since T11.F06; retained wire counter formerly counted relaxation passes).
    pub graph_relax_iters_total: u64,
    /// Hebbian plus reward-modulated plasticity weight updates applied.
    pub plasticity_updates_total: u64,
    /// Creatures that ran the mesh, summed per tick.
    pub creature_ticks_total: u64,
    /// Mesh dispatches that stopped because the creature ran out of energy
    /// mid-chain, summed in queue order after the parallel mesh phase.
    pub mesh_dispatches_energy_exhausted_total: u64,
    /// Every action the action phase executed (move, eat, noop, reproduce, steal).
    pub actions_applied_total: u64,

    // ── Phase wall-clock (cumulative, observational, never asserted) ─────────
    /// Cumulative wall-clock per tick phase (T10.F09).
    pub phase_wall_clock: PhaseWallClock,

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
    /// Mean depletion across passable cells after the Phase 0 food update.
    pub last_tick_food_occupancy_depletion_mean: f32,
    /// Number of occupied cells that deposited depletion during the Phase 0 food update.
    pub last_tick_food_occupancy_depletion_occupied_cells: u32,
    /// Total food growth amount suppressed by occupancy depletion during the Phase 0 food update.
    pub last_tick_food_growth_suppressed_by_occupancy_depletion: f32,
    /// Number of cells where at least one food type experienced cross-type growth inhibition.
    pub last_tick_food_cells_with_type_inhibition: u32,
    /// Total food growth amount suppressed by cross-type inhibition during the Phase 0 food update.
    pub last_tick_food_growth_suppressed_by_type_inhibition: f32,
    /// Standing density per food type after the Phase 0 food update, indexed by
    /// [`OrdinaryFoodTypeId`]. Read from the applied growth summary, so it is
    /// post-growth and pre-action for the tick that just ran.
    pub last_tick_food_total_density_by_type: Vec<f32>,
}

const OUTCOME_GENERATION_BUCKET_WIDTH: u64 = 128;
const OUTCOME_BASELINE_MIN_MEDIUM_CONFIDENCE: u64 = 20;
const OUTCOME_BASELINE_MIN_HIGH_CONFIDENCE: u64 = 100;
const OUTCOME_BASELINE_MIN_CLASSIFICATION: u64 = 5;
const OUTCOME_SCORE_STD_DEV_FLOOR: f64 = 0.05;
const OUTCOME_SCORE_Z_THRESHOLD: f64 = 0.25;

impl SimStats {
    /// Clear the per-tick (`last_tick_*`) counters at the start of a tick.
    ///
    /// Cumulative totals and the `*_by_*` breakdown maps are untouched.
    pub fn reset_tick_counters(&mut self) {
        self.last_tick_move = 0;
        self.last_tick_eat = 0;
        self.last_tick_noop = 0;
        self.last_tick_reproduce = 0;
        self.last_tick_steal = 0;
        self.last_tick_predation_events.clear();
        self.last_tick_predation_kills = 0;
        self.last_tick_compute_total_mean = 0.0;
        self.last_tick_compute_total_min = 0.0;
        self.last_tick_compute_total_max = 0.0;
        self.last_tick_compute_vm_mean = 0.0;
        self.last_tick_compute_graph_mean = 0.0;
        self.last_tick_priority_bid_mean = 0.0;
        self.last_tick_priority_bidders_count = 0;
        self.last_tick_food_occupancy_depletion_mean = 0.0;
        self.last_tick_food_occupancy_depletion_occupied_cells = 0;
        self.last_tick_food_growth_suppressed_by_occupancy_depletion = 0.0;
        self.last_tick_food_cells_with_type_inhibition = 0;
        self.last_tick_food_growth_suppressed_by_type_inhibition = 0.0;
        self.last_tick_food_total_density_by_type.clear();
    }

    pub fn record_food_growth_summary(&mut self, summary: FoodGrowthSummary) {
        self.last_tick_food_occupancy_depletion_mean = summary.mean_occupancy_depletion;
        self.last_tick_food_occupancy_depletion_occupied_cells =
            summary.occupied_cells_with_depletion;
        self.last_tick_food_growth_suppressed_by_occupancy_depletion =
            summary.growth_suppressed_by_occupancy_depletion;
        self.last_tick_food_cells_with_type_inhibition = summary.cells_with_type_inhibition;
        self.last_tick_food_growth_suppressed_by_type_inhibition =
            summary.growth_suppressed_by_type_inhibition;
        self.last_tick_food_total_density_by_type.clear();
        self.last_tick_food_total_density_by_type
            .extend(summary.per_type.iter().map(|entry| entry.total_density));
    }

    #[must_use]
    pub fn classify_mutation_outcome(
        &mut self,
        generation: u64,
        viability_score: f64,
    ) -> MutationOutcomeEvaluation {
        let generation_bucket = generation / OUTCOME_GENERATION_BUCKET_WIDTH;
        let bucket_baseline = self
            .mutation_outcome_baseline_by_generation_bucket
            .get(&generation_bucket)
            .copied()
            .unwrap_or_default();
        let global_baseline = self.mutation_outcome_baseline_global;

        let baseline = if bucket_baseline.count >= OUTCOME_BASELINE_MIN_MEDIUM_CONFIDENCE {
            bucket_baseline
        } else {
            global_baseline
        };

        let confidence = if baseline.count >= OUTCOME_BASELINE_MIN_HIGH_CONFIDENCE {
            MutationOutcomeConfidence::High
        } else if baseline.count >= OUTCOME_BASELINE_MIN_MEDIUM_CONFIDENCE {
            MutationOutcomeConfidence::Medium
        } else {
            MutationOutcomeConfidence::Low
        };

        let score_delta = if baseline.count == 0 {
            0.0
        } else {
            viability_score - baseline.mean
        };

        let class = if baseline.count < OUTCOME_BASELINE_MIN_CLASSIFICATION {
            MutationOutcomeClass::Neutral
        } else {
            let std_dev = baseline.std_dev().max(OUTCOME_SCORE_STD_DEV_FLOOR);
            let z = score_delta / std_dev;
            if z >= OUTCOME_SCORE_Z_THRESHOLD {
                MutationOutcomeClass::Helpful
            } else if z <= -OUTCOME_SCORE_Z_THRESHOLD {
                MutationOutcomeClass::Detrimental
            } else {
                MutationOutcomeClass::Neutral
            }
        };

        self.mutation_outcome_baseline_global.push(viability_score);
        self.mutation_outcome_baseline_by_generation_bucket
            .entry(generation_bucket)
            .or_default()
            .push(viability_score);

        MutationOutcomeEvaluation {
            class,
            confidence,
            score_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_value_totals_record_outcome_updates_classification_and_confidence_counters() {
        let mut totals = MutationValueTotals::default();
        totals.record_outcome(
            MutationOutcomeObservation {
                survival_ticks: 64,
                offspring_spawned_total: 2,
                final_energy: 12.0,
                viability_score: 0.75,
                mean_lifetime_energy: 10.0,
                action_attempted_total: 20,
                blocked_move_total: 3,
                invalid_reproduce_total: 2,
                invalid_action_total: 5,
                survived_short_horizon: true,
                survived_long_horizon: false,
                reproduced_once: true,
            },
            MutationOutcomeEvaluation {
                class: MutationOutcomeClass::Helpful,
                confidence: MutationOutcomeConfidence::Medium,
                score_delta: 0.15,
            },
        );

        assert_eq!(totals.carriers_observed_total, 1);
        assert_eq!(totals.helpful_total, 1);
        assert_eq!(totals.neutral_total, 0);
        assert_eq!(totals.detrimental_total, 0);
        assert_eq!(totals.confidence_low_total, 0);
        assert_eq!(totals.confidence_medium_total, 1);
        assert_eq!(totals.confidence_high_total, 0);
        assert_eq!(totals.invalid_action_total, 5);
        assert_eq!(totals.action_attempted_total, 20);
    }

    #[test]
    fn classify_mutation_outcome_uses_global_baseline_before_bucket_is_populated() {
        let mut stats = SimStats::default();
        for _ in 0..30 {
            stats.mutation_outcome_baseline_global.push(0.20);
        }

        let eval = stats.classify_mutation_outcome(999, 0.40);
        assert_eq!(eval.confidence, MutationOutcomeConfidence::Medium);
        assert_eq!(eval.class, MutationOutcomeClass::Helpful);
        assert!(
            eval.score_delta > 0.0,
            "expected positive score delta, got {}",
            eval.score_delta
        );
    }

    #[test]
    fn work_counter_totals_default_to_zero() {
        let stats = SimStats::default();
        assert_eq!(stats.mesh_hops_total, 0);
        assert_eq!(stats.vm_steps_total, 0);
        assert_eq!(stats.graph_relax_iters_total, 0);
        assert_eq!(stats.plasticity_updates_total, 0);
        assert_eq!(stats.creature_ticks_total, 0);
        assert_eq!(stats.actions_applied_total, 0);
    }

    #[test]
    fn record_food_growth_summary_updates_food_depletion_fields() {
        let mut stats = SimStats::default();
        stats.record_food_growth_summary(FoodGrowthSummary {
            mean_occupancy_depletion: 0.12,
            occupied_cells_with_depletion: 3,
            growth_suppressed_by_occupancy_depletion: 0.7,
            cells_with_type_inhibition: 2,
            growth_suppressed_by_type_inhibition: 0.4,
            per_type: vec![],
        });

        assert!((stats.last_tick_food_occupancy_depletion_mean - 0.12).abs() < 1e-6);
        assert_eq!(stats.last_tick_food_occupancy_depletion_occupied_cells, 3);
        assert!((stats.last_tick_food_growth_suppressed_by_occupancy_depletion - 0.7).abs() < 1e-6);
        assert_eq!(stats.last_tick_food_cells_with_type_inhibition, 2);
        assert!((stats.last_tick_food_growth_suppressed_by_type_inhibition - 0.4).abs() < 1e-6);
    }
}

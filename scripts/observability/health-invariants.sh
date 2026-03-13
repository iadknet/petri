#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

snapshot_json="$(fetch_json "/v3/simulation/snapshot?zoom_tier=detail")"

result_json="$(
	echo "${snapshot_json}" | "${JQ_BIN}" '
		def sum_values($obj):
			($obj // {} | to_entries | map(.value) | add // 0);
		.health as $h
		| ($h.mutation_events_attempted_total // 0) as $mutation_attempted
		| ($h.mutation_events_applied_total // 0) as $mutation_applied
		| ($h.mutation_events_skipped_total // 0) as $mutation_skipped
		| (sum_values($h.mutation_events_attempted_total_by_domain)) as $attempted_by_domain
		| (sum_values($h.mutation_events_applied_total_by_domain)) as $applied_by_domain
		| (sum_values($h.mutation_events_attempted_total_by_operator)) as $attempted_by_operator
		| (sum_values($h.mutation_events_applied_total_by_operator)) as $applied_by_operator
		| (sum_values($h.mutation_events_skipped_total_by_operator)) as $skipped_by_operator
		| (
			($h.mutation_target_reachability_total.reachable // 0)
			+ ($h.mutation_target_reachability_total.unreachable // 0)
			+ ($h.mutation_target_reachability_total.not_applicable // 0)
		) as $reachability_total
		| (
			($h.mutation_events_applied_total_semantic_noop // 0)
			+ ($h.mutation_events_applied_total_semantic_change // 0)
		) as $semantic_total
		| ($h.reproduction_actions_attempted_total // 0) as $repro_attempted
		| ($h.reproduction_actions_spawned_total // 0) as $repro_spawned
		| ($h.reproduction_actions_rejected_total // 0) as $repro_rejected
		| (sum_values($h.reproduction_actions_rejected_total_by_reason)) as $repro_rejected_by_reason
		| (($h.reproduction_actions_rejected_total_by_reason.RejectedInvalidTarget // 0)) as $repro_rejected_invalid_reason
		| (sum_values($h.reproduction_actions_rejected_invalid_target_total_by_cause)) as $repro_rejected_invalid_by_cause
		| (sum_values($h.move_actions_blocked_total_by_cause)) as $move_blocked_total
		| (sum_values($h.move_actions_blocked_avoidable_total_by_reader_state)) as $move_avoidable_total
		| (sum_values($h.reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state)) as $repro_avoidable_total
		| (sum_values($h.move_attempts_with_barrier_neighbor_total_by_reader_state)) as $move_neighbor_attempts
		| (sum_values($h.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state)) as $move_neighbor_blocked_barrier
		| (sum_values($h.reproduction_attempts_with_barrier_neighbor_total_by_reader_state)) as $repro_neighbor_attempts
		| (sum_values($h.reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state)) as $repro_neighbor_invalid_barrier
		| ($h.mutation_outcome_summary.carriers_observed_total // 0) as $outcome_carriers
		| (
			($h.mutation_outcome_summary.helpful_total // 0)
			+ ($h.mutation_outcome_summary.neutral_total // 0)
			+ ($h.mutation_outcome_summary.detrimental_total // 0)
		) as $outcome_class_total
		| (
			($h.mutation_outcome_summary.confidence_low_total // 0)
			+ ($h.mutation_outcome_summary.confidence_medium_total // 0)
			+ ($h.mutation_outcome_summary.confidence_high_total // 0)
		) as $outcome_conf_total
		| [
			{
				name: "mutation_attempted = applied + skipped",
				ok: ($mutation_attempted == ($mutation_applied + $mutation_skipped)),
				actual: $mutation_attempted,
				expected: ($mutation_applied + $mutation_skipped)
			},
			{
				name: "mutation_attempted_by_domain sum = mutation_attempted",
				ok: ($attempted_by_domain == $mutation_attempted),
				actual: $attempted_by_domain,
				expected: $mutation_attempted
			},
			{
				name: "mutation_applied_by_domain sum = mutation_applied",
				ok: ($applied_by_domain == $mutation_applied),
				actual: $applied_by_domain,
				expected: $mutation_applied
			},
			{
				name: "mutation_attempted_by_operator sum <= mutation_attempted",
				ok: ($attempted_by_operator <= $mutation_attempted),
				actual: $attempted_by_operator,
				expected_max: $mutation_attempted
			},
			{
				name: "mutation_applied_by_operator sum = mutation_applied",
				ok: ($applied_by_operator == $mutation_applied),
				actual: $applied_by_operator,
				expected: $mutation_applied
			},
			{
				name: "mutation_skipped_by_operator sum <= mutation_skipped",
				ok: ($skipped_by_operator <= $mutation_skipped),
				actual: $skipped_by_operator,
				expected_max: $mutation_skipped
			},
			{
				name: "mutation_reachability_total sum = mutation_applied",
				ok: ($reachability_total == $mutation_applied),
				actual: $reachability_total,
				expected: $mutation_applied
			},
			{
				name: "mutation_semantic_noop + semantic_change = mutation_applied",
				ok: ($semantic_total == $mutation_applied),
				actual: $semantic_total,
				expected: $mutation_applied
			},
			{
				name: "reproduction_attempted = spawned + rejected",
				ok: ($repro_attempted == ($repro_spawned + $repro_rejected)),
				actual: $repro_attempted,
				expected: ($repro_spawned + $repro_rejected)
			},
			{
				name: "reproduction_rejected_by_reason sum = reproduction_rejected_total",
				ok: ($repro_rejected_by_reason == $repro_rejected),
				actual: $repro_rejected_by_reason,
				expected: $repro_rejected
			},
			{
				name: "reproduction_invalid_target_by_cause sum <= rejected_invalid_target",
				ok: ($repro_rejected_invalid_by_cause <= $repro_rejected_invalid_reason),
				actual: $repro_rejected_invalid_by_cause,
				expected_max: $repro_rejected_invalid_reason
			},
			{
				name: "move_avoidable_total <= move_blocked_total",
				ok: ($move_avoidable_total <= $move_blocked_total),
				actual: $move_avoidable_total,
				expected_max: $move_blocked_total
			},
			{
				name: "reproduction_avoidable_total <= reproduction_invalid_target_by_cause",
				ok: ($repro_avoidable_total <= $repro_rejected_invalid_by_cause),
				actual: $repro_avoidable_total,
				expected_max: $repro_rejected_invalid_by_cause
			},
			{
				name: "move_barrier_blocked_with_neighbor <= move_attempts_with_neighbor",
				ok: ($move_neighbor_blocked_barrier <= $move_neighbor_attempts),
				actual: $move_neighbor_blocked_barrier,
				expected_max: $move_neighbor_attempts
			},
			{
				name: "repro_invalid_barrier_with_neighbor <= repro_attempts_with_neighbor",
				ok: ($repro_neighbor_invalid_barrier <= $repro_neighbor_attempts),
				actual: $repro_neighbor_invalid_barrier,
				expected_max: $repro_neighbor_attempts
			},
			{
				name: "mutation_outcome_summary class totals = carriers_observed_total",
				ok: ($outcome_class_total == $outcome_carriers),
				actual: $outcome_class_total,
				expected: $outcome_carriers
			},
			{
				name: "mutation_outcome_summary confidence totals = carriers_observed_total",
				ok: ($outcome_conf_total == $outcome_carriers),
				actual: $outcome_conf_total,
				expected: $outcome_carriers
			}
		] as $checks
		| {
			tick,
			population: .status.population,
			all_passed: ($checks | all(.ok)),
			checks: $checks
		}'
)"

echo "${result_json}"

all_passed="$(echo "${result_json}" | "${JQ_BIN}" -r '.all_passed')"
if [[ "${all_passed}" != "true" ]]; then
	exit 1
fi

#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

fetch_json "/v3/simulation/status" | "${JQ_BIN}" '{
	state,
	tick,
	population,
	mean_energy,
	move_actions_blocked_total_by_cause,
	move_actions_blocked_avoidable_total_by_reader_state: (
		.move_actions_blocked_avoidable_total_by_reader_state // {}
	),
	reproduction_actions_rejected_invalid_target_total_by_cause,
	reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state: (
		.reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state // {}
	),
	mutation_target_reachability_total,
	mutation_events_skipped_total,
	mutation_events_applied_total,
	mutation_outcome_summary: (.mutation_outcome_summary // {}),
	mutation_value_totals_by_operator: (.mutation_value_totals_by_operator // {})
}'

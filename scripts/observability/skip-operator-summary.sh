#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

threshold="${1:-1000}"

fetch_json "/v3/simulation/status" | "${JQ_BIN}" --argjson threshold "${threshold}" '
	(.mutation_events_attempted_total_by_operator) as $attempted
	| (.mutation_events_skipped_total_by_operator) as $skipped
	| (.mutation_value_totals_by_operator // {}) as $value
	| {
		tick,
		population,
		skip_total: .mutation_events_skipped_total,
		applied_total: .mutation_events_applied_total,
		skip_ratio: (
			.mutation_events_skipped_total
			/ (.mutation_events_skipped_total + .mutation_events_applied_total)
		),
		top_skipped_operators: (
			[
				$attempted
				| to_entries[]
				| . as $attempt
				| {
					operator: $attempt.key,
					attempted: $attempt.value,
					skipped: ($skipped[$attempt.key] // 0),
					skip_rate: (
						if $attempt.value == 0
						then 0
						else (($skipped[$attempt.key] // 0) / $attempt.value)
						end
					)
				}
			]
			| map(select(.attempted >= $threshold))
			| sort_by(.skip_rate)
			| reverse
			| .[0:20]
		),
		top_value_operators: (
			[
				$value
				| to_entries[]
				| {
					operator: .key,
					carriers_observed_total: (.value.carriers_observed_total // 0),
					mean_survival_ticks: (
						if (.value.carriers_observed_total // 0) == 0
						then 0
						else ((.value.survival_ticks_sum // 0) / .value.carriers_observed_total)
						end
					),
					mean_offspring_spawned: (
						if (.value.carriers_observed_total // 0) == 0
						then 0
						else ((.value.offspring_spawned_sum // 0) / .value.carriers_observed_total)
						end
					),
					mean_final_energy: (
						if (.value.carriers_observed_total // 0) == 0
						then 0
						else ((.value.final_energy_sum // 0) / .value.carriers_observed_total)
						end
					)
				}
			]
			| map(select(.carriers_observed_total >= $threshold))
			| sort_by(.mean_offspring_spawned)
			| reverse
			| .[0:20]
		)
	}'

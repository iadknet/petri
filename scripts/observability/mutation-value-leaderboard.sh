#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

min_carriers="${1:-1000}"
top_n="${2:-20}"

fetch_json "/v3/simulation/status" | "${JQ_BIN}" \
	--argjson min_carriers "${min_carriers}" \
	--argjson top_n "${top_n}" '
	(.mutation_value_totals_by_operator // {}) as $value
	| [
		$value
		| to_entries[]
		| {
			operator: .key,
			carriers_observed_total: (.value.carriers_observed_total // 0),
			survival_ticks_sum: (.value.survival_ticks_sum // 0),
			offspring_spawned_sum: (.value.offspring_spawned_sum // 0),
			final_energy_sum: (.value.final_energy_sum // 0)
		}
	] as $rows
	| ($rows | map(.carriers_observed_total) | add // 0) as $carrier_total
	| ($rows | map(.survival_ticks_sum) | add // 0) as $survival_total
	| ($rows | map(.offspring_spawned_sum) | add // 0) as $offspring_total
	| ($rows | map(.final_energy_sum) | add // 0) as $energy_total
	| (if $carrier_total == 0 then {
		mean_survival_ticks: 0,
		mean_offspring_spawned: 0,
		mean_final_energy: 0
	   } else {
		mean_survival_ticks: ($survival_total / $carrier_total),
		mean_offspring_spawned: ($offspring_total / $carrier_total),
		mean_final_energy: ($energy_total / $carrier_total)
	   } end) as $global
	| {
		tick,
		population,
		global: {
			carriers_observed_total: $carrier_total,
			mean_survival_ticks: $global.mean_survival_ticks,
			mean_offspring_spawned: $global.mean_offspring_spawned,
			mean_final_energy: $global.mean_final_energy
		},
		leaderboard: (
			$rows
			| map(select(.carriers_observed_total >= $min_carriers))
			| map(
				. + {
					mean_survival_ticks: (
						if .carriers_observed_total == 0 then 0
						else (.survival_ticks_sum / .carriers_observed_total)
						end
					),
					mean_offspring_spawned: (
						if .carriers_observed_total == 0 then 0
						else (.offspring_spawned_sum / .carriers_observed_total)
						end
					),
					mean_final_energy: (
						if .carriers_observed_total == 0 then 0
						else (.final_energy_sum / .carriers_observed_total)
						end
					)
				}
			)
			| map(
				. + {
					delta_vs_global: {
						survival_ticks: (.mean_survival_ticks - $global.mean_survival_ticks),
						offspring_spawned: (.mean_offspring_spawned - $global.mean_offspring_spawned),
						final_energy: (.mean_final_energy - $global.mean_final_energy)
					}
				}
			)
			| map(
				. + {
					above_global_metric_count: (
						(if .delta_vs_global.survival_ticks > 0 then 1 else 0 end)
						+ (if .delta_vs_global.offspring_spawned > 0 then 1 else 0 end)
						+ (if .delta_vs_global.final_energy > 0 then 1 else 0 end)
					)
				}
			)
			| sort_by(.mean_offspring_spawned, .mean_survival_ticks, .mean_final_energy)
			| reverse
			| .[0:$top_n]
		)
	}'

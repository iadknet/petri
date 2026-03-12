#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

sample_size="${1:-200}"
snapshot_json="$(fetch_json "/v3/simulation/snapshot?zoom_tier=detail")"
status_json="$(fetch_json "/v3/simulation/status")"

total=0
missing_diagnostics=0
barrier_neighbors=0
neighbor_with_reader=0
neighbor_with_reader_and_writer=0
neighbor_with_reader_no_writer=0
neighbor_with_no_reader=0
neighbor_with_any_recent_failure=0
neighbor_with_reader_writer_recent_failure=0
neighbor_with_reader_no_writer_recent_failure=0
neighbor_with_no_reader_recent_failure=0

while IFS= read -r id; do
	[[ -z "${id}" ]] && continue

	payload="$(fetch_json "/v3/simulation/creature/${id}?exclude=genome,shared_memory,action_log")"
	total=$((total + 1))

	has_diagnostics="$(echo "${payload}" | "${JQ_BIN}" 'if .diagnostics == null then 0 else 1 end')"
	if [[ "${has_diagnostics}" -eq 0 ]]; then
		missing_diagnostics=$((missing_diagnostics + 1))
		continue
	fi

	neighbor_barriers_count="$(echo "${payload}" | "${JQ_BIN}" '[(.diagnostics.current_inputs.neighbor_barrier // [])[] | select(. > 0)] | length')"
	if [[ "${neighbor_barriers_count}" -eq 0 ]]; then
		continue
	fi
	barrier_neighbors=$((barrier_neighbors + 1))

	has_reader="$(echo "${payload}" | "${JQ_BIN}" 'if (.diagnostics.live_circuit.barrier_reader_reachable_node_count // 0) > 0 then 1 else 0 end')"
	has_writer="$(echo "${payload}" | "${JQ_BIN}" 'if (.diagnostics.live_circuit.barrier_decision_writer_reachable_node_count // 0) > 0 then 1 else 0 end')"
	has_recent_failure="$(echo "${payload}" | "${JQ_BIN}" 'if ((.diagnostics.recent_actions.blocked_move_count // 0) > 0 or (.diagnostics.recent_actions.invalid_target_reproduce_count // 0) > 0) then 1 else 0 end')"

	if [[ "${has_recent_failure}" -eq 1 ]]; then
		neighbor_with_any_recent_failure=$((neighbor_with_any_recent_failure + 1))
	fi

	if [[ "${has_reader}" -eq 1 ]]; then
		neighbor_with_reader=$((neighbor_with_reader + 1))
		if [[ "${has_writer}" -eq 1 ]]; then
			neighbor_with_reader_and_writer=$((neighbor_with_reader_and_writer + 1))
			if [[ "${has_recent_failure}" -eq 1 ]]; then
				neighbor_with_reader_writer_recent_failure=$((neighbor_with_reader_writer_recent_failure + 1))
			fi
		else
			neighbor_with_reader_no_writer=$((neighbor_with_reader_no_writer + 1))
			if [[ "${has_recent_failure}" -eq 1 ]]; then
				neighbor_with_reader_no_writer_recent_failure=$((neighbor_with_reader_no_writer_recent_failure + 1))
			fi
		fi
	else
		neighbor_with_no_reader=$((neighbor_with_no_reader + 1))
		if [[ "${has_recent_failure}" -eq 1 ]]; then
			neighbor_with_no_reader_recent_failure=$((neighbor_with_no_reader_recent_failure + 1))
		fi
	fi
done <<< "$(sample_creature_ids "${sample_size}" "${snapshot_json}")"

echo "${status_json}" | "${JQ_BIN}" \
	--argjson sample_size "${total}" \
	--argjson missing_diagnostics "${missing_diagnostics}" \
	--argjson barrier_neighbors "${barrier_neighbors}" \
	--argjson neighbor_with_reader "${neighbor_with_reader}" \
	--argjson neighbor_with_reader_and_writer "${neighbor_with_reader_and_writer}" \
	--argjson neighbor_with_reader_no_writer "${neighbor_with_reader_no_writer}" \
	--argjson neighbor_with_no_reader "${neighbor_with_no_reader}" \
	--argjson neighbor_with_any_recent_failure "${neighbor_with_any_recent_failure}" \
	--argjson neighbor_with_reader_writer_recent_failure "${neighbor_with_reader_writer_recent_failure}" \
	--argjson neighbor_with_reader_no_writer_recent_failure "${neighbor_with_reader_no_writer_recent_failure}" \
	--argjson neighbor_with_no_reader_recent_failure "${neighbor_with_no_reader_recent_failure}" '
	{
		tick,
		population,
		sample: {
			size: $sample_size,
			missing_diagnostics: $missing_diagnostics,
			missing_diagnostics_ratio: (if $sample_size == 0 then 0 else ($missing_diagnostics / $sample_size) end),
			with_barrier_neighbors: $barrier_neighbors,
			with_barrier_neighbors_ratio: (if $sample_size == 0 then 0 else ($barrier_neighbors / $sample_size) end)
		},
		funnel: {
			barrier_neighbors: $barrier_neighbors,
			barrier_neighbors_with_reader: $neighbor_with_reader,
			barrier_neighbors_with_reader_and_decision_writer: $neighbor_with_reader_and_writer,
			barrier_neighbors_with_reader_without_decision_writer: $neighbor_with_reader_no_writer,
			barrier_neighbors_without_reader: $neighbor_with_no_reader
		},
		recent_failure_rates: {
			any_barrier_neighbor: {
				count: $neighbor_with_any_recent_failure,
				ratio: (if $barrier_neighbors == 0 then 0 else ($neighbor_with_any_recent_failure / $barrier_neighbors) end)
			},
			barrier_reader_with_decision_writer: {
				count: $neighbor_with_reader_writer_recent_failure,
				ratio: (if $neighbor_with_reader_and_writer == 0 then 0 else ($neighbor_with_reader_writer_recent_failure / $neighbor_with_reader_and_writer) end)
			},
			barrier_reader_without_decision_writer: {
				count: $neighbor_with_reader_no_writer_recent_failure,
				ratio: (if $neighbor_with_reader_no_writer == 0 then 0 else ($neighbor_with_reader_no_writer_recent_failure / $neighbor_with_reader_no_writer) end)
			},
			no_barrier_reader: {
				count: $neighbor_with_no_reader_recent_failure,
				ratio: (if $neighbor_with_no_reader == 0 then 0 else ($neighbor_with_no_reader_recent_failure / $neighbor_with_no_reader) end)
			}
		},
		status_counters: {
			move_attempts_with_barrier_neighbor_total_by_reader_state: (.move_attempts_with_barrier_neighbor_total_by_reader_state // {}),
			move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: (.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state // {}),
			move_actions_blocked_avoidable_total_by_reader_state: (.move_actions_blocked_avoidable_total_by_reader_state // {}),
			reproduction_attempts_with_barrier_neighbor_total_by_reader_state: (.reproduction_attempts_with_barrier_neighbor_total_by_reader_state // {}),
			reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state: (.reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state // {}),
			reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state: (.reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state // {})
		}
	}'

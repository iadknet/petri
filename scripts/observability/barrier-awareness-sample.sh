#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

sample_size="${1:-200}"
snapshot_json="$(fetch_json "/v3/simulation/snapshot?zoom_tier=detail")"

total=0
barrier_neighbors_count=0
barrier_reader_count=0
barrier_neighbor_and_reader_count=0
blocked_recent_count=0
invalid_recent_count=0
missing_diagnostics_count=0
reader_total=0
reader_blocked=0
no_reader_neighbor_total=0
no_reader_neighbor_blocked=0
reader_neighbor_total=0
reader_neighbor_blocked=0
barrier_decision_writer_total=0
barrier_reader_without_decision_writer_total=0
barrier_reader_with_decision_writer_total=0
example_neighbor_no_reader=""
example_neighbor_reader=""

while IFS= read -r id; do
	[[ -z "${id}" ]] && continue

	payload="$(fetch_json "/v3/simulation/creature/${id}?exclude=genome,shared_memory,action_log")"
	barrier_neighbors="$(echo "${payload}" | "${JQ_BIN}" '[(.diagnostics.current_inputs.neighbor_barrier // [])[] | select(. > 0)] | length')"
	barrier_readers="$(echo "${payload}" | "${JQ_BIN}" '(.diagnostics.live_circuit.barrier_reader_reachable_node_count // 0)')"
	barrier_decision_writers="$(echo "${payload}" | "${JQ_BIN}" '(.diagnostics.live_circuit.barrier_decision_writer_reachable_node_count // 0)')"
	barrier_reader_without_decision_writers="$(echo "${payload}" | "${JQ_BIN}" '(.diagnostics.live_circuit.barrier_reader_without_decision_writer_reachable_node_count // 0)')"
	blocked_recent="$(echo "${payload}" | "${JQ_BIN}" '(.diagnostics.recent_actions.blocked_move_count // 0)')"
	invalid_recent="$(echo "${payload}" | "${JQ_BIN}" '(.diagnostics.recent_actions.invalid_target_reproduce_count // 0)')"
	has_diagnostics="$(echo "${payload}" | "${JQ_BIN}" 'if .diagnostics == null then 0 else 1 end')"

	total=$((total + 1))
	if [[ "${has_diagnostics}" -eq 0 ]]; then
		missing_diagnostics_count=$((missing_diagnostics_count + 1))
	fi
	if [[ "${barrier_neighbors}" -gt 0 ]]; then
		barrier_neighbors_count=$((barrier_neighbors_count + 1))
	fi
	if [[ "${barrier_readers}" -gt 0 ]]; then
		barrier_reader_count=$((barrier_reader_count + 1))
		reader_total=$((reader_total + 1))
	fi
	if [[ "${barrier_neighbors}" -gt 0 && "${barrier_readers}" -gt 0 ]]; then
		barrier_neighbor_and_reader_count=$((barrier_neighbor_and_reader_count + 1))
		reader_neighbor_total=$((reader_neighbor_total + 1))
	fi
	if [[ "${blocked_recent}" -gt 0 ]]; then
		blocked_recent_count=$((blocked_recent_count + 1))
	fi
	if [[ "${invalid_recent}" -gt 0 ]]; then
		invalid_recent_count=$((invalid_recent_count + 1))
	fi
	if [[ "${barrier_readers}" -gt 0 && "${blocked_recent}" -gt 0 ]]; then
		reader_blocked=$((reader_blocked + 1))
	fi
	if [[ "${barrier_decision_writers}" -gt 0 ]]; then
		barrier_decision_writer_total=$((barrier_decision_writer_total + 1))
	fi
	if [[ "${barrier_reader_without_decision_writers}" -gt 0 ]]; then
		barrier_reader_without_decision_writer_total=$((barrier_reader_without_decision_writer_total + 1))
	fi
	if [[ "${barrier_readers}" -gt 0 && "${barrier_decision_writers}" -gt 0 ]]; then
		barrier_reader_with_decision_writer_total=$((barrier_reader_with_decision_writer_total + 1))
	fi
	if [[ "${barrier_neighbors}" -gt 0 && "${barrier_readers}" -eq 0 ]]; then
		no_reader_neighbor_total=$((no_reader_neighbor_total + 1))
		if [[ "${blocked_recent}" -gt 0 ]]; then
			no_reader_neighbor_blocked=$((no_reader_neighbor_blocked + 1))
		fi
	fi
	if [[ "${barrier_neighbors}" -gt 0 && "${barrier_readers}" -gt 0 && "${blocked_recent}" -gt 0 ]]; then
		reader_neighbor_blocked=$((reader_neighbor_blocked + 1))
	fi

	if [[ "${barrier_neighbors}" -gt 0 && "${barrier_readers}" -eq 0 && -z "${example_neighbor_no_reader}" ]]; then
		example_neighbor_no_reader="$(echo "${payload}" | "${JQ_BIN}" -c '{
			id,
			generation,
			energy,
			barrier_neighbors: ([.diagnostics.current_inputs.neighbor_barrier[] | select(. > 0)] | length),
			barrier_readers: .diagnostics.live_circuit.barrier_reader_reachable_node_count,
			barrier_decision_writers: (.diagnostics.live_circuit.barrier_decision_writer_reachable_node_count // 0),
			barrier_readers_without_decision_writers: (
				.diagnostics.live_circuit.barrier_reader_without_decision_writer_reachable_node_count // 0
			),
			read_classes: .diagnostics.live_circuit.reachable_read_class_counts,
			recent: .diagnostics.recent_actions.by_action_result
		}')"
	fi

	if [[ "${barrier_neighbors}" -gt 0 && "${barrier_readers}" -gt 0 && -z "${example_neighbor_reader}" ]]; then
		example_neighbor_reader="$(echo "${payload}" | "${JQ_BIN}" -c '{
			id,
			generation,
			energy,
			barrier_neighbors: ([.diagnostics.current_inputs.neighbor_barrier[] | select(. > 0)] | length),
			barrier_readers: .diagnostics.live_circuit.barrier_reader_reachable_node_count,
			barrier_decision_writers: (.diagnostics.live_circuit.barrier_decision_writer_reachable_node_count // 0),
			barrier_readers_without_decision_writers: (
				.diagnostics.live_circuit.barrier_reader_without_decision_writer_reachable_node_count // 0
			),
			read_classes: .diagnostics.live_circuit.reachable_read_class_counts,
			recent: .diagnostics.recent_actions.by_action_result
		}')"
	fi
done <<< "$(sample_creature_ids "${sample_size}" "${snapshot_json}")"

"${JQ_BIN}" -n \
	--argjson total "${total}" \
	--argjson barrier_neighbors_count "${barrier_neighbors_count}" \
	--argjson barrier_reader_count "${barrier_reader_count}" \
	--argjson barrier_neighbor_and_reader_count "${barrier_neighbor_and_reader_count}" \
	--argjson blocked_recent_count "${blocked_recent_count}" \
	--argjson invalid_recent_count "${invalid_recent_count}" \
	--argjson missing_diagnostics_count "${missing_diagnostics_count}" \
	--argjson reader_total "${reader_total}" \
	--argjson reader_blocked "${reader_blocked}" \
	--argjson no_reader_neighbor_total "${no_reader_neighbor_total}" \
	--argjson no_reader_neighbor_blocked "${no_reader_neighbor_blocked}" \
	--argjson reader_neighbor_total "${reader_neighbor_total}" \
	--argjson reader_neighbor_blocked "${reader_neighbor_blocked}" \
	--argjson barrier_decision_writer_total "${barrier_decision_writer_total}" \
	--argjson barrier_reader_without_decision_writer_total "${barrier_reader_without_decision_writer_total}" \
	--argjson barrier_reader_with_decision_writer_total "${barrier_reader_with_decision_writer_total}" \
	--arg example_neighbor_no_reader "${example_neighbor_no_reader}" \
	--arg example_neighbor_reader "${example_neighbor_reader}" '
	{
		sample_size: $total,
		missing_diagnostics_count: $missing_diagnostics_count,
		missing_diagnostics_ratio: (if $total == 0 then 0 else ($missing_diagnostics_count / $total) end),
		with_barrier_neighbors: $barrier_neighbors_count,
		with_barrier_neighbors_ratio: (if $total == 0 then 0 else ($barrier_neighbors_count / $total) end),
		with_barrier_readers: $barrier_reader_count,
		with_barrier_readers_ratio: (if $total == 0 then 0 else ($barrier_reader_count / $total) end),
		with_barrier_neighbors_and_readers: $barrier_neighbor_and_reader_count,
		with_recent_blocked_moves: $blocked_recent_count,
		with_recent_invalid_repro: $invalid_recent_count,
		barrier_causal_path: {
			with_barrier_decision_writers: $barrier_decision_writer_total,
			with_barrier_readers_without_decision_writers: $barrier_reader_without_decision_writer_total,
			with_barrier_readers_and_decision_writers: $barrier_reader_with_decision_writer_total
		},
		barrier_readers: {
			count: $reader_total,
			blocked_recent_count: $reader_blocked,
			blocked_recent_ratio: (if $reader_total == 0 then 0 else ($reader_blocked / $reader_total) end)
		},
		barrier_neighbors_without_readers: {
			count: $no_reader_neighbor_total,
			blocked_recent_count: $no_reader_neighbor_blocked,
			blocked_recent_ratio: (
				if $no_reader_neighbor_total == 0
				then 0
				else ($no_reader_neighbor_blocked / $no_reader_neighbor_total)
				end
			)
		},
		barrier_neighbors_with_readers: {
			count: $reader_neighbor_total,
			blocked_recent_count: $reader_neighbor_blocked,
			blocked_recent_ratio: (
				if $reader_neighbor_total == 0
				then 0
				else ($reader_neighbor_blocked / $reader_neighbor_total)
				end
			)
		},
		example_barrier_neighbor_without_reader: (
			if ($example_neighbor_no_reader | length) > 0
			then ($example_neighbor_no_reader | fromjson)
			else null
			end
		),
		example_barrier_neighbor_with_reader: (
			if ($example_neighbor_reader | length) > 0
			then ($example_neighbor_reader | fromjson)
			else null
			end
		)
	}'

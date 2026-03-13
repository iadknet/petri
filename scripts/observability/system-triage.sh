#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

sample_size="${1:-200}"

config_json="$(fetch_json "/v3/simulation/config")"
world_width="$(echo "${config_json}" | "${JQ_BIN}" -r '.config.world.width // 400')"
world_height="$(echo "${config_json}" | "${JQ_BIN}" -r '.config.world.height // 400')"
snapshot_json="$(fetch_json "/v3/simulation/snapshot?zoom_tier=detail&x=0&y=0&width=${world_width}&height=${world_height}")"
payload_file="$(mktemp 2>/dev/null || /usr/bin/mktemp)"
trap 'rm -f "${payload_file}"' EXIT

attempted_creature_fetches=0
payload_records_written=0
failed_creature_fetches=0

while IFS= read -r id; do
	[[ -z "${id}" ]] && continue
	attempted_creature_fetches=$((attempted_creature_fetches + 1))
	if payload="$(fetch_json "/v3/simulation/creature/${id}?exclude=shared_memory,action_log")"; then
		if [[ -n "${payload}" ]]; then
			echo "${payload}" >> "${payload_file}"
			echo "" >> "${payload_file}"
			payload_records_written=$((payload_records_written + 1))
		else
			failed_creature_fetches=$((failed_creature_fetches + 1))
		fi
	else
		failed_creature_fetches=$((failed_creature_fetches + 1))
	fi
done <<< "$(sample_creature_ids "${sample_size}" "${snapshot_json}")"

sampled_creatures=0
reachable_nodes_total=0
action_writer_reachable_nodes_total=0
sensor_action_reachable_nodes_total=0
creatures_with_sensor_action_path=0
live_read_input_total=0
invalid_live_read_input_total=0
live_read_world_input_total=0
live_read_area_world_input_total=0
typeset -A live_world_input_counts

while IFS= read -r payload; do
	[[ -z "${payload}" ]] && continue
	is_valid="$(echo "${payload}" | "${JQ_BIN}" -r 'if .genome != null and .mesh_annotations != null then 1 else 0 end' 2>/dev/null || echo 0)"
	[[ "${is_valid}" -ne 1 ]] && continue

	sampled_creatures=$((sampled_creatures + 1))

	creature_counts="$(echo "${payload}" | "${JQ_BIN}" -r '
		(.mesh_annotations // []) as $annotations
		| ($annotations | map(select(.reachable == true))) as $reachable
		| [
			($reachable | length),
			([$reachable[] | select((.write_classes // []) | index("action") != null)] | length),
			([
				$reachable[]
				| select(
					((.write_classes // []) | index("action") != null)
					and ((.read_classes // []) | length > 0)
				)
			] | length)
		]
		| @tsv
	')"
	creature_reachable_nodes=0
	creature_action_writer_nodes=0
	creature_sensor_action_nodes=0
	IFS=$'\t' read -r creature_reachable_nodes creature_action_writer_nodes creature_sensor_action_nodes <<< "${creature_counts}"

	reachable_nodes_total=$((reachable_nodes_total + creature_reachable_nodes))
	action_writer_reachable_nodes_total=$((action_writer_reachable_nodes_total + creature_action_writer_nodes))
	sensor_action_reachable_nodes_total=$((sensor_action_reachable_nodes_total + creature_sensor_action_nodes))
	if [[ "${creature_sensor_action_nodes}" -gt 0 ]]; then
		creatures_with_sensor_action_path=$((creatures_with_sensor_action_path + 1))
	fi

	while IFS= read -r token; do
		[[ -z "${token}" ]] && continue
		live_read_input_total=$((live_read_input_total + 1))
		if [[ "${token}" == "__INVALID__" ]]; then
			invalid_live_read_input_total=$((invalid_live_read_input_total + 1))
		elif [[ "${token}" == "__NON_WORLD__" ]]; then
			:
		else
			live_read_world_input_total=$((live_read_world_input_total + 1))
			if [[ "${token}" == Area* ]]; then
				live_read_area_world_input_total=$((live_read_area_world_input_total + 1))
			fi
			if [[ -n "${live_world_input_counts[${token}]:-}" ]]; then
				live_world_input_counts[${token}]=$((live_world_input_counts[${token}] + 1))
			else
				live_world_input_counts[${token}]=1
			fi
		fi
	done <<< "$(echo "${payload}" | "${JQ_BIN}" -r '
		(.genome.nodes // []) as $nodes
		| (.mesh_annotations // [])
		| map(select(.reachable == true))
		| .[] as $annotation
		| (first($nodes[]? | select(.node_id == $annotation.node_id))) as $node
		| select($node.backend_def.Vm? != null)
		| ($node.backend_def.Vm.program // []) as $program
		| ($node.input_refs // []) as $input_refs
		| ($annotation.live_instruction_indices // [])[]
		| ($program[.] .ReadInput? // empty) as $read_input
		| if (($read_input.ref_idx // 0) >= ($input_refs | length)) then
			"__INVALID__"
		  else
			($input_refs[$read_input.ref_idx].World? // "__NON_WORLD__")
		  end
	')"
done < "${payload_file}"

world_input_counts_file="$(mktemp 2>/dev/null || /usr/bin/mktemp)"
trap 'rm -f "${payload_file}" "${world_input_counts_file}"' EXIT
for key in "${(@k)live_world_input_counts}"; do
	echo "${key}	${live_world_input_counts[${key}]}" >> "${world_input_counts_file}"
done
world_input_counts_json="$("${JQ_BIN}" -Rn '
	[
		inputs
		| select(length > 0)
		| split("\t")
		| {(.[0]): (.[1] | tonumber)}
	]
	| add // {}
' < "${world_input_counts_file}")"

invalid_live_read_ratio="$("${AWK_BIN}" -v n="${invalid_live_read_input_total}" -v d="${live_read_input_total}" 'BEGIN { if (d == 0) print 0; else print n / d; }')"
sensor_action_coverage_ratio="$("${AWK_BIN}" -v n="${sensor_action_reachable_nodes_total}" -v d="${action_writer_reachable_nodes_total}" 'BEGIN { if (d == 0) print 0; else print n / d; }')"
area_world_input_read_ratio="$("${AWK_BIN}" -v n="${live_read_area_world_input_total}" -v d="${live_read_world_input_total}" 'BEGIN { if (d == 0) print 0; else print n / d; }')"
creature_sensor_action_path_ratio="$("${AWK_BIN}" -v n="${creatures_with_sensor_action_path}" -v d="${sampled_creatures}" 'BEGIN { if (d == 0) print 0; else print n / d; }')"

execution_json="$("${JQ_BIN}" -n \
	--argjson sampled_creatures "${sampled_creatures}" \
	--argjson reachable_nodes_total "${reachable_nodes_total}" \
	--argjson action_writer_reachable_nodes_total "${action_writer_reachable_nodes_total}" \
	--argjson sensor_action_reachable_nodes_total "${sensor_action_reachable_nodes_total}" \
	--argjson creatures_with_sensor_action_path "${creatures_with_sensor_action_path}" \
	--argjson live_read_input_total "${live_read_input_total}" \
	--argjson invalid_live_read_input_total "${invalid_live_read_input_total}" \
	--argjson live_read_world_input_total "${live_read_world_input_total}" \
	--argjson live_read_area_world_input_total "${live_read_area_world_input_total}" \
	--argjson live_read_world_input_by_key "${world_input_counts_json}" \
	--argjson invalid_live_read_ratio "${invalid_live_read_ratio}" \
	--argjson sensor_action_coverage_ratio "${sensor_action_coverage_ratio}" \
	--argjson area_world_input_read_ratio "${area_world_input_read_ratio}" \
	--argjson creature_sensor_action_path_ratio "${creature_sensor_action_path_ratio}" \
	'{
		sampled_creatures: $sampled_creatures,
		reachable_nodes_total: $reachable_nodes_total,
		action_writer_reachable_nodes_total: $action_writer_reachable_nodes_total,
		sensor_action_reachable_nodes_total: $sensor_action_reachable_nodes_total,
		creatures_with_sensor_action_path: $creatures_with_sensor_action_path,
		live_read_input_total: $live_read_input_total,
		invalid_live_read_input_total: $invalid_live_read_input_total,
		live_read_world_input_total: $live_read_world_input_total,
		live_read_area_world_input_total: $live_read_area_world_input_total,
		live_read_world_input_by_key: $live_read_world_input_by_key,
		invalid_live_read_ratio: $invalid_live_read_ratio,
		sensor_action_coverage_ratio: $sensor_action_coverage_ratio,
		area_world_input_read_ratio: $area_world_input_read_ratio,
		creature_sensor_action_path_ratio: $creature_sensor_action_path_ratio
	}
')"

status_json="$(fetch_json "/v3/simulation/status")"
awareness_json="$("${SCRIPT_DIR}/barrier-awareness-sample.sh" "${sample_size}")"
funnel_json="$("${SCRIPT_DIR}/barrier-causal-funnel.sh" "${sample_size}")"

"${JQ_BIN}" -n \
	--argjson sample_size "${sample_size}" \
	--argjson attempted_creature_fetches "${attempted_creature_fetches}" \
	--argjson payload_records_written "${payload_records_written}" \
	--argjson failed_creature_fetches "${failed_creature_fetches}" \
	--argjson status "${status_json}" \
	--argjson execution "${execution_json}" \
	--argjson awareness "${awareness_json}" \
	--argjson funnel "${funnel_json}" '
	($status.mutation_events_applied_total // 0) as $applied
	| ($status.mutation_events_skipped_total // 0) as $skipped
	| ($applied + $skipped) as $mutation_attempts
	| ($status.mutation_outcome_summary.helpful_total // 0) as $helpful
	| ($status.mutation_outcome_summary.neutral_total // 0) as $neutral
	| ($status.mutation_outcome_summary.detrimental_total // 0) as $detrimental
	| ($helpful + $neutral + $detrimental) as $outcome_total
	| ($execution.invalid_live_read_ratio <= 0.05) as $execution_ref_integrity_pass
	| ($execution.sensor_action_coverage_ratio >= 0.25) as $execution_sensor_action_pass
	| ($execution_ref_integrity_pass and $execution_sensor_action_pass) as $execution_pass
	| (
		($mutation_attempts > 0)
		and (($skipped / $mutation_attempts) <= 0.2)
		and (
			if $outcome_total == 0 then false
			else (($helpful / $outcome_total) >= ($detrimental / $outcome_total))
			end
		)
	) as $mutation_signal_pass
	| (
		($awareness.with_barrier_neighbors_ratio // 0) >= 0.2
		and ($awareness.with_barrier_readers_ratio // 0) >= 0.1
		and ($execution.area_world_input_read_ratio >= 0.02)
	) as $cognition_utilization_pass
	| (if ($execution_pass | not) then "execution"
	   elif ($mutation_signal_pass | not) then "mutation"
	   elif ($cognition_utilization_pass | not) then "cognition"
	   else "none_detected"
	   end) as $primary_layer
	| {
		tick: ($status.tick // null),
		population: ($status.population // null),
		sample_size_requested: $sample_size,
		creature_payload_fetches: {
			attempted: $attempted_creature_fetches,
			written: $payload_records_written,
			failed: $failed_creature_fetches
		},
		thresholds: {
			execution_max_invalid_live_read_ratio: 0.05,
			execution_min_sensor_action_coverage_ratio: 0.25,
			mutation_max_skip_ratio: 0.2,
			cognition_min_barrier_neighbor_ratio: 0.2,
			cognition_min_barrier_reader_ratio: 0.1,
			cognition_min_area_world_input_read_ratio: 0.02
		},
		checks: {
			execution: {
				pass: $execution_pass,
				ref_integrity_pass: $execution_ref_integrity_pass,
				sensor_action_pass: $execution_sensor_action_pass
			},
			mutation_signal: {
				pass: $mutation_signal_pass
			},
			cognition_utilization: {
				pass: $cognition_utilization_pass
			}
		},
		likely_primary_failure_layer: $primary_layer,
		execution: $execution,
		mutation_signal: {
			mutation_events_applied_total: $applied,
			mutation_events_skipped_total: $skipped,
			mutation_skip_ratio: (
				if $mutation_attempts == 0 then 0
				else ($skipped / $mutation_attempts)
				end
			),
			outcomes: {
				helpful_total: $helpful,
				neutral_total: $neutral,
				detrimental_total: $detrimental,
				helpful_ratio: (
					if $outcome_total == 0 then 0
					else ($helpful / $outcome_total)
					end
				),
				neutral_ratio: (
					if $outcome_total == 0 then 0
					else ($neutral / $outcome_total)
					end
				),
				detrimental_ratio: (
					if $outcome_total == 0 then 0
					else ($detrimental / $outcome_total)
					end
				)
			}
		},
		cognition: {
			barrier_awareness_sample: $awareness,
			barrier_causal_funnel: $funnel,
			area_world_input_read_ratio: ($execution.area_world_input_read_ratio // 0)
		}
	}
'

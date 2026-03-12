#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

COLLECT_SCRIPT="${SCRIPT_DIR}/collect-metrics.sh"
SUMMARIZE_SCRIPT="${SCRIPT_DIR}/summarize-suite.sh"

SUITE_FILE="${1:-}"
OUTPUT_ROOT="${2:-${ROOT_DIR}/artifacts/experiments}"
SAMPLE_SIZE="${EXPERIMENT_SAMPLE_SIZE:-300}"

if [[ -z "${SUITE_FILE}" ]]; then
	echo "Usage: $0 <suite-manifest.json> [output-root]" >&2
	exit 1
fi

if [[ ! -f "${SUITE_FILE}" ]]; then
	echo "Suite manifest not found: ${SUITE_FILE}" >&2
	exit 1
fi

SUITE_JSON="$("${JQ_BIN}" -c '.' "${SUITE_FILE}")"
SUITE_ID="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.suite_id // empty')"
OBJECTIVE="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.objective // ""')"
TARGET_TICKS="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.run.target_ticks // 1000')"
STEP_BATCH="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.run.step_batch // 500')"
SEED_COUNT="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '(.seeds // []) | length')"
VARIANT_COUNT="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '(.variants // []) | length')"
SUITE_API_BASE="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.api_base // empty')"
RUN_TOTAL=$((SEED_COUNT * VARIANT_COUNT))

if [[ -z "${SUITE_ID}" ]]; then
	echo "Suite manifest is missing required key: suite_id" >&2
	exit 1
fi

if (( TARGET_TICKS < 1 )); then
	echo "run.target_ticks must be >= 1" >&2
	exit 1
fi

if (( STEP_BATCH < 1 )); then
	echo "run.step_batch must be >= 1" >&2
	exit 1
fi

if (( SEED_COUNT == 0 )); then
	echo "Suite manifest must include at least one seed" >&2
	exit 1
fi

if (( VARIANT_COUNT == 0 )); then
	echo "Suite manifest must include at least one variant" >&2
	exit 1
fi

if [[ -n "${SUITE_API_BASE}" ]]; then
	export API_BASE="${SUITE_API_BASE}"
fi

SUITE_TIMESTAMP="$(timestamp_utc)"
SUITE_DIR="${OUTPUT_ROOT}/${SUITE_ID}/${SUITE_TIMESTAMP}"
mkdir -p "${SUITE_DIR}"
SUITE_STARTED_EPOCH="$("${DATE_BIN}" +%s)"
RUN_INDEX=0

echo "${SUITE_JSON}" | "${JQ_BIN}" '.' > "${SUITE_DIR}/suite_manifest_snapshot.json"
echo "${SUITE_JSON}" | "${JQ_BIN}" \
	--arg started_at "${SUITE_TIMESTAMP}" \
	--arg api_base "${API_BASE}" \
	'{suite_id, objective, run, seeds, variants: [.variants[].id], started_at: $started_at, api_base: $api_base}' \
	> "${SUITE_DIR}/suite_run_metadata.json"

DEFAULT_PATTERNS="$(echo "${SUITE_JSON}" | "${JQ_BIN}" -c '.default_patterns // []')"

post_json_checked() {
	local path="${1:?path is required}"
	local payload="${2:?payload is required}"
	local output_path="${3:?output path is required}"
	local response_file
	response_file="$(mktemp 2>/dev/null || /usr/bin/mktemp)"
	local http_code
	http_code="$("${CURL_BIN}" -sS \
		-o "${response_file}" \
		-w "%{http_code}" \
		-X POST \
		-H "content-type: application/json" \
		--data "${payload}" \
		"${API_BASE}${path}")"

	if [[ "${http_code}" != 2* ]]; then
		echo "Request failed: POST ${path} (HTTP ${http_code})" >&2
		/bin/cat "${response_file}" >&2
		/bin/rm -f "${response_file}"
		return 1
	fi

	"${JQ_BIN}" '.' "${response_file}" > "${output_path}"
	/bin/rm -f "${response_file}"
}

echo "Running suite '${SUITE_ID}' at ${SUITE_DIR}"
echo "Objective: ${OBJECTIVE}"
echo "Seeds: ${SEED_COUNT}, Variants: ${VARIANT_COUNT}, Runs: ${RUN_TOTAL}, Target ticks: ${TARGET_TICKS}, Step batch: ${STEP_BATCH}"
echo "API base: ${API_BASE}"

while IFS= read -r seed; do
	[[ -z "${seed}" ]] && continue
	while IFS= read -r variant_json; do
		[[ -z "${variant_json}" ]] && continue
		variant_id="$(echo "${variant_json}" | "${JQ_BIN}" -r '.id')"
		startup_patch="$(echo "${variant_json}" | "${JQ_BIN}" -c '.startup_patch // {}')"
		variant_patterns="$(echo "${variant_json}" | "${JQ_BIN}" -c '.patterns // []')"
		all_patterns="$("${JQ_BIN}" -cn \
			--argjson defaults "${DEFAULT_PATTERNS}" \
			--argjson variant "${variant_patterns}" \
			'$defaults + $variant')"
		run_id="seed-${seed}__${variant_id}"
		run_dir="${SUITE_DIR}/${run_id}"
		mkdir -p "${run_dir}"
		RUN_INDEX=$((RUN_INDEX + 1))
		run_started_epoch="$("${DATE_BIN}" +%s)"

		echo ""
		echo "=== [${RUN_INDEX}/${RUN_TOTAL}] ${run_id} ==="

		startup_payload="$("${JQ_BIN}" -cn --argjson seed "${seed}" --argjson patch "${startup_patch}" '$patch + {seed: $seed}')"
		echo "${startup_payload}" | "${JQ_BIN}" '.' > "${run_dir}/startup_payload.json"
		echo "${variant_json}" | "${JQ_BIN}" '.' > "${run_dir}/variant_manifest.json"

		post_json_checked "/v3/simulation/startup" "${startup_payload}" "${run_dir}/startup_response.json"
		echo "  startup: ok"

		pattern_index=0
		while IFS= read -r pattern_payload; do
			[[ -z "${pattern_payload}" ]] && continue
			post_json_checked \
				"/v3/simulation/pattern/apply" \
				"${pattern_payload}" \
				"${run_dir}/pattern_apply_${pattern_index}.json"
			pattern_index=$((pattern_index + 1))
		done <<< "$(echo "${all_patterns}" | "${JQ_BIN}" -c '.[]')"
		if (( pattern_index == 0 )); then
			echo "  patterns: none"
		else
			echo "  patterns: applied ${pattern_index}"
		fi

		post_json_checked "/v3/simulation/start" '{}' "${run_dir}/start_response.json"
		post_json_checked "/v3/simulation/pause" '{}' "${run_dir}/pause_response.json"
		echo "  state: running -> paused"

		current_tick="$("${JQ_BIN}" -r '.tick' "${run_dir}/pause_response.json")"
		step_iterations=0
		echo "  stepping: tick ${current_tick}/${TARGET_TICKS}"
		while (( current_tick < TARGET_TICKS )); do
			remaining=$((TARGET_TICKS - current_tick))
			steps="${STEP_BATCH}"
			if (( remaining < STEP_BATCH )); then
				steps="${remaining}"
			fi
			step_payload="$("${JQ_BIN}" -cn --argjson steps "${steps}" '{steps: $steps}')"
			post_json_checked "/v3/simulation/step" "${step_payload}" "${run_dir}/step_response_last.json"
			current_tick="$("${JQ_BIN}" -r '.tick' "${run_dir}/step_response_last.json")"
			step_iterations=$((step_iterations + 1))
			if (( step_iterations == 1 || step_iterations % 10 == 0 || current_tick >= TARGET_TICKS )); then
				progress_pct=$((current_tick * 100 / TARGET_TICKS))
				if (( progress_pct > 100 )); then
					progress_pct=100
				fi
				echo "    progress: tick ${current_tick}/${TARGET_TICKS} (${progress_pct}%)"
			fi
		done

		echo "  metrics: collecting probes"
		if ! "${COLLECT_SCRIPT}" "${run_dir}" "${SAMPLE_SIZE}" > "${run_dir}/collect_metrics.log" 2>&1; then
			echo "  metrics: failed (see ${run_dir}/collect_metrics.log)" >&2
			exit 1
		fi
		echo "  metrics: complete"

		"${JQ_BIN}" -n \
			--arg run_id "${run_id}" \
			--arg variant_id "${variant_id}" \
			--arg objective "${OBJECTIVE}" \
			--argjson seed "${seed}" \
			--argjson target_ticks "${TARGET_TICKS}" \
			--argjson step_batch "${STEP_BATCH}" \
			--argjson sample_size "${SAMPLE_SIZE}" \
			--arg api_base "${API_BASE}" \
			'{
				run_id: $run_id,
				variant_id: $variant_id,
				seed: $seed,
				objective: $objective,
				target_ticks: $target_ticks,
				step_batch: $step_batch,
				sample_size: $sample_size,
				api_base: $api_base
			}' \
			> "${run_dir}/run_metadata.json"

		run_finished_epoch="$("${DATE_BIN}" +%s)"
		run_elapsed_seconds=$((run_finished_epoch - run_started_epoch))
		echo "  done: ${run_elapsed_seconds}s"
	done <<< "$(echo "${SUITE_JSON}" | "${JQ_BIN}" -c '.variants[]')"
done <<< "$(echo "${SUITE_JSON}" | "${JQ_BIN}" -r '.seeds[]')"

"${SUMMARIZE_SCRIPT}" "${SUITE_DIR}" > "${SUITE_DIR}/summary.json"
SUITE_FINISHED_EPOCH="$("${DATE_BIN}" +%s)"
SUITE_ELAPSED_SECONDS=$((SUITE_FINISHED_EPOCH - SUITE_STARTED_EPOCH))
echo ""
echo "Suite complete."
echo "Artifacts: ${SUITE_DIR}"
echo "Summary:   ${SUITE_DIR}/summary.json"
echo "Elapsed:   ${SUITE_ELAPSED_SECONDS}s"

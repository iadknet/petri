#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

OUTPUT_DIR="${1:-}"
SAMPLE_SIZE="${2:-300}"

if [[ -z "${OUTPUT_DIR}" ]]; then
	echo "Usage: $0 <output-dir> [sample-size]" >&2
	exit 1
fi

OBS_DIR="${ROOT_DIR}/scripts/observability"
if [[ ! -d "${OBS_DIR}" ]]; then
	echo "Observability scripts directory not found: ${OBS_DIR}" >&2
	exit 1
fi

# The local simulation endpoint can flap briefly while probes run; use
# conservative retry defaults for suite collection stability.
: "${API_RETRIES:=20}"
: "${API_RETRY_DELAY_SECONDS:=0.5}"
export API_RETRIES
export API_RETRY_DELAY_SECONDS

mkdir -p "${OUTPUT_DIR}"

PROBE_TOTAL=8
PROBE_INDEX=0

run_probe() {
	local probe_name="${1:?probe name is required}"
	local output_file="${2:?output file is required}"
	shift 2
	PROBE_INDEX=$((PROBE_INDEX + 1))
	echo "    probe [${PROBE_INDEX}/${PROBE_TOTAL}] ${probe_name}" >&2
	"$@" > "${OUTPUT_DIR}/${output_file}"
}

api_get_json "/v3/simulation/status" > "${OUTPUT_DIR}/raw_status.json"
api_get_json "/v3/simulation/config" > "${OUTPUT_DIR}/raw_config.json"

run_probe "world_summary" "world_summary.json" "${OBS_DIR}/world-summary.sh"
run_probe "status_summary" "status_summary.json" "${OBS_DIR}/status-summary.sh"
run_probe "health_invariants" "health_invariants.json" "${OBS_DIR}/health-invariants.sh"
run_probe "skip_operator_summary" "skip_operator_summary.json" "${OBS_DIR}/skip-operator-summary.sh"
run_probe "mutation_value_leaderboard" "mutation_value_leaderboard.json" "${OBS_DIR}/mutation-value-leaderboard.sh"
run_probe "system_triage" "system_triage.json" "${OBS_DIR}/system-triage.sh" "${SAMPLE_SIZE}"
run_probe "barrier_awareness_sample" "barrier_awareness_sample.json" "${OBS_DIR}/barrier-awareness-sample.sh" "${SAMPLE_SIZE}"
run_probe "barrier_causal_funnel" "barrier_causal_funnel.json" "${OBS_DIR}/barrier-causal-funnel.sh" "${SAMPLE_SIZE}"

echo "${OUTPUT_DIR}"

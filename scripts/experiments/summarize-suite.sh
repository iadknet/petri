#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

SUITE_DIR="${1:-}"

if [[ -z "${SUITE_DIR}" ]]; then
	echo "Usage: $0 <suite-output-dir>" >&2
	exit 1
fi

if [[ ! -d "${SUITE_DIR}" ]]; then
	echo "Suite output directory not found: ${SUITE_DIR}" >&2
	exit 1
fi

TMP_JSONL="$(mktemp 2>/dev/null || /usr/bin/mktemp)"
trap '/bin/rm -f "${TMP_JSONL}"' EXIT

for run_dir in "${SUITE_DIR}"/seed-*__*; do
	[[ -d "${run_dir}" ]] || continue

	meta_file="${run_dir}/run_metadata.json"
	status_file="${run_dir}/raw_status.json"
	barrier_sample_file="${run_dir}/barrier_awareness_sample.json"
	funnel_file="${run_dir}/barrier_causal_funnel.json"

	[[ -f "${meta_file}" ]] || continue
	[[ -f "${status_file}" ]] || continue
	[[ -f "${barrier_sample_file}" ]] || continue
	[[ -f "${funnel_file}" ]] || continue

	"${JQ_BIN}" -n \
		--slurpfile meta "${meta_file}" \
		--slurpfile status "${status_file}" \
		--slurpfile barrier "${barrier_sample_file}" \
		--slurpfile funnel "${funnel_file}" \
		'
		($status[0].reproduction_actions_attempted_total // 0) as $repro_attempted
		| ($status[0].reproduction_actions_spawned_total // 0) as $repro_spawned
		| ($status[0].tick // 0) as $tick
		| ($status[0].mutation_events_applied_total // 0) as $mut_applied
		| ($status[0].mutation_events_skipped_total // 0) as $mut_skipped
		| ($funnel[0].funnel.barrier_neighbors // 0) as $barrier_neighbors
		| ($funnel[0].funnel.barrier_neighbors_with_reader // 0) as $barrier_neighbors_with_reader
		| ($funnel[0].funnel.barrier_neighbors_with_reader_and_decision_writer // 0) as $barrier_neighbors_with_reader_writer
		| {
			run_id: $meta[0].run_id,
			seed: $meta[0].seed,
			variant_id: $meta[0].variant_id,
			tick_end: $tick,
			population_end: ($status[0].population // 0),
			non_extinct: (($status[0].population // 0) > 0),
			mean_energy_end: ($status[0].mean_energy // 0),
			reproduction_attempted_total: $repro_attempted,
			reproduction_spawned_total: $repro_spawned,
			reproduction_success_ratio: (
				if $repro_attempted == 0 then 0
				else ($repro_spawned / $repro_attempted)
				end
			),
			reproduction_spawned_per_tick: (
				if $tick == 0 then 0
				else ($repro_spawned / $tick)
				end
			),
			mutation_skip_ratio: (
				if ($mut_applied + $mut_skipped) == 0 then 0
				else ($mut_skipped / ($mut_applied + $mut_skipped))
				end
			),
			barrier_neighbor_reader_ratio_sample: (
				$barrier[0].with_barrier_neighbors_and_readers
				/
				(if ($barrier[0].with_barrier_neighbors // 0) == 0
				 then 1
				 else $barrier[0].with_barrier_neighbors
				 end)
			),
			barrier_reader_ratio_funnel: (
				if $barrier_neighbors == 0 then 0
				else ($barrier_neighbors_with_reader / $barrier_neighbors)
				end
			),
			barrier_reader_writer_ratio_funnel: (
				if $barrier_neighbors == 0 then 0
				else ($barrier_neighbors_with_reader_writer / $barrier_neighbors)
				end
			)
		}
		' >> "${TMP_JSONL}"
done

if [[ ! -s "${TMP_JSONL}" ]]; then
	echo "No complete run artifacts found under ${SUITE_DIR}" >&2
	exit 1
fi

"${JQ_BIN}" -s \
	--arg suite_dir "${SUITE_DIR}" \
	'
	{
		suite_dir: $suite_dir,
		run_count: length,
		runs: (sort_by(.variant_id, .seed)),
		best_population_run: (
			sort_by(.population_end) | reverse | .[0]
		),
		best_spawn_per_tick_run: (
			sort_by(.reproduction_spawned_per_tick) | reverse | .[0]
		)
	}
	' "${TMP_JSONL}"

#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/common.zsh"

snapshot_json="$(fetch_json "/v3/simulation/snapshot?zoom_tier=detail")"

echo "${snapshot_json}" | "${JQ_BIN}" '{
	tick,
	world: {
		width: .world_static.width,
		height: .world_static.height,
		barrier_cells: ([.world_static.barrier_mask[] | select(. != 0)] | length),
		barrier_density: (
			([.world_static.barrier_mask[] | select(. != 0)] | length)
			/ ((.world_static.width * .world_static.height) | tonumber)
		)
	},
	creature_count: (.view.creatures | length)
}'

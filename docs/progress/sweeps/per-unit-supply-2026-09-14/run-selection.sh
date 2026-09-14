#!/bin/sh
set -eu
WT=/Users/istefanek/projects/petri/.worktrees/per-unit-supply-probe
S=/private/tmp/claude-501/-Users-istefanek-projects-petri/b1584415-0cb2-4372-b919-1421722d5c02/scratchpad
: > "$S/selection-timing.txt"
for arm in orch-pu005-b09 orch-pu005-b00; do
  start=$(date +%s)
  "$WT/scripts/bench-wait" "$WT/target/release/v3-cli" run \
    --ticks 6000 --sample-every 500 --seed 11 --config "$S/$arm-config.json" \
    > "$S/selection-$arm.ndjson"
  end=$(date +%s)
  echo "$arm wall_seconds=$((end - start))" >> "$S/selection-timing.txt"
done
echo "SELECTION COMPLETE"

#!/bin/sh
# T03.F11 paired long run: cost arm (default genome_replication_cost_per_unit
# 0.1, orchards-in-grassland.json unchanged) vs control arm (same recipe plus
# energy.lifecycle.genome_replication_cost_per_unit 0.0 as its only addition).
# Both arms: seed 11, 12,000 ticks, sample every 500, release binary,
# sequential under scripts/bench-wait.
set -eu

WT=/Users/istefanek/projects/petri/.claude/worktrees/t03-f11
OUT=$WT/docs/progress/sweeps/t03-f11

cd "$WT"

: > "$OUT/timing.txt"

run_arm() {
  arm=$1
  recipe=$2
  cfg_out=$3
  start=$(date +%s)
  "$WT/scripts/bench-wait" "$WT/target/release/v3-cli" run \
    --ticks 12000 --sample-every 500 --seed 11 \
    --config "$recipe" --save-config "$cfg_out" \
    > "$OUT/$arm.ndjson"
  end=$(date +%s)
  echo "$arm wall_seconds=$((end - start))" >> "$OUT/timing.txt"
  echo "$arm done in $((end - start))s"
}

run_arm cost "$WT/experiments/worlds/orchards-in-grassland.json" "$OUT/cost-config.json"
run_arm control "$OUT/control-recipe.json" "$OUT/control-config.json"
echo "PAIR COMPLETE"

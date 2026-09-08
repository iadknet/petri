#!/bin/sh
# T03.F08 paired long run: cost arm (default rate) vs control arm (rate 0.0).
# Both arms: 1600x1600, 10,000 founders, seed 11, 12,000 ticks, sample every 500.
set -eu

WT=/Users/istefanek/projects/petri/.claude/worktrees/t03-f08
S=/private/tmp/claude-501/-Users-istefanek-projects-petri/48d48ae4-7808-4176-8a1d-99990757d965/scratchpad

cd "$WT"

run_arm() {
  arm=$1
  cfg=$2
  start=$(date +%s)
  "$WT/scripts/bench-wait" "$WT/target/release/v3-cli" run \
    --ticks 12000 --sample-every 500 --seed 11 --config "$cfg" \
    > "$S/longrun-$arm.ndjson"
  end=$(date +%s)
  echo "$arm wall_seconds=$((end - start))" >> "$S/longrun-timing.txt"
  echo "$arm done in $((end - start))s"
}

: > "$S/longrun-timing.txt"
run_arm cost "$S/cost-config.json"
run_arm control "$S/control-config.json"
echo "PAIR COMPLETE"

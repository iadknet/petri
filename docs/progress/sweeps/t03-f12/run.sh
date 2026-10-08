#!/bin/sh
# T03.F12 long Canyon run (seed 22, 40,000 ticks); run from the feature worktree
# with the release binary built. Raw output stays in the main checkout's ignored tree.
# Two executions, one script: form B (f380899e, shipped) wrote its raw files to "$raw/" and its
# timing.txt; form A (0436dbf6, executed retreat, reverted by user decision 2026-10-08) was run
# after the form-B files were moved aside, and its files now sit in "$raw/form-a/". The form-B
# files were moved back to "$raw/" at the restore (2026-10-08).
set -eu
raw=/Users/istefanek/projects/petri/.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon
mkdir -p "$raw"
# bench-wait waits for host blockers, then execs the run; the timer starts
# after a blocker-free preflight so the wait is not counted.
scripts/bench-wait true
start=$(date +%s)
status=0
scripts/bench-wait target/release/v3-cli run --ticks 40000 --sample-every 500 --seed 22 \
  --config experiments/worlds/canyon-country.json \
  --save-config "$raw/canyon-config.json" > "$raw/canyon.ndjson" || status=$?
end=$(date +%s)
printf 'wall_seconds %s\nexit_status %s\n' "$((end - start))" "$status" > "$raw/timing.txt"
exit "$status"

# Observability Probes

Reusable local probes for inspecting a paused V3 run without reconstructing ad
hoc `curl` and `jq` commands in the terminal.

Default target:

```bash
API_BASE=http://localhost:3000
```

Override it when needed:

```bash
API_BASE=http://localhost:3100 ./scripts/observability/status-summary.sh
```

Available probes:

```bash
./scripts/observability/status-summary.sh
./scripts/observability/world-summary.sh
./scripts/observability/skip-operator-summary.sh 1000
./scripts/observability/barrier-awareness-sample.sh 200
./scripts/observability/barrier-causal-funnel.sh 200
./scripts/observability/mutation-value-leaderboard.sh 1000 20
./scripts/observability/health-invariants.sh
```

What each probe returns:

- `status-summary.sh`
  Current high-level run state plus blocked-action cause/avoidability,
  mutation reachability, and per-operator mutation value totals.
- `world-summary.sh`
  World dimensions, barrier cell count/density, and visible creature count from
  a detail snapshot.
- `skip-operator-summary.sh`
  Operator skip-rate ranking plus operator value ranking (mean offspring,
  survival, final energy) from live status telemetry. Positional argument is the
  minimum attempt/carrier threshold.
- `barrier-awareness-sample.sh`
  Randomized creature sample from the current detail snapshot. Reports barrier
  adjacency prevalence, reachable barrier-reader prevalence, barrier
  reader-to-decision causal-path counts, recent blocked-action rates, and one
  example creature with and without live barrier readers.
- `barrier-causal-funnel.sh`
  Focused sampled funnel for barrier contexts: barrier neighbors -> barrier
  readers -> barrier decision writers -> recent failures, plus matching
  cumulative status counters.
- `mutation-value-leaderboard.sh`
  Per-operator mutation value ranking using `mutation_value_totals_by_operator`
  with means and deltas against global means. Arguments:
  `min_carriers` and `top_n`.
- `health-invariants.sh`
  Reconciliation checks for core telemetry invariants. Exits non-zero when any
  invariant fails so it can be used as a CI/smoke gate.

Notes:

- These probes assume the server is paused or at least stable enough that
  multiple creature detail fetches are meaningful.
- `barrier-awareness-sample.sh` intentionally excludes `genome`,
  `shared_memory`, and `action_log` to keep requests lighter while still
  pulling diagnostics.
- `barrier-awareness-sample.sh` is null-safe when diagnostics are missing and
  reports `missing_diagnostics_count` / `missing_diagnostics_ratio`.

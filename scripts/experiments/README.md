# Experiment Harness

Reusable scaffolding for running deterministic, repeatable simulation experiments
and collecting comparable telemetry artifacts.

## What This Provides

- `run-suite.sh`: runs a manifest-defined suite across seeds and variants.
- `collect-metrics.sh`: collects observability probe outputs for one paused run.
- `summarize-suite.sh`: builds a normalized JSON summary across suite runs.

All runs are artifacted under:

`artifacts/experiments/<suite_id>/<timestamp>/`

## Suite Manifest

See template:

- `experiments/templates/suite-template.json`

Included first suite:

- `experiments/suites/founder-sparse-food-v1.json`

## Workflow

1. Start the server locally.
2. Run a suite:

```bash
./scripts/experiments/run-suite.sh experiments/suites/founder-sparse-food-v1.json
```

3. Rebuild summary for a completed suite directory:

```bash
./scripts/experiments/summarize-suite.sh artifacts/experiments/founder-sparse-food-v1/<timestamp>
```

## Notes

- Runs are stepped in paused mode (`/step`) to hit exact target ticks.
- Startup config is patched per variant at `/v3/simulation/startup`.
- Founder genome can be selected via
  `population.founder_profile` in `startup_patch`
  (`"v3_alpha1"` or `"forage_first_sparse"`).
- Optional barrier patterns can be applied deterministically via
  `/v3/simulation/pattern/apply`.
- The harness currently targets API-compatible, config-driven experiments.

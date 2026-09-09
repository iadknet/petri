# V3 CLI Contract Spec

Reference specification for the minimal canonical v3alpha1 CLI contract.

Status: Active

Related references:
- `v3-startup-seeding-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-runtime-config-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-evolution-observability-spec.md`

---

## 1. Purpose and Scope

This document defines:
- minimal v3alpha1 CLI mode and ownership posture;
- canonical NDJSON event schemas for run output;
- required protocol-version field for CLI events;
- recipe save/load and recipe-backed sweep config metadata;
- determinism/testing expectations for fixture-stable CLI output.

This document does not define:
- server endpoint semantics (owned by `v3-server-api-protocol-spec.md`);
- internal runtime behavior contracts (owned by runtime/tick/reference specs);
- analysis/reporting algorithms beyond the recipe config contract.

---

## 2. CLI Mode (v3alpha1)

Canonical mode: local core runner.

Rules:
- CLI executes simulation directly through `v3-core` (no required server
  dependency for canonical run path).
- CLI startup and seeding behavior must align with
  `v3-startup-seeding-spec.md`.
- CLI config semantics must align with `v3-runtime-config-spec.md` and
  `v3-world-grid-spec.md`.
- Server HTTP/WS protocol version bumps do not automatically change the CLI
  NDJSON contract; CLI versioning changes only when this file's event/output
  contract changes.

Remote server-client mode is out of scope for this spec version.

---

## 3. Command Contract (Minimal)

Canonical command:

```text
v3-cli run --ticks <u64> --sample-every <u16> --seed <u64> [--config <path>] [--save-config <path>]
```

Rules:
- `--ticks` is required and must be `>= 1`.
- `--sample-every` defaults to `1` when omitted.
- `--sample-every` must be `>= 1`.
- `--seed` is required for deterministic replay and fixture reproducibility.
- `--config` is optional; the config file must be JSON with the same schema as
  the server startup request body (Section 4.1 of
  `v3-server-api-protocol-spec.md`), excluding the `seed` field (seed is
  provided via `--seed`). Malformed or missing config files are fatal errors.
- Recipes are partial or complete `SimulationConfig` JSON objects. Recursively
  merge objects over current defaults; arrays and all other values replace,
  including null. `{}` is valid. Unknown fields and invalid shapes are rejected.
  Startup ramp validation precedes normalization and startup overrides, using
  the same core resolver as server startup.
- `--save-config` writes the full effective config as readable JSON before the
  run starts, with or without `--config`. It excludes the run seed and NDJSON.
  Read, parse and write failures stop the run with a nonzero exit.
- Save/load regenerates the same tick-zero world only with the same run seed
  and locked code/dependencies/platform. A fixed world seed fixes terrain and
  fertility; food, founders and runtime still use `--seed`. Manual paint and
  evolved state are not part of a recipe.

Example (use an existing recipe with small dimensions):

```sh
v3-cli run --config world.json --save-config applied.json --seed 42 --ticks 1
v3-cli run --config applied.json --seed 42 --ticks 1
```

`bench --profile sweep --config world.json --seeds 11,22,33 --ticks 2000
--out sweep.json` uses the same resolver. Omitted width, height and founders
come from the recipe; explicit flags and `--food-coverage` override their
respective fields, followed by normalization/startup overrides. Without a
recipe, existing required sweep arguments and fixed-profile behavior remain.
`--config` is rejected for gate and goal. The full config seeds every run.
The report profile records `recipe_path` and effective `config_digest`;
requested founder count and dimensions match the normalized config. Actual
placement retains the passable-cell clamp. Omitted coverage is labeled
`recipe`, while explicit coverage reports the normalized applied value.
Different recipe digests cannot compare as the same profile. Recipe-free and
historical reports omit these metadata fields and keep existing comparisons.

From T12.F04, the standard `bench --profile goal` selects three built-in
recipe cases: Orchards in grassland/11, Canyon country/22 and Confluence/33.
Each executes once at 2,000 ticks with production creature policies and full
goal observations, using that case's effective config and food-type count.
The profile identifies case name, recipe path, effective digest and run seed;
case-specific neighborhood/drift observations retain that attribution. Each
case also reports `reachable_structure_size_distribution` over its complete
final population; historical missing/null values are unavailable, not zero or a
distribution inferred from the evolved sample. The top-level distribution is
explicitly pooled across all final populations.
From T12.F04 every profile's persistence samples, taken on the same cadence
as `births_total`, also carry the run's cumulative applied behavior:
`typed_eats_total` per food type (applied Eat actions that consumed food),
`food_density_total` per food type (standing density from that tick's applied
growth summary), `moves_attempted_total`, `moves_blocked_barrier_total`, and
`moves_blocked_avoidable_by_reader_state`. Each world-set case observation
carries the same end-of-run totals plus the fractions they imply: each type's
share of every applied eat, barrier-blocked moves against attempted moves, and
avoidable blocked moves against attempted moves by barrier-reader state. All of
these fields are serde-defaulted; a report stored before T12.F04 is unmeasured,
not zero.

For `goal-worlds-v1`, a reference whose per-case block differs only in
`config_digest` is comparable: profile identity for that series excludes the
case list, so a recipe edit required by the standard-baseline contract never
discards a completed run. Each reference comparison then carries one entry per
current case with both digests, `inputs_changed`, `absent_in_reference` when
the reference never ran that case, and current/reference/percent-delta readings
for persistence, per-case work counters, typed eat share, blocked moves,
lineage, memory, drift, neighborhood, and structure. An extinction tick is
reported as values with a null delta. Per-case entries carry no severity level;
the profile-total work counters keep the existing regression rule. Every other
profile difference, and any difference outside the case list, remains a hard
error.
There is no mandatory fourth Plains run or three-seed-per-recipe sweep set.
The new `goal-worlds-v1` series preserves `goal-v1` history and does not
compare across profile definitions. The short gate profile, its epoch and
numerical thresholds remain unchanged. Existing sample sizes, mutation floors
and observation budgets apply to the complete goal profile, not multiplied
per case. Public `--config` remains sweep-only; built-in goal recipes do not
permit arbitrary profile overrides.

Saved-world inspection (T12.F04):

```text
v3-cli world inspect --config <recipe path> --seed <u64> [--png <path>]
```

Rules:
- The recipe resolves over the production defaults through the same resolver
  `run` and the goal cases use: deep merge, ramp validation, normalization,
  then startup overrides. The goal profile additionally forces its own world
  size and founder count; the checked-in baseline recipes set neither, so the
  inspected map is the map that profile runs.
- The world is seeded and never ticked. Every reading is tick zero.
- Standard output is exactly one JSON object: recipe path, run seed, world
  width/height, edge mode, the effective `world_seed` with whether the recipe
  pinned it, `passable_connectivity`, founders actually placed, and per food
  type its index, name, effective energy per unit, fertile cells (effective
  tick-zero fertility above zero on a passable cell), that count as a fraction
  of every cell, mean fertility over those cells alone, cells holding food, and
  standing energy (seeded density times effective energy per unit).
- `--png` writes a two-panel preview: habitat on the left (barriers gray, the
  first two food types' effective tick-zero fertility blended as intensity) and
  tick-zero food density on the right, in the same colors. Both panels are
  downsampled by one integer factor chosen so each panel's longer side is at
  most 512 px, max-pooling barriers and mean-pooling every other layer.
- A missing or unreadable recipe, or one the resolver rejects, is a validation
  error that exits `1` and writes neither the readings nor the preview.

Exit codes:
- `0`: successful completion (run finished normally).
- `1`: validation error (malformed config, constraint violations, and other
  argument checks the command makes itself).
- `2`: runtime error (unexpected failure during simulation execution), and the
  clap usage error for an argument that fails to parse at all.

---

## 4. NDJSON Event Envelope

Each output line is a JSON object with required keys:
- `protocol_version`
- `event_type`

Canonical version:
- `protocol_version = "v3alpha1"`

Top-level unknown keys are not allowed in fixture-validated output for this
minimal contract.

---

## 5. Required Event Schemas

### 5.1 `run_started`

```json
{
  "protocol_version": "v3alpha1",
  "event_type": "run_started",
  "seed": 42,
  "ticks_requested": 1000,
  "sample_every": 25,
  "config_digest": "sha256:<lowercase hex>"
}
```

`config_digest` hashes the effective config actually seeded, using the server's
compact recursively key-sorted JSON SHA-256 encoding, not the input file bytes.

### 5.2 `tick_sample`

```json
{
  "protocol_version": "v3alpha1",
  "event_type": "tick_sample",
  "tick": 250,
  "population": 48,
  "mean_energy": 37.4,
  "reproduction_actions_attempted_total": 721,
  "reproduction_actions_spawned_total": 129,
  "reproduction_actions_rejected_total": 592,
  "mutation_events_attempted_total": 509,
  "mutation_events_applied_total": 321,
  "mutation_events_skipped_total": 188,
  "mutation_events_attempted_total_by_domain": {
    "Topology": 164,
    "Vm": 129,
    "Graph": 116,
    "InputRef": 100
  },
  "mutation_events_applied_total_by_domain": {
    "Topology": 102,
    "Vm": 83,
    "Graph": 74,
    "InputRef": 62
  },
  "mutation_events_attempted_total_by_operator": {
    "Topology.AddNode": 21,
    "Vm.VmInstructionMutation": 40
  },
  "mutation_events_applied_total_by_operator": {
    "Topology.AddNode": 13,
    "Vm.VmInstructionMutation": 25
  },
  "mutation_events_applied_total_semantic_noop": 37,
  "mutation_events_applied_total_semantic_change": 284
}
```

Mutation map-key rules:
- Domain map keys use stable domain strings (`Topology`, `Vm`, `Graph`,
  `InputRef`).
- Operator map keys use stable `Domain.Operator` strings (for example
  `Topology.AddNode`, `Vm.VmInstructionMutation`).

### 5.3 `run_completed`

```json
{
  "protocol_version": "v3alpha1",
  "event_type": "run_completed",
  "ticks_executed": 1000,
  "final_population": 46,
  "final_mean_energy": 36.1
}
```

Event semantics:
- `run_started` appears exactly once at run start.
- `tick_sample` is emitted at ticks `sample_every`, `2 * sample_every`, ...,
  plus the final tick if it is not already aligned to a sample boundary.
  Tick numbering starts at `1` (the first completed tick).
- `run_completed` appears exactly once at run end.

---

## 6. Determinism and Test Expectations

For fixed seed + config + tick budget in deterministic test mode:
- CLI NDJSON output must be reproducible for assertions.
- Event ordering is fixed: `run_started -> tick_sample* -> run_completed`.
- Numeric field semantics must map one-to-one with canonical runtime/tick/
  observability contracts.

Production-level determinism remains non-required by product policy; this
constraint is for harnesses and regression confidence.

---

## 7. Cross-Spec Ownership Map

- Startup policy and founder baseline: `v3-startup-seeding-spec.md`
- Runtime/energy/world config key defaults: `v3-runtime-config-spec.md` and
  `v3-world-grid-spec.md`
- Tick ordering and action semantics: `v3-tick-orchestration-spec.md`
- Required observability counters/reasons: `v3-evolution-observability-spec.md`
- Server transport APIs: `v3-server-api-protocol-spec.md`

This file remains canonical for minimal v3alpha1 CLI event/output contract.

---

## 8. Policy References

- Determinism scope is canonical in root `AGENTS.md`.

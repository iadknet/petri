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
- determinism/testing expectations for fixture-stable CLI output.

This document does not define:
- server endpoint semantics (owned by `v3-server-api-protocol-spec.md`);
- internal runtime behavior contracts (owned by runtime/tick/reference specs);
- rich analysis/export/reporting modes beyond the minimal run surface.

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
v3-cli run --ticks <u64> --sample-every <u16> --seed <u64> [--config <path>]
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
- Unknown/invalid fields in the config file are rejected.

Exit codes:
- `0`: successful completion (run finished normally).
- `1`: validation error (invalid arguments, malformed config, constraint
  violations).
- `2`: runtime error (unexpected failure during simulation execution).

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
  "sample_every": 25
}
```

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

- Determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).

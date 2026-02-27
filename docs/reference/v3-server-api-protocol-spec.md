# V3 Server API and Protocol Spec

Reference specification for canonical v3alpha1 server HTTP/WebSocket contracts,
lifecycle transitions, and error semantics.

Status: Active

Related references:
- `v3-startup-seeding-spec.md`
- `v3-world-grid-spec.md`
- `v3-runtime-config-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-cli-contract-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document defines:
- v3alpha1 protocol-version posture;
- canonical HTTP endpoint contracts for simulation lifecycle, status/frame, and
  config read/patch surfaces;
- canonical WebSocket event envelope and payload mapping;
- lifecycle transition rules and state restrictions;
- canonical non-2xx error envelope and error-code mapping.

This document does not define:
- internal simulation execution logic;
- startup seeding policy details (owned by `v3-startup-seeding-spec.md`);
- world/runtime default tables (owned by `v3-world-grid-spec.md` and
  `v3-runtime-config-spec.md`).

---

## 2. Protocol Versioning

Version contract:
- `protocol_version` is required in all top-level HTTP responses and WebSocket
  event envelopes.
- Canonical value: `"v3alpha1"`.
- Breaking payload changes require protocol-version bump.
- Documented v3alpha1 exception: food-density payloads and food config shape
  are migrated in-place to normalized `f32` semantics (`density: f32`,
  world food recovery/spread/max fields) without a version bump, to keep core,
  server, and frontend aligned in one compatibility window.

---

## 3. Canonical State Model and Lifecycle Rules

Canonical server state enum:
- `idle`
- `running`
- `paused`

Lifecycle transitions:

| Endpoint | Allowed current state(s) | Result state | Notes |
| --- | --- | --- | --- |
| `POST /v3/simulation/startup` | `idle`, `running`, `paused` | `idle` | Re-initializes simulation to tick `0` on success. |
| `POST /v3/simulation/start` | `idle`, `paused`, `running` | `running` | Idempotent when already `running`. |
| `POST /v3/simulation/pause` | `running`, `paused` | `paused` | Idempotent when already `paused`. |
| `POST /v3/simulation/step` | `paused` only | `paused` | State-restricted endpoint. |

State restriction errors use `409 invalid_state_transition`.

---

## 4. HTTP API Contract

Base path: `/v3`.

### 4.1 `POST /v3/simulation/startup`

Request (conceptual v3alpha1 shape):

```json
{
  "seed": 42,
  "population": {
    "initial_creatures": 2000,
    "max_creatures": 100000
  },
  "world": {
    "width": 400,
    "height": 400,
    "edge_mode": "wrap",
    "food": {
      "growth_rate": 0.096,
      "initial_density": 1.0,
      "initial_coverage": 0.15,
      "spread_threshold_ratio": 0.8,
      "recovery_spawn_rate": 0.01,
      "recovery_floor_ratio": 0.01,
      "max_density": 1.0
    }
  },
  "energy": {
    "lifecycle": {
      "initial_energy": 20.0,
      "max_energy": 200.0,
      "energy_decay_per_tick": 0.5,
      "min_reproduce_energy": 1.0,
      "default_offspring_energy": 8.0
    },
    "costs": {
      "move_cost": 1.0,
      "eat_cost": 0.0,
      "noop_cost": 0.05,
      "reproduce_cost": 0.1,
      "eat_reward_per_food": 12.0
    }
  },
  "runtime": {
    "max_mesh_hops": 1024,
    "max_vm_steps": 10000,
    "max_graph_relax_iters": 15,
    "graph_convergence_epsilon": 0.001,
    "graph_convergence_stable_passes": 2,
    "graph_node_base_cost": 0.00001,
    "vm": {
      "opcode_cost_multiplier": 0.000001
    }
  },
  "mutation": {
    "mutation_probability": 0.303,
    "per_birth_mutation_events_min": 1,
    "per_birth_mutation_events_max": 10,
    "phenotype": {
      "channel_step": 1,
      "channel_change_chance": 0.001,
      "polarity_flip_chance": 0.0002
    }
  }
}
```

Request rules:
- `seed` is required.
- Other domains are optional structured overrides.
- Unspecified fields fall back to canonical defaults from world/runtime config
  specs.
- Startup/config keyspace is canonical from `v3-world-grid-spec.md` and
  `v3-runtime-config-spec.md`.
- Mutation tuning lives at top-level `mutation.*` in the startup/config keyspace
  (not under `runtime.*`).
- No `founder_profile` request field is supported in v3alpha1.
- Unknown request fields are rejected.
- Invalid/non-viable startup requests are rejected with
  `422 validation_rejected`; no auto-normalization behavior is canonical.

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "idle",
  "tick": 0,
  "config_digest": "sha256:<hex>",
  "seeded_creatures": 2000
}
```

`config_digest` computation:
- Serialize the effective config (after applying defaults and accepted
  overrides) as canonical JSON with keys sorted alphabetically at every nesting
  level, no whitespace.
- Compute SHA-256 of the resulting UTF-8 byte string.
- Format as `"sha256:<lowercase-hex>"`.

### 4.2 `POST /v3/simulation/start`

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "running",
  "tick": 123
}
```

Rules:
- Idempotent when already `running`.

### 4.3 `POST /v3/simulation/pause`

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "paused",
  "tick": 123
}
```

Rules:
- Idempotent when already `paused`.
- `idle` -> `pause` is invalid-state transition.

### 4.4 `POST /v3/simulation/step`

Request:

```json
{
  "steps": 1
}
```

Rules:
- Omitted `steps` defaults to `1`.
- Valid range: `1..=1000`.
- Endpoint allowed only in `paused` state.

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "paused",
  "tick": 124,
  "steps_applied": 1
}
```

### 4.5 `GET /v3/simulation/status`

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "running",
  "tick": 124,
  "population": 48,
  "mean_energy": 37.4,
  "last_tick_actions": {
    "move": 31,
    "eat": 12,
    "reproduce": 3,
    "noop": 2
  },
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
  "mutation_events_applied_total_semantic_change": 284,
  "last_tick_compute_total_mean": 18.7,
  "last_tick_compute_total_min": 0.0,
  "last_tick_compute_total_max": 52.0,
  "last_tick_compute_vm_mean": 8.1,
  "last_tick_compute_graph_mean": 10.6
}
```

### 4.6 `GET /v3/simulation/frame`

Response (full sparse frame):

```json
{
  "protocol_version": "v3alpha1",
  "state": "running",
  "tick": 124,
  "width": 400,
  "height": 400,
  "creatures": [
    {
      "id": 7,
      "x": 10,
      "y": 22,
      "energy": 41.0,
      "generation": 3,
      "phenotype_rgb": [204, 61, 61]
    }
  ],
  "food": [
    { "x": 5, "y": 9, "density": 0.42 }
  ],
  "barriers": [
    { "x": 12, "y": 8 }
  ]
}
```

Sparse-frame rules:
- `creatures` includes all live creatures.
- `food` includes only cells with non-zero food density.
- `barriers` includes only blocked cells.

### 4.7 `GET /v3/simulation/config`

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "paused",
  "config": {
    "population": {
      "initial_creatures": 2000,
      "max_creatures": 100000
    },
    "world": {
      "width": 400,
      "height": 400,
      "edge_mode": "wrap",
      "food": {
        "growth_rate": 0.096,
        "initial_density": 1.0,
        "initial_coverage": 0.15,
        "spread_threshold_ratio": 0.8,
        "recovery_spawn_rate": 0.01,
        "recovery_floor_ratio": 0.01,
        "max_density": 1.0
      }
    },
    "energy": {
      "lifecycle": {
        "initial_energy": 20.0,
        "max_energy": 200.0,
        "energy_decay_per_tick": 0.5,
        "min_reproduce_energy": 1.0,
        "default_offspring_energy": 8.0
      },
      "costs": {
        "move_cost": 1.0,
        "eat_cost": 0.0,
        "noop_cost": 0.05,
        "reproduce_cost": 0.1,
        "eat_reward_per_food": 12.0
      }
    },
    "runtime": {
      "max_mesh_hops": 1024,
      "max_vm_steps": 10000,
      "max_graph_relax_iters": 15,
      "graph_convergence_epsilon": 0.001,
      "graph_convergence_stable_passes": 2,
      "graph_node_base_cost": 0.00001,
      "vm": {
        "opcode_cost_multiplier": 0.000001
      }
    },
    "mutation": {
      "mutation_probability": 0.303,
      "per_birth_mutation_events_min": 1,
      "per_birth_mutation_events_max": 10,
      "phenotype": {
        "channel_step": 1,
        "channel_change_chance": 0.001,
        "polarity_flip_chance": 0.0002
      }
    }
  }
}
```

### 4.8 `PATCH /v3/simulation/config`

Request shape:
- Partial object with same structural domains as `GET /config` `config` payload.

Rules:
- Unknown fields rejected.
- PATCH supports the full canonical keyspace from `GET /config`, including all
  top-level `mutation.*` keys owned by `v3-runtime-config-spec.md`.
- PATCH uses deep merge: only specified keys are updated; unspecified keys
  retain their existing values at every nesting level.
- Invalid values rejected with `422 validation_rejected`; transport does not
  apply fallback/clamp normalization to invalid submitted values.
- World topology fields (`world.width`, `world.height`, `world.edge_mode`) are
  editable only in `idle`.
- Runtime and energy fields are editable in `idle` and `paused`.
- Editing configuration in disallowed states returns
  `409 invalid_state_transition`.

Response:

```json
{
  "protocol_version": "v3alpha1",
  "state": "paused",
  "config": { "...": "effective config" }
}
```

---

## 5. WebSocket Stream Contract (`/v3/ws`)

Event envelope:

```json
{
  "protocol_version": "v3alpha1",
  "event": "status",
  "tick": 124,
  "payload": {}
}
```

Allowed events:
- `status`
- `frame`
- `health`

Payload mapping:
- `status` payload is the compact status payload (`state`, `population`,
  `mean_energy`, `last_tick_actions`, reproduction totals, and last-tick
  compute summaries).
- `frame` payload is the compact frame payload (`width`, `height`, `creatures`,
  `food`, `barriers`).
- `health` payload:

```json
{
  "population": 48,
  "mean_energy": 37.4,
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
  "mutation_events_applied_total_semantic_change": 284,
  "reproduction_actions_attempted_total": 721,
  "reproduction_actions_spawned_total": 129,
  "reproduction_actions_rejected_total": 592,
  "reproduction_actions_rejected_total_by_reason": {
    "RejectedInvalidTarget": 512,
    "RejectedEnergyConstraints": 80
  },
  "mutation_events_skipped_total_by_reason": {
    "ParseabilityViolation": 58,
    "NoApplicableTarget": 130
  },
  "genome_complexity_mean": 21.4,
  "genome_complexity_min": 3,
  "genome_complexity_max": 91
}
```

Mutation map-key rules:
- Domain map keys use stable domain strings (`Topology`, `Vm`, `Graph`,
  `InputRef`).
- Operator map keys use stable `Domain.Operator` strings (for example
  `Topology.AddNode`, `Vm.VmInstructionMutation`).

Envelope/payload consistency rules:
- Envelope `tick` is canonical for each event.
- `status` and `frame` payloads must not duplicate envelope-owned keys
  (`protocol_version`, `tick`).
- Any duplicated envelope-owned keys inside payload are protocol-invalid.

Emission frequency:
- Events are emitted once per completed tick while the simulation is `running`.
- Implementations may throttle emission (for example emit every Nth tick) for
  performance, but must document the throttle policy. The default v3alpha1
  behavior is per-tick emission.

Ordering rules:
1. `tick` is monotonic non-decreasing per connection.
2. For the same tick, emission order is `status -> frame -> health` when all are
   emitted.
3. Event/payload mismatches are protocol-invalid.

---

## 6. Error Envelope (Normative)

All non-2xx HTTP responses use:

```json
{
  "protocol_version": "v3alpha1",
  "error": {
    "code": "validation_rejected",
    "message": "startup request failed validation",
    "details": {
      "endpoint": "/v3/simulation/startup",
      "field_errors": [
        { "field": "population.initial_creatures", "reason": "must be >= 1" }
      ],
      "expected_state": "paused",
      "current_state": "running"
    }
  }
}
```

Required error codes:
- `invalid_request` (`400`)
- `invalid_state_transition` (`409`)
- `validation_rejected` (`422`)
- `internal_error` (`500`)

`details` fields are optional and context-specific.

---

## 7. Out-of-Scope API Surfaces (v3alpha1)

Not part of this version:
- `GET /v3/simulation/snapshot`
- `POST /v3/simulation/snapshot`

Snapshot import/export contracts require a follow-up spec.

---

## 8. Cross-Spec Ownership Map

- Startup seeding and founder baseline policy:
  `v3-startup-seeding-spec.md`
- World defaults and spatial validity behavior: `v3-world-grid-spec.md`
- Runtime/energy/mutation key defaults: `v3-runtime-config-spec.md`
- Phenotype state model and mutation algorithm: `v3-phenotype-spec.md`
- Tick action ordering/arbitration: `v3-tick-orchestration-spec.md`
- Required observability semantics: `v3-evolution-observability-spec.md`
- Local runner NDJSON output contract: `v3-cli-contract-spec.md`

This file remains canonical for v3alpha1 server transport/API semantics.

---

## 9. Policy References

- Runtime truthfulness invariant is canonical in
  `docs/strategy/architecture.md` (`Runtime Truthfulness Invariant`).
- Determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).

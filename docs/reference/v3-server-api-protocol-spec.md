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
    "initial_creatures": 50,
    "max_creatures": 1000
  },
  "world": {
    "width": 400,
    "height": 400,
    "edge_mode": "wrap",
    "food": {
      "growth_rate": 0.02,
      "initial_density": 80,
      "initial_coverage": 0.3
    }
  },
  "energy": {
    "lifecycle": {
      "initial_energy": 20.0,
      "max_energy": 100.0,
      "energy_decay_per_tick": 0.2,
      "min_reproduce_energy": 24.0,
      "default_offspring_energy": 20.0
    },
    "costs": {
      "move_cost": 0.2,
      "eat_cost": 0.0,
      "noop_cost": 0.0,
      "reproduce_cost": 2.0,
      "eat_reward_per_food": 1.0
    }
  },
  "runtime": {
    "max_mesh_hops": 128,
    "max_vm_steps": 1024,
    "max_graph_relax_iters": 4,
    "graph_convergence_epsilon": 0.001,
    "graph_convergence_stable_passes": 1,
    "graph_node_base_cost": 1.0,
    "vm": {
      "opcode_cost_multiplier": 1.0
    },
    "mutation": {
      "mutation_probability": 0.01,
      "per_birth_mutation_events_min": 1,
      "per_birth_mutation_events_max": 4,
      "domain_selection_weights": {
        "Topology": 1.0,
        "Vm": 1.0,
        "Graph": 1.0
      },
      "operator_selection_weights": {
        "Topology": { "AddNode": 1.0, "RemoveNode": 1.0 },
        "Vm": {
          "VmInstructionMutation": 1.0,
          "VmConstantMutation": 1.0
        },
        "Graph": {
          "AddInternalGraphNode": 1.0,
          "RemoveInternalGraphNode": 1.0
        }
      },
      "operator_modifier_scale": 1.0,
      "phenotype": {
        "channel_step": 2,
        "polarity_flip_chance": 0.002,
        "channel_weight_min": 0.05,
        "channel_weight_max": 1.0
      }
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
- `runtime.mutation` accepts the full canonical key set, including
  `domain_selection_weights` and `operator_selection_weights`.
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
  "config_digest": "sha256-hex",
  "seeded_creatures": 50
}
```

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
  "reproduction_actions_rejected_total": 592
}
```

### 4.6 `GET /v3/simulation/frame`

Response (full sparse frame):

```json
{
  "protocol_version": "v3alpha1",
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
    { "x": 5, "y": 9, "density": 17 }
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
      "initial_creatures": 50,
      "max_creatures": 1000
    },
    "world": {
      "width": 400,
      "height": 400,
      "edge_mode": "wrap",
      "food": {
        "growth_rate": 0.02,
        "initial_density": 80,
        "initial_coverage": 0.3
      }
    },
    "energy": {
      "lifecycle": {
        "initial_energy": 20.0,
        "max_energy": 100.0,
        "energy_decay_per_tick": 0.2,
        "min_reproduce_energy": 24.0,
        "default_offspring_energy": 20.0
      },
      "costs": {
        "move_cost": 0.2,
        "eat_cost": 0.0,
        "noop_cost": 0.0,
        "reproduce_cost": 2.0,
        "eat_reward_per_food": 1.0
      }
    },
    "runtime": {
      "max_mesh_hops": 128,
      "max_vm_steps": 1024,
      "max_graph_relax_iters": 4,
      "graph_convergence_epsilon": 0.001,
      "graph_convergence_stable_passes": 1,
      "graph_node_base_cost": 1.0,
      "vm": {
        "opcode_cost_multiplier": 1.0
      },
      "mutation": {
        "mutation_probability": 0.01,
        "per_birth_mutation_events_min": 1,
        "per_birth_mutation_events_max": 4,
        "domain_selection_weights": {
          "Topology": 1.0,
          "Vm": 1.0,
          "Graph": 1.0
        },
        "operator_selection_weights": {
          "Topology": { "AddNode": 1.0, "RemoveNode": 1.0 },
          "Vm": {
            "VmInstructionMutation": 1.0,
            "VmConstantMutation": 1.0
          },
          "Graph": {
            "AddInternalGraphNode": 1.0,
            "RemoveInternalGraphNode": 1.0
          }
        },
        "operator_modifier_scale": 1.0,
        "phenotype": {
          "channel_step": 2,
          "polarity_flip_chance": 0.002,
          "channel_weight_min": 0.05,
          "channel_weight_max": 1.0
        }
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
  runtime mutation keys owned by `v3-runtime-config-spec.md`.
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
- `status` payload uses `GET /v3/simulation/status` schema excluding top-level
  `protocol_version` and `tick` fields.
- `frame` payload uses `GET /v3/simulation/frame` schema excluding top-level
  `protocol_version` and `tick` fields.
- `health` payload:

```json
{
  "population": 48,
  "mean_energy": 37.4,
  "mutation_events_attempted_total": 509,
  "mutation_events_applied_total": 321,
  "mutation_events_skipped_total": 188,
  "reproduction_actions_attempted_total": 721,
  "reproduction_actions_spawned_total": 129,
  "reproduction_actions_rejected_total": 592,
  "reproduction_actions_rejected_total_by_reason": {
    "RejectedInvalidTarget": 512,
    "RejectedEnergyConstraints": 80
  }
}
```

Envelope/payload consistency rules:
- Envelope `tick` is canonical for each event.
- `status` and `frame` payloads must not duplicate envelope-owned keys
  (`protocol_version`, `tick`).
- Any duplicated envelope-owned keys inside payload are protocol-invalid.

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

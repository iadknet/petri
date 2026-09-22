# V3 Server API and Protocol Spec

Reference specification for canonical v3alpha3 server HTTP/WebSocket contracts,
lifecycle transitions, and error semantics.

Status: Active

Related references:
- `v3-sensor-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-world-grid-spec.md`
- `v3-runtime-config-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-server-query-projection-spec.md`
- `v3-cli-contract-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document defines:
- v3alpha3 protocol-version posture;
- canonical HTTP endpoint contracts for simulation lifecycle, status/frame, and
  config read/patch surfaces;
- canonical WebSocket event envelope and payload mapping;
- lifecycle transition rules and state restrictions;
- canonical non-2xx error envelope and error-code mapping.

This document does not define:
- internal simulation execution logic;
- server-local projection publication, freshness, or spatial-index semantics
  (owned by `v3-server-query-projection-spec.md`);
- startup seeding policy details (owned by `v3-startup-seeding-spec.md`);
- world/runtime default tables (owned by `v3-world-grid-spec.md` and
  `v3-runtime-config-spec.md`).

Current viewport-transport addenda:
- overview transport is aggregate-only and intentionally omits predation-event
  payloads;
- paint responses return invalidation metadata, and the frontend follows with a
  viewport-scoped `GET /v3/simulation/snapshot` refresh rather than receiving an
  inline snapshot in the paint response.

---

## 2. Protocol Versioning

Version contract:
- `protocol_version` is required in all top-level HTTP responses and WebSocket
  event envelopes.
- Canonical value: `"v3alpha3"`.
- Breaking payload changes require protocol-version bump. `v3alpha3`
  (T19.F04) removed the action bank, execute gate, and VM metadata trace
  fields and the old termination reasons, and added pass records.
- Server transport versioning is independent from the CLI NDJSON contract; a
  server bump does not automatically change the CLI protocol version.
- Documented v3alpha2 exception: food-density payloads and food config shape
  are migrated in-place to normalized `f32` semantics (`density: f32`,
  `world.food.shared.*`, `world.food.types[]`, and fertility layer targets)
  without a version bump, to keep core, server, and frontend aligned in one
  compatibility window.

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

Request (conceptual v3alpha3 shape):

```json
{
  "seed": 42,
  "population": {
    "initial_creatures": 10000,
    "max_creatures": 100000,
    "founder_profile": "v3_alpha1"
  },
  "world": {
    "width": 1600,
    "height": 1600,
    "edge_mode": "wrap",
    "food": {
      "shared": {
        "growth_rate": 0.09,
        "initial_density": 1.0,
        "initial_coverage": 0.54,
        "spread_threshold_ratio": 0.8,
        "spread_density_ratio": 0.25,
        "recovery_spawn_rate": 0.01,
        "recovery_floor_ratio": 0.01,
        "max_density": 1.0,
        "occupancy_depletion": {
          "enabled": true,
          "deposit_per_occupied_tick": 0.08
        },
        "grazing": {
          "enabled": true,
          "factor": 0.5,
          "floor": 0.05,
          "recovery_ticks": 1000
        }
      },
      "types": [
        {
          "name": "Primary Food",
          "color": "#22c55e",
          "initial_density": 1.0,
          "initial_coverage": 0.54,
          "growth_inhibitor": 0.2
        }
      ],
      "fertility": {
        "enabled": true,
        "min_fertility": 0.0,
        "max_fertility": 2.0,
        "layers": [
          {
            "algorithm": {
              "PoissonBlobs": {
                "blob_count": 900,
                "min_radius": 5.0,
                "max_radius": 15.0,
                "falloff": 0.5,
                "seed": null
              }
            },
            "weight": 1.0,
            "target": "AllFoods"
          }
        ]
      }
    }
  },
  "energy": {
    "lifecycle": {
      "initial_energy": 20.0,
      "max_energy": 200.0,
      "energy_decay_per_tick": 0.5,
      "min_reproduce_energy": 30.0,
      "min_reproduce_age": 20,
      "default_offspring_energy": 100.0
    },
    "costs": {
      "move_cost": 0.2,
      "eat_cost": 0.0,
      "eat_reward_per_food": 5.0,
      "noop_cost": 0.05,
      "reproduce_cost": 0.1,
      "failed_action_penalty": 1.0
    }
  },
  "startup": {
    "ramps": {
      "failed_action_penalty": {
        "enabled": true,
        "start": 5.0,
        "end": 30.0,
        "target_tick": 1000
      }
    }
  },
  "runtime": {
    "max_mesh_hops": 64,
    "max_vm_steps": 10000,
    "graph_node_base_cost": 0.00001,
    "vm": {
      "opcode_cost_multiplier": 0.000001
    }
  },
  "mutation": {
    "per_unit_supply_enabled": true,
    "per_unit_rate": 0.005,
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
- `world.terrain` and `world.world_seed` are accepted through the same partial
  config deep merge. Terrain is an ordered list of existing tagged
  `PatternParams` layers with optional/null bounds and seeds; the map seed is
  optional/null. GET config returns these effective fields and tick-zero
  projections contain their applied barriers before food/founders.
- Unspecified fields fall back to canonical defaults from world/runtime config
  specs.
- Startup/config keyspace is canonical from `v3-world-grid-spec.md` and
  `v3-runtime-config-spec.md`.
- Mutation tuning lives at top-level `mutation.*` in the startup/config keyspace
  (not under `runtime.*`).
- Startup-only controls live under `startup.*` and are restart-only, as are
  `population.initial_creatures` and `population.founder_profile` (Section
  4.8).
- `population.founder_profile` is optional and selects the founder genome
  profile applied uniformly to every seeded founder. Wire values are the
  `snake_case` profile names tabulated in `v3-startup-seeding-spec.md`
  Section 5.1; absent means `v3_alpha1`. An unrecognized name is rejected with
  `422 validation_rejected`. The key is canonical in
  `v3-runtime-config-spec.md` Section 5.
- Unknown request fields are rejected.
- Invalid/non-viable startup requests are rejected with
  `422 validation_rejected`; no auto-normalization behavior is canonical.

Response:

```json
{
  "protocol_version": "v3alpha3",
  "state": "idle",
  "tick": 0,
  "config_digest": "sha256:<hex>",
  "seeded_creatures": 10000
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
  "protocol_version": "v3alpha3",
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
  "protocol_version": "v3alpha3",
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
  "protocol_version": "v3alpha3",
  "state": "paused",
  "tick": 124,
  "steps_applied": 1
}
```

### 4.5 `GET /v3/simulation/status`

Response:

```json
{
  "protocol_version": "v3alpha3",
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
  "last_tick_compute_total_mean": 18.7,
  "last_tick_compute_total_min": 0.0,
  "last_tick_compute_total_max": 52.0,
  "last_tick_compute_vm_mean": 8.1,
  "last_tick_compute_graph_mean": 10.6,
  "last_tick_food_occupancy_depletion_mean": 0.12,
  "last_tick_food_occupancy_depletion_occupied_cells": 3,
  "last_tick_food_growth_suppressed_by_occupancy_depletion": 0.7
}
```

### 4.6 `GET /v3/simulation/frame`

Response (full sparse frame):

```json
{
  "protocol_version": "v3alpha3",
  "state": "running",
  "tick": 124,
  "width": 1600,
  "height": 1600,
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

With `?format=recipe`, returns just the complete current effective config as
JSON, without the protocol/state envelope or run seed. The app downloads the
response text directly so u64 seeds are preserved exactly. The default
response below remains unchanged. Save fetches current applied values rather
than pending form edits.

The app's Load Recipe action sends the original recipe JSON with its current
Run Seed through startup, using the same restart confirmation for running or
paused worlds. Recipes must be objects and exclude top-level `seed`. On success,
the app refreshes applied config/startup controls and tick-zero state without
reapplying the previous runtime config. Failures are visible; refresh failure
after a successful restart is identified separately. Large file seeds retain
u64 precision; ordinary numeric editing/restart requires safe integer seeds.

Response:

```json
{
  "protocol_version": "v3alpha3",
  "state": "paused",
  "config": {
    "population": {
      "initial_creatures": 10000,
      "max_creatures": 100000,
      "founder_profile": "v3_alpha1"
    },
    "world": {
      "width": 1600,
      "height": 1600,
      "edge_mode": "wrap",
      "food": {
        "shared": {
          "growth_rate": 0.09,
          "initial_density": 1.0,
          "initial_coverage": 0.54,
          "spread_threshold_ratio": 0.8,
          "spread_density_ratio": 0.25,
          "recovery_spawn_rate": 0.01,
          "recovery_floor_ratio": 0.01,
          "max_density": 1.0,
          "occupancy_depletion": {
            "enabled": true,
            "deposit_per_occupied_tick": 0.08
          },
          "grazing": {
            "enabled": true,
            "factor": 0.5,
            "floor": 0.05,
            "recovery_ticks": 1000
          }
        },
      "types": [
        {
          "name": "Primary Food",
          "color": "#22c55e",
          "initial_density": 1.0,
          "initial_coverage": 0.54,
          "growth_inhibitor": 0.2
        }
      ],
        "fertility": {
          "enabled": true,
          "min_fertility": 0.0,
          "max_fertility": 2.0,
          "layers": [
            {
              "algorithm": {
                "PoissonBlobs": {
                  "blob_count": 900,
                  "min_radius": 5.0,
                  "max_radius": 15.0,
                  "falloff": 0.5,
                  "seed": null
                }
              },
              "weight": 1.0,
              "target": "AllFoods"
            }
          ]
        }
      }
    },
    "energy": {
      "lifecycle": {
        "initial_energy": 20.0,
        "max_energy": 200.0,
        "energy_decay_per_tick": 0.5,
        "min_reproduce_energy": 30.0,
        "min_reproduce_age": 20,
        "default_offspring_energy": 100.0
      },
      "costs": {
        "move_cost": 0.2,
        "eat_cost": 0.0,
        "eat_reward_per_food": 5.0,
        "noop_cost": 0.05,
        "reproduce_cost": 0.1,
        "failed_action_penalty": 1.0
      }
    },
    "runtime": {
      "max_mesh_hops": 64,
      "max_vm_steps": 10000,
      "graph_node_base_cost": 0.00001,
      "vm": {
        "opcode_cost_multiplier": 0.000001
      }
    },
    "mutation": {
      "per_unit_supply_enabled": true,
      "per_unit_rate": 0.005,
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
- `energy.costs.eat_reward_per_food` is live-editable and applies to types inheriting it (`energy_per_unit` absent/null). Food type `energy_per_unit`, `growth_rate`, `recovery_spawn_rate` overrides and `initial_fertility_only` are startup/restart-only fields; optional/null rates inherit their shared live values and explicit zero overrides.
- PATCH supports the full canonical keyspace from `GET /config`, including all
  top-level `mutation.*` keys owned by `v3-runtime-config-spec.md`, plus the
  runtime-editable `world.food.shared.occupancy_depletion.*` and
  `world.food.shared.grazing.*` keys cross-referenced by
  `v3-runtime-config-spec.md` and semantically owned by
  `v3-world-grid-spec.md`.
- PATCH uses deep merge: only specified keys are updated; unspecified keys
  retain their existing values at every nesting level.
- Invalid values rejected with `422 validation_rejected`; transport does not
  apply fallback/clamp normalization to invalid submitted values.
- A rejected patch applies nothing and names every offending path in
  `error.details.field_errors`.
- World topology fields (`world.width`, `world.height`, `world.edge_mode`,
  `world.terrain`, `world.world_seed`) are restart-only and rejected from PATCH
  by key presence, including null/empty values, atomically without state changes.
- Runtime and energy fields are editable in `idle` and `paused`.
- `startup.*` fields are restart-only and rejected from PATCH.
- `population.initial_creatures` is restart-only and rejected from PATCH:
  founders are seeded only by `POST /v3/simulation/startup`, so a patched value
  would never take effect. `population.founder_profile` is restart-only and
  rejected from PATCH for the same reason: it is consumed once at startup
  seeding and never re-read afterward. `population.max_creatures` is a live
  reproduction cap and stays editable.
- `world.food.types[]` and `world.food.fertility.layers` are restart-only and
  rejected from PATCH.
- `energy.costs.failed_action_penalty` is rejected while an active startup
  failed-action-penalty ramp is still in progress (`tick < target_tick`).
- Editing configuration in disallowed states returns
  `409 invalid_state_transition`.

Response:

```json
{
  "protocol_version": "v3alpha3",
  "state": "paused",
  "config": { "...": "effective config" }
}
```

### 4.9 `POST /v3/simulation/creature/:id/sample`

Request:

```json
{
  "ticks": 5,
  "include_perception_debug": false
}
```

Rules:
- endpoint is allowed only when simulation state is `running` or `paused`
- `ticks` defaults to `5`
- valid range for `ticks` is `1..=10`
- `include_perception_debug` is optional and defaults to `false`
- starting a new sample replaces any existing active sample
- when `include_perception_debug = true`, completed tick samples may include the
  optional `debug_perception` payload defined by the execution-sampler contract
  for each tick

Response:

```json
{
  "protocol_version": "v3alpha3",
  "status": "recording",
  "ticks_requested": 5,
  "include_perception_debug": false
}
```

### 4.10 `GET /v3/simulation/creature/:id/sample`

Recording response:

```json
{
  "protocol_version": "v3alpha3",
  "status": "recording",
  "ticks_completed": 2,
  "ticks_remaining": 3
}
```

Complete response:

```json
{
  "protocol_version": "v3alpha3",
  "status": "complete",
  "sample": {
    "creature_id": 7,
    "ticks": [
      {
        "tick_number": 124,
        "hops": [
          {
            "hop_index": 0,
            "route": {
              "kind": "vm_wrap",
              "raw_value": 2.5,
              "resolved_target_index": 2
            },
            "backend_trace": {
              "Vm": {
                "final_route_value": 2.5
              }
            }
          }
        ],
        "static_inputs": { "...": "existing local snapshot" },
        "debug_perception": null
      }
    ]
  }
}
```

Execution-sampler rules:
- when the request did not ask for perception debugging,
  `debug_perception` is `null`
- when the request asked for perception debugging, `debug_perception` contains
  the extended-perception snapshot for that tick
- sampler route payload is `route { kind, raw_value, resolved_target_index }`
- VM trace route scalar field is `final_route_value`
- Graph trace payload includes effect-phase structural records:
  - `output_sinks[]` entries with `wired`, `weighted_sum`, `applied`,
    `applied_value`
- each tick carries `passes[]`, one record per pass (T19.F04): `pass_index`,
  `end_reason` (`Decided`, `PassCapReached`, `NoTargets`, `MissingNode`,
  `EnergyExhausted`), `votes` (27 floats in `VoteSink` catalog order, the
  pass's final vote vector), `effective_votes` (four floats, kind order
  `Eat`, `Move`, `Reproduce`, `StealEnergy`, against the bars the pass
  started with), `committed` (the action the pass committed, or `null`), and
  `hops`
- each tick carries `termination_reason` (`NoDecision`, `TerminateVoted`,
  `ActionCapReached`, `EnergyExhausted`) and `commit_counts`, the final
  per-kind bars (commits of each kind this tick)
- each hop carries `pass_index` beside the tick-wide `hop_index`, and
  `vote_contribution`, the 27 floats that hop committed; zeros when the
  dispatch ended exhausted and committed nothing
- the VM trace has no metadata buffer (`final_meta` was removed)
- sampler wire DTO ownership and mapping live in `v3-server/src/transport/`
  (`sample_protocol.rs`, `sample_assembler.rs`)
- detailed perception field ownership remains in `v3-sensor-spec.md`

### 4.11 `GET /v3/simulation/creature/:id`

Path parameters:
- `:id` — creature FFI ID (u64).

Query parameters (all optional):
- `since_tick` (u64) — when present, only action_log entries with
  `entry.tick > since_tick` are returned. Omit for the full log.
- `exclude` (string) — comma-separated field names to omit from the response.
  Valid values: `genome`, `action_log`, `shared_memory`. Excluded fields are absent
  from the JSON (not set to null). Unknown names are silently ignored.

Response (full, no query parameters):

```json
{
  "protocol_version": "v3alpha3",
  "id": 7,
  "position": { "x": 10, "y": 22 },
  "energy": 41.0,
  "max_energy": 20.0,
  "age": 84,
  "generation": 3,
  "complexity": 12,
  "phenotype": {
    "channels": [128, 0, 64, 0, 0, 0],
    "active_channel": 0,
    "polarity": [true, false, true, false, false, false],
    "rgb": [128, 0, 64]
  },
  "latest_tick": 84,
  "genome": { "entry_node_id": 0, "nodes": ["..."] },
  "shared_memory": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
  "action_log": [
    {
      "tick": 84,
      "action_type": "Eat",
      "result": "Success",
      "direction": 255,
      "energy_before": 35.5,
      "energy_after": 41.0,
      "amount": 6.0,
      "food_type": 0,
      "priority_bid": 0.5
    }
  ]
}
```

Field definitions:

| Field | Type | Description |
|-------|------|-------------|
| `protocol_version` | string | Always `"v3alpha3"`. |
| `id` | u64 | Creature FFI ID. |
| `position` | `{x, y}` | Current grid position. |
| `energy` | f32 | Current energy level. |
| `max_energy` | f32 | Maximum energy (from config). |
| `age` | u64 | Ticks alive. |
| `generation` | u64 | Reproduction generation (0 = founder). |
| `complexity` | u32 | Genome complexity (node count). |
| `phenotype` | object | Visual phenotype state. |
| `latest_tick` | u64 | Tick of the most recent action_log entry (0 if empty). Always present, even when `action_log` is excluded. Used as cursor for `since_tick`. |
| `genome` | object | Full genome (omitted when `exclude` contains `genome`). |
| `shared_memory` | `f32[]` | 16-slot creature shared memory (omitted when `exclude` contains `shared_memory`). |
| `action_log` | array | Action log entries (omitted when `exclude` contains `action_log`). |

`ActionLogEntry` field definitions:

| Field | Type | Description |
|-------|------|-------------|
| `tick` | u64 | Simulation tick when the action was executed. |
| `action_type` | string | One of `NoOp`, `Eat`, `Move`, `Reproduce`, `StealEnergy`. |
| `result` | string | One of `Success`, `NoFood`, `Blocked`, `InvalidTarget`, `AgeConstraints`, `EnergyConstraints`, `PopulationCap`, `TransferredAndKilled`, `NoVictim`. |
| `direction` | u8 | Direction parameter (0-7 cardinal+diagonal, 255 = N/A). |
| `energy_before` | f32 | Creature energy before the action. |
| `energy_after` | f32 | Creature energy after the action (includes costs). |
| `amount` | f32 | Action-specific amount (food consumed, energy stolen, transfer fraction in [0, 1] for Reproduce, 0 otherwise). |
| `food_type` | u16 or null | Selected ordinary-food type for Eat; `null` for non-Eat actions. |
| `priority_bid` | f32 | Priority bid value for that tick. |

`phenotype` sub-object:

| Field | Type | Description |
|-------|------|-------------|
| `channels` | `u8[6]` | Per-channel phenotype values (2 per RGB component). |
| `active_channel` | usize | Index of the currently mutating channel. |
| `polarity` | `bool[6]` | Per-channel drift direction (true = incrementing). |
| `rgb` | `u8[3]` | Derived RGB color. |

Incremental polling pattern:
1. First request: `GET /v3/simulation/creature/:id` (no query params) — full response.
2. Subsequent requests: `GET /v3/simulation/creature/:id?since_tick={latest_tick}&exclude=genome` — returns only new action_log entries and omits immutable genome.
3. Client appends returned action_log entries to its local cache, trimming to ring buffer capacity.

Error cases:
- `404 not_found` — creature ID does not exist.

---

## 5. WebSocket Stream Contract (`/v3/ws`)

Event envelope:

```json
{
  "protocol_version": "v3alpha3",
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
  `mean_energy`, `last_tick_actions`, reproduction totals, last-tick compute
  summaries, and last-tick food occupancy depletion summaries).
- `frame` payload is the compact frame payload (`width`, `height`, `creatures`,
  `food`, `barriers`).
- `health` payload mirrors the same last-tick food occupancy depletion
  summaries alongside the cumulative health counters:

```json
{
  "population": 48,
  "mean_energy": 37.4,
  "last_tick_food_occupancy_depletion_mean": 0.12,
  "last_tick_food_occupancy_depletion_occupied_cells": 3,
  "last_tick_food_growth_suppressed_by_occupancy_depletion": 0.7,
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
  "reproduction_actions_attempted_total": 721,
  "reproduction_actions_spawned_total": 129,
  "reproduction_actions_rejected_total": 592,
  "reproduction_actions_rejected_total_by_reason": {
    "RejectedInvalidTarget": 512,
    "RejectedAgeConstraints": 30,
    "RejectedEnergyConstraints": 50
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
  performance, but must document the throttle policy. The default v3alpha3
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
  "protocol_version": "v3alpha3",
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

## 7. Out-of-Scope API Surfaces (v3alpha3)

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
- Extended-perception field layout and formulas: `v3-sensor-spec.md`
- Phenotype state model and mutation algorithm: `v3-phenotype-spec.md`
- Tick action ordering/arbitration: `v3-tick-orchestration-spec.md`
- Required observability semantics: `v3-evolution-observability-spec.md`
- Local runner NDJSON output contract: `v3-cli-contract-spec.md`

This file remains canonical for v3alpha3 server transport/API semantics.

---

## 9. Policy References

- Runtime truthfulness invariant is canonical in
  `docs/strategy/architecture.md` (`Runtime Truthfulness Invariant`).
- Determinism scope is canonical in root `AGENTS.md`.

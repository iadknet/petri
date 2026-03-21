# OpenTelemetry Observability Design

## Overview

Adopt OpenTelemetry as the observability foundation for the Petri simulation, backed by a self-hosted SigNoz stack. Provides structured tracing of tick phases and creature cognition at configurable depth, OTel metrics for population-level aggregates, and SQL-queryable storage for agent-driven analysis.

## Goals

- Full wall-clock timing decomposition of every tick phase and sub-phase
- High-fidelity creature cognition tracing with per-node signal flow, energy accounting, and learning weight updates
- Population-level metrics dashboards derived from existing `SimStats`
- Agent-queryable storage (ClickHouse SQL over HTTP) for Claude/Codex troubleshooting and insight generation
- Runtime-adjustable tracing controls via config panel
- Zero impact on `v3-core` — all instrumentation lives in `v3-server`

## Non-Goals

- v3-cli instrumentation (future work)
- Lineage linking across traces (separate feature)
- Tail-based sampling in OTel Collector (future enhancement — probabilistic head-based sampling for now)

---

## Infrastructure Layer

### SigNoz Docker-Compose Stack

Lives in `observability/` at the repo root:

```
observability/
  docker-compose.yml              # SigNoz stack
  otel-collector-config.yaml      # Custom collector config
```

Start with `docker compose -f observability/docker-compose.yml up -d` before running Petri.

**Containers:** ClickHouse, SigNoz query-service, SigNoz frontend, OTel Collector, alertmanager.

**Ports:**

| Service | Port | Purpose |
|---------|------|---------|
| OTel Collector (gRPC) | 4317 | OTLP span/metric ingestion |
| OTel Collector (HTTP) | 4318 | OTLP HTTP ingestion |
| SigNoz UI | 3301 | Human exploration UI (avoids conflict with Petri frontend) |
| ClickHouse HTTP | 8123 | Agent SQL queries |

**Data flow:**

```
v3-server (Rust)
  ├─ tracing spans ──→ tracing-opentelemetry ──→ OTLP gRPC ──→ OTel Collector (:4317)
  └─ OTel metrics ──→ OTLP gRPC ─────────────→ OTel Collector (:4317)
                                                       │
                                              probabilistic sampling
                                                       │
                                                       ▼
                                              SigNoz (ClickHouse)
                                                       │
                                              ┌────────┴────────┐
                                              │                 │
                                        SigNoz UI (:3301)   HTTP SQL API (:8123)
                                        (human use)         (agent queries)
```

**Retention:** ClickHouse TTL set to 14 days (configurable via SigNoz settings).

**Graceful degradation:** If SigNoz is down, the OTel batch span processor silently drops spans. The simulation is never blocked.

### Agent Query Access

ClickHouse exposes an HTTP interface on `:8123`. Agents send `POST` requests with SQL:

```sql
SELECT SpanName, count()
FROM signoz_traces.distributed_signoz_index_v3
WHERE Timestamp > now() - INTERVAL 1 HOUR
GROUP BY SpanName
ORDER BY count() DESC
```

---

## Rust Instrumentation Architecture

### Crate Dependencies

Added to workspace `Cargo.toml`:

- `opentelemetry` — core API
- `opentelemetry_sdk` — SDK with batch span processor
- `opentelemetry-otlp` — OTLP gRPC exporter (via tonic)
- `tracing-opentelemetry` — bridge from `tracing` spans to OTel spans

The `tracing` crate is already a workspace dependency.

### Instrumentation Boundary

`v3-core` remains a pure simulation library with zero observability dependencies. All OTel instrumentation lives in `v3-server`.

To enable per-phase instrumentation, `v3-core` exposes phase functions (`run_phase_0()`, `run_phase_1()`, `run_phase_2()`, `run_phase_2_5()`) that the server calls in sequence, wrapping each with tracing spans. This aligns with the planned tick-phase-system refactor (`docs/features/needs_refinement/refactors/tick-phase-system.md`).

### Feature Flag

All OTel instrumentation gated behind a cargo feature `otel` in `v3-server`. When disabled, the `tracing` subscriber drops all spans. Useful for CI, profiling, and minimal builds.

### Initialization

At server startup:
1. Configure OTel pipeline with OTLP gRPC exporter pointing at collector endpoint
2. Set up `tracing-opentelemetry` layer
3. Register as global tracing subscriber
4. On shutdown, flush batch processor gracefully

---

## Span Model

### Tick-Level Trace Hierarchy

```
tick                                       (root span, every tick)
├── phase_0_world_updates
│   ├── food_growth
│   ├── creature_aging
│   ├── energy_decay
│   ├── shared_memory_decay
│   └── death_removal
├── phase_1_cognition
│   ├── sensor_assembly
│   ├── mesh_execution
│   └── decision_sorting
├── phase_2_actions
│   └── creature_tick                      (per traced creature — see below)
└── phase_2_5_learning
    └── creature_learn                     (per traced creature — see below)
```

### Tick-Level Attributes

| Attribute | Type | Source |
|-----------|------|--------|
| `tick.number` | u64 | `sim.tick` |
| `tick.population` | u32 | creature count |
| `tick.food_count` | u32 | total food cells |
| `tick.total_energy` | f64 | sum of creature energy |
| `tick.births` | u32 | from `SimStats` |
| `tick.deaths` | u32 | from death removal |
| `tick.predation_kills` | u32 | from `SimStats` |

### Phase-Level Attributes (examples)

- `death_removal`: `deaths.starvation`, `deaths.total`
- `food_growth`: `food.spawned`, `food.total_after`
- `mesh_execution`: `creatures.executed`, `creatures.traced`
- `decision_sorting`: `decisions.count`, `priority_bid.mean`, `priority_bid.max`

### Creature Cognition Traces

**Two tiers:**

| Tier | Who | What |
|------|-----|------|
| **Off** | Most creatures (99%+) | No spans emitted |
| **Full** | Random sample + flagged creatures | Complete cognition trace with per-node execution, signal flow, energy accounting, learning |

**Creature span tree:**

```
creature_tick                              (one per traced creature per tick)
├── creature_sense
│   ├── static_input_assembly
│   └── extended_perception
├── creature_think
│   ├── mesh_node_{id}                     (one per reachable node, in execution order)
│   │   ├── node_input_routing
│   │   ├── node_compute
│   │   └── node_output_routing
│   └── mesh_execution_summary
├── creature_decide
├── creature_action[0]
│   └── creature_action_outcome
├── creature_action[1]
│   └── creature_action_outcome
├── creature_action[N]
│   └── creature_action_outcome
└── creature_learn
    ├── outcome_signal_computation
    └── weight_update[0..M]
```

### creature_tick Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `creature.id` | string | SlotMap key |
| `creature.generation` | u32 | birth generation |
| `creature.age` | u32 | ticks alive |
| `creature.energy` | f64 | energy before actions |
| `creature.phenotype_rgb` | string | e.g., "128,64,200" |
| `creature.mesh_nodes_total` | u32 | |
| `creature.mesh_nodes_reachable` | u32 | |
| `creature.trace_reason` | string | "sampled" or "flagged" |
| `creature.actions_attempted` | u32 | |
| `creature.actions_succeeded` | u32 | |
| `creature.energy_start_of_tick` | f64 | |
| `creature.energy_end_of_tick` | f64 | |
| `creature.total_energy_delta` | f64 | net change |

### creature_sense Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `sense.inputs_count` | u32 | WorldInputKey values assembled |
| `sense.food_here` | f64 | food at creature's cell |
| `sense.food_ahead` | f64 | food in facing direction |
| `sense.neighbors_visible` | u32 | creatures detected |
| `sense.nearest_food_distance` | f64 | |
| `sense.nearest_creature_distance` | f64 | |
| `sense.energy_normalized` | f64 | own energy as input signal |
| `sense.age_normalized` | f64 | own age as input signal |
| `sense.extended_perception` | bool | whether extended perception ran |

### creature_think Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `think.total_energy_cost` | f64 | sum of all node costs |
| `think.vm_energy_cost` | f64 | total VM node cost |
| `think.graph_energy_cost` | f64 | total graph node cost |
| `think.costliest_node_id` | u32 | which node spent the most |
| `think.costliest_node_cost` | f64 | that node's cost |
| `think.nodes_executed` | u32 | reachable nodes evaluated |
| `think.nodes_skipped` | u32 | unreachable nodes |
| `think.memory_reads` | u32 | |
| `think.memory_writes` | u32 | |
| `think.plasticity_updates` | u32 | |

### mesh_node_{id} Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `node.id` | u32 | mesh index |
| `node.execution_order` | u32 | position in topological execution sequence |
| `node.backend` | string | "vm" / "graph" |
| `node.graph_op` | string | graph only — operation type |
| `node.vm_instructions_executed` | u32 | VM only |
| **Inputs** | | |
| `node.input_sources` | string[] | e.g., `["sensor:food_here", "node:3:output", "memory:slot_7"]` |
| `node.input_values` | f64[] | corresponding values |
| **Outputs** | | |
| `node.output_value` | f64 | computed result |
| `node.output_targets` | string[] | e.g., `["node:5:input_0", "output_slot:action_move"]` |
| **Energy** | | |
| `node.energy_cost` | f64 | energy consumed by this node |
| `node.energy_cost_vm` | f64 | VM portion (0 if graph node) |
| `node.energy_cost_graph` | f64 | graph portion (0 if VM node) |
| **Plasticity** | | |
| `node.plasticity_enabled` | bool | |
| `node.weight_before` | f64 | pre-update weight |
| `node.weight_after` | f64 | post-update weight |
| `node.eligibility_trace` | f64 | current trace value |
| **Routing** | | |
| `node.routing_value` | f64 | if routing node |
| `node.routing_decision` | string | which branch taken |

### creature_decide Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `decide.actions_count` | u32 | number of actions resolved |
| `decide.actions_list` | string[] | e.g., `["Move(N)", "Eat", "Reproduce"]` |
| `decide.output_slots` | string | JSON of raw slot values |
| `decide.priority_bid` | f64 | |
| `decide.confidence` | f64 | margin between top action and runner-up |

### creature_action[N] Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `action.index` | u32 | 0-based position in tick's action sequence |
| `action.type` | string | "Move", "Eat", "Reproduce", "Steal", "Noop" |
| `action.direction` | string | if applicable |
| `action.target` | string | if applicable |
| `outcome.result` | string | "success", "blocked_barrier", "blocked_occupied", etc. |
| `outcome.energy_before` | f64 | energy entering this action |
| `outcome.energy_after` | f64 | energy after this action |
| `outcome.energy_gained` | f64 | |
| `outcome.energy_spent` | f64 | |
| `outcome.offspring_id` | string | if reproduce succeeded |
| `outcome.victim_id` | string | if predation |

### creature_learn Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `learn.outcome_signal_count` | u32 | signal bank entries computed |
| `learn.plasticity_edges_total` | u32 | total plasticity-enabled edges |
| `learn.edges_updated` | u32 | edges with non-zero weight update |
| `learn.total_weight_delta` | f64 | sum of absolute weight changes |
| `learn.max_weight_delta` | f64 | largest single weight change |
| `learn.energy_cost` | f64 | cost of learning phase |

### outcome_signal_computation Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `signal.action_outcomes` | string | JSON summary of action results |
| `signal.reward_signal` | f64 | aggregate reward signal |
| `signal.novelty_signal` | f64 | if applicable |
| `signal.energy_delta_signal` | f64 | energy change as learning signal |

### weight_update Span Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `edge.source_node` | u32 | |
| `edge.target_node` | u32 | |
| `edge.weight_before` | f64 | |
| `edge.weight_after` | f64 | |
| `edge.delta` | f64 | |
| `edge.eligibility_trace` | f64 | current trace value |
| `edge.hebbian_term` | f64 | pre/post correlation |
| `edge.modulation_factor` | f64 | reward modulation applied |

---

## OTel Metrics

Exported via OTel SDK, derived from existing `SimStats` each tick.

### Population (gauges)

- `petri.population.total`
- `petri.population.mean_energy`
- `petri.population.total_energy`
- `petri.population.mean_age`
- `petri.population.oldest_age`

### Per-Tick Actions (monotonic counters)

- `petri.tick.births`
- `petri.tick.deaths`
- `petri.tick.predation_kills`
- `petri.tick.actions.{move,eat,reproduce,steal,noop}`

### Food Economics (gauges)

- `petri.food.total`
- `petri.food.spawned_last_tick`
- `petri.food.consumed_last_tick`

### Mutation Funnel (monotonic counters)

- `petri.mutation.attempted`
- `petri.mutation.applied`
- `petri.mutation.skipped` — with attribute `reason`

### Reproduction Funnel (monotonic counters)

- `petri.reproduction.attempted`
- `petri.reproduction.succeeded`
- `petri.reproduction.rejected` — with attribute `reason` (barrier, occupied, contention, energy, age)

### Compute Cost (gauges)

- `petri.compute.mean_total_cost`
- `petri.compute.mean_vm_cost`
- `petri.compute.mean_graph_cost`

### Tick Performance (histograms)

- `petri.tick.duration_ms`
- `petri.tick.phase_0_duration_ms`
- `petri.tick.phase_1_duration_ms`
- `petri.tick.phase_2_duration_ms`
- `petri.tick.phase_2_5_duration_ms`

### Transport (histograms)

- `petri.transport.projection_publish_ms`
- `petri.transport.ws_frame_publish_ms`

---

## Server/Transport Instrumentation

### HTTP Handlers (auto-instrumented)

`tower-http::TraceLayer` on the axum router provides automatic spans for all HTTP requests with method, path, status code, and latency.

### Manual Transport Spans

```
ws_frame_publish                           (each frame broadcast cycle, ~10/sec)
├── projection_compute
│   ├── dirty_rect_calculation
│   ├── creature_sampling
│   └── payload_assembly
├── frame_encoding
└── ws_broadcast
    └── ws_send_client[N]
```

#### ws_frame_publish Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `frame.tick` | u64 | tick this frame represents |
| `frame.connected_clients` | u32 | |
| `frame.creatures_in_viewport` | u32 | creatures in frame |
| `frame.payload_bytes` | u32 | serialized frame size |
| `frame.encoding_format` | string | "msgpack" |

#### projection_compute Attributes

| Attribute | Type | Notes |
|-----------|------|-------|
| `projection.dirty_rects` | u32 | dirty regions |
| `projection.creatures_total` | u32 | total in sim |
| `projection.creatures_sampled` | u32 | in this frame |
| `projection.cache_hit` | bool | projection reused |

---

## Config Panel Integration

### New "Observability" Section

Runtime-adjustable tracing controls exposed in the config panel UI.

#### Sampling Controls

- **Creature trace sampling rate** — slider, 0% to 5%, default 0.1%. At 100K creatures, 0.1% = ~100 traced/tick.
- **Tick trace enabled** — toggle, default on. Controls tick-level phase span emission.

#### Creature Flagging

- **Flag creature for tracing** — button in creature inspector. Toggles persistent full tracing for the selected creature.
- **Flagged creature list** — shows flagged creature IDs with remove buttons. Capped at ~20.

#### Transport Controls

- **OTel export enabled** — master toggle, default on (when built with `otel` feature).
- **OTel endpoint** — text field, default `http://localhost:4317`.

### Server-Side State

```rust
pub struct OtelConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub creature_sample_rate: f64,           // 0.0 to 0.05
    pub tick_tracing_enabled: bool,
    pub flagged_creature_ids: HashSet<CreatureId>,  // max ~20
}
```

This is runtime state in `v3-server`, not `SimulationConfig` in `v3-core`. Changes take effect on the next tick via the existing config panel WebSocket protocol. No restart required.

---

## Sampling Strategy

**Probabilistic head-based sampling** for the initial implementation:

- At the start of each tick, for each creature, roll against `creature_sample_rate` to decide whether to trace
- Flagged creatures are always traced regardless of sample rate
- Tick-level phase spans are always emitted (unless tick tracing is toggled off)
- Metrics are always exported (not sampled)

**Future enhancement:** Tail-based sampling in the OTel Collector with rules like "always keep ticks with deaths above threshold" or "always keep traces containing extinction events."

---

## Volume Estimates

At 100K creatures, ~60 ticks/sec, 0.1% creature sample rate:

| Signal | Volume | Notes |
|--------|--------|-------|
| Tick phase spans | ~360 spans/sec | 6 spans/tick × 60 ticks/sec |
| Creature cognition spans | ~6K–60K spans/sec | ~100 creatures × ~10-100 spans each × 60 ticks/sec |
| Metrics | ~30 time series | population, food, mutation, performance |
| Transport spans | ~50 spans/sec | ~10 frames/sec × ~5 spans/frame |

Total: manageable for local ClickHouse. Adjustable via sample rate slider.

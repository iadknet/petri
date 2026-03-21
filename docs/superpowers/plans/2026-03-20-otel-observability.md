# OpenTelemetry Observability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add OpenTelemetry observability to the Petri simulation with SigNoz backend, covering tick tracing, creature cognition traces, population metrics, and runtime config controls.

**Architecture:** All OTel/tracing dependencies live in `v3-server` behind an `otel` cargo feature. `v3-core` gains new API surface (SimStats counters, batch traced execution) but zero OTel dependencies. A self-hosted SigNoz stack (docker-compose) provides storage, visualization, and SQL-queryable access for coding agents.

**Tech Stack:** `opentelemetry` + `opentelemetry_sdk` + `opentelemetry-otlp` + `tracing-opentelemetry` (Rust), SigNoz + ClickHouse + OTel Collector (Docker), React config panel components (frontend)

**Spec:** `docs/superpowers/specs/2026-03-20-opentelemetry-observability-design.md`

---

## Prerequisites

- **Tick phase system refactor** (`docs/features/needs_refinement/refactors/tick-phase-system.md`) — required for Phase 2 (tick phase spans) and Phase 3 (creature cognition tracing). Phases 1 and 1.5 can proceed without this.

## Phase Overview

| Phase | Name | Blocked? | Companion File |
|-------|------|----------|----------------|
| 1 | Foundation | No | `phase-1-foundation.md` |
| 1.5 | Metrics, Transport & Basic Tick Span | No | `phase-1.5-metrics-transport.md` |
| 2 | Config Panel | No | `phase-2-config-panel.md` |
| 3 | Tick Phase Tracing | Yes — tick-phase-system refactor | `phase-3-advanced-tracing.md` |
| 4 | Creature Cognition Tracing | Yes — Phase 3 | `phase-3-advanced-tracing.md` |

**See also:**
- `phase-1-foundation.md` — SigNoz infrastructure, Rust deps, OTel pipeline init, SimStats additions
- `phase-1.5-metrics-transport.md` — OTel metrics export, transport instrumentation, basic tick span
- `phase-2-config-panel.md` — Observability API endpoints, frontend config section, inspector flag button
- `phase-3-advanced-tracing.md` — Tick phase spans, batch traced execution, creature cognition spans (blocked)

## High-Level Task Outline

### Phase 1: Foundation (no prerequisites)

1. **SigNoz Docker-Compose Stack** — Create `observability/` directory with docker-compose.yml and OTel Collector config. Verify stack starts and SigNoz UI is accessible.

2. **Rust Dependencies & Feature Flag** — Add OTel workspace dependencies to `v3/Cargo.toml`. Add `otel` feature to `v3-server/Cargo.toml` gating the OTel deps. Verify build with and without feature.

3. **OTel Pipeline Initialization** — Create `v3-server/src/observability/` module with pipeline init/shutdown, run_id generation, and tracing-opentelemetry subscriber layer. Verify spans export to collector.

4. **SimStats Additions (TDD)** — Add `last_tick_births`, `last_tick_deaths`, `last_tick_food_consumed` to `SimStats` and `food_spawned` to `FoodGrowthSummary`. Increment in tick.rs. Test-first.

### Phase 1.5: Metrics, Transport & Basic Tick Span (no prerequisites)

5. **OTel Metrics Export** — Create metrics recording module. Export population gauges, action counters, mutation/reproduction/predation funnels, food economics, compute costs. Verify in SigNoz.

6. **Transport Instrumentation** — Add `tower-http::TraceLayer` to axum router. Add manual spans for `ws_frame_publish` cycle. Verify HTTP and transport spans in SigNoz.

7. **Basic Tick Span** — Wrap `run_tick()` call with a root `tick` span carrying tick-level attributes (tick number, population, run_id). This works with the current monolithic `run_tick()` — sub-phase spans come in Phase 3.

### Phase 2: Config Panel (no prerequisites, but benefits from Phase 1)

8. **Observability API Endpoints (TDD)** — `GET/PATCH /v3/observability/config`, `POST /v3/observability/flag/{creature_id}`, `GET /v3/observability/flagged`. Separate from simulation config.

9. **Config Panel Frontend — Observability Section** — New `ObservabilitySection.tsx` with master toggle, sample rate slider, tick tracing toggle, endpoint field. Wire into `RuntimeConfigPanel`.

10. **Inspector Flag Button** — Add "Flag for Tracing" button to creature inspector. Wire to flag toggle endpoint.

### Phase 3: Tick Phase Tracing (blocked on tick-phase-system refactor)

11. **Tick Phase Spans** — Wrap each exposed phase function with tracing spans. Add `queue_build` span with `stable_sort` and `shuffle` children. Sub-phase spans for food_growth, death_removal, sensor_assembly, mesh_execution, decision_sorting.

12. **Phase Timing Metrics** — Export `petri.tick.phase_{0,1,2,2_5}_duration_ms` histograms from wall-clock measurements around phase calls.

### Phase 4: Creature Cognition Tracing (blocked on Phase 3)

13. **Batch Traced Execution in Core** — New core API that accepts a set of creature IDs to trace within the parallel cognition pass. Returns `Vec<MeshHopTrace>` per traced creature alongside normal `MeshOutput`.

14. **Creature Cognition Span Emission** — Server converts `MeshHopTrace` data into OTel spans: `creature_tick` → `creature_sense` → `creature_think` (with per-node `mesh_node_{id}` spans) → `creature_decide` → `creature_action[N]` → `creature_learn`.

15. **Sampling Integration** — Wire sampling rate from `OtelConfig` into the batch traced execution. Roll per-creature at tick start, merge with flagged set, pass to core.

## Implementation Steps (Review Gates)

- [ ] Phase 1 tasks (1–4) implemented and tested
- [ ] Phase 1.5 tasks (5–7) implemented and tested
- [ ] Phase 2 tasks (8–10) implemented and tested
- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent covering all implemented phases. Fix findings, re-review until clean. Invoke `rust-skills` for Rust code quality.
- [ ] Review Gate: Architecture & decomposition review — verify boundary compliance (no OTel deps in core), consistency with `docs/strategy/architecture.md`, dependency direction, and public API surface.
- [ ] Re-run completion gate checks after review-introduced changes
- [ ] Phase 3 tasks (11–12) implemented and tested (after tick-phase-system refactor)
- [ ] Phase 4 tasks (13–15) implemented and tested (after Phase 3)
- [ ] Review Gate: Code review for Phases 3–4
- [ ] Review Gate: Architecture & decomposition review for Phases 3–4

## Completion Gate

Before claiming completion of each phase, run and confirm all pass:

1. `cd v3 && cargo fmt --all -- --check`
2. `cd v3 && cargo test --workspace`
3. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
4. `cd v3 && cargo test --workspace --features otel` (OTel-specific tests)
5. `cd frontend && npm run build` (if frontend files were touched)

## Implementation Notes

- **Cargo.lock changes:** Adding 6+ OTel workspace dependencies will significantly change `Cargo.lock`. These are intentional dependency additions — do NOT revert lockfile changes.
- **Transport span sub-tree deferred:** The spec defines detailed `ws_frame_publish` sub-spans (`projection_compute` with 3 children, `frame_encoding`, `ws_broadcast`). Phase 1.5 Task 6 implements only the top-level `ws_frame_publish` span. Full sub-tree is deferred to a future iteration.
- **Global subscriber conflict in tests:** The OTel pipeline calls `tracing_subscriber::registry().init()` which sets the global default subscriber. Test code must NOT call `init_otel_pipeline` — use `try_init` or test-specific subscribers to avoid panics from double-init.

## Boundary Impact

- **Dependency direction:** `v3-server` depends on `v3-core` (unchanged). OTel crates are `v3-server`-only behind `otel` feature.
- **Public API changes in v3-core:** New SimStats fields (4), new `food_spawned` field in `FoodGrowthSummary`, future batch traced execution API.
- **New server API surface:** `/v3/observability/*` endpoints (4 routes).
- **Wire format:** No changes to existing WS protocol. New endpoints use JSON.
- **Test migration:** Existing tests unchanged. New tests for SimStats counters and observability endpoints.

**Review cycles:** 0 (initial draft)

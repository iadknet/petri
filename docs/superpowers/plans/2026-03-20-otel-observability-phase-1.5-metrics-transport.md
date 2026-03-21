# Phase 1.5: Metrics, Transport & Basic Tick Span

**Parent plan:** `2026-03-20-otel-observability.md`

---

### Task 5: OTel Metrics Export

**Files:**
- Create: `v3/crates/v3-server/src/observability/metrics.rs`
- Modify: `v3/crates/v3-server/src/observability/mod.rs` (add metrics module)
- Modify: `v3/crates/v3-server/src/observability/pipeline.rs` (add metrics pipeline init)
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (call metrics recording after each tick)

- [ ] **Step 1: Add metrics pipeline initialization**

In `pipeline.rs`, add meter provider setup alongside the trace provider:

```rust
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub struct OtelPipeline {
    pub tracer_provider: SdkTracerProvider,
    pub meter_provider: SdkMeterProvider,
}

pub fn init_otel_pipeline(endpoint: &str) -> OtelPipeline {
    // ... existing tracer setup ...

    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("failed to build OTLP metrics exporter");

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(metrics_exporter)
        .build();

    OtelPipeline {
        tracer_provider: provider,
        meter_provider,
    }
}
```

Note: Exact API depends on pinned crate versions. Adjust builder methods as needed.

- [ ] **Step 2: Create metrics recording module**

```rust
// v3/crates/v3-server/src/observability/metrics.rs
use opentelemetry::metrics::Meter;
use v3_core::simulation::Simulation;

/// Record all OTel metrics for the current tick state.
/// Called once per tick from the run loop.
pub fn record_tick_metrics(sim: &Simulation, run_id: &str, meter: &Meter) {
    let attrs = [opentelemetry::KeyValue::new("simulation.run_id", run_id.to_string())];

    // Population gauges
    let population = sim.creatures.len() as u64;
    meter.u64_gauge("petri.population.total")
        .build()
        .record(population, &attrs);

    let (total_energy, total_age, oldest_age) = sim.creatures.values().fold(
        (0.0_f64, 0_u64, 0_u32),
        |(energy, age_sum, oldest), c| {
            (energy + c.energy as f64, age_sum + c.age_ticks as u64, oldest.max(c.age_ticks))
        },
    );

    meter.f64_gauge("petri.population.total_energy")
        .build()
        .record(total_energy, &attrs);

    if population > 0 {
        meter.f64_gauge("petri.population.mean_energy")
            .build()
            .record(total_energy / population as f64, &attrs);
        meter.f64_gauge("petri.population.mean_age")
            .build()
            .record(total_age as f64 / population as f64, &attrs);
    }

    meter.u64_gauge("petri.population.oldest_age")
        .build()
        .record(oldest_age as u64, &attrs);

    // Per-tick action counters
    let stats = &sim.stats;
    meter.u64_counter("petri.tick.births")
        .build()
        .add(stats.last_tick_births as u64, &attrs);
    meter.u64_counter("petri.tick.deaths")
        .build()
        .add(stats.last_tick_deaths as u64, &attrs);
    meter.u64_counter("petri.tick.actions.move")
        .build()
        .add(stats.last_tick_move as u64, &attrs);
    meter.u64_counter("petri.tick.actions.eat")
        .build()
        .add(stats.last_tick_eat as u64, &attrs);
    meter.u64_counter("petri.tick.actions.reproduce")
        .build()
        .add(stats.last_tick_reproduce as u64, &attrs);
    meter.u64_counter("petri.tick.actions.steal_energy")
        .build()
        .add(stats.last_tick_steal as u64, &attrs);
    meter.u64_counter("petri.tick.actions.noop")
        .build()
        .add(stats.last_tick_noop as u64, &attrs);

    // Food economics
    let food_count = sim.world.food().occupied_count();
    meter.u64_gauge("petri.food.total")
        .build()
        .record(food_count as u64, &attrs);
    meter.u64_counter("petri.tick.food_consumed")
        .build()
        .add(stats.last_tick_food_consumed as u64, &attrs);

    // Compute cost gauges
    meter.f64_gauge("petri.compute.mean_total_cost")
        .build()
        .record(stats.last_tick_compute_total_mean as f64, &attrs);
    meter.f64_gauge("petri.compute.mean_vm_cost")
        .build()
        .record(stats.last_tick_compute_vm_mean as f64, &attrs);
    meter.f64_gauge("petri.compute.mean_graph_cost")
        .build()
        .record(stats.last_tick_compute_graph_mean as f64, &attrs);
}
```

Note: This is a simplified version. The actual implementation should also record predation, mutation, and reproduction funnel counters from the cumulative `SimStats` fields. The pattern is the same — read from `SimStats`, record via OTel meter. The exact method names (`u64_gauge`, `u64_counter`, `f64_gauge`) and builder patterns depend on the pinned `opentelemetry` crate version.

- [ ] **Step 3: Add metrics module to observability/mod.rs**

```rust
#[cfg(feature = "otel")]
pub mod metrics;
```

- [ ] **Step 4: Call metrics recording from run loop**

In `handlers/lifecycle.rs`, after `run_tick()`:

```rust
#[cfg(feature = "otel")]
if handle.otel_config.enabled {
    // meter should be stored in AppState or obtained from global provider
    crate::observability::metrics::record_tick_metrics(
        &handle.sim,
        &handle.run_id,
        &meter,
    );
}
```

The `Meter` instance should be created once during pipeline init and stored in `AppState` (or obtained from the global `MeterProvider`). Add it to the `OtelPipeline` return and thread it through.

- [ ] **Step 5: Verify metrics appear in SigNoz**

Start SigNoz + Petri server with `otel` feature. Run simulation for a few ticks. Open SigNoz UI → Metrics Explorer → search for `petri.population.total`.

Expected: Metric data points visible.

- [ ] **Step 6: Commit**

```bash
git add v3/crates/v3-server/src/observability/
git commit -m "feat: add OTel metrics export for population, actions, food, compute"
```

---

### Task 6: Transport Instrumentation

**Files:**
- Modify: `v3/crates/v3-server/Cargo.toml` (add tower-http trace feature)
- Modify: `v3/crates/v3-server/src/lib.rs` (add TraceLayer to router)
- Modify: `v3/crates/v3-server/src/state.rs` (add spans to publish_ws_frame_update)

- [ ] **Step 1: Add tower-http tracing feature**

Check if `tower-http` already has the `trace` feature enabled. If not, add it:

```toml
# In v3/crates/v3-server/Cargo.toml
tower-http = { workspace = true, features = ["compression-gzip", "trace"] }
```

This may need to be conditional on the `otel` feature, or unconditionally available (since `tracing` is already a workspace dep and TraceLayer works with any tracing subscriber).

- [ ] **Step 2: Add TraceLayer to router**

In `v3/crates/v3-server/src/lib.rs`:

```rust
use tower_http::trace::TraceLayer;

pub fn router(state: app_state::AppState) -> axum::Router {
    axum::Router::new()
        // ... existing routes ...
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}
```

Note: Layer order matters in axum — TraceLayer should wrap outside CompressionLayer so timing includes compression.

- [ ] **Step 3: Add spans to ws_frame_publish**

In `state.rs`, wrap `publish_ws_frame_update` with a tracing span:

```rust
pub fn publish_ws_frame_update(
    &self,
    frame: WsFrame,
    dirty_rect: Option<DirtyRect>,
    world_static_changed: bool,
) {
    let _span = tracing::info_span!(
        "ws_frame_publish",
        frame.tick = frame.tick,
        frame.connected_clients = tracing::field::Empty,
        frame.payload_bytes = tracing::field::Empty,
    ).entered();

    let started = Instant::now();
    // ... existing publish logic ...
}
```

The `tracing::field::Empty` fields can be filled later with `Span::current().record()` once the values are known. This is a progressive enhancement — start with what's easy, add more attributes as you verify the spans appear correctly.

- [ ] **Step 4: Verify HTTP spans in SigNoz**

Start SigNoz + server with `otel` feature. Make an HTTP request:

```bash
curl http://localhost:3000/v3/simulation/status
```

Check SigNoz Traces tab for an HTTP span with method=GET, path=/v3/simulation/status.

- [ ] **Step 5: Commit**

```bash
git add v3/crates/v3-server/
git commit -m "feat: add transport instrumentation (HTTP TraceLayer + ws_frame_publish spans)"
```

---

### Task 7: Basic Tick Span

**Files:**
- Create: `v3/crates/v3-server/src/observability/tracing.rs`
- Modify: `v3/crates/v3-server/src/observability/mod.rs` (add tracing module)
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (wrap run_tick with span)

- [ ] **Step 1: Create tracing helpers module**

```rust
// v3/crates/v3-server/src/observability/tracing.rs

/// Create the root tick span with standard attributes.
/// This works with the current monolithic run_tick() — sub-phase spans
/// will be added in Phase 3 after the tick-phase-system refactor.
pub fn tick_span(tick_number: u64, population: usize, run_id: &str) -> tracing::Span {
    tracing::info_span!(
        "tick",
        "simulation.run_id" = run_id,
        "tick.number" = tick_number,
        "tick.population" = population as u32,
        "tick.food_count" = tracing::field::Empty,
        "tick.total_energy" = tracing::field::Empty,
        "tick.births" = tracing::field::Empty,
        "tick.deaths" = tracing::field::Empty,
        "tick.predation_kills" = tracing::field::Empty,
    )
}

/// Record post-tick attributes on the current tick span.
pub fn record_tick_post_attributes(
    span: &tracing::Span,
    food_count: u32,
    total_energy: f64,
    births: u32,
    deaths: u32,
    predation_kills: u32,
) {
    span.record("tick.food_count", food_count);
    span.record("tick.total_energy", total_energy);
    span.record("tick.births", births);
    span.record("tick.deaths", deaths);
    span.record("tick.predation_kills", predation_kills);
}
```

- [ ] **Step 2: Add tracing module to observability/mod.rs**

```rust
#[cfg(feature = "otel")]
pub mod tracing_spans;  // avoid name collision with `tracing` crate
```

Rename the file to `tracing_spans.rs` to avoid collision with the `tracing` crate name.

- [ ] **Step 3: Wrap run_tick in lifecycle.rs**

In the `run_loop` function and the `step` handler, wrap the `run_tick()` call:

```rust
// In run_loop:
#[cfg(feature = "otel")]
let tick_span = if h.otel_config.enabled && h.otel_config.tick_tracing_enabled {
    let span = crate::observability::tracing_spans::tick_span(
        h.sim.tick,
        h.sim.creatures.len(),
        &h.run_id,
    );
    Some(span)
} else {
    None
};

#[cfg(feature = "otel")]
let _tick_guard = tick_span.as_ref().map(|s| s.enter());

run_tick(&mut h.sim, &mut h.active_trace);

#[cfg(feature = "otel")]
if let Some(ref span) = tick_span {
    let total_energy: f64 = h.sim.creatures.values().map(|c| c.energy as f64).sum();
    crate::observability::tracing_spans::record_tick_post_attributes(
        span,
        h.sim.world.food().occupied_count() as u32,
        total_energy,
        h.sim.stats.last_tick_births,
        h.sim.stats.last_tick_deaths,
        h.sim.stats.last_tick_predation_kills,
    );
}
```

- [ ] **Step 4: Verify tick spans in SigNoz**

Start SigNoz + server with `otel` feature. Run simulation. Check SigNoz Traces for `tick` spans with `tick.number`, `tick.population`, `simulation.run_id` attributes.

Expected: Tick spans visible with correct attributes.

- [ ] **Step 5: Verify run_id changes on startup**

Call `POST /v3/simulation/startup` to reset. Verify new tick spans have a different `simulation.run_id`.

- [ ] **Step 6: Run full test suite**

```bash
cd v3 && cargo test --workspace && cargo test --workspace --features otel
```

Expected: All pass.

- [ ] **Step 7: Commit**

```bash
git add v3/crates/v3-server/src/observability/ v3/crates/v3-server/src/handlers/lifecycle.rs
git commit -m "feat: add basic tick span with tick-level attributes and run_id"
```

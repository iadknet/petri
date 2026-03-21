# Phase 1: Foundation

**Parent plan:** `2026-03-20-otel-observability.md`

---

### Task 1: SigNoz Docker-Compose Stack

**Files:**
- Create: `observability/docker-compose.yml`
- Create: `observability/otel-collector-config.yaml`
- Create: `observability/.env`

- [ ] **Step 1: Create OTel Collector config**

```yaml
# observability/otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  clickhousetraces:
    datasource: tcp://clickhouse:9000/?database=signoz_traces
  clickhousemetricsv4:
    datasource: tcp://clickhouse:9000/?database=signoz_metrics
  clickhouselogsv2:
    datasource: tcp://clickhouse:9000/?database=signoz_logs

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [clickhousetraces]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [clickhousemetricsv4]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [clickhouselogsv2]
```

Note: SigNoz provides its own collector config. The above is a starting template — the actual config will be derived from SigNoz's official docker-compose by overriding the collector config file. Consult the pinned SigNoz version's documentation for the correct exporter names and schema.

- [ ] **Step 2: Create docker-compose.yml**

Use SigNoz's official `docker-compose.yaml` as the base. Pin to a specific SigNoz release tag. Key overrides:
- SigNoz frontend port: `3301:3301` (avoid conflict with Petri frontend on 3000)
- Mount custom `otel-collector-config.yaml`
- ClickHouse HTTP port exposed: `8123:8123` (for agent SQL queries)

Fetch the official SigNoz docker-compose for the pinned version and customize. Do NOT write from scratch — SigNoz has ~10 interconnected services with specific version requirements.

```bash
# Fetch SigNoz docker-compose (pin version)
cd observability
# Follow https://signoz.io/docs/install/docker/ for the correct setup
```

- [ ] **Step 3: Create .env with port overrides**

```env
# observability/.env
SIGNOZ_FRONTEND_PORT=3301
```

- [ ] **Step 4: Verify stack starts**

```bash
cd observability && docker compose up -d
```

Expected: All containers healthy. SigNoz UI accessible at `http://localhost:3301`.

- [ ] **Step 5: Verify ClickHouse HTTP API**

```bash
curl 'http://localhost:8123/' --data-binary 'SELECT 1'
```

Expected: `1`

- [ ] **Step 6: Add observability/ to .gitignore for data volumes**

Add to root `.gitignore`:
```
observability/clickhouse-data/
observability/signoz-data/
```

- [ ] **Step 7: Commit**

```bash
git add observability/ .gitignore
git commit -m "feat: add SigNoz docker-compose stack for OTel observability"
```

---

### Task 2: Rust Dependencies & Feature Flag

**Files:**
- Modify: `v3/Cargo.toml` (workspace dependencies)
- Modify: `v3/crates/v3-server/Cargo.toml` (otel feature + deps)

- [ ] **Step 1: Add OTel workspace dependencies**

Add to `[workspace.dependencies]` in `v3/Cargo.toml`:

```toml
opentelemetry = "0.28"
opentelemetry_sdk = { version = "0.28", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.28", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.29"
tracing-subscriber = { version = "0.3", features = ["env-filter", "registry"] }
uuid = { version = "1", features = ["v4"] }
```

Note: Pin to specific compatible versions. The OTel Rust ecosystem has breaking changes between minor versions. Check crate compatibility before finalizing versions — the versions above are illustrative. Use the latest compatible set at implementation time.

- [ ] **Step 2: Add otel feature to v3-server**

In `v3/crates/v3-server/Cargo.toml`:

```toml
[features]
default = []
otel = [
    "dep:opentelemetry",
    "dep:opentelemetry_sdk",
    "dep:opentelemetry-otlp",
    "dep:tracing-opentelemetry",
    "dep:tracing-subscriber",
]

[dependencies]
# Always compiled (no OTel dependency)
uuid = { workspace = true }

# OTel deps (optional, behind otel feature)
opentelemetry = { workspace = true, optional = true }
opentelemetry_sdk = { workspace = true, optional = true }
opentelemetry-otlp = { workspace = true, optional = true }
tracing-opentelemetry = { workspace = true, optional = true }
tracing-subscriber = { workspace = true, optional = true }
```

- [ ] **Step 3: Verify build without feature**

```bash
cd v3 && cargo check -p v3-server
```

Expected: Compiles cleanly. No OTel code pulled in.

- [ ] **Step 4: Verify build with feature**

```bash
cd v3 && cargo check -p v3-server --features otel
```

Expected: Compiles cleanly with OTel deps available.

- [ ] **Step 5: Commit**

```bash
git add v3/Cargo.toml v3/crates/v3-server/Cargo.toml
git commit -m "feat: add OTel workspace dependencies and otel feature flag"
```

---

### Task 3: OTel Pipeline Initialization

**Files:**
- Create: `v3/crates/v3-server/src/observability/mod.rs`
- Create: `v3/crates/v3-server/src/observability/pipeline.rs`
- Create: `v3/crates/v3-server/src/observability/config.rs`
- Modify: `v3/crates/v3-server/src/lib.rs` (add observability module, init pipeline)
- Modify: `v3/crates/v3-server/src/bin/server.rs` (call init before serve)
- Modify: `v3/crates/v3-server/src/state.rs` (add run_id and OtelConfig to AppState/SimHandle)

- [ ] **Step 1: Create observability module root**

```rust
// v3/crates/v3-server/src/observability/mod.rs
#[cfg(feature = "otel")]
pub mod pipeline;

pub mod config;
```

- [ ] **Step 2: Create OtelConfig struct**

```rust
// v3/crates/v3-server/src/observability/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use v3_core::contracts::CreatureId;

/// Maximum number of creatures that can be flagged for tracing simultaneously.
pub const MAX_FLAGGED_CREATURES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub creature_sample_rate: f64,
    pub tick_tracing_enabled: bool,
    #[serde(skip)]
    pub flagged_creature_ids: HashSet<CreatureId>,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:4317".to_string(),
            creature_sample_rate: 0.001, // 0.1%
            tick_tracing_enabled: true,
            flagged_creature_ids: HashSet::new(),
        }
    }
}
```

- [ ] **Step 3: Create pipeline init/shutdown**

```rust
// v3/crates/v3-server/src/observability/pipeline.rs
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize the OTel tracing pipeline. Call once at server startup.
/// Returns a guard that flushes on drop.
pub fn init_otel_pipeline(endpoint: &str) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("failed to build OTLP exporter");

    let resource = Resource::builder()
        .with_service_name("petri-server")
        .build();

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("petri-server");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(otel_layer)
        .init();

    provider
}

/// Flush and shut down the OTel pipeline gracefully.
pub fn shutdown_otel_pipeline(provider: SdkTracerProvider) {
    if let Err(e) = provider.shutdown() {
        eprintln!("OTel shutdown error: {e}");
    }
}
```

Note: The exact API may differ based on the `opentelemetry` crate version pinned in Task 2. Consult the crate docs for the pinned version. The above follows the 0.28.x API pattern — adjust types and builder methods as needed.

- [ ] **Step 4: Add run_id and OtelConfig to server state**

In `v3/crates/v3-server/src/state.rs`, add to `SimHandle`:

```rust
pub struct SimHandle {
    pub sim: Simulation,
    pub status: SimulationStatus,
    pub active_trace: Option<v3_core::runtime::trace::recording::ActiveTrace>,
    pub cached_fertility_u8: Arc<[u8]>,
    pub run_id: String,  // UUID v4, regenerated on startup/reset
    pub otel_config: crate::observability::config::OtelConfig,
}
```

**Design decision:** `OtelConfig` and `run_id` are **always compiled** (no `#[cfg(feature = "otel")]` gating). The config management (sample rates, flagged creatures) does not require OTel libraries. Only the pipeline init, span emission, and metrics export use OTel deps and need `#[cfg]` gating. This means the observability API endpoints (`/v3/observability/*`) are always available, and the frontend can always render controls. When the `otel` feature is disabled, controls are present but export does nothing.

The `uuid` crate must also be an unconditional dependency (not optional) since `run_id` is always generated.

In `SimHandle::new_default()`, generate initial `run_id`:

```rust
pub fn new_default() -> Self {
    // ...existing code...
    Self {
        // ...existing fields...
        run_id: uuid::Uuid::new_v4().to_string(),
        otel_config: Default::default(),
    }
}
```

In the `startup` handler (`handlers/lifecycle.rs`), regenerate `run_id` and clear flags:

```rust
// Inside startup handler, after creating new SimHandle:
handle.run_id = uuid::Uuid::new_v4().to_string();
handle.otel_config.flagged_creature_ids.clear();
```

- [ ] **Step 5: Wire pipeline init into server binary**

In `v3/crates/v3-server/src/bin/server.rs`:

```rust
#[tokio::main]
async fn main() {
    #[cfg(feature = "otel")]
    let provider = v3_server::observability::pipeline::init_otel_pipeline("http://localhost:4317");

    let bind_addr = resolve_bind_addr(std::env::var(BIND_ADDR_ENV).ok());
    let app = v3_server::router(v3_server::state::AppState::new());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    #[cfg(feature = "otel")]
    v3_server::observability::pipeline::shutdown_otel_pipeline(provider);
}
```

- [ ] **Step 6: Add module to lib.rs**

In `v3/crates/v3-server/src/lib.rs`, add:

```rust
pub mod observability;
```

- [ ] **Step 7: Verify build and startup**

```bash
cd v3 && cargo build -p v3-server --features otel
```

Expected: Compiles. Server binary starts and initializes OTel pipeline (may warn about collector connection if SigNoz is not running — that's expected graceful degradation).

- [ ] **Step 8: Integration test — verify spans reach SigNoz**

Start SigNoz stack, start server with `otel` feature, make a request, verify a span appears in SigNoz UI at `http://localhost:3301`.

- [ ] **Step 9: Commit**

```bash
git add v3/crates/v3-server/src/observability/ v3/crates/v3-server/src/lib.rs v3/crates/v3-server/src/bin/server.rs v3/crates/v3-server/src/state.rs
git commit -m "feat: add OTel pipeline initialization with run_id and OtelConfig"
```

---

### Task 4: SimStats Additions (TDD)

**Files:**
- Modify: `v3/crates/v3-core/src/simulation/stats.rs` (add counters)
- Modify: `v3/crates/v3-core/src/kernel/food_resource/mod.rs` (add food_spawned to FoodGrowthSummary)
- Modify: `v3/crates/v3-core/src/simulation/tick.rs` (increment counters)
- Test: `v3/crates/v3-core/src/simulation/stats.rs` (inline tests)

- [ ] **Step 1: Add new per-tick counter fields to SimStats**

In `SimStats` struct, in the per-tick section (alongside `last_tick_move`, `last_tick_eat`, etc.):

```rust
/// Per-tick birth count (successful reproduction spawns). Reset each tick.
pub last_tick_births: u32,
/// Per-tick death count (creatures removed in Phase 0). Reset each tick.
pub last_tick_deaths: u32,
/// Per-tick food consumption count (successful Eat actions). Reset each tick.
pub last_tick_food_consumed: u32,
/// Per-tick food cells spawned (cells transitioning from 0 to >0). Reset each tick.
pub last_tick_food_spawned: u32,
```

- [ ] **Step 2: Verify build compiles**

```bash
cd v3 && cargo check -p v3-core
```

Expected: May fail if existing tests construct `FoodGrowthSummary` with struct literal syntax. Fix any compilation errors in existing tests by adding the new fields (see Step 12b below).

- [ ] **Step 9: Write failing test for food_spawned in FoodGrowthSummary**

Add to `v3/crates/v3-core/src/kernel/food_resource/mod.rs` tests (or create inline test module):

```rust
#[test]
fn food_growth_summary_includes_food_spawned() {
    let summary = FoodGrowthSummary::default();
    assert_eq!(summary.food_spawned, 0);
}
```

- [ ] **Step 10: Add food_spawned field to FoodGrowthSummary**

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FoodGrowthSummary {
    pub mean_occupancy_depletion: f32,
    pub occupied_cells_with_depletion: u32,
    pub growth_suppressed_by_occupancy_depletion: f32,
    /// Number of cells that received food during this tick's food growth.
    pub food_spawned: u32,
}
```

- [ ] **Step 11: Verify test passes**

```bash
cd v3 && cargo test -p v3-core food_growth_summary_includes_food_spawned
```

Expected: PASS

- [ ] **Step 12: Increment food_spawned in food growth logic**

The food growth implementation is in `v3/crates/v3-core/src/kernel/food_resource/growth.rs`. Food growth is continuous (not discrete spawning), so "food spawned" means **cells that transitioned from 0 to >0 food density** during this tick's growth pass. In the `apply_growth` function, check each cell's food value before and after growth application. If it was 0 before and >0 after, increment `food_spawned`.

The `FoodGrowthSummary` is constructed in `FoodResource::grow()` which calls `apply_growth`. The `food_spawned` count should be accumulated during the growth pass and returned as part of the summary.

Also, the existing `SimStats::record_food_growth_summary()` method should be updated to store `food_spawned` in a new SimStats field (e.g., `last_tick_food_spawned: u32`) so it's available for metrics export.

- [ ] **Step 12b: Update existing test that constructs FoodGrowthSummary**

The test `record_food_growth_summary_updates_food_depletion_fields` in `stats.rs` (line ~429) constructs a `FoodGrowthSummary` with struct literal syntax. Adding `food_spawned` to the struct will break this test at compile time. Update it to include `food_spawned: 5` (or any value) and verify the field is stored correctly:

```rust
stats.record_food_growth_summary(FoodGrowthSummary {
    mean_occupancy_depletion: 0.12,
    occupied_cells_with_depletion: 3,
    growth_suppressed_by_occupancy_depletion: 0.7,
    food_spawned: 5,
});
// Add assertion for food_spawned if stored in SimStats
```

- [ ] **Step 13: Increment last_tick_deaths in tick.rs death removal**

In `tick.rs`, the death removal section (around line 117–127), after collecting dead IDs:

```rust
let dead_ids: Vec<_> = sim
    .creatures
    .iter()
    .filter(|(_, c)| c.energy <= 0.0)
    .map(|(id, _)| id)
    .collect();

sim.stats.last_tick_deaths = dead_ids.len() as u32;

for id in dead_ids {
    remove_creature_from_sim(sim, id);
}
```

- [ ] **Step 14: Increment last_tick_births in reproduction success path**

In `tick.rs`, find the `WorldAction::Reproduce` arm (around line ~585). The cumulative counter `reproduction_actions_spawned_total` is incremented in `actions/reproduction.rs`, but the per-tick `last_tick_births` counter should be incremented in `tick.rs` where the reproduce action outcome is checked. Look for the code path where reproduction succeeds (offspring is created), and add:

```rust
sim.stats.last_tick_births += 1;
```

Do NOT put this in `actions/reproduction.rs` — keep per-tick counters in `tick.rs` where the other per-tick counters (`last_tick_move`, `last_tick_eat`, etc.) are incremented.

- [ ] **Step 15: Increment last_tick_food_consumed in eat success path**

In `tick.rs`, in the `WorldAction::Eat` arm (~line 442). The existing code structure is:

```rust
let succeeded = apply_eat(creature, &mut sim.world, &sim.config);
// ...
if succeeded {
    amount = food_before;
} else {
    // action_result = ActionResult::NoFood; etc.
}
```

Add the increment **inside** the existing `if succeeded` block (do not restructure the if/else):

```rust
if succeeded {
    amount = food_before;
    sim.stats.last_tick_food_consumed += 1;  // <-- add this line
} else {
    // ... existing code unchanged ...
}
```

- [ ] **Step 16: Reset new per-tick counters at tick start**

In `tick.rs`, find where existing per-tick counters are reset (e.g., `sim.stats.last_tick_move = 0`). Add:

```rust
sim.stats.last_tick_births = 0;
sim.stats.last_tick_deaths = 0;
sim.stats.last_tick_food_consumed = 0;
```

- [ ] **Step 17: Run full test suite**

```bash
cd v3 && cargo test --workspace
```

Expected: All tests pass, including viability tests.

- [ ] **Step 18: Run clippy**

```bash
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
```

Expected: No warnings.

- [ ] **Step 19: Commit**

```bash
git add v3/crates/v3-core/src/simulation/stats.rs v3/crates/v3-core/src/kernel/food_resource/ v3/crates/v3-core/src/simulation/tick.rs
git commit -m "feat: add per-tick births, deaths, food_consumed counters to SimStats"
```

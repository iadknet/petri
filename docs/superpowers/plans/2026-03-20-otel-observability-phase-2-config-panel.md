# Phase 2: Config Panel

**Parent plan:** `2026-03-20-otel-observability.md`

---

### Task 8: Observability API Endpoints (TDD)

**Files:**
- Create: `v3/crates/v3-server/src/handlers/observability.rs`
- Modify: `v3/crates/v3-server/src/handlers/mod.rs` (add observability module)
- Modify: `v3/crates/v3-server/src/lib.rs` (add routes)

- [ ] **Step 1: Write failing test for GET /v3/observability/config**

Add to `v3/crates/v3-server/tests/server.rs`, following the existing test pattern that uses `app()` + `do_request()` with tower `oneshot`:

```rust
#[tokio::test]
async fn get_observability_config_returns_defaults() {
    let req = Request::builder()
        .uri("/v3/observability/config")
        .body(Body::empty())
        .unwrap();
    let (status, body) = do_request(app(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], true);
    assert_eq!(body["creature_sample_rate"], 0.001);
    assert_eq!(body["tick_tracing_enabled"], true);
    assert_eq!(body["endpoint"], "http://localhost:4317");
}
```

Note: This follows the existing test pattern in `server.rs` where `app()` returns `router(AppState::new())` and `do_request()` sends via tower `oneshot`.

- [ ] **Step 2: Run test to verify it fails**

```bash
cd v3 && cargo test -p v3-server get_observability_config_returns_defaults
```

Expected: FAIL — handler does not exist.

- [ ] **Step 3: Implement GET /v3/observability/config handler**

```rust
// v3/crates/v3-server/src/handlers/observability.rs
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::error::AppError;

#[derive(Serialize)]
struct ObservabilityConfigResponse {
    enabled: bool,
    endpoint: String,
    creature_sample_rate: f64,
    tick_tracing_enabled: bool,
    flagged_creature_ids: Vec<String>,
}

pub async fn get_config(
    State(app): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let config = &handle.otel_config;
    Ok(Json(ObservabilityConfigResponse {
        enabled: config.enabled,
        endpoint: config.endpoint.clone(),
        creature_sample_rate: config.creature_sample_rate,
        tick_tracing_enabled: config.tick_tracing_enabled,
        flagged_creature_ids: config.flagged_creature_ids
            .iter()
            .map(|id| format!("{:?}", id))
            .collect(),
    }))
}
```

- [ ] **Step 4: Add route and verify test passes**

In `lib.rs`, add route:

```rust
.route("/v3/observability/config", get(http::observability::get_config))
```

In `handlers/mod.rs`:

```rust
pub mod observability;
```

```bash
cd v3 && cargo test -p v3-server get_observability_config_returns_defaults
```

Expected: PASS

- [ ] **Step 5: Write failing test for PATCH /v3/observability/config**

Note: PATCH + GET in the same test requires a shared `AppState` since `app()` creates a new state each time. Use `AppState::new()` once and build two routers, or use the `spawn_ws_app` pattern for a persistent server. Follow the existing test patterns in `server.rs` for stateful tests.

```rust
#[tokio::test]
async fn patch_observability_config_updates_sample_rate() {
    let state = AppState::new();

    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/v3/observability/config")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"creature_sample_rate": 0.05}"#))
        .unwrap();
    let (status, _) = do_request(router(state.clone()), patch_req).await;
    assert_eq!(status, StatusCode::OK);

    let get_req = Request::builder()
        .uri("/v3/observability/config")
        .body(Body::empty())
        .unwrap();
    let (status, body) = do_request(router(state), get_req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["creature_sample_rate"], 0.05);
}
```

- [ ] **Step 6: Implement PATCH handler**

```rust
#[derive(Deserialize)]
pub struct PatchObservabilityConfig {
    pub enabled: Option<bool>,
    pub endpoint: Option<String>,
    pub creature_sample_rate: Option<f64>,
    pub tick_tracing_enabled: Option<bool>,
}

pub async fn patch_config(
    State(app): State<AppState>,
    Json(patch): Json<PatchObservabilityConfig>,
) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;
    let config = &mut handle.otel_config;

    if let Some(enabled) = patch.enabled {
        config.enabled = enabled;
    }
    if let Some(endpoint) = patch.endpoint {
        config.endpoint = endpoint;
    }
    if let Some(rate) = patch.creature_sample_rate {
        config.creature_sample_rate = rate.clamp(0.0, 0.05);
    }
    if let Some(tick_tracing) = patch.tick_tracing_enabled {
        config.tick_tracing_enabled = tick_tracing;
    }

    Ok(StatusCode::OK)
}
```

Add route: `.route("/v3/observability/config", patch(http::observability::patch_config))`

- [ ] **Step 7: Verify PATCH test passes**

```bash
cd v3 && cargo test -p v3-server patch_observability_config_updates_sample_rate
```

Expected: PASS

- [ ] **Step 8: Write failing test for POST /v3/observability/flag/{creature_id}**

This test needs a running simulation with creatures. Use `AppState::new()`, start the simulation (POST /v3/simulation/startup + POST /v3/simulation/start + POST /v3/simulation/step to create creatures), then test flagging. Follow existing stateful test patterns in `server.rs`.

```rust
#[tokio::test]
async fn flag_creature_toggles_tracing() {
    let state = AppState::new();

    // Start simulation and step to create creatures
    let startup_req = Request::builder()
        .method("POST")
        .uri("/v3/simulation/startup")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    do_request(router(state.clone()), startup_req).await;

    // Get a creature ID from the simulation
    // (implementation detail: access state.sim to find a creature ID,
    //  or GET /v3/simulation/status and extract a creature from the response)
    let creature_id = {
        let handle = state.sim.lock().await;
        let (id, _) = handle.sim.creatures.iter().next().expect("need at least one creature");
        format!("{}", id.data().as_ffi())
    };

    // Flag creature
    let flag_req = Request::builder()
        .method("POST")
        .uri(&format!("/v3/observability/flag/{}", creature_id))
        .body(Body::empty())
        .unwrap();
    let (status, _) = do_request(router(state.clone()), flag_req).await;
    assert_eq!(status, StatusCode::OK);

    // Verify flagged
    let flagged_req = Request::builder()
        .uri("/v3/observability/flagged")
        .body(Body::empty())
        .unwrap();
    let (status, body) = do_request(router(state.clone()), flagged_req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.as_array().unwrap().is_empty());

    // Toggle off
    let unflag_req = Request::builder()
        .method("POST")
        .uri(&format!("/v3/observability/flag/{}", creature_id))
        .body(Body::empty())
        .unwrap();
    do_request(router(state.clone()), unflag_req).await;

    let flagged_req = Request::builder()
        .uri("/v3/observability/flagged")
        .body(Body::empty())
        .unwrap();
    let (_, body) = do_request(router(state), flagged_req).await;
    assert!(body.as_array().unwrap().is_empty());
}
```

- [ ] **Step 9: Implement flag toggle and flagged list handlers**

```rust
pub async fn toggle_flag(
    State(app): State<AppState>,
    axum::extract::Path(creature_id_str): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;
    let creature_id = parse_creature_id(&creature_id_str, &handle.sim)?;

    let config = &mut handle.otel_config;
    if config.flagged_creature_ids.contains(&creature_id) {
        config.flagged_creature_ids.remove(&creature_id);
    } else {
        if config.flagged_creature_ids.len() >= crate::observability::config::MAX_FLAGGED_CREATURES {
            return Err(AppError::InvalidRequest(
                "Maximum flagged creatures reached (20)".into(),
            ));
        }
        config.flagged_creature_ids.insert(creature_id);
    }

    Ok(StatusCode::OK)
}

pub async fn get_flagged(
    State(app): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let ids: Vec<String> = handle.otel_config.flagged_creature_ids
        .iter()
        .map(|id| format!("{:?}", id))
        .collect();
    Ok(Json(ids))
}
```

Add routes:

```rust
.route("/v3/observability/flag/:creature_id", post(http::observability::toggle_flag))
.route("/v3/observability/flagged", get(http::observability::get_flagged))
```

Note: The `parse_creature_id` helper needs to convert a string creature ID from the URL path into a `CreatureId` (SlotMap key). Follow the pattern used by the existing `GET /v3/simulation/creature/:id` handler for ID parsing.

- [ ] **Step 10: Verify flag test passes**

```bash
cd v3 && cargo test -p v3-server flag_creature_toggles_tracing
```

Expected: PASS

- [ ] **Step 11: Write test for flag cap enforcement**

```rust
#[tokio::test]
async fn flag_creature_returns_400_at_cap() {
    // Flag MAX_FLAGGED_CREATURES creatures, then try one more
    // Expect 400 Bad Request (AppError::InvalidRequest)
    // Note: AppError has no Conflict variant. Use InvalidRequest.
}
```

- [ ] **Step 12: Verify cap test passes**

- [ ] **Step 13: Write test for dead creature flag cleanup**

When a flagged creature dies (energy <= 0, removed in Phase 0), its flag should be automatically removed. This cleanup should happen in the death removal path or at the start of each tick.

```rust
#[tokio::test]
async fn flagged_creature_removed_on_death() {
    // Flag a creature, kill it (set energy to 0), run a tick
    // Verify flag is removed from flagged set
}
```

- [ ] **Step 14: Implement dead creature flag cleanup**

In `handlers/lifecycle.rs`, after `run_tick()`:

```rust
// Clean up flags for dead creatures
handle.otel_config.flagged_creature_ids.retain(|id| handle.sim.creatures.contains_key(*id));
```

- [ ] **Step 15: Run full test suite**

```bash
cd v3 && cargo test --workspace
```

Expected: All pass.

- [ ] **Step 16: Commit**

```bash
git add v3/crates/v3-server/src/handlers/observability.rs v3/crates/v3-server/src/handlers/mod.rs v3/crates/v3-server/src/lib.rs
git commit -m "feat: add observability API endpoints (config, flag, flagged)"
```

---

### Task 9: Config Panel Frontend — Observability Section

**Files:**
- Create: `frontend/src/components/config-panel/runtime/ObservabilitySection.tsx`
- Modify: `frontend/src/components/config-panel/runtime/RuntimeConfigPanel.tsx` (add section)
- Modify: `frontend/src/api.ts` or equivalent (add observability API calls)

- [ ] **Step 1: Add observability API client methods**

The API client lives at `frontend/src/api/rest.ts` and uses a class-based pattern (`ApiClient` class with `private async request<T>()` method). Add methods to the existing class:

```typescript
// In ApiClient class in frontend/src/api/rest.ts

async getObservabilityConfig(): Promise<ObservabilityConfig> {
  return this.request<ObservabilityConfig>('/v3/observability/config');
}

async patchObservabilityConfig(patch: Partial<ObservabilityConfig>): Promise<void> {
  await this.request('/v3/observability/config', {
    method: 'PATCH',
    body: JSON.stringify(patch),
  });
}

async toggleCreatureTraceFlag(creatureId: string): Promise<void> {
  await this.request(`/v3/observability/flag/${creatureId}`, { method: 'POST' });
}

async getFlaggedCreatures(): Promise<string[]> {
  return this.request<string[]>('/v3/observability/flagged');
}
```

Add the `ObservabilityConfig` interface in the appropriate types file:

```typescript
export interface ObservabilityConfig {
  enabled: boolean;
  endpoint: string;
  creature_sample_rate: number;
  tick_tracing_enabled: boolean;
  flagged_creature_ids: string[];
}
```

- [ ] **Step 2: Create ObservabilitySection component**

Follow the pattern of existing runtime sections (e.g., `MutationSection.tsx`). Use the shared `FieldRow`, `ToggleRow`, and `Section` components:

```tsx
// frontend/src/components/config-panel/runtime/ObservabilitySection.tsx
import { Section } from '../shared/Section';
import { ToggleRow } from '../shared/ToggleRow';
import { FieldRow } from '../shared/FieldRow';
import { useObservabilityConfig } from '../../../hooks/useObservabilityConfig';

export function ObservabilitySection() {
  const { config, patch } = useObservabilityConfig();

  if (!config) return null;

  return (
    <Section title="Observability">
      <ToggleRow
        label="OTel Export Enabled"
        tooltip="Master toggle for OpenTelemetry span and metric export"
        value={config.enabled}
        onChange={(v) => patch({ enabled: v })}
      />
      <ToggleRow
        label="Tick Tracing"
        tooltip="Emit tracing spans for each simulation tick"
        value={config.tick_tracing_enabled}
        onChange={(v) => patch({ tick_tracing_enabled: v })}
      />
      <FieldRow
        label="Creature Sample Rate"
        tooltip="Fraction of creatures traced per tick (0-5%)"
        value={config.creature_sample_rate}
        min={0}
        max={0.05}
        step={0.001}
        onChange={(v) => patch({ creature_sample_rate: v })}
      />
      {config.flagged_creature_ids.length > 0 && (
        <div>
          <label>Flagged Creatures ({config.flagged_creature_ids.length}/20)</label>
          <ul>
            {config.flagged_creature_ids.map((id) => (
              <li key={id}>{id}</li>
            ))}
          </ul>
        </div>
      )}
    </Section>
  );
}
```

**Important:** The existing `RuntimeConfigPanel` uses a data-driven pattern: sections are defined as arrays of `FieldDef` objects and rendered through `RuntimeFieldGroup` components, all backed by the shared `localDraft`/`serverConfig` state for `SimulationConfig`. The observability config is **independent server-side state** (not part of `SimulationConfig`), so this section **cannot use the same data-driven pattern**. It needs its own data fetching via the new API endpoints. Place it as a standalone section at the bottom of the runtime panel, visually separated, using the shared `Section`, `FieldRow`, and `ToggleRow` components for consistent styling but with its own state management via `useObservabilityConfig` hook.

- [ ] **Step 3: Create useObservabilityConfig hook**

```typescript
// frontend/src/hooks/useObservabilityConfig.ts
import { useState, useEffect, useCallback } from 'react';
import { getObservabilityConfig, patchObservabilityConfig, ObservabilityConfig } from '../api';

export function useObservabilityConfig() {
  const [config, setConfig] = useState<ObservabilityConfig | null>(null);

  useEffect(() => {
    getObservabilityConfig().then(setConfig);
  }, []);

  const patch = useCallback(async (update: Partial<ObservabilityConfig>) => {
    await patchObservabilityConfig(update);
    const updated = await getObservabilityConfig();
    setConfig(updated);
  }, []);

  return { config, patch };
}
```

- [ ] **Step 4: Wire into RuntimeConfigPanel**

In `RuntimeConfigPanel.tsx`, import and add `<ObservabilitySection />` at an appropriate position (likely at the end, since it's a cross-cutting concern rather than a simulation parameter).

- [ ] **Step 5: Verify section renders**

```bash
cd frontend && npm run build && npm run dev
```

Open the config panel in the browser. Verify the Observability section appears with toggle, slider, and field controls.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/components/config-panel/runtime/ObservabilitySection.tsx frontend/src/hooks/useObservabilityConfig.ts frontend/src/api.ts frontend/src/components/config-panel/runtime/RuntimeConfigPanel.tsx
git commit -m "feat: add Observability section to config panel"
```

---

### Task 10: Inspector Flag Button

**Files:**
- Modify: creature inspector component (find the exact file — likely in `frontend/src/components/inspector/`)
- Modify: `frontend/src/api.ts` (add flag toggle API call)

- [ ] **Step 1: Add flag toggle API function**

```typescript
export async function toggleCreatureTraceFlag(creatureId: string): Promise<void> {
  await fetch(`/v3/observability/flag/${creatureId}`, { method: 'POST' });
}
```

- [ ] **Step 2: Add flag button to inspector**

Find the creature inspector component. Add a button that calls the flag toggle endpoint:

```tsx
<button
  onClick={() => toggleCreatureTraceFlag(creature.id)}
  title="Toggle detailed OTel tracing for this creature"
>
  {isFlagged ? '🔍 Unflag' : '🔍 Flag for Tracing'}
</button>
```

The `isFlagged` state can be derived by checking if the creature's ID is in the `flagged_creature_ids` list from the observability config.

Note: Follow the inspector's existing button/action patterns for styling and layout. Check the existing inspector components to match the UI conventions.

- [ ] **Step 3: Verify button works**

Open inspector for a creature, click "Flag for Tracing". Verify the creature appears in the Observability section's flagged list. Click again to unflag.

- [ ] **Step 4: Verify frontend builds**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add frontend/
git commit -m "feat: add 'Flag for Tracing' button to creature inspector"
```

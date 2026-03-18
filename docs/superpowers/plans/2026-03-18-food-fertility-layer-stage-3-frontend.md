# Stage 3: Frontend — Companion Plan

**Parent plan:** `2026-03-18-food-fertility-layer.md`
**Spec:** `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md`
**Prerequisite:** Stage 2 complete (server transport includes `food_fertility_u8`)

> **For agentic workers:** Use `vercel-react-best-practices` and `vercel-composition-patterns` for all React/TypeScript code. Use `frontend-design` skill for overlay visual design.

---

## Task 16: Protocol Types

**Files:**
- Modify: `frontend/src/types/config.ts`

- [ ] **Step 1:** Add fertility and annealing config types

```typescript
export interface FertilityConfig {
  enabled: boolean;
  min_fertility: number;
  max_fertility: number;
  layers: FertilityLayer[];
}

export interface FertilityLayer {
  algorithm: FertilityAlgorithm;
  weight: number;
}

export type FertilityAlgorithm =
  | { Uniform: { value: number } }
  | { Fbm: { octaves: number; frequency: number; lacunarity: number; persistence: number; seed: number | null } }
  | { PoissonBlobs: { blob_count: number; min_radius: number; max_radius: number; falloff: number; seed: number | null } };

export interface AnnealingConfig {
  enabled: boolean;
  ramp_ticks: number;
  initial_min_fertility: number;
  initial_max_fertility: number;
}
```

- [ ] **Step 2:** Update `FoodConfig` to include new fields

```typescript
export interface FoodConfig {
  growth_rate: number;
  initial_density: number;
  initial_coverage: number;
  spread_threshold_ratio: number;
  spread_density_ratio: number;
  recovery_spawn_rate: number;
  recovery_floor_ratio: number;
  max_density: number;
  fertility?: FertilityConfig;
  annealing?: AnnealingConfig;
}
```

- [ ] **Step 3:** Add world-static fertility type for received data

```typescript
export interface WorldStaticData {
  width: number;
  height: number;
  barriers: Uint8Array;
  food_fertility_u8?: Uint8Array;
}
```

- [ ] **Step 4:** Verify build

Run: `cd frontend && npm run build`
Expected: compiles without errors

- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add fertility protocol types to frontend"
```

---

## Task 17: Store Wiring

**Files:**
- Modify: `frontend/src/stores/simulation.ts`

- [ ] **Step 1:** Add fertility grid to store state

Add a `fertilityGrid: Uint8Array | null` field to the simulation store. Update the store to ingest `food_fertility_u8` from the world-static payload when received.

```typescript
interface SimulationState {
  // ... existing fields
  fertilityGrid: Uint8Array | null;
  worldWidth: number;
  worldHeight: number;
}
```

- [ ] **Step 2:** Write test for store update

```typescript
test('stores fertility grid from world static payload', () => {
  const store = useSimulationStore.getState();
  const fertilityData = new Uint8Array([0, 128, 255, 64]);
  store.setFertilityGrid(fertilityData, 2, 2);
  expect(store.fertilityGrid).toEqual(fertilityData);
});
```

- [ ] **Step 3:** Implement store update action
- [ ] **Step 4:** Wire into message handler that receives world-static data
- [ ] **Step 5:** Run tests

Run: `cd frontend && npm test`

- [ ] **Step 6:** Commit

```bash
git commit -m "feat: store fertility grid data in simulation store"
```

---

## Task 18: Fertility Overlay Rendering

**Files:**
- Create: `frontend/src/canvas/fertility-overlay.ts` (or modify existing overlay system)
- Modify: `frontend/src/canvas/renderer.ts`

- [ ] **Step 1:** Implement fertility overlay renderer

Create a renderer that draws a semi-transparent color layer over the canvas:
- Read `fertilityGrid` from store
- Map `u8` values to colors: 0 (barren) → brown/red, 128 (neutral) → transparent, 255 (fertile) → green
- Apply configurable opacity

- [ ] **Step 2:** Add toggle control

Add a toggle button/checkbox to the control bar:
- Label: "Fertility"
- When toggled on, render the overlay
- When toggled off, hide it
- Default: off

- [ ] **Step 3:** Write tests

```typescript
test('overlay toggle shows/hides fertility layer', () => {
  // render component, toggle on, verify overlay element present
  // toggle off, verify overlay element absent
});

test('overlay handles missing fertility data gracefully', () => {
  // render with fertilityGrid = null, verify no errors
});
```

- [ ] **Step 4:** Run tests and verify build

Run: `cd frontend && npm test && npm run build`

- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add fertility overlay rendering with toggle control"
```

---

## Stage 3 Gates

- [ ] Run: `cd frontend && npm test` — all pass
- [ ] Run: `cd frontend && npm run build` — builds clean
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `vercel-react-best-practices` and `vercel-composition-patterns`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run frontend tests after review-introduced changes

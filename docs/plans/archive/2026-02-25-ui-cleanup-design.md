# UI Cleanup + Frontend Hygiene

- **Goal**: Improve the frontend config panel UX with per-field reset, clean startup/runtime split, restart preservation, tooltips, zoom controls, and structural hygiene fixes.
- **Goal IDs**: GP-02 (clean architecture boundaries), GP-04 (keep behavior observable)
- **Scope**: Frontend only (`frontend/src/`). No backend changes.
- **Docs Impact**: This plan only. No canonical docs touched.
- **Supersedes**: none
- **Superseded-By**: none

> **Tracking**: Update checkboxes `[ ]` → `[x]` as each item is completed.

## Goal Alignment

| Goal | Alignment |
|------|-----------|
| GP-02 | Deduplicate repeated section components, extract shared utils, rename misleading components, split oversized files |
| GP-04 | Add tooltips for config field discoverability, per-field reset for clarity, zoom controls for world inspection |

## Boundary Impact

- **Frontend only** — no crate or API changes.
- **No public API/wire-format changes** — restart behavior change uses existing `POST /startup` + `PATCH /config` endpoints sequentially.
- **Component file renames** — `CollapsibleGroup` → `FieldGroup`. Import paths updated in all consumers.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-server` REST API | keep | All changes use existing `POST /startup` and `PATCH /config` endpoints. No server modifications needed. |
| `frontend/src/stores/` | keep | Store interfaces stay the same. We add a `buildStartupRequest()` method to `useStartupConfigStore` and modify `handleRestart` in ControlBar to re-PATCH runtime config, but store shapes are unchanged. |

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| Should `growth_rate` appear in startup panel? | No — it's live-patchable, runtime-only | user | resolved |
| Should `initial_energy` move to startup? | Yes — only used at seed time | user | resolved |
| Tooltip implementation approach? | CSS-only (no external library) | user | resolved |
| Zoom button behavior? | Incremental 1.5x steps + fit-to-world | user | resolved |

---

## Stage 1: Hygiene Fixes

These structural improvements prepare the codebase for the feature additions.

- [x] **1.1 Extract shared `deepSet` utility**
  - Create `frontend/src/utils/deepSet.ts`
  - Move `deepSet` from `stores/config.ts` and `stores/startupConfig.ts` into shared module
  - Update imports in both stores

- [x] **1.2 Rename `CollapsibleGroup` → `FieldGroup`**
  - Rename `frontend/src/components/config-panel/shared/CollapsibleGroup.tsx` → `FieldGroup.tsx`
  - Update the component name and all import references

- [x] **1.3 Deduplicate runtime section components**
  - Create `frontend/src/components/config-panel/runtime/RuntimeFieldGroup.tsx`
  - Single component accepting `fields: FieldDef[]`, `title: string`, plus existing runtime props
  - Replace 6 identical section components with data-only field constant exports + `RuntimeFieldGroup` usage in `RuntimeConfigPanel.tsx`
  - Files affected: `FoodParametersSection.tsx`, `PopulationSection.tsx`, `EnergyLifecycleSection.tsx`, `EnergyCostsSection.tsx`, `RuntimeSection.tsx`, `MutationSection.tsx`

- [x] **1.4 Split `StatsPanel.tsx`**
  - Create `frontend/src/components/stats/` directory
  - Extract: `Gauge.tsx`, `ActionBar.tsx`, `MiniChart.tsx` → `components/stats/shared/`
  - Extract: `OverviewTab.tsx`, `ActionsTab.tsx`, `EvolutionTab.tsx`, `ComputationTab.tsx` → `components/stats/`
  - `StatsPanel.tsx` becomes a thin shell importing tabs

- [x] **1.5 Extract restart business logic**
  - Add `buildStartupRequest()` method to `useStartupConfigStore`
  - Simplify `ControlBar.tsx` `handleRestart` to call `store.buildStartupRequest()`

## Stage 2: Config Panel Split

- [x] **2.1 Move `initial_energy` to startup panel**
  - Add `energy.lifecycle.initial_energy` field to startup panel's field definitions
  - Add to `StartupPreset` type and `buildDefaultPreset()`
  - Remove from `EnergyLifecycleSection` runtime fields
  - Include in the `buildStartupRequest()` payload

- [x] **2.2 Remove `growth_rate` from startup panel**
  - Remove `world.food.growth_rate` from startup `FoodParametersSection` field definitions
  - It remains in the runtime `FoodParametersSection` only

- [x] **2.3 Verify no field overlap between panels**
  - Manually confirm each field appears in exactly one panel (startup or runtime)

## Stage 3: Per-Field Reset Buttons

- [x] **3.1 Define default values map**
  - Create `frontend/src/components/config-panel/shared/defaults.ts`
  - Export `RUNTIME_DEFAULTS: Record<string, number>` derived from `SimulationConfig` default values
  - Startup defaults already exist in `buildDefaultPreset()`

- [x] **3.2 Add reset button to `FieldRow`**
  - Add optional `defaultValue: number` prop to `FieldRow`
  - Render a `↺` reset icon at the right edge when `value !== defaultValue`
  - On click, call `onChange(field.path, defaultValue)`
  - Style: small, muted, appears/disappears based on value diff

## Stage 4: Restart Preserves Runtime Config

- [x] **4.1 Modify `handleRestart` to re-PATCH runtime config**
  - After `api.startup()` succeeds, read `useConfigStore.getState().serverConfig`
  - Build a patch containing only runtime-patchable fields (exclude startup-only fields)
  - Call `api.patchConfig(patch)`
  - Then re-fetch and commit as before

- [x] **4.2 Define `RUNTIME_PATCH_FIELDS` list**
  - Verify the existing `RUNTIME_PATCH_FIELDS` constant in `ConfigPanel.tsx` covers exactly the right fields
  - Use it for the restart re-PATCH logic (or move to a shared location)

## Stage 5: Field Tooltips

- [x] **5.1 Create `Tooltip` component**
  - Create `frontend/src/components/config-panel/shared/Tooltip.tsx`
  - CSS-only tooltip: positioned absolutely relative to a trigger element
  - Appears on hover with a brief delay
  - Simple arrow/callout styling consistent with petri theme

- [x] **5.2 Add tooltip text to field definitions**
  - Add optional `tooltip: string` property to `FieldDef` type
  - Populate tooltip descriptions for all config fields

- [x] **5.3 Integrate tooltip into `FieldRow`**
  - Add `ⓘ` icon next to field label (only when `field.tooltip` is defined)
  - Wrap with `Tooltip` component showing the description text

## Stage 6: Zoom Controls

- [x] **6.1 Create `ZoomControls` component**
  - Create `frontend/src/components/ZoomControls.tsx`
  - Three buttons: `+` (zoom in), `-` (zoom out), fit-to-world icon
  - Styled: `bg-slate-800/60 backdrop-blur-sm`, rounded, hover brightness
  - Positioned absolutely in upper-right of canvas viewport

- [x] **6.2 Wire zoom controls to renderer**
  - `+` button: call `renderer.zoomAt(centerX, centerY, -1)` (zoom in by ~1.5x toward center)
  - `-` button: call `renderer.zoomAt(centerX, centerY, 1)` (zoom out by ~1.5x from center)
  - Fit button: call `renderer.resetView()`

- [x] **6.3 Add to `WorldViewport`**
  - Render `ZoomControls` inside the viewport container
  - Pass renderer ref for zoom/fit callbacks

---

## Verification

```bash
cd frontend && npm run build          # no build errors
cd frontend && npx vitest run         # all existing tests pass
```

Manual verification:
- Config panel shows no overlapping fields between startup/runtime
- Reset buttons appear only when value differs from default
- Restart preserves current energy/mutation/runtime config values
- Tooltips show on hover for config fields
- Zoom +/- and fit buttons work in canvas viewport

## Risks / Rollback

- **Risk**: Component renames/splits may break existing tests → run full test suite after each stage
- **Rollback**: All changes are frontend-only, easily reverted via git

**Review cycles:** 1

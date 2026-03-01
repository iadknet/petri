# UI Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve config panel UX (per-field reset, clean startup/runtime split, restart preservation, tooltips, zoom controls) and clean up frontend structural hygiene.

**Architecture:** Config panel uses Zustand stores (`useConfigStore`, `useStartupConfigStore`) with FieldRow-based rendering. Zoom controls overlay the canvas viewport. Structural cleanup deduplicates 6 identical runtime section components into a single data-driven component.

**Tech Stack:** React 19, TypeScript 5.7, Tailwind CSS 4, Zustand 5, Vite 6, Vitest

**See also:** `docs/plans/2026-02-25-ui-cleanup-design.md` (design doc)

> **Tracking**: Update checkboxes `[ ]` → `[x]` as each item is completed.

---

### Task 1: Extract shared `deepSet` utility

**Files:**
- Create: `frontend/src/utils/deepSet.ts`
- Modify: `frontend/src/stores/config.ts:22-35`
- Modify: `frontend/src/stores/startupConfig.ts:62-75`

**Step 1: Create the shared utility file**

```ts
// frontend/src/utils/deepSet.ts
export function deepSet<T extends Record<string, unknown>>(obj: T, path: string, value: unknown): T {
	const clone = structuredClone(obj);
	const keys = path.split(".");
	let current: Record<string, unknown> = clone;
	for (let i = 0; i < keys.length - 1; i++) {
		const key = keys[i]!;
		if (typeof current[key] !== "object" || current[key] === null) {
			current[key] = {};
		}
		current = current[key] as Record<string, unknown>;
	}
	current[keys[keys.length - 1]!] = value;
	return clone;
}
```

**Step 2: Update `stores/config.ts`**

Remove the local `deepSet` function (lines 22-35). Add import:
```ts
import { deepSet } from "../utils/deepSet.ts";
```

**Step 3: Update `stores/startupConfig.ts`**

Remove the local `deepSet` function (lines 62-75). Add import:
```ts
import { deepSet } from "../utils/deepSet.ts";
```

**Step 4: Run tests to verify**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 5: Commit**
```
feat: extract shared deepSet utility from stores
```

- [x] Task 1 complete

---

### Task 2: Rename `CollapsibleGroup` → `FieldGroup`

**Files:**
- Rename: `frontend/src/components/config-panel/shared/CollapsibleGroup.tsx` → `FieldGroup.tsx`
- Modify: all files that import `CollapsibleGroup` (6 runtime sections + 4 startup sections)

**Step 1: Rename file and component**

Rename `CollapsibleGroup.tsx` to `FieldGroup.tsx`. Change the component name inside:

```tsx
// frontend/src/components/config-panel/shared/FieldGroup.tsx
import type { ReactNode } from "react";

interface FieldGroupProps {
	title: string;
	children: ReactNode;
}

export function FieldGroup({ title, children }: FieldGroupProps) {
	return (
		<div className="border-b border-petri-border">
			<div className="w-full px-3 py-2 text-xs font-medium text-slate-300 bg-slate-900/20">
				{title}
			</div>
			<div className="px-3 pb-2 flex flex-col gap-0.5">{children}</div>
		</div>
	);
}
```

**Step 2: Update all imports**

In every file that imports `CollapsibleGroup`, change to:
```ts
import { FieldGroup } from "../shared/FieldGroup.tsx";
```
And replace `<CollapsibleGroup` with `<FieldGroup` in JSX.

Files to update:
- `frontend/src/components/config-panel/runtime/EnergyCostsSection.tsx`
- `frontend/src/components/config-panel/runtime/EnergyLifecycleSection.tsx`
- `frontend/src/components/config-panel/runtime/FoodParametersSection.tsx`
- `frontend/src/components/config-panel/runtime/MutationSection.tsx`
- `frontend/src/components/config-panel/runtime/PopulationSection.tsx`
- `frontend/src/components/config-panel/runtime/RuntimeSection.tsx`
- `frontend/src/components/config-panel/startup/FoodParametersSection.tsx`
- `frontend/src/components/config-panel/startup/PopulationSection.tsx`
- `frontend/src/components/config-panel/startup/WorldTopologySection.tsx`
- `frontend/src/components/config-panel/startup/RunSettingsSection.tsx`

**Step 3: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 4: Commit**
```
refactor: rename CollapsibleGroup to FieldGroup (it was never collapsible)
```

- [x] Task 2 complete

---

### Task 3: Deduplicate runtime section components

**Files:**
- Create: `frontend/src/components/config-panel/runtime/RuntimeFieldGroup.tsx`
- Modify: `frontend/src/components/config-panel/runtime/RuntimeConfigPanel.tsx`
- Modify: 6 runtime section files (keep field constant exports, remove component exports)

**Step 1: Create `RuntimeFieldGroup` component**

```tsx
// frontend/src/components/config-panel/runtime/RuntimeFieldGroup.tsx
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

interface RuntimeFieldGroupProps extends RuntimePanelProps {
	title: string;
	fields: FieldDef[];
}

export function RuntimeFieldGroup({
	title,
	fields,
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimeFieldGroupProps) {
	return (
		<FieldGroup title={title}>
			{fields.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</FieldGroup>
	);
}
```

**Step 2: Convert 6 runtime section files to data-only exports**

Each file keeps its `FieldDef[]` constant export but removes the component function. Example for `EnergyCostsSection.tsx`:

```tsx
// frontend/src/components/config-panel/runtime/EnergyCostsSection.tsx
import type { FieldDef } from "../shared/types.ts";

export const ENERGY_COSTS_FIELDS: FieldDef[] = [
	// ... same field definitions as before ...
];
```

Remove imports of `FieldGroup`, `FieldRow`, `getByPath`, and the component function from each of:
- `EnergyCostsSection.tsx`
- `EnergyLifecycleSection.tsx`
- `FoodParametersSection.tsx`
- `MutationSection.tsx`
- `PopulationSection.tsx`
- `RuntimeSection.tsx`

**Step 3: Update `RuntimeConfigPanel.tsx`**

Replace 6 individual section component imports with `RuntimeFieldGroup`:

```tsx
import { Section } from "../shared/Section.tsx";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";
import { ENERGY_COSTS_FIELDS } from "./EnergyCostsSection.tsx";
import { ENERGY_LIFECYCLE_FIELDS } from "./EnergyLifecycleSection.tsx";
import { FOOD_PARAMETERS_FIELDS } from "./FoodParametersSection.tsx";
import { MUTATION_FIELDS } from "./MutationSection.tsx";
import { POPULATION_FIELDS } from "./PopulationSection.tsx";
import { RUNTIME_FIELDS } from "./RuntimeSection.tsx";
import { RuntimeFieldGroup } from "./RuntimeFieldGroup.tsx";

export const RUNTIME_PATCH_FIELDS: FieldDef[] = [
	...FOOD_PARAMETERS_FIELDS,
	...POPULATION_FIELDS,
	...ENERGY_LIFECYCLE_FIELDS,
	...ENERGY_COSTS_FIELDS,
	...RUNTIME_FIELDS,
	...MUTATION_FIELDS,
];

// ... props interface unchanged ...

export function RuntimeConfigPanel({ localDraft, serverConfig, simState, updateDraft }: RuntimeConfigPanelProps) {
	return (
		<Section
			title="Runtime (Live) Config"
			description="These settings apply to the current simulation when allowed by lifecycle state."
			collapsible
			sectionClassName="bg-sky-950/10 border-l-2 border-l-sky-500"
		>
			{!localDraft || !serverConfig ? (
				<div className="p-3 text-sm text-slate-500">
					Runtime config unavailable. Restart to initialize the simulation.
				</div>
			) : (
				<>
					<RuntimeFieldGroup title="Food Parameters" fields={FOOD_PARAMETERS_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Population" fields={POPULATION_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Energy > Lifecycle" fields={ENERGY_LIFECYCLE_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Energy > Costs" fields={ENERGY_COSTS_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Runtime" fields={RUNTIME_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Mutation" fields={MUTATION_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
				</>
			)}
		</Section>
	);
}
```

**Step 4: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass. The existing tests check testIds on specific fields (e.g., `config-field-energy-costs-move-cost`) which are still rendered by `RuntimeFieldGroup`.

**Step 5: Commit**
```
refactor: deduplicate 6 runtime section components into data-driven RuntimeFieldGroup
```

- [x] Task 3 complete

---

### Task 4: Split `StatsPanel.tsx`

**Files:**
- Create: `frontend/src/components/stats/shared/Gauge.tsx`
- Create: `frontend/src/components/stats/shared/ActionBar.tsx`
- Create: `frontend/src/components/stats/shared/MiniChart.tsx`
- Create: `frontend/src/components/stats/OverviewTab.tsx`
- Create: `frontend/src/components/stats/ActionsTab.tsx`
- Create: `frontend/src/components/stats/EvolutionTab.tsx`
- Create: `frontend/src/components/stats/ComputationTab.tsx`
- Modify: `frontend/src/components/StatsPanel.tsx`

**Step 1: Create `Gauge.tsx`**

```tsx
// frontend/src/components/stats/shared/Gauge.tsx
export function Gauge({
	label,
	value,
	delta,
	format,
}: { label: string; value: number; delta?: number; format?: (v: number) => string }) {
	const fmt = format ?? ((v: number) => v.toLocaleString());
	return (
		<div className="flex flex-col">
			<span className="text-[11px] font-sans text-slate-400">{label}</span>
			<div className="flex items-baseline gap-1.5">
				<span className="text-xl font-mono text-slate-100 tabular-nums">{fmt(value)}</span>
				{delta !== undefined && delta !== 0 && (
					<span
						className={`text-[11px] font-mono tabular-nums ${
							delta > 0 ? "text-emerald-400" : "text-red-400"
						}`}
					>
						{delta > 0 ? "+" : ""}
						{fmt(delta)}
					</span>
				)}
			</div>
		</div>
	);
}
```

**Step 2: Create `ActionBar.tsx`**

```tsx
// frontend/src/components/stats/shared/ActionBar.tsx
export function ActionBar({
	move,
	eat,
	reproduce,
	noop,
}: { move: number; eat: number; reproduce: number; noop: number }) {
	const total = move + eat + reproduce + noop || 1;
	const pct = (n: number) => `${((n / total) * 100).toFixed(1)}%`;

	return (
		<div className="flex flex-col gap-1">
			<span className="text-[11px] text-slate-400">Actions</span>
			<div className="flex h-3 rounded overflow-hidden">
				<div className="bg-blue-500" style={{ width: pct(move) }} title={`Move: ${move}`} />
				<div className="bg-emerald-500" style={{ width: pct(eat) }} title={`Eat: ${eat}`} />
				<div className="bg-amber-500" style={{ width: pct(reproduce) }} title={`Reproduce: ${reproduce}`} />
				<div className="bg-slate-600" style={{ width: pct(noop) }} title={`Noop: ${noop}`} />
			</div>
			<div className="flex gap-3 text-[10px] text-slate-500">
				<span className="flex items-center gap-1"><span className="w-2 h-2 bg-blue-500 rounded-sm" />Move {move}</span>
				<span className="flex items-center gap-1"><span className="w-2 h-2 bg-emerald-500 rounded-sm" />Eat {eat}</span>
				<span className="flex items-center gap-1"><span className="w-2 h-2 bg-amber-500 rounded-sm" />Repro {reproduce}</span>
				<span className="flex items-center gap-1"><span className="w-2 h-2 bg-slate-600 rounded-sm" />Noop {noop}</span>
			</div>
		</div>
	);
}
```

**Step 3: Create `MiniChart.tsx`**

```tsx
// frontend/src/components/stats/shared/MiniChart.tsx
import { useEffect, useRef } from "react";

export function MiniChart({
	data,
	color,
	height = 60,
}: { data: { tick: number; value: number }[]; color: string; height?: number }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas || data.length < 2) return;

		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		const w = canvas.width;
		const h = canvas.height;
		ctx.clearRect(0, 0, w, h);

		const values = data.map((d) => d.value);
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min || 1;

		ctx.strokeStyle = color;
		ctx.lineWidth = 1.5;
		ctx.beginPath();

		for (let i = 0; i < data.length; i++) {
			const x = (i / (data.length - 1)) * w;
			const y = h - ((values[i]! - min) / range) * (h - 4) - 2;
			if (i === 0) ctx.moveTo(x, y);
			else ctx.lineTo(x, y);
		}
		ctx.stroke();
	}, [data, color]);

	return (
		<canvas
			ref={canvasRef}
			width={300}
			height={height}
			className="w-full"
			style={{ height: `${height}px` }}
		/>
	);
}
```

**Step 4: Create the 4 tab components**

Extract each tab function into its own file under `frontend/src/components/stats/`. Each imports `Gauge`, `ActionBar`, `MiniChart` from `./shared/` and store hooks from `../../stores/`. Keep the exact same logic from `StatsPanel.tsx`.

- `OverviewTab.tsx` (from lines 125-162)
- `ActionsTab.tsx` (from lines 164-195)
- `EvolutionTab.tsx` (from lines 197-260)
- `ComputationTab.tsx` (from lines 262-306)

**Step 5: Slim down `StatsPanel.tsx`**

```tsx
// frontend/src/components/StatsPanel.tsx
import { useCallback, useState } from "react";
import { ActionsTab } from "./stats/ActionsTab.tsx";
import { ComputationTab } from "./stats/ComputationTab.tsx";
import { EvolutionTab } from "./stats/EvolutionTab.tsx";
import { OverviewTab } from "./stats/OverviewTab.tsx";

type Tab = "overview" | "actions" | "evolution" | "computation";

export function StatsPanel() {
	const [tab, setTab] = useState<Tab>("overview");

	const tabButton = useCallback(
		(t: Tab, label: string, testId: string) => (
			<button
				type="button"
				data-testid={testId}
				onClick={() => setTab(t)}
				className={`px-3 py-1 text-xs font-medium rounded-t ${
					tab === t ? "bg-petri-panel text-slate-200" : "text-slate-500 hover:text-slate-300"
				}`}
			>
				{label}
			</button>
		),
		[tab],
	);

	return (
		<div
			data-testid="stats-panel"
			className="bg-petri-panel border-t border-petri-border flex flex-col"
			style={{ height: "280px" }}
		>
			<div className="flex gap-1 px-3 pt-1 bg-slate-950">
				{tabButton("overview", "Overview", "stats-tab-overview")}
				{tabButton("actions", "Actions", "stats-tab-actions")}
				{tabButton("evolution", "Evolution", "stats-tab-evolution")}
				{tabButton("computation", "Computation", "stats-tab-computation")}
			</div>
			<div className="flex-1 overflow-y-auto">
				{tab === "overview" && <OverviewTab />}
				{tab === "actions" && <ActionsTab />}
				{tab === "evolution" && <EvolutionTab />}
				{tab === "computation" && <ComputationTab />}
			</div>
		</div>
	);
}
```

**Step 6: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 7: Commit**
```
refactor: split StatsPanel into individual tab and primitive components
```

- [x] Task 4 complete

---

### Task 5: Extract restart business logic to store

**Files:**
- Modify: `frontend/src/stores/startupConfig.ts`
- Modify: `frontend/src/components/ControlBar.tsx:129-156`

**Step 1: Add `buildStartupRequest()` to `useStartupConfigStore`**

Add this method to the store interface and implementation in `frontend/src/stores/startupConfig.ts`. Since it's a derived value from `preset`, add it as a standalone exported function (not a store action):

```ts
// Add at the bottom of startupConfig.ts, after the store creation

/** Build the startup API request from the current preset */
export function buildStartupRequest(preset: StartupPreset): {
	seed: number;
	population: { initial_creatures: number };
	world: {
		width: number;
		height: number;
		food: StartupPreset["world"]["food"];
	};
} {
	return {
		seed: preset.seed,
		population: { initial_creatures: preset.population.initial_creatures },
		world: {
			width: preset.world.width,
			height: preset.world.height,
			food: { ...preset.world.food },
		},
	};
}
```

**Step 2: Simplify `handleRestart` in `ControlBar.tsx`**

Replace the manual payload construction (lines 139-156) with:

```ts
import { buildStartupRequest, useStartupConfigStore } from "../stores/startupConfig.ts";

// In handleRestart:
const startup = useStartupConfigStore.getState().preset;
const res = await api.startup(buildStartupRequest(startup));
```

**Step 3: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 4: Commit**
```
refactor: extract startup request builder from ControlBar into store module
```

- [x] Task 5 complete

---

### Task 6: Clean startup/runtime config split

**Files:**
- Modify: `frontend/src/stores/startupConfig.ts` — add `initial_energy` to `StartupPreset`
- Modify: `frontend/src/components/config-panel/startup/StartupConfigPanel.tsx` — add Energy section
- Create: `frontend/src/components/config-panel/startup/EnergySection.tsx` — new startup section
- Modify: `frontend/src/components/config-panel/startup/FoodParametersSection.tsx` — remove non-startup fields
- Modify: `frontend/src/components/config-panel/runtime/EnergyLifecycleSection.tsx` — remove `initial_energy`
- Modify: `frontend/src/components/ConfigPanel.test.tsx` — update assertions

**Step 1: Write a failing test**

Add to `ConfigPanel.test.tsx`:

```tsx
it("initial_energy appears only in startup panel, not runtime", () => {
	render(<ConfigPanel />);
	// Should exist in startup
	expect(screen.getByTestId("startup-field-energy-initial-energy")).toBeInTheDocument();
	// Should not exist in runtime
	expect(screen.queryByTestId("runtime-field-energy-initial-energy")).not.toBeInTheDocument();
});

it("growth_rate appears only in runtime panel, not startup", () => {
	render(<ConfigPanel />);
	// Should not exist in startup
	expect(screen.queryByTestId("startup-field-food-growth-rate")).not.toBeInTheDocument();
});
```

**Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run -- --reporter verbose -t "initial_energy"`
Expected: FAIL — `startup-field-energy-initial-energy` not found (it's currently in runtime only), and `startup-field-food-growth-rate` exists (it's currently in startup).

**Step 3: Add `initial_energy` to `StartupPreset` type and defaults**

In `frontend/src/stores/startupConfig.ts`:

Add to `StartupPreset` interface:
```ts
export interface StartupPreset {
	seed: number;
	population: {
		initial_creatures: number;
	};
	world: {
		width: number;
		height: number;
		food: {
			initial_density: number;
			initial_coverage: number;
		};
	};
	energy: {
		initial_energy: number;
	};
}
```

Note: the `food` sub-object now only contains startup-only fields: `initial_density` and `initial_coverage`. The 5 runtime food fields (`growth_rate`, `spread_threshold_ratio`, `recovery_spawn_rate`, `recovery_floor_ratio`, `max_density`) are removed from `StartupPreset`.

Update `buildDefaultPreset()`:
```ts
function buildDefaultPreset(): StartupPreset {
	return {
		seed: randomSeed(),
		population: {
			initial_creatures: 2000,
		},
		world: {
			width: 400,
			height: 400,
			food: {
				initial_density: 1.0,
				initial_coverage: 0.15,
			},
		},
		energy: {
			initial_energy: 20.0,
		},
	};
}
```

Update `fromServerConfig()`:
```ts
function fromServerConfig(config: SimulationConfig): StartupPreset {
	return {
		seed: randomSeed(),
		population: {
			initial_creatures: config.population.initial_creatures,
		},
		world: {
			width: config.world.width,
			height: config.world.height,
			food: {
				initial_density: config.world.food.initial_density,
				initial_coverage: config.world.food.initial_coverage,
			},
		},
		energy: {
			initial_energy: config.energy.lifecycle.initial_energy,
		},
	};
}
```

Update `buildStartupRequest()`:
```ts
export function buildStartupRequest(preset: StartupPreset) {
	return {
		seed: preset.seed,
		population: { initial_creatures: preset.population.initial_creatures },
		world: {
			width: preset.world.width,
			height: preset.world.height,
			food: { ...preset.world.food },
		},
		energy: {
			lifecycle: {
				initial_energy: preset.energy.initial_energy,
			},
		},
	};
}
```

**Step 4: Update startup food section**

In `frontend/src/components/config-panel/startup/FoodParametersSection.tsx`, keep only:
- `world.food.initial_density`
- `world.food.initial_coverage`

Remove `growth_rate`, `spread_threshold_ratio`, `recovery_spawn_rate`, `recovery_floor_ratio`, `max_density` from the `FIELDS` array.

**Step 5: Create startup energy section**

```tsx
// frontend/src/components/config-panel/startup/EnergySection.tsx
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "energy.initial_energy",
		label: "Initial Energy",
		min: 0,
		max: 500,
		step: 0.5,
		testId: "startup-field-energy-initial-energy",
	},
];

export function EnergySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Energy">
			{FIELDS.map((field) => (
				<FieldRow
					key={`startup-${field.path}`}
					field={field}
					id={`startup-${field.path.replaceAll(".", "-")}`}
					value={getByPath(startupPreset, field.path) as number}
					disabled={false}
					onChange={updateStartupPreset}
					testId={field.testId}
				/>
			))}
		</FieldGroup>
	);
}
```

**Step 6: Add energy section to `StartupConfigPanel.tsx`**

```tsx
import { EnergySection } from "./EnergySection.tsx";

// In the JSX, add after WorldTopologySection and before FoodParametersSection:
<EnergySection startupPreset={startupPreset} updateStartupPreset={updateStartupPreset} />
```

**Step 7: Remove `initial_energy` from runtime `EnergyLifecycleSection.tsx`**

Remove the first entry from `ENERGY_LIFECYCLE_FIELDS` (the one with path `"energy.lifecycle.initial_energy"`).

**Step 8: Update test assertions**

In `ConfigPanel.test.tsx`, update the first test that checks for `startup-field-food-growth-rate`:
```tsx
// Remove this assertion since growth_rate is no longer in startup:
// expect(screen.getByTestId("startup-field-food-growth-rate")).toBeInTheDocument();

// The test "startup edits do not modify runtime local draft for shared food fields"
// needs to be updated to use a field that's still in startup (e.g., initial_density)
```

**Step 9: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass including the new split assertions.

**Step 10: Commit**
```
feat: clean startup/runtime config split — no field overlap
```

- [x] Task 6 complete

---

### Task 7: Per-field reset buttons

**Files:**
- Create: `frontend/src/components/config-panel/shared/defaults.ts`
- Modify: `frontend/src/components/config-panel/shared/types.ts` — add `defaultValue` to `FieldDef`
- Modify: `frontend/src/components/config-panel/shared/FieldRow.tsx` — add reset icon

**Step 1: Write a failing test**

Add to `ConfigPanel.test.tsx`:

```tsx
it("shows reset button when runtime field differs from default, hides when at default", () => {
	render(<ConfigPanel />);

	const moveCostInput = screen.getByTestId("config-field-energy-costs-move-cost");
	// MOCK_CONFIG has move_cost=0.02, default is 1.0 — should show reset
	const resetBtn = screen.getByTestId("reset-energy-costs-move-cost");
	expect(resetBtn).toBeInTheDocument();

	// Click reset — should set to default (1.0)
	fireEvent.click(resetBtn);
	expect(moveCostInput).toHaveValue(1);

	// Reset button should now be hidden (value equals default)
	expect(screen.queryByTestId("reset-energy-costs-move-cost")).not.toBeInTheDocument();
});
```

**Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run -- -t "shows reset button"`
Expected: FAIL — `reset-energy-costs-move-cost` not found.

**Step 3: Add `defaultValue` to `FieldDef`**

In `frontend/src/components/config-panel/shared/types.ts`:

```ts
export interface FieldDef {
	path: string;
	label: string;
	min: number;
	max: number;
	step: number;
	testId?: string;
	topologyField?: boolean;
	tooltip?: string;
	defaultValue?: number;
}
```

**Step 4: Create defaults map and add `defaultValue` to all field definitions**

Create `frontend/src/components/config-panel/shared/defaults.ts`:

```ts
/** Default values from SimulationConfig::default() on the Rust side */
export const RUNTIME_DEFAULTS: Record<string, number> = {
	"world.food.growth_rate": 0.096,
	"world.food.spread_threshold_ratio": 0.8,
	"world.food.recovery_spawn_rate": 0.01,
	"world.food.recovery_floor_ratio": 0.01,
	"world.food.max_density": 1.0,
	"population.max_creatures": 100000,
	"energy.lifecycle.max_energy": 200.0,
	"energy.lifecycle.energy_decay_per_tick": 0.5,
	"energy.lifecycle.min_reproduce_energy": 1.0,
	"energy.lifecycle.default_offspring_energy": 8.0,
	"energy.costs.move_cost": 1.0,
	"energy.costs.eat_cost": 0.0,
	"energy.costs.noop_cost": 0.05,
	"energy.costs.reproduce_cost": 0.1,
	"energy.costs.eat_reward_per_food": 12.0,
	"runtime.max_mesh_hops": 1024,
	"runtime.max_vm_steps": 10000,
	"runtime.max_graph_relax_iters": 15,
	"runtime.graph_convergence_epsilon": 0.001,
	"runtime.graph_convergence_stable_passes": 2,
	"runtime.graph_node_base_cost": 0.00001,
	"runtime.vm.opcode_cost_multiplier": 0.000001,
	"mutation.mutation_probability": 0.303,
	"mutation.per_birth_mutation_events_min": 1,
	"mutation.per_birth_mutation_events_max": 10,
	"mutation.phenotype.channel_step": 1,
	"mutation.phenotype.channel_change_chance": 0.001,
	"mutation.phenotype.polarity_flip_chance": 0.0002,
};

export const STARTUP_DEFAULTS: Record<string, number> = {
	"population.initial_creatures": 2000,
	"world.width": 400,
	"world.height": 400,
	"world.food.initial_density": 1.0,
	"world.food.initial_coverage": 0.15,
	"energy.initial_energy": 20.0,
};
```

Then add `defaultValue` to each field definition in all 6 runtime field constant arrays and the startup field arrays. For example in `EnergyCostsSection.tsx`:

```ts
{
	path: "energy.costs.move_cost",
	label: "Move Cost",
	min: 0,
	max: 10,
	step: 0.01,
	testId: "config-field-energy-costs-move-cost",
	defaultValue: 1.0,
},
```

**Step 5: Add reset button to `FieldRow`**

Update `frontend/src/components/config-panel/shared/FieldRow.tsx`:

```tsx
import type { FieldDef } from "./types.ts";

interface FieldRowProps {
	field: FieldDef;
	id: string;
	value: number;
	serverValue?: number;
	disabled: boolean;
	onChange: (path: string, value: number) => void;
	testId?: string;
}

export function FieldRow({
	field,
	id,
	value,
	serverValue,
	disabled,
	onChange,
	testId,
}: FieldRowProps) {
	const isDirty = serverValue !== undefined ? value !== serverValue : false;
	const inputId = `${id}-input`;
	const sliderAccentClass = id.startsWith("runtime-") ? "accent-sky-500" : "accent-emerald-500";
	const showReset = field.defaultValue !== undefined && value !== field.defaultValue;

	return (
		<div
			className={`flex flex-col gap-1.5 py-1.5 pl-2 border-l-2 ${isDirty ? "border-blue-500" : "border-transparent"}`}
		>
			<div className="flex items-center justify-between gap-2">
				<label htmlFor={inputId} className="flex items-center gap-1 text-xs text-slate-300">
					{disabled && (
						<span className="text-slate-500" title="Locked in current state">
							&#x1f512;
						</span>
					)}
					{field.label}
				</label>
				{showReset && (
					<button
						type="button"
						data-testid={`reset-${field.path.replaceAll(".", "-")}`}
						onClick={() => onChange(field.path, field.defaultValue!)}
						disabled={disabled}
						className="text-[10px] text-slate-500 hover:text-slate-300 disabled:opacity-40"
						title={`Reset to default (${field.defaultValue})`}
					>
						&#x21ba;
					</button>
				)}
			</div>
			<div className="flex items-center gap-2">
				<input
					id={inputId}
					data-testid={testId}
					type="number"
					disabled={disabled}
					value={value}
					min={field.min}
					max={field.max}
					step={field.step}
					onChange={(e) => onChange(field.path, Number(e.target.value))}
					className="w-24 px-1.5 py-0.5 text-xs font-mono text-right bg-slate-800 border border-slate-700 rounded text-slate-200 disabled:opacity-40"
				/>
				<input
					aria-label={`${field.label} slider`}
					type="range"
					disabled={disabled}
					value={value}
					min={field.min}
					max={field.max}
					step={field.step}
					onChange={(e) => onChange(field.path, Number(e.target.value))}
					className={`w-full h-1 ${sliderAccentClass} disabled:opacity-40`}
				/>
			</div>
		</div>
	);
}
```

**Step 6: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass including the new reset button test.

**Step 7: Commit**
```
feat: add per-field reset button that conditionally appears when value differs from default
```

- [x] Task 7 complete

---

### Task 8: Restart preserves runtime config

**Files:**
- Modify: `frontend/src/components/ControlBar.tsx`

**Step 1: Modify `handleRestart` to re-PATCH runtime config after startup**

In `frontend/src/components/ControlBar.tsx`, update `handleRestart`:

```ts
const handleRestart = useCallback(async () => {
	if (
		(simState === "running" || simState === "paused") &&
		!window.confirm("Restart the simulation with the current startup settings?")
	) {
		return;
	}

	setRestarting(true);
	try {
		const startup = useStartupConfigStore.getState().preset;
		const res = await api.startup(buildStartupRequest(startup));

		useSimulationStore.getState().setSimState(res.state);
		useSimulationStore.getState().setTick(res.tick);
		useStatsHistoryStore.getState().reset();

		// Re-apply current runtime config values so they survive restart
		const prevConfig = useConfigStore.getState().serverConfig;
		if (prevConfig) {
			const patch: Record<string, unknown> = {};
			for (const field of RUNTIME_PATCH_FIELDS) {
				mergePatch(patch, buildPatch(field.path, getByPath(prevConfig, field.path) as number));
			}
			await api.patchConfig(patch as DeepPartial<SimulationConfig>);
		}

		const configRes = await api.getConfig();
		useConfigStore.getState().commitServerConfig(configRes.config, configRes.state);
	} catch (e) {
		console.error("Restart failed:", e);
	} finally {
		setRestarting(false);
	}
}, [simState]);
```

This requires importing `RUNTIME_PATCH_FIELDS`, `buildPatch`, `getByPath`, `mergePatch`, and `DeepPartial` into `ControlBar.tsx`.

**Step 2: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 3: Run build**

Run: `cd frontend && npm run build`
Expected: No build errors.

**Step 4: Commit**
```
feat: restart preserves current runtime config values via re-PATCH
```

- [x] Task 8 complete

---

### Task 9: Field tooltips

**Files:**
- Create: `frontend/src/components/config-panel/shared/Tooltip.tsx`
- Modify: `frontend/src/components/config-panel/shared/FieldRow.tsx`
- Modify: all field definition arrays to add `tooltip` strings

**Step 1: Create `Tooltip` component**

```tsx
// frontend/src/components/config-panel/shared/Tooltip.tsx
import type { ReactNode } from "react";

interface TooltipProps {
	text: string;
	children: ReactNode;
}

export function Tooltip({ text, children }: TooltipProps) {
	return (
		<span className="relative group/tooltip inline-flex">
			{children}
			<span className="absolute left-1/2 -translate-x-1/2 bottom-full mb-1.5 px-2 py-1 text-[10px] leading-tight text-slate-200 bg-slate-700 rounded shadow-lg whitespace-normal max-w-48 pointer-events-none opacity-0 group-hover/tooltip:opacity-100 transition-opacity z-50">
				{text}
			</span>
		</span>
	);
}
```

**Step 2: Add tooltip to `FieldRow`**

In `FieldRow.tsx`, import `Tooltip` and add an info icon next to the label when `field.tooltip` exists:

```tsx
import { Tooltip } from "./Tooltip.tsx";

// Inside the label, after {field.label}:
{field.tooltip && (
	<Tooltip text={field.tooltip}>
		<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
	</Tooltip>
)}
```

**Step 3: Add tooltip text to all field definitions**

Add `tooltip` strings to field defs. Examples:

```ts
// Energy costs
{ path: "energy.costs.move_cost", label: "Move Cost", tooltip: "Energy subtracted each time a creature moves to an adjacent cell", ... }
{ path: "energy.costs.eat_cost", label: "Eat Cost", tooltip: "Energy subtracted when a creature eats food from its cell", ... }
{ path: "energy.costs.noop_cost", label: "Noop Cost", tooltip: "Energy subtracted when a creature takes no action", ... }
{ path: "energy.costs.reproduce_cost", label: "Reproduce Cost", tooltip: "Energy subtracted from parent when reproduction occurs", ... }
{ path: "energy.costs.eat_reward_per_food", label: "Eat Reward", tooltip: "Energy gained per unit of food density consumed", ... }

// Energy lifecycle
{ path: "energy.lifecycle.max_energy", label: "Max Energy", tooltip: "Upper bound on creature energy — excess is clamped", ... }
{ path: "energy.lifecycle.energy_decay_per_tick", label: "Decay / Tick", tooltip: "Energy lost by every creature each tick (maintenance cost)", ... }
{ path: "energy.lifecycle.min_reproduce_energy", label: "Min Reproduce Energy", tooltip: "Minimum energy required for a creature to reproduce", ... }
{ path: "energy.lifecycle.default_offspring_energy", label: "Offspring Energy", tooltip: "Starting energy given to newborn creatures", ... }

// Food
{ path: "world.food.growth_rate", label: "Food Growth Rate", tooltip: "Rate at which existing food cells regenerate density each tick", ... }
{ path: "world.food.spread_threshold_ratio", label: "Spread Threshold Ratio", tooltip: "Minimum neighbor density ratio to trigger food spread to empty cells", ... }
{ path: "world.food.recovery_spawn_rate", label: "Recovery Spawn Rate", tooltip: "Probability of spontaneous food spawn on empty cells each tick", ... }
{ path: "world.food.recovery_floor_ratio", label: "Recovery Floor Ratio", tooltip: "Minimum population-to-capacity ratio below which recovery spawning activates", ... }
{ path: "world.food.max_density", label: "Food Max Density", tooltip: "Maximum food density per cell (0-1 scale)", ... }

// Mutation
{ path: "mutation.mutation_probability", label: "Mutation Prob.", tooltip: "Probability that a newborn genome undergoes mutation", ... }
{ path: "mutation.per_birth_mutation_events_min", label: "Min Events/Birth", tooltip: "Minimum number of mutation events per birth when mutation triggers", ... }
{ path: "mutation.per_birth_mutation_events_max", label: "Max Events/Birth", tooltip: "Maximum number of mutation events per birth when mutation triggers", ... }
{ path: "mutation.phenotype.channel_step", label: "Channel Step", tooltip: "RGB step size applied per tick to the active color channel", ... }
{ path: "mutation.phenotype.polarity_flip_chance", label: "Polarity Flip", tooltip: "Probability of reversing the drift direction of the active color channel", ... }
{ path: "mutation.phenotype.channel_change_chance", label: "Channel Switch", tooltip: "Probability of switching to a different active color channel (R/G/B)", ... }

// Runtime
{ path: "runtime.max_mesh_hops", label: "Max Mesh Hops", tooltip: "Maximum signal propagation hops through the genome mesh per tick", ... }
{ path: "runtime.max_vm_steps", label: "Max VM Steps", tooltip: "Maximum VM instructions executed per creature per tick", ... }
{ path: "runtime.max_graph_relax_iters", label: "Graph Relax Iters", tooltip: "Maximum iterations for graph relaxation convergence", ... }
{ path: "runtime.graph_convergence_epsilon", label: "Convergence Epsilon", tooltip: "Threshold below which graph relaxation is considered converged", ... }
{ path: "runtime.graph_convergence_stable_passes", label: "Stable Passes", tooltip: "Consecutive stable passes required before declaring convergence", ... }
{ path: "runtime.graph_node_base_cost", label: "Node Base Cost", tooltip: "Base compute cost per graph node during evaluation", ... }
{ path: "runtime.vm.opcode_cost_multiplier", label: "Opcode Cost Mult.", tooltip: "Multiplier applied to each VM opcode's compute cost", ... }

// Population
{ path: "population.max_creatures", label: "Max Creatures", tooltip: "Hard cap on total population — reproduction blocked above this", ... }

// Startup fields
{ path: "population.initial_creatures", label: "Initial Creatures", tooltip: "Number of creatures spawned at world initialization", ... }
{ path: "world.width", label: "Width", tooltip: "World grid width in cells", ... }
{ path: "world.height", label: "Height", tooltip: "World grid height in cells", ... }
{ path: "world.food.initial_density", label: "Food Initial Density", tooltip: "Starting food density (0-1) for cells selected during world generation", ... }
{ path: "world.food.initial_coverage", label: "Food Coverage", tooltip: "Fraction of world cells that start with food during generation", ... }
{ path: "energy.initial_energy", label: "Initial Energy", tooltip: "Starting energy given to each founder creature at world initialization", ... }
```

**Step 4: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 5: Commit**
```
feat: add informative tooltips to all config fields
```

- [x] Task 9 complete

---

### Task 10: Zoom controls

**Files:**
- Create: `frontend/src/components/ZoomControls.tsx`
- Modify: `frontend/src/components/WorldViewport.tsx`

**Step 1: Create `ZoomControls` component**

```tsx
// frontend/src/components/ZoomControls.tsx

interface ZoomControlsProps {
	onZoomIn: () => void;
	onZoomOut: () => void;
	onFitToWorld: () => void;
}

export function ZoomControls({ onZoomIn, onZoomOut, onFitToWorld }: ZoomControlsProps) {
	const btnClass =
		"w-8 h-8 flex items-center justify-center text-sm text-slate-300 bg-slate-800/60 backdrop-blur-sm rounded hover:bg-slate-700/80 transition-colors";

	return (
		<div className="absolute top-3 right-3 flex flex-col gap-1 z-10">
			<button type="button" onClick={onZoomIn} className={btnClass} title="Zoom in">
				+
			</button>
			<button type="button" onClick={onZoomOut} className={btnClass} title="Zoom out">
				&minus;
			</button>
			<button type="button" onClick={onFitToWorld} className={btnClass} title="Fit to world (Home)">
				&#x2B1C;
			</button>
		</div>
	);
}
```

**Step 2: Wire into `WorldViewport`**

In `frontend/src/components/WorldViewport.tsx`, import and render `ZoomControls`:

```tsx
import { ZoomControls } from "./ZoomControls.tsx";

// Add zoom callbacks:
const handleZoomIn = useCallback(() => {
	const renderer = rendererRef.current;
	if (!renderer) return;
	const cx = renderer.canvas.width / 2;
	const cy = renderer.canvas.height / 2;
	renderer.zoomAt(cx + renderer.canvas.getBoundingClientRect().left, cy + renderer.canvas.getBoundingClientRect().top, -1);
}, []);

const handleZoomOut = useCallback(() => {
	const renderer = rendererRef.current;
	if (!renderer) return;
	const cx = renderer.canvas.width / 2;
	const cy = renderer.canvas.height / 2;
	renderer.zoomAt(cx + renderer.canvas.getBoundingClientRect().left, cy + renderer.canvas.getBoundingClientRect().top, 1);
}, []);

const handleFitToWorld = useCallback(() => {
	rendererRef.current?.resetView();
}, []);
```

Note: `zoomAt` expects client coordinates. We need to add canvas `getBoundingClientRect()` offsets. Alternatively, add a simpler `zoomCenter(delta)` method to `WorldRenderer` that zooms toward canvas center without needing client coords. The simpler approach:

Add to `WorldRenderer`:
```ts
zoomCenter(delta: number): void {
	const cx = this.canvas.width / 2;
	const cy = this.canvas.height / 2;
	const factor = delta > 0 ? 0.9 : 1.1;
	const newZoom = Math.max(0.5, Math.min(20, this.camera.zoom * factor));
	const ratio = newZoom / this.camera.zoom;
	this.camera.x = cx - (cx - this.camera.x) * ratio;
	this.camera.y = cy - (cy - this.camera.y) * ratio;
	this.camera.zoom = newZoom;
	this.invalidate();
}
```

Then the callbacks simplify to:
```tsx
const handleZoomIn = useCallback(() => {
	rendererRef.current?.zoomCenter(-1);
}, []);

const handleZoomOut = useCallback(() => {
	rendererRef.current?.zoomCenter(1);
}, []);
```

In the JSX, render `ZoomControls` inside the container div:

```tsx
return (
	<div ref={containerRef} className="relative w-full h-full overflow-hidden bg-petri-bg">
		<canvas ... />
		<ZoomControls
			onZoomIn={handleZoomIn}
			onZoomOut={handleZoomOut}
			onFitToWorld={handleFitToWorld}
		/>
	</div>
);
```

**Step 3: Run tests**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 4: Run build**

Run: `cd frontend && npm run build`
Expected: No build errors.

**Step 5: Commit**
```
feat: add translucent zoom in/out/fit buttons to canvas viewport
```

- [x] Task 10 complete

---

### Task 11: Final verification

**Step 1: Run full test suite**

Run: `cd frontend && npx vitest run`
Expected: All tests pass.

**Step 2: Run build**

Run: `cd frontend && npm run build`
Expected: Clean build, no errors.

**Step 3: Run type check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No type errors.

- [x] Task 11 complete

---

## Verification Commands

```bash
cd frontend && npx vitest run          # all tests pass
cd frontend && npm run build           # clean build
cd frontend && npx tsc --noEmit        # no type errors
```

## Risks / Rollback

- Component renames/splits may break test selectors → run tests after each task
- All changes frontend-only → easily reverted via git

**Review cycles:** 1

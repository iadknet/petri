import type { ReactNode } from "react";
import { useCallback, useState } from "react";
import type { DeepPartial } from "../api/rest.ts";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import type { SimulationConfig } from "../types/api.ts";

interface FieldDef {
	path: string;
	label: string;
	min: number;
	max: number;
	step: number;
	testId?: string;
	topologyField?: boolean;
}

const STARTUP_FIELD_GROUPS: { title: string; fields: FieldDef[] }[] = [
	{
		title: "World Topology",
		fields: [
			{
				path: "world.width",
				label: "Width",
				min: 10,
				max: 2000,
				step: 10,
				testId: "startup-field-world-width",
			},
			{
				path: "world.height",
				label: "Height",
				min: 10,
				max: 2000,
				step: 10,
				testId: "startup-field-world-height",
			},
		],
	},
	{
		title: "Food Parameters",
		fields: [
			{
				path: "world.food.growth_rate",
				label: "Food Growth Rate",
				min: 0,
				max: 1,
				step: 0.001,
				testId: "startup-field-food-growth-rate",
			},
			{
				path: "world.food.initial_density",
				label: "Food Initial Density",
				min: 0,
				max: 255,
				step: 1,
				testId: "startup-field-food-initial-density",
			},
			{
				path: "world.food.initial_coverage",
				label: "Food Coverage",
				min: 0,
				max: 1,
				step: 0.01,
				testId: "startup-field-food-initial-coverage",
			},
		],
	},
	{
		title: "Population",
		fields: [
			{
				path: "population.initial_creatures",
				label: "Initial Creatures",
				min: 1,
				max: 10000,
				step: 1,
				testId: "startup-field-population-initial-creatures",
			},
		],
	},
];

const RUNTIME_FIELD_GROUPS: { title: string; fields: FieldDef[] }[] = [
	{
		title: "Food Parameters",
		fields: [
			{
				path: "world.food.growth_rate",
				label: "Food Growth Rate",
				min: 0,
				max: 1,
				step: 0.001,
			},
			{
				path: "world.food.initial_density",
				label: "Food Initial Density",
				min: 0,
				max: 255,
				step: 1,
			},
			{
				path: "world.food.initial_coverage",
				label: "Food Coverage",
				min: 0,
				max: 1,
				step: 0.01,
			},
		],
	},
	{
		title: "Population",
		fields: [
			{
				path: "population.initial_creatures",
				label: "Initial Creatures",
				min: 1,
				max: 10000,
				step: 1,
			},
			{
				path: "population.max_creatures",
				label: "Max Creatures",
				min: 1,
				max: 100000,
				step: 100,
			},
		],
	},
	{
		title: "World Topology",
		fields: [
			{
				path: "world.width",
				label: "Width",
				min: 10,
				max: 2000,
				step: 10,
				topologyField: true,
				testId: "config-field-world-width",
			},
			{
				path: "world.height",
				label: "Height",
				min: 10,
				max: 2000,
				step: 10,
				topologyField: true,
			},
		],
	},
	{
		title: "Energy > Lifecycle",
		fields: [
			{
				path: "energy.lifecycle.initial_energy",
				label: "Initial Energy",
				min: 0,
				max: 500,
				step: 0.5,
			},
			{
				path: "energy.lifecycle.max_energy",
				label: "Max Energy",
				min: 1,
				max: 1000,
				step: 1,
			},
			{
				path: "energy.lifecycle.energy_decay_per_tick",
				label: "Decay / Tick",
				min: 0,
				max: 10,
				step: 0.01,
			},
			{
				path: "energy.lifecycle.min_reproduce_energy",
				label: "Min Reproduce Energy",
				min: 0,
				max: 500,
				step: 0.5,
			},
			{
				path: "energy.lifecycle.default_offspring_energy",
				label: "Offspring Energy",
				min: 0,
				max: 500,
				step: 0.5,
			},
		],
	},
	{
		title: "Energy > Costs",
		fields: [
			{
				path: "energy.costs.move_cost",
				label: "Move Cost",
				min: 0,
				max: 10,
				step: 0.01,
				testId: "config-field-energy-costs-move-cost",
			},
			{
				path: "energy.costs.eat_cost",
				label: "Eat Cost",
				min: 0,
				max: 10,
				step: 0.01,
			},
			{
				path: "energy.costs.noop_cost",
				label: "Noop Cost",
				min: 0,
				max: 10,
				step: 0.01,
			},
			{
				path: "energy.costs.reproduce_cost",
				label: "Reproduce Cost",
				min: 0,
				max: 50,
				step: 0.1,
			},
			{
				path: "energy.costs.eat_reward_per_food",
				label: "Eat Reward",
				min: 0,
				max: 50,
				step: 0.1,
			},
		],
	},
	{
		title: "Runtime",
		fields: [
			{
				path: "runtime.max_mesh_hops",
				label: "Max Mesh Hops",
				min: 1,
				max: 1024,
				step: 1,
			},
			{
				path: "runtime.max_vm_steps",
				label: "Max VM Steps",
				min: 1,
				max: 10000,
				step: 1,
			},
			{
				path: "runtime.max_graph_relax_iters",
				label: "Graph Relax Iters",
				min: 1,
				max: 100,
				step: 1,
			},
			{
				path: "runtime.graph_convergence_epsilon",
				label: "Convergence Epsilon",
				min: 0.0001,
				max: 1,
				step: 0.0001,
			},
			{
				path: "runtime.graph_convergence_stable_passes",
				label: "Stable Passes",
				min: 1,
				max: 10,
				step: 1,
			},
			{
				path: "runtime.graph_node_base_cost",
				label: "Node Base Cost",
				min: 0,
				max: 100,
				step: 0.1,
			},
			{
				path: "runtime.vm.opcode_cost_multiplier",
				label: "Opcode Cost Mult.",
				min: 0,
				max: 10,
				step: 0.1,
			},
		],
	},
	{
		title: "Mutation",
		fields: [
			{
				path: "runtime.mutation.mutation_probability",
				label: "Mutation Prob.",
				min: 0,
				max: 1,
				step: 0.001,
			},
			{
				path: "runtime.mutation.per_birth_mutation_events_min",
				label: "Min Events/Birth",
				min: 0,
				max: 20,
				step: 1,
			},
			{
				path: "runtime.mutation.per_birth_mutation_events_max",
				label: "Max Events/Birth",
				min: 0,
				max: 20,
				step: 1,
			},
			{
				path: "runtime.mutation.operator_modifier_scale",
				label: "Modifier Scale",
				min: 0,
				max: 10,
				step: 0.1,
			},
			{
				path: "runtime.mutation.phenotype.channel_step",
				label: "Channel Step",
				min: 1,
				max: 50,
				step: 1,
			},
			{
				path: "runtime.mutation.phenotype.polarity_flip_chance",
				label: "Polarity Flip",
				min: 0,
				max: 1,
				step: 0.001,
			},
			{
				path: "runtime.mutation.phenotype.channel_weight_min",
				label: "Weight Min",
				min: 0,
				max: 1,
				step: 0.01,
			},
			{
				path: "runtime.mutation.phenotype.channel_weight_max",
				label: "Weight Max",
				min: 0,
				max: 10,
				step: 0.01,
			},
		],
	},
];

function getByPath(obj: unknown, path: string): unknown {
	let current = obj;
	for (const key of path.split(".")) {
		if (current == null || typeof current !== "object") return undefined;
		current = (current as Record<string, unknown>)[key];
	}
	return current;
}

function buildPatch(path: string, value: number): Record<string, unknown> {
	const keys = path.split(".");
	const result: Record<string, unknown> = {};
	let current = result;
	for (let i = 0; i < keys.length - 1; i++) {
		const next: Record<string, unknown> = {};
		current[keys[i]!] = next;
		current = next;
	}
	current[keys[keys.length - 1]!] = value;
	return result;
}

function mergePatch(target: Record<string, unknown>, source: Record<string, unknown>): void {
	for (const [key, value] of Object.entries(source)) {
		if (value && typeof value === "object" && !Array.isArray(value)) {
			if (
				!target[key] ||
				typeof target[key] !== "object" ||
				target[key] === null ||
				Array.isArray(target[key])
			) {
				target[key] = {};
			}
			mergePatch(target[key] as Record<string, unknown>, value as Record<string, unknown>);
			continue;
		}
		target[key] = value;
	}
}

function FieldRow({
	field,
	id,
	value,
	serverValue,
	disabled,
	onChange,
	testId,
}: {
	field: FieldDef;
	id: string;
	value: number;
	serverValue?: number;
	disabled: boolean;
	onChange: (path: string, value: number) => void;
	testId?: string;
}) {
	const isDirty = serverValue !== undefined ? value !== serverValue : false;
	const inputId = `${id}-input`;

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
					className="w-full h-1 accent-emerald-500 disabled:opacity-40"
				/>
			</div>
		</div>
	);
}

function SeedRow({
	value,
	onChange,
	onRandomize,
}: {
	value: number;
	onChange: (value: number) => void;
	onRandomize: () => void;
}) {
	const inputId = "startup-seed-input";

	return (
		<div className="flex flex-col gap-1.5 py-1.5 pl-2 border-l-2 border-transparent">
			<div className="flex items-center justify-between gap-2">
				<label htmlFor={inputId} className="text-xs text-slate-300">
					Seed
				</label>
			</div>
			<div className="flex items-center gap-2">
				<input
					id={inputId}
					data-testid="startup-field-seed"
					type="number"
					value={value}
					onChange={(e) => onChange(Number(e.target.value))}
					className="flex-1 px-2 py-1 text-xs font-mono bg-slate-800 border border-slate-700 rounded text-slate-200"
				/>
				<button
					type="button"
					data-testid="startup-seed-randomize"
					onClick={onRandomize}
					className="px-2 py-1 text-[11px] text-slate-200 bg-slate-700 hover:bg-slate-600 rounded"
				>
					Randomize
				</button>
			</div>
		</div>
	);
}

function CollapsibleGroup({
	title,
	children,
	defaultOpen = true,
}: {
	title: string;
	children: ReactNode;
	defaultOpen?: boolean;
}) {
	const [open, setOpen] = useState(defaultOpen);
	return (
		<div className="border-b border-petri-border">
			<button
				type="button"
				onClick={() => setOpen(!open)}
				className="w-full flex items-center justify-between px-3 py-2 text-xs font-medium text-slate-300 hover:bg-slate-800/50"
			>
				{title}
				<span className={`transition-transform ${open ? "rotate-180" : ""}`}>&#x25B4;</span>
			</button>
			{open && <div className="px-3 pb-2 flex flex-col gap-0.5">{children}</div>}
		</div>
	);
}

function Section({
	title,
	description,
	children,
}: {
	title: string;
	description: string;
	children: ReactNode;
}) {
	return (
		<section className="border-b border-petri-border">
			<div className="px-3 pt-3 pb-2">
				<h3 className="text-xs font-semibold uppercase tracking-wide text-slate-200">{title}</h3>
				<p className="mt-1 text-[11px] text-slate-500">{description}</p>
			</div>
			{children}
		</section>
	);
}

export function ConfigPanel() {
	const localDraft = useConfigStore((s) => s.localDraft);
	const serverConfig = useConfigStore((s) => s.serverConfig);
	const isDirty = useConfigStore((s) => s.isDirty);
	const simState = useSimulationStore((s) => s.simState);
	const updateDraft = useConfigStore((s) => s.updateDraft);
	const resetDraft = useConfigStore((s) => s.resetDraft);
	const commitServerConfig = useConfigStore((s) => s.commitServerConfig);

	const startupPreset = useStartupConfigStore((s) => s.preset);
	const updateStartupPreset = useStartupConfigStore((s) => s.updatePreset);
	const randomizeSeed = useStartupConfigStore((s) => s.randomizeSeed);

	const [error, setError] = useState<string | null>(null);
	const [applying, setApplying] = useState(false);

	const handleApply = useCallback(async () => {
		if (!localDraft || !serverConfig) return;
		setApplying(true);
		setError(null);

		try {
			const patch: Record<string, unknown> = {};
			for (const group of RUNTIME_FIELD_GROUPS) {
				for (const field of group.fields) {
					const draft = getByPath(localDraft, field.path);
					const server = getByPath(serverConfig, field.path);
					if (draft !== server) {
						mergePatch(patch, buildPatch(field.path, draft as number));
					}
				}
			}

			const res = await api.patchConfig(patch as DeepPartial<SimulationConfig>);
			commitServerConfig(res.config, res.state);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Config update failed");
		} finally {
			setApplying(false);
		}
	}, [commitServerConfig, localDraft, serverConfig]);

	return (
		<aside
			data-testid="config-panel"
			className="w-80 bg-petri-panel border-r border-petri-border overflow-y-auto flex flex-col"
		>
			<div className="flex-1 overflow-y-auto">
				<Section
					title="Startup Config"
					description="These settings apply on Restart. Shared fields can diverge from live runtime values."
				>
					<CollapsibleGroup title="Run Settings">
						<SeedRow
							value={startupPreset.seed}
							onChange={(value) => updateStartupPreset("seed", value)}
							onRandomize={randomizeSeed}
						/>
					</CollapsibleGroup>
					{STARTUP_FIELD_GROUPS.map((group) => (
						<CollapsibleGroup key={`startup-${group.title}`} title={group.title}>
							{group.fields.map((field) => (
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
						</CollapsibleGroup>
					))}
				</Section>

				<Section
					title="Runtime (Live) Config"
					description="These settings apply to the current simulation when allowed by lifecycle state."
				>
					{!localDraft || !serverConfig ? (
						<div className="p-3 text-sm text-slate-500">
							Runtime config unavailable. Restart to initialize the simulation.
						</div>
					) : (
						RUNTIME_FIELD_GROUPS.map((group) => (
							<CollapsibleGroup key={`runtime-${group.title}`} title={group.title}>
								{group.fields.map((field) => {
									const disabled = field.topologyField
										? simState !== "idle"
										: simState === "running";
									return (
										<FieldRow
											key={`runtime-${field.path}`}
											field={field}
											id={`runtime-${field.path.replaceAll(".", "-")}`}
											value={getByPath(localDraft, field.path) as number}
											serverValue={getByPath(serverConfig, field.path) as number}
											disabled={disabled}
											onChange={updateDraft}
											testId={field.testId}
										/>
									);
								})}
							</CollapsibleGroup>
						))
					)}
				</Section>
			</div>

			{localDraft && serverConfig && (
				<div className="p-3 border-t border-petri-border flex flex-col gap-2">
					{error && <p className="text-xs text-red-400">{error}</p>}
					<div className="flex gap-2">
						<button
							type="button"
							data-testid="config-apply"
							disabled={!isDirty || applying}
							onClick={handleApply}
							className="flex-1 px-3 py-1.5 text-xs font-medium bg-emerald-600 text-white rounded hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed"
						>
							{applying ? "Applying..." : "Apply Changes"}
						</button>
						<button
							type="button"
							data-testid="config-reset"
							disabled={!isDirty}
							onClick={resetDraft}
							className="px-3 py-1.5 text-xs text-slate-400 bg-slate-800 rounded hover:bg-slate-700 disabled:opacity-40"
						>
							Reset
						</button>
					</div>
				</div>
			)}
		</aside>
	);
}

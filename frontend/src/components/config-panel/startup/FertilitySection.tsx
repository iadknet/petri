import type {
	FertilityAlgorithm,
	FertilityLayer,
	FoodFertilityLayerTarget,
} from "../../../types/config.ts";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { Tooltip } from "../shared/Tooltip.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, StartupSectionProps } from "../shared/types.ts";

const FERTILITY_TOGGLE: BooleanFieldDef = {
	path: "world.food.fertility.enabled",
	label: "Enable Fertility Layer",
	testId: "startup-field-fertility-enabled",
	defaultValue: true,
	tooltip: "When enabled, spatial fertility multipliers affect food growth rates across the world",
};

const FERTILITY_FIELDS: FieldDef[] = [
	{
		path: "world.food.fertility.min_fertility",
		label: "Min Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-fertility-min",
		defaultValue: 0.0,
		tooltip: "Minimum growth multiplier for barren zones",
	},
	{
		path: "world.food.fertility.max_fertility",
		label: "Max Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-fertility-max",
		defaultValue: 2.0,
		tooltip: "Maximum growth multiplier for fertile zones",
	},
];

const ANNEALING_TOGGLE: BooleanFieldDef = {
	path: "world.food.annealing.enabled",
	label: "Enable Warmup Annealing",
	testId: "startup-field-annealing-enabled",
	defaultValue: false,
	tooltip:
		"When enabled, fertility severity ramps gradually from friendly initial values to target values",
};

const ANNEALING_FIELDS: FieldDef[] = [
	{
		path: "world.food.annealing.ramp_ticks",
		label: "Ramp Ticks",
		min: 100,
		max: 50000,
		step: 100,
		testId: "startup-field-annealing-ramp-ticks",
		defaultValue: 5000,
		tooltip: "Ticks over which fertility severity ramps from initial to target values",
	},
	{
		path: "world.food.annealing.initial_min_fertility",
		label: "Initial Min Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-annealing-initial-min",
		defaultValue: 0.3,
		tooltip: "Starting minimum fertility (friendlier for founders)",
	},
	{
		path: "world.food.annealing.initial_max_fertility",
		label: "Initial Max Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-annealing-initial-max",
		defaultValue: 1.5,
		tooltip: "Starting maximum fertility (less extreme early)",
	},
];

type FertilityAlgorithmType = "Uniform" | "Fbm" | "PoissonBlobs";

const FBM_TOOLTIPS = {
	octaves:
		"How many detail layers are stacked together for richer texture. More octaves means more tiny detail.",
	frequency:
		"How zoomed in the pattern is. Higher makes smaller, busier patches. Lower makes larger zones.",
	lacunarity:
		"How much smaller each next detail layer gets. Higher values add finer little details.",
	persistence:
		"How strong those smaller detail layers stay. Lower fades them out faster, higher keeps them visible.",
	useSeed:
		"Turn this on to lock the random pattern so it repeats exactly when you use the same seed.",
	seed: "A number that picks the pattern. Same seed gives the same pattern; different seed gives a different one.",
} as const;

const UNIFORM_TOOLTIPS = {
	value:
		"One single flat fertility value everywhere. Higher boosts growth everywhere, lower slows growth everywhere.",
} as const;

const POISSON_TOOLTIPS = {
	blobCount: "How many fertility islands to drop on the map.",
	minRadius: "Smallest island size allowed.",
	maxRadius: "Largest island size allowed.",
	falloff: "How softly each island fades at the edges. Higher means gentler blending.",
	useSeed:
		"Turn this on to lock island placement so it repeats exactly when you use the same seed.",
	seed: "A number that picks island placement. Same seed gives the same island layout.",
} as const;

function defaultAlgorithm(type: FertilityAlgorithmType): FertilityAlgorithm {
	switch (type) {
		case "Uniform":
			return { Uniform: { value: 0.0 } };
		case "Fbm":
			return {
				Fbm: {
					octaves: 4,
					frequency: 0.04,
					lacunarity: 2.0,
					persistence: 0.5,
				},
			};
		case "PoissonBlobs":
			return {
				PoissonBlobs: {
					blob_count: 900,
					min_radius: 5.0,
					max_radius: 15.0,
					falloff: 0.5,
				},
			};
		default:
			return { Uniform: { value: 0.0 } };
	}
}

function targetValue(target: FoodFertilityLayerTarget | undefined): string {
	if (!target || target === "AllFoods") {
		return "all_foods";
	}
	return `single_type:${target.SingleType.type_idx}`;
}

function parseTarget(value: string): FoodFertilityLayerTarget {
	if (value === "all_foods") {
		return "AllFoods";
	}

	const [kind, idx] = value.split(":");
	if (kind === "single_type") {
		const parsed = Number(idx);
		if (Number.isFinite(parsed) && parsed >= 0) {
			return { SingleType: { type_idx: parsed } };
		}
	}

	return "AllFoods";
}

function algorithmType(algorithm: FertilityAlgorithm): FertilityAlgorithmType {
	if ("Uniform" in algorithm) return "Uniform";
	if ("Fbm" in algorithm) return "Fbm";
	return "PoissonBlobs";
}

interface LayerFieldProps {
	label: string;
	testId: string;
	value: number;
	onChange: (value: number) => void;
	tooltip?: string;
	min?: number;
	max?: number;
	step?: number;
}

function LayerField({ label, testId, value, onChange, tooltip, min, max, step }: LayerFieldProps) {
	return (
		<div className="flex items-center justify-between gap-2 py-0.5 pl-2">
			<label
				htmlFor={`${testId}-input`}
				className="flex items-center gap-1 text-[11px] text-slate-400"
			>
				{label}
				{tooltip && (
					<Tooltip text={tooltip}>
						<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
					</Tooltip>
				)}
			</label>
			<input
				id={`${testId}-input`}
				data-testid={testId}
				type="number"
				value={value}
				min={min}
				max={max}
				step={step}
				onChange={(e) => onChange(Number(e.target.value))}
				className="w-28 px-1.5 py-0.5 text-xs font-mono text-right bg-slate-800 border border-slate-700 rounded text-slate-200"
			/>
		</div>
	);
}

interface FertilitySectionProps extends StartupSectionProps {
	addFertilityLayer: () => void;
	updateFertilityLayerTarget: (index: number, target: FoodFertilityLayerTarget) => void;
}

export function FertilitySection({
	startupPreset,
	updateStartupPreset,
	addFertilityLayer,
	updateFertilityLayerTarget,
}: FertilitySectionProps) {
	const fertilityEnabled = (getByPath(startupPreset, FERTILITY_TOGGLE.path) as boolean) ?? false;
	const annealingEnabled = (getByPath(startupPreset, ANNEALING_TOGGLE.path) as boolean) ?? false;
	const fertilityLayers = startupPreset.world.food.fertility.layers;

	const setLayers = (layers: FertilityLayer[]) => {
		updateStartupPreset("world.food.fertility.layers", layers);
	};

	const updateLayer = (index: number, updater: (layer: FertilityLayer) => FertilityLayer) => {
		const next = fertilityLayers.map((layer, currentIndex) =>
			currentIndex === index ? updater(layer) : layer,
		);
		setLayers(next);
	};

	const handleFertilityToggle = (_path: string, enabled: boolean) => {
		updateStartupPreset(FERTILITY_TOGGLE.path, enabled);
		if (enabled && fertilityLayers.length === 0) {
			addFertilityLayer();
			addFertilityLayer();
		}
	};

	const addLayer = () => {
		addFertilityLayer();
	};

	const removeLayer = (index: number) => {
		setLayers(fertilityLayers.filter((_, currentIndex) => currentIndex !== index));
	};

	return (
		<>
			<FieldGroup title="Fertility Layer">
				<ToggleRow
					field={FERTILITY_TOGGLE}
					id={`startup-${FERTILITY_TOGGLE.path.replaceAll(".", "-")}`}
					value={fertilityEnabled}
					disabled={false}
					onChange={handleFertilityToggle}
					testId={FERTILITY_TOGGLE.testId}
				/>
				{fertilityEnabled &&
					FERTILITY_FIELDS.map((field) => (
						<FieldRow
							key={`startup-${field.path}`}
							field={field}
							id={`startup-${field.path.replaceAll(".", "-")}`}
							value={
								(getByPath(startupPreset, field.path) as number | undefined) ??
								field.defaultValue ??
								0
							}
							disabled={false}
							onChange={updateStartupPreset}
							testId={field.testId}
						/>
					))}
				{fertilityEnabled && (
					<div className="mt-1 border border-slate-800 rounded bg-slate-900/30 p-2 flex flex-col gap-2">
						<div className="flex items-center justify-between">
							<p className="text-[11px] font-medium text-slate-300">Fertility Layers</p>
							<button
								type="button"
								data-testid="startup-fertility-add-layer"
								onClick={addLayer}
								className="px-2 py-0.5 text-[11px] text-slate-200 bg-slate-700 hover:bg-slate-600 rounded"
							>
								Add Layer
							</button>
						</div>
						{fertilityLayers.length === 0 && (
							<p className="text-[11px] text-slate-500">
								No explicit layers configured. Add one to override backend defaults.
							</p>
						)}
						{fertilityLayers.map((layer, index) => {
							const layerAlgorithmType = algorithmType(layer.algorithm);
							const baseTestId = `startup-field-fertility-layer-${index}`;
							const fbm = "Fbm" in layer.algorithm ? layer.algorithm.Fbm : null;
							const poisson =
								"PoissonBlobs" in layer.algorithm ? layer.algorithm.PoissonBlobs : null;
							return (
								<div
									key={`fertility-layer-${index}`}
									className="border border-slate-800 rounded p-2 bg-slate-900/40"
								>
									<div className="flex items-center justify-between gap-2">
										<p className="text-[11px] uppercase tracking-wide text-slate-400">
											Layer {index + 1}
										</p>
										<button
											type="button"
											data-testid={`startup-fertility-remove-layer-${index}`}
											onClick={() => removeLayer(index)}
											className="px-2 py-0.5 text-[11px] text-rose-200 bg-rose-900/30 hover:bg-rose-900/50 rounded"
										>
											Remove
										</button>
									</div>

									<div className="mt-2 flex items-center justify-between gap-2">
										<label
											htmlFor={`${baseTestId}-target-input`}
											className="text-[11px] text-slate-400"
										>
											Target
										</label>
										<select
											id={`${baseTestId}-target-input`}
											data-testid={`${baseTestId}-target`}
											value={targetValue(layer.target)}
											onChange={(e) =>
												updateFertilityLayerTarget(index, parseTarget(e.target.value))
											}
											className="px-1.5 py-0.5 text-xs bg-slate-800 border border-slate-700 rounded text-slate-200"
										>
											<option value="all_foods">All Foods</option>
											{startupPreset.world.food.types.map((foodType, typeIndex) => (
												<option
													key={`${baseTestId}-target-${typeIndex}`}
													value={`single_type:${typeIndex}`}
												>
													{`Type ${typeIndex + 1}: ${foodType.name}`}
												</option>
											))}
										</select>
									</div>

									<div className="mt-2 flex items-center justify-between gap-2">
										<label
											htmlFor={`${baseTestId}-algorithm-input`}
											className="text-[11px] text-slate-400"
										>
											Algorithm
										</label>
										<select
											id={`${baseTestId}-algorithm-input`}
											data-testid={`${baseTestId}-algorithm`}
											value={layerAlgorithmType}
											onChange={(e) => {
												updateLayer(index, (currentLayer) => ({
													...currentLayer,
													algorithm: defaultAlgorithm(e.target.value as FertilityAlgorithmType),
												}));
											}}
											className="px-1.5 py-0.5 text-xs bg-slate-800 border border-slate-700 rounded text-slate-200"
										>
											<option value="Uniform">Uniform</option>
											<option value="Fbm">FBM</option>
											<option value="PoissonBlobs">Poisson Blobs</option>
										</select>
									</div>

									<LayerField
										label="Weight"
										testId={`${baseTestId}-weight`}
										value={layer.weight}
										min={0}
										max={5}
										step={0.05}
										onChange={(value) =>
											updateLayer(index, (currentLayer) => ({ ...currentLayer, weight: value }))
										}
									/>

									{layerAlgorithmType === "Uniform" && "Uniform" in layer.algorithm && (
										<LayerField
											label="Value"
											testId={`${baseTestId}-uniform-value`}
											value={layer.algorithm.Uniform.value}
											tooltip={UNIFORM_TOOLTIPS.value}
											min={-1}
											max={1}
											step={0.05}
											onChange={(value) =>
												updateLayer(index, (currentLayer) => ({
													...currentLayer,
													algorithm: { Uniform: { value } },
												}))
											}
										/>
									)}

									{layerAlgorithmType === "Fbm" && fbm && (
										<>
											<LayerField
												label="Octaves"
												testId={`${baseTestId}-fbm-octaves`}
												value={fbm.octaves}
												tooltip={FBM_TOOLTIPS.octaves}
												min={1}
												max={12}
												step={1}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															Fbm: {
																...fbm,
																octaves: Math.round(value),
															},
														},
													}))
												}
											/>
											<LayerField
												label="Frequency"
												testId={`${baseTestId}-fbm-frequency`}
												value={fbm.frequency}
												tooltip={FBM_TOOLTIPS.frequency}
												min={0.001}
												max={1}
												step={0.001}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: { Fbm: { ...fbm, frequency: value } },
													}))
												}
											/>
											<LayerField
												label="Lacunarity"
												testId={`${baseTestId}-fbm-lacunarity`}
												value={fbm.lacunarity}
												tooltip={FBM_TOOLTIPS.lacunarity}
												min={0.5}
												max={4}
												step={0.05}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															Fbm: { ...fbm, lacunarity: value },
														},
													}))
												}
											/>
											<LayerField
												label="Persistence"
												testId={`${baseTestId}-fbm-persistence`}
												value={fbm.persistence}
												tooltip={FBM_TOOLTIPS.persistence}
												min={0}
												max={1}
												step={0.01}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															Fbm: { ...fbm, persistence: value },
														},
													}))
												}
											/>
											<div className="flex items-center justify-between gap-2 py-0.5 pl-2">
												<label
													htmlFor={`${baseTestId}-fbm-seed-enabled-input`}
													className="flex items-center gap-1 text-[11px] text-slate-400"
												>
													Use Seed
													<Tooltip text={FBM_TOOLTIPS.useSeed}>
														<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
													</Tooltip>
												</label>
												<input
													id={`${baseTestId}-fbm-seed-enabled-input`}
													data-testid={`${baseTestId}-fbm-seed-enabled`}
													type="checkbox"
													checked={fbm.seed != null}
													onChange={(e) => {
														updateLayer(index, () => ({
															...layer,
															algorithm: {
																Fbm: e.target.checked
																	? {
																			...fbm,
																			seed: fbm.seed ?? 0,
																		}
																	: {
																			octaves: fbm.octaves,
																			frequency: fbm.frequency,
																			lacunarity: fbm.lacunarity,
																			persistence: fbm.persistence,
																		},
															},
														}));
													}}
													className="accent-emerald-500"
												/>
											</div>
											{fbm.seed != null && (
												<LayerField
													label="Seed"
													testId={`${baseTestId}-fbm-seed`}
													value={fbm.seed ?? 0}
													tooltip={FBM_TOOLTIPS.seed}
													min={0}
													step={1}
													onChange={(value) =>
														updateLayer(index, () => ({
															...layer,
															algorithm: {
																Fbm: {
																	...fbm,
																	seed: Math.max(0, Math.round(value)),
																},
															},
														}))
													}
												/>
											)}
										</>
									)}

									{layerAlgorithmType === "PoissonBlobs" && poisson && (
										<>
											<LayerField
												label="Blob Count"
												testId={`${baseTestId}-poisson-blob-count`}
												value={poisson.blob_count}
												tooltip={POISSON_TOOLTIPS.blobCount}
												min={1}
												max={5000}
												step={1}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															PoissonBlobs: {
																...poisson,
																blob_count: Math.max(1, Math.round(value)),
															},
														},
													}))
												}
											/>
											<LayerField
												label="Min Radius"
												testId={`${baseTestId}-poisson-min-radius`}
												value={poisson.min_radius}
												tooltip={POISSON_TOOLTIPS.minRadius}
												min={0.1}
												max={100}
												step={0.1}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															PoissonBlobs: {
																...poisson,
																min_radius: value,
															},
														},
													}))
												}
											/>
											<LayerField
												label="Max Radius"
												testId={`${baseTestId}-poisson-max-radius`}
												value={poisson.max_radius}
												tooltip={POISSON_TOOLTIPS.maxRadius}
												min={0.1}
												max={100}
												step={0.1}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															PoissonBlobs: {
																...poisson,
																max_radius: value,
															},
														},
													}))
												}
											/>
											<LayerField
												label="Falloff"
												testId={`${baseTestId}-poisson-falloff`}
												value={poisson.falloff}
												tooltip={POISSON_TOOLTIPS.falloff}
												min={0.01}
												max={10}
												step={0.01}
												onChange={(value) =>
													updateLayer(index, () => ({
														...layer,
														algorithm: {
															PoissonBlobs: {
																...poisson,
																falloff: value,
															},
														},
													}))
												}
											/>
											<div className="flex items-center justify-between gap-2 py-0.5 pl-2">
												<label
													htmlFor={`${baseTestId}-poisson-seed-enabled-input`}
													className="flex items-center gap-1 text-[11px] text-slate-400"
												>
													Use Seed
													<Tooltip text={POISSON_TOOLTIPS.useSeed}>
														<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
													</Tooltip>
												</label>
												<input
													id={`${baseTestId}-poisson-seed-enabled-input`}
													data-testid={`${baseTestId}-poisson-seed-enabled`}
													type="checkbox"
													checked={poisson.seed != null}
													onChange={(e) => {
														updateLayer(index, () => ({
															...layer,
															algorithm: {
																PoissonBlobs: e.target.checked
																	? {
																			...poisson,
																			seed: poisson.seed ?? 0,
																		}
																	: {
																			blob_count: poisson.blob_count,
																			min_radius: poisson.min_radius,
																			max_radius: poisson.max_radius,
																			falloff: poisson.falloff,
																		},
															},
														}));
													}}
													className="accent-emerald-500"
												/>
											</div>
											{poisson.seed != null && (
												<LayerField
													label="Seed"
													testId={`${baseTestId}-poisson-seed`}
													value={poisson.seed ?? 0}
													tooltip={POISSON_TOOLTIPS.seed}
													min={0}
													step={1}
													onChange={(value) =>
														updateLayer(index, () => ({
															...layer,
															algorithm: {
																PoissonBlobs: {
																	...poisson,
																	seed: Math.max(0, Math.round(value)),
																},
															},
														}))
													}
												/>
											)}
										</>
									)}
								</div>
							);
						})}
					</div>
				)}
			</FieldGroup>
			{fertilityEnabled && (
				<FieldGroup title="Warmup Annealing">
					<ToggleRow
						field={ANNEALING_TOGGLE}
						id={`startup-${ANNEALING_TOGGLE.path.replaceAll(".", "-")}`}
						value={annealingEnabled}
						disabled={false}
						onChange={updateStartupPreset}
						testId={ANNEALING_TOGGLE.testId}
					/>
					{annealingEnabled &&
						ANNEALING_FIELDS.map((field) => (
							<FieldRow
								key={`startup-${field.path}`}
								field={field}
								id={`startup-${field.path.replaceAll(".", "-")}`}
								value={
									(getByPath(startupPreset, field.path) as number | undefined) ??
									field.defaultValue ??
									0
								}
								disabled={false}
								onChange={updateStartupPreset}
								testId={field.testId}
							/>
						))}
				</FieldGroup>
			)}
		</>
	);
}

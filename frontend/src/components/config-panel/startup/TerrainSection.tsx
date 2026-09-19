import { randomSeed } from "../../../stores/startupConfig.ts";
import type { TerrainLayer } from "../../../types/config.ts";
import { DEFAULT_PATTERN_PARAMS, type PatternType } from "../../../types/pattern.ts";
import { PatternParamsPanel } from "../../pattern-params/PatternParamsPanel.tsx";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { RandomSeedButton } from "../shared/RandomSeedButton.tsx";
import type { StartupSectionProps } from "../shared/types.ts";

const inputClass =
	"min-w-0 w-full px-2 py-1 text-xs bg-slate-800 border border-slate-700 rounded text-slate-200";
function OptionalSeed({
	label,
	testId,
	value,
	onChange,
}: {
	label: string;
	testId: string;
	value: number | null | undefined;
	onChange: (value: number | null) => void;
}) {
	return (
		<div className="flex flex-col gap-1 text-xs text-slate-300">
			<label htmlFor={`${testId}-input`}>{label}</label>
			<div className="flex items-center gap-1">
				<input
					id={`${testId}-input`}
					aria-label={label}
					type="number"
					min={0}
					max={Number.MAX_SAFE_INTEGER}
					step={1}
					value={value ?? ""}
					placeholder="Use derived seed"
					className={inputClass}
					onChange={(event) => {
						const raw = event.target.value;
						const seed = Number(raw);
						if (raw === "") onChange(null);
						else if (Number.isSafeInteger(seed) && seed >= 0) onChange(seed);
					}}
				/>
				<RandomSeedButton
					label={label}
					testId={`${testId}-randomize`}
					onClick={() => onChange(randomSeed())}
				/>
			</div>
		</div>
	);
}
export function TerrainSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	const layers = startupPreset.world.terrain;
	const updateLayer = (index: number, patch: Partial<TerrainLayer>) =>
		updateStartupPreset(
			"world.terrain",
			layers.map((layer, i) => (i === index ? { ...layer, ...patch } : layer)),
		);
	return (
		<FieldGroup title="Terrain">
			<p className="text-xs text-slate-400 py-1">
				Barriers are added before food and founders. Layers combine in order.
			</p>
			<OptionalSeed
				label="Map seed"
				testId="startup-map-seed"
				value={startupPreset.world.world_seed}
				onChange={(seed) => updateStartupPreset("world.world_seed", seed)}
			/>
			<p className="text-xs text-slate-400">
				Fixes terrain and fertility. Blank uses the run seed; food placement and founders still use
				the run seed. Seeds: 0–9,007,199,254,740,991.
			</p>
			{layers.map((layer, index) => (
				<div key={index} className="border border-slate-700 rounded p-2 my-1 flex flex-col gap-2">
					<label className="text-xs text-slate-300">
						Layer {index + 1} pattern
						<select
							aria-label={`Layer ${index + 1} pattern`}
							className={inputClass}
							value={layer.params.pattern_type}
							onChange={(e) =>
								updateLayer(index, {
									params: structuredClone(DEFAULT_PATTERN_PARAMS[e.target.value as PatternType]),
								})
							}
						>
							{Object.keys(DEFAULT_PATTERN_PARAMS).map((pattern) => (
								<option key={pattern} value={pattern}>
									{pattern}
								</option>
							))}
						</select>
					</label>
					<PatternParamsPanel
						params={layer.params}
						onChange={(params) => updateLayer(index, { params })}
					/>
					<OptionalSeed
						label={`Layer ${index + 1} seed`}
						testId={`startup-terrain-layer-${index}-seed`}
						value={layer.seed}
						onChange={(seed) => updateLayer(index, { seed })}
					/>
					<p className="text-xs text-slate-400">Blank uses the map seed plus {index}.</p>
					<label className="text-xs text-slate-300 flex gap-2">
						<input
							type="checkbox"
							checked={layer.bounds != null}
							onChange={(e) =>
								updateLayer(index, {
									bounds: e.target.checked
										? {
												x: 0,
												y: 0,
												width: startupPreset.world.width,
												height: startupPreset.world.height,
											}
										: null,
								})
							}
						/>
						Layer {index + 1} explicit bounds
					</label>
					{layer.bounds ? (
						<div className="grid grid-cols-2 gap-2">
							{(["x", "y", "width", "height"] as const).map((key) => (
								<label key={key} className="text-xs text-slate-300">
									{key}
									<input
										aria-label={`Layer ${index + 1} ${key}`}
										type="number"
										min={0}
										max={65535}
										step={1}
										className={inputClass}
										value={layer.bounds?.[key]}
										onChange={(e) => {
											const value = Number(e.target.value);
											if (Number.isInteger(value) && value >= 0 && value <= 65535 && layer.bounds)
												updateLayer(index, { bounds: { ...layer.bounds, [key]: value } });
										}}
									/>
								</label>
							))}
						</div>
					) : (
						<p className="text-xs text-slate-400">Whole world</p>
					)}
					<button
						type="button"
						className="text-xs text-slate-200 bg-slate-700 rounded px-2 py-1 hover:bg-slate-600"
						onClick={() =>
							updateStartupPreset(
								"world.terrain",
								layers.filter((_, i) => i !== index),
							)
						}
					>
						Remove terrain layer {index + 1}
					</button>
				</div>
			))}
			<button
				type="button"
				className="text-xs text-slate-200 bg-slate-700 rounded px-2 py-1 my-2 hover:bg-slate-600"
				onClick={() =>
					updateStartupPreset("world.terrain", [
						...layers,
						{ params: structuredClone(DEFAULT_PATTERN_PARAMS.Maze), bounds: null, seed: null },
					])
				}
			>
				Add terrain layer
			</button>
		</FieldGroup>
	);
}

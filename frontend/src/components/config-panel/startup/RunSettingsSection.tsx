import { FieldGroup } from "../shared/FieldGroup.tsx";

interface RunSettingsSectionProps {
	seed: number;
	updateSeed: (value: number) => void;
	randomizeSeed: () => void;
}

interface SeedRowProps {
	value: number;
	onChange: (value: number) => void;
	onRandomize: () => void;
}

function SeedRow({ value, onChange, onRandomize }: SeedRowProps) {
	const inputId = "startup-seed-input";

	return (
		<div className="flex flex-col gap-1.5 py-1.5 pl-2 border-l-2 border-transparent">
			<div className="flex items-center justify-between gap-2">
				<label htmlFor={inputId} className="text-xs text-slate-300">
					Run seed
				</label>
			</div>
			<div className="flex items-center gap-2">
				<input
					id={inputId}
					data-testid="startup-field-seed"
					type="number"
					min={0}
					max={Number.MAX_SAFE_INTEGER}
					step={1}
					title="Controls food placement, founders, and runtime draws; also the map when map seed is blank."
					value={value}
					onChange={(e) => {
						const seed = Number(e.target.value);
						if (Number.isSafeInteger(seed) && seed >= 0) onChange(seed);
					}}
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

export function RunSettingsSection({ seed, updateSeed, randomizeSeed }: RunSettingsSectionProps) {
	return (
		<FieldGroup title="Run Settings">
			<SeedRow value={seed} onChange={updateSeed} onRandomize={randomizeSeed} />
		</FieldGroup>
	);
}

import { useConfigStore } from "../../../stores/config.ts";
import type { FoodTypeConfig } from "../../../types/config.ts";
import { FieldRow, NUMBER_INPUT_CLASS } from "../shared/FieldRow.tsx";
import { NumberInput } from "../shared/NumberInput.tsx";
import { formatConfigNumber } from "../shared/numberFormat.ts";
import type { FieldDef } from "../shared/types.ts";

interface FoodTypeCardProps {
	index: number;
	foodType: FoodTypeConfig;
	sharedGrowthRate?: number;
	sharedRecoveryRate?: number;
	canRemove: boolean;
	onRemove: () => void;
	onChange: (patch: Partial<FoodTypeConfig>) => void;
}

function TextControl({
	label,
	testId,
	value,
	onChange,
	type = "text",
}: {
	label: string;
	testId: string;
	value: string;
	onChange: (next: string) => void;
	type?: "text" | "color";
}) {
	return (
		<div className="flex items-center justify-between gap-2 py-0.5 pl-2">
			<label htmlFor={`${testId}-input`} className="text-[11px] text-slate-400">
				{label}
			</label>
			<input
				id={`${testId}-input`}
				data-testid={testId}
				type={type}
				value={value}
				onChange={(e) => onChange(e.target.value)}
				className={`w-32 px-1.5 py-0.5 text-xs bg-slate-800 border border-slate-700 rounded text-slate-200 ${
					type === "color" ? "h-7 p-0 overflow-hidden" : ""
				}`}
			/>
		</div>
	);
}

export function FoodTypeCard({
	index,
	foodType,
	sharedGrowthRate = 0.09,
	sharedRecoveryRate = 0.01,
	canRemove,
	onRemove,
	onChange,
}: FoodTypeCardProps) {
	const sharedReward = useConfigStore(
		(state) => state.serverConfig?.energy.costs.eat_reward_per_food ?? 5,
	);
	// The server refuses an initial density above the live shared maximum.
	const maxDensity = useConfigStore(
		(state) => state.localDraft?.world.food.shared.max_density ?? 1,
	);
	const overrides = [
		["energy_per_unit", "Energy per density unit", sharedReward, undefined],
		["growth_rate", "Growth per tick", sharedGrowthRate, 1],
		["recovery_spawn_rate", "Recovery attempts / cell / tick", sharedRecoveryRate, 1],
	] as const;
	const densityField: FieldDef = {
		path: `world.food.types.${index}.initial_density`,
		label: "Initial Density",
		min: 0,
		max: maxDensity,
		step: 0.01,
		defaultValue: 1.0,
		tooltip: "Starting density for this food type during world generation",
	};
	const coverageField: FieldDef = {
		path: `world.food.types.${index}.initial_coverage`,
		label: "Coverage",
		min: 0,
		max: 1,
		step: 0.01,
		// Keep the card reset value aligned with the production fallback in
		// `createFoodType`: each configured type starts with 54% coverage.
		defaultValue: 0.54,
		tooltip:
			"Fraction of eligible passable cells seeded with this food type; fertile-only placement restricts eligibility",
	};
	const inhibitorField: FieldDef = {
		path: `world.food.types.${index}.growth_inhibitor`,
		label: "Growth Inhibitor",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.2,
		tooltip: "How strongly this type suppresses other food growth in occupied cells",
	};

	return (
		<div
			data-testid={`startup-food-type-card-${index}`}
			className="rounded border border-slate-800 bg-slate-900/40 p-2 flex flex-col gap-2"
		>
			<div className="flex items-start justify-between gap-2">
				<p className="flex items-baseline gap-2 text-xs font-medium text-slate-200">
					Food Type {index + 1}
					<span className="font-mono text-[10px] font-normal text-slate-400">type_idx {index}</span>
				</p>
				<button
					type="button"
					data-testid={`startup-food-type-remove-${index}`}
					disabled={!canRemove}
					onClick={onRemove}
					className="px-2 py-0.5 text-[11px] text-rose-200 bg-rose-900/30 hover:bg-rose-900/50 rounded disabled:opacity-40 disabled:cursor-not-allowed"
				>
					Remove
				</button>
			</div>

			<TextControl
				label="Name"
				testId={`startup-field-food-type-${index}-name`}
				value={foodType.name}
				onChange={(name) => onChange({ name })}
			/>

			<TextControl
				label="Color"
				testId={`startup-field-food-type-${index}-color`}
				value={foodType.color}
				type="color"
				onChange={(color) => onChange({ color })}
			/>

			<FieldRow
				field={densityField}
				id={`startup-food-type-${index}-initial-density`}
				value={foodType.initial_density}
				disabled={false}
				onChange={(_, value) => onChange({ initial_density: value })}
				testId={`startup-field-food-type-${index}-initial-density`}
			/>

			<FieldRow
				field={coverageField}
				id={`startup-food-type-${index}-initial-coverage`}
				value={foodType.initial_coverage}
				disabled={false}
				onChange={(_, value) => onChange({ initial_coverage: value })}
				testId={`startup-field-food-type-${index}-initial-coverage`}
			/>

			<label className="flex items-center gap-2 text-xs text-slate-300">
				<input
					type="checkbox"
					role="switch"
					aria-checked={foodType.initial_fertility_only ?? false}
					className="config-switch config-switch-startup config-switch-sm"
					checked={foodType.initial_fertility_only ?? false}
					onChange={(event) => onChange({ initial_fertility_only: event.target.checked })}
					data-testid={`startup-food-type-${index}-fertile-only`}
				/>
				Seed only positive-fertility cells
			</label>
			<p className="text-xs text-slate-400">
				Coverage is a fraction of eligible passable cells. Type edits apply on Restart.
			</p>
			{overrides.map(([key, label, sharedValue, max]) => {
				const inherited = foodType[key] == null;
				const id = `startup-food-type-${index}-${key}`;
				return (
					<div key={key} className="flex flex-col gap-1 border-t border-slate-800 pt-2">
						<label htmlFor={id} className="text-xs text-slate-300">
							{label}
						</label>
						<div className="flex items-center gap-2">
							<label className="flex items-center gap-1 text-xs text-slate-400">
								<input
									type="checkbox"
									role="switch"
									aria-checked={inherited}
									className="config-switch config-switch-startup config-switch-sm"
									checked={inherited}
									onChange={(event) =>
										onChange({ [key]: event.target.checked ? null : sharedValue })
									}
									aria-label={`Use shared ${label}`}
								/>
								Shared ({formatConfigNumber(sharedValue)})
							</label>
							<NumberInput
								id={id}
								min={0}
								max={max}
								step={0.01}
								value={foodType[key] ?? sharedValue}
								disabled={inherited}
								onChange={(value) => onChange({ [key]: value })}
								className={NUMBER_INPUT_CLASS}
							/>
						</div>
					</div>
				);
			})}

			<FieldRow
				field={inhibitorField}
				id={`startup-food-type-${index}-growth-inhibitor`}
				value={foodType.growth_inhibitor}
				disabled={false}
				onChange={(_, value) => onChange({ growth_inhibitor: value })}
				testId={`startup-field-food-type-${index}-growth-inhibitor`}
			/>
		</div>
	);
}

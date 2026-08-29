import type { FoodTypeConfig } from "../../../types/config.ts";
import { FieldRow } from "../shared/FieldRow.tsx";
import type { FieldDef } from "../shared/types.ts";

interface FoodTypeCardProps {
	index: number;
	foodType: FoodTypeConfig;
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
	canRemove,
	onRemove,
	onChange,
}: FoodTypeCardProps) {
	const densityField: FieldDef = {
		path: `world.food.types.${index}.initial_density`,
		label: "Initial Density",
		min: 0,
		max: 1,
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
		defaultValue: 0.54,
		tooltip: "Fraction of world cells seeded with this food type",
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
				<div className="flex flex-col gap-0.5">
					<p className="text-[11px] uppercase tracking-wide text-slate-400">
						Food Type {index + 1}
					</p>
					<p className="text-[11px] text-slate-500">type_idx {index}</p>
				</div>
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

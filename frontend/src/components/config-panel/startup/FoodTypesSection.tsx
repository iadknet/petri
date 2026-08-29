import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { FoodTypeConfig } from "../../../types/config.ts";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FoodTypeCard } from "./FoodTypeCard.tsx";

interface FoodTypesSectionProps {
	startupPreset: StartupPreset;
	addFoodType: () => void;
	removeFoodType: (index: number) => void;
	updateFoodType: (index: number, patch: Partial<FoodTypeConfig>) => void;
}

export function FoodTypesSection({
	startupPreset,
	addFoodType,
	removeFoodType,
	updateFoodType,
}: FoodTypesSectionProps) {
	const foodTypes = startupPreset.world.food.types;

	return (
		<FieldGroup title="Food Types">
			<div className="flex items-center justify-between gap-2 px-1 pb-1">
				<p className="text-[11px] text-slate-500">
					List order defines `type_idx` for sensors and fertility targets.
				</p>
				<button
					type="button"
					data-testid="startup-food-type-add"
					onClick={addFoodType}
					className="px-2 py-0.5 text-[11px] text-slate-200 bg-slate-700 hover:bg-slate-600 rounded"
				>
					Add Food Type
				</button>
			</div>
			{foodTypes.map((foodType, index) => (
				<FoodTypeCard
					key={`food-type-${index}`}
					index={index}
					foodType={foodType}
					canRemove={foodTypes.length > 1}
					onRemove={() => removeFoodType(index)}
					onChange={(patch) => updateFoodType(index, patch)}
				/>
			))}
		</FieldGroup>
	);
}

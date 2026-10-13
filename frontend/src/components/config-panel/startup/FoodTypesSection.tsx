import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { FoodTypeConfig } from "../../../types/config.ts";
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
		<div className="flex flex-col gap-2 px-3 pt-1">
			<div className="flex items-center justify-between gap-2 px-1 pb-1">
				<p className="text-[11px] text-slate-400">
					List order sets each type's <code className="font-mono text-slate-400">type_idx</code> for
					sensors and fertility targets.
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
					sharedGrowthRate={startupPreset.world.food.shared.growth_rate}
					sharedRecoveryRate={startupPreset.world.food.shared.recovery_spawn_rate}
					canRemove={foodTypes.length > 1}
					onRemove={() => removeFoodType(index)}
					onChange={(patch) => updateFoodType(index, patch)}
				/>
			))}
		</div>
	);
}

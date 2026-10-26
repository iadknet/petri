import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { SimulationConfig } from "../../../types/generated/SimulationConfig.ts";
import { Category, Unfiltered } from "../shared/Category.tsx";
import { baselineValueAt } from "../shared/baseline.ts";
import { matchesTerms, useConfigFilter } from "../shared/filter.ts";
import { sameConfigNumber } from "../shared/numberFormat.ts";
import { getByPath } from "../shared/pathUtils.ts";
import type { StartupPanelProps } from "../shared/types.ts";
import { ENERGY_STARTUP_FIELDS, EnergySection } from "./EnergySection.tsx";
import {
	ANNEALING_FIELDS,
	ANNEALING_TOGGLE,
	FERTILITY_FIELDS,
	FERTILITY_TOGGLE,
	FertilitySection,
} from "./FertilitySection.tsx";
import { FoodTypesSection } from "./FoodTypesSection.tsx";
import {
	FOUNDER_PROFILE_FIELD,
	POPULATION_STARTUP_FIELDS,
	PopulationSection,
} from "./PopulationSection.tsx";
import { RecipeControls } from "./RecipeControls.tsx";
import { RunSettingsSection } from "./RunSettingsSection.tsx";
import { TerrainSection } from "./TerrainSection.tsx";
import {
	EDGE_MODE_FIELD,
	WORLD_TOPOLOGY_FIELDS,
	WorldTopologySection,
} from "./WorldTopologySection.tsx";

interface StartupCategoryDef {
	id: string;
	title: string;
	summary: string;
	/** Setting names the search can find; list editors are matched whole. */
	keywords: string[];
}

export const STARTUP_CATEGORIES: StartupCategoryDef[] = [
	{
		id: "world",
		title: "World",
		summary: "Seed, map size and edges, and the founders",
		keywords: [
			"Run Seed seed",
			"Initial Creatures population founders",
			"Founder Profile genome forage first",
			"Initial Energy founders litter",
			"Edge Mode wrap bounded",
			"Width world size",
			"Height world size",
		],
	},
	{
		id: "terrain",
		title: "Terrain",
		summary: "Barriers laid down before food and founders",
		keywords: [
			"Terrain barriers walls",
			"Map Seed world seed",
			"Terrain layer pattern",
			"FbmThreshold Maze Spiral Noise ParallelLines Star",
			"Octaves Frequency Lacunarity Persistence Barrier threshold",
			"Density Cluster size",
			"Explicit bounds x y width height",
		],
	},
	{
		id: "food",
		title: "Food Types",
		summary: "What grows, where it starts, and what it is worth",
		keywords: [
			"Food Types type_idx",
			"Name Color",
			"Initial Density",
			"Coverage fertile-only positive-fertility",
			"Energy per density unit",
			"Growth per tick",
			"Recovery attempts per cell",
			"Growth Inhibitor",
		],
	},
	{
		id: "fertility",
		title: "Fertility",
		summary: "Where food grows faster or slower, and the warmup ramp",
		keywords: [
			"Fertility Layer Min Fertility Max Fertility",
			"Fertility layers target algorithm",
			"Uniform FBM Poisson Blobs Channels",
			"Blend Weighted Average Max Weight",
			"Blob Count Min Radius Max Radius Falloff",
			"Channel Width Value",
			"Octaves Frequency Lacunarity Persistence",
			"Use Seed",
			"Warmup Annealing Ramp Ticks Initial Min Fertility Initial Max Fertility",
		],
	},
];

/** Setting names on the Restart tab that match a search. */
export function startupMatchCount(terms: string[]): number {
	if (terms.length === 0) return 0;
	return STARTUP_CATEGORIES.reduce(
		(count, category) =>
			count + category.keywords.filter((keyword) => matchesTerms(terms, [keyword])).length,
		0,
	);
}

/** Preset paths that have a row of their own, so a pending change can be counted. */
function startupRowPaths(preset: StartupPreset): string[] {
	return [
		...POPULATION_STARTUP_FIELDS,
		FOUNDER_PROFILE_FIELD,
		...ENERGY_STARTUP_FIELDS,
		EDGE_MODE_FIELD,
		...WORLD_TOPOLOGY_FIELDS,
		FERTILITY_TOGGLE,
		...FERTILITY_FIELDS,
		ANNEALING_TOGGLE,
		...ANNEALING_FIELDS,
	]
		.map((field) => field.path)
		.concat(
			preset.world.food.types.flatMap((_, index) =>
				["initial_density", "initial_coverage", "growth_inhibitor"].map(
					(key) => `world.food.types.${index}.${key}`,
				),
			),
		);
}

/** Restart-tab rows whose preset differs from the config the running world started with. */
export function startupChangedCount(
	preset: StartupPreset,
	baseline: SimulationConfig | null,
): number {
	if (!baseline) return 0;
	return startupRowPaths(preset).filter((path) => {
		const next = getByPath(preset, path);
		const applied = baselineValueAt(baseline, path);
		if (applied === undefined || next === undefined) return false;
		return typeof next === "number" && typeof applied === "number"
			? !sameConfigNumber(next, applied)
			: next !== applied;
	}).length;
}

export function StartupConfigPanel({
	startupPreset,
	updateStartupPreset,
	addFoodType,
	removeFoodType,
	updateFoodType,
	addFertilityLayer,
	updateFertilityLayerTarget,
	randomizeSeed,
}: StartupPanelProps) {
	const { terms } = useConfigFilter();
	const searching = terms.length > 0;
	const visible = (id: string) => {
		const category = STARTUP_CATEGORIES.find((candidate) => candidate.id === id)!;
		return !searching || category.keywords.some((keyword) => matchesTerms(terms, [keyword]));
	};
	const meta = (id: string) => {
		const { title, summary } = STARTUP_CATEGORIES.find((candidate) => candidate.id === id)!;
		return { id: `restart-${id}`, title, summary, forceOpen: searching };
	};

	return (
		<>
			{!searching && <RecipeControls />}
			{visible("world") && (
				<Category {...meta("world")}>
					<RunSettingsSection
						seed={startupPreset.seed}
						updateSeed={(value) => updateStartupPreset("seed", value)}
						randomizeSeed={randomizeSeed}
					/>
					<PopulationSection
						startupPreset={startupPreset}
						updateStartupPreset={updateStartupPreset}
					/>
					<EnergySection startupPreset={startupPreset} updateStartupPreset={updateStartupPreset} />
					<WorldTopologySection
						startupPreset={startupPreset}
						updateStartupPreset={updateStartupPreset}
					/>
				</Category>
			)}
			{visible("terrain") && (
				<Category {...meta("terrain")}>
					<Unfiltered>
						<TerrainSection
							startupPreset={startupPreset}
							updateStartupPreset={updateStartupPreset}
						/>
					</Unfiltered>
				</Category>
			)}
			{visible("food") && (
				<Category {...meta("food")}>
					<Unfiltered>
						<FoodTypesSection
							startupPreset={startupPreset}
							addFoodType={addFoodType}
							removeFoodType={removeFoodType}
							updateFoodType={updateFoodType}
						/>
					</Unfiltered>
				</Category>
			)}
			{visible("fertility") && (
				<Category {...meta("fertility")}>
					<Unfiltered>
						<FertilitySection
							startupPreset={startupPreset}
							updateStartupPreset={updateStartupPreset}
							addFertilityLayer={addFertilityLayer}
							updateFertilityLayerTarget={updateFertilityLayerTarget}
						/>
					</Unfiltered>
				</Category>
			)}
		</>
	);
}

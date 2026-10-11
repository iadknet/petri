import type { AnyFieldDef } from "../shared/types.ts";
import { ACTION_LOG_FIELDS } from "./ActionLogSection.tsx";
import { AGE_COST_ALL_FIELDS } from "./AgeEnergyCostSection.tsx";
import { COMPLEXITY_COST_ALL_FIELDS } from "./ComplexityEnergyCostSection.tsx";
import { ENERGY_COSTS_FIELDS } from "./EnergyCostsSection.tsx";
import { ENERGY_LIFECYCLE_FIELDS } from "./EnergyLifecycleSection.tsx";
import {
	FOOD_GRAZING_FIELDS,
	FOOD_GRAZING_TOGGLES,
	FOOD_OCCUPANCY_DEPLETION_FIELDS,
	FOOD_OCCUPANCY_DEPLETION_TOGGLES,
	FOOD_PARAMETERS_FIELDS,
} from "./FoodParametersSection.tsx";
import { MUTATION_ALL_FIELDS } from "./MutationSection.tsx";
import { POPULATION_FIELDS } from "./PopulationSection.tsx";
import { PREDATION_FIELDS } from "./PredationSection.tsx";
import { RUNTIME_FIELDS } from "./RuntimeSection.tsx";
import { SHARED_MEMORY_FIELDS } from "./SharedMemorySection.tsx";

/** Every field the live panel can PATCH, and the set Restart re-applies. */
export const RUNTIME_PATCH_FIELDS: AnyFieldDef[] = [
	...FOOD_OCCUPANCY_DEPLETION_TOGGLES,
	...FOOD_OCCUPANCY_DEPLETION_FIELDS,
	...FOOD_GRAZING_TOGGLES,
	...FOOD_GRAZING_FIELDS,
	...FOOD_PARAMETERS_FIELDS,
	...POPULATION_FIELDS,
	...ENERGY_LIFECYCLE_FIELDS,
	...ENERGY_COSTS_FIELDS,
	...COMPLEXITY_COST_ALL_FIELDS,
	...AGE_COST_ALL_FIELDS,
	...RUNTIME_FIELDS,
	...MUTATION_ALL_FIELDS,
	...PREDATION_FIELDS,
	...ACTION_LOG_FIELDS,
	...SHARED_MEMORY_FIELDS,
];

const BY_PATH = new Map(RUNTIME_PATCH_FIELDS.map((field) => [field.path, field]));

function pick(...paths: string[]): AnyFieldDef[] {
	return paths.map((path) => {
		const field = BY_PATH.get(path);
		if (!field) throw new Error(`no live config field at ${path}`);
		return field;
	});
}

export interface ConfigSubgroup {
	title: string;
	fields: AnyFieldDef[];
}

export interface ConfigCategory {
	id: string;
	title: string;
	/** One plain line on what the category governs. */
	summary: string;
	subgroups: ConfigSubgroup[];
}

/**
 * The live settings, grouped by what they mean to a creature rather than by
 * config struct. Subgroups stay at seven rows or fewer.
 */
export const RUNTIME_CATEGORIES: ConfigCategory[] = [
	{
		id: "food",
		title: "Food",
		summary: "How food grows, spreads and recovers",
		subgroups: [
			{
				title: "Growth and spread",
				fields: pick(
					"world.food.shared.growth_rate",
					"world.food.shared.spread_threshold_ratio",
					"world.food.shared.spread_density_ratio",
					"world.food.shared.max_density",
				),
			},
			{
				title: "Recovery",
				fields: pick(
					"world.food.shared.recovery_spawn_rate",
					"world.food.shared.recovery_floor_ratio",
				),
			},
			{
				title: "Occupancy depletion",
				fields: pick(
					"world.food.shared.occupancy_depletion.enabled",
					"world.food.shared.occupancy_depletion.deposit_per_occupied_tick",
				),
			},
			{
				title: "Grazing",
				fields: pick(
					"world.food.shared.grazing.enabled",
					"world.food.shared.grazing.factor",
					"world.food.shared.grazing.floor",
					"world.food.shared.grazing.recovery_ticks",
				),
			},
		],
	},
	{
		id: "energy",
		title: "Energy",
		summary: "What staying alive and acting cost",
		subgroups: [
			{
				title: "Budget",
				fields: pick(
					"energy.lifecycle.max_energy",
					"energy.lifecycle.energy_decay_per_tick",
					"energy.costs.eat_reward_per_food",
					"energy.lifecycle.genome_carry_cost_per_unit",
				),
			},
			{
				title: "Action effort",
				fields: pick(
					"energy.costs.noop_cost",
					"energy.costs.move_cost",
					"energy.costs.eat_cost",
					"energy.costs.reproduce_attempt_cost",
				),
			},
			{
				title: "Age cost",
				fields: pick(
					"energy.age_cost.enabled",
					"energy.age_cost.grace_ticks",
					"energy.age_cost.age_cap",
					"energy.age_cost.max_multiplier",
				),
			},
			{
				title: "Complexity cost",
				fields: pick(
					"energy.complexity_cost.enabled",
					"energy.complexity_cost.threshold",
					"energy.complexity_cost.scaling_factor",
				),
			},
		],
	},
	{
		id: "reproduction",
		title: "Reproduction",
		summary: "When creatures can breed and what a birth costs",
		subgroups: [
			{
				title: "Who may breed",
				fields: pick(
					"energy.lifecycle.min_reproduce_energy",
					"energy.lifecycle.min_reproduce_age",
					"population.max_creatures",
				),
			},
			{
				title: "Cost of a birth",
				fields: pick(
					"energy.costs.reproduce_cost",
					"energy.lifecycle.genome_replication_cost_per_unit",
					"energy.lifecycle.gestation_ticks_per_founder_genome",
					"energy.lifecycle.default_offspring_energy",
				),
			},
		],
	},
	{
		id: "brain",
		title: "Brain and Senses",
		summary: "How much a brain may compute, and the cost",
		subgroups: [
			{
				title: "Limits",
				fields: pick(
					"runtime.max_mesh_hops",
					"runtime.max_vm_steps",
					"runtime.max_actions_per_turn",
				),
			},
			{
				title: "Compute cost",
				fields: pick(
					"runtime.graph_node_base_cost",
					"runtime.vm.opcode_cost_multiplier",
					"runtime.vm.step_ramp_allowance",
					"runtime.vm.step_ramp_cost",
					"runtime.hop_ramp_allowance",
					"runtime.hop_ramp_cost",
				),
			},
			{
				title: "Learning cost",
				fields: pick("runtime.plasticity_update_cost", "runtime.reward_learning_cost"),
			},
			{
				title: "Senses and memory",
				fields: pick(
					"runtime.perception.vision_radius",
					"energy.lifecycle.age_reference_ticks",
					"shared_memory.decay_rate",
				),
			},
		],
	},
	{
		id: "mutation",
		title: "Mutation",
		summary: "How often offspring genomes change, and where",
		subgroups: [
			{
				title: "Supply",
				fields: pick(
					"mutation.per_unit_rate",
					"mutation.mesh_layer_probability",
					"mutation.large_copy_weight_percent",
				),
			},
			{
				title: "Genome size",
				fields: pick("mutation.genome_size_cap", "mutation.genome_size_pressure_enabled"),
			},
			{
				title: "Targeting",
				fields: pick(
					"mutation.executed_bias",
					"mutation.executed_window_ticks",
					"mutation.reachable_bias.topology",
					"mutation.reachable_bias.vm",
					"mutation.reachable_bias.graph",
					"mutation.reachable_bias.input_ref",
				),
			},
			{
				title: "Experimental arms",
				fields: pick(
					"mutation.structured_heritable_refinement",
					"mutation.neutral_input_recruitment",
				),
			},
			{
				title: "Colour drift",
				fields: pick(
					"mutation.phenotype.channel_step",
					"mutation.phenotype.polarity_flip_chance",
					"mutation.phenotype.channel_change_chance",
				),
			},
		],
	},
	{
		id: "predation",
		title: "Predation",
		summary: "What stealing costs and what a kill pays",
		subgroups: [
			{
				title: "Stealing and kills",
				fields: pick("predation.steal_cost_rate", "predation.kill_complexity_bonus_multiplier"),
			},
		],
	},
	{
		id: "diagnostics",
		title: "Diagnostics",
		summary: "What the simulation records for inspection",
		subgroups: [{ title: "Action log", fields: pick("action_log.capacity") }],
	},
];

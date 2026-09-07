import { describe, expect, it } from "vitest";
import { MOCK_CONFIG } from "../../../test/fixtures.ts";
import type { SimulationConfig } from "../../../types/api.ts";
import type { FieldDef } from "../shared/types.ts";
import { RUNTIME_PATCH_FIELDS } from "./RuntimeConfigPanel.tsx";
import { resolveRuntimeBounds } from "./bounds.ts";

function fieldFor(path: string): FieldDef {
	const field = RUNTIME_PATCH_FIELDS.find((candidate) => candidate.path === path);
	if (!field || !("min" in field)) {
		throw new Error(`no numeric runtime field at ${path}`);
	}
	return field;
}

function draftWith(patch: (draft: SimulationConfig) => void): SimulationConfig {
	const draft = structuredClone(MOCK_CONFIG);
	patch(draft);
	return draft;
}

describe("resolveRuntimeBounds", () => {
	it("returns the static bounds for fields without a cross-field constraint", () => {
		const field = fieldFor("energy.costs.move_cost");

		expect(resolveRuntimeBounds(field, MOCK_CONFIG)).toEqual({ min: field.min, max: field.max });
	});

	it.each([
		["population.max_creatures", { min: 50, max: 100000 }],
		["world.food.shared.max_density", { min: 1, max: 1 }],
		["runtime.max_actions_per_turn", { min: 4, max: 20 }],
		["mutation.action_queue_cap", { min: 1, max: 10 }],
		["mutation.per_birth_mutation_events_min", { min: 1, max: 4 }],
		["mutation.per_birth_mutation_events_max", { min: 1, max: 20 }],
		["action_log.capacity", { min: 1, max: 5000 }],
	])("derives %s bounds from the draft", (path, expected) => {
		expect(resolveRuntimeBounds(fieldFor(path), MOCK_CONFIG)).toEqual(expected);
	});

	it("takes the largest per-type initial density as the max density floor", () => {
		const draft = draftWith((d) => {
			d.world.food.types = [
				{ ...d.world.food.types[0]!, initial_density: 0.4 },
				{ ...d.world.food.types[0]!, initial_density: 0.7 },
			];
		});

		expect(resolveRuntimeBounds(fieldFor("world.food.shared.max_density"), draft).min).toBe(0.7);
	});

	it("falls back to the static minimum when the draft has no food types", () => {
		const field = fieldFor("world.food.shared.max_density");
		const draft = draftWith((d) => {
			d.world.food.types = [];
		});

		expect(resolveRuntimeBounds(field, draft)).toEqual({ min: field.min, max: field.max });
	});

	it("keeps the resolved pair ordered when the derived min exceeds the static max", () => {
		const draft = draftWith((d) => {
			d.population.initial_creatures = 200000;
		});

		expect(resolveRuntimeBounds(fieldFor("population.max_creatures"), draft)).toEqual({
			min: 200000,
			max: 200000,
		});
	});

	it("keeps the resolved pair ordered when the derived max is below the static min", () => {
		const draft = draftWith((d) => {
			d.mutation.per_birth_mutation_events_max = 0;
		});

		expect(resolveRuntimeBounds(fieldFor("mutation.per_birth_mutation_events_min"), draft)).toEqual(
			{ min: 1, max: 1 },
		);
	});
});

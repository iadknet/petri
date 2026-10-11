import { describe, expect, it } from "vitest";
import { MOCK_CONFIG } from "../../../test/fixtures.ts";
import { isEnumField, isNumberField } from "../shared/types.ts";
import { RUNTIME_CATEGORIES, RUNTIME_PATCH_FIELDS } from "./groups.ts";

/** Config the server owns at startup: PATCH refuses it, or the Restart tab sets it. */
const STARTUP_OWNED = [
	"population.initial_creatures",
	"population.founder_profile",
	"world.width",
	"world.height",
	"world.edge_mode",
	"world.terrain",
	"world.world_seed",
	"world.food.types",
	"world.food.fertility",
	"world.food.annealing",
	"energy.lifecycle.initial_energy",
];

function leafPaths(value: unknown, prefix = ""): string[] {
	if (value === null || typeof value !== "object" || Array.isArray(value)) return [prefix];
	return Object.entries(value).flatMap(([key, child]) =>
		leafPaths(child, prefix ? `${prefix}.${key}` : key),
	);
}

const grouped = RUNTIME_CATEGORIES.flatMap((category) =>
	category.subgroups.flatMap((subgroup) => subgroup.fields.map((field) => field.path)),
);

describe("live config coverage", () => {
	it("offers every live-patchable config value", () => {
		const patchable = leafPaths(MOCK_CONFIG).filter(
			(path) => !STARTUP_OWNED.some((owned) => path === owned || path.startsWith(`${owned}.`)),
		);
		const offered = new Set(RUNTIME_PATCH_FIELDS.map((field) => field.path));
		expect(patchable.filter((path) => !offered.has(path))).toEqual([]);
	});

	it("lists each field once and shows each in exactly one group", () => {
		const paths = RUNTIME_PATCH_FIELDS.map((field) => field.path);
		expect(new Set(paths).size).toBe(paths.length);
		expect([...grouped].sort()).toEqual([...paths].sort());
	});

	it("keeps every group small enough to scan", () => {
		for (const category of RUNTIME_CATEGORIES) {
			for (const subgroup of category.subgroups) {
				expect(subgroup.fields.length, `${category.title} / ${subgroup.title}`).toBeLessThanOrEqual(
					7,
				);
			}
		}
	});

	it("gives every number field a default inside its bounds and a step that fits it", () => {
		for (const field of RUNTIME_PATCH_FIELDS) {
			if (!isNumberField(field) || field.defaultValue === undefined) continue;
			expect(field.defaultValue, field.path).toBeGreaterThanOrEqual(field.min);
			expect(field.defaultValue, field.path).toBeLessThanOrEqual(field.max);
			if (field.defaultValue > 0) {
				expect(field.step, field.path).toBeLessThanOrEqual(field.defaultValue);
			}
		}
	});

	it("offers the default among an enum field's options", () => {
		for (const field of RUNTIME_PATCH_FIELDS) {
			if (!isEnumField(field)) continue;
			expect(field.options.map((option) => option.value)).toContain(field.defaultValue);
		}
	});
});

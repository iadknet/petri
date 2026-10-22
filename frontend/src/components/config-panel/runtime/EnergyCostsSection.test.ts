import { describe, expect, it } from "vitest";
import { ENERGY_COSTS_FIELDS } from "./EnergyCostsSection.tsx";
import { EXCHANGE_FIELDS, PREDATION_FIELDS } from "./PredationSection.tsx";

const FIELDS = [...ENERGY_COSTS_FIELDS, ...EXCHANGE_FIELDS, ...PREDATION_FIELDS];

function fieldFor(path: string) {
	const field = FIELDS.find((candidate) => candidate.path === path);
	if (!field) {
		throw new Error(`no field at ${path}`);
	}
	return field;
}

describe("effort-priced action costs (T16.F02)", () => {
	it("retires the failed action penalty field", () => {
		expect(FIELDS.map((field) => field.path)).not.toContain("energy.costs.failed_action_penalty");
	});

	it("shows the eat effort default and the reproduce attempt effort", () => {
		expect(fieldFor("energy.costs.eat_cost").defaultValue).toBe(0.05);
		expect(fieldFor("energy.costs.reproduce_attempt_cost")).toMatchObject({
			label: "Reproduce Attempt Cost",
			min: 0,
			defaultValue: 1.0,
			testId: "config-field-energy-costs-reproduce-attempt-cost",
		});
	});

	it("keeps Noop Cost above zero", () => {
		const noop = fieldFor("energy.costs.noop_cost");
		expect(noop.min).toBeGreaterThan(0);
		expect(noop.defaultValue).toBe(0.4);
	});

	it("shows the move cost default", () => {
		expect(fieldFor("energy.costs.move_cost").defaultValue).toBe(1.6);
	});

	it.each([
		"energy.costs.move_cost",
		"energy.costs.eat_cost",
		"energy.costs.noop_cost",
		"energy.costs.reproduce_attempt_cost",
		"exchange.effort_rate",
	])("%s says it is charged on every attempt and never below Noop Cost", (path) => {
		const tooltip = fieldFor(path).tooltip ?? "";
		expect(tooltip).toMatch(/every attempt/i);
		expect(tooltip).toMatch(/never below Noop Cost/);
	});

	it("labels Reproduce Cost as the replication cost paid only on a conception", () => {
		const tooltip = fieldFor("energy.costs.reproduce_cost").tooltip ?? "";
		expect(tooltip).toMatch(/replication cost/i);
		expect(tooltip).toMatch(/only on a conception/i);
	});
});

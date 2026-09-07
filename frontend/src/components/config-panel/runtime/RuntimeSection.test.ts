import { describe, expect, it } from "vitest";
import { MOCK_CONFIG } from "../../../test/fixtures.ts";
import { RUNTIME_FIELDS } from "./RuntimeSection.tsx";

function fieldFor(path: string) {
	const field = RUNTIME_FIELDS.find((candidate) => candidate.path === path);
	if (!field) {
		throw new Error(`no runtime field at ${path}`);
	}
	return field;
}

describe("activity-ramped compute cost rows", () => {
	it("exposes the step ramp allowance with the runtime default", () => {
		expect(fieldFor("runtime.vm.step_ramp_allowance")).toMatchObject({
			min: 0,
			max: 10000,
			step: 1,
			defaultValue: 100,
		});
	});

	it("exposes the step ramp cost with the runtime default", () => {
		expect(fieldFor("runtime.vm.step_ramp_cost")).toMatchObject({
			min: 0,
			max: 1,
			step: 0.000001,
			defaultValue: 0.000001,
		});
	});

	it("carries both ramp fields in the config transport shape", () => {
		expect(MOCK_CONFIG.runtime.vm.step_ramp_allowance).toBe(100);
		expect(MOCK_CONFIG.runtime.vm.step_ramp_cost).toBe(0.000001);
	});
});

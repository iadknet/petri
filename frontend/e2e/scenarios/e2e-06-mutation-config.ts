import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioMutationConfig: ScenarioDefinition = {
	id: "E2E-06",
	description: "Validates the mutation rate field is populated and can be applied",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 97531, 40);

		const rawMutationRate = (await ctx.browser.getValue(selectors.configMutationRateField)).value;
		assert(rawMutationRate.trim().length > 0, "mutation rate field should have an initial value");

		const currentValue = Number(rawMutationRate);
		const updatedValue = Math.min(0.1, currentValue + 0.001).toFixed(4);

		await ctx.browser.fill(selectors.configMutationRateField, updatedValue);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled;
		}, "apply button enabled after mutation edit", 10_000);

		await ctx.browser.click(selectors.configApply);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button disabled after mutation config apply", 10_000);

		const persistedValue = Number(
			(await ctx.browser.getValue(selectors.configMutationRateField)).value,
		);
		assert(
			Math.abs(persistedValue - Number(updatedValue)) < 0.00001,
			"mutation rate value should persist after apply",
		);
	},
};

import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioMutationConfig: ScenarioDefinition = {
	id: "E2E-06",
	description: "Validates mutation config fields are populated and can be applied",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 97531, 40);

		const rawMutationProbability = (
			await ctx.browser.getValue(selectors.configMutationProbabilityField)
		).value;
		assert(
			rawMutationProbability.trim().length > 0,
			"mutation probability field should have an initial value",
		);

		const currentValue = Number(rawMutationProbability);
		const updatedValue = Math.min(1, currentValue + 0.01).toFixed(3);

		await ctx.browser.fill(selectors.configMutationProbabilityField, updatedValue);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled;
		}, "apply button enabled after mutation edit", 10_000);

		await ctx.browser.click(selectors.configApply);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button disabled after mutation config apply", 10_000);

		const persistedValue = Number(
			(await ctx.browser.getValue(selectors.configMutationProbabilityField)).value,
		);
		assert(
			Math.abs(persistedValue - Number(updatedValue)) < 0.0001,
			"mutation probability value should persist after apply",
		);
	},
};

import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioConfigApplyResetLocks: ScenarioDefinition = {
	id: "E2E-03",
	description: "Validates config apply/reset flow and runtime field lock rules by lifecycle state",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 7878, 32);

		const currentValue = Number((await ctx.browser.getValue(selectors.configMoveCostField)).value);
		const updatedValue = (Math.round(Math.max(0, currentValue + 0.1) * 100) / 100).toFixed(2);

		await ctx.browser.fill(selectors.configMoveCostField, updatedValue);

		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled;
		}, "apply button enabled after config edit", 10_000);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configReset)).enabled;
		}, "reset button enabled after config edit", 10_000);

		await ctx.browser.click(selectors.configReset);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button disabled after reset", 10_000);

		await ctx.browser.fill(selectors.configMoveCostField, updatedValue);
		await ctx.browser.click(selectors.configApply);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button disabled after successful apply", 10_000);

		const persistedValue = Number((await ctx.browser.getValue(selectors.configMoveCostField)).value);
		assert(Math.abs(persistedValue - Number(updatedValue)) < 0.0001, "move cost value should persist after apply");

		await ctx.browser.click(selectors.controlStart);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configMoveCostField)).enabled === false;
		}, "non-topology config field disabled while running", 10_000);

		await ctx.browser.click(selectors.controlPause);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configMoveCostField)).enabled;
		}, "non-topology field re-enabled when paused", 10_000);
	},
};

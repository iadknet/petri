import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import {
	DEFAULT_E2E_WORLD_SIZE,
	openDashboardAndWaitConnection,
	restartAndWaitForBusyCycle,
	selectors,
} from "./common.ts";

async function expectNumericValue(
	ctx: Parameters<ScenarioDefinition["run"]>[0],
	selector: string,
	expected: number,
	message: string,
): Promise<void> {
	const actual = Number((await ctx.browser.getValue(selector)).value);
	assert(Math.abs(actual - expected) < 0.0001, message);
}

export const scenarioFertilityStartupConfig: ScenarioDefinition = {
	id: "E2E-09",
	description: "Validates fertility startup controls apply values and persist across restart",
	run: async (ctx) => {
		await openDashboardAndWaitConnection(ctx);

		const fertilityVisibleBeforeToggle = (
			await ctx.browser.isVisible(selectors.startupFertilityMin)
		).visible;
		if (!fertilityVisibleBeforeToggle) {
			await ctx.browser.click(selectors.startupFertilityEnabled);
		}

		await waitForCondition(async () => {
			return (await ctx.browser.isVisible(selectors.startupFertilityMin)).visible;
		}, "fertility numeric controls visible after enabling fertility", 10_000);

		await ctx.browser.fill(selectors.startupFertilityMin, "0.6");
		await ctx.browser.fill(selectors.startupFertilityMax, "1.8");
		await ctx.browser.eval(`(() => {
			const select = document.querySelector(${JSON.stringify(selectors.startupFertilityLayer0Algorithm)});
			if (!(select instanceof HTMLSelectElement)) {
				throw new Error("layer algorithm select not found");
			}
			select.value = "Fbm";
			select.dispatchEvent(new Event("change", { bubbles: true }));
			return select.value;
		})()`);
		await waitForCondition(async () => {
			return (await ctx.browser.isVisible(selectors.startupFertilityLayer0FbmOctaves)).visible;
		}, "fbm fields visible after selecting FBM algorithm", 10_000);
		await ctx.browser.fill(selectors.startupFertilityLayer0Weight, "0.65");
		await ctx.browser.fill(selectors.startupFertilityLayer0FbmOctaves, "5");
		await ctx.browser.fill(selectors.startupFertilityLayer0FbmFrequency, "0.08");
		await ctx.browser.click(selectors.startupFertilityLayer0FbmSeedEnabled);
		await ctx.browser.fill(selectors.startupFertilityLayer0FbmSeed, "4242");

		await ctx.browser.click(selectors.startupAnnealingEnabled);
		await ctx.browser.fill(selectors.startupAnnealingRampTicks, "9000");
		await ctx.browser.fill(selectors.startupAnnealingInitialMin, "0.8");
		await ctx.browser.fill(selectors.startupAnnealingInitialMax, "1.4");
		await ctx.browser.fill(selectors.startupWorldWidth, String(DEFAULT_E2E_WORLD_SIZE));
		await ctx.browser.fill(selectors.startupWorldHeight, String(DEFAULT_E2E_WORLD_SIZE));

		await restartAndWaitForBusyCycle(ctx);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.controlStart)).enabled;
		}, "start control enabled after fertility-configured restart", ctx.startupTimeoutMs);

		await expectNumericValue(
			ctx,
			selectors.startupFertilityMin,
			0.6,
			"fertility min should persist after restart",
		);
		await expectNumericValue(
			ctx,
			selectors.startupFertilityMax,
			1.8,
			"fertility max should persist after restart",
		);
		assert(
			(await ctx.browser.getValue(selectors.startupFertilityLayer0Algorithm)).value === "Fbm",
			"fertility layer algorithm should persist after restart",
		);
		await expectNumericValue(
			ctx,
			selectors.startupFertilityLayer0Weight,
			0.65,
			"fertility layer weight should persist after restart",
		);
		assert(
			(await ctx.browser.getValue(selectors.startupFertilityLayer0FbmOctaves)).value === "5",
			"fertility layer fbm octaves should persist after restart",
		);
		await expectNumericValue(
			ctx,
			selectors.startupFertilityLayer0FbmFrequency,
			0.08,
			"fertility layer fbm frequency should persist after restart",
		);
		assert(
			(await ctx.browser.getValue(selectors.startupFertilityLayer0FbmSeed)).value === "4242",
			"fertility layer fbm seed should persist after restart",
		);
		assert(
			(await ctx.browser.getValue(selectors.startupAnnealingRampTicks)).value === "9000",
			"annealing ramp ticks should persist after restart",
		);
		await expectNumericValue(
			ctx,
			selectors.startupAnnealingInitialMin,
			0.8,
			"annealing initial min should persist after restart",
		);
		await expectNumericValue(
			ctx,
			selectors.startupAnnealingInitialMax,
			1.4,
			"annealing initial max should persist after restart",
		);
	},
};

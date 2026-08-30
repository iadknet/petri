import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioPaintDrawing: ScenarioDefinition = {
	id: "E2E-07",
	description: "Exercises paint mode toggle, toolbar tools/brushes, and canvas drawing",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 9999, 30);

		// 1. Paint toggle button should be visible in paused state
		const toggleVisible = await ctx.browser.isVisible(selectors.paintToggle);
		assert(toggleVisible.visible, "paint toggle button should be visible in paused state");

		// 2. Click paint toggle → toolbar should appear
		await ctx.browser.click(selectors.paintToggle);
		await waitForCondition(async () => {
			return (await ctx.browser.isVisible(selectors.paintToolbar)).visible;
		}, "paint toolbar visible after toggle", 5_000);

		// 3. Verify toolbar has 4 tool buttons and 3 brush buttons
		const toolCount = await ctx.browser.getCount('[data-testid^="paint-tool-"]');
		assert(toolCount.count === 4, `expected 4 tool buttons, got ${toolCount.count}`);
		const brushCount = await ctx.browser.getCount('[data-testid^="paint-brush-"]');
		assert(brushCount.count === 3, `expected 3 brush buttons, got ${brushCount.count}`);

		// 4. Verify default tool (barrier) is active
		const barrierActive = await ctx.browser.getCount(
			'[data-testid="paint-tool-barrier"][aria-pressed="true"]',
		);
		assert(barrierActive.count === 1, "barrier tool should be active by default");

		// 5. Click food tool → verify it becomes active and barrier becomes inactive
		await ctx.browser.click(selectors.paintToolFood);
		const foodActive = await ctx.browser.getCount(
			'[data-testid="paint-tool-food"][aria-pressed="true"]',
		);
		assert(foodActive.count === 1, "food tool should be active after click");
		const barrierInactive = await ctx.browser.getCount(
			'[data-testid="paint-tool-barrier"][aria-pressed="false"]',
		);
		assert(barrierInactive.count === 1, "barrier tool should be inactive after selecting food");

		// 6. Click brush size 1 (3x3) → verify active
		await ctx.browser.click(selectors.paintBrush1);
		const brush1Active = await ctx.browser.getCount(
			'[data-testid="paint-brush-1"][aria-pressed="true"]',
		);
		assert(brush1Active.count === 1, "brush 3x3 should be active after click");

		// 7. Click paint toggle to exit → toolbar disappears
		await ctx.browser.click(selectors.paintToggle);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.paintToolbar)).count === 0;
		}, "paint toolbar hidden after toggle off", 5_000);

		// 8. Re-enter paint mode → toolbar reappears
		await ctx.browser.click(selectors.paintToggle);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.paintToolbar)).count > 0;
		}, "paint toolbar visible after re-toggle", 5_000);

		// 9. Click on canvas to simulate a paint stroke
		await ctx.browser.click(selectors.worldCanvas);

		// 10. No browser errors
		const errors = await ctx.browser.errors();
		assert(errors.errors.length === 0, `expected no browser errors, got: ${JSON.stringify(errors.errors)}`);
	},
};

import { DETAIL_ZOOM_THRESHOLD, INSPECT_ZOOM_THRESHOLD } from "../../src/stores/viewport.ts";
import { assert, waitForCondition } from "../lib/assertions.ts";
import type { RuntimeContext, ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

interface ViewportProbe {
	camera: { x: number; y: number; zoom: number };
	canvasSize: { width: number; height: number };
	worldSize: { width: number; height: number } | null;
	viewRequest: {
		rect: { x: number; y: number; width: number; height: number };
		canvas: { width: number; height: number };
		zoomTier: "overview" | "detail" | "inspect";
	} | null;
}

interface WorldViewProbe {
	projectionRevision: number;
	worldStaticRevision: number;
	viewKind: "overview" | "detail" | null;
	frameCreatureCount: number;
	frameFoodCount: number;
}

interface VisibleCreature {
	id: number;
	x: number;
	y: number;
}

interface CanvasPoint {
	clientX: number;
	clientY: number;
}

interface CanvasContentSummary {
	width: number;
	height: number;
	nonBackgroundSamples: number;
}

async function readViewportState(ctx: RuntimeContext): Promise<ViewportProbe> {
	return ctx.browser.eval<ViewportProbe>("window.__PETRI_E2E__.getViewportState()");
}

async function readWorldViewState(ctx: RuntimeContext): Promise<WorldViewProbe> {
	return ctx.browser.eval<WorldViewProbe>("window.__PETRI_E2E__.getWorldViewState()");
}

async function listVisibleCreatures(ctx: RuntimeContext): Promise<VisibleCreature[]> {
	return ctx.browser.eval<VisibleCreature[]>("window.__PETRI_E2E__.listVisibleCreatures()");
}

async function getCanvasPointForWorld(
	ctx: RuntimeContext,
	x: number,
	y: number,
): Promise<CanvasPoint | null> {
	return ctx.browser.eval<CanvasPoint | null>(
		`window.__PETRI_E2E__.getCanvasPointForWorld(${x}, ${y})`,
	);
}

async function getSelectedCreatureId(ctx: RuntimeContext): Promise<number | null> {
	return ctx.browser.eval<number | null>("window.__PETRI_E2E__.getSelectedCreatureId()");
}

async function getCanvasContentSummary(ctx: RuntimeContext): Promise<CanvasContentSummary | null> {
	return ctx.browser.eval<CanvasContentSummary | null>(
		"window.__PETRI_E2E__.getCanvasContentSummary()",
	);
}

async function zoomToTier(
	ctx: RuntimeContext,
	targetTier: "detail" | "inspect",
	maxClicks = 16,
): Promise<void> {
	for (let attempt = 0; attempt < maxClicks; attempt++) {
		const viewport = await readViewportState(ctx);
		const hasReachedDetail = viewport.camera.zoom >= DETAIL_ZOOM_THRESHOLD;
		const hasReachedInspect = viewport.camera.zoom >= INSPECT_ZOOM_THRESHOLD;
		if (targetTier === "detail" && hasReachedDetail) {
			return;
		}
		if (targetTier === "inspect" && hasReachedInspect) {
			return;
		}
		await ctx.browser.click(selectors.zoomIn);
		await waitForCondition(
			async () => {
				const nextViewport = await readViewportState(ctx);
				return nextViewport.camera.zoom > viewport.camera.zoom;
			},
			`zoom transition toward ${targetTier}`,
			10_000,
		);
	}

	const viewport = await readViewportState(ctx);
	throw new Error(`failed to reach ${targetTier} tier, current zoom=${viewport.camera.zoom}`);
}

export const scenarioViewportSelection: ScenarioDefinition = {
	id: "E2E-08",
	description: "Verifies overview/detail rendering transitions and real creature selection",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 424242, 800);

		await waitForCondition(
			async () => {
				return Boolean(await ctx.browser.eval("Boolean(window.__PETRI_E2E__)"));
			},
			"E2E hooks ready",
			ctx.startupTimeoutMs,
		);

		await waitForCondition(
			async () => {
				const viewport = await readViewportState(ctx);
				return viewport.camera.zoom > 0;
			},
			"initial overview state settled",
			10_000,
		);

		const initialViewport = await readViewportState(ctx);
		const initialWorldView = await readWorldViewState(ctx);

		assert(
			initialViewport.camera.zoom < DETAIL_ZOOM_THRESHOLD,
			"initial camera zoom should be overview scale",
		);
		assert(initialWorldView.viewKind === "overview", "initial world view should be overview");
		assert(initialWorldView.frameCreatureCount === 0, "overview should not expose frame creatures");
		await waitForCondition(
			async () => {
				const summary = await getCanvasContentSummary(ctx);
				return (summary?.nonBackgroundSamples ?? 0) > 0;
			},
			"overview canvas renders non-background content",
			10_000,
		);

		await zoomToTier(ctx, "detail");

		await waitForCondition(
			async () => {
				const worldView = await readWorldViewState(ctx);
				return worldView.viewKind === "detail";
			},
			"detail view payload delivered",
			10_000,
		);

		const detailViewport = await readViewportState(ctx);
		const detailWorldView = await readWorldViewState(ctx);

		assert(
			detailViewport.camera.zoom >= DETAIL_ZOOM_THRESHOLD,
			"camera zoom should advance to detail scale",
		);
		assert(
			detailViewport.camera.zoom > initialViewport.camera.zoom,
			"camera zoom should increase after zooming in",
		);
		assert(detailWorldView.viewKind === "detail", "world view should switch to detail");
		await waitForCondition(
			async () => {
				const summary = await getCanvasContentSummary(ctx);
				return (summary?.nonBackgroundSamples ?? 0) > 0;
			},
			"detail canvas renders non-background content",
			10_000,
		);

		await waitForCondition(
			async () => {
				const creatures = await listVisibleCreatures(ctx);
				return creatures.length > 0;
			},
			"detail viewport has visible creatures",
			10_000,
		);

		const creatures = await listVisibleCreatures(ctx);
		const target = creatures[0];
		assert(target !== undefined, "detail viewport should expose at least one creature");

		const detailClickPoint = await getCanvasPointForWorld(ctx, target.x, target.y);
		assert(detailClickPoint !== null, "target creature should map to a canvas click point");

		await ctx.browser.clickAt(detailClickPoint.clientX, detailClickPoint.clientY);

		await waitForCondition(
			async () => {
				return (await getSelectedCreatureId(ctx)) !== null;
			},
			"creature selection after detail click",
			10_000,
		);
		await waitForCondition(
			async () => {
				return (await ctx.browser.isVisible(selectors.creatureInspector)).visible;
			},
			"creature inspector visible after detail click",
			10_000,
		);

		await ctx.browser.click(selectors.zoomFit);
		await waitForCondition(
			async () => {
				const viewport = await readViewportState(ctx);
				return viewport.camera.zoom < DETAIL_ZOOM_THRESHOLD;
			},
			"returned to overview tier",
			10_000,
		);
		await waitForCondition(
			async () => {
				const worldView = await readWorldViewState(ctx);
				return worldView.viewKind === "overview";
			},
			"overview payload delivered after zoom fit",
			10_000,
		);

		const overviewClickPoint = await getCanvasPointForWorld(ctx, target.x, target.y);
		assert(
			overviewClickPoint !== null,
			"target world position should map to an overview click point",
		);

		await ctx.browser.clickAt(overviewClickPoint.clientX, overviewClickPoint.clientY);

		await waitForCondition(
			async () => {
				return (await getSelectedCreatureId(ctx)) === null;
			},
			"overview click clears selection",
			10_000,
		);
		await waitForCondition(
			async () => {
				return !(await ctx.browser.isVisible(selectors.creatureInspector)).visible;
			},
			"creature inspector hidden in overview after click",
			10_000,
		);

		const errors = await ctx.browser.errors();
		assert(
			errors.errors.length === 0,
			`expected no browser errors, got: ${JSON.stringify(errors.errors)}`,
		);
	},
};

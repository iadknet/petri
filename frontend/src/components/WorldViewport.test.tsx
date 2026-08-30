import { act, render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { usePaintStore } from "../stores/paint.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import type { Frame } from "../types/api.ts";
import { WorldViewport } from "./WorldViewport.tsx";

const rendererSpies = vi.hoisted(() => ({
	fitToWorld: vi.fn((width: number, height: number) => ({
		x: -width,
		y: -height,
		zoom: 1,
	})),
	invalidate: vi.fn(),
	resize: vi.fn(),
	start: vi.fn(),
	stop: vi.fn(),
}));

vi.mock("../canvas/renderer.ts", () => ({
	WorldRenderer: class {
		start() {
			rendererSpies.start();
		}

		stop() {
			rendererSpies.stop();
		}

		resize(width: number, height: number) {
			rendererSpies.resize(width, height);
		}

		fitToWorld(width: number, height: number) {
			return rendererSpies.fitToWorld(width, height);
		}

		invalidate() {
			rendererSpies.invalidate();
		}
	},
}));

function frame(width: number, height: number): Frame {
	return { width, height, creatures: [], food: [], barriers: [] };
}

describe("WorldViewport frame sizing", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
		useViewportStore.getState().reset();
		useWorldViewStore.getState().reset();
		usePaintStore.getState().setPaintMode(false);
		vi.clearAllMocks();
	});

	it("refits when a same-tick restart replaces a 1600 world with a 512 world", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useSimulationStore.getState().setTick(42);
			useWorldViewStore.setState({ frame: frame(1600, 1600) });
		});
		const userCamera = { x: -320, y: -180, zoom: 4 };
		act(() => {
			useViewportStore.getState().setCamera(userCamera);
		});
		rendererSpies.fitToWorld.mockClear();

		// Act
		act(() => {
			useSimulationStore.getState().setTick(42);
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});

		// Assert
		expect(rendererSpies.fitToWorld).toHaveBeenCalledWith(512, 512);
		expect(useViewportStore.getState().camera).toEqual({ x: -512, y: -512, zoom: 1 });
	});

	it("preserves user pan and zoom for same-dimension frame updates", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});
		const userCamera = { x: -320, y: -180, zoom: 4 };
		act(() => {
			useViewportStore.getState().setCamera(userCamera);
		});
		rendererSpies.fitToWorld.mockClear();
		rendererSpies.invalidate.mockClear();

		// Act
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});

		// Assert
		expect(rendererSpies.fitToWorld).not.toHaveBeenCalled();
		expect(useViewportStore.getState().camera).toEqual(userCamera);
		expect(rendererSpies.invalidate).toHaveBeenCalledTimes(1);
	});
});

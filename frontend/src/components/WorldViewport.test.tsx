import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { usePaintStore } from "../stores/paint.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import type { Frame } from "../types/api.ts";
import { StatsPanel } from "./StatsPanel.tsx";
import { WorldViewport } from "./WorldViewport.tsx";

vi.mock("./stats/OverviewTab.tsx", () => ({ OverviewTab: () => <p>overview body</p> }));
vi.mock("./stats/ActionsTab.tsx", () => ({ ActionsTab: () => <p>actions body</p> }));
vi.mock("./stats/EvolutionTab.tsx", () => ({ EvolutionTab: () => <p>evolution body</p> }));
vi.mock("./stats/ComputationTab.tsx", () => ({ ComputationTab: () => <p>computation body</p> }));
vi.mock("./stats/PredationTab.tsx", () => ({ PredationTab: () => <p>predation body</p> }));

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
	resetView: vi.fn((_frame: { width: number; height: number }) => ({ x: 0, y: 0, zoom: 1 })),
	pan: vi.fn((camera: { x: number; y: number; zoom: number }, dx: number, dy: number) => ({
		x: camera.x + dx,
		y: camera.y + dy,
		zoom: camera.zoom,
	})),
	zoomCenterByFactor: vi.fn((camera: { x: number; y: number; zoom: number }, factor: number) => ({
		...camera,
		zoom: camera.zoom * factor,
	})),
	centerOn: vi.fn((worldX: number, worldY: number, zoom?: number) => ({
		x: -worldX,
		y: -worldY,
		zoom: zoom ?? 1,
	})),
	setSelectedCreature: vi.fn(),
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

		resetView(f: { width: number; height: number }) {
			return rendererSpies.resetView(f);
		}

		pan(camera: { x: number; y: number; zoom: number }, dx: number, dy: number) {
			return rendererSpies.pan(camera, dx, dy);
		}

		zoomCenterByFactor(camera: { x: number; y: number; zoom: number }, factor: number) {
			return rendererSpies.zoomCenterByFactor(camera, factor);
		}

		centerOn(worldX: number, worldY: number, zoom?: number) {
			return rendererSpies.centerOn(worldX, worldY, zoom);
		}

		setSelectedCreature(id: number | null) {
			rendererSpies.setSelectedCreature(id);
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

	it("keeps the user's zoom and view center when the viewport resizes", () => {
		// Arrange
		const observers: ResizeObserverCallback[] = [];
		const OriginalResizeObserver = globalThis.ResizeObserver;
		globalThis.ResizeObserver = class {
			constructor(callback: ResizeObserverCallback) {
				observers.push(callback);
			}
			observe() {}
			unobserve() {}
			disconnect() {}
		} as unknown as typeof ResizeObserver;
		const resizeTo = (width: number, height: number) =>
			act(() => {
				for (const callback of observers) {
					callback(
						[{ contentRect: { width, height } } as ResizeObserverEntry],
						{} as ResizeObserver,
					);
				}
			});

		try {
			render(<WorldViewport />);
			act(() => {
				useWorldViewStore.setState({ frame: frame(512, 512) });
			});
			resizeTo(400, 200);
			act(() => {
				useViewportStore.getState().setCamera({ x: -100, y: -50, zoom: 2 });
			});
			rendererSpies.fitToWorld.mockClear();

			// Act
			resizeTo(600, 300);

			// Assert
			expect(rendererSpies.fitToWorld).not.toHaveBeenCalled();
			expect(useViewportStore.getState().camera).toEqual({ x: 0, y: 0, zoom: 2 });
		} finally {
			globalThis.ResizeObserver = OriginalResizeObserver;
		}
	});
});

function creature(id: number, x: number, y: number): Frame["creatures"][number] {
	return { id, x, y } as Frame["creatures"][number];
}

function selectedId() {
	return creatureInspectorSelectors.selectedCreatureId(useCreatureInspectorStore.getState());
}

describe("WorldViewport keyboard access", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
		useViewportStore.getState().reset();
		useWorldViewStore.getState().reset();
		useCreatureInspectorStore.getState().clearSelection();
		usePaintStore.getState().setPaintMode(false);
		vi.clearAllMocks();
	});

	it("Escape in a text input does not deselect", () => {
		// Arrange
		render(
			<>
				<input type="text" aria-label="Search settings" />
				<WorldViewport />
			</>,
		);
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(7);
		});

		// Act
		fireEvent.keyDown(screen.getByLabelText("Search settings"), { key: "Escape" });

		// Assert
		expect(selectedId()).toBe(7);
	});

	it("Home in a number input does not reset the camera", () => {
		// Arrange
		render(
			<>
				<input type="number" aria-label="Seed" />
				<WorldViewport />
			</>,
		);
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});
		const userCamera = { x: -320, y: -180, zoom: 4 };
		act(() => {
			useViewportStore.getState().setCamera(userCamera);
		});

		// Act
		fireEvent.keyDown(screen.getByLabelText("Seed"), { key: "Home" });

		// Assert
		expect(rendererSpies.resetView).not.toHaveBeenCalled();
		expect(useViewportStore.getState().camera).toEqual(userCamera);
	});

	it("Home on the focused stats resize handle resizes the dock without resetting the camera", () => {
		// Arrange
		render(
			<>
				<StatsPanel />
				<WorldViewport />
			</>,
		);
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});
		const userCamera = { x: -320, y: -180, zoom: 4 };
		act(() => {
			useViewportStore.getState().setCamera(userCamera);
		});
		const handle = screen.getByRole("separator", { name: "Resize statistics" });
		handle.focus();

		// Act
		fireEvent.keyDown(handle, { key: "Home" });

		// Assert
		expect(handle).toHaveAttribute("aria-valuenow", "160");
		expect(rendererSpies.resetView).not.toHaveBeenCalled();
		expect(useViewportStore.getState().camera).toEqual(userCamera);
	});

	it("ignores modified Home and Escape so browser and system shortcuts pass through", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
			useCreatureInspectorStore.getState().selectCreature(7);
		});

		// Act
		for (const modifier of ["ctrlKey", "metaKey", "altKey"]) {
			fireEvent.keyDown(document.body, { key: "Home", [modifier]: true });
			fireEvent.keyDown(document.body, { key: "Escape", [modifier]: true });
		}

		// Assert
		expect(rendererSpies.resetView).not.toHaveBeenCalled();
		expect(selectedId()).toBe(7);
	});

	it("still clears the selection on Escape outside editable fields", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(7);
		});

		// Act
		fireEvent.keyDown(document.body, { key: "Escape" });

		// Assert
		expect(selectedId()).toBeNull();
	});

	it("exposes the map canvas as a focusable, named keyboard widget", () => {
		// Arrange / Act
		render(<WorldViewport />);
		const canvas = screen.getByTestId("world-canvas");

		// Assert
		expect(canvas).toHaveAttribute("tabindex", "0");
		expect(canvas).toHaveAttribute("role", "application");
		expect(canvas).toHaveAccessibleName(/world map/i);
		expect(canvas).toHaveAccessibleDescription(/arrow keys/i);
	});

	it("pans with arrow keys so ArrowRight reveals what lies to the right", () => {
		// Arrange
		render(<WorldViewport />);
		const start = { x: -100, y: -100, zoom: 2 };
		act(() => {
			useViewportStore.getState().setCamera(start);
		});
		const canvas = screen.getByTestId("world-canvas");

		// Act
		fireEvent.keyDown(canvas, { key: "ArrowRight" });
		const afterRight = useViewportStore.getState().camera;
		fireEvent.keyDown(canvas, { key: "ArrowDown" });
		const afterDown = useViewportStore.getState().camera;

		// Assert
		expect(afterRight.x).toBeLessThan(start.x);
		expect(afterRight.y).toBe(start.y);
		expect(afterDown.y).toBeLessThan(start.y);
	});

	it("zooms with + and - and fits with 0", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useWorldViewStore.setState({ frame: frame(512, 512) });
		});
		act(() => {
			useViewportStore.getState().setCamera({ x: -10, y: -10, zoom: 2 });
		});
		const canvas = screen.getByTestId("world-canvas");

		// Act / Assert
		fireEvent.keyDown(canvas, { key: "+" });
		expect(useViewportStore.getState().camera.zoom).toBeCloseTo(2.9);
		fireEvent.keyDown(canvas, { key: "-" });
		expect(useViewportStore.getState().camera.zoom).toBeCloseTo(2);
		fireEvent.keyDown(canvas, { key: "0" });
		expect(rendererSpies.resetView).toHaveBeenCalledTimes(1);
		expect(useViewportStore.getState().camera).toEqual({ x: 0, y: 0, zoom: 1 });
	});

	it("leaves browser zoom shortcuts alone", () => {
		// Arrange
		render(<WorldViewport />);
		const start = { x: -10, y: -10, zoom: 2 };
		act(() => {
			useViewportStore.getState().setCamera(start);
		});

		// Act
		fireEvent.keyDown(screen.getByTestId("world-canvas"), { key: "+", metaKey: true });

		// Assert
		expect(useViewportStore.getState().camera).toEqual(start);
	});

	it("cycles creature selection with n and p and centers on the selected creature", () => {
		// Arrange
		render(<WorldViewport />);
		act(() => {
			useWorldViewStore.setState({
				frame: { ...frame(512, 512), creatures: [creature(9, 30, 40), creature(4, 10, 20)] },
			});
		});
		act(() => {
			useViewportStore.getState().setCamera({ x: 0, y: 0, zoom: 3 });
		});
		const canvas = screen.getByTestId("world-canvas");

		// Act / Assert
		fireEvent.keyDown(canvas, { key: "n" });
		expect(selectedId()).toBe(4);
		expect(useViewportStore.getState().camera).toEqual({ x: -10, y: -20, zoom: 3 });
		fireEvent.keyDown(canvas, { key: "n" });
		expect(selectedId()).toBe(9);
		fireEvent.keyDown(canvas, { key: "n" });
		expect(selectedId()).toBe(4);
		fireEvent.keyDown(canvas, { key: "p" });
		expect(selectedId()).toBe(9);
		fireEvent.keyDown(canvas, { key: "Escape" });
		expect(selectedId()).toBeNull();
	});
});

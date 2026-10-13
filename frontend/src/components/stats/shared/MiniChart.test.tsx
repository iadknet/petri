import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MiniChart, chartPoints } from "./MiniChart.tsx";

type Call = [string, number, number];

let calls: Call[];
let observers: { cb: ResizeObserverCallback; el: Element | null }[];
const originalRO = globalThis.ResizeObserver;
const originalDpr = window.devicePixelRatio;

function fakeContext() {
	return {
		setTransform: () => {},
		clearRect: () => {},
		beginPath: () => {},
		stroke: () => {},
		moveTo: (x: number, y: number) => calls.push(["moveTo", x, y]),
		lineTo: (x: number, y: number) => calls.push(["lineTo", x, y]),
		strokeStyle: "",
		lineWidth: 0,
	};
}

function resizeTo(width: number) {
	act(() => {
		for (const o of observers) {
			o.cb(
				[{ target: o.el, contentRect: { width } } as unknown as ResizeObserverEntry],
				{} as ResizeObserver,
			);
		}
	});
}

const series = (values: number[], firstTick = 10) =>
	values.map((value, i) => ({ tick: firstTick + i, value }));

beforeEach(() => {
	calls = [];
	observers = [];
	vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(
		() => fakeContext() as unknown as CanvasRenderingContext2D,
	);
	globalThis.ResizeObserver = class {
		private entry: { cb: ResizeObserverCallback; el: Element | null };
		constructor(cb: ResizeObserverCallback) {
			this.entry = { cb, el: null };
			observers.push(this.entry);
		}
		observe(el: Element) {
			this.entry.el = el;
		}
		unobserve() {}
		disconnect() {}
	} as unknown as typeof ResizeObserver;
});

afterEach(() => {
	vi.restoreAllMocks();
	globalThis.ResizeObserver = originalRO;
	Object.defineProperty(window, "devicePixelRatio", { value: originalDpr, configurable: true });
});

describe("chartPoints", () => {
	it("maps the minimum to the bottom inset and the maximum to the top inset", () => {
		const pts = chartPoints([5, 10, 15], 200, 60);
		expect(pts.map((p) => p.x)).toEqual([0, 100, 200]);
		expect(pts[0]!.y).toBe(58);
		expect(pts[2]!.y).toBe(2);
		expect(pts[1]!.y).toBe(30);
	});

	it("draws a constant series through the vertical centre, not the bottom edge", () => {
		const pts = chartPoints([7, 7, 7], 200, 60);
		expect(pts.every((p) => p.y === 30)).toBe(true);
	});
});

describe("MiniChart", () => {
	it("labels the scale with the minimum, maximum and tick span", () => {
		render(<MiniChart data={series([4, 12.5, 9])} color="#fff" />);
		expect(screen.getByText("max 12.5")).toBeInTheDocument();
		expect(screen.getByText("min 4")).toBeInTheDocument();
		expect(screen.getByText("ticks 10–12")).toBeInTheDocument();
	});

	it("labels a constant series as no change with its value", () => {
		render(<MiniChart data={series([0, 0, 0, 0])} color="#fff" />);
		expect(screen.getByText("no change: 0")).toBeInTheDocument();
		expect(screen.queryByText(/^max /)).not.toBeInTheDocument();
	});

	it("gives the canvas an accessible summary", () => {
		render(<MiniChart data={series([4, 12.5, 9])} color="#fff" />);
		expect(screen.getByRole("img")).toHaveAccessibleName("Range 4 to 12.5 over ticks 10–12");
	});

	it("sizes the backing store to the rendered width times the device pixel ratio", () => {
		Object.defineProperty(window, "devicePixelRatio", { value: 2, configurable: true });
		render(<MiniChart data={series([1, 2, 3])} color="#fff" height={60} />);
		resizeTo(400);
		const canvas = screen.getByRole("img") as HTMLCanvasElement;
		expect(canvas.width).toBe(800);
		expect(canvas.height).toBe(120);
		expect(canvas.style.height).toBe("60px");
	});

	it("draws in CSS pixels across the full measured width, centring a constant series", () => {
		render(<MiniChart data={series([3, 3, 3])} color="#fff" height={60} />);
		resizeTo(500);
		const last = calls[calls.length - 1]!;
		expect(last[1]).toBe(500);
		expect(calls.every((c) => c[2] === 30)).toBe(true);
	});
});

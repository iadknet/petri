import { describe, it, expect } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { ActionTimeline, directionLabel, energyPath, tickLabels } from "./ActionTimeline.tsx";
import { ActionType, ActionResult } from "../../types/action-log.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";

function makeEntry(overrides: Partial<ActionLogEntry> = {}): ActionLogEntry {
	return {
		tick: 1,
		action_type: ActionType.Move,
		result: ActionResult.Success,
		direction: 2,
		energy_before: 100,
		energy_after: 95,
		amount: 0,
		priority_bid: 0.5,
		...overrides,
	};
}

describe("ActionTimeline", () => {
	it("renders the section header", () => {
		render(<ActionTimeline actionLog={[]} maxEnergy={200} />);
		expect(screen.getByText("Action Timeline")).toBeDefined();
	});

	it("renders one segment per entry", () => {
		const entries = [makeEntry({ tick: 1 }), makeEntry({ tick: 2 }), makeEntry({ tick: 3 })];
		const { container } = render(<ActionTimeline actionLog={entries} maxEnergy={200} />);
		const segments = container.querySelectorAll("[title]");
		expect(segments).toHaveLength(3);
	});

	it("colors Move segments blue", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry({ action_type: ActionType.Move })]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.backgroundColor).toBe("rgb(96, 165, 250)");
	});

	it("colors Eat segments green", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry({ action_type: ActionType.Eat })]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.backgroundColor).toBe("rgb(52, 211, 153)");
	});

	it("colors Reproduce segments amber", () => {
		const { container } = render(
			<ActionTimeline
				actionLog={[makeEntry({ action_type: ActionType.Reproduce })]}
				maxEnergy={200}
			/>,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.backgroundColor).toBe("rgb(251, 191, 36)");
	});

	it("colors StealEnergy segments red", () => {
		const { container } = render(
			<ActionTimeline
				actionLog={[makeEntry({ action_type: ActionType.StealEnergy })]}
				maxEnergy={200}
			/>,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.backgroundColor).toBe("rgb(248, 113, 113)");
	});

	it("colors NoOp segments gray", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry({ action_type: ActionType.NoOp })]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.backgroundColor).toBe("rgb(71, 85, 105)");
	});

	it("shows red top border on failed actions", () => {
		const { container } = render(
			<ActionTimeline
				actionLog={[makeEntry({ result: ActionResult.Blocked })]}
				maxEnergy={200}
			/>,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.borderTop).toMatch(/2px solid (rgb\(239, 68, 68\)|#ef4444)/);
	});

	it("has no red top border on successful actions", () => {
		const { container } = render(
			<ActionTimeline
				actionLog={[makeEntry({ result: ActionResult.Success })]}
				maxEnergy={200}
			/>,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.borderTop).toBe("");
	});

	it("sets segment width to 6px", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry()]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		expect(seg.style.width).toBe("6px");
		expect(seg.style.minWidth).toBe("6px");
	});
});

describe("ActionDetail", () => {
	it("shows detail panel when a segment is clicked", () => {
		const entry = makeEntry({
			tick: 42,
			action_type: ActionType.Eat,
			result: ActionResult.Success,
			direction: 3,
			energy_before: 100,
			energy_after: 110,
			amount: 10,
			priority_bid: 0.75,
		});
		const { container } = render(<ActionTimeline actionLog={[entry]} maxEnergy={200} />);
		const seg = container.querySelector("[title]") as HTMLElement;
		fireEvent.click(seg);
		const detail = screen.getByTestId("action-detail");
		expect(detail).toBeDefined();
		expect(detail.textContent).toContain("42");
		expect(detail.textContent).toContain("Eat");
		expect(detail.textContent).toContain("Success");
		expect(detail.textContent).toContain("SE");
		expect(detail.textContent).toContain("100.0");
		expect(detail.textContent).toContain("110.0");
		expect(detail.textContent).toContain("+10.0");
		expect(detail.textContent).toContain("10.0");
		expect(detail.textContent).toContain("0.75");
	});

	it("hides detail panel when same segment is clicked again", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry()]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		fireEvent.click(seg);
		expect(screen.getByTestId("action-detail")).toBeDefined();
		fireEvent.click(seg);
		expect(screen.queryByTestId("action-detail")).toBeNull();
	});

	it("hides amount row when amount is zero", () => {
		const { container } = render(
			<ActionTimeline actionLog={[makeEntry({ amount: 0 })]} maxEnergy={200} />,
		);
		const seg = container.querySelector("[title]") as HTMLElement;
		fireEvent.click(seg);
		const detail = screen.getByTestId("action-detail");
		expect(detail.textContent).not.toContain("Amount");
	});

	it("shows negative energy delta in red", () => {
		const entry = makeEntry({ energy_before: 100, energy_after: 90 });
		const { container } = render(<ActionTimeline actionLog={[entry]} maxEnergy={200} />);
		const seg = container.querySelector("[title]") as HTMLElement;
		fireEvent.click(seg);
		const detail = screen.getByTestId("action-detail");
		expect(detail.textContent).toContain("-10.0");
	});
});

describe("TickAxis", () => {
	it("generates labels at tick multiples of 50", () => {
		const entries = Array.from({ length: 100 }, (_, i) => makeEntry({ tick: i + 1 }));
		const labels = tickLabels(entries);
		expect(labels.map((l) => l.tick)).toEqual([50, 100]);
	});

	it("positions labels based on entry index", () => {
		const entries = Array.from({ length: 100 }, (_, i) => makeEntry({ tick: i + 1 }));
		const labels = tickLabels(entries);
		// tick 50 is at index 49 => 49 * 6 = 294px
		expect(labels[0]?.offsetPx).toBe(294);
		// tick 100 is at index 99 => 99 * 6 = 594px
		expect(labels[1]?.offsetPx).toBe(594);
	});

	it("returns empty array when no ticks are multiples of 50", () => {
		const entries = [makeEntry({ tick: 1 }), makeEntry({ tick: 2 })];
		const labels = tickLabels(entries);
		expect(labels).toHaveLength(0);
	});

	it("renders tick labels in the DOM", () => {
		const entries = Array.from({ length: 60 }, (_, i) => makeEntry({ tick: i + 1 }));
		render(<ActionTimeline actionLog={entries} maxEnergy={200} />);
		expect(screen.getByText("50")).toBeDefined();
	});
});

describe("EnergyOverlay", () => {
	it("generates SVG path from energy data", () => {
		const entries = [
			makeEntry({ energy_after: 100 }),
			makeEntry({ energy_after: 50 }),
			makeEntry({ energy_after: 0 }),
		];
		const path = energyPath(entries, 100, 20);
		// entry 0: x = (0*6 + 3) = 3.0, y = 20*(1 - 100/100) = 0.0
		// entry 1: x = (1*6 + 3) = 9.0, y = 20*(1 - 50/100) = 10.0
		// entry 2: x = (2*6 + 3) = 15.0, y = 20*(1 - 0/100) = 20.0
		expect(path).toBe("M3.0,0.0L9.0,10.0L15.0,20.0");
	});

	it("returns empty string for empty entries", () => {
		expect(energyPath([], 100, 20)).toBe("");
	});

	it("returns empty string for zero maxEnergy", () => {
		expect(energyPath([makeEntry()], 0, 20)).toBe("");
	});

	it("clamps energy ratio to 1.0", () => {
		const entries = [makeEntry({ energy_after: 200 })];
		const path = energyPath(entries, 100, 20);
		// ratio clamped to 1.0, y = 20*(1-1) = 0.0
		expect(path).toBe("M3.0,0.0");
	});

	it("rounds coordinates to 1 decimal place", () => {
		const entries = [makeEntry({ energy_after: 33 })];
		const path = energyPath(entries, 100, 20);
		// y = 20 * (1 - 0.33) = 13.4
		expect(path).toBe("M3.0,13.4");
	});

	it("renders SVG element in the DOM", () => {
		const entries = [makeEntry({ energy_after: 50 })];
		render(<ActionTimeline actionLog={entries} maxEnergy={100} />);
		const svg = screen.getByTestId("energy-overlay");
		expect(svg).toBeDefined();
		expect(svg.querySelector("path")).toBeDefined();
	});
});

describe("directionLabel", () => {
	it("maps 0-7 to compass directions", () => {
		expect(directionLabel(0)).toBe("N");
		expect(directionLabel(1)).toBe("NE");
		expect(directionLabel(2)).toBe("E");
		expect(directionLabel(3)).toBe("SE");
		expect(directionLabel(4)).toBe("S");
		expect(directionLabel(5)).toBe("SW");
		expect(directionLabel(6)).toBe("W");
		expect(directionLabel(7)).toBe("NW");
	});

	it("returns N/A for 255", () => {
		expect(directionLabel(255)).toBe("N/A");
	});

	it("returns number string for unknown values", () => {
		expect(directionLabel(99)).toBe("99");
	});
});

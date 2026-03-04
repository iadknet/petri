import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ActionTimeline } from "./ActionTimeline.tsx";
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

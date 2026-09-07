import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ActionResult, ActionType } from "../../types/action-log.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";
import type { CreaturePhenotype } from "../../types/genome.ts";
import { VitalsBanner } from "./VitalsBanner.tsx";

vi.mock("./PhenotypeDetail.tsx", () => ({
	PhenotypeDetail: () => <div data-testid="phenotype-detail">PhenotypeDetail</div>,
}));

function makeEntry(overrides: Partial<ActionLogEntry> = {}): ActionLogEntry {
	return {
		tick: 1,
		action_type: ActionType.Move,
		result: ActionResult.Success,
		direction: 2,
		energy_before: 100,
		energy_after: 95,
		amount: 0,
		food_type: null,
		priority_bid: 0.5,
		...overrides,
	};
}

const defaultProps = {
	id: 42,
	rgb: [120, 200, 80] as [number, number, number],
	generation: 5,
	age: 150,
	energy: 62,
	maxEnergy: 100,
	position: { x: 10, y: 25 },
	actionLog: null as ActionLogEntry[] | null,
	isDead: false,
	onClose: vi.fn(),
};

function makePhenotype(): CreaturePhenotype {
	return {
		channels: [100, 150, 200, 50, 75, 125],
		active_channel: 0,
		polarity: [true, false, true, false, true, false],
		rgb: [120, 200, 80],
	};
}

describe("VitalsBanner", () => {
	it("renders creature ID and stats", () => {
		render(<VitalsBanner {...defaultProps} />);
		expect(screen.getByText(/^#42$/)).toBeDefined();
		expect(screen.getByText(/Gen 5/)).toBeDefined();
		expect(screen.getByText(/Age 150/)).toBeDefined();
	});

	it("renders color swatch with correct rgb", () => {
		const { container } = render(<VitalsBanner {...defaultProps} />);
		const swatch = container.querySelector("[data-testid='color-swatch']") as HTMLElement;
		expect(swatch).not.toBeNull();
		expect(swatch.style.backgroundColor).toBe("rgb(120, 200, 80)");
	});

	it("renders energy bar with green color when above 30%", () => {
		const { container } = render(<VitalsBanner {...defaultProps} energy={62} maxEnergy={100} />);
		const bar = container.querySelector("[data-testid='energy-bar-fill']") as HTMLElement;
		expect(bar).not.toBeNull();
		expect(bar.className).toContain("bg-emerald-500");
	});

	it("renders energy bar with red color when at or below 30%", () => {
		const { container } = render(<VitalsBanner {...defaultProps} energy={30} maxEnergy={100} />);
		const bar = container.querySelector("[data-testid='energy-bar-fill']") as HTMLElement;
		expect(bar).not.toBeNull();
		expect(bar.className).toContain("bg-red-500");
	});

	it("omits the removed reproductive reserve", () => {
		const { container } = render(<VitalsBanner {...defaultProps} />);
		expect(screen.queryByText(/Reserve/)).toBeNull();
		expect(container.querySelector("[data-testid='reserve-bar-fill']")).toBeNull();
	});

	it("renders dead banner when isDead is true", () => {
		render(<VitalsBanner {...defaultProps} isDead={true} />);
		expect(screen.getByText(/dead/i)).toBeDefined();
	});

	it("does not render dead banner when alive", () => {
		render(<VitalsBanner {...defaultProps} isDead={false} />);
		expect(screen.queryByText(/dead/i)).toBeNull();
	});

	it("calls onClose when close button clicked", () => {
		const onClose = vi.fn();
		render(<VitalsBanner {...defaultProps} onClose={onClose} />);
		const closeButton = screen.getByRole("button", { name: /close/i });
		fireEvent.click(closeButton);
		expect(onClose).toHaveBeenCalledOnce();
	});

	it("renders action dots from actionLog", () => {
		const entries = Array.from({ length: 55 }, (_, i) =>
			makeEntry({
				tick: i + 1,
				action_type: i % 2 === 0 ? ActionType.Move : ActionType.Eat,
			}),
		);
		const { container } = render(<VitalsBanner {...defaultProps} actionLog={entries} />);
		const dots = container.querySelectorAll("[data-testid='action-dot']");
		// Should show last 50 actions
		expect(dots.length).toBe(50);
	});

	it("toggles phenotype drawer on click", () => {
		render(<VitalsBanner {...defaultProps} phenotype={makePhenotype()} />);
		// PhenotypeDetail should not be visible initially
		expect(screen.queryByTestId("phenotype-detail")).toBeNull();

		// Click the disclosure toggle
		const toggle = screen.getByRole("button", { name: /phenotype/i });
		fireEvent.click(toggle);

		// PhenotypeDetail should now be visible
		expect(screen.getByTestId("phenotype-detail")).toBeDefined();

		// Click again to hide
		fireEvent.click(toggle);
		expect(screen.queryByTestId("phenotype-detail")).toBeNull();
	});
});

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { StatsPanel } from "./StatsPanel.tsx";

vi.mock("./stats/OverviewTab.tsx", () => ({ OverviewTab: () => <p>overview body</p> }));
vi.mock("./stats/ActionsTab.tsx", () => ({ ActionsTab: () => <p>actions body</p> }));
vi.mock("./stats/EvolutionTab.tsx", () => ({ EvolutionTab: () => <p>evolution body</p> }));
vi.mock("./stats/ComputationTab.tsx", () => ({ ComputationTab: () => <p>computation body</p> }));
vi.mock("./stats/PredationTab.tsx", () => ({ PredationTab: () => <p>predation body</p> }));

function selectedTab() {
	return screen.getByRole("tab", { selected: true });
}

describe("StatsPanel tabs", () => {
	it("renders a labelled region with a level-2 heading", () => {
		render(<StatsPanel />);
		const heading = screen.getByRole("heading", { level: 2, name: "Statistics" });
		const region = screen.getByRole("region", { name: "Statistics" });
		expect(region).toContainElement(heading);
	});

	it("exposes a tablist whose tabs control labelled tabpanels", () => {
		render(<StatsPanel />);
		expect(screen.getByRole("tablist")).toHaveAccessibleName();
		const tabs = screen.getAllByRole("tab");
		expect(tabs).toHaveLength(5);
		expect(selectedTab()).toHaveTextContent("Overview");
		expect(tabs.filter((t) => t.tabIndex === 0)).toEqual([selectedTab()]);
		for (const tab of tabs) {
			const panelId = tab.getAttribute("aria-controls");
			expect(panelId).toBeTruthy();
			const panel = document.getElementById(panelId as string);
			expect(panel).toHaveAttribute("role", "tabpanel");
			expect(panel).toHaveAttribute("aria-labelledby", tab.id);
		}
		const visible = screen.getByRole("tabpanel");
		expect(visible).toHaveAttribute("aria-labelledby", selectedTab().id);
		expect(visible).toHaveTextContent("overview body");
	});

	it("marks the active tab with more than colour", () => {
		render(<StatsPanel />);
		expect(selectedTab().className).toContain("border-sky-400");
		for (const tab of screen.getAllByRole("tab", { selected: false })) {
			expect(tab.className).not.toContain("border-sky-400");
		}
	});

	it("moves selection and focus with arrow, Home and End keys", () => {
		render(<StatsPanel />);
		fireEvent.keyDown(selectedTab(), { key: "ArrowRight" });
		expect(selectedTab()).toHaveTextContent("Actions");
		expect(selectedTab()).toHaveFocus();
		expect(screen.getByRole("tabpanel")).toHaveTextContent("actions body");

		fireEvent.keyDown(selectedTab(), { key: "ArrowLeft" });
		fireEvent.keyDown(selectedTab(), { key: "ArrowLeft" });
		expect(selectedTab()).toHaveTextContent("Predation");
		expect(selectedTab()).toHaveFocus();

		fireEvent.keyDown(selectedTab(), { key: "ArrowRight" });
		expect(selectedTab()).toHaveTextContent("Overview");

		fireEvent.keyDown(selectedTab(), { key: "End" });
		expect(selectedTab()).toHaveTextContent("Predation");
		expect(selectedTab()).toHaveFocus();

		fireEvent.keyDown(selectedTab(), { key: "Home" });
		expect(selectedTab()).toHaveTextContent("Overview");
		expect(selectedTab()).toHaveFocus();
		expect(selectedTab().tabIndex).toBe(0);
	});

	it("selects a tab on click", () => {
		render(<StatsPanel />);
		fireEvent.click(screen.getByRole("tab", { name: "Evolution" }));
		expect(selectedTab()).toHaveTextContent("Evolution");
		expect(screen.getByRole("tabpanel")).toHaveTextContent("evolution body");
	});
});

import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useStatsHistoryStore } from "../stores/stats.ts";
import { MOCK_HEALTH_CENSUS } from "../test/fixtures.ts";
import type { HealthPayload } from "../types/api.ts";
import { StatsPanel } from "./StatsPanel.tsx";

function health(overrides: Partial<HealthPayload>): HealthPayload {
	return {
		...MOCK_HEALTH_CENSUS,
		population: 100,
		mean_energy: 20,
		mutation_events_attempted_total: 0,
		mutation_events_applied_total: 0,
		mutation_events_skipped_total: 0,
		mutation_events_attempted_total_by_domain: {},
		mutation_events_applied_total_by_domain: {},
		mutation_events_attempted_total_by_operator: {},
		mutation_events_applied_total_by_operator: {},
		reproduction_actions_attempted_total: 0,
		reproduction_actions_spawned_total: 0,
		reproduction_actions_rejected_total: 0,
		reproduction_actions_rejected_total_by_reason: {},
		mutation_events_skipped_total_by_reason: {},
		predation_actions_attempted_total: 0,
		predation_actions_transferred_total: 0,
		predation_actions_rejected_total: 0,
		predation_kills_total: 0,
		predation_actions_by_result: {},
		genome_complexity_mean: 0,
		genome_complexity_min: 0,
		genome_complexity_max: 0,
		...overrides,
	};
}

function seedRun() {
	const store = useStatsHistoryStore.getState();
	store.recordHealth(0, health({ population: 200 }));
	store.recordHealth(
		10,
		health({
			population: 150,
			reproduction_actions_spawned_total: 10,
			deaths_total: 60,
			deaths_total_by_cause: { action_move: 45, predation: 15 },
			reproduction_actions_rejected_total_by_reason: { RejectedEnergyConstraints: 7 },
		}),
	);
}

describe("StatsPanel", () => {
	beforeEach(() => {
		useStatsHistoryStore.getState().reset();
		const map = new Map<string, string>();
		Object.defineProperty(window, "localStorage", {
			configurable: true,
			value: {
				getItem: (k: string) => map.get(k) ?? null,
				setItem: (k: string, v: string) => map.set(k, v),
				removeItem: (k: string) => map.delete(k),
				clear: () => map.clear(),
			},
		});
	});

	it("opens on the Mechanics tab", () => {
		seedRun();
		render(<StatsPanel />);
		expect(screen.getByRole("tab", { name: /Mechanics/ })).toHaveAttribute("aria-selected", "true");
		const panel = screen.getByRole("tabpanel");
		expect(within(panel).getByText("Moving")).toBeInTheDocument();
		expect(within(panel).getByText("Killed by a predator")).toBeInTheDocument();
		expect(within(panel).getByText("Not enough energy")).toBeInTheDocument();
	});

	it("labels each band tab with one verdict word and keeps the sentence in its tooltip", () => {
		seedRun();
		render(<StatsPanel />);
		const world = screen.getByRole("tab", { name: /World/ });
		expect(within(world).getByText("Shrinking")).toBeInTheDocument();
		expect(world).toHaveAttribute("title", expect.stringMatching(/1\.00 born vs 6\.00 died/));
	});

	it("shows a band's readings with value, signed change and a better/worse word", () => {
		seedRun();
		render(<StatsPanel />);
		fireEvent.click(screen.getByRole("tab", { name: /World/ }));
		const reading = screen.getByRole("group", { name: "Creatures alive" });
		expect(within(reading).getByTestId("reading-value")).toHaveTextContent("150");
		expect(within(reading).getByText("\u221250")).toBeInTheDocument();
		expect(within(reading).getByText("worse")).toBeInTheDocument();
	});

	it("moves between tabs with the arrow keys", () => {
		render(<StatsPanel />);
		const mechanics = screen.getByRole("tab", { name: /Mechanics/ });
		fireEvent.keyDown(mechanics, { key: "ArrowRight" });
		expect(screen.getByRole("tab", { name: /World/ })).toHaveAttribute("aria-selected", "true");
		fireEvent.keyDown(screen.getByRole("tab", { name: /World/ }), { key: "End" });
		expect(screen.getByRole("tab", { name: /Brains/ })).toHaveAttribute("aria-selected", "true");
	});

	it("scrubs every chart in a band back to an earlier tick, and returns to now on blur", () => {
		seedRun();
		render(<StatsPanel />);
		fireEvent.click(screen.getByRole("tab", { name: /World/ }));
		const slider = screen.getByRole("slider", { name: "Scrub through time" });
		fireEvent.change(slider, { target: { value: "0" } });
		expect(screen.getByText("At tick 0")).toBeInTheDocument();
		const reading = screen.getByRole("group", { name: "Creatures alive" });
		expect(within(reading).getByTestId("reading-value")).toHaveTextContent("200");
		fireEvent.blur(slider);
		expect(screen.queryByText("At tick 0")).not.toBeInTheDocument();
	});

	it("collapses to the tab bar, remembers it, and reopens on a tab click", () => {
		seedRun();
		render(<StatsPanel />);
		fireEvent.click(screen.getByRole("button", { name: "Collapse statistics" }));
		expect(screen.queryByRole("tabpanel")).not.toBeInTheDocument();
		expect(JSON.parse(window.localStorage.getItem("petri.statsDock") ?? "{}").collapsed).toBe(true);
		fireEvent.click(screen.getByRole("tab", { name: /Evolution/ }));
		expect(screen.getByRole("tabpanel")).toBeInTheDocument();
		expect(screen.getByRole("tab", { name: /Evolution/ })).toHaveAttribute("aria-selected", "true");
	});

	it("resizes from the keyboard on the separator", () => {
		render(<StatsPanel />);
		const handle = screen.getByRole("separator", { name: "Resize statistics" });
		const before = Number(handle.getAttribute("aria-valuenow"));
		fireEvent.keyDown(handle, { key: "ArrowUp" });
		expect(Number(handle.getAttribute("aria-valuenow"))).toBeGreaterThan(before);
	});
});

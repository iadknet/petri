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

	it("leads with three short verdicts and keeps the full sentence on each band", () => {
		seedRun();
		render(<StatsPanel />);
		const strip = screen.getByTestId("stats-verdicts");
		expect(within(strip).getByText("Shrinking · \u221250 creatures")).toBeInTheDocument();
		expect(within(strip).getByTitle(/1\.00 born vs 6\.00 died per tick/)).toBeInTheDocument();
		const world = screen.getByRole("rowheader", { name: /^World Shrinking/ });
		expect(
			within(world).getByText(
				"Shrinking · \u221250 creatures in 10 ticks · 1.00 born vs 6.00 died per tick",
			),
		).toBeInTheDocument();
	});

	it("shows a row per reading with its value now and its change over the window", () => {
		seedRun();
		render(<StatsPanel />);
		const row = screen.getByRole("row", { name: /Creatures alive/ });
		expect(within(row).getByText("150")).toBeInTheDocument();
		expect(within(row).getByText("−50")).toBeInTheDocument();
	});

	it("scrubs every row back to an earlier tick with the time slider", () => {
		seedRun();
		render(<StatsPanel />);
		const slider = screen.getByRole("slider", { name: "Scrub through time" });
		fireEvent.change(slider, { target: { value: "0" } });
		expect(screen.getByText("At tick 0")).toBeInTheDocument();
		const row = screen.getByRole("row", { name: /Creatures alive/ });
		expect(within(row).getByText("200")).toBeInTheDocument();
		fireEvent.keyDown(slider, { key: "Escape" });
		expect(screen.queryByText("At tick 0")).not.toBeInTheDocument();
	});

	it("labels the live strip while rows show the past, and returns to now on blur", () => {
		seedRun();
		render(<StatsPanel />);
		const slider = screen.getByRole("slider", { name: "Scrub through time" });
		fireEvent.change(slider, { target: { value: "0" } });
		expect(within(screen.getByTestId("stats-verdicts")).getByText("Now")).toBeInTheDocument();
		fireEvent.blur(slider);
		expect(screen.queryByText("At tick 0")).not.toBeInTheDocument();
		expect(within(screen.getByTestId("stats-verdicts")).queryByText("Now")).not.toBeInTheDocument();
	});

	it("collapses to the verdict strip and remembers it", () => {
		seedRun();
		render(<StatsPanel />);
		fireEvent.click(screen.getByRole("button", { name: "Collapse statistics" }));
		expect(screen.queryByRole("table")).not.toBeInTheDocument();
		const strip = screen.getByTestId("stats-verdicts");
		expect(within(strip).getByText("Shrinking · \u221250 creatures")).toBeInTheDocument();
		expect(JSON.parse(window.localStorage.getItem("petri.statsDock") ?? "{}").collapsed).toBe(true);
	});

	it("resizes from the keyboard on the separator", () => {
		render(<StatsPanel />);
		const handle = screen.getByRole("separator", { name: "Resize statistics" });
		const before = Number(handle.getAttribute("aria-valuenow"));
		fireEvent.keyDown(handle, { key: "ArrowUp" });
		expect(Number(handle.getAttribute("aria-valuenow"))).toBeGreaterThan(before);
	});

	it("keeps mechanics detail in a folded section", () => {
		seedRun();
		render(<StatsPanel />);
		fireEvent.click(screen.getByText("Mechanics"));
		expect(screen.getByText("Moving")).toBeInTheDocument();
		expect(screen.getByText("Killed by a predator")).toBeInTheDocument();
		expect(screen.getByText("Not enough energy")).toBeInTheDocument();
	});
});

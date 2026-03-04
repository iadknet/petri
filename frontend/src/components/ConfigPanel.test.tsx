import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import { ConfigPanel } from "./ConfigPanel.tsx";

describe("ConfigPanel", () => {
	beforeEach(() => {
		useConfigStore.getState().reset();
		useSimulationStore.getState().reset();
		useStartupConfigStore.getState().reset();
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		useSimulationStore.getState().setSimState("paused");
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
	});

	it("renders startup and runtime top-level sections with food subgroup", () => {
		render(<ConfigPanel />);

		expect(screen.getByText("Startup Config")).toBeInTheDocument();
		expect(screen.getByText("Runtime (Live) Config")).toBeInTheDocument();
		expect(screen.getByTestId("startup-seed-randomize")).toBeInTheDocument();
		expect(screen.getAllByText("Food Parameters").length).toBeGreaterThan(0);
		expect(screen.getByTestId("startup-field-food-initial-density")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-initial-coverage")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-energy-initial-energy")).toBeInTheDocument();
	});

	it("startup edits do not modify runtime local draft for startup-only fields", () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("startup-field-food-initial-density"), {
			target: { value: "0.75" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.initial_density).toBe(0.75);
		expect(useConfigStore.getState().localDraft?.world.food.initial_density).toBe(1.0);
	});

	it("applies runtime field disable rules by simulation state", () => {
		render(<ConfigPanel />);
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("idle");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("running");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).toBeDisabled();
	});

	it("shows runtime unavailable message when runtime config is missing", () => {
		useConfigStore.getState().reset();
		render(<ConfigPanel />);

		expect(
			screen.getByText("Runtime config unavailable. Restart to initialize the simulation."),
		).toBeInTheDocument();
	});
});

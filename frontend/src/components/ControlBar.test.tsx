import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/rest.ts";
import { PanelLayoutProvider } from "../stores/layout.tsx";
import { useSimulationStore } from "../stores/simulation.ts";
import { ControlBar } from "./ControlBar.tsx";

vi.mock("../api/rest.ts", () => ({
	api: {
		start: vi.fn(),
		pause: vi.fn(),
		step: vi.fn(),
	},
}));

describe("ControlBar", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
		vi.clearAllMocks();
	});

	it("updates tick from pause response", async () => {
		vi.mocked(api.pause).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "paused",
			tick: 42,
		});
		useSimulationStore.getState().setSimState("running");
		useSimulationStore.getState().setTick(10);

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-pause"));

		await waitFor(() => {
			expect(useSimulationStore.getState().simState).toBe("paused");
			expect(useSimulationStore.getState().tick).toBe(42);
		});
	});
});

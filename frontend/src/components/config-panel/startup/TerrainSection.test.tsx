import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { buildStartupRequest, useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { TerrainSection } from "./TerrainSection.tsx";

function Harness() {
	const preset = useStartupConfigStore((s) => s.preset);
	const update = useStartupConfigStore((s) => s.updatePreset);
	return <TerrainSection startupPreset={preset} updateStartupPreset={update} />;
}
beforeEach(() => useStartupConfigStore.getState().reset());
it("edits ordered layers, bounds and seeds in the startup request", () => {
	render(<Harness />);
	fireEvent.click(screen.getByRole("button", { name: "Add terrain layer" }));
	fireEvent.change(screen.getByLabelText("Layer 1 pattern"), { target: { value: "Noise" } });
	fireEvent.change(screen.getByLabelText("Map seed"), { target: { value: "123" } });
	fireEvent.change(screen.getByLabelText("Layer 1 seed"), { target: { value: "456" } });
	fireEvent.click(screen.getByLabelText("Layer 1 explicit bounds"));
	fireEvent.change(screen.getByLabelText("Layer 1 width"), { target: { value: "12" } });
	const request = buildStartupRequest(useStartupConfigStore.getState().preset);
	expect(request.world?.world_seed).toBe(123);
	expect(request.world?.terrain?.[0]).toMatchObject({
		params: { pattern_type: "Noise" },
		seed: 456,
		bounds: { width: 12 },
	});
	fireEvent.change(screen.getByLabelText("Map seed"), { target: { value: "9007199254740993" } });
	expect(useStartupConfigStore.getState().preset.world.world_seed).toBe(123);
	fireEvent.change(screen.getByLabelText("Map seed"), { target: { value: "" } });
	expect(useStartupConfigStore.getState().preset.world.world_seed).toBeNull();
	fireEvent.click(screen.getByRole("button", { name: "Remove terrain layer 1" }));
	expect(buildStartupRequest(useStartupConfigStore.getState().preset).world?.terrain).toEqual([]);
});

it("switches through all pattern editors and keeps surviving layer order", () => {
	render(<Harness />);
	fireEvent.click(screen.getByRole("button", { name: "Add terrain layer" }));
	for (const pattern of ["Maze", "Spiral", "Noise", "ParallelLines", "Star", "FbmThreshold"]) {
		fireEvent.change(screen.getByLabelText("Layer 1 pattern"), { target: { value: pattern } });
		expect(useStartupConfigStore.getState().preset.world.terrain[0]?.params.pattern_type).toBe(
			pattern,
		);
	}
	fireEvent.click(screen.getByRole("button", { name: "Add terrain layer" }));
	fireEvent.change(screen.getByLabelText("Layer 2 seed"), { target: { value: "789" } });
	fireEvent.click(screen.getByRole("button", { name: "Remove terrain layer 1" }));
	expect(useStartupConfigStore.getState().preset.world.terrain[0]?.seed).toBe(789);
});

it("randomizes map and layer seeds with the seed buttons", () => {
	const random = vi.spyOn(Math, "random").mockReturnValue(0.5);
	try {
		render(<Harness />);
		fireEvent.click(screen.getByRole("button", { name: "Add terrain layer" }));
		fireEvent.click(screen.getByTestId("startup-map-seed-randomize"));
		expect(useStartupConfigStore.getState().preset.world.world_seed).toBe(2147483648);
		fireEvent.click(screen.getByTestId("startup-terrain-layer-0-seed-randomize"));
		expect(useStartupConfigStore.getState().preset.world.terrain[0]?.seed).toBe(2147483648);
	} finally {
		random.mockRestore();
	}
});

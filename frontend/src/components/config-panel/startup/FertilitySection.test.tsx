import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it } from "vitest";
import { buildStartupRequest, useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { FertilitySection } from "./FertilitySection.tsx";

function Harness() {
	const preset = useStartupConfigStore((s) => s.preset);
	const update = useStartupConfigStore((s) => s.updatePreset);
	const addLayer = useStartupConfigStore((s) => s.addFertilityLayer);
	const updateTarget = useStartupConfigStore((s) => s.updateFertilityLayerTarget);
	return (
		<FertilitySection
			startupPreset={preset}
			updateStartupPreset={update}
			addFertilityLayer={addLayer}
			updateFertilityLayerTarget={updateTarget}
		/>
	);
}

function firstRequestLayer() {
	return buildStartupRequest(useStartupConfigStore.getState().preset).world?.food?.fertility
		?.layers[0];
}

beforeEach(() => useStartupConfigStore.getState().reset());

it("edits a channels layer with max blend into the startup request", () => {
	render(<Harness />);
	const algorithm = screen.getByTestId("startup-field-fertility-layer-0-algorithm");
	fireEvent.change(algorithm, { target: { value: "Channels" } });
	expect((algorithm as HTMLSelectElement).value).toBe("Channels");
	fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-channels-width"), {
		target: { value: "4" },
	});
	fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-channels-value"), {
		target: { value: "0.25" },
	});
	fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-blend"), {
		target: { value: "Max" },
	});

	const layer = firstRequestLayer();
	expect(layer?.blend).toBe("Max");
	expect(layer?.algorithm).toMatchObject({ Channels: { width: 4, value: 0.25 } });

	fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-blend"), {
		target: { value: "WeightedAverage" },
	});
	expect(firstRequestLayer()).not.toHaveProperty("blend");
});

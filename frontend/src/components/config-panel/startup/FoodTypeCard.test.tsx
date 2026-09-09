import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it } from "vitest";
import { buildStartupRequest, useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { FoodTypesSection } from "./FoodTypesSection.tsx";
function Harness() {
	const state = useStartupConfigStore();
	return (
		<FoodTypesSection
			startupPreset={state.preset}
			addFoodType={state.addFoodType}
			removeFoodType={state.removeFoodType}
			updateFoodType={state.updateFoodType}
		/>
	);
}
beforeEach(() => useStartupConfigStore.getState().reset());
it("makes shared inheritance explicit and preserves an explicit zero override", () => {
	render(<Harness />);
	const input = screen.getByLabelText("Energy per density unit");
	expect(input).toBeDisabled();
	fireEvent.click(screen.getByLabelText("Use shared Energy per density unit"));
	expect(input).not.toBeDisabled();
	fireEvent.change(input, { target: { value: "0" } });
	fireEvent.click(screen.getByLabelText("Seed only positive-fertility cells"));
	expect(
		buildStartupRequest(useStartupConfigStore.getState().preset).world?.food?.types?.[0],
	).toMatchObject({ energy_per_unit: 0, initial_fertility_only: true });
	fireEvent.click(screen.getByLabelText("Use shared Energy per density unit"));
	expect(
		buildStartupRequest(useStartupConfigStore.getState().preset).world?.food?.types?.[0]
			?.energy_per_unit,
	).toBeNull();
});

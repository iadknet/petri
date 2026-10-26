import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { InputsPanel } from "./InputsPanel.tsx";

describe("InputsPanel", () => {
	it("shows each input's short label and value, explained in a tooltip", () => {
		render(
			<InputsPanel
				inputRefs={[{ family: "world", key: "NeighborFoodRing" }]}
				upstreamSlots={[0.5]}
			/>,
		);
		const label = screen.getByText("FoodRing");
		expect(label).not.toHaveAttribute("title");
		expect(screen.getByText("0.50")).toBeInTheDocument();
		fireEvent.mouseEnter(label);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/8 neighboring cells/);
	});
});

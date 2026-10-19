import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Tip, TipBubble } from "./Tooltip.tsx";

describe("Tip", () => {
	it("keeps a short label on screen and explains it on hover", () => {
		render(<Tip content="Food on each of the 8 neighboring cells.">FoodRing</Tip>);
		const label = screen.getByText("FoodRing");
		expect(screen.queryByTestId("tooltip")).toBeNull();
		fireEvent.mouseEnter(label);
		expect(screen.getByTestId("tooltip")).toHaveTextContent("Food on each of the 8");
		fireEvent.mouseLeave(label);
		expect(screen.queryByTestId("tooltip")).toBeNull();
	});

	it("opens on keyboard focus, describes the label and closes on Escape", () => {
		render(<Tip content="Explains the label">Label</Tip>);
		const label = screen.getByText("Label");
		expect(label).toHaveAttribute("tabindex", "0");
		expect(label).toHaveAccessibleDescription("Explains the label");
		fireEvent.focus(label);
		expect(screen.getByTestId("tooltip")).toBeInTheDocument();
		fireEvent.keyDown(label, { key: "Escape" });
		expect(screen.queryByTestId("tooltip")).toBeNull();
	});

	it("opens on a tap", () => {
		render(<Tip content="Explains the label">Label</Tip>);
		fireEvent.click(screen.getByText("Label"));
		expect(screen.getByTestId("tooltip")).toBeInTheDocument();
	});
});

describe("TipBubble", () => {
	it("renders structured content at an anchor", () => {
		render(
			<TipBubble anchor={new DOMRect(10, 10, 0, 0)}>
				<b>Edge</b> weight 0.5
			</TipBubble>,
		);
		expect(screen.getByTestId("tooltip")).toHaveTextContent("Edge weight 0.5");
	});
});

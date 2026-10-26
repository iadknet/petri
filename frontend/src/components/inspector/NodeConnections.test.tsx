import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { InputRef } from "../../types/generated/InputRef.ts";
import { NodeConnections } from "./NodeConnections.tsx";

describe("NodeConnections", () => {
	it("lists input refs by label, each explained in a tooltip", () => {
		const inputRefs: InputRef[] = [
			{ family: "world", key: "AreaBarrierSummary" },
			{ family: "upstream", key: "UpstreamSlot", slot: 0 },
		];

		const { container } = render(
			<NodeConnections
				inputRefs={inputRefs}
				targets={[{ target_id: 2, slot: 0, gate_bias: 0.0 }]}
			/>,
		);

		const bus = screen.getByText("Bus[0]");
		const labels = container.querySelectorAll("[tabindex='0']");
		expect(Array.from(labels, (label) => label.textContent)).toEqual(["AreaBarrier", "Bus[0]"]);
		expect(screen.getByText("AreaBarrier")).toHaveAccessibleDescription(/barriers/);
		expect(bus).not.toHaveAttribute("title");
		fireEvent.mouseEnter(bus);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/previous mesh node/);
	});

	it("renders target node IDs", () => {
		render(
			<NodeConnections
				inputRefs={[]}
				targets={[
					{ target_id: 2, slot: 0, gate_bias: 0.0 },
					{ target_id: 4, slot: 1, gate_bias: 0.0 },
				]}
			/>,
		);

		expect(screen.getByText("#2, #4")).toBeInTheDocument();
	});

	it("shows dash when no targets", () => {
		render(<NodeConnections inputRefs={[]} targets={[]} />);

		// "out" label row shows "—" for empty targets
		const outValues = screen.getAllByText("—");
		expect(outValues.length).toBeGreaterThanOrEqual(1);
	});
});

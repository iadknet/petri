import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import { NodeConnections } from "./NodeConnections.tsx";

describe("NodeConnections", () => {
	it("lists input refs by label, each explained in a tooltip", () => {
		const inputRefs: InputReference[] = [{ World: "AreaBarrierSummary" }, { UpstreamSlot: 0 }];

		render(
			<NodeConnections
				inputRefs={inputRefs}
				targets={[{ target_id: 2, slot: 0, gate_bias: 0.0 }]}
			/>,
		);

		const bus = screen.getByText(/Bus\[0\]/);
		expect(bus.parentElement?.textContent).toBe("AreaBarrier, Bus[0]");
		expect(screen.getByText("AreaBarrier").getAttribute("title")).toMatch(/barriers/);
		expect(bus.getAttribute("title")).toMatch(/previous mesh node/);
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

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import { NodeConnections } from "./NodeConnections.tsx";

vi.mock("./inputRefUtils.ts", () => ({
	formatInputRef: (ref: unknown) => JSON.stringify(ref),
}));

describe("NodeConnections", () => {
	it("renders input refs as comma-separated list", () => {
		const inputRefs: InputReference[] = [{ World: "FoodHere" }, { UpstreamSlot: 0 }];

		render(
			<NodeConnections
				inputRefs={inputRefs}
				targets={[{ target_id: 2, slot: 0, gate_bias: 0.0 }]}
			/>,
		);

		const text = `${JSON.stringify({ World: "FoodHere" })}, ${JSON.stringify({ UpstreamSlot: 0 })}`;
		expect(screen.getByText(text)).toBeInTheDocument();
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

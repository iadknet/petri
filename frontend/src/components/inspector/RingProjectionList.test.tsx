import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RingProjectionList } from "./RingProjectionList.tsx";

describe("RingProjectionList", () => {
	it("lists each projection's ring, kind, offset and weight", () => {
		render(
			<RingProjectionList
				projections={[
					{ ref_idx: 1, kind: "Move", offset: 0, weight: 0 },
					{ ref_idx: 9, kind: "StealEnergy", offset: 3, weight: -0.25 },
				]}
				inputRefs={[{ World: "FoodHere" }, { World: "NeighborBarrierRing" }]}
			/>,
		);
		const rows = screen.getAllByTestId("ring-projection");
		expect(rows).toHaveLength(2);
		expect(rows[0]).toHaveTextContent("Move");
		expect(rows[0]).toHaveTextContent("offset 0");
		expect(rows[0]).toHaveTextContent("w=0.000");
		expect(rows[1]).toHaveTextContent("ref 9 (missing)");
		expect(rows[1]).toHaveTextContent("StealEnergy");
		expect(rows[1]).toHaveTextContent("offset 3");
		expect(rows[1]).toHaveTextContent("w=-0.250");
	});

	it("renders nothing without projections", () => {
		const { container } = render(<RingProjectionList projections={undefined} inputRefs={[]} />);
		expect(container.innerHTML).toBe("");
	});
});

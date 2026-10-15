import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { GraphBackendDef, GraphEdge, OutputSink } from "../../types/genome.ts";
import { EdgeGroupList } from "./EdgeGroupList.tsx";

const leaf = (ref_idx: number, sub_idx: number, own_weight: number, group?: number): GraphEdge => ({
	source: { InputLeaf: { ref_idx, sub_idx } },
	own_weight,
	...(group === undefined ? {} : { membership: { group } }),
});

/** A Move group at offset 1 on ring 1 missing its channel-3 member. */
function grouped(): GraphBackendDef {
	const sinks: OutputSink[] = Array.from({ length: 8 }, (_, d) => ({
		kind: { ActionVote: { Move: d } },
		inputs: [] as GraphEdge[],
	}));
	for (let channel = 0; channel < 8; channel++) {
		if (channel === 3) continue;
		sinks[(channel + 1) % 8]?.inputs.push(leaf(1, channel, channel === 0 ? 0.25 : 0, 0));
	}
	return {
		compute_nodes: [],
		output_sinks: sinks,
		groups: [{ shape: { Ring: { ring_ref: 1 } }, kind: "Move", offset: 1, scale: 0.5 }],
	};
}

describe("EdgeGroupList", () => {
	it("lists each group's ring, kind, offset, scale and members present with holes", () => {
		render(
			<EdgeGroupList
				graphDef={grouped()}
				inputRefs={[{ World: "FoodHere" }, { World: "NeighborBarrierRing" }]}
				graphWeights={{
					node_id: 0,
					scales: [{ group: 0, genomic: 0.5 }],
					edges: [{ surface: "sink", container: 1, edge: 0, genomic: 0.75 }],
				}}
			/>,
		);
		const row = screen.getByTestId("edge-group");
		expect(row).toHaveTextContent("Move");
		expect(row).toHaveTextContent("offset 1");
		expect(row).toHaveTextContent("scale=0.500");
		expect(screen.getByTestId("edge-group-members")).toHaveTextContent("7/8 ●●●○●●●●");
		const members = screen.getAllByTestId("edge-group-member");
		expect(members).toHaveLength(7);
		expect(members[0]).toHaveTextContent("cell N → Move[NE] own=0.250 genomic=0.750");
	});

	it("collapses and reopens a group's member rows, open by default", () => {
		render(<EdgeGroupList graphDef={grouped()} inputRefs={[]} graphWeights={null} />);
		const toggle = screen.getByRole("button");
		expect(toggle).toHaveAttribute("aria-expanded", "true");
		expect(screen.getAllByTestId("edge-group-member")).toHaveLength(7);
		expect(screen.getByTestId("edge-group")).toHaveTextContent("ref 1 (missing)");
		fireEvent.click(toggle);
		expect(toggle).toHaveAttribute("aria-expanded", "false");
		expect(screen.queryAllByTestId("edge-group-member")).toHaveLength(0);
		fireEvent.click(toggle);
		expect(screen.getAllByTestId("edge-group-member")).toHaveLength(7);
	});

	it("renders nothing without groups", () => {
		const { container } = render(
			<EdgeGroupList
				graphDef={{ compute_nodes: [], output_sinks: [] }}
				inputRefs={[]}
				graphWeights={null}
			/>,
		);
		expect(container.innerHTML).toBe("");
	});
});

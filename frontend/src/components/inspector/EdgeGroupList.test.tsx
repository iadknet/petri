import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { GraphBackendDef, GraphEdge, OutputSink } from "../../types/genome.ts";
import { EdgeGroupList } from "./EdgeGroupList.tsx";

const leaf = (ref_idx: number, sub_idx: number, own_weight: number, group?: number): GraphEdge => ({
	source: { InputLeaf: { ref_idx, sub_idx } },
	own_weight,
	...(group === undefined ? {} : { membership: { group, scope: "Edge" as const } }),
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
		groups: [
			{
				shape: { Ring: { ring_ref: 1 } },
				kind: "Move",
				offset: 1,
				scale: 0.5,
				inherit_scale: false,
				retention: 1,
			},
		],
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
					edges: [{ surface: "sink", container: 1, edge: 0, genomic: 0.75, applied: 0.75 }],
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
		fireEvent.mouseEnter(screen.getAllByText(/own=0\.250/)[0] as HTMLElement);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(
			"scale × pattern + correction0.500 × 1.00 + 0.250 = 0.750",
		);
	});

	it("shows the learned scale, inherit_scale, and each member's scope and applied weight", () => {
		const def = grouped();
		const group = def.groups?.[0];
		if (group) group.inherit_scale = true;
		const member = def.output_sinks[1]?.inputs[0];
		if (member?.membership) member.membership.scope = "Group";
		render(
			<EdgeGroupList
				graphDef={def}
				inputRefs={[{ World: "FoodHere" }, { World: "NeighborBarrierRing" }]}
				graphWeights={{
					node_id: 0,
					scales: [{ group: 0, genomic: 0.5, learned: 1.25 }],
					edges: [{ surface: "sink", container: 1, edge: 0, genomic: 0.75, applied: 1.5 }],
				}}
			/>,
		);
		expect(screen.getByTestId("edge-group")).toHaveTextContent("scale=0.500");
		expect(screen.getByTestId("edge-group-learned-scale")).toHaveTextContent("learned=1.250");
		expect(screen.getByTestId("edge-group-inherit-scale")).toHaveTextContent("inherits scale");
		expect(screen.queryByTestId("edge-group-retention")).toBeNull();
		const members = screen.getAllByTestId("edge-group-member");
		expect(members[0]).toHaveTextContent("own=0.250 genomic=0.750 applied=1.500 group scope");
		expect(members[1]).toHaveTextContent("edge scope");
		expect(members[1]).not.toHaveTextContent("applied=");
	});

	it("shows a group's retention when it decays (T25.F09)", () => {
		const def = grouped();
		const group = def.groups?.[0];
		if (group) group.retention = 0.99;
		render(<EdgeGroupList graphDef={def} inputRefs={[]} graphWeights={null} />);
		expect(screen.getByTestId("edge-group-retention")).toHaveTextContent("retention=0.9900");
	});

	it("shows neither a learned scale nor applied weights without the payload", () => {
		render(<EdgeGroupList graphDef={grouped()} inputRefs={[]} graphWeights={null} />);
		expect(screen.queryByTestId("edge-group-learned-scale")).toBeNull();
		expect(screen.getByTestId("edge-group-inherit-scale")).toHaveTextContent(
			"genomic scale at birth",
		);
		expect(screen.getAllByTestId("edge-group-member")[0]).not.toHaveTextContent("applied=");
	});

	it("explains the offset and the members glyphs in tooltips, outside the toggle", () => {
		render(<EdgeGroupList graphDef={grouped()} inputRefs={[]} graphWeights={null} />);
		const toggle = screen.getByRole("button");
		const offset = screen.getByText("offset 1");
		expect(toggle).not.toContainElement(offset);
		expect(offset).not.toHaveAttribute("title");
		fireEvent.mouseEnter(offset);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/1 step clockwise/);
		fireEvent.mouseLeave(offset);
		fireEvent.focus(screen.getByTestId("edge-group-members"));
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/by ring cell/);
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

	it("lists a pair group's fields, kind, offset, scale and members over twelve with holes", () => {
		// AreaFoodSummary pair 1 is (nearestDx 3, nearestDy 4); a Move group at
		// offset 2 without its x member at relative E (sink 4), plus a
		// hand-built zero-pattern x member at relative N (sink 2).
		const sinks: OutputSink[] = Array.from({ length: 8 }, (_, d) => ({
			kind: { ActionVote: { Move: d } },
			inputs: [] as GraphEdge[],
		}));
		const birth: [number, number][] = [
			...[1, 2, 3, 5, 6, 7].map((r): [number, number] => [3, r]),
			...[0, 1, 3, 4, 5, 7].map((r): [number, number] => [4, r]),
		];
		for (const [field, relative] of birth) {
			if (field === 3 && relative === 2) continue;
			sinks[(relative + 2) % 8]?.inputs.push(leaf(0, field, 0, 0));
		}
		sinks[2]?.inputs.push(leaf(0, 3, 0.125, 0));
		render(
			<EdgeGroupList
				graphDef={{
					compute_nodes: [],
					output_sinks: sinks,
					groups: [
						{
							shape: { DirectionPair: { ref_idx: 0, pair: 1 } },
							kind: "Move",
							offset: 2,
							scale: 1,
							inherit_scale: false,
							retention: 1,
						},
					],
				}}
				inputRefs={[{ World: { AreaFoodSummary: { type_idx: 0 } } }]}
				graphWeights={null}
			/>,
		);
		const row = screen.getByTestId("edge-group");
		expect(row).toHaveTextContent("Pair");
		expect(row).toHaveTextContent("nearestDx/nearestDy #0");
		expect(row).toHaveTextContent("offset 2");
		expect(row).toHaveTextContent("scale=1.000");
		expect(screen.getByTestId("edge-group-members")).toHaveTextContent("11/12 ●○●●●●●●●●●● +1");
		const members = screen.getAllByTestId("edge-group-member");
		expect(members).toHaveLength(12);
		expect(members.filter((m) => m.textContent?.includes("(zero pattern)"))).toHaveLength(1);
		expect(members.some((m) => m.textContent?.includes("y:N → Move[E]"))).toBe(true);
		fireEvent.mouseEnter(screen.getByText(/own=0\.125/));
		expect(screen.getByTestId("tooltip")).toHaveTextContent("1.000 × 0.00 + 0.125 = 0.125");
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

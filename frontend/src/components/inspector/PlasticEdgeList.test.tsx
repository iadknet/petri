import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type { GraphBackendDef, PlasticityConfig } from "../../types/genome.ts";
import { PlasticEdgeList, collectPlasticEdges, plasticityBadge } from "./PlasticEdgeList.tsx";
import { buildFullGraphModel } from "./graphFullModel.ts";

const hebbian: PlasticityConfig = {
	rule: "Oja",
	learning_rate: 0.1,
	weight_clamp: 2,
	lamarckian: false,
};
const rewarded: PlasticityConfig = {
	rule: "Classic",
	learning_rate: 0.1,
	weight_clamp: 2,
	lamarckian: true,
	modulation: { reward_source: "EnergyDelta", trace_decay: 0.5 },
};

/** One compute node with a plastic memory edge feeding an `Eat` vote whose
 * edge is reward-modulated, beside a static vote edge. */
function plastic(): GraphBackendDef {
	return {
		compute_nodes: [
			{
				kind: "Add",
				inputs: [
					{
						source: { SharedMemory: { slot: 0, previous: false } },
						own_weight: 0.5,
						plasticity: hebbian,
					},
				],
			},
		],
		output_sinks: [
			{
				kind: { ActionVote: "Eat" },
				inputs: [
					{ source: { ComputeNode: 0 }, own_weight: 1.5, plasticity: rewarded },
					{ source: { ComputeNode: 0 }, own_weight: 0.75 },
				],
			},
		],
	};
}

const weights: NodeGraphWeights = {
	node_id: 0,
	scales: [],
	edges: [
		{ surface: "compute", container: 0, edge: 0, genomic: 0.5, applied: 0.5 },
		{ surface: "sink", container: 0, edge: 0, genomic: 1.5, applied: 1.25, learned: 1.25 },
	],
};

describe("PlasticEdgeList", () => {
	it("names each edge's rule, pathway and inheritance", () => {
		expect(plasticityBadge(hebbian)).toBe("Oja · hebbian · darwinian");
		expect(plasticityBadge(rewarded)).toBe("Classic · reward EnergyDelta · lamarckian");
	});

	it("lists every plastic edge on both surfaces with its weights and badge", () => {
		const rows = collectPlasticEdges(plastic(), [], weights);
		expect(rows.map((row) => [row.surface, row.container, row.edge])).toEqual([
			["compute", 0, 0],
			["sink", 0, 0],
		]);
		expect(rows[0]?.learned).toBeUndefined();
		expect(rows[1]?.applied).toBe(1.25);
		render(<PlasticEdgeList graphDef={plastic()} inputRefs={[]} graphWeights={weights} />);
		expect(screen.getAllByTestId("plastic-edge")).toHaveLength(2);
		const badges = screen.getAllByTestId("plasticity-badge").map((badge) => badge.textContent);
		expect(badges).toEqual([
			"Oja · hebbian · darwinian",
			"Classic · reward EnergyDelta · lamarckian",
		]);
		expect(screen.getByText(/not initialized/)).toBeTruthy();
	});

	it("renders nothing for a graph without a plastic edge", () => {
		const { container } = render(
			<PlasticEdgeList
				graphDef={{ compute_nodes: [], output_sinks: [] }}
				inputRefs={[]}
				graphWeights={null}
			/>,
		);
		expect(container.textContent).toBe("");
	});

	it("draws a plastic edge at its applied weight and a static one at its own", () => {
		const model = buildFullGraphModel(plastic(), [], weights);
		const sinkEdges = model.edges.filter((edge) => edge.edgeType === "compute_to_output");
		expect(sinkEdges.map((edge) => [edge.weight, edge.isPlastic])).toEqual([
			[1.25, true],
			[0.75, false],
		]);
		const withoutPayload = buildFullGraphModel(plastic(), [], null);
		expect(
			withoutPayload.edges
				.filter((edge) => edge.edgeType === "compute_to_output")
				.map((edge) => edge.weight),
		).toEqual([1.5, 0.75]);
	});
});

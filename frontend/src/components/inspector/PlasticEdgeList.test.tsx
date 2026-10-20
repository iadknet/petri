import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type { GraphBackendDef, InputReference, PlasticityConfig } from "../../types/genome.ts";
import {
	PlasticEdgeList,
	collectPlasticEdges,
	plasticityBadge,
	plasticityDetail,
} from "./PlasticEdgeList.tsx";
import { buildFullGraphModel } from "./graphFullModel.ts";
import { weightLabel } from "./graphInternalsEdges.tsx";

const hebbian: PlasticityConfig = {
	rule: "Oja",
	learning_rate: 0.1,
	weight_clamp: 2,
	lamarckian: false,
	retention: 1,
};
const rewarded: PlasticityConfig = {
	rule: "Classic",
	learning_rate: 0.1,
	weight_clamp: 2,
	lamarckian: true,
	retention: 1,
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

const refs: InputReference[] = [{ World: "FoodHere" }, { World: "NeighborBarrierRing" }];

/** A ring group's one member on `Move[1]`, a group-scope learner. */
function groupLearner(lamarckian: boolean, inheritScale: boolean): GraphBackendDef {
	return {
		compute_nodes: [],
		output_sinks: [
			{
				kind: { ActionVote: { Move: 1 } },
				inputs: [
					{
						source: { InputLeaf: { ref_idx: 1, sub_idx: 0 } },
						own_weight: 0.25,
						plasticity: { ...hebbian, lamarckian },
						membership: { group: 0, scope: "Group" },
					},
				],
			},
		],
		groups: [
			{
				shape: { Ring: { ring_ref: 1 } },
				kind: "Move",
				offset: 1,
				scale: 0.5,
				inherit_scale: inheritScale,
				retention: 1,
			},
		],
	};
}

/** The group learner's payload: no own quantity, the scale slot uninitialized. */
const groupWeights: NodeGraphWeights = {
	node_id: 0,
	scales: [{ group: 0, genomic: 0.5 }],
	edges: [{ surface: "sink", container: 0, edge: 0, genomic: 0.25, applied: 0.75 }],
};

describe("PlasticEdgeList", () => {
	it("names each edge's rule, pathway, trace decay and inheritance", () => {
		expect(plasticityBadge(hebbian)).toBe("Oja · hebbian · darwinian");
		expect(plasticityBadge(rewarded)).toBe(
			"Classic · reward EnergyDelta · decay 0.50 · lamarckian",
		);
	});

	it("names a member's scope in the badge and its tooltip", () => {
		const group = { scope: "Group", inheritScale: false } as const;
		const edge = { scope: "Edge", inheritScale: false } as const;
		expect(plasticityBadge(hebbian, group)).toBe("Oja · hebbian · darwinian · group scope");
		expect(plasticityBadge(hebbian, edge)).toBe("Oja · hebbian · darwinian · edge scope");
		expect(plasticityDetail(hebbian, group)[0]).toMatch(/shared scale/);
		expect(plasticityDetail(hebbian)).toHaveLength(3);
	});

	it("explains a group learner's inheritance by inherit_scale, not its lamarckian flag", () => {
		const lamarckian = { ...hebbian, lamarckian: true };
		const darwinianScale = { scope: "Group", inheritScale: false } as const;
		expect(plasticityBadge(lamarckian, darwinianScale)).toBe(
			"Oja · hebbian · darwinian · group scope",
		);
		const detail = plasticityDetail(lamarckian, darwinianScale);
		expect(detail.at(-1)).toMatch(/genomic scale/);
		expect(detail.at(-1)).toMatch(/Lamarckian flag is inert/);
		expect(detail.join(" ")).not.toMatch(/inherit the learned weight/);
		const inheritedScale = { scope: "Group", inheritScale: true } as const;
		expect(plasticityBadge(hebbian, inheritedScale)).toBe(
			"Oja · hebbian · lamarckian · group scope",
		);
		expect(plasticityDetail(hebbian, inheritedScale).at(-1)).toMatch(/learned scale/);
		// An edge learner keeps its own flag.
		expect(plasticityBadge(lamarckian, { scope: "Edge", inheritScale: false })).toBe(
			"Oja · hebbian · lamarckian · edge scope",
		);
	});

	it("states an edge's retention only when it decays (T25.F09)", () => {
		expect(plasticityDetail(hebbian).join(" ")).not.toMatch(/Decays/);
		const decaying = { ...hebbian, retention: 0.999 };
		const detail = plasticityDetail(decaying);
		expect(detail).toHaveLength(4);
		expect(detail.join(" ")).toMatch(/keeps 0\.9990 of its learned change/);
	});

	it("infers a group learner's initialization from its group's learned scale", () => {
		const scales = [{ group: 0, genomic: 0.5, learned: 0.8 }];
		const learned = { ...groupWeights, scales };
		const [row] = collectPlasticEdges(groupLearner(true, false), refs, learned);
		expect(row?.initialized).toBe(true);
		expect(row?.learned).toBeUndefined();
		render(
			<PlasticEdgeList
				graphDef={groupLearner(true, false)}
				inputRefs={refs}
				graphWeights={learned}
			/>,
		);
		expect(screen.getByTestId("plastic-edge")).not.toHaveTextContent("not initialized");
		const badge = screen.getByTestId("plasticity-badge");
		expect(badge).toHaveTextContent("darwinian · group scope");
		fireEvent.mouseEnter(badge);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/genomic scale/);
	});

	it("marks a group learner uninitialized while its scale slot is", () => {
		const [row] = collectPlasticEdges(groupLearner(true, false), refs, groupWeights);
		expect(row?.initialized).toBe(false);
		render(
			<PlasticEdgeList
				graphDef={groupLearner(true, false)}
				inputRefs={refs}
				graphWeights={groupWeights}
			/>,
		);
		expect(screen.getByTestId("plastic-edge")).toHaveTextContent("not initialized");
	});

	it("spells out every learning attribute for the badge's tooltip", () => {
		expect(plasticityDetail(hebbian)).toEqual([
			"Oja rule, learning rate 0.100, weight clamp ±2.00.",
			"Hebbian: learns from co-activity alone.",
			"Darwinian: offspring start from the genomic weight.",
		]);
		expect(plasticityDetail(rewarded)).toEqual([
			"Classic rule, learning rate 0.100, weight clamp ±2.00.",
			"Reward-modulated by EnergyDelta; its eligibility trace keeps 0.50 of itself each tick.",
			"Lamarckian: offspring inherit the learned weight.",
		]);
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
			"Classic · reward EnergyDelta · decay 0.50 · lamarckian",
		]);
		expect(screen.getByText(/not initialized/)).toBeTruthy();
		fireEvent.mouseEnter(screen.getAllByTestId("plasticity-badge")[1] as HTMLElement);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/eligibility trace keeps 0\.50/);
	});

	it("marks the applied weight unavailable without the runtime payload", () => {
		render(<PlasticEdgeList graphDef={plastic()} inputRefs={[]} graphWeights={null} />);
		const rows = screen.getAllByTestId("plastic-edge").map((row) => row.textContent ?? "");
		expect(rows).toHaveLength(2);
		for (const row of rows) {
			expect(row).toContain("applied=unavailable");
			expect(row).not.toContain("not initialized");
		}
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
		).toEqual([null, 0.75]);
	});

	it("carries a plastic edge's learning and weights for its tooltip", () => {
		const model = buildFullGraphModel(plastic(), [], weights);
		const sinkEdge = model.edges.find((edge) => edge.id === "sink:0:0");
		expect(sinkEdge?.plastic).toEqual({
			config: rewarded,
			genomic: 1.5,
			applied: 1.25,
			learned: 1.25,
			initialized: true,
		});
		expect(sinkEdge?.own).toBe(1.5);
		expect(model.edges.find((edge) => edge.id === "sink:0:1")?.plastic).toBeUndefined();
		const withoutPayload = buildFullGraphModel(plastic(), [], null);
		expect(withoutPayload.edges.find((edge) => edge.id === "sink:0:0")?.plastic).toEqual({
			config: rewarded,
			genomic: 1.5,
			initialized: false,
		});
	});

	it("labels an unavailable edge weight n/a and the unit weight not at all", () => {
		expect(weightLabel({ weight: null, opacity: 0.2 })).toBe("n/a");
		expect(weightLabel({ weight: 1, opacity: 1 })).toBeNull();
		expect(weightLabel({ weight: 0.75, opacity: 0.75 })).toBe("0.75");
		expect(weightLabel(undefined)).toBeNull();
	});
});

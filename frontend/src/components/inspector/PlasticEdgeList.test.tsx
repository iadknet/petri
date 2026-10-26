import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { graphBrain, learner, ringGroup } from "../../test/brainFixtures.ts";
import type { EdgeLearner } from "../../types/generated/EdgeLearner.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import {
	PlasticEdgeList,
	collectPlasticEdges,
	plasticityBadge,
	plasticityDetail,
} from "./PlasticEdgeList.tsx";
import { buildFullGraphModel } from "./graphFullModel.ts";
import { weightLabel } from "./graphInternalsEdges.tsx";

const hebbian: EdgeLearner = learner({ rule: "Oja" });
const rewarded: EdgeLearner = learner({
	inherits: true,
	modulation: { reward_source: "EnergyDelta", trace_decay: 0.5 },
});

/** One compute node with a plastic memory edge feeding an `Eat` vote whose
 * edge is reward-modulated, beside a static vote edge. The unseeded
 * compute learner's own weight 3 is past its clamp, so the runtime starts
 * it at the bound: applied 2 against genomic 3 (preservation case: the
 * uninitialized learner, T25.F09 invariant 2). */
function plastic(): GraphBrain {
	return graphBrain({
		compute_nodes: [
			{
				kind: "Add",
				edges: [
					{
						source: { SharedMemory: { slot: 0, previous: false } },
						own: 3,
						genomic: 3,
						applied: 2,
						learner: hebbian,
					},
				],
			},
		],
		output_sinks: [
			{
				kind: { ActionVote: "Eat" },
				edges: [
					{
						source: { ComputeNode: 0 },
						own: 1.5,
						genomic: 1.5,
						applied: 1.5,
						learner: rewarded,
					},
					{ source: { ComputeNode: 0 }, own: 0.75, genomic: 0.75, applied: 0.75 },
				],
			},
		],
	});
}

const learning: NodeLearning = {
	node_id: 0,
	scales: [],
	edges: [
		{ surface: "compute", container: 0, edge: 0, applied: 2, initialized: false },
		{
			surface: "sink",
			container: 0,
			edge: 0,
			applied: 1.25,
			learned: 1.25,
			trace: 0,
			initialized: true,
		},
	],
};

const refs: InputRef[] = [
	{ family: "world", key: "FoodHere" },
	{ family: "world", key: "NeighborBarrierRing" },
];

/** A ring group's two group-scope learners on `Move[1]` and `Move[2]` that
 * differ in rule, rate, reward source and trace: one learned scale moves
 * both (preservation case: heterogeneous contributors, T25.F07 invariant 4). */
function groupLearners(initialized: boolean): GraphBrain {
	const member = (field: number) => ({
		group: 0,
		scope: "Group" as const,
		site: { Direction: field + 1 },
		field,
		pattern: 1,
		scale: 0.5,
	});
	return graphBrain({
		output_sinks: [
			{
				kind: { ActionVote: { Directed: ["Move", 1] } },
				edges: [
					{
						source: { InputLeaf: { ref_idx: 1, sub_idx: 0 } },
						own: 0.25,
						genomic: 0.75,
						applied: 0.75,
						member: member(0),
						learner: learner({ rule: "Oja", scope: "group", initialized, inherits: false }),
					},
				],
			},
			{
				kind: { ActionVote: { Directed: ["Move", 2] } },
				edges: [
					{
						source: { InputLeaf: { ref_idx: 1, sub_idx: 1 } },
						own: 0,
						genomic: 0.5,
						applied: 0.5,
						member: member(1),
						learner: learner({
							rule: "Covariance",
							learning_rate: 0.3,
							modulation: { reward_source: "DamageDelta", trace_decay: 0.25 },
							scope: "group",
							initialized,
							inherits: false,
						}),
					},
				],
			},
		],
		groups: [ringGroup(1, { offset: 1, scale: 0.5 })],
	});
}

/** The group learners' poll: no own quantity, the scale slot initialized. */
const groupLearning: NodeLearning = {
	node_id: 0,
	scales: [{ group: 0, learned: 0.8, initialized: true }],
	edges: [
		{ surface: "sink", container: 0, edge: 0, applied: 1.05, initialized: true },
		{ surface: "sink", container: 1, edge: 0, applied: 0.8, initialized: true },
	],
};

describe("PlasticEdgeList", () => {
	it("names each edge's rule, pathway, trace decay and inheritance", () => {
		expect(plasticityBadge(hebbian)).toBe("Oja · hebbian · darwinian");
		expect(plasticityBadge(rewarded)).toBe(
			"Classic · reward EnergyDelta · decay 0.50 · lamarckian",
		);
	});

	it("names a member's scope in the badge and its tooltip", () => {
		const group = { ...hebbian, scope: "group" } as const;
		const edge = { ...hebbian, scope: "edge" } as const;
		expect(plasticityBadge(group)).toBe("Oja · hebbian · darwinian · group scope");
		expect(plasticityBadge(edge)).toBe("Oja · hebbian · darwinian · edge scope");
		expect(plasticityDetail(group)[0]).toMatch(/shared scale/);
		expect(plasticityDetail(hebbian)).toHaveLength(3);
	});

	it("explains a group learner's inheritance by the view's inherits, its lamarckian flag inert", () => {
		const darwinianScale = { ...hebbian, scope: "group", inherits: false } as const;
		expect(plasticityBadge(darwinianScale)).toBe("Oja · hebbian · darwinian · group scope");
		const detail = plasticityDetail(darwinianScale);
		expect(detail.at(-1)).toMatch(/genomic scale/);
		expect(detail.at(-1)).toMatch(/Lamarckian flag is inert/);
		expect(detail.join(" ")).not.toMatch(/inherit the learned weight/);
		const inheritedScale = { ...hebbian, scope: "group", inherits: true } as const;
		expect(plasticityBadge(inheritedScale)).toBe("Oja · hebbian · lamarckian · group scope");
		expect(plasticityDetail(inheritedScale).at(-1)).toMatch(/learned scale/);
		const lamarckianEdge = { ...hebbian, scope: "edge", inherits: true } as const;
		expect(plasticityBadge(lamarckianEdge)).toBe("Oja · hebbian · lamarckian · edge scope");
	});

	it("states an edge's retention only when it decays (T25.F09)", () => {
		expect(plasticityDetail(hebbian).join(" ")).not.toMatch(/Decays/);
		const decaying = { ...hebbian, retention: 0.999 };
		const detail = plasticityDetail(decaying);
		expect(detail).toHaveLength(4);
		expect(detail.join(" ")).toMatch(/keeps 0\.9990 of its learned change/);
	});

	it("names a group learner's own retention inert: its group's retention governs", () => {
		const forgetful = { ...hebbian, retention: 0 };
		const detail = plasticityDetail({ ...forgetful, scope: "group" }).join(" ");
		expect(detail).not.toMatch(/Decays/);
		expect(detail).toMatch(/Edge retention is inert at group scope/);
		expect(plasticityDetail({ ...forgetful, scope: "edge" }).join(" ")).toMatch(
			/Decays toward its birth weight/,
		);
	});

	it("shows heterogeneous contributors of one scale as their own learners", () => {
		const rows = collectPlasticEdges(groupLearners(false), refs, groupLearning);
		expect(rows.map((row) => [row.learner.rule, row.learner.learning_rate])).toEqual([
			["Oja", 0.1],
			["Covariance", 0.3],
		]);
		expect(rows.every((row) => row.initialized && row.learned === undefined)).toBe(true);
		expect(rows.map((row) => row.applied)).toEqual([1.05, 0.8]);
		render(
			<PlasticEdgeList graphDef={groupLearners(false)} inputRefs={refs} learning={groupLearning} />,
		);
		const rendered = screen.getAllByTestId("plastic-edge");
		expect(rendered).toHaveLength(2);
		for (const row of rendered) expect(row).not.toHaveTextContent("not initialized");
		const badges = screen.getAllByTestId("plasticity-badge");
		expect(badges[1]).toHaveTextContent("Covariance · reward DamageDelta · decay 0.25");
		fireEvent.mouseEnter(badges[0] as HTMLElement);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/genomic scale/);
	});

	it("takes a learner's initialization from the brain when the poll lists nothing", () => {
		const [row] = collectPlasticEdges(groupLearners(true), refs, null);
		expect(row?.initialized).toBe(true);
		const [unborn] = collectPlasticEdges(groupLearners(false), refs, null);
		expect(unborn?.initialized).toBe(false);
		render(<PlasticEdgeList graphDef={groupLearners(false)} inputRefs={refs} learning={null} />);
		expect(screen.getAllByTestId("plastic-edge")[0]).toHaveTextContent("not initialized");
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
		const rows = collectPlasticEdges(plastic(), [], learning);
		expect(rows.map((row) => [row.surface, row.container, row.edge])).toEqual([
			["compute", 0, 0],
			["sink", 0, 0],
		]);
		expect(rows[0]?.learned).toBeUndefined();
		expect(rows[1]?.applied).toBe(1.25);
		render(<PlasticEdgeList graphDef={plastic()} inputRefs={[]} learning={learning} />);
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

	it("shows an uninitialized learner at the runtime's applied weight, not its genomic one", () => {
		render(<PlasticEdgeList graphDef={plastic()} inputRefs={[]} learning={learning} />);
		const [unseeded] = screen.getAllByTestId("plastic-edge");
		expect(unseeded).toHaveTextContent("genomic=3.000 applied=2.000 (not initialized)");
	});

	it("renders nothing for a graph without a plastic edge", () => {
		const { container } = render(
			<PlasticEdgeList graphDef={graphBrain()} inputRefs={[]} learning={null} />,
		);
		expect(container.textContent).toBe("");
	});

	it("draws a listed edge at learning's applied weight, else at the brain's", () => {
		const model = buildFullGraphModel(plastic(), [], learning);
		const sinkEdges = model.edges.filter((edge) => edge.edgeType === "compute_to_output");
		expect(sinkEdges.map((edge) => [edge.weight, edge.isPlastic])).toEqual([
			[1.25, true],
			[0.75, false],
		]);
		const withoutPoll = buildFullGraphModel(plastic(), [], null);
		expect(
			withoutPoll.edges
				.filter((edge) => edge.edgeType === "compute_to_output")
				.map((edge) => edge.weight),
		).toEqual([1.5, 0.75]);
	});

	it("carries a plastic edge's learner and weights for its tooltip", () => {
		const model = buildFullGraphModel(plastic(), [], learning);
		const sinkEdge = model.edges.find((edge) => edge.id === "sink:0:0");
		expect(sinkEdge?.plastic).toEqual({
			learner: rewarded,
			genomic: 1.5,
			applied: 1.25,
			learned: 1.25,
			initialized: true,
		});
		expect(sinkEdge?.own).toBe(1.5);
		expect(model.edges.find((edge) => edge.id === "sink:0:1")?.plastic).toBeUndefined();
	});

	it("labels the unit weight not at all", () => {
		expect(weightLabel({ weight: 1, opacity: 1 })).toBeNull();
		expect(weightLabel({ weight: 0.75, opacity: 0.75 })).toBe("0.75");
		expect(weightLabel(undefined)).toBeNull();
	});
});

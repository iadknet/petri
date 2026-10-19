import { fireEvent, render, screen } from "@testing-library/react";
import { Position } from "@xyflow/react";
import type { ComponentProps } from "react";
import { describe, expect, it } from "vitest";
import type { GraphBackendDef } from "../../types/genome.ts";
import { buildGraphFlowScene } from "./graphFlowAdapter.ts";
import { buildFullGraphModel } from "./graphFullModel.ts";
import {
	BackwardWeightEdge,
	type EdgeTip,
	WeightEdge,
	type WeightEdgeData,
} from "./graphInternalsEdges.tsx";

function edgeProps(tip: EdgeTip): ComponentProps<typeof WeightEdge> {
	const data: WeightEdgeData = { weight: tip.weight, opacity: 1, tip };
	return {
		id: "sink:0:0",
		source: "in:leaf:0:0",
		target: "sink:0",
		sourceX: 0,
		sourceY: 0,
		targetX: 100,
		targetY: 40,
		sourcePosition: Position.Right,
		targetPosition: Position.Left,
		data,
	} as ComponentProps<typeof WeightEdge>;
}

function hover(tip: EdgeTip, Edge = WeightEdge): HTMLElement {
	render(
		<svg>
			<title>graph edge</title>
			<Edge {...edgeProps(tip)} />
		</svg>,
	);
	fireEvent.mouseEnter(screen.getByTestId("edge-hit"), { clientX: 50, clientY: 20 });
	return screen.getByTestId("tooltip");
}

describe("graph edge tooltips", () => {
	it("names a plain edge's endpoints and weight", () => {
		const tip = hover({ source: "FoodHere", target: "CN0", weight: 0.5, own: 0.5 });
		expect(tip).toHaveTextContent("FoodHere → CN0");
		expect(tip).toHaveTextContent("weight 0.500");
	});

	it("shows a member as scale × pattern + correction", () => {
		const tip = hover({
			source: "OccRing.N",
			target: "Move[NE]",
			weight: 0.75,
			own: 0.25,
			member: { group: 2, scale: 0.5, pattern: 1, scope: "Edge", inheritScale: false },
		});
		expect(tip).toHaveTextContent("group #2 member");
		expect(tip).toHaveTextContent("scale × pattern + correction");
		expect(tip).toHaveTextContent("0.500 × 1.00 + 0.250 = 0.750");
		expect(tip).toHaveTextContent("applied unavailable · scope Edge");
	});

	it("shows a plastic edge's weights, rule, reward source, trace decay and inheritance", () => {
		const tip = hover(
			{
				source: "CN0",
				target: "CN0",
				weight: 1.25,
				own: 1.5,
				plastic: {
					config: {
						rule: "Classic",
						learning_rate: 0.1,
						weight_clamp: 2,
						lamarckian: true,
						modulation: { reward_source: "EnergyDelta", trace_decay: 0.5 },
					},
					genomic: 1.5,
					applied: 1.25,
					initialized: false,
				},
			},
			BackwardWeightEdge,
		);
		expect(tip).toHaveTextContent("genomic 1.500 · applied 1.250 (not initialized)");
		expect(tip).toHaveTextContent("Reward-modulated by EnergyDelta");
		expect(tip).toHaveTextContent("keeps 0.50 of itself each tick");
		expect(tip).toHaveTextContent("Lamarckian");
	});

	it("marks a plastic edge's applied weight unavailable without the payload", () => {
		const tip = hover({
			source: "CN0",
			target: "Eat",
			weight: null,
			own: 1.5,
			plastic: {
				config: { rule: "Oja", learning_rate: 0.1, weight_clamp: 2, lamarckian: false },
				genomic: 1.5,
				initialized: false,
			},
		});
		expect(tip).toHaveTextContent("genomic 1.500 · applied unavailable");
		expect(tip).toHaveTextContent("Hebbian");
	});

	it("shows an initialized group learner's scope and scale inheritance", () => {
		const graphDef: GraphBackendDef = {
			compute_nodes: [],
			output_sinks: [
				{
					kind: { ActionVote: { Move: 1 } },
					inputs: [
						{
							source: { InputLeaf: { ref_idx: 1, sub_idx: 0 } },
							own_weight: 0.25,
							plasticity: { rule: "Oja", learning_rate: 0.1, weight_clamp: 2, lamarckian: true },
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
					inherit_scale: false,
				},
			],
		};
		const model = buildFullGraphModel(
			graphDef,
			[{ World: "FoodHere" }, { World: "NeighborBarrierRing" }],
			{
				node_id: 0,
				scales: [{ group: 0, genomic: 0.5, learned: 0.8 }],
				edges: [{ surface: "sink", container: 0, edge: 0, genomic: 0.25, applied: 1.05 }],
			},
		);
		const edge = model.edges.find((candidate) => candidate.id === "sink:0:0");
		expect(edge?.plastic?.initialized).toBe(true);
		expect(edge?.plastic?.learned).toBeUndefined();
		expect(edge?.member?.inheritScale).toBe(false);
		if (edge === undefined) throw new Error("the member is drawn");
		const tip = hover({ ...edge, source: "OccRing.N", target: "Move[NE]" });
		expect(tip).toHaveTextContent("genomic 0.250 · applied 1.050");
		expect(tip).not.toHaveTextContent("not initialized");
		expect(tip).toHaveTextContent(/shared scale/);
		expect(tip).toHaveTextContent(/genomic scale/);
		expect(tip).not.toHaveTextContent(/inherit the learned weight/);
	});

	it("closes when the pointer leaves", () => {
		hover({ source: "A", target: "B", weight: 1, own: 1 });
		fireEvent.mouseLeave(screen.getByTestId("edge-hit"));
		expect(screen.queryByTestId("tooltip")).toBeNull();
	});
});

describe("buildGraphFlowScene edge tips", () => {
	it("labels each edge's tip with its endpoints' node labels", () => {
		const model = buildFullGraphModel(
			{
				compute_nodes: [],
				output_sinks: [
					{
						kind: { ActionVote: "Eat" },
						inputs: [{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, own_weight: 0.5 }],
					},
				],
			},
			[{ World: "FoodHere" }],
		);
		const box = { x: 0, y: 0, width: 80, height: 24 };
		const scene = buildGraphFlowScene({
			model,
			layout: {
				positions: new Map(model.nodes.map((node) => [node.id, box])),
				totalWidth: 200,
				totalHeight: 40,
			},
			traceOverlay: null,
			liveSet: new Set(),
		});
		const source = model.nodes.find((node) => node.nodeType === "input")?.label;
		const target = model.nodes.find((node) => node.nodeType === "output_sink")?.label;
		expect(scene.edges[0]?.data?.tip).toEqual({ source, target, weight: 0.5, own: 0.5 });
		// The node's content is its one tab stop, not React Flow's wrapper too.
		expect(scene.nodes.every((node) => node.focusable === false)).toBe(true);
	});

	it("opens an edge's tooltip on a tap", () => {
		render(
			<svg>
				<title>graph edge</title>
				<WeightEdge {...edgeProps({ source: "A", target: "B", weight: 1, own: 1 })} />
			</svg>,
		);
		fireEvent.pointerDown(screen.getByTestId("edge-hit"), { clientX: 5, clientY: 5 });
		expect(screen.getByTestId("tooltip")).toHaveTextContent("A → B");
	});
});

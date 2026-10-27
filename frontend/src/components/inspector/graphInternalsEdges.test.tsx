import { fireEvent, render, screen } from "@testing-library/react";
import { Position } from "@xyflow/react";
import type { ComponentProps } from "react";
import { describe, expect, it } from "vitest";
import { graphBrain, learner, ringBundle } from "../../test/brainFixtures.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
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
		const tip = hover({ source: "FoodHere", target: "CN0", weight: 0.5, own: 0.5, genomic: 0.5 });
		expect(tip).toHaveTextContent("FoodHere → CN0");
		expect(tip).toHaveTextContent("weight 0.500");
	});

	it("prints a member's scale, pattern, own and genomic weight as the view resolved them", () => {
		const tip = hover({
			source: "OccRing.N",
			target: "Move[NE]",
			weight: 1.5,
			own: 0.25,
			genomic: 0.75,
			member: { bundle: 2, scale: 0.5, pattern: 1, scope: "Edge", applied: 1.5 },
		});
		expect(tip).toHaveTextContent("bundle #2 member");
		expect(tip).toHaveTextContent("scale × pattern + correction");
		expect(tip).toHaveTextContent("0.500 × 1.00 + 0.250 = 0.750");
		expect(tip).toHaveTextContent("applied 1.500 · scope Edge");
	});

	it("shows a plastic edge's weights, rule, reward source, trace decay and inheritance", () => {
		const tip = hover(
			{
				source: "CN0",
				target: "CN0",
				weight: 1.25,
				own: 1.5,
				genomic: 1.5,
				plastic: {
					learner: learner({
						inherits: true,
						modulation: { reward_source: "EnergyDelta", trace_decay: 0.5 },
					}),
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

	it("shows an initialized bundle learner's scope and scale inheritance", () => {
		const graphDef: GraphBrain = graphBrain({
			output_sinks: [
				{
					kind: { ActionVote: { Directed: ["Move", 1] } },
					edges: [
						{
							source: { InputLeaf: { ref_idx: 1, sub_idx: 0 } },
							own: 0.25,
							genomic: 0.75,
							applied: 0.75,
							learner: learner({ rule: "Oja", scope: "group", inherits: false }),
							member: {
								bundle: 0,
								scope: "Group",
								site: { Direction: 1 },
								field: 0,
								pattern: 1,
								scale: 0.5,
							},
						},
					],
				},
			],
			bundles: [ringBundle(1, { offset: 1, scale: 0.5 })],
		});
		const model = buildFullGraphModel(
			graphDef,
			[
				{ family: "world", key: "FoodHere" },
				{ family: "world", key: "NeighborBarrierRing" },
			],
			{
				node_id: 0,
				scales: [{ bundle: 0, learned: 0.8, initialized: true }],
				edges: [{ surface: "sink", container: 0, edge: 0, applied: 1.05, initialized: true }],
			},
		);
		const edge = model.edges.find((candidate) => candidate.id === "sink:0:0");
		expect(edge?.plastic?.initialized).toBe(true);
		expect(edge?.plastic?.learned).toBeUndefined();
		if (edge === undefined) throw new Error("the member is drawn");
		const tip = hover({ ...edge, source: "OccRing.N", target: "Move[NE]" });
		expect(tip).toHaveTextContent("genomic 0.750 · applied 1.050");
		expect(tip).not.toHaveTextContent("not initialized");
		expect(tip).toHaveTextContent(/shared scale/);
		expect(tip).toHaveTextContent(/genomic scale/);
		expect(tip).not.toHaveTextContent(/inherit the learned weight/);
	});

	it("closes when the pointer leaves", () => {
		hover({ source: "A", target: "B", weight: 1, own: 1, genomic: 1 });
		fireEvent.mouseLeave(screen.getByTestId("edge-hit"));
		expect(screen.queryByTestId("tooltip")).toBeNull();
	});
});

describe("buildGraphFlowScene edge tips", () => {
	it("labels each edge's tip with its endpoints' node labels", () => {
		const model = buildFullGraphModel(
			graphBrain({
				output_sinks: [
					{
						kind: { ActionVote: "Eat" },
						edges: [
							{
								source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } },
								own: 0.5,
								genomic: 0.5,
								applied: 0.5,
							},
						],
					},
				],
			}),
			[{ family: "world", key: "FoodHere" }],
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
		expect(scene.edges[0]?.data?.tip).toEqual({
			source,
			target,
			weight: 0.5,
			own: 0.5,
			genomic: 0.5,
		});
		// The node's content is its one tab stop, not React Flow's wrapper too.
		expect(scene.nodes.every((node) => node.focusable === false)).toBe(true);
	});

	it("opens an edge's tooltip on a tap", () => {
		render(
			<svg>
				<title>graph edge</title>
				<WeightEdge {...edgeProps({ source: "A", target: "B", weight: 1, own: 1, genomic: 1 })} />
			</svg>,
		);
		fireEvent.pointerDown(screen.getByTestId("edge-hit"), { clientX: 5, clientY: 5 });
		expect(screen.getByTestId("tooltip")).toHaveTextContent("A → B");
	});
});

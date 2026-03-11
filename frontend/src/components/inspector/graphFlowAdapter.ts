import { type Edge, MarkerType, type Node } from "@xyflow/react";
import type { GraphInternalsNodeData } from "./GraphInternalsNode.tsx";
import type { FullGraphModel } from "./graphFullModel.ts";
import type { WeightEdgeData } from "./graphInternalsEdges.tsx";
import type { LayoutOutput } from "./graphInternalsLayout.ts";
import type { NodeTraceData } from "./graphTraceOverlay.ts";

export interface GraphFlowScene {
	nodes: Node<GraphInternalsNodeData>[];
	edges: Edge<WeightEdgeData>[];
}

const EDGE_TINTS: Record<string, { r: number; g: number; b: number }> = {
	input_to_compute: { r: 16, g: 185, b: 129 },
	input_to_output: { r: 251, g: 191, b: 36 },
	compute_to_output: { r: 251, g: 191, b: 36 },
};

export function buildGraphFlowScene(input: {
	model: FullGraphModel;
	layout: LayoutOutput;
	traceOverlay: Map<string, NodeTraceData> | null;
	liveSet: Set<number>;
}): GraphFlowScene {
	const { model, layout, traceOverlay, liveSet } = input;

	const flowNodes: Node<GraphInternalsNodeData>[] = [];
	for (const mn of model.nodes) {
		const pos = layout.positions.get(mn.id);
		if (!pos) continue;

		const trace = traceOverlay?.get(mn.id) ?? null;
		const isLive = mn.nodeType === "compute" ? liveSet.has(mn.arrayIndex) : true;

		flowNodes.push({
			id: mn.id,
			type: "graphInternal",
			position: { x: pos.x, y: pos.y },
			draggable: false,
			connectable: false,
			selectable: false,
			style: { width: pos.width, height: pos.height },
			data: {
				index: mn.arrayIndex,
				kindLabel: mn.label,
				subtitle: mn.subtitle,
				category: mn.category,
				isLive,
				initialValue: trace?.initialValue ?? null,
				outputValue: trace?.outputValue ?? null,
				stateChange: trace?.stateChange ?? null,
				routeTargets: null,
				selectedTarget: null,
				// Output trace data
				weightedSum: trace?.weightedSum ?? null,
				appliedValue: trace?.appliedValue ?? null,
				applied: trace?.applied ?? null,
				fired: trace?.fired ?? null,
				emittedAction: trace?.emittedAction ?? null,
				queueDelta: trace?.queueDelta ?? null,
				gateFired: trace?.gateFired ?? null,
				queueNonEmpty: trace?.queueNonEmpty ?? null,
			},
		});
	}

	const nodeById = new Map(model.nodes.map((n) => [n.id, n]));

	const flowEdges: Edge<WeightEdgeData>[] = [];
	for (const me of model.edges) {
		const sourceNode = nodeById.get(me.sourceId);
		const targetNode = nodeById.get(me.targetId);
		// Input and output nodes are always "live"; only compute nodes use liveSet
		const sourceLive =
			sourceNode?.nodeType === "compute" ? liveSet.has(sourceNode.arrayIndex) : true;
		const targetLive =
			targetNode?.nodeType === "compute" ? liveSet.has(targetNode.arrayIndex) : true;
		const endpointsLive = sourceLive && targetLive;

		const weightOpacity = Math.max(0.2, Math.min(Math.abs(me.weight), 1));
		const opacity = endpointsLive ? weightOpacity : weightOpacity * 0.35;

		const tint = EDGE_TINTS[me.edgeType];
		const strokeColor = tint
			? `rgba(${tint.r},${tint.g},${tint.b},${opacity})`
			: `rgba(148,163,184,${opacity})`;
		const markerColor = me.isBackward ? `rgba(251,191,36,${opacity})` : strokeColor;

		if (me.isBackward) {
			flowEdges.push({
				id: me.id,
				source: me.targetId,
				target: me.sourceId,
				sourceHandle: "bottom-out",
				targetHandle: "bottom-in",
				type: "backwardEdge",
				selectable: false,
				focusable: false,
				markerEnd: {
					type: MarkerType.ArrowClosed,
					color: markerColor,
					width: 10,
					height: 10,
				},
				data: {
					weight: me.weight,
					opacity,
					tint: tint ?? undefined,
				},
			});
		} else {
			flowEdges.push({
				id: me.id,
				source: me.sourceId,
				target: me.targetId,
				type: "weightEdge",
				selectable: false,
				focusable: false,
				markerEnd: {
					type: MarkerType.ArrowClosed,
					color: strokeColor,
					width: 10,
					height: 10,
				},
				style: { stroke: strokeColor },
				data: {
					weight: me.weight,
					opacity,
					tint: tint ?? undefined,
				},
			});
		}
	}

	return { nodes: flowNodes, edges: flowEdges };
}

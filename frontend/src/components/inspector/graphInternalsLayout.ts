import type { ElkExtendedEdge, ElkNode } from "elkjs/lib/elk-api";
import ELK from "elkjs/lib/elk.bundled.js";

const elk = new ELK();

export interface LayoutInputNode {
	id: string;
	width: number;
	height: number;
	layerConstraint?: "FIRST" | "LAST";
}

export interface LayoutInputEdge {
	id: string;
	sourceId: string;
	targetId: string;
}

export interface LayoutOutput {
	positions: Map<string, { x: number; y: number; width: number; height: number }>;
	totalWidth: number;
	totalHeight: number;
}

export async function layoutGraphInternals(
	nodes: LayoutInputNode[],
	edges: LayoutInputEdge[],
): Promise<LayoutOutput> {
	if (nodes.length === 0) {
		return { positions: new Map(), totalWidth: 0, totalHeight: 0 };
	}

	const elkEdges: ElkExtendedEdge[] = edges.map((e) => ({
		id: e.id,
		sources: [e.sourceId],
		targets: [e.targetId],
	}));

	const elkGraph: ElkNode = {
		id: "graph-internals",
		layoutOptions: {
			"elk.algorithm": "layered",
			"elk.direction": "RIGHT",
			"elk.edgeRouting": "ORTHOGONAL",
			"elk.padding": "[left=12,top=12,right=12,bottom=28]",
			"elk.spacing.nodeNode": "16",
			"elk.layered.spacing.nodeNodeBetweenLayers": "32",
			"elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
			"elk.separateConnectedComponents": "false",
		},
		children: nodes.map((n) => {
			const layoutOptions: Record<string, string> = {};
			if (n.layerConstraint) {
				layoutOptions["elk.layered.layerConstraint"] = n.layerConstraint;
			}
			return {
				id: n.id,
				width: n.width,
				height: n.height,
				layoutOptions,
			};
		}),
		edges: elkEdges,
	};

	const layoutResult = await elk.layout(elkGraph);

	const positions = new Map<string, { x: number; y: number; width: number; height: number }>();
	for (const child of layoutResult.children ?? []) {
		positions.set(child.id, {
			x: child.x ?? 0,
			y: child.y ?? 0,
			width: child.width ?? 0,
			height: child.height ?? 0,
		});
	}

	return {
		positions,
		totalWidth: layoutResult.width ?? 0,
		totalHeight: layoutResult.height ?? 0,
	};
}

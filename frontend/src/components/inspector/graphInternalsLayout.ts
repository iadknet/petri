import type { ElkExtendedEdge, ElkNode } from "elkjs/lib/elk-api";
import ELK from "elkjs/lib/elk.bundled.js";
import type { ComputeNode, GraphSource } from "../../types/genome.ts";

const elk = new ELK();

const DEFAULT_NODE_WIDTH = 72;
const NODE_HEIGHT = 24;

export interface GraphInternalLayoutNode {
	index: number;
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface GraphInternalLayoutEdge {
	id: string;
	fromIndex: number;
	toIndex: number;
	weight: number;
	isBackward: boolean;
}

export interface GraphInternalsLayoutResult {
	nodes: GraphInternalLayoutNode[];
	edges: GraphInternalLayoutEdge[];
	width: number;
	height: number;
}

export type NodeCategory = "constant" | "processing";

/** Extract the source compute node index from a GraphSource, or null if not a ComputeNode source. */
function computeNodeSourceIndex(source: GraphSource): number | null {
	if ("ComputeNode" in source) return source.ComputeNode;
	return null;
}

export async function layoutGraphInternals(
	computeNodes: ComputeNode[],
	nodeWidths?: number[],
	nodeCategories?: NodeCategory[],
	nodeHeights?: number[],
): Promise<GraphInternalsLayoutResult> {
	if (computeNodes.length === 0) {
		return { nodes: [], edges: [], width: 0, height: 0 };
	}

	const elkEdges: ElkExtendedEdge[] = [];
	const resultEdges: GraphInternalLayoutEdge[] = [];

	for (let toIdx = 0; toIdx < computeNodes.length; toIdx++) {
		const node = computeNodes[toIdx];
		if (!node) continue;
		for (let inputIdx = 0; inputIdx < node.inputs.length; inputIdx++) {
			const edge = node.inputs[inputIdx];
			if (!edge) continue;
			const fromIdx = computeNodeSourceIndex(edge.source);
			// Only layout edges between compute nodes (InputLeaf/SharedMemory are implicit)
			if (fromIdx === null || fromIdx < 0 || fromIdx >= computeNodes.length) continue;
			const edgeId = `CN${fromIdx}->${toIdx}:${inputIdx}`;
			elkEdges.push({
				id: edgeId,
				sources: [String(fromIdx)],
				targets: [String(toIdx)],
			});
			resultEdges.push({
				id: edgeId,
				fromIndex: fromIdx,
				toIndex: toIdx,
				weight: edge.weight,
				isBackward: fromIdx >= toIdx,
			});
		}
	}

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
		children: computeNodes.map((_, index) => {
			const category = nodeCategories?.[index];
			const layoutOptions: Record<string, string> = {};
			if (category === "constant") {
				layoutOptions["elk.layered.layerConstraint"] = "FIRST";
			}
			return {
				id: String(index),
				width: nodeWidths?.[index] ?? DEFAULT_NODE_WIDTH,
				height: nodeHeights?.[index] ?? NODE_HEIGHT,
				layoutOptions,
			};
		}),
		edges: elkEdges,
	};

	const layoutResult = await elk.layout(elkGraph);

	const nodes: GraphInternalLayoutNode[] = (layoutResult.children ?? []).map((child) => ({
		index: Number(child.id),
		x: child.x ?? 0,
		y: child.y ?? 0,
		width: child.width ?? DEFAULT_NODE_WIDTH,
		height: child.height ?? NODE_HEIGHT,
	}));

	return {
		nodes,
		edges: resultEdges,
		width: layoutResult.width ?? 0,
		height: layoutResult.height ?? 0,
	};
}

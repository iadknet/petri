import type { ElkExtendedEdge, ElkNode } from "elkjs/lib/elk-api";
import ELK from "elkjs/lib/elk.bundled.js";
import type { GraphInternalNode } from "../../types/genome.ts";

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

export type NodeCategory = "input" | "constant" | "memory" | "processing" | "output";

export async function layoutGraphInternals(
	internalNodes: GraphInternalNode[],
	nodeWidths?: number[],
	nodeCategories?: NodeCategory[],
	nodeHeights?: number[],
): Promise<GraphInternalsLayoutResult> {
	if (internalNodes.length === 0) {
		return { nodes: [], edges: [], width: 0, height: 0 };
	}

	const elkEdges: ElkExtendedEdge[] = [];
	const resultEdges: GraphInternalLayoutEdge[] = [];

	for (let toIdx = 0; toIdx < internalNodes.length; toIdx++) {
		const node = internalNodes[toIdx];
		if (!node) continue;
		for (let inputIdx = 0; inputIdx < node.inputs.length; inputIdx++) {
			const input = node.inputs[inputIdx];
			if (!input) continue;
			// Skip edges referencing out-of-bounds nodes (junk from mutation)
			if (input.source_idx < 0 || input.source_idx >= internalNodes.length) continue;
			const edgeId = `${input.source_idx}->${toIdx}:${inputIdx}`;
			elkEdges.push({
				id: edgeId,
				sources: [String(input.source_idx)],
				targets: [String(toIdx)],
			});
			resultEdges.push({
				id: edgeId,
				fromIndex: input.source_idx,
				toIndex: toIdx,
				weight: input.weight,
				isBackward: input.source_idx >= toIdx,
			});
		}
	}

	// Add invisible anchor edges for disconnected nodes so FIRST/LAST constraints work
	if (nodeCategories) {
		const connectedNodes = new Set<number>();
		for (const edge of elkEdges) {
			connectedNodes.add(Number(edge.sources[0]));
			connectedNodes.add(Number(edge.targets[0]));
		}
		const isSourceCategory = (c: NodeCategory) => c === "input" || c === "constant" || c === "memory";
		const firstSource = nodeCategories.findIndex(isSourceCategory);
		const firstOutput = nodeCategories.findIndex((c) => c === "output");
		for (let i = 0; i < internalNodes.length; i++) {
			if (connectedNodes.has(i)) continue;
			const cat = nodeCategories[i];
			if (cat === "output" && firstSource >= 0) {
				elkEdges.push({
					id: `_anchor-${firstSource}->${i}`,
					sources: [String(firstSource)],
					targets: [String(i)],
				});
			} else if (cat && isSourceCategory(cat) && firstOutput >= 0) {
				elkEdges.push({
					id: `_anchor-${i}->${firstOutput}`,
					sources: [String(i)],
					targets: [String(firstOutput)],
				});
			}
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
		children: internalNodes.map((_, index) => {
			const category = nodeCategories?.[index];
			const layoutOptions: Record<string, string> = {};
			if (category === "input" || category === "constant" || category === "memory") {
				layoutOptions["elk.layered.layerConstraint"] = "FIRST";
			} else if (category === "output") {
				layoutOptions["elk.layered.layerConstraint"] = "LAST";
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

	const nodes: GraphInternalLayoutNode[] = (layoutResult.children ?? []).map(
		(child) => ({
			index: Number(child.id),
			x: child.x ?? 0,
			y: child.y ?? 0,
			width: child.width ?? DEFAULT_NODE_WIDTH,
			height: child.height ?? NODE_HEIGHT,
		}),
	);

	return {
		nodes,
		edges: resultEdges,
		width: layoutResult.width ?? 0,
		height: layoutResult.height ?? 0,
	};
}

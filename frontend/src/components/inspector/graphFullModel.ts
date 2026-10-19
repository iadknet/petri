import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type {
	GraphBackendDef,
	GraphEdge,
	GraphSource,
	InputReference,
} from "../../types/genome.ts";
import type { NodeCategory } from "./graphNodeCategories.ts";
import { categorizeComputeNode } from "./graphNodeCategories.ts";
import {
	describeComputeKind,
	describeGraphSource,
	describeOutputSink,
} from "./graphNodeFormatters.ts";

export interface GraphModelNode {
	id: string;
	nodeType: "input" | "compute" | "output_sink";
	category: NodeCategory;
	label: string;
	/** Plain-words tooltip: what the node reads, computes or does. */
	description: string;
	subtitle?: string;
	arrayIndex: number;
	baseWidth: number;
	height: number;
}

export interface GraphModelEdge {
	id: string;
	sourceId: string;
	targetId: string;
	/**
	 * The weight drawn: a member's genomic weight (the server's
	 * `graph_weights` value), a plastic edge's applied weight (T25.F06), else
	 * its own weight.
	 */
	weight: number;
	edgeType: "input_to_compute" | "compute_to_compute" | "compute_to_output" | "input_to_output";
	isBackward: boolean;
	/** Whether the edge is an edge-group member (T25.F03), drawn dashed. */
	isMember: boolean;
	/** Whether the edge carries learning attributes (T25.F06). */
	isPlastic: boolean;
}

/**
 * A member's genomic weight: the server's `graph_weights` value, or v3-core's
 * `compose(own, scale, 1)` for a `Ring` member (`cgp/group.rs`) when the
 * payload is missing. Never its own weight alone.
 */
function memberWeight(
	graphDef: GraphBackendDef,
	edge: GraphEdge,
	genomic: number | undefined,
): number {
	if (genomic !== undefined) return genomic;
	const group = edge.membership ? graphDef.groups?.[edge.membership.group] : undefined;
	return (group?.scale ?? 0) + edge.own_weight;
}

export interface FullGraphModel {
	nodes: GraphModelNode[];
	edges: GraphModelEdge[];
}

function inputNodeId(source: GraphSource): string | null {
	if ("InputLeaf" in source) {
		const { ref_idx, sub_idx } = source.InputLeaf;
		return `in:leaf:${ref_idx}:${sub_idx}`;
	}
	if ("SharedMemory" in source) {
		const { slot, previous } = source.SharedMemory;
		return `in:mem:${slot}:${previous}`;
	}
	return null;
}

function computeBaseWidth(label: string): number {
	return Math.max(72, Math.min(label.length * 7 + 24, 200));
}

function collectEdgesFromSources(
	edges: GraphEdge[],
	weightOf: (edge: GraphEdge, index: number) => number,
	targetId: string,
	targetArrayIndex: number,
	targetType: "compute" | "output",
	inputNodeIds: Set<string>,
	edgePrefix: string,
	computeNodeCount: number,
): GraphModelEdge[] {
	const result: GraphModelEdge[] = [];
	for (let i = 0; i < edges.length; i++) {
		const edge = edges[i];
		if (!edge) continue;
		const source = edge.source;
		if ("ComputeNode" in source) {
			const idx = source.ComputeNode;
			if (idx < 0 || idx >= computeNodeCount) continue;
			const sourceId = `cn:${idx}`;
			const edgeType = targetType === "compute" ? "compute_to_compute" : "compute_to_output";
			const isBackward = targetType === "compute" && idx >= targetArrayIndex;
			result.push({
				id: `${edgePrefix}:${i}`,
				sourceId,
				targetId,
				weight: weightOf(edge, i),
				edgeType,
				isBackward,
				isMember: edge.membership !== undefined,
				isPlastic: edge.plasticity !== undefined,
			});
		} else {
			const inId = inputNodeId(source);
			if (inId && inputNodeIds.has(inId)) {
				const edgeType = targetType === "compute" ? "input_to_compute" : "input_to_output";
				result.push({
					id: `${edgePrefix}:${i}`,
					sourceId: inId,
					targetId,
					weight: weightOf(edge, i),
					edgeType,
					isBackward: false,
					isMember: edge.membership !== undefined,
					isPlastic: edge.plasticity !== undefined,
				});
			}
		}
	}
	return result;
}

export function buildFullGraphModel(
	graphDef: GraphBackendDef,
	inputRefs: InputReference[],
	graphWeights: NodeGraphWeights | null = null,
): FullGraphModel {
	const weights = new Map(
		(graphWeights?.edges ?? []).map((entry) => [
			`${entry.surface}:${entry.container}:${entry.edge}`,
			entry,
		]),
	);
	const weightOf =
		(surface: "compute" | "sink", container: number) => (edge: GraphEdge, index: number) => {
			const entry = weights.get(`${surface}:${container}:${index}`);
			if (edge.membership) return memberWeight(graphDef, edge, entry?.genomic);
			// A plastic edge is drawn at the weight in force (T25.F06).
			if (edge.plasticity) return entry?.applied ?? edge.own_weight;
			return edge.own_weight;
		};
	const nodes: GraphModelNode[] = [];
	const allEdges: GraphModelEdge[] = [];
	const inputNodeIds = new Set<string>();

	// Phase 1: Discover all unique input sources across the entire graph
	const allGraphEdges: GraphEdge[] = [];
	for (const cn of graphDef.compute_nodes) {
		for (const edge of cn.inputs) allGraphEdges.push(edge);
	}
	for (const sink of graphDef.output_sinks) {
		for (const edge of sink.inputs) allGraphEdges.push(edge);
	}

	for (const edge of allGraphEdges) {
		const inId = inputNodeId(edge.source);
		if (inId && !inputNodeIds.has(inId)) {
			inputNodeIds.add(inId);
			const { label, description } = describeGraphSource(edge.source, inputRefs);
			nodes.push({
				id: inId,
				nodeType: "input",
				category: "input",
				label,
				description,
				arrayIndex: -1,
				baseWidth: computeBaseWidth(label),
				height: 24,
			});
		}
	}

	// Phase 2: Create compute nodes
	for (let i = 0; i < graphDef.compute_nodes.length; i++) {
		const cn = graphDef.compute_nodes[i];
		if (!cn) continue;
		const { label, description } = describeComputeKind(cn.kind);
		const category = categorizeComputeNode(cn.kind);
		nodes.push({
			id: `cn:${i}`,
			nodeType: "compute",
			category,
			label,
			description,
			arrayIndex: i,
			baseWidth: computeBaseWidth(label),
			height: 24,
		});
	}

	// Phase 3: Create output nodes (only wired sinks)
	for (let i = 0; i < graphDef.output_sinks.length; i++) {
		const sink = graphDef.output_sinks[i];
		if (!sink || sink.inputs.length === 0) continue;
		const { label, subtitle, description } = describeOutputSink(sink.kind);
		nodes.push({
			id: `sink:${i}`,
			nodeType: "output_sink",
			category:
				"ActionVote" in sink.kind || "ActionParam" in sink.kind ? "output_action" : "output_value",
			label,
			description,
			subtitle,
			arrayIndex: i,
			baseWidth: computeBaseWidth(label),
			height: subtitle ? 32 : 24,
		});
	}

	// Phase 4: Collect all edges
	const cnCount = graphDef.compute_nodes.length;
	for (let i = 0; i < cnCount; i++) {
		const cn = graphDef.compute_nodes[i];
		if (!cn) continue;
		allEdges.push(
			...collectEdgesFromSources(
				cn.inputs,
				weightOf("compute", i),
				`cn:${i}`,
				i,
				"compute",
				inputNodeIds,
				`cn:${i}`,
				cnCount,
			),
		);
	}

	for (let i = 0; i < graphDef.output_sinks.length; i++) {
		const sink = graphDef.output_sinks[i];
		if (!sink || sink.inputs.length === 0) continue;
		allEdges.push(
			...collectEdgesFromSources(
				sink.inputs,
				weightOf("sink", i),
				`sink:${i}`,
				i,
				"output",
				inputNodeIds,
				`sink:${i}`,
				cnCount,
			),
		);
	}

	return { nodes, edges: allEdges };
}

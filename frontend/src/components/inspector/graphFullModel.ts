import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { EdgeLearner } from "../../types/generated/EdgeLearner.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { GraphSource } from "../../types/generated/GraphSource.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { Scope } from "../../types/generated/Scope.ts";
import type { WeightSurface } from "../../types/generated/WeightSurface.ts";
import { joinLearning } from "./brainLearning.ts";
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
	 * The weight drawn: the weight in force, a `learning` entry's when the
	 * poll lists the edge, else the brain's (T30.F01 invariant 5).
	 */
	weight: number;
	edgeType: "input_to_compute" | "compute_to_compute" | "compute_to_output" | "input_to_output";
	isBackward: boolean;
	/** Whether the edge is an edge-group member (T25.F03), drawn dashed. */
	isMember: boolean;
	/** Whether the edge carries learning attributes (T25.F06). */
	isPlastic: boolean;
	/** The edge's own weight: a member's correction. */
	own: number;
	/** The edge's genomic weight: a member's composed weight. */
	genomic: number;
	/** A member's composition, as the brain view resolved it (T25.F03). */
	member?: MemberComposition;
	/** A plastic edge's learner and runtime state (T25.F06). */
	plastic?: PlasticFacts;
}

/** A member's composition: `genomic = scale × pattern + own`, each term the
 * server's. */
export interface MemberComposition {
	group: number;
	scope: Scope;
	scale: number;
	pattern: number;
	/** The weight in force. */
	applied: number;
}

export interface PlasticFacts {
	learner: EdgeLearner;
	genomic: number;
	/** The weight in force. */
	applied: number;
	/** The learned own quantity, once initialized; never for a group-scope member. */
	learned?: number;
	/** Whether the learner is initialized (at group scope, its scale slot). */
	initialized: boolean;
}

/** What the model knows of an edge beyond its endpoints. */
type EdgeFacts = Pick<GraphModelEdge, "weight" | "own" | "genomic" | "member" | "plastic">;

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
	edges: BrainEdge[],
	factsOf: (edge: BrainEdge, index: number) => EdgeFacts,
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
				...factsOf(edge, i),
				edgeType,
				isBackward,
				isMember: edge.member !== undefined,
				isPlastic: edge.learner !== undefined,
			});
		} else {
			const inId = inputNodeId(source);
			if (inId && inputNodeIds.has(inId)) {
				const edgeType = targetType === "compute" ? "input_to_compute" : "input_to_output";
				result.push({
					id: `${edgePrefix}:${i}`,
					sourceId: inId,
					targetId,
					...factsOf(edge, i),
					edgeType,
					isBackward: false,
					isMember: edge.member !== undefined,
					isPlastic: edge.learner !== undefined,
				});
			}
		}
	}
	return result;
}

export function buildFullGraphModel(
	graphDef: GraphBrain,
	inputRefs: InputRef[],
	learning: NodeLearning | null = null,
): FullGraphModel {
	const join = joinLearning(learning);
	const factsOf =
		(surface: WeightSurface, container: number) =>
		(edge: BrainEdge, index: number): EdgeFacts => {
			const state = join.edge(surface, container, index, edge);
			const member = edge.member && {
				group: edge.member.group,
				scope: edge.member.scope,
				scale: edge.member.scale,
				pattern: edge.member.pattern,
				applied: state.applied,
			};
			const plastic = edge.learner && {
				learner: edge.learner,
				genomic: edge.genomic,
				applied: state.applied,
				...(state.learned === undefined ? {} : { learned: state.learned }),
				initialized: state.initialized ?? false,
			};
			return {
				weight: state.applied,
				own: edge.own,
				genomic: edge.genomic,
				...(member ? { member } : {}),
				...(plastic ? { plastic } : {}),
			};
		};
	const nodes: GraphModelNode[] = [];
	const allEdges: GraphModelEdge[] = [];
	const inputNodeIds = new Set<string>();

	// Phase 1: Discover all unique input sources across the entire graph
	const allGraphEdges = [...graphDef.compute_nodes, ...graphDef.output_sinks].flatMap(
		(container) => container.edges,
	);

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
		if (!sink || sink.edges.length === 0) continue;
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
				cn.edges,
				factsOf("compute", i),
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
		if (!sink || sink.edges.length === 0) continue;
		allEdges.push(
			...collectEdgesFromSources(
				sink.edges,
				factsOf("sink", i),
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

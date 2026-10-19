import type { EdgeGenomicWeight, NodeGraphWeights } from "../../types/creature-detail.ts";
import type {
	GraphBackendDef,
	GraphEdge,
	GraphSource,
	InputReference,
	Membership,
	OutputSinkKind,
	PlasticityConfig,
	Scope,
} from "../../types/genome.ts";
import { compose, memberPattern, sinkDirection } from "./edgeGroupWeights.ts";
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
	 * The weight drawn: a listed edge's applied weight (the server's
	 * `graph_weights` value, T25.F06, T25.F07); without the payload a
	 * member's genomic composition (its applied weight marked unavailable),
	 * `null` for an ungrouped plastic edge (runtime state the UI does not
	 * invent), else the own weight.
	 */
	weight: number | null;
	edgeType: "input_to_compute" | "compute_to_compute" | "compute_to_output" | "input_to_output";
	isBackward: boolean;
	/** Whether the edge is an edge-group member (T25.F03), drawn dashed. */
	isMember: boolean;
	/** Whether the edge carries learning attributes (T25.F06). */
	isPlastic: boolean;
	/** The edge's own weight: a member's correction. */
	own: number;
	/** A member's composition: genomic = scale × pattern + own (T25.F03). */
	member?: MemberComposition;
	/** A plastic edge's learning and weights (T25.F06). */
	plastic?: PlasticFacts;
}

/**
 * How a member learns when plastic (T25.F07): its scope and its group's
 * `inherit_scale`, which governs a group learner's inheritance in place of
 * its inert `lamarckian` flag.
 */
export interface MemberLearning {
	scope: Scope;
	inheritScale: boolean;
}

export interface MemberComposition extends MemberLearning {
	group: number;
	scale: number;
	pattern: number;
	/** The weight in force, when the payload carries it; else unavailable. */
	applied?: number;
}

export interface PlasticFacts {
	config: PlasticityConfig;
	genomic: number;
	/** The weight in force, when the payload carries it. */
	applied?: number;
	/** The learned own quantity, once initialized; never for a group-scope member. */
	learned?: number;
	/** Whether the learner is initialized, as `learnerInitialized`. */
	initialized: boolean;
}

/** The field a non-leaf member reads: none (v3-core's `u16::MAX`). */
const NO_FIELD = 0xffff;

/** What the model knows of an edge beyond its endpoints. */
type EdgeFacts = Pick<GraphModelEdge, "weight" | "own" | "member" | "plastic">;

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
	factsOf: (edge: GraphEdge, index: number) => EdgeFacts,
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
					...factsOf(edge, i),
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

/** A lookup of the server's `graph_weights` entry by surface, container and edge index. */
export function edgeWeightLookup(
	graphWeights: NodeGraphWeights | null,
): (surface: "compute" | "sink", container: number, edge: number) => EdgeGenomicWeight | undefined {
	const weights = new Map(
		(graphWeights?.edges ?? []).map((entry) => [
			`${entry.surface}:${entry.container}:${entry.edge}`,
			entry,
		]),
	);
	return (surface, container, edge) => weights.get(`${surface}:${container}:${edge}`);
}

/** A lookup of each group's learned scale in `graph_weights`; absent for an uninitialized scale slot. */
export function learnedScaleLookup(
	graphWeights: NodeGraphWeights | null,
): (group: number) => number | undefined {
	const learned = new Map(
		(graphWeights?.scales ?? []).map((scale) => [scale.group, scale.learned]),
	);
	return (group) => learned.get(group);
}

/**
 * Whether a learner is initialized (T25.F07): a group-scope member stores no
 * own quantity, so it is by its group's scale slot; any other learner by its own.
 */
export function learnerInitialized(
	membership: Membership | undefined,
	entry: EdgeGenomicWeight | undefined,
	learnedScale: (group: number) => number | undefined,
): boolean {
	return membership?.scope === "Group"
		? learnedScale(membership.group) !== undefined
		: entry?.learned !== undefined;
}

export function buildFullGraphModel(
	graphDef: GraphBackendDef,
	inputRefs: InputReference[],
	graphWeights: NodeGraphWeights | null = null,
): FullGraphModel {
	const weightEntry = edgeWeightLookup(graphWeights);
	const learnedScale = learnedScaleLookup(graphWeights);
	const factsOf =
		(surface: "compute" | "sink", container: number, sink?: OutputSinkKind) =>
		(edge: GraphEdge, index: number): EdgeFacts => {
			const entry = weightEntry(surface, container, index);
			const applied = entry === undefined ? {} : { applied: entry.applied };
			const group = edge.membership ? graphDef.groups?.[edge.membership.group] : undefined;
			const member =
				edge.membership && group
					? {
							group: edge.membership.group,
							scale: group.scale,
							// As v3-core's `pattern_at`: a non-leaf member reads no field.
							pattern: memberPattern(
								group,
								inputRefs,
								"InputLeaf" in edge.source ? edge.source.InputLeaf.sub_idx : NO_FIELD,
								sinkDirection(sink),
							),
							scope: edge.membership.scope,
							inheritScale: group.inherit_scale,
							...applied,
						}
					: undefined;
			const plastic = edge.plasticity
				? {
						config: edge.plasticity,
						genomic: entry?.genomic ?? edge.own_weight,
						...applied,
						...(entry?.learned === undefined ? {} : { learned: entry.learned }),
						initialized: learnerInitialized(edge.membership, entry, learnedScale),
					}
				: undefined;
			// A listed edge is drawn at the weight in force (T25.F06, T25.F07);
			// without the payload a member at its genomic composition, as
			// v3-core's `compose` (`cgp/group.rs`), never its own weight alone.
			const weight =
				entry !== undefined
					? entry.applied
					: member
						? compose(edge.own_weight, member.scale, member.pattern)
						: plastic
							? null
							: edge.own_weight;
			return {
				weight,
				own: edge.own_weight,
				...(member ? { member } : {}),
				...(plastic ? { plastic } : {}),
			};
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
		if (!sink || sink.inputs.length === 0) continue;
		allEdges.push(
			...collectEdgesFromSources(
				sink.inputs,
				factsOf("sink", i, sink.kind),
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

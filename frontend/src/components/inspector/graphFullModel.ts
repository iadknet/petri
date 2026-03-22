import type {
	GraphBackendDef,
	GraphEdge,
	GraphSource,
	InputReference,
} from "../../types/genome.ts";
import type { NodeCategory } from "./graphNodeCategories.ts";
import { categorizeComputeNode } from "./graphNodeCategories.ts";
import {
	actionSlotSubtitle,
	formatActionSlotBehavior,
	formatOutputSinkKind,
	getKindLabel,
	outputSinkSubtitle,
} from "./graphNodeFormatters.ts";
import { formatInputRefWithSubIndex } from "./inputRefUtils.ts";

export interface GraphModelNode {
	id: string;
	nodeType: "input" | "compute" | "output_sink" | "action_slot" | "execute_gate";
	category: NodeCategory;
	label: string;
	subtitle?: string;
	arrayIndex: number;
	baseWidth: number;
	height: number;
}

export interface GraphModelEdge {
	id: string;
	sourceId: string;
	targetId: string;
	weight: number;
	edgeType: "input_to_compute" | "compute_to_compute" | "compute_to_output" | "input_to_output";
	isBackward: boolean;
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

function inputLabel(source: GraphSource, inputRefs: InputReference[]): string {
	if ("InputLeaf" in source) {
		const { ref_idx, sub_idx } = source.InputLeaf;
		if (ref_idx === 0xffff) return sub_idx > 0 ? `Dead[${sub_idx}]` : "Dead";
		const ref = inputRefs[ref_idx];
		if (!ref) return `In(${ref_idx})`;
		return formatInputRefWithSubIndex(ref, sub_idx);
	}
	if ("SharedMemory" in source) {
		const { slot, previous } = source.SharedMemory;
		return previous ? `Prev s[${slot}]` : `Read s[${slot}]`;
	}
	return "?";
}

function computeBaseWidth(label: string): number {
	return Math.max(72, Math.min(label.length * 7 + 24, 140));
}

function collectEdgesFromSources(
	edges: GraphEdge[],
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
				weight: edge.weight,
				edgeType,
				isBackward,
			});
		} else {
			const inId = inputNodeId(source);
			if (inId && inputNodeIds.has(inId)) {
				const edgeType = targetType === "compute" ? "input_to_compute" : "input_to_output";
				result.push({
					id: `${edgePrefix}:${i}`,
					sourceId: inId,
					targetId,
					weight: edge.weight,
					edgeType,
					isBackward: false,
				});
			}
		}
	}
	return result;
}

export function buildFullGraphModel(
	graphDef: GraphBackendDef,
	inputRefs: InputReference[],
): FullGraphModel {
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
	for (const slot of graphDef.action_bank) {
		for (const edge of slot.gate_inputs) allGraphEdges.push(edge);
		for (const edge of slot.param_inputs) allGraphEdges.push(edge);
	}
	for (const edge of graphDef.execute_gate.inputs) allGraphEdges.push(edge);

	for (const edge of allGraphEdges) {
		const inId = inputNodeId(edge.source);
		if (inId && !inputNodeIds.has(inId)) {
			inputNodeIds.add(inId);
			const label = inputLabel(edge.source, inputRefs);
			nodes.push({
				id: inId,
				nodeType: "input",
				category: "input",
				label,
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
		const label = getKindLabel(cn.kind, inputRefs);
		const category = categorizeComputeNode(cn.kind);
		nodes.push({
			id: `cn:${i}`,
			nodeType: "compute",
			category,
			label,
			arrayIndex: i,
			baseWidth: computeBaseWidth(label),
			height: 24,
		});
	}

	// Phase 3: Create output nodes (only wired sinks)
	for (let i = 0; i < graphDef.output_sinks.length; i++) {
		const sink = graphDef.output_sinks[i];
		if (!sink || sink.inputs.length === 0) continue;
		const label = formatOutputSinkKind(sink.kind);
		const subtitle = outputSinkSubtitle(sink.kind);
		nodes.push({
			id: `sink:${i}`,
			nodeType: "output_sink",
			category: "output_value",
			label,
			subtitle,
			arrayIndex: i,
			baseWidth: computeBaseWidth(label),
			height: subtitle ? 32 : 24,
		});
	}

	for (let i = 0; i < graphDef.action_bank.length; i++) {
		const slot = graphDef.action_bank[i];
		if (!slot || (slot.gate_inputs.length === 0 && slot.param_inputs.length === 0)) continue;
		const behaviorLabel = formatActionSlotBehavior(slot.behavior);
		const subtitle = actionSlotSubtitle(slot.behavior);
		nodes.push({
			id: `act:${i}`,
			nodeType: "action_slot",
			category: "output_action",
			label: `Act: ${behaviorLabel}`,
			subtitle,
			arrayIndex: i,
			baseWidth: computeBaseWidth(`Act: ${behaviorLabel}`),
			height: 32,
		});
	}

	if (graphDef.execute_gate.inputs.length > 0) {
		nodes.push({
			id: "gate",
			nodeType: "execute_gate",
			category: "output_gate",
			label: "Exec Gate",
			subtitle: "Σ > 0 ∧ queue → emit actions",
			arrayIndex: 0,
			baseWidth: computeBaseWidth("Σ > 0 ∧ queue → emit actions"),
			height: 32,
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
				`sink:${i}`,
				i,
				"output",
				inputNodeIds,
				`sink:${i}`,
				cnCount,
			),
		);
	}

	for (let i = 0; i < graphDef.action_bank.length; i++) {
		const slot = graphDef.action_bank[i];
		if (!slot || (slot.gate_inputs.length === 0 && slot.param_inputs.length === 0)) continue;
		allEdges.push(
			...collectEdgesFromSources(
				slot.gate_inputs,
				`act:${i}`,
				i,
				"output",
				inputNodeIds,
				`act:${i}:gate`,
				cnCount,
			),
		);
		allEdges.push(
			...collectEdgesFromSources(
				slot.param_inputs,
				`act:${i}`,
				i,
				"output",
				inputNodeIds,
				`act:${i}:param`,
				cnCount,
			),
		);
	}

	if (graphDef.execute_gate.inputs.length > 0) {
		allEdges.push(
			...collectEdgesFromSources(
				graphDef.execute_gate.inputs,
				"gate",
				0,
				"output",
				inputNodeIds,
				"gate",
				cnCount,
			),
		);
	}

	return { nodes, edges: allEdges };
}

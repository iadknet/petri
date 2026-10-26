import type { BrainNode } from "../../../types/generated/BrainNode.ts";
import type { BrainView } from "../../../types/generated/BrainView.ts";
import type { MeshReadClass } from "../../../types/generated/MeshReadClass.ts";
import type { MeshWriteClass } from "../../../types/generated/MeshWriteClass.ts";
import type { NodeAnnotation } from "../../../types/generated/NodeAnnotation.ts";
import { formatInputRef } from "../inputRefUtils.ts";
import { type MeshAnalysis, type MeshBackendKind, analyzeMesh } from "./meshAnalysis.ts";
import type { RuntimeIoBadge } from "./runtimeIoSemantics.ts";
import { classifyVmInstruction } from "./runtimeIoSemantics.ts";

export type MeshNodeBadge = RuntimeIoBadge;
export type MeshSemanticRole =
	| "route_selector"
	| "action_writer"
	| "memory_writer"
	| "memory_reader"
	| "payload_emitter"
	| "stateful_mixer"
	| "sensor_reader"
	| "relay";
export type MeshSemanticConfidence = "low" | "medium" | "high";
export type MeshSourceClass = MeshReadClass | null;

export interface MeshNodeRationale {
	role: string;
	source: string | null;
	confidence: string;
}

export interface MeshNodeSemantics {
	nodeId: number;
	backendKind: MeshBackendKind;
	reachable: boolean;
	role: MeshSemanticRole;
	sourceClass: MeshSourceClass;
	confidence: MeshSemanticConfidence;
	shortLabel: string;
	label: string;
	rationale: MeshNodeRationale;
	badges: MeshNodeBadge[];
	readClasses: MeshReadClass[];
	writeClasses: MeshWriteClass[];
	hasStatefulBehavior: boolean;
	slotReads: boolean;
	inputTexts: string[];
	searchTokens: string[];
	searchText: string;
	liveInstructionIndices: number[];
	liveInternalNodeIndices: number[];
}

export interface MeshSemantics {
	nodes: MeshNodeSemantics[];
	nodesById: Map<number, MeshNodeSemantics>;
}

const SOURCE_PRIORITY: MeshReadClass[] = [
	"food",
	"neighbor",
	"barrier",
	"occupancy",
	"introspection",
	"decision",
	"action_queue",
	"upstream",
];

const BADGE_ORDER: MeshNodeBadge[] = ["input", "slot", "action", "route", "output", "stateful"];

export function deriveMeshSemantics(
	genome: BrainView,
	precomputedAnalysis?: MeshAnalysis,
): MeshSemantics {
	const analysis = precomputedAnalysis ?? analyzeMesh(genome);
	const nodes = analysis.nodes.map((analysisNode) =>
		deriveNodeSemantics(analysisNode.node, analysisNode.backendKind),
	);

	return {
		nodes,
		nodesById: new Map(nodes.map((node) => [node.nodeId, node])),
	};
}

function deriveNodeSemantics(node: BrainNode, backendKind: MeshBackendKind): MeshNodeSemantics {
	// The brain view carries each node's annotation (T30.F01).
	const annotation = node.annotation;
	const readClasses = unique(annotation.read_classes);
	const writeClasses = unique(annotation.write_classes);
	const hasStatefulBehavior = annotation.has_stateful_behavior;
	const slotReads = hasSlotRead(node);
	const sourceClass = deriveSourceClass(readClasses);
	const role = deriveRole({
		readClasses,
		writeClasses,
		hasStatefulBehavior,
		slotReads,
	});
	const confidence = deriveConfidence({
		annotation,
		role,
		writeClasses,
		readClasses,
		hasStatefulBehavior,
		slotReads,
	});
	const shortLabel = formatRoleLabel(role, backendKind);
	const label = sourceClass ? `${shortLabel} · ${formatSourceClass(sourceClass)}` : shortLabel;
	const inputTexts = node.input_refs.map((ref) => formatInputRef(ref));
	const badges = deriveBadges({
		readClasses,
		writeClasses,
		hasStatefulBehavior,
		slotReads,
	});
	const liveInstructionIndices = annotation.live_instruction_indices;
	const liveInternalNodeIndices = annotation.live_internal_node_indices;
	const searchTokens = buildSearchTokens({
		nodeId: node.node_id,
		backendKind,
		label,
		shortLabel,
		badges,
		readClasses,
		writeClasses,
		inputTexts,
	});

	return {
		nodeId: node.node_id,
		backendKind,
		reachable: annotation.reachable,
		role,
		sourceClass,
		confidence,
		shortLabel,
		label,
		rationale: {
			role: describeRoleReason(role),
			source: sourceClass ? `Primary source class: ${formatSourceClass(sourceClass)}.` : null,
			confidence: describeConfidenceReason(confidence),
		},
		badges,
		readClasses,
		writeClasses,
		hasStatefulBehavior,
		slotReads,
		inputTexts,
		searchTokens,
		searchText: searchTokens.join(" "),
		liveInstructionIndices,
		liveInternalNodeIndices,
	};
}

function hasSlotRead(node: BrainNode): boolean {
	if ("Vm" in node.backend) {
		return node.backend.Vm.program.some((instruction) => {
			const semantics = classifyVmInstruction(instruction);
			return semantics.readsSlot || semantics.readsPrevSlot;
		});
	}

	if ("Graph" in node.backend) {
		// SharedMemory sources in any edge indicate slot reads
		const graph = node.backend.Graph;
		return graph.compute_nodes.some((cn) => cn.edges.some((edge) => "SharedMemory" in edge.source));
	}

	return false;
}

function deriveSourceClass(readClasses: MeshReadClass[]): MeshSourceClass {
	for (const sourceClass of SOURCE_PRIORITY) {
		if (readClasses.includes(sourceClass)) {
			return sourceClass;
		}
	}
	return null;
}

function deriveRole({
	readClasses,
	writeClasses,
	hasStatefulBehavior,
	slotReads,
}: {
	readClasses: MeshReadClass[];
	writeClasses: MeshWriteClass[];
	hasStatefulBehavior: boolean;
	slotReads: boolean;
}): MeshSemanticRole {
	if (writeClasses.includes("route")) {
		return "route_selector";
	}
	if (writeClasses.includes("action")) {
		return "action_writer";
	}
	if (writeClasses.includes("memory")) {
		return "memory_writer";
	}
	if (slotReads) {
		return "memory_reader";
	}
	if (writeClasses.includes("payload")) {
		return "payload_emitter";
	}
	if (hasStatefulBehavior) {
		return "stateful_mixer";
	}
	if (readClasses.some((sourceClass) => sourceClass !== "upstream")) {
		return "sensor_reader";
	}
	return "relay";
}

function deriveConfidence({
	annotation,
	role,
	writeClasses,
	readClasses,
	hasStatefulBehavior,
	slotReads,
}: {
	annotation: NodeAnnotation;
	role: MeshSemanticRole;
	writeClasses: MeshWriteClass[];
	readClasses: MeshReadClass[];
	hasStatefulBehavior: boolean;
	slotReads: boolean;
}): MeshSemanticConfidence {
	const hasAnnotationSignal =
		annotation.read_classes.length > 0 ||
		annotation.write_classes.length > 0 ||
		annotation.has_stateful_behavior ||
		annotation.live_instruction_indices.length > 0 ||
		annotation.live_internal_node_indices.length > 0;

	if (hasAnnotationSignal && role !== "memory_reader" && role !== "relay") {
		return "high";
	}

	if (
		writeClasses.length > 0 ||
		readClasses.length > 0 ||
		hasStatefulBehavior ||
		slotReads ||
		role !== "relay"
	) {
		return "medium";
	}

	return "low";
}

function deriveBadges({
	readClasses,
	writeClasses,
	hasStatefulBehavior,
	slotReads,
}: {
	readClasses: MeshReadClass[];
	writeClasses: MeshWriteClass[];
	hasStatefulBehavior: boolean;
	slotReads: boolean;
}): MeshNodeBadge[] {
	const badges = new Set<MeshNodeBadge>();
	if (readClasses.length > 0) {
		badges.add("input");
	}
	if (writeClasses.includes("memory") || slotReads) {
		badges.add("slot");
	}
	if (writeClasses.includes("action")) {
		badges.add("action");
	}
	if (writeClasses.includes("route")) {
		badges.add("route");
	}
	if (writeClasses.includes("payload")) {
		badges.add("output");
	}
	if (hasStatefulBehavior) {
		badges.add("stateful");
	}
	return BADGE_ORDER.filter((badge) => badges.has(badge));
}

function formatRoleLabel(role: MeshSemanticRole, backendKind: MeshBackendKind): string {
	switch (role) {
		case "route_selector":
			return "Route Selector";
		case "action_writer":
			return "Action Writer";
		case "memory_writer":
			return "Memory Writer";
		case "memory_reader":
			return "Memory Reader";
		case "payload_emitter":
			return "Bus Writer";
		case "stateful_mixer":
			return "Stateful Mixer";
		case "sensor_reader":
			return "Sensor Reader";
		case "relay":
			return backendKind === "vm" ? "VM Relay" : "Graph Relay";
	}
}

function formatSourceClass(sourceClass: MeshReadClass): string {
	switch (sourceClass) {
		case "food":
			return "Food";
		case "neighbor":
			return "Neighbor";
		case "barrier":
			return "Barrier";
		case "occupancy":
			return "Occupancy";
		case "introspection":
			return "Introspection";
		case "upstream":
			return "Bus";
		case "action_queue":
			return "Action Queue";
		case "decision":
			return "Decision";
	}
}

function describeRoleReason(role: MeshSemanticRole): string {
	switch (role) {
		case "route_selector":
			return "Scores route slots to choose which mesh node runs next.";
		case "action_writer":
			return "Votes on actions or sets action parameters.";
		case "memory_writer":
			return "Writes shared-memory slots.";
		case "memory_reader":
			return "Reads shared-memory slots without owning a stronger write role.";
		case "payload_emitter":
			return "Writes bus slots that the next mesh node reads.";
		case "stateful_mixer":
			return "Uses stateful graph internals to carry signal over time.";
		case "sensor_reader":
			return "Primarily reads world or introspection inputs.";
		case "relay":
			return "Relays inputs downstream without a dominant semantic role.";
	}
}

function describeConfidenceReason(confidence: MeshSemanticConfidence): string {
	switch (confidence) {
		case "high":
			return "Derived from server-provided mesh annotations.";
		case "medium":
			return "Inferred from genome structure and instruction usage.";
		case "low":
			return "Fallback relay classification from backend type only.";
	}
}

function buildSearchTokens({
	nodeId,
	backendKind,
	label,
	shortLabel,
	badges,
	readClasses,
	writeClasses,
	inputTexts,
}: {
	nodeId: number;
	backendKind: MeshBackendKind;
	label: string;
	shortLabel: string;
	badges: MeshNodeBadge[];
	readClasses: MeshReadClass[];
	writeClasses: MeshWriteClass[];
	inputTexts: string[];
}): string[] {
	const rawTerms = [
		String(nodeId),
		`#${nodeId}`,
		`node ${nodeId}`,
		label,
		shortLabel,
		backendKind,
		backendKind === "vm" ? "VM" : "Graph",
		...badges,
		...readClasses,
		...writeClasses,
		...(writeClasses.includes("memory") || badges.includes("slot")
			? ["slot", "shared memory"]
			: []),
		...inputTexts,
	];

	return unique(
		rawTerms.flatMap((value) => {
			const normalized = normalizeSearchText(value);
			return [normalized, ...tokenizeSearchText(normalized)];
		}),
	);
}

function normalizeSearchText(value: unknown): string {
	return String(value ?? "")
		.trim()
		.toLowerCase()
		.replace(/\s+/g, " ");
}

export function tokenizeSearchText(value: string): string[] {
	return normalizeSearchText(value)
		.split(/[^a-z0-9#]+/i)
		.map((term) => term.trim())
		.filter(Boolean);
}

function unique<T>(values: Iterable<T>): T[] {
	return [...new Set(values)];
}

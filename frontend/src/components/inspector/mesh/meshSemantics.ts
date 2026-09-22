import type {
	CreatureDetail,
	CreatureMeshAnnotation,
	MeshReadClass,
	MeshWriteClass,
} from "../../../types/creature-detail.ts";
import type { CreatureGenome, InputReference, NodeGenome } from "../../../types/genome.ts";
import { formatInputRef, parseWorldInputRef } from "../inputRefUtils.ts";
import { type MeshAnalysis, type MeshBackendKind, analyzeMesh } from "./meshAnalysis.ts";
import type { RuntimeIoBadge } from "./runtimeIoSemantics.ts";
import { classifyComputeNodeKind, classifyVmInstruction } from "./runtimeIoSemantics.ts";

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
	"action_queue",
	"upstream",
];

const BADGE_ORDER: MeshNodeBadge[] = ["input", "slot", "action", "route", "output", "stateful"];

type CreatureMeshAnnotations = CreatureDetail["mesh_annotations"] | null | undefined;

export function deriveMeshSemantics(
	genome: CreatureGenome,
	annotations: CreatureMeshAnnotations = null,
	precomputedAnalysis?: MeshAnalysis,
): MeshSemantics {
	const analysis = precomputedAnalysis ?? analyzeMesh(genome);
	const annotationById = new Map(
		(annotations ?? []).map((annotation) => [annotation.node_id, annotation]),
	);
	const nodes = analysis.nodes.map((analysisNode) =>
		deriveNodeSemantics(
			analysisNode.node,
			analysisNode.backendKind,
			analysisNode.reachable,
			annotationById.get(analysisNode.id),
		),
	);

	return {
		nodes,
		nodesById: new Map(nodes.map((node) => [node.nodeId, node])),
	};
}

function deriveNodeSemantics(
	node: NodeGenome,
	backendKind: MeshBackendKind,
	reachableFallback: boolean,
	annotation?: CreatureMeshAnnotation,
): MeshNodeSemantics {
	const readClasses = unique([
		...(annotation?.read_classes ?? []),
		...inferReadClasses(node.input_refs),
	]);
	const writeClasses = unique([...(annotation?.write_classes ?? []), ...inferWriteClasses(node)]);
	const hasStatefulBehavior = annotation?.has_stateful_behavior ?? inferStatefulBehavior(node);
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
	const liveInstructionIndices =
		annotation?.live_instruction_indices ??
		("Vm" in node.backend_def ? node.backend_def.Vm.program.map((_, index) => index) : []);
	const liveInternalNodeIndices =
		annotation?.live_internal_node_indices ??
		("Graph" in node.backend_def
			? node.backend_def.Graph.compute_nodes.map((_, index) => index)
			: []);
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
		reachable: annotation?.reachable ?? reachableFallback,
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

function inferReadClasses(inputRefs: InputReference[]): MeshReadClass[] {
	const classes = new Set<MeshReadClass>();
	for (const inputRef of inputRefs) {
		if (typeof inputRef === "string") {
			if (inputRef === "ActionQueue") {
				classes.add("action_queue");
			}
			continue;
		}

		if ("World" in inputRef) {
			const worldKey = parseWorldInputRef(inputRef.World).key;
			if (worldKey === "NeighborFoodRing") {
				classes.add("neighbor");
				classes.add("food");
			} else if (worldKey === "NeighborBarrierRing") {
				classes.add("neighbor");
				classes.add("barrier");
			} else if (worldKey === "NeighborOccupiedRing") {
				classes.add("neighbor");
				classes.add("occupancy");
			} else if (String(worldKey).toLowerCase().includes("food")) {
				classes.add("food");
			}
			continue;
		}

		if ("StaticIntrospection" in inputRef || "DynamicIntrospection" in inputRef) {
			classes.add("introspection");
			continue;
		}

		if ("UpstreamSlot" in inputRef) {
			classes.add("upstream");
		}
	}

	return [...classes];
}

function inferWriteClasses(node: NodeGenome): MeshWriteClass[] {
	const classes = new Set<MeshWriteClass>();

	if ("Vm" in node.backend_def) {
		for (const instruction of node.backend_def.Vm.program) {
			const semantics = classifyVmInstruction(instruction);
			if (semantics.writesRoute) {
				classes.add("route");
			}
			if (semantics.writesAction) {
				classes.add("action");
			}
			if (semantics.writesPayload) {
				classes.add("payload");
			}
			if (semantics.writesSlot) {
				classes.add("memory");
			}
		}
	}

	if ("Graph" in node.backend_def) {
		const graph = node.backend_def.Graph;

		// Derive write classes from fixed structural outputs (wired sinks only)
		for (const sink of graph.output_sinks) {
			if (sink.inputs.length === 0) continue;
			const kind = sink.kind;
			if (typeof kind !== "string" && "RouterGate" in kind) {
				classes.add("route");
			} else if ("CustomOutput" in kind) {
				classes.add("payload");
			} else if ("WriteSlot" in kind || "ClearSlot" in kind) {
				classes.add("memory");
			} else if ("ActionVote" in kind || "ActionParam" in kind) {
				// The inert vote surface (T19.F03) writes the action channel.
				classes.add("action");
			}
		}

		// Action bank contributes action writes
		for (const slot of graph.action_bank) {
			if (slot.gate_inputs.length > 0 || slot.param_inputs.length > 0) {
				classes.add("action");
			}
		}
	}

	return [...classes];
}

function inferStatefulBehavior(node: NodeGenome): boolean {
	if ("Vm" in node.backend_def) {
		return node.backend_def.Vm.program.some((instruction) => {
			const semantics = classifyVmInstruction(instruction);
			return (
				semantics.stateful || semantics.readsSlot || semantics.readsPrevSlot || semantics.writesSlot
			);
		});
	}

	if (!("Graph" in node.backend_def)) {
		return false;
	}

	const graph = node.backend_def.Graph;
	// Compute nodes may be stateful (DecayIntegrator, Momentum, etc.)
	const hasStatefulCompute = graph.compute_nodes.some((cn) => {
		const semantics = classifyComputeNodeKind(cn.kind);
		return semantics.stateful;
	});
	// SharedMemory sources in edges indicate slot reads
	const hasSharedMemoryRead = graph.compute_nodes.some((cn) =>
		cn.inputs.some((edge) => "SharedMemory" in edge.source),
	);
	// WriteSlot/ClearSlot sinks indicate slot writes
	const hasSlotWrite = graph.output_sinks.some(
		(sink) =>
			sink.inputs.length > 0 &&
			typeof sink.kind !== "string" &&
			("WriteSlot" in sink.kind || "ClearSlot" in sink.kind),
	);
	return hasStatefulCompute || hasSharedMemoryRead || hasSlotWrite;
}

function hasSlotRead(node: NodeGenome): boolean {
	if ("Vm" in node.backend_def) {
		return node.backend_def.Vm.program.some((instruction) => {
			const semantics = classifyVmInstruction(instruction);
			return semantics.readsSlot || semantics.readsPrevSlot;
		});
	}

	if ("Graph" in node.backend_def) {
		// SharedMemory sources in any edge indicate slot reads
		const graph = node.backend_def.Graph;
		return graph.compute_nodes.some((cn) =>
			cn.inputs.some((edge) => "SharedMemory" in edge.source),
		);
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
	annotation?: CreatureMeshAnnotation;
	role: MeshSemanticRole;
	writeClasses: MeshWriteClass[];
	readClasses: MeshReadClass[];
	hasStatefulBehavior: boolean;
	slotReads: boolean;
}): MeshSemanticConfidence {
	const hasAnnotationSignal = Boolean(
		annotation &&
			(annotation.read_classes.length > 0 ||
				annotation.write_classes.length > 0 ||
				annotation.has_stateful_behavior ||
				(annotation.live_instruction_indices?.length ?? 0) > 0 ||
				(annotation.live_internal_node_indices?.length ?? 0) > 0),
	);

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
			return "Slot Writer";
		case "memory_reader":
			return "Slot Reader";
		case "payload_emitter":
			return "Payload Emitter";
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
			return "Upstream";
		case "action_queue":
			return "Action Queue";
	}
}

function describeRoleReason(role: MeshSemanticRole): string {
	switch (role) {
		case "route_selector":
			return "Writes route targets for downstream routing.";
		case "action_writer":
			return "Writes world actions or action metadata.";
		case "memory_writer":
			return "Writes shared-memory slots.";
		case "memory_reader":
			return "Reads shared-memory slots without owning a stronger write role.";
		case "payload_emitter":
			return "Emits payload/custom output signals.";
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

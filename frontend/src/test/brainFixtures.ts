import type { BrainBundle } from "../types/generated/BrainBundle.ts";
import type { BrainEdge } from "../types/generated/BrainEdge.ts";
import type { BrainView } from "../types/generated/BrainView.ts";
import type { DirectedVoteKind } from "../types/generated/DirectedVoteKind.ts";
import type { EdgeLearner } from "../types/generated/EdgeLearner.ts";
import type { GraphBrain } from "../types/generated/GraphBrain.ts";
import type { GraphSource } from "../types/generated/GraphSource.ts";
import type { InputKey } from "../types/generated/InputKey.ts";
import type { InputRef } from "../types/generated/InputRef.ts";
import type { NodeAnnotation } from "../types/generated/NodeAnnotation.ts";
import type { NodeLearning } from "../types/generated/NodeLearning.ts";

/** A node annotation with nothing read, written or live, unless overridden. */
export function annotation(overrides: Partial<NodeAnnotation> = {}): NodeAnnotation {
	return {
		reachable: true,
		read_classes: [],
		write_classes: [],
		has_stateful_behavior: false,
		live_instruction_indices: [],
		live_internal_node_indices: [],
		...overrides,
	};
}

/** `brain` with the nodes in `byId` annotated (unlisted fields as `annotation()`). */
export function withAnnotations(
	brain: BrainView,
	byId: Record<number, Partial<NodeAnnotation>>,
): BrainView {
	return {
		...brain,
		nodes: brain.nodes.map((node) => {
			const overrides = byId[node.node_id];
			return overrides === undefined ? node : { ...node, annotation: annotation(overrides) };
		}),
	};
}

/** A static, ungrouped edge: genomic and applied equal its own weight. */
export function edge(source: GraphSource, own = 1, overrides: Partial<BrainEdge> = {}): BrainEdge {
	return { source, own, genomic: own, applied: own, ...overrides };
}

/** A static edge reading `input_refs[refIdx]` field `subIdx`. */
export function leafEdge(
	refIdx: number,
	subIdx: number,
	own = 1,
	overrides: Partial<BrainEdge> = {},
): BrainEdge {
	return edge({ InputLeaf: { ref_idx: refIdx, sub_idx: subIdx } }, own, overrides);
}

/** A static edge reading compute node `index`. */
export function computeEdge(index: number, own = 1): BrainEdge {
	return edge({ ComputeNode: index }, own);
}

/** A world input reference by key, with a food type for a typed food sensor. */
export function worldRef(key: InputKey, foodType?: number): InputRef {
	return foodType === undefined
		? { family: "world", key }
		: { family: "world", key, food_type: foodType };
}

/** An upstream-slot input reference. */
export function upstreamRef(slot: number): InputRef {
	return { family: "upstream", key: "UpstreamSlot", slot };
}

/** A Graph backend with no edges and no bundles, unless given. */
export function graphBrain(overrides: Partial<GraphBrain> = {}): GraphBrain {
	return { compute_nodes: [], output_sinks: [], bundles: [], ...overrides };
}

/** A Hebbian ungrouped learner, uninitialized and Darwinian unless overridden. */
export function learner(overrides: Partial<EdgeLearner> = {}): EdgeLearner {
	return {
		rule: "Classic",
		learning_rate: 0.1,
		weight_clamp: 2,
		retention: 1,
		scope: "ungrouped",
		initialized: false,
		inherits: false,
		...overrides,
	};
}

/** A ring bundle on `ringRef` into `kind` (default `Move`) at `offset`
 * (default 0) and scale 0, with all eight members present. */
export function ringBundle(
	ringRef: number,
	{
		kind = "Move",
		offset = 0,
		...overrides
	}: Partial<BrainBundle> & { kind?: DirectedVoteKind; offset?: number } = {},
): BrainBundle {
	return {
		source: { Sensor: { ref_idx: ringRef, field_set: 0 } },
		wiring: { Ring: { kind, offset } },
		scale: 0,
		inherit_scale: false,
		retention: 1,
		coordinates: Array.from({ length: 8 }, (_, c) => ({
			field: c,
			site: { Direction: (c + offset) % 8 },
			present: true,
		})),
		...overrides,
	};
}

/** A node's empty `learning` entry. */
export function nodeLearning(nodeId: number, overrides: Partial<NodeLearning> = {}): NodeLearning {
	return { node_id: nodeId, edges: [], scales: [], ...overrides };
}

import type { SamplerPosition } from "../../../stores/samplePlayback.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import type { ExecutionSample } from "../../../types/trace.ts";
import { formatMeshEdgeId, getMeshEdgeOccurrenceIndex } from "./meshAnalysis.ts";

export interface TraceFocus {
	activeNodeId: number | null;
	activeEdgeId: string | null;
	routeTargetId: number | null;
	currentTickNumber: number | null;
	currentHopIndex: number | null;
	backendLabel: "VM" | "Graph" | null;
}

export const EMPTY_TRACE_FOCUS: TraceFocus = {
	activeNodeId: null,
	activeEdgeId: null,
	routeTargetId: null,
	currentTickNumber: null,
	currentHopIndex: null,
	backendLabel: null,
};

export function deriveTraceFocus(
	genome: CreatureGenome | null,
	sample: ExecutionSample | null,
	position: SamplerPosition,
): TraceFocus {
	if (!genome || !sample) {
		return EMPTY_TRACE_FOCUS;
	}

	const currentTick = sample.ticks[position.tickIndex];
	const currentHop = currentTick?.hops[position.hopIndex];
	if (!currentTick || !currentHop) {
		return EMPTY_TRACE_FOCUS;
	}

	const node = genome.nodes.find((candidate) => candidate.node_id === currentHop.node_id);
	const routeTargetIdx = currentHop.route.resolved_target_index;
	const routeTargetId =
		node && routeTargetIdx >= 0 && routeTargetIdx < node.targets.length
			? (node.targets[routeTargetIdx] ?? null)
			: null;

	return {
		activeNodeId: currentHop.node_id,
		activeEdgeId:
			routeTargetId !== null
				? formatMeshEdgeId(
						currentHop.node_id,
						routeTargetId,
						getMeshEdgeOccurrenceIndex(node?.targets ?? [], routeTargetIdx),
					)
				: null,
		routeTargetId,
		currentTickNumber: currentTick.tick_number,
		currentHopIndex: currentHop.hop_index,
		backendLabel:
			typeof currentHop.backend_trace === "string"
				? null
				: "Vm" in currentHop.backend_trace
					? "VM"
					: "Graph",
	};
}

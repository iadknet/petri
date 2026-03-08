import type { MeshHopTrace } from "../../types/trace.ts";
import { GraphExecutionView } from "../inspector/GraphExecutionView.tsx";
import { VmExecutionView } from "../inspector/VmExecutionView.tsx";

interface NodeExecutionTraceProps {
	executionHop: {
		hop: MeshHopTrace;
		detailIndex: number;
	} | null;
}

export function NodeExecutionTrace({ executionHop }: NodeExecutionTraceProps) {
	if (!executionHop) {
		return null;
	}

	const { hop, detailIndex } = executionHop;

	if ("Vm" in hop.backend_trace) {
		return (
			<VmExecutionView
				trace={hop.backend_trace.Vm}
				inputRefs={hop.input_refs}
				upstreamSlots={hop.upstream_slots}
				detailIndex={detailIndex}
			/>
		);
	}

	if ("Graph" in hop.backend_trace) {
		return (
			<GraphExecutionView
				trace={hop.backend_trace.Graph}
				inputRefs={hop.input_refs}
				upstreamSlots={hop.upstream_slots}
				detailIndex={detailIndex}
			/>
		);
	}

	return null;
}

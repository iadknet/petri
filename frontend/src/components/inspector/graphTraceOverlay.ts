import type { GraphTrace } from "../../types/trace.ts";

export interface NodeTraceData {
	// Compute nodes
	initialValue?: number;
	outputValue?: number;
	stateChange?: string;
	// Output sinks
	weightedSum?: number;
	appliedValue?: number;
	applied?: boolean;
}

const STATEFUL_KINDS = new Set(["DecayIntegrator", "Momentum", "Oscillator", "AdaptiveGain"]);

export function buildComputeTraceOverlay(
	trace: GraphTrace,
	detailIndex: number,
): Map<string, NodeTraceData> {
	const result = new Map<string, NodeTraceData>();
	const firstPass = trace.passes[0];
	const currentPass = trace.passes[detailIndex];
	if (!currentPass) return result;

	const initialByIndex = new Map<number, number>();
	if (firstPass) {
		for (const evalNode of firstPass.node_evaluations) {
			initialByIndex.set(evalNode.node_index, evalNode.output);
		}
	}

	for (const evalNode of currentPass.node_evaluations) {
		const isStateful = STATEFUL_KINDS.has(evalNode.kind);
		const stateChanged = isStateful && evalNode.state_before !== evalNode.state_after;
		result.set(`cn:${evalNode.node_index}`, {
			initialValue: initialByIndex.get(evalNode.node_index),
			outputValue: evalNode.output,
			stateChange: stateChanged
				? `${evalNode.state_before.toFixed(2)}\u2192${evalNode.state_after.toFixed(2)}${trace.temporal_committed ? "" : " (not committed)"}`
				: undefined,
		});
	}

	return result;
}

export function buildOutputTraceOverlay(trace: GraphTrace): Map<string, NodeTraceData> {
	const result = new Map<string, NodeTraceData>();

	const sinks = trace.output_sinks ?? [];
	for (let i = 0; i < sinks.length; i++) {
		const sink = sinks[i];
		if (!sink) continue;
		result.set(`sink:${i}`, {
			weightedSum: sink.weighted_sum,
			appliedValue: sink.applied_value,
			applied: sink.applied,
		});
	}

	return result;
}

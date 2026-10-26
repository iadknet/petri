import { Suspense, lazy, useMemo } from "react";
import type { GraphTracePayload } from "../../types/generated/GraphTracePayload.ts";
import type { MeshHopTracePayload } from "../../types/generated/MeshHopTracePayload.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { VmTracePayload } from "../../types/generated/VmTracePayload.ts";
import { NodeBackendDetail } from "./NodeBackendDetail.tsx";
import { NodeConnections } from "./NodeConnections.tsx";
import { NodeExecutionTrace } from "./NodeExecutionTrace.tsx";
import { NodeIdentity } from "./NodeIdentity.tsx";
import { NodeMemorySlots } from "./NodeMemorySlots.tsx";
import type { MeshAnalyzedNode } from "./mesh/meshAnalysis.ts";
import { summarizeBackendDef } from "./mesh/meshPresentation.ts";
import type { MeshNodeSemantics } from "./mesh/meshSemantics.ts";

const GraphInternalsViz = lazy(async () => {
	const module = await import("./GraphInternalsViz.tsx");
	return { default: module.GraphInternalsViz };
});

interface NodeInspectorProps {
	node: MeshAnalyzedNode | null;
	semantics: MeshNodeSemantics | null;
	/** This node's member genomic weights, when it carries an edge group. */
	learning?: NodeLearning | null;
	sharedMemory: number[] | null;
	executionHop: { hop: MeshHopTracePayload; detailIndex: number } | null;
}

export function NodeInspector({
	node,
	semantics,
	learning = null,
	sharedMemory,
	executionHop,
}: NodeInspectorProps) {
	// Hooks must be called unconditionally (Rules of Hooks)
	const isGraph = node ? "Graph" in node.node.backend : false;
	const isVm = node ? "Vm" in node.node.backend : false;

	const graphTrace: GraphTracePayload | null = useMemo(() => {
		if (!executionHop || !isGraph) return null;
		const bt = executionHop.hop.backend_trace;
		if ("Graph" in bt) return bt.Graph;
		return null;
	}, [executionHop, isGraph]);

	const vmTrace: VmTracePayload | null = useMemo(() => {
		if (!executionHop || !isVm) return null;
		const bt = executionHop.hop.backend_trace;
		if ("Vm" in bt) return bt.Vm;
		return null;
	}, [executionHop, isVm]);

	if (!node) {
		return (
			<div className="px-3 py-3 text-center">
				<p className="text-[10px] font-mono text-slate-400">Select a node to inspect</p>
			</div>
		);
	}

	const badges = semantics?.badges ?? [];
	const summary = summarizeBackendDef(node.node.backend);
	const hasMemoryWrite = semantics?.writeClasses.includes("memory") ?? false;

	return (
		<div className="divide-y divide-white/6">
			<NodeIdentity
				nodeId={node.id}
				backendKind={node.backendKind}
				backendSummary={summary}
				isEntry={node.isEntry}
				reachable={node.reachable}
				semanticLabel={semantics?.label ?? null}
				badges={badges}
			/>
			<NodeConnections inputRefs={node.node.input_refs} targets={node.node.targets} />
			{isGraph && "Graph" in node.node.backend ? (
				<Suspense
					fallback={
						<div className="px-3 py-4 text-center">
							<p className="text-[10px] font-mono text-slate-400">Loading graph viz…</p>
						</div>
					}
				>
					<GraphInternalsViz
						graphDef={node.node.backend.Graph}
						learning={learning}
						liveIndices={semantics?.liveInternalNodeIndices ?? []}
						inputRefs={node.node.input_refs}
						targets={node.node.targets}
						routeTargetIdx={executionHop?.hop.route?.selected_target_idx ?? null}
						trace={graphTrace}
						detailIndex={executionHop?.detailIndex ?? 0}
					/>
				</Suspense>
			) : (
				<>
					<NodeBackendDetail
						backendDef={node.node.backend}
						inputRefs={node.node.input_refs}
						liveInstructionIndices={semantics?.liveInstructionIndices ?? []}
						liveInternalNodeIndices={semantics?.liveInternalNodeIndices ?? []}
						vmTrace={vmTrace}
						detailIndex={executionHop?.detailIndex ?? 0}
					/>
					<NodeExecutionTrace executionHop={executionHop} />
				</>
			)}
			<NodeMemorySlots sharedMemory={sharedMemory} hasMemoryWrite={hasMemoryWrite} />
		</div>
	);
}

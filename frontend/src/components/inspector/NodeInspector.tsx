import { Suspense, lazy, useMemo } from "react";
import type { GraphTrace, VmTrace } from "../../types/trace.ts";
import type { MeshHopTrace } from "../../types/trace.ts";
import { NodeBackendDetail } from "./NodeBackendDetail.tsx";
import { NodeConnections } from "./NodeConnections.tsx";
import { NodeExecutionTrace } from "./NodeExecutionTrace.tsx";
import { NodeIdentity } from "./NodeIdentity.tsx";
import { NodeMemorySlots } from "./NodeMemorySlots.tsx";
import type { MeshAnalyzedNode } from "./mesh/meshAnalysis.ts";
import { collectNodeBadges, summarizeBackendDef } from "./mesh/meshPresentation.ts";
import type { MeshNodeSemantics } from "./mesh/meshSemantics.ts";

const GraphInternalsViz = lazy(async () => {
	const module = await import("./GraphInternalsViz.tsx");
	return { default: module.GraphInternalsViz };
});

interface NodeInspectorProps {
	node: MeshAnalyzedNode | null;
	semantics: MeshNodeSemantics | null;
	sharedMemory: number[] | null;
	executionHop: { hop: MeshHopTrace; detailIndex: number } | null;
}

export function NodeInspector({ node, semantics, sharedMemory, executionHop }: NodeInspectorProps) {
	// Hooks must be called unconditionally (Rules of Hooks)
	const isGraph = node ? "Graph" in node.node.backend_def : false;
	const isVm = node ? "Vm" in node.node.backend_def : false;

	const graphTrace: GraphTrace | null = useMemo(() => {
		if (!executionHop || !isGraph) return null;
		const bt = executionHop.hop.backend_trace;
		if ("Graph" in bt) return bt.Graph;
		return null;
	}, [executionHop, isGraph]);

	const vmTrace: VmTrace | null = useMemo(() => {
		if (!executionHop || !isVm) return null;
		const bt = executionHop.hop.backend_trace;
		if ("Vm" in bt) return bt.Vm;
		return null;
	}, [executionHop, isVm]);

	if (!node) {
		return (
			<div className="px-3 py-3 text-center">
				<p className="text-[10px] font-mono text-slate-500">Select a node to inspect</p>
			</div>
		);
	}

	const badges = semantics?.badges ?? collectNodeBadges(node.node);
	const summary = summarizeBackendDef(node.node.backend_def);
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
			{isGraph && "Graph" in node.node.backend_def ? (
				<Suspense
					fallback={
						<div className="px-3 py-4 text-center">
							<p className="text-[10px] font-mono text-slate-500">Loading graph viz…</p>
						</div>
					}
				>
					<GraphInternalsViz
						graphDef={node.node.backend_def.Graph}
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
						backendDef={node.node.backend_def}
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

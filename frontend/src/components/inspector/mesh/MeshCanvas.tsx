import { Suspense, lazy } from "react";
import type { MeshFocusMode } from "../../../stores/inspectorWorkspace.ts";
import type { MeshViewportCommand } from "./MeshWorkspaceController.ts";
import type { MeshAnalysis } from "./meshAnalysis.ts";
import type { MeshLayout } from "./meshLayout.ts";
import type { MeshNodeSemantics } from "./meshSemantics.ts";

const MeshFlowCanvas = lazy(async () => {
	const module = await import("./MeshFlowCanvas.tsx");
	return { default: module.MeshFlowCanvas };
});

interface MeshCanvasProps {
	analysis: MeshAnalysis;
	semanticsById: Map<number, MeshNodeSemantics> | null;
	layout: MeshLayout | null;
	selectedNodeId: number | null;
	activeNodeId: number | null;
	activeEdgeId: string | null;
	dimmedNodeIds: Set<number>;
	focusMode: MeshFocusMode;
	viewportCommand: MeshViewportCommand | null;
	onSelectNode: (nodeId: number) => void;
	complexity?: number;
	genomeSize?: number;
}

export function MeshCanvas({
	analysis,
	semanticsById,
	layout,
	selectedNodeId,
	activeNodeId,
	activeEdgeId,
	dimmedNodeIds,
	focusMode,
	viewportCommand,
	onSelectNode,
	complexity,
	genomeSize,
}: MeshCanvasProps) {
	if (!layout) {
		return (
			<div className="flex h-full items-center justify-center bg-slate-950">
				<p className="text-sm text-slate-500">Computing stable mesh layout…</p>
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col bg-slate-950">
			<div className="flex-1 min-h-0 overflow-hidden">
				<Suspense
					fallback={
						<div className="flex h-full items-center justify-center bg-slate-950">
							<p className="text-sm text-slate-500">Loading topology canvas…</p>
						</div>
					}
				>
					<MeshFlowCanvas
						analysis={analysis}
						semanticsById={semanticsById}
						layout={layout}
						selectedNodeId={selectedNodeId}
						activeNodeId={activeNodeId}
						activeEdgeId={activeEdgeId}
						dimmedNodeIds={dimmedNodeIds}
						focusMode={focusMode}
						viewportCommand={viewportCommand}
						onSelectNode={onSelectNode}
						complexity={complexity}
						genomeSize={genomeSize}
					/>
				</Suspense>
			</div>
		</div>
	);
}

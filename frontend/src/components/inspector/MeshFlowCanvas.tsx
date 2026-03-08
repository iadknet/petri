import "@xyflow/react/dist/style.css";

import {
	type EdgeTypes,
	MiniMap,
	type NodeTypes,
	Panel,
	ReactFlow,
	ReactFlowProvider,
	useReactFlow,
} from "@xyflow/react";
import { memo, useEffect, useMemo } from "react";
import type { MeshFocusMode } from "../../stores/inspectorWorkspace.ts";
import { MeshFlowEdge } from "./MeshFlowEdge.tsx";
import { MeshFlowNode } from "./MeshFlowNode.tsx";
import type { MeshViewportCommand } from "./MeshWorkspaceController.ts";
import type { MeshAnalysis } from "./mesh/meshAnalysis.ts";
import { buildMeshFlowScene } from "./mesh/meshFlowAdapter.ts";
import type {
	MeshFlowEdge as MeshFlowEdgeModel,
	MeshFlowNodeData,
	MeshFlowNode as MeshFlowNodeModel,
} from "./mesh/meshFlowAdapter.ts";
import type { MeshLayout } from "./mesh/meshLayout.ts";
import type { MeshNodeSemantics } from "./mesh/meshSemantics.ts";

const NODE_TYPES = {
	meshNode: MeshFlowNode,
} satisfies NodeTypes;

const EDGE_TYPES = {
	meshEdge: MeshFlowEdge,
} satisfies EdgeTypes;

interface MeshFlowCanvasProps {
	analysis: MeshAnalysis;
	semanticsById: Map<number, MeshNodeSemantics> | null;
	layout: MeshLayout;
	selectedNodeId: number | null;
	activeNodeId: number | null;
	activeEdgeId: string | null;
	dimmedNodeIds: Set<number>;
	focusMode: MeshFocusMode;
	viewportCommand: MeshViewportCommand | null;
	onSelectNode: (nodeId: number) => void;
}

export function MeshFlowCanvas(props: MeshFlowCanvasProps) {
	return (
		<ReactFlowProvider>
			<MeshFlowScene {...props} />
		</ReactFlowProvider>
	);
}

const MeshFlowScene = memo(function MeshFlowScene({
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
}: MeshFlowCanvasProps) {
	const reactFlow = useReactFlow<MeshFlowNodeModel, MeshFlowEdgeModel>();
	const scene = useMemo(
		() =>
			buildMeshFlowScene({
				analysis,
				layout,
				semanticsById,
				selectedNodeId,
				activeNodeId,
				activeEdgeId,
				dimmedNodeIds,
			}),
		[activeEdgeId, activeNodeId, analysis, dimmedNodeIds, layout, selectedNodeId, semanticsById],
	);

	useEffect(() => {
		if (!layout.topologyKey) {
			return;
		}

		requestAnimationFrame(() => {
			reactFlow.fitView({ duration: 180, padding: 0.18 });
		});
	}, [layout.topologyKey, reactFlow]);

	useEffect(() => {
		if (!viewportCommand) {
			return;
		}

		requestAnimationFrame(() => {
			if (viewportCommand.kind === "reset") {
				reactFlow.setViewport({ x: 0, y: 0, zoom: 1 }, { duration: 0 });
				reactFlow.fitView({ duration: 180, padding: 0.18 });
				return;
			}
			if (viewportCommand.kind === "center-node") {
				const targetNode = layout.nodesById.get(viewportCommand.nodeId);
				if (!targetNode) {
					return;
				}
				reactFlow.setCenter(
					targetNode.x + targetNode.width / 2,
					targetNode.y + targetNode.height / 2,
					{
						zoom: Math.max(reactFlow.getZoom(), 0.9),
						duration: 180,
					},
				);
				return;
			}
			reactFlow.fitView({ duration: 180, padding: 0.18 });
		});
	}, [layout.nodesById, reactFlow, viewportCommand]);

	return (
		<div className="h-full min-h-[24rem]">
			<ReactFlow<MeshFlowNodeModel, MeshFlowEdgeModel>
				nodes={scene.nodes}
				edges={scene.edges}
				nodeTypes={NODE_TYPES}
				edgeTypes={EDGE_TYPES}
				onNodeClick={(_, node) => onSelectNode(Number(node.id))}
				nodesDraggable={false}
				nodesConnectable={false}
				nodesFocusable={false}
				elementsSelectable={false}
				proOptions={{ hideAttribution: true }}
				panOnScroll={false}
				zoomOnScroll={false}
				preventScrolling={false}
				zoomOnDoubleClick
				minZoom={0.35}
				maxZoom={1.8}
				fitView
				defaultViewport={{ x: 0, y: 0, zoom: 1 }}
				className="rounded-b-[1.5rem] bg-[radial-gradient(circle_at_top_left,rgba(56,189,248,0.1),transparent_32%),linear-gradient(180deg,rgba(15,23,42,0.94),rgba(2,6,23,0.92))]"
			>
				<MiniMap<MeshFlowNodeModel>
					pannable
					zoomable
					maskColor="rgba(2,6,23,0.42)"
					nodeColor={(node) => getMiniMapNodeColor(node.data)}
					nodeStrokeColor={(node) => getMiniMapNodeStroke(node.data)}
					nodeBorderRadius={10}
					className="!bottom-4 !right-4 !border !border-white/10 !bg-slate-950/90"
				/>
				<Panel position="top-right">
					<div className="rounded-2xl border border-white/8 bg-slate-950/88 px-3 py-2 text-[10px] font-mono uppercase tracking-[0.16em] text-slate-400 shadow-[0_18px_40px_rgba(2,6,23,0.32)]">
						<div>{scene.nodes.length} nodes</div>
						<div>{scene.edges.length} routed edges</div>
						<div>{focusMode === "none" ? "full view" : `${focusMode} focus`}</div>
					</div>
				</Panel>
			</ReactFlow>
		</div>
	);
});

function getMiniMapNodeColor(data: MeshFlowNodeData): string {
	return data.minimapColor;
}

function getMiniMapNodeStroke(data: MeshFlowNodeData): string {
	return data.active ? "#facc15" : data.backendAccent;
}

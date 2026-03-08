import { type Edge, MarkerType, type Node, Position } from "@xyflow/react";
import type { MeshAnalysis } from "./meshAnalysis.ts";
import type { MeshLayout } from "./meshLayout.ts";
import { meshBackendTones, summarizeBackendDef } from "./meshPresentation.ts";
import type { MeshNodeSemantics } from "./meshSemantics.ts";

export interface MeshFlowNodeData extends Record<string, unknown> {
	nodeId: number;
	label: string;
	shortLabel: string;
	backendLabel: string;
	backendAccent: string;
	backendSurface: string;
	backendMuted: string;
	summaryDetail: string;
	inputPreview: string;
	badges: string[];
	isEntry: boolean;
	reachable: boolean;
	region: "reachable" | "unreachable";
	active: boolean;
	dimmed: boolean;
	isSelected: boolean;
	hasSharedMemory: boolean;
	minimapColor: string;
}

export interface MeshFlowEdgeData extends Record<string, unknown> {
	points: Array<{ x: number; y: number }>;
	active: boolean;
	dimmed: boolean;
}

export type MeshFlowNode = Node<MeshFlowNodeData, "meshNode">;
export type MeshFlowEdge = Edge<MeshFlowEdgeData, "meshEdge">;

export const MESH_FLOW_TARGET_HANDLE_ID = "in";
export const MESH_FLOW_SOURCE_HANDLE_ID = "out";

export interface MeshFlowScene {
	nodes: MeshFlowNode[];
	edges: MeshFlowEdge[];
}

export function buildMeshFlowScene({
	analysis,
	layout,
	semanticsById,
	selectedNodeId,
	activeNodeId,
	activeEdgeId,
	dimmedNodeIds,
}: {
	analysis: MeshAnalysis;
	layout: MeshLayout;
	semanticsById: Map<number, MeshNodeSemantics> | null;
	selectedNodeId: number | null;
	activeNodeId: number | null;
	activeEdgeId: string | null;
	dimmedNodeIds: Set<number>;
}): MeshFlowScene {
	const nodes: MeshFlowNode[] = [];
	for (const layoutNode of layout.nodes) {
		const analysisNode = analysis.nodesById.get(layoutNode.id);
		if (!analysisNode) {
			continue;
		}

		const semantics = semanticsById?.get(layoutNode.id) ?? null;
		const tone = meshBackendTones[analysisNode.backendKind];
		const summary = summarizeBackendDef(analysisNode.node.backend_def);
		const dimmed = dimmedNodeIds.has(layoutNode.id);
		const active = activeNodeId === layoutNode.id;
		const inputPreview = semantics?.inputTexts.slice(0, 2).join(" · ") ?? "";

		nodes.push({
			id: String(layoutNode.id),
			type: "meshNode",
			position: { x: layoutNode.x, y: layoutNode.y },
			sourcePosition: Position.Right,
			targetPosition: Position.Left,
			draggable: false,
			connectable: false,
			selectable: false,
			selected: selectedNodeId === layoutNode.id,
			style: {
				width: layoutNode.width,
				height: layoutNode.height,
			},
			data: {
				nodeId: layoutNode.id,
				label: semantics?.label ?? `Node ${layoutNode.id}`,
				shortLabel: semantics?.shortLabel ?? `Node ${layoutNode.id}`,
				backendLabel: tone.label,
				backendAccent: tone.accent,
				backendSurface: tone.surface,
				backendMuted: tone.muted,
				summaryDetail: summary.detail,
				inputPreview,
				badges: semantics?.badges ?? [],
				isEntry: analysisNode.isEntry,
				reachable: analysisNode.reachable,
				region: layoutNode.region,
				active,
				dimmed,
				isSelected: selectedNodeId === layoutNode.id,
				hasSharedMemory: semantics?.writeClasses.includes("memory") ?? false,
				minimapColor: getMinimapColor({
					reachable: analysisNode.reachable,
					active,
					dimmed,
					accent: tone.accent,
				}),
			},
		});
	}

	const edges: MeshFlowEdge[] = layout.edges.map((edge) => {
		const dimmed = dimmedNodeIds.has(edge.fromId) || dimmedNodeIds.has(edge.toId);
		const active = activeEdgeId === edge.id;
		const stroke = active
			? "rgba(250,204,21,0.9)"
			: dimmed
				? "rgba(100,116,139,0.3)"
				: "rgba(148,163,184,0.68)";

		return {
			id: edge.id,
			source: String(edge.fromId),
			target: String(edge.toId),
			sourceHandle: MESH_FLOW_SOURCE_HANDLE_ID,
			targetHandle: MESH_FLOW_TARGET_HANDLE_ID,
			type: "meshEdge",
			selectable: false,
			focusable: false,
			markerEnd: {
				type: MarkerType.ArrowClosed,
				color: stroke,
				width: 16,
				height: 16,
			},
			style: {
				stroke,
			},
			data: {
				points: edge.points.map((point) => ({ x: point.x, y: point.y })),
				active,
				dimmed,
			},
		};
	});

	return {
		nodes,
		edges,
	};
}

function getMinimapColor({
	reachable,
	active,
	dimmed,
	accent,
}: {
	reachable: boolean;
	active: boolean;
	dimmed: boolean;
	accent: string;
}): string {
	if (active) {
		return "#facc15";
	}
	if (!reachable) {
		return "#475569";
	}
	if (dimmed) {
		return "#64748b";
	}
	return accent;
}

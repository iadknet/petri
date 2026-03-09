import { useCallback, useDeferredValue, useMemo } from "react";
import {
	type MeshBackendFilter,
	type MeshFocusMode,
	useInspectorWorkspaceStore,
} from "../../../stores/inspectorWorkspace.ts";
import type { CreatureMeshAnnotation } from "../../../types/creature-detail.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import {
	type MeshWorkspaceController,
	useMeshWorkspaceController,
} from "../mesh/MeshWorkspaceController.ts";
import {
	type MeshAnalysis,
	type MeshAnalyzedNode,
	analyzeMesh,
	collectMeshClosure,
	matchesBackendFilter,
} from "../mesh/meshAnalysis.ts";
import type { MeshLayout } from "../mesh/meshLayout.ts";
import {
	type MeshNodeSemantics,
	type MeshSemantics,
	deriveMeshSemantics,
} from "../mesh/meshSemantics.ts";
import { useMeshLayout } from "../mesh/useMeshLayout.ts";

export interface UseMeshDerivationResult {
	analysis: MeshAnalysis;
	semantics: MeshSemantics | null;
	layout: MeshLayout | null;
	layoutError: string | null;
	detailNode: MeshAnalyzedNode | null;
	detailSemantics: MeshNodeSemantics | null;
	dimmedNodeIds: Set<number>;
	meshController: MeshWorkspaceController;
	handleNodeSelect: (nodeId: number) => void;
	backendFilter: MeshBackendFilter;
	focusMode: MeshFocusMode;
	dimUnreachable: boolean;
	setBackendFilter: (filter: MeshBackendFilter) => void;
	setFocusMode: (mode: MeshFocusMode) => void;
	setDimUnreachable: (dim: boolean) => void;
}

export function useMeshDerivation(
	genome: CreatureGenome,
	meshAnnotations: CreatureMeshAnnotation[] | null,
	activeNodeId: number | null,
): UseMeshDerivationResult {
	const selectedNodeId = useInspectorWorkspaceStore((state) => state.selectedNodeId);
	const backendFilter = useInspectorWorkspaceStore((state) => state.backendFilter);
	const focusMode = useInspectorWorkspaceStore((state) => state.focusMode);
	const dimUnreachable = useInspectorWorkspaceStore((state) => state.dimUnreachable);
	const setSelectedNodeId = useInspectorWorkspaceStore((state) => state.setSelectedNodeId);
	const setBackendFilter = useInspectorWorkspaceStore((state) => state.setBackendFilter);
	const setFocusMode = useInspectorWorkspaceStore((state) => state.setFocusMode);
	const setDimUnreachable = useInspectorWorkspaceStore((state) => state.setDimUnreachable);
	const meshController = useMeshWorkspaceController();

	const analysis = useMemo(() => analyzeMesh(genome), [genome]);
	const deferredAnalysis = useDeferredValue(analysis);
	const { layout, error: layoutError } = useMeshLayout(deferredAnalysis);

	const semantics = useMemo(
		() => deriveMeshSemantics(genome, meshAnnotations, analysis),
		[genome, meshAnnotations, analysis],
	);

	const detailNodeId = selectedNodeId ?? activeNodeId ?? analysis.entryNodeId;
	const detailNode = analysis.nodesById.get(detailNodeId) ?? analysis.nodes[0] ?? null;
	const detailSemantics = detailNode ? (semantics?.nodesById.get(detailNode.id) ?? null) : null;

	const focusNodeIds = useMemo(() => {
		if (focusMode === "none" || detailNodeId === null) {
			return null;
		}
		return collectMeshClosure(analysis, detailNodeId, focusMode);
	}, [analysis, detailNodeId, focusMode]);

	const dimmedNodeIds = useMemo(() => {
		const dimmed = new Set<number>();
		for (const node of analysis.nodes) {
			if (!matchesBackendFilter(node, backendFilter)) {
				dimmed.add(node.id);
				continue;
			}
			if (dimUnreachable && !node.reachable) {
				dimmed.add(node.id);
				continue;
			}
			if (focusNodeIds && !focusNodeIds.has(node.id)) {
				dimmed.add(node.id);
			}
		}
		return dimmed;
	}, [analysis.nodes, backendFilter, dimUnreachable, focusNodeIds]);

	const handleNodeSelect = useCallback(
		(nodeId: number) => {
			setSelectedNodeId(nodeId);
		},
		[setSelectedNodeId],
	);

	return {
		analysis,
		semantics,
		layout,
		layoutError,
		detailNode,
		detailSemantics,
		dimmedNodeIds,
		meshController,
		handleNodeSelect,
		backendFilter,
		focusMode,
		dimUnreachable,
		setBackendFilter,
		setFocusMode,
		setDimUnreachable,
	};
}

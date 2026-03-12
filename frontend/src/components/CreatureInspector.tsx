import { useCallback, useEffect, useRef } from "react";
import { useCreatureDetail } from "../hooks/useCreatureDetail.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { useInspectorWorkspaceStore } from "../stores/inspectorWorkspace.ts";
import { UnifiedInspector } from "./inspector/UnifiedInspector.tsx";

function CreatureInspector() {
	useCreatureDetail();

	const stats = useCreatureInspectorStore(creatureInspectorSelectors.creatureStats);
	const genome = useCreatureInspectorStore(creatureInspectorSelectors.creatureGenome);
	const meshAnnotations = useCreatureInspectorStore(
		creatureInspectorSelectors.creatureMeshAnnotations,
	);
	const sharedMemory = useCreatureInspectorStore(creatureInspectorSelectors.creatureSharedMemory);
	const actionLog = useCreatureInspectorStore(creatureInspectorSelectors.actionLog);
	const diagnostics = useCreatureInspectorStore(creatureInspectorSelectors.creatureDiagnostics);
	const isLoading = useCreatureInspectorStore(creatureInspectorSelectors.isLoading);
	const isDead = useCreatureInspectorStore(creatureInspectorSelectors.isDead);
	const error = useCreatureInspectorStore(creatureInspectorSelectors.error);
	const selectedCreatureId = useCreatureInspectorStore(
		creatureInspectorSelectors.selectedCreatureId,
	);
	const clearSelection = useCreatureInspectorStore((s) => s.clearSelection);
	const resetWorkspaceForCreatureChange = useInspectorWorkspaceStore(
		(state) => state.resetForCreatureChange,
	);

	const prevSelectedCreatureIdRef = useRef<number | null>(selectedCreatureId);

	useEffect(() => {
		resetWorkspaceForCreatureChange();
	}, [resetWorkspaceForCreatureChange]);

	useEffect(() => {
		if (selectedCreatureId !== prevSelectedCreatureIdRef.current) {
			resetWorkspaceForCreatureChange();
			prevSelectedCreatureIdRef.current = selectedCreatureId;
		}
	}, [resetWorkspaceForCreatureChange, selectedCreatureId]);

	const handleClose = useCallback(() => {
		clearSelection();
	}, [clearSelection]);

	if (isLoading && !stats) {
		return <LoadingSkeleton />;
	}

	if (error) {
		return (
			<div className="px-4 py-6 text-center">
				<p className="text-sm text-red-400">{error}</p>
				<button
					type="button"
					onClick={clearSelection}
					className="mt-2 text-xs text-slate-400 hover:text-slate-200"
				>
					Dismiss
				</button>
			</div>
		);
	}

	if (!stats || !genome) {
		return null;
	}

	return (
		<UnifiedInspector
			creatureId={stats.id}
			genome={genome}
			meshAnnotations={meshAnnotations}
			sharedMemory={sharedMemory}
			actionLog={actionLog}
			diagnostics={diagnostics}
			isDead={isDead}
			stats={stats}
			onClose={handleClose}
		/>
	);
}

function LoadingSkeleton() {
	return (
		<div className="animate-pulse space-y-3 px-4 py-4">
			<div className="flex gap-3">
				<div className="h-10 w-10 rounded-md bg-slate-800" />
				<div className="flex-1 space-y-2">
					<div className="h-4 w-20 rounded bg-slate-800" />
					<div className="h-3 w-32 rounded bg-slate-800" />
				</div>
			</div>
			<div className="h-2 rounded-full bg-slate-800" />
			<div className="space-y-2">
				<div className="h-3 w-full rounded bg-slate-800" />
				<div className="h-3 w-3/4 rounded bg-slate-800" />
			</div>
		</div>
	);
}

export default CreatureInspector;

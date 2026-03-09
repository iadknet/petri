import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useInspectorWorkspaceStore } from "../../stores/inspectorWorkspace.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";
import type { CreatureMeshAnnotation } from "../../types/creature-detail.ts";
import type { CreatureGenome, CreaturePhenotype } from "../../types/genome.ts";
import { InspectorEmptyState } from "../inspector/InspectorEmptyState.tsx";
import { MeshCanvas } from "../inspector/MeshCanvas.tsx";
import { MeshControls } from "../inspector/MeshControls.tsx";
import { deriveTraceFocus } from "../inspector/mesh/traceFocus.ts";
import { NodeInspector } from "./NodeInspector.tsx";
import { SamplerBar } from "./SamplerBar.tsx";
import { VitalsBanner } from "./VitalsBanner.tsx";
import { useMeshDerivation } from "./hooks/useMeshDerivation.ts";
import { useSamplerControls } from "./hooks/useSamplerControls.ts";

interface UnifiedInspectorProps {
	creatureId: number;
	genome: CreatureGenome;
	meshAnnotations: CreatureMeshAnnotation[] | null;
	sharedMemory: number[] | null;
	actionLog: ActionLogEntry[] | null;
	isDead: boolean;
	stats: {
		id: number;
		energy: number;
		maxEnergy: number;
		age: number;
		generation: number;
		complexity: number;
		genomeSize: number;
		position: { x: number; y: number };
		phenotype: CreaturePhenotype;
	};
	onClose: () => void;
}

export function UnifiedInspector({
	creatureId,
	genome,
	meshAnnotations,
	sharedMemory,
	actionLog,
	isDead,
	stats,
	onClose,
}: UnifiedInspectorProps) {
	// ── Sampler domain ──────────────────────────────────────────────────
	const sampler = useSamplerControls(creatureId);

	// ── Cross-domain derivation: traceFocus (genome + sampler state) ───
	const traceFocus = useMemo(
		() => deriveTraceFocus(genome, sampler.sample, sampler.position),
		[genome, sampler.sample, sampler.position],
	);

	const activeNodeId = useMemo(() => {
		if (sampler.playbackState === "idle" || sampler.playbackState === "sampling") {
			return null;
		}
		return traceFocus.activeNodeId;
	}, [sampler.playbackState, traceFocus.activeNodeId]);

	// ── Mesh domain ─────────────────────────────────────────────────────
	const mesh = useMeshDerivation(genome, meshAnnotations, activeNodeId);

	// ── Cross-domain derivation: executionHop (mesh + sampler) ─────────
	const executionHop = useMemo(() => {
		if (!sampler.sample || !mesh.detailNode) {
			return null;
		}
		const currentTick = sampler.sample.ticks[sampler.position.tickIndex];
		if (!currentTick) {
			return null;
		}
		const matchingHop = currentTick.hops.find((hop) => hop.node_id === mesh.detailNode!.id);
		if (!matchingHop) {
			return null;
		}
		return { hop: matchingHop, detailIndex: sampler.position.detailIndex };
	}, [sampler.sample, sampler.position, mesh.detailNode]);

	// ── Resize handle ───────────────────────────────────────────────────
	const inspectorWidth = useInspectorWorkspaceStore((s) => s.inspectorWidth);
	const setInspectorWidth = useInspectorWorkspaceStore((s) => s.setInspectorWidth);
	const resizeRef = useRef<{ startWidth: number; startX: number } | null>(null);
	const [isResizing, setIsResizing] = useState(false);

	useEffect(() => {
		if (!isResizing) return undefined;

		const handlePointerMove = (event: MouseEvent) => {
			const current = resizeRef.current;
			if (!current) return;
			const deltaX = event.clientX - current.startX;
			setInspectorWidth(current.startWidth - deltaX);
		};

		const stopResizing = () => {
			resizeRef.current = null;
			setIsResizing(false);
			document.body.style.removeProperty("cursor");
			document.body.style.removeProperty("user-select");
		};

		window.addEventListener("mousemove", handlePointerMove);
		window.addEventListener("mouseup", stopResizing, { once: true });

		return () => {
			window.removeEventListener("mousemove", handlePointerMove);
			window.removeEventListener("mouseup", stopResizing);
			document.body.style.removeProperty("cursor");
			document.body.style.removeProperty("user-select");
		};
	}, [isResizing, setInspectorWidth]);

	const handleResizeStart = useCallback(
		(event: React.MouseEvent<HTMLDivElement>) => {
			event.preventDefault();
			resizeRef.current = {
				startWidth: inspectorWidth,
				startX: event.clientX,
			};
			setIsResizing(true);
			document.body.style.cursor = "col-resize";
			document.body.style.userSelect = "none";
		},
		[inspectorWidth],
	);

	const handleResizeKeyDown = useCallback(
		(event: React.KeyboardEvent<HTMLDivElement>) => {
			if (event.key === "ArrowLeft") {
				event.preventDefault();
				setInspectorWidth(inspectorWidth - 24);
			}
			if (event.key === "ArrowRight") {
				event.preventDefault();
				setInspectorWidth(inspectorWidth + 24);
			}
		},
		[inspectorWidth, setInspectorWidth],
	);

	// ── Layout ──────────────────────────────────────────────────────────
	return (
		<div
			role="complementary"
			aria-label="Creature Inspector"
			className="relative flex h-full min-h-0 flex-col overflow-hidden border-l border-slate-800 bg-slate-950 shadow-[inset_1px_0_0_rgba(255,255,255,0.03)]"
			data-testid="unified-inspector"
		>
			{/* Resize handle */}
			{/* biome-ignore lint/a11y/useSemanticElements: interactive resize handle cannot use <hr> */}
			<div
				role="separator"
				tabIndex={0}
				aria-label="Resize inspector"
				aria-orientation="vertical"
				aria-valuemin={480}
				aria-valuemax={960}
				aria-valuenow={inspectorWidth}
				onMouseDown={handleResizeStart}
				onKeyDown={handleResizeKeyDown}
				className="absolute inset-y-0 left-0 z-20 w-3 cursor-col-resize bg-transparent outline-none"
				data-testid="inspector-resize-handle"
			>
				<span
					className={`absolute inset-y-0 left-[5px] w-px transition-colors ${
						isResizing ? "bg-cyan-400" : "bg-slate-800"
					}`}
				/>
			</div>

			{/* Vitals banner (top, full width) */}
			<VitalsBanner
				id={stats.id}
				rgb={stats.phenotype.rgb}
				generation={stats.generation}
				age={stats.age}
				energy={stats.energy}
				maxEnergy={stats.maxEnergy}
				position={stats.position}
				actionLog={actionLog}
				phenotype={stats.phenotype}
				isDead={isDead}
				onClose={onClose}
			/>

			{/* Canvas area (full width) + node inspector */}
			<div className="flex flex-1 min-h-0 flex-col overflow-hidden">
				{/* Controls header */}
				<MeshControls
					backendFilter={mesh.backendFilter}
					focusMode={mesh.focusMode}
					dimUnreachable={mesh.dimUnreachable}
					onBackendFilterChange={mesh.setBackendFilter}
					onFocusModeChange={mesh.setFocusMode}
					onDimUnreachableChange={mesh.setDimUnreachable}
					onFitView={mesh.meshController.requestFitView}
					onResetView={mesh.meshController.requestResetView}
				/>

				{/* Canvas — shrinks to fit, minimum 120px so nodes stay visible */}
				<div className="flex-1 min-h-[120px]">
					{mesh.layoutError ? (
						<InspectorEmptyState
							title="Layout failed"
							description="The mesh layout engine could not produce a stable topology for this genome."
						/>
					) : (
						<MeshCanvas
							analysis={mesh.analysis}
							semanticsById={mesh.semantics?.nodesById ?? null}
							layout={mesh.layout}
							selectedNodeId={mesh.detailNode?.id ?? null}
							activeNodeId={activeNodeId}
							activeEdgeId={traceFocus.activeEdgeId}
							dimmedNodeIds={mesh.dimmedNodeIds}
							focusMode={mesh.focusMode}
							viewportCommand={mesh.meshController.viewportCommand}
							onSelectNode={mesh.handleNodeSelect}
							complexity={stats.complexity}
							genomeSize={stats.genomeSize}
						/>
					)}
				</div>

				{/* Node inspector */}
				{mesh.detailNode && (
					<div className="max-h-[65%] shrink-0 overflow-y-auto border-t border-slate-800 bg-slate-950">
						<NodeInspector
							node={mesh.detailNode}
							semantics={mesh.detailSemantics}
							sharedMemory={sharedMemory}
							executionHop={executionHop}
						/>
					</div>
				)}
			</div>

			{/* Sampler bar (bottom, full width) */}
			<SamplerBar
				sample={sampler.sample}
				position={sampler.position}
				playbackState={sampler.playbackState}
				playbackSpeed={sampler.playbackSpeed}
				totalTicks={sampler.totalTicks}
				totalHops={sampler.totalHops}
				samplingError={sampler.samplingError}
				isDead={isDead}
				meshSemantics={mesh.semantics}
				onSample={sampler.handleSample}
				onResample={sampler.handleResample}
				onClear={sampler.clearSample}
				onPlay={sampler.setPlaying}
				onPause={sampler.setPaused}
				onStepForward={sampler.stepForward}
				onStepBackward={sampler.stepBackward}
				onSpeedChange={sampler.setPlaybackSpeed}
				onTickSelect={sampler.handleTickSelect}
				onHopSelect={sampler.handleHopSelect}
			/>
		</div>
	);
}

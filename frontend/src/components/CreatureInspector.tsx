import { useCallback, useEffect, useRef } from "react";
import { useCreatureDetail } from "../hooks/useCreatureDetail.ts";
import { useExecutionSampler } from "../hooks/useExecutionSampler.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { getDetailCount, useExecutionSamplerStore } from "../stores/executionSampler.ts";
import { ActionTimeline } from "./inspector/ActionTimeline.tsx";
import { InspectorHeader } from "./inspector/InspectorHeader.tsx";
import { MemoryHexView } from "./inspector/MemoryHexView.tsx";
import { NodeGraph } from "./inspector/NodeGraph.tsx";
import { PhenotypeDetail } from "./inspector/PhenotypeDetail.tsx";
import { SamplerControls } from "./inspector/SamplerControls.tsx";
import { SamplerPlaybackPanel } from "./inspector/SamplerPlaybackPanel.tsx";
import { StatsSection } from "./inspector/StatsSection.tsx";

// Stable action refs accessed via getState() — no need to subscribe.
const samplerActions = () => {
	const s = useExecutionSamplerStore.getState();
	return {
		clearSample: s.clearSample,
		setPlaying: s.setPlaying,
		setPaused: s.setPaused,
		stepForward: s.stepForward,
		stepBackward: s.stepBackward,
		setPlaybackSpeed: s.setPlaybackSpeed,
		jumpToPosition: s.jumpToPosition,
	};
};

function CreatureInspector() {
	useCreatureDetail();
	const { startSampling } = useExecutionSampler();

	const stats = useCreatureInspectorStore((s) => s.creatureStats);
	const genome = useCreatureInspectorStore((s) => s.creatureGenome);
	const memory = useCreatureInspectorStore((s) => s.creatureMemory);
	const actionLog = useCreatureInspectorStore((s) => s.actionLog);
	const isLoading = useCreatureInspectorStore((s) => s.isLoading);
	const isDead = useCreatureInspectorStore((s) => s.isDead);
	const error = useCreatureInspectorStore((s) => s.error);
	const clearSelection = useCreatureInspectorStore((s) => s.clearSelection);

	// Subscribe only to state used in rendering.
	const sample = useExecutionSamplerStore((s) => s.sample);
	const playbackState = useExecutionSamplerStore((s) => s.playbackState);
	const position = useExecutionSamplerStore((s) => s.position);
	const playbackSpeed = useExecutionSamplerStore((s) => s.playbackSpeed);

	const creatureId = stats?.id ?? null;
	const prevCreatureIdRef = useRef(creatureId);

	// Clear sample when selected creature changes.
	useEffect(() => {
		if (prevCreatureIdRef.current !== null && prevCreatureIdRef.current !== creatureId) {
			samplerActions().clearSample();
		}
		prevCreatureIdRef.current = creatureId;
	}, [creatureId]);

	const handleSample = useCallback(() => {
		if (creatureId !== null) startSampling(creatureId);
	}, [creatureId, startSampling]);

	const handleResample = useCallback(() => {
		samplerActions().clearSample();
		if (creatureId !== null) startSampling(creatureId);
	}, [creatureId, startSampling]);

	// Derive totals for SamplerControls position indicator.
	const totalTicks = sample?.ticks.length ?? 0;
	const currentTick = sample?.ticks[position.tickIndex];
	const totalHops = currentTick?.hops.length ?? 0;
	const currentHop = currentTick?.hops[position.hopIndex];
	const totalDetails = currentHop ? getDetailCount(currentHop.backend_trace) : 0;

	// Derive activeNodeId from current hop for NodeGraph glow.
	const activeNodeId = currentHop?.node_id ?? null;

	const handleTickSelect = useCallback(
		(index: number) =>
			samplerActions().jumpToPosition({ tickIndex: index, hopIndex: 0, detailIndex: 0 }),
		[],
	);

	const handleHopSelect = useCallback((index: number) => {
		const currentPos = useExecutionSamplerStore.getState().position;
		samplerActions().jumpToPosition({ ...currentPos, hopIndex: index, detailIndex: 0 });
	}, []);

	const { clearSample, setPlaying, setPaused, stepForward, stepBackward, setPlaybackSpeed } =
		samplerActions();

	return (
		<div
			role="complementary"
			aria-label="Creature Inspector"
			className="h-full bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden"
		>
			{isLoading && !stats && <LoadingSkeleton />}

			{error && (
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
			)}

			{stats && (
				<>
					{isDead && (
						<div className="px-4 py-2 bg-red-950/50 border-b border-red-900/50">
							<p className="text-xs text-red-400 text-center">This creature has died</p>
						</div>
					)}

					<InspectorHeader
						id={stats.id}
						rgb={stats.phenotype.rgb}
						generation={stats.generation}
						age={stats.age}
						complexity={stats.complexity}
						onClose={clearSelection}
					/>

					<div className="flex-1 overflow-y-auto min-h-0">
						<div className="border-t border-slate-800">
							<StatsSection
								energy={stats.energy}
								maxEnergy={stats.maxEnergy}
								position={stats.position}
								age={stats.age}
								generation={stats.generation}
							/>
						</div>

						{/* Action timeline */}
						{actionLog && actionLog.length > 0 && (
							<div className="border-t border-slate-800">
								<ActionTimeline actionLog={actionLog} maxEnergy={stats.maxEnergy} />
							</div>
						)}

						{/* Phenotype channels */}
						<div className="border-t border-slate-800">
							<PhenotypeDetail phenotype={stats.phenotype} />
						</div>

						{/* Genome mesh graph */}
						{genome && (
							<div className="border-t border-slate-800">
								<NodeGraph
									genome={genome}
									activeNodeId={
										playbackState !== "idle" && playbackState !== "sampling" ? activeNodeId : null
									}
								/>
							</div>
						)}

						{/* Memory hex view */}
						{memory && (
							<div className="border-t border-slate-800">
								<MemoryHexView memory={memory} />
							</div>
						)}

						{/* Execution sampler */}
						{(!isDead || playbackState !== "idle") && (
							<div className="border-t border-slate-800">
								<SamplerControls
									playbackState={playbackState}
									position={position}
									totalTicks={totalTicks}
									totalHops={totalHops}
									totalDetails={totalDetails}
									playbackSpeed={playbackSpeed}
									onSample={handleSample}
									onResample={handleResample}
									onClear={clearSample}
									onPlay={setPlaying}
									onPause={setPaused}
									onStepForward={stepForward}
									onStepBackward={stepBackward}
									onSpeedChange={setPlaybackSpeed}
								/>
							</div>
						)}

						{/* Sampler playback panel */}
						{sample && currentTick && (
							<SamplerPlaybackPanel
								sample={sample}
								position={position}
								onTickSelect={handleTickSelect}
								onHopSelect={handleHopSelect}
							/>
						)}
					</div>
				</>
			)}
		</div>
	);
}

function LoadingSkeleton() {
	return (
		<div className="px-4 py-4 space-y-3 animate-pulse">
			<div className="flex gap-3">
				<div className="w-10 h-10 rounded-md bg-slate-800" />
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

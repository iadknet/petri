import { memo } from "react";
import type { PlaybackState, SamplerPosition } from "../../stores/samplePlayback.ts";
import type { ExecutionSample } from "../../types/trace.ts";
import { MeshHopTimeline } from "./MeshHopTimeline.tsx";
import { TickTimeline } from "./TickTimeline.tsx";
import { formatActionList } from "./inputRefUtils.ts";
import type { MeshSemantics } from "./mesh/meshSemantics.ts";

const SPEED_OPTIONS = [250, 500, 1000, 2000] as const;

interface SamplerBarProps {
	sample: ExecutionSample | null;
	position: SamplerPosition;
	playbackState: PlaybackState | "sampling";
	playbackSpeed: number;
	totalTicks: number;
	totalHops: number;
	samplingError: string | null;
	isDead: boolean;
	meshSemantics: MeshSemantics | null;
	onSample: () => void;
	onResample: () => void;
	onClear: () => void;
	onPlay: () => void;
	onPause: () => void;
	onStepForward: () => void;
	onStepBackward: () => void;
	onSpeedChange: (ms: number) => void;
	onTickSelect: (index: number) => void;
	onHopSelect: (index: number) => void;
}

export const SamplerBar = memo(function SamplerBar({
	sample,
	position,
	playbackState,
	playbackSpeed,
	totalTicks,
	totalHops,
	samplingError,
	isDead,
	meshSemantics,
	onSample,
	onResample,
	onClear,
	onPlay,
	onPause,
	onStepForward,
	onStepBackward,
	onSpeedChange,
	onTickSelect,
	onHopSelect,
}: SamplerBarProps) {
	const isActive =
		sample !== null &&
		(playbackState === "loaded" || playbackState === "playing" || playbackState === "stepping");

	const currentTick = sample?.ticks[position.tickIndex];
	const currentHops = currentTick?.hops ?? [];

	// Error state
	if (samplingError !== null) {
		return (
			<div className="border-t border-white/[0.06] bg-slate-950/80 px-3 py-2 flex items-center gap-2">
				<span className="text-amber-400 text-xs">&#x26A0;</span>
				<span className="text-xs text-red-400 flex-1 truncate">{samplingError}</span>
				<button
					type="button"
					aria-label="Retry"
					onClick={onSample}
					className="text-[11px] px-2 py-0.5 rounded bg-white/[0.06] text-cyan-400 hover:bg-white/[0.1] transition-colors"
				>
					Retry
				</button>
			</div>
		);
	}

	// Sampling state
	if (playbackState === "sampling") {
		return (
			<div className="border-t border-white/[0.06] bg-slate-950/80 px-3 py-2 flex items-center gap-2">
				<span className="text-cyan-400 text-xs animate-spin">&#x25CC;</span>
				<span className="text-xs text-slate-400">Sampling...</span>
			</div>
		);
	}

	// Active state with transport controls + timelines
	if (isActive && sample) {
		const isPlaying = playbackState === "playing";

		return (
			<div className="border-t border-white/[0.06] bg-slate-950/80">
				{/* Transport bar */}
				<div className="px-3 py-2 flex items-center gap-2">
					<button
						type="button"
						aria-label="Step backward"
						onClick={onStepBackward}
						className="text-xs px-1.5 py-0.5 rounded bg-white/[0.06] text-slate-300 hover:bg-white/[0.1] hover:text-cyan-400 transition-colors"
					>
						&#x25C4;
					</button>
					{isPlaying ? (
						<button
							type="button"
							aria-label="Pause"
							onClick={onPause}
							className="text-xs px-1.5 py-0.5 rounded bg-white/[0.06] text-cyan-400 hover:bg-white/[0.1] transition-colors"
						>
							&#x275A;&#x275A;
						</button>
					) : (
						<button
							type="button"
							aria-label="Play"
							onClick={onPlay}
							className="text-xs px-1.5 py-0.5 rounded bg-white/[0.06] text-slate-300 hover:bg-white/[0.1] hover:text-cyan-400 transition-colors"
						>
							&#x25B6;
						</button>
					)}
					<button
						type="button"
						aria-label="Step forward"
						onClick={onStepForward}
						className="text-xs px-1.5 py-0.5 rounded bg-white/[0.06] text-slate-300 hover:bg-white/[0.1] hover:text-cyan-400 transition-colors"
					>
						&#x25BA;
					</button>

					{/* Speed selector */}
					<select
						aria-label="Playback speed"
						value={playbackSpeed}
						onChange={(e) => onSpeedChange(Number(e.target.value))}
						className="text-[10px] font-mono bg-white/[0.06] text-slate-300 rounded px-1 py-0.5 border-0 outline-none"
					>
						{SPEED_OPTIONS.map((ms) => (
							<option key={ms} value={ms}>
								{ms}ms
							</option>
						))}
					</select>

					{/* Position indicator */}
					<span className="text-[10px] font-mono text-slate-400">
						T:{position.tickIndex + 1}/{totalTicks}
					</span>
					<span className="text-[10px] font-mono text-slate-400">
						H:{position.hopIndex + 1}/{totalHops}
					</span>

					<div className="flex-1" />

					<button
						type="button"
						aria-label="Resample"
						onClick={onResample}
						className="text-[11px] px-2 py-0.5 rounded bg-white/[0.06] text-slate-300 hover:bg-white/[0.1] hover:text-cyan-400 transition-colors"
					>
						Resample
					</button>
					<button
						type="button"
						aria-label="Clear"
						onClick={onClear}
						className="text-[11px] px-2 py-0.5 rounded bg-white/[0.06] text-slate-400 hover:bg-white/[0.1] hover:text-red-400 transition-colors"
					>
						Clear
					</button>
				</div>

				{/* Tick timeline */}
				<TickTimeline
					ticks={sample.ticks}
					activeTickIndex={position.tickIndex}
					onTickSelect={onTickSelect}
				/>

				{/* Mesh hop timeline */}
				{currentTick && (
					<MeshHopTimeline
						hops={currentHops}
						meshSemantics={meshSemantics}
						activeHopIndex={position.hopIndex}
						terminationReason={currentTick.termination_reason}
						finalAction={formatActionList(currentTick.final_actions)}
						onHopSelect={onHopSelect}
					/>
				)}
			</div>
		);
	}

	// Idle state (with optional dead notice)
	return (
		<div className="border-t border-white/[0.06] bg-slate-950/80 px-3 py-2 flex items-center gap-2">
			{isDead && <span className="text-xs text-red-400">Creature has died</span>}
			<div className="flex-1" />
			<button
				type="button"
				aria-label="Sample 5 Ticks"
				onClick={onSample}
				disabled={isDead}
				className={`text-[11px] px-2 py-0.5 rounded transition-colors ${
					isDead
						? "bg-white/[0.03] text-slate-600 cursor-not-allowed"
						: "bg-white/[0.06] text-cyan-400 hover:bg-white/[0.1]"
				}`}
			>
				Sample 5 Ticks
			</button>
		</div>
	);
});

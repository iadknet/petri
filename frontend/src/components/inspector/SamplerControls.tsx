import { memo } from "react";
import type { PlaybackState, SamplerPosition } from "../../stores/samplePlayback.ts";

interface SamplerControlsProps {
	playbackState: PlaybackState | "sampling";
	position: SamplerPosition;
	totalTicks: number;
	totalHops: number;
	totalDetails: number;
	playbackSpeed: number;
	isDead: boolean;
	onSample: () => void;
	onResample: () => void;
	onClear: () => void;
	onPlay: () => void;
	onPause: () => void;
	onStepForward: () => void;
	onStepBackward: () => void;
	onSpeedChange: (ms: number) => void;
}

const SPEEDS = [100, 250, 500, 1000];

export const SamplerControls = memo(function SamplerControls(props: SamplerControlsProps) {
	const {
		playbackState,
		position,
		totalTicks,
		totalHops,
		totalDetails,
		playbackSpeed,
		isDead,
		onSample,
		onResample,
		onClear,
		onPlay,
		onPause,
		onStepForward,
		onStepBackward,
		onSpeedChange,
	} = props;

	return (
		<div className="px-3 py-2">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-2">
				Execution Sampler
			</div>

			{playbackState === "idle" && (
				<button
					type="button"
					onClick={onSample}
					className="px-3 py-1.5 text-xs font-mono bg-slate-700 hover:bg-slate-600 text-slate-200 rounded transition-colors"
				>
					Sample 5 Ticks
				</button>
			)}

			{playbackState === "sampling" && (
				<div className="flex items-center gap-2 text-xs text-slate-400">
					<span className="inline-block w-3 h-3 border-2 border-slate-400 border-t-transparent rounded-full animate-spin" />
					Sampling...
				</div>
			)}

			{(playbackState === "loaded" ||
				playbackState === "playing" ||
				playbackState === "stepping") && (
				<div className="space-y-2">
					<div className="flex items-center gap-1">
						{/* Transport controls */}
						<button
							type="button"
							onClick={onStepBackward}
							className="px-1.5 py-1 text-xs font-mono bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors"
							title="Step backward"
						>
							&#9664;&#9664;
						</button>
						{playbackState === "playing" ? (
							<button
								type="button"
								onClick={onPause}
								className="px-2 py-1 text-xs font-mono bg-sky-700 hover:bg-sky-600 text-slate-200 rounded transition-colors"
								title="Pause"
							>
								&#9646;&#9646;
							</button>
						) : (
							<button
								type="button"
								onClick={onPlay}
								className="px-2 py-1 text-xs font-mono bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors"
								title="Play"
							>
								&#9654;
							</button>
						)}
						<button
							type="button"
							onClick={onStepForward}
							className="px-1.5 py-1 text-xs font-mono bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors"
							title="Step forward"
						>
							&#9654;&#9654;
						</button>

						{/* Speed selector */}
						<select
							value={playbackSpeed}
							onChange={(e) => onSpeedChange(Number(e.target.value))}
							className="ml-2 px-1 py-1 text-xs font-mono bg-slate-700 text-slate-300 rounded border border-slate-600"
						>
							{SPEEDS.map((s) => (
								<option key={s} value={s}>
									{s}ms
								</option>
							))}
						</select>

						<div className="flex-1" />

						{/* Resample & Clear */}
						<button
							type="button"
							onClick={onResample}
							disabled={isDead}
							className="px-2 py-1 text-xs font-mono bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors disabled:cursor-not-allowed disabled:bg-slate-800 disabled:text-slate-600"
						>
							Resample
						</button>
						<button
							type="button"
							onClick={onClear}
							className="px-1.5 py-1 text-xs font-mono bg-slate-700 hover:bg-red-900/50 text-slate-400 hover:text-red-300 rounded transition-colors"
							title="Clear sample"
						>
							&#x2715;
						</button>
					</div>

					{/* Position indicator */}
					<div className="text-[10px] font-mono text-slate-500">
						Tick {position.tickIndex + 1}/{totalTicks} · Hop {position.hopIndex + 1}/{totalHops} ·
						Detail {position.detailIndex + 1}/{totalDetails}
					</div>
				</div>
			)}
		</div>
	);
});

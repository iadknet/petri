import { memo } from "react";
import type { SamplerPosition } from "../../stores/executionSampler.ts";
import type { ExecutionSample } from "../../types/api.ts";
import { GraphExecutionView } from "./GraphExecutionView.tsx";
import { MeshHopTimeline } from "./MeshHopTimeline.tsx";
import { TickTimeline } from "./TickTimeline.tsx";
import { VmExecutionView } from "./VmExecutionView.tsx";
import { formatAction } from "./inputRefUtils.ts";

interface SamplerPlaybackPanelProps {
	sample: ExecutionSample;
	position: SamplerPosition;
	onTickSelect: (index: number) => void;
	onHopSelect: (index: number) => void;
}

export const SamplerPlaybackPanel = memo(function SamplerPlaybackPanel({
	sample,
	position,
	onTickSelect,
	onHopSelect,
}: SamplerPlaybackPanelProps) {
	const currentTick = sample.ticks[position.tickIndex];
	if (!currentTick) return null;

	const currentHop = currentTick.hops[position.hopIndex];

	return (
		<div className="space-y-0">
			<div className="border-t border-slate-800">
				<TickTimeline
					ticks={sample.ticks}
					activeTickIndex={position.tickIndex}
					onTickSelect={onTickSelect}
				/>
			</div>

			{currentTick.hops.length > 0 && (
				<div className="border-t border-slate-800">
					<MeshHopTimeline
						hops={currentTick.hops}
						activeHopIndex={position.hopIndex}
						terminationReason={currentTick.termination_reason}
						finalAction={formatAction(currentTick.final_action)}
						onHopSelect={onHopSelect}
					/>
				</div>
			)}

			{currentHop && (
				<div className="border-t border-slate-800">
					{typeof currentHop.backend_trace === "string" ? (
						<div className="px-3 py-2 text-[10px] text-slate-500 font-mono">
							Unsupported trace type: {currentHop.backend_trace}
						</div>
					) : "Vm" in currentHop.backend_trace ? (
						<VmExecutionView
							trace={currentHop.backend_trace.Vm}
							inputRefs={currentHop.input_refs}
							upstreamSlots={currentHop.upstream_slots}
							detailIndex={position.detailIndex}
						/>
					) : (
						<GraphExecutionView
							trace={currentHop.backend_trace.Graph}
							inputRefs={currentHop.input_refs}
							upstreamSlots={currentHop.upstream_slots}
							detailIndex={position.detailIndex}
						/>
					)}
				</div>
			)}
		</div>
	);
});

import { memo } from "react";
import type { SamplerPosition } from "../../stores/executionSampler.ts";
import type { ExecutionSample, WorldAction } from "../../types/api.ts";
import { GraphExecutionView } from "./GraphExecutionView.tsx";
import { MeshHopTimeline } from "./MeshHopTimeline.tsx";
import { TickTimeline } from "./TickTimeline.tsx";
import { VmExecutionView } from "./VmExecutionView.tsx";

interface SamplerPlaybackPanelProps {
	sample: ExecutionSample;
	position: SamplerPosition;
	onTickSelect: (index: number) => void;
	onHopSelect: (index: number) => void;
}

function formatAction(action: WorldAction): string {
	if (typeof action === "string") return action;
	if ("Move" in action) return `Move(${action.Move})`;
	if ("Reproduce" in action) return "Reproduce";
	return "?";
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
					{"Vm" in currentHop.backend_trace ? (
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

import { memo } from "react";
import type { MeshHopTrace, TerminationReason } from "../../types/api.ts";

interface MeshHopTimelineProps {
	hops: MeshHopTrace[];
	activeHopIndex: number;
	terminationReason: TerminationReason;
	finalAction: string;
	onHopSelect: (index: number) => void;
}

const VM_COLOR = "#f59e0b";
const GRAPH_COLOR = "#8b5cf6";

function hopSummary(hop: MeshHopTrace): string {
	if (typeof hop.backend_trace === "string") return "";
	if ("Vm" in hop.backend_trace) {
		return `${hop.backend_trace.Vm.steps.length} ops`;
	}
	if ("Graph" in hop.backend_trace) {
		const g = hop.backend_trace.Graph;
		return `${g.passes.length} pass`;
	}
	return "";
}

function hopBackendType(hop: MeshHopTrace): string {
	if (typeof hop.backend_trace === "string") return hop.backend_trace;
	return "Vm" in hop.backend_trace ? "Vm" : "Graph";
}

export const MeshHopTimeline = memo(function MeshHopTimeline({
	hops,
	activeHopIndex,
	terminationReason,
	finalAction,
	onHopSelect,
}: MeshHopTimelineProps) {
	return (
		<div className="px-3 py-2">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Mesh Hops
			</div>
			<div className="flex items-center gap-1 overflow-x-auto pb-1">
				{hops.map((hop, i) => {
					const type = hopBackendType(hop);
					const color = type === "Vm" ? VM_COLOR : type === "Graph" ? GRAPH_COLOR : "#94a3b8";
					const isActive = i === activeHopIndex;

					return (
						<div key={hop.hop_index} className="flex items-center gap-1 flex-shrink-0">
							{i > 0 && <span className="text-slate-600 text-xs">→</span>}
							<button
								type="button"
								onClick={() => onHopSelect(i)}
								className={`px-2 py-1 rounded text-[10px] font-mono transition-colors ${
									isActive
										? "ring-1 ring-white/10 text-slate-200"
										: "hover:bg-slate-700/50 text-slate-400"
								}`}
								style={{
									backgroundColor: isActive ? `${color}20` : "transparent",
									borderLeft: `2px solid ${color}`,
								}}
							>
								<div className="flex items-center gap-1">
									<span style={{ color }} className="font-medium text-[9px]">
										{type}
									</span>
									<span className="text-slate-500">#{hop.node_id}</span>
								</div>
								<div className="text-[9px] text-slate-500">{hopSummary(hop)}</div>
							</button>
						</div>
					);
				})}

				{/* Terminal marker */}
				<div className="flex items-center gap-1 flex-shrink-0">
					{hops.length > 0 && <span className="text-slate-600 text-xs">→</span>}
					<span className="text-[10px] font-mono text-slate-500 px-1">
						{terminationReason === "ActionEmitted" ? finalAction : terminationReason}
					</span>
				</div>
			</div>
		</div>
	);
});

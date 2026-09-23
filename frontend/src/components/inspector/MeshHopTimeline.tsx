import { memo } from "react";
import type {
	MeshHopTrace,
	MeshPassTrace,
	TerminationReason,
	WorldAction,
} from "../../types/trace.ts";
import { formatAction } from "./inputRefUtils.ts";
import type { MeshSemantics } from "./mesh/meshSemantics.ts";

interface MeshHopTimelineProps {
	hops: MeshHopTrace[];
	passes: readonly MeshPassTrace[];
	meshSemantics: MeshSemantics | null;
	activeHopIndex: number;
	terminationReason: TerminationReason;
	finalActions: readonly WorldAction[];
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

interface HopButtonProps {
	hop: MeshHopTrace;
	index: number;
	isActive: boolean;
	semanticsLabel: string | null;
	onHopSelect: (index: number) => void;
}

/** One recorded dispatch; the route line appears only when it applied one. */
function HopButton({ hop, index, isActive, semanticsLabel, onHopSelect }: HopButtonProps) {
	const type = hopBackendType(hop);
	const color = type === "Vm" ? VM_COLOR : type === "Graph" ? GRAPH_COLOR : "#94a3b8";
	return (
		<button
			type="button"
			data-testid={`hop-${hop.hop_index}`}
			onClick={() => onHopSelect(index)}
			className={`px-2 py-1 rounded text-[10px] font-mono transition-colors ${
				isActive ? "ring-1 ring-white/10 text-slate-200" : "hover:bg-slate-700/50 text-slate-400"
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
			{semanticsLabel ? (
				<div className="mt-0.5 max-w-[11rem] truncate text-[9px] text-slate-300">
					{semanticsLabel}
				</div>
			) : null}
			<div className="text-[9px] text-slate-500">{hopSummary(hop)}</div>
			{hop.route ? (
				<div className="text-[9px] text-cyan-400/80">→ #{hop.route.selected_target_id}</div>
			) : null}
		</button>
	);
}

/**
 * The sampled tick as passes: one group per recorded pass, in order, headed
 * by its index, end reason, recorded hop count, and committed action, with
 * the hops recorded in it beneath. A pass with no recorded hop (a missing
 * entry, an unaffordable first hop) keeps its group. The tick's termination
 * reason and final queue follow the last pass.
 */
export const MeshHopTimeline = memo(function MeshHopTimeline({
	hops,
	passes,
	meshSemantics,
	activeHopIndex,
	terminationReason,
	finalActions,
	onHopSelect,
}: MeshHopTimelineProps) {
	const indexed = hops.map((hop, index) => ({ hop, index }));
	return (
		<div className="px-3 py-2">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Passes
			</div>
			<div className="flex flex-col gap-1.5">
				{passes.map((pass) => {
					const passHops = indexed.filter(({ hop }) => hop.pass_index === pass.pass_index);
					return (
						<div key={pass.pass_index} data-testid={`pass-group-${pass.pass_index}`}>
							<div className="text-[10px] font-mono text-slate-400 mb-0.5">
								Pass {pass.pass_index} · {pass.end_reason} · {pass.hops} hops ·{" "}
								<span className={pass.committed ? "text-orange-300" : "text-slate-500"}>
									{pass.committed ? `commit ${formatAction(pass.committed)}` : "no commit"}
								</span>
							</div>
							{passHops.length === 0 ? (
								<div className="text-[9px] font-mono text-slate-600">no recorded hop</div>
							) : (
								<div className="flex items-center gap-1 overflow-x-auto pb-1">
									{passHops.map(({ hop, index }, position) => (
										<div key={hop.hop_index} className="flex items-center gap-1 flex-shrink-0">
											{position > 0 && <span className="text-slate-600 text-xs">→</span>}
											<HopButton
												hop={hop}
												index={index}
												isActive={index === activeHopIndex}
												semanticsLabel={meshSemantics?.nodesById.get(hop.node_id)?.label ?? null}
												onHopSelect={onHopSelect}
											/>
										</div>
									))}
								</div>
							)}
						</div>
					);
				})}
			</div>
			<div data-testid="tick-end" className="mt-1.5 text-[10px] font-mono text-slate-500">
				{terminationReason} · queue{" "}
				{finalActions.length === 0 ? "empty" : finalActions.map(formatAction).join(", ")}
			</div>
		</div>
	);
});

import { memo } from "react";
import type { GraphTrace, InputReference } from "../../types/api.ts";
import { InputsPanel } from "./InputsPanel.tsx";

interface GraphExecutionViewProps {
	trace: GraphTrace;
	inputRefs: InputReference[];
	upstreamSlots: number[];
	detailIndex: number;
}

const STATEFUL_KINDS = new Set(["DecayIntegrator", "Momentum", "Oscillator", "AdaptiveGain"]);

export const GraphExecutionView = memo(function GraphExecutionView({
	trace,
	inputRefs,
	upstreamSlots,
	detailIndex,
}: GraphExecutionViewProps) {
	const currentPass = trace.passes[detailIndex];

	return (
		<div className="px-3 py-2 space-y-2">
			{/* Inputs panel */}
			<InputsPanel inputRefs={inputRefs} upstreamSlots={upstreamSlots} />

			{/* Pass selector */}
			<div>
				<div className="text-[10px] text-slate-500 uppercase tracking-wider font-medium mb-1">
					Pass {detailIndex + 1}/{trace.passes.length}
					{currentPass && (
						<span className="ml-2 text-slate-600">
							Δ={currentPass.max_delta.toFixed(4)} cost={currentPass.energy_cost.toFixed(4)}e
						</span>
					)}
				</div>

				{/* Node evaluations for current pass */}
				{currentPass && (
					<div className="space-y-px max-h-[200px] overflow-y-auto">
						{currentPass.node_evaluations.map((node) => {
							const isStateful = STATEFUL_KINDS.has(node.kind);
							const stateChanged = isStateful && node.state_before !== node.state_after;

							return (
								<div
									key={node.node_index}
									className="flex items-baseline gap-2 px-1.5 py-0.5 text-[10px] font-mono"
								>
									<span className="text-slate-600 w-4 text-right flex-shrink-0">
										{node.node_index}
									</span>
									<span className="text-slate-300 w-20 flex-shrink-0 truncate">{node.kind}</span>
									<span className="text-slate-500 text-[9px] w-14 flex-shrink-0">
										Σ={node.weighted_sum.toFixed(2)}
									</span>
									{isStateful ? (
										<span
											className={`text-[9px] flex-shrink-0 ${stateChanged ? "text-amber-400" : "text-slate-600"}`}
										>
											{node.state_before.toFixed(2)}→{node.state_after.toFixed(2)}
										</span>
									) : null}
									<span className="text-emerald-400/70 text-[9px] flex-shrink-0">
										→{node.output.toFixed(3)}
									</span>
								</div>
							);
						})}
					</div>
				)}
			</div>

			{/* Convergence footer */}
			<div className="text-[10px] font-mono flex items-center gap-2">
				<span className={trace.converged ? "text-emerald-400" : "text-amber-400"}>
					{trace.converged ? "Converged" : "Not converged"}
				</span>
				<span className="text-slate-600">
					{trace.stable_passes_count} stable · {trace.passes.length} passes
				</span>
			</div>
		</div>
	);
});

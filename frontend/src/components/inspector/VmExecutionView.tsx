import { memo } from "react";
import type { InputReference, VmInstruction, VmTrace } from "../../types/api.ts";
import { formatInputRef, inputRefColor } from "./inputRefUtils.ts";

interface VmExecutionViewProps {
	trace: VmTrace;
	inputRefs: InputReference[];
	upstreamSlots: number[];
	detailIndex: number;
}

function formatInstruction(instr: VmInstruction): string {
	if (typeof instr === "string") return instr;
	const key = Object.keys(instr)[0];
	return key ?? "?";
}

function instructionDetail(instr: VmInstruction): string {
	if (typeof instr === "string") return "";
	const key = Object.keys(instr)[0] as keyof typeof instr;
	const fields = (instr as Record<string, Record<string, number>>)[key];
	if (!fields) return "";
	return Object.entries(fields)
		.map(([k, v]) => `${k}:${v}`)
		.join(" ");
}

export const VmExecutionView = memo(function VmExecutionView({
	trace,
	inputRefs,
	upstreamSlots,
	detailIndex,
}: VmExecutionViewProps) {
	return (
		<div className="px-3 py-2 space-y-2">
			{/* Inputs panel */}
			<div>
				<div className="text-[10px] text-slate-500 uppercase tracking-wider font-medium mb-1">
					Inputs
				</div>
				<div className="grid grid-cols-2 gap-x-4 gap-y-0.5 text-[10px] font-mono">
					{inputRefs.map((ref, i) => (
						<div key={i} className="flex justify-between">
							<span style={{ color: inputRefColor(ref) }}>{formatInputRef(ref)}</span>
							<span className="text-slate-400">{(upstreamSlots[i] ?? 0).toFixed(2)}</span>
						</div>
					))}
				</div>
			</div>

			{/* Registers */}
			<div>
				<div className="text-[10px] text-slate-500 uppercase tracking-wider font-medium mb-1">
					Registers ({trace.register_count})
				</div>
				<div className="grid grid-cols-4 gap-x-2 gap-y-0.5 text-[10px] font-mono max-h-[120px] overflow-y-auto">
					{trace.final_registers.map((v, i) => {
						// Check if this register changed at the current step.
						const step = trace.steps[detailIndex];
						const changed = step?.register_changes.some(([r]) => r === i);
						return (
							<div key={i} className={changed ? "text-emerald-400" : "text-slate-400"}>
								r{i}: {v.toFixed(2)}
							</div>
						);
					})}
				</div>
			</div>

			{/* Instruction list */}
			<div>
				<div className="text-[10px] text-slate-500 uppercase tracking-wider font-medium mb-1">
					Instructions ({trace.steps.length})
				</div>
				<div className="space-y-px max-h-[200px] overflow-y-auto">
					{trace.steps.map((step, i) => {
						const isCurrent = i === detailIndex;
						const isCompleted = i < detailIndex;

						return (
							<div
								key={i}
								className={`flex items-baseline gap-2 px-1.5 py-0.5 text-[10px] font-mono rounded ${
									isCurrent
										? "bg-sky-900/30 border-l-2 border-sky-400"
										: isCompleted
											? "opacity-50"
											: ""
								}`}
							>
								<span className="text-slate-600 w-4 text-right flex-shrink-0">{step.pc}</span>
								<span className="text-slate-300 w-24 flex-shrink-0 truncate">
									{formatInstruction(step.instruction)}
								</span>
								<span className="text-slate-600 text-[9px] truncate flex-1">
									{instructionDetail(step.instruction)}
								</span>
								{step.register_changes.length > 0 && (
									<span className="text-emerald-400/70 text-[9px] flex-shrink-0">
										{step.register_changes.map(([r, v]) => `r${r}←${v.toFixed(2)}`).join(" ")}
									</span>
								)}
							</div>
						);
					})}
				</div>
			</div>

			{/* Outputs */}
			<div className="text-[10px] font-mono text-slate-500">
				<span className="text-slate-600">route:</span> {trace.final_route_target.toFixed(2)}
				{trace.memory_writes.length > 0 && (
					<span className="ml-2 text-amber-400/70">{trace.memory_writes.length} mem writes</span>
				)}
			</div>
		</div>
	);
});

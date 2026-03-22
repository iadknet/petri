import { memo, useMemo } from "react";
import type { InputReference } from "../../types/genome.ts";
import type { VmTrace } from "../../types/trace.ts";
import { InputsPanel } from "./InputsPanel.tsx";

const DIRECTION_NAMES = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;

/** Format action meta slot value with direction decoding for slot 0. */
function formatMetaSlot(slotIdx: number, value: number): string {
	// Slot 0 often holds a direction index (0-7) — decode to compass name
	if (slotIdx === 0 && Number.isInteger(value) && value >= 0 && value <= 7) {
		return `[${slotIdx}]=${DIRECTION_NAMES[value]}(${value.toFixed(0)})`;
	}
	return `[${slotIdx}]=${value.toFixed(2)}`;
}

interface VmExecutionViewProps {
	trace: VmTrace;
	inputRefs: InputReference[];
	upstreamSlots: number[];
	detailIndex: number;
}

function computeRegistersAtStep(trace: VmTrace, stepIndex: number): number[] {
	const regs = new Array<number>(trace.register_count).fill(0);
	const limit = Math.min(stepIndex + 1, trace.steps.length);
	for (let i = 0; i < limit; i++) {
		const step = trace.steps[i];
		if (!step) continue;
		for (const [regIdx, newVal] of step.register_changes) {
			regs[regIdx] = newVal;
		}
	}
	return regs;
}

export const VmExecutionView = memo(function VmExecutionView({
	trace,
	inputRefs,
	upstreamSlots,
	detailIndex,
}: VmExecutionViewProps) {
	const registersAtStep = useMemo(
		() => computeRegistersAtStep(trace, detailIndex),
		[trace, detailIndex],
	);

	const changedRegs = useMemo(() => {
		const step = trace.steps[detailIndex];
		if (!step) return new Set<number>();
		return new Set(step.register_changes.map(([r]) => r));
	}, [trace, detailIndex]);

	const registerEntries = useMemo(
		() =>
			registersAtStep.map((value, regIdx) => ({
				id: `r${regIdx}`,
				regIdx,
				value,
			})),
		[registersAtStep],
	);

	return (
		<div className="space-y-2 px-3 py-2">
			<InputsPanel inputRefs={inputRefs} upstreamSlots={upstreamSlots} />

			<div>
				<div className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					Registers ({trace.register_count})
				</div>
				<div className="grid max-h-[120px] grid-cols-4 gap-x-2 gap-y-0.5 overflow-y-auto text-[10px] font-mono">
					{registerEntries.map(({ id, regIdx, value }) => (
						<div
							key={id}
							className={changedRegs.has(regIdx) ? "text-emerald-400" : "text-slate-400"}
						>
							r{regIdx}: {value.toFixed(2)}
						</div>
					))}
				</div>
			</div>

			<div className="grid gap-2 md:grid-cols-2">
				<div className="rounded border border-white/5 bg-white/[0.02] px-2 py-2 text-[10px] font-mono text-slate-400">
					<div>
						<span className="text-slate-600">gates:</span>{" "}
						{trace.final_registers.length > 0 ? "see mesh route" : "—"}
					</div>
					<div>
						<span className="text-slate-600">payload:</span>{" "}
						{trace.final_payload.length > 0
							? trace.final_payload.map((value) => value.toFixed(2)).join(" ")
							: "—"}
					</div>
					<div>
						<span className="text-slate-600">meta:</span>{" "}
						{trace.final_meta.length > 0
							? trace.final_meta
									.map((value, idx) => formatMetaSlot(idx, value))
									.join(" ")
							: "—"}
					</div>
				</div>
				<div className="rounded border border-white/5 bg-white/[0.02] px-2 py-2 text-[10px] font-mono text-slate-400">
					<div className="mb-1 text-slate-600">slot writes</div>
					{trace.slot_writes.length > 0 ? (
						<div className="space-y-1 [content-visibility:auto]">
							{trace.slot_writes.map((write, index) => (
								<div key={`${write.slot_idx}-${index}`} className="text-amber-300/80">
									s{write.slot_idx}: {write.old_value.toFixed(2)}→{write.new_value.toFixed(2)}
								</div>
							))}
						</div>
					) : (
						<div className="text-slate-600">none</div>
					)}
				</div>
			</div>
		</div>
	);
});

import { useMemo } from "react";
import type { BackendDef, InputReference } from "../../types/genome.ts";
import type { VmTrace } from "../../types/trace.ts";
import { buildActionContext, formatReadableInstruction } from "./vmInstructionFormat.ts";

interface NodeBackendDetailProps {
	backendDef: BackendDef;
	inputRefs: InputReference[];
	liveInstructionIndices: number[];
	liveInternalNodeIndices: number[];
	vmTrace?: VmTrace | null;
	detailIndex?: number;
}

export function NodeBackendDetail({
	backendDef,
	inputRefs,
	liveInstructionIndices,
	vmTrace,
	detailIndex = 0,
}: NodeBackendDetailProps) {
	// Compute execution overlay unconditionally (Rules of Hooks)
	const execOverlay = useMemo(() => {
		if (!vmTrace || !("Vm" in backendDef)) return null;
		const currentStep = vmTrace.steps[detailIndex];
		const currentPc = currentStep?.pc ?? -1;
		const executedPcs = new Set<number>();
		const regChangesAtPc = new Map<number, [number, number][]>();
		for (let i = 0; i <= detailIndex && i < vmTrace.steps.length; i++) {
			const step = vmTrace.steps[i];
			if (!step) continue;
			executedPcs.add(step.pc);
			if (step.pc === currentPc) {
				regChangesAtPc.set(step.pc, step.register_changes);
			}
		}
		return { currentPc, executedPcs, regChangesAtPc };
	}, [vmTrace, detailIndex, backendDef]);

	if ("Vm" in backendDef) {
		const vm = backendDef.Vm;
		const liveSet = new Set(liveInstructionIndices);
		const actionCtx = buildActionContext(vm.program);

		return (
			<div className="px-3 py-2">
				<div className="mb-1.5 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					VM Program ({vm.program.length}) · {vm.register_count} regs
				</div>
				<div className="space-y-px overflow-y-auto">
					{vm.program.map((instruction, index) => {
						const readable = formatReadableInstruction(
							instruction,
							inputRefs,
							vm.constants,
							index,
							actionCtx,
						);
						const isLive = liveSet.has(index);

						// Execution state overlay
						const isCurrent = execOverlay?.currentPc === index;
						const wasExecuted = execOverlay?.executedPcs.has(index) ?? false;
						const regChanges = isCurrent ? execOverlay?.regChangesAtPc.get(index) : undefined;
						const hasOverlay = execOverlay !== null;

						let rowClass: string;
						if (isCurrent) {
							rowClass = "border-l-2 border-sky-400 bg-sky-900/30";
						} else if (hasOverlay && !wasExecuted) {
							rowClass = "opacity-25";
						} else if (hasOverlay && wasExecuted) {
							rowClass = "opacity-60";
						} else if (!isLive) {
							rowClass = "opacity-40";
						} else {
							rowClass = "";
						}

						return (
							<div
								// biome-ignore lint/suspicious/noArrayIndexKey: instructions are indexed by position
								key={index}
								data-testid={`vm-instruction-${index}`}
								data-junk={isLive ? "false" : "true"}
								className={`flex items-baseline gap-2 rounded px-1.5 py-0.5 text-[11px] font-mono leading-relaxed ${rowClass}`}
							>
								<span className="w-5 shrink-0 text-right text-slate-600">{index}</span>
								<span className="w-[5.5rem] shrink-0 truncate text-slate-400">
									{readable.label}
								</span>
								<span className="flex-1 truncate text-slate-200">{readable.operands}</span>
								{readable.badges.length > 0 ? (
									<span className="shrink-0 text-[9px] uppercase text-cyan-400/70">
										{readable.badges.join(" ")}
									</span>
								) : null}
								{regChanges && regChanges.length > 0 ? (
									<span className="shrink-0 text-[9px] text-emerald-400/70">
										{regChanges.map(([r, v]) => `r${r}\u2190${v.toFixed(2)}`).join(" ")}
									</span>
								) : null}
							</div>
						);
					})}
				</div>
			</div>
		);
	}

	return null;
}

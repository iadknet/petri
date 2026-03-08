import { useMemo } from "react";
import type { BackendDef } from "../../types/genome.ts";
import type { VmTrace } from "../../types/trace.ts";
import {
	describeGraphInternalNode,
	describeVmInstruction,
} from "../inspector/mesh/meshPresentation.ts";

interface NodeBackendDetailProps {
	backendDef: BackendDef;
	liveInstructionIndices: number[];
	liveInternalNodeIndices: number[];
	vmTrace?: VmTrace | null;
	detailIndex?: number;
}

export function NodeBackendDetail({
	backendDef,
	liveInstructionIndices,
	liveInternalNodeIndices,
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

		return (
			<div className="px-3 py-2">
				<div className="mb-1 flex items-baseline gap-2">
					<span className="text-[10px] font-medium uppercase tracking-wider text-slate-500">
						VM Program ({vm.program.length})
					</span>
					{vm.constants.length > 0 ? (
						<span className="text-[10px] font-mono text-slate-500">
							const {vm.constants.map((v, i) => `${i}:${v}`).join(" ")}
						</span>
					) : null}
				</div>
				<div className="max-h-[200px] space-y-px overflow-y-auto">
					{vm.program.map((instruction, index) => {
						const presentation = describeVmInstruction(instruction);
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
								className={`flex items-baseline gap-2 rounded px-1.5 py-0.5 text-[10px] font-mono ${rowClass}`}
							>
								<span className="w-4 shrink-0 text-right text-slate-600">{index}</span>
								<span className="w-24 shrink-0 truncate text-slate-300">
									{presentation.label}
								</span>
								{presentation.detail ? (
									<span className="flex-1 truncate text-[9px] text-slate-600">
										{presentation.detail}
									</span>
								) : null}
								{presentation.badges.length > 0 ? (
									<span className="shrink-0 text-[9px] uppercase text-cyan-400/70">
										{presentation.badges.join(" ")}
									</span>
								) : null}
								{regChanges && regChanges.length > 0 ? (
									<span className="shrink-0 text-[9px] text-emerald-400/70">
										{regChanges
											.map(([r, v]) => `r${r}←${v.toFixed(2)}`)
											.join(" ")}
									</span>
								) : null}
							</div>
						);
					})}
				</div>
			</div>
		);
	}

	if ("Graph" in backendDef) {
		const graph = backendDef.Graph;
		const liveSet = new Set(liveInternalNodeIndices);

		return (
			<div className="px-3 py-2">
				<div className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					Graph Internals ({graph.internal_nodes.length})
				</div>
				<div className="max-h-[200px] space-y-px overflow-y-auto">
					{graph.internal_nodes.map((internalNode, index) => {
						const presentation = describeGraphInternalNode(internalNode.kind);
						const isLive = liveSet.has(index);

						return (
							<div
								// biome-ignore lint/suspicious/noArrayIndexKey: internal nodes are indexed by position
								key={index}
								data-testid={`graph-internal-${index}`}
								data-junk={isLive ? "false" : "true"}
								className={`flex items-baseline gap-2 rounded px-1.5 py-0.5 text-[10px] font-mono ${
									isLive ? "" : "opacity-40"
								}`}
							>
								<span className="w-4 shrink-0 text-right text-slate-600">{index}</span>
								<span className="shrink-0 truncate text-slate-300">
									{presentation.label}
								</span>
								{presentation.detail ? (
									<span className="text-[9px] text-slate-600">{presentation.detail}</span>
								) : null}
								{internalNode.inputs.length > 0 ? (
									<span className="text-[9px] text-slate-500">
										{internalNode.inputs
											.map((input) => `src:${input.source_idx} \u00d7 ${input.weight.toFixed(2)}`)
											.join(" \u00b7 ")}
									</span>
								) : null}
								{presentation.badges.length > 0 ? (
									<span className="shrink-0 text-[9px] uppercase text-cyan-400/70">
										{presentation.badges.join(" ")}
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

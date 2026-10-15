import type { BackendDef, ComputeNodeKind, VmInstruction } from "../../../types/genome.ts";
import type { RuntimeIoBadge } from "./runtimeIoSemantics.ts";
import {
	classifyComputeNodeKind,
	classifyGraphTraceKind,
	classifyVmInstruction,
} from "./runtimeIoSemantics.ts";

export type MeshNodeBadge = RuntimeIoBadge;

export interface MeshPresentationSummary {
	label: string;
	detail: string;
}

export interface MeshInstructionPresentation extends MeshPresentationSummary {
	badges: MeshNodeBadge[];
}

export interface MeshBackendTone {
	label: string;
	accent: string;
	surface: string;
	muted: string;
}

export const meshBackendTones: Record<"vm" | "graph", MeshBackendTone> = {
	vm: {
		label: "VM",
		accent: "#f59e0b",
		surface: "rgba(245,158,11,0.12)",
		muted: "#fcd34d",
	},
	graph: {
		label: "Graph",
		accent: "#38bdf8",
		surface: "rgba(56,189,248,0.12)",
		muted: "#67e8f9",
	},
};

export function summarizeBackendDef(backendDef: BackendDef): MeshPresentationSummary {
	if ("Vm" in backendDef) {
		return {
			label: "VM",
			detail: `${backendDef.Vm.program.length} ops \u00b7 ${backendDef.Vm.register_count} regs`,
		};
	}

	const graph = backendDef.Graph;
	const wired = graph.output_sinks.filter((s) => s.inputs.length > 0);
	const wiredActions = wired.filter(
		(s) => "ActionVote" in s.kind || "ActionParam" in s.kind,
	).length;
	const wiredSinks = wired.length - wiredActions;
	return {
		label: "Graph",
		detail: `${graph.compute_nodes.length} compute \u00b7 ${wiredSinks} sinks \u00b7 ${wiredActions} actions`,
	};
}

export function describeVmInstruction(instruction: VmInstruction): MeshInstructionPresentation {
	const semantics = classifyVmInstruction(instruction);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
}

export function describeComputeNodeKind(kind: ComputeNodeKind): MeshInstructionPresentation {
	const semantics = classifyComputeNodeKind(kind);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
}

export function describeGraphTraceKind(kind: string): MeshInstructionPresentation {
	const semantics = classifyGraphTraceKind(kind);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
}

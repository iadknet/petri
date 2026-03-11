import type {
	BackendDef,
	ComputeNodeKind,
	NodeGenome,
	OutputSinkKind,
	VmInstruction,
} from "../../../types/genome.ts";
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
	const wiredSinks = graph.output_sinks.filter((s) => s.inputs.length > 0).length;
	const wiredActions = graph.action_bank.filter(
		(s) => s.gate_inputs.length > 0 || s.param_inputs.length > 0,
	).length;
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

/** Get the kind name from an OutputSinkKind. */
function getOutputSinkKindName(kind: OutputSinkKind): string {
	if (typeof kind === "string") return kind;
	const [name] = Object.entries(kind)[0] ?? ["?"];
	return name;
}

export function collectNodeBadges(node: NodeGenome): MeshNodeBadge[] {
	const badgeSet = new Set<MeshNodeBadge>();

	if ("Vm" in node.backend_def) {
		for (const instruction of node.backend_def.Vm.program) {
			for (const badge of describeVmInstruction(instruction).badges) {
				badgeSet.add(badge);
			}
		}
	}

	if ("Graph" in node.backend_def) {
		const graph = node.backend_def.Graph;

		// Compute node badges (stateful)
		for (const computeNode of graph.compute_nodes) {
			for (const badge of describeComputeNodeKind(computeNode.kind).badges) {
				badgeSet.add(badge);
			}
		}

		// Output sink badges
		for (const sink of graph.output_sinks) {
			if (sink.inputs.length === 0) continue; // unwired sinks don't contribute
			const sinkName = getOutputSinkKindName(sink.kind);
			if (sinkName === "RouterOutput") badgeSet.add("route");
			else if (sinkName === "CustomOutput") badgeSet.add("output");
			else if (sinkName === "WriteSlot" || sinkName === "ClearSlot") badgeSet.add("slot");
		}

		// Action bank badges
		for (const slot of graph.action_bank) {
			if (slot.gate_inputs.length > 0 || slot.param_inputs.length > 0) {
				badgeSet.add("action");
			}
		}
	}

	return ["input", "slot", "action", "route", "output", "stateful"].filter(
		(badge): badge is MeshNodeBadge => badgeSet.has(badge as MeshNodeBadge),
	);
}

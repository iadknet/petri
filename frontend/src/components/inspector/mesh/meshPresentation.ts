import type {
	BackendDef,
	GraphNodeKind,
	NodeGenome,
	VmInstruction,
} from "../../../types/genome.ts";
import type { RuntimeIoBadge } from "./runtimeIoSemantics.ts";
import {
	classifyGraphKind,
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

export function summarizeBackendDef(
	backendDef: BackendDef,
): MeshPresentationSummary {
	if ("Vm" in backendDef) {
		return {
			label: "VM",
			detail: `${backendDef.Vm.program.length} ops · ${backendDef.Vm.register_count} regs`,
		};
	}

	return {
		label: "Graph",
		detail: `${backendDef.Graph.internal_nodes.length} nodes`,
	};
}

export function describeVmInstruction(
	instruction: VmInstruction,
): MeshInstructionPresentation {
	const semantics = classifyVmInstruction(instruction);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
}

export function describeGraphInternalNode(
	kind: GraphNodeKind,
): MeshInstructionPresentation {
	const semantics = classifyGraphKind(kind);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
}

export function describeGraphTraceKind(
	kind: string,
): MeshInstructionPresentation {
	const semantics = classifyGraphTraceKind(kind);
	return {
		label: semantics.name,
		detail: semantics.detail,
		badges: semantics.badges,
	};
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
		for (const internalNode of node.backend_def.Graph.internal_nodes) {
			for (const badge of describeGraphInternalNode(internalNode.kind).badges) {
				badgeSet.add(badge);
			}
		}
	}

	return ["input", "slot", "action", "route", "output", "stateful"].filter(
		(badge): badge is MeshNodeBadge => badgeSet.has(badge as MeshNodeBadge),
	);
}

import type { ComputeNodeKind, VmInstruction } from "../../../types/genome.ts";

export type RuntimeIoBadge = "action" | "input" | "slot" | "output" | "route" | "stateful";

interface RuntimeIoFlags {
	readsInput: boolean;
	readsSlot: boolean;
	readsPrevSlot: boolean;
	readsActionQueue: boolean;
	writesAction: boolean;
	writesRoute: boolean;
	writesPayload: boolean;
	writesSlot: boolean;
	stateful: boolean;
}

export interface RuntimeIoSemantics extends RuntimeIoFlags {
	name: string;
	detail: string;
	badges: RuntimeIoBadge[];
}

const BADGE_ORDER: RuntimeIoBadge[] = ["input", "slot", "action", "route", "output", "stateful"];

const EMPTY_FLAGS: RuntimeIoFlags = {
	readsInput: false,
	readsSlot: false,
	readsPrevSlot: false,
	readsActionQueue: false,
	writesAction: false,
	writesRoute: false,
	writesPayload: false,
	writesSlot: false,
	stateful: false,
};

export function classifyVmInstruction(instruction: VmInstruction): RuntimeIoSemantics {
	if (typeof instruction === "string") {
		return withBadges({ name: instruction, detail: "", ...EMPTY_FLAGS });
	}

	const [name, payload] = Object.entries(instruction)[0] ?? ["?", {}];
	const flags: RuntimeIoFlags = { ...EMPTY_FLAGS };

	switch (name) {
		case "ReadInput":
			flags.readsInput = true;
			break;
		case "ReadActionQueueLength":
		case "ReadActionQueueType":
		case "ReadActionQueueParam":
			flags.readsActionQueue = true;
			break;
		// Votes and parameter writes feed the pass's commit.
		case "WriteActionParam":
		case "AddVote":
		case "SetPriorityBid":
			flags.writesAction = true;
			break;
		case "WriteRouteGate":
			flags.writesRoute = true;
			break;
		case "WriteInternalPayload":
			flags.writesPayload = true;
			break;
		case "LoadSlot":
		case "LoadSlotImm":
			flags.readsSlot = true;
			flags.stateful = true;
			break;
		case "LoadSlotPrev":
			flags.readsPrevSlot = true;
			flags.stateful = true;
			break;
		case "StoreSlot":
		case "StoreSlotImm":
		case "ClearSlot":
			flags.writesSlot = true;
			flags.stateful = true;
			break;
		default:
			break;
	}

	return withBadges({
		name,
		detail: formatPayload(payload),
		...flags,
	});
}

export function classifyComputeNodeKind(kind: ComputeNodeKind): RuntimeIoSemantics {
	if (typeof kind === "string") {
		return classifyComputeKindName(kind, "");
	}

	const [name, rawValue] = Object.entries(kind)[0] ?? ["?", 0];
	const detail = typeof rawValue === "number" ? rawValue.toString() : "";
	return classifyComputeKindName(name, detail);
}

export function classifyGraphTraceKind(kindName: string): RuntimeIoSemantics {
	return classifyComputeKindName(kindName, "");
}

function classifyComputeKindName(name: string, detail: string): RuntimeIoSemantics {
	const flags: RuntimeIoFlags = { ...EMPTY_FLAGS };

	switch (name) {
		case "AdaptiveGain":
		case "DecayIntegrator":
		case "Momentum":
		case "Oscillator":
			flags.stateful = true;
			break;
		default:
			break;
	}

	return withBadges({
		name,
		detail,
		...flags,
	});
}

function withBadges(semantics: Omit<RuntimeIoSemantics, "badges">): RuntimeIoSemantics {
	const badges = new Set<RuntimeIoBadge>();
	if (semantics.readsInput) {
		badges.add("input");
	}
	if (semantics.readsSlot || semantics.readsPrevSlot || semantics.writesSlot) {
		badges.add("slot");
	}
	if (semantics.writesAction || semantics.readsActionQueue) {
		badges.add("action");
	}
	if (semantics.writesRoute) {
		badges.add("route");
	}
	if (semantics.writesPayload) {
		badges.add("output");
	}
	if (semantics.stateful) {
		badges.add("stateful");
	}

	return {
		...semantics,
		badges: BADGE_ORDER.filter((badge) => badges.has(badge)),
	};
}

function formatPayload(payload: unknown): string {
	if (!payload || typeof payload !== "object") {
		return "";
	}

	return Object.entries(payload)
		.map(([key, value]) => `${key}:${String(value)}`)
		.join(" ");
}

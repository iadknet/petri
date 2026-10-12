import type { InputReference, VmInstruction } from "../../types/genome.ts";
import {
	actionParamDescription,
	actionParamTargetLabel,
	describeVoteSink,
	voteSinkLabel,
} from "./graphNodeFormatters.ts";
import { describeInputRef } from "./inputRefUtils.ts";
import type { RuntimeIoBadge } from "./mesh/runtimeIoSemantics.ts";
import { classifyVmInstruction } from "./mesh/runtimeIoSemantics.ts";
import { voteSinkFromIndex } from "./voteSinks.ts";

export interface ReadableInstruction {
	label: string;
	operands: string;
	/** Plain-words tooltip for what the instruction reads or writes. */
	description?: string;
	badges: RuntimeIoBadge[];
}

/**
 * Map raw opcode names to short, readable display labels.
 * These must fit in the ~13-character label column (w-[5.5rem] at 11px mono).
 */
const OPCODE_LABELS: Record<string, string> = {
	// Data movement
	ReadInput: "read",
	LoadConst: "const",
	Move: "copy",
	// Arithmetic
	Add: "add",
	Sub: "sub",
	Mul: "mul",
	Div: "div",
	Min: "min",
	Max: "max",
	// Unary
	Abs: "abs",
	Neg: "neg",
	Clamp01: "clamp",
	Not: "not",
	// Comparison
	CmpGt: "cmp.gt",
	CmpLt: "cmp.lt",
	CmpEq: "cmp.eq",
	// Logic
	And: "and",
	Or: "or",
	// Casts
	ToI32: "to_i32",
	ToU8: "to_u8",
	ToBool: "to_bool",
	// Control flow
	JumpIfZero: "branch",
	Jump: "jump",
	// Route / payload / action
	WriteRouteGate: "route",
	WriteInternalPayload: "bus",
	WriteActionParam: "param",
	AddVote: "vote",
	SetPriorityBid: "priority",
	ReadActionQueueLength: "queue",
	ReadActionQueueType: "queue",
	ReadActionQueueParam: "queue",
	// Memory
	LoadSlot: "load",
	StoreSlot: "store",
	LoadSlotImm: "load",
	StoreSlotImm: "store",
	LoadSlotPrev: "load prev",
	ClearSlot: "clear",
};

/**
 * Format a VM instruction with human-readable label and operands.
 * Resolves input ref indices to sensor names, constant indices to values,
 * jump offsets to absolute line numbers, and parameter writes to param names.
 *
 * @param index - The instruction's position in the program (for computing jump targets).
 */
export function formatReadableInstruction(
	instruction: VmInstruction,
	inputRefs: InputReference[],
	constants: number[],
	index: number,
): ReadableInstruction {
	const semantics = classifyVmInstruction(instruction);
	const badges = semantics.badges;

	if (typeof instruction === "string") {
		return formatStringInstruction(instruction, badges);
	}

	const [name, rawPayload] = Object.entries(instruction)[0] ?? ["?", {}];
	const payload =
		rawPayload && typeof rawPayload === "object" ? (rawPayload as Record<string, number>) : {};

	const label = OPCODE_LABELS[name] ?? name.toLowerCase();
	const operands = formatOperands(name, payload, inputRefs, constants, index);
	const description = describeOperands(name, payload, inputRefs);
	return { label, operands, badges, ...(description ? { description } : {}) };
}

/** Format string-type instructions (Noop, Halt). */
function formatStringInstruction(
	instruction: string,
	badges: RuntimeIoBadge[],
): ReadableInstruction {
	switch (instruction) {
		case "Noop":
			return { label: "nop", operands: "", badges };
		case "Halt":
			return { label: "halt", operands: "", badges };
		default:
			return { label: instruction.toLowerCase(), operands: "", badges };
	}
}

/** Safe field access — defaults to 0 for missing keys. */
function f(p: Record<string, number>, key: string): number {
	return p[key] ?? 0;
}

function reg(idx: number): string {
	return `r${idx}`;
}

function resolveInputRef(refIdx: number, subIdx: number, inputRefs: InputReference[]): string {
	const ref = inputRefs[refIdx];
	if (!ref) return `input[${refIdx}]`;
	return describeInputRef(ref, subIdx).label;
}

/** Tooltip for instructions whose operands name something outside the VM. */
function describeOperands(
	name: string,
	p: Record<string, number>,
	inputRefs: InputReference[],
): string | undefined {
	switch (name) {
		case "ReadInput": {
			const ref = inputRefs[f(p, "ref_idx") || f(p, "input_idx")];
			return ref
				? describeInputRef(ref, f(p, "sub_idx")).description
				: "This node has no such input; reads 0.";
		}
		case "AddVote": {
			const sink = voteSinkFromIndex(f(p, "sink"));
			return sink ? describeVoteSink(sink) : "Not a vote sink.";
		}
		case "WriteActionParam":
			return actionParamDescription(f(p, "field_idx"));
		case "WriteRouteGate":
			return "Score for this route slot. The next mesh node is the target with the highest bias + score.";
		case "WriteInternalPayload":
			return "Writes a bus slot; the next mesh node to run reads it as Bus[n].";
		case "SetPriorityBid":
			return "Energy bid paid at the end of the tick; higher bids act first. A bid at or above current energy ends the tick.";
		case "LoadSlotPrev":
			return "Shared memory as it stood at the start of this tick (last tick's final value).";
		case "LoadSlot":
		case "LoadSlotImm":
			return "Shared memory now, including writes made earlier this tick.";
		case "ReadActionQueueLength":
		case "ReadActionQueueType":
		case "ReadActionQueueParam":
			return "Actions already committed this tick. Type codes: 0 none, 1 Eat, 2 Move, 3 Reproduce, 4 StealEnergy.";
		default:
			return undefined;
	}
}

function formatOperands(
	name: string,
	p: Record<string, number>,
	inputRefs: InputReference[],
	constants: number[],
	index: number,
): string {
	switch (name) {
		// ── Input ──
		case "ReadInput": {
			const refIdx = f(p, "ref_idx") || f(p, "input_idx");
			const subIdx = f(p, "sub_idx");
			return `${reg(f(p, "dst"))} ← ${resolveInputRef(refIdx, subIdx, inputRefs)}`;
		}

		// ── Constants ──
		case "LoadConst": {
			const constIdx = f(p, "const_idx");
			const value = constants[constIdx];
			const formatted = value != null ? fmtNum(value) : `c${constIdx}`;
			return `${reg(f(p, "dst"))} ← ${formatted}`;
		}

		// ── Copy (register move) ──
		case "Move":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "src"))}`;

		// ── Binary arithmetic ──
		case "Add":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} + ${reg(f(p, "b"))}`;
		case "Sub":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} − ${reg(f(p, "b"))}`;
		case "Mul":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} × ${reg(f(p, "b"))}`;
		case "Div":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} ÷ ${reg(f(p, "b"))}`;
		case "Min":
			return `${reg(f(p, "dst"))} ← min(${reg(f(p, "a"))}, ${reg(f(p, "b"))})`;
		case "Max":
			return `${reg(f(p, "dst"))} ← max(${reg(f(p, "a"))}, ${reg(f(p, "b"))})`;

		// ── Unary ──
		case "Abs":
			return `${reg(f(p, "dst"))} ← |${reg(f(p, "src"))}|`;
		case "Neg":
			return `${reg(f(p, "dst"))} ← −${reg(f(p, "src"))}`;
		case "Clamp01":
			return `${reg(f(p, "dst"))} ← clamp01(${reg(f(p, "src"))})`;
		case "Not":
			return `${reg(f(p, "dst"))} ← !${reg(f(p, "src"))}`;

		// ── Comparison ──
		case "CmpGt":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} > ${reg(f(p, "b"))}`;
		case "CmpLt":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} < ${reg(f(p, "b"))}`;
		case "CmpEq":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} ≈ ${reg(f(p, "b"))} (ε=${reg(f(p, "eps"))})`;

		// ── Logic ──
		case "And":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} & ${reg(f(p, "b"))}`;
		case "Or":
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} | ${reg(f(p, "b"))}`;

		// ── Casts ──
		case "ToI32":
			return `${reg(f(p, "dst"))} ← i32(${reg(f(p, "src"))})`;
		case "ToU8":
			return `${reg(f(p, "dst"))} ← u8(${reg(f(p, "src"))})`;
		case "ToBool":
			return `${reg(f(p, "dst"))} ← bool(${reg(f(p, "src"))})`;

		// ── Control flow (absolute jump targets) ──
		case "JumpIfZero": {
			const target = index + f(p, "offset");
			return `if ${reg(f(p, "cond"))} = 0 → line ${target}`;
		}
		case "Jump": {
			const target = index + f(p, "offset");
			return `→ line ${target}`;
		}

		// ── Route / payload / action ──
		case "WriteRouteGate":
			return `route[${f(p, "slot")}] ← ${reg(f(p, "src"))}`;
		case "WriteInternalPayload":
			return `bus[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "AddVote":
			return `vote[${voteSinkLabel(f(p, "sink"))}] += ${reg(f(p, "src"))}`;
		case "WriteActionParam":
			return `${actionParamTargetLabel(f(p, "field_idx"))} ← ${reg(f(p, "src"))}`;
		case "SetPriorityBid":
			return `priority ← ${reg(f(p, "src"))}`;
		case "ReadActionQueueLength":
			return `${reg(f(p, "dst"))} ← queue.len`;
		case "ReadActionQueueType":
			return `${reg(f(p, "dst"))} ← queue[${reg(f(p, "index_src"))}].type`;
		case "ReadActionQueueParam":
			return `${reg(f(p, "dst"))} ← queue[${reg(f(p, "index_src"))}].p${f(p, "param_slot")}`;

		// ── Shared memory ──
		case "LoadSlot":
			return `${reg(f(p, "dst"))} ← mem[${reg(f(p, "slot_reg"))}]`;
		case "StoreSlot":
			return `mem[${reg(f(p, "slot_reg"))}] ← ${reg(f(p, "src"))}`;
		case "LoadSlotImm":
			return `${reg(f(p, "dst"))} ← mem[${f(p, "slot_idx")}]`;
		case "StoreSlotImm":
			return `mem[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "LoadSlotPrev":
			return `${reg(f(p, "dst"))} ← prev_mem[${f(p, "slot_idx")}]`;
		case "ClearSlot":
			return `clear mem[${f(p, "slot_idx")}]`;

		default:
			return Object.entries(p)
				.map(([k, v]) => `${k}:${v}`)
				.join(" ");
	}
}

function fmtNum(value: number): string {
	if (Number.isInteger(value)) return String(value);
	// Show up to 4 decimal places, trim trailing zeros
	return value.toFixed(4).replace(/0+$/, "").replace(/\.$/, "");
}

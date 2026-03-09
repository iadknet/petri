import type { InputReference, VmInstruction } from "../../types/genome.ts";
import { directionName, formatInputRef, isRingSensor } from "./inputRefUtils.ts";
import type { RuntimeIoBadge } from "./mesh/runtimeIoSemantics.ts";
import { classifyVmInstruction } from "./mesh/runtimeIoSemantics.ts";

export interface ReadableInstruction {
	label: string;
	operands: string;
	badges: RuntimeIoBadge[];
}

/**
 * Format a VM instruction with human-readable operands.
 * Resolves input ref indices to their names (e.g. "Food.S", "slot[0]")
 * and constant indices to their actual values.
 */
export function formatReadableInstruction(
	instruction: VmInstruction,
	inputRefs: InputReference[],
	constants: number[],
): ReadableInstruction {
	const semantics = classifyVmInstruction(instruction);
	const badges = semantics.badges;

	if (typeof instruction === "string") {
		return { label: instruction.toLowerCase(), operands: "", badges };
	}

	const [name, payload] = Object.entries(instruction)[0] as [string, Record<string, number>];

	const operands = formatOperands(name, payload, inputRefs, constants);
	return { label: name, operands, badges };
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
	const label = formatInputRef(ref);
	// Ring sensors: show direction name instead of raw sub_idx
	if (typeof ref !== "string" && "World" in ref && isRingSensor(ref.World)) {
		return `${label}[${directionName(subIdx)}]`;
	}
	if (subIdx > 0) return `${label}[${subIdx}]`;
	return label;
}

function formatOperands(
	name: string,
	p: Record<string, number>,
	inputRefs: InputReference[],
	constants: number[],
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

		// ── Move ──
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
			return `${reg(f(p, "dst"))} ← ${reg(f(p, "a"))} ≈ ${reg(f(p, "b"))}`;

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

		// ── Control flow ──
		case "JumpIfZero":
			return `if ${reg(f(p, "cond"))} = 0 → +${f(p, "offset")}`;
		case "Jump":
			return `→ +${f(p, "offset")}`;

		// ── Route / payload / action ──
		case "WriteRouteTarget":
			return `route ← ${reg(f(p, "src"))}`;
		case "WriteInternalPayload":
			return `payload[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "WriteWorldActionMeta":
			return `meta[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "PushAction":
			return `push action(${f(p, "action_type")})`;
		case "SetPriorityBid":
			return `priority ← ${reg(f(p, "src"))}`;
		case "ReadActionQueueLength":
			return `${reg(f(p, "dst"))} ← queue.len`;
		case "ReadActionQueueType":
			return `${reg(f(p, "dst"))} ← queue[${reg(f(p, "index_src"))}].type`;
		case "ReadActionQueueParam":
			return `${reg(f(p, "dst"))} ← queue[${reg(f(p, "index_src"))}].p${f(p, "param_slot")}`;

		// ── Memory slots ──
		case "LoadSlot":
			return `${reg(f(p, "dst"))} ← slot[${reg(f(p, "slot_reg"))}]`;
		case "StoreSlot":
			return `slot[${reg(f(p, "slot_reg"))}] ← ${reg(f(p, "src"))}`;
		case "LoadSlotImm":
			return `${reg(f(p, "dst"))} ← slot[${f(p, "slot_idx")}]`;
		case "StoreSlotImm":
			return `slot[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "LoadSlotPrev":
			return `${reg(f(p, "dst"))} ← prev[${f(p, "slot_idx")}]`;
		case "ClearSlot":
			return `clear slot[${f(p, "slot_idx")}]`;

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

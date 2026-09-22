import type { InputReference, VmInstruction } from "../../types/genome.ts";
import { formatInputRefWithSubIndex } from "./inputRefUtils.ts";
import type { RuntimeIoBadge } from "./mesh/runtimeIoSemantics.ts";
import { classifyVmInstruction } from "./mesh/runtimeIoSemantics.ts";

export interface ReadableInstruction {
	label: string;
	operands: string;
	badges: RuntimeIoBadge[];
}

/** Action type index → human-readable name (matches WorldActionKind ordering). */
const ACTION_TYPE_NAMES = ["NoOp", "Eat", "Move", "Reproduce", "Steal"] as const;

/**
 * Per-action-type parameter names, indexed by [action_type][slot_idx].
 * Slot 0 = primary param, slot 1 = secondary param.
 */
const ACTION_PARAM_NAMES: readonly (readonly string[])[] = [
	[], // 0: NoOp — no params
	["food"], // 1: Eat — food type index
	["dir"], // 2: Move — direction
	["dir", "frac"], // 3: Reproduce — direction + transfer fraction of parent energy
	["dir", "amt"], // 4: StealEnergy — direction + amount
];

// ── Action context: connects WriteWorldActionMeta ↔ PushAction ──────────────

/** Context for a WriteWorldActionMeta instruction (what param name it sets). */
interface MetaWriteContext {
	/** Human-readable param name (e.g. "dir", "energy") based on the next PushAction. */
	paramName: string;
}

/** Context for a PushAction instruction (which registers supply its params). */
interface PushActionContext {
	/** Source register for each param slot, from preceding WriteWorldActionMeta. */
	paramSources: (string | null)[];
}

export interface ActionContext {
	metaWrites: Map<number, MetaWriteContext>;
	pushActions: Map<number, PushActionContext>;
}

/**
 * Pre-scan the program to connect WriteWorldActionMeta → PushAction.
 * For each PushAction, scan backwards to find which registers were written
 * to each meta slot. For each WriteWorldActionMeta, look forward to find
 * the next PushAction and derive the param name from its action type.
 */
export function buildActionContext(program: VmInstruction[]): ActionContext {
	const metaWrites = new Map<number, MetaWriteContext>();
	const pushActions = new Map<number, PushActionContext>();

	for (let i = 0; i < program.length; i++) {
		const instr = program[i];
		if (typeof instr === "string" || !instr || !("PushAction" in instr)) continue;

		const actionType = (instr.PushAction as { action_type: number }).action_type;
		const paramNames = ACTION_PARAM_NAMES[actionType] ?? [];
		const paramSources: (string | null)[] = paramNames.map(() => null);

		// Scan backwards to find the most recent WriteWorldActionMeta for each slot
		for (let j = i - 1; j >= 0; j--) {
			const prev = program[j];
			if (typeof prev === "string") {
				// Stop at ExecuteActionQueue or Halt — these break the action block
				if (prev === "ExecuteActionQueue" || prev === "Halt") break;
				continue;
			}
			if (!prev) continue;
			// Stop at another PushAction — its meta writes belong to it, not us
			if ("PushAction" in prev) break;

			if ("WriteWorldActionMeta" in prev) {
				const meta = prev.WriteWorldActionMeta as { slot_idx: number; src: number };
				const slotIdx = meta.slot_idx;
				const srcReg = `r${meta.src}`;

				// Record this meta write's param name
				if (slotIdx < paramNames.length) {
					const paramName = paramNames[slotIdx];
					if (paramName) {
						metaWrites.set(j, { paramName });
						paramSources[slotIdx] = srcReg;
					}
				}
			}
		}

		pushActions.set(i, { paramSources });
	}

	return { metaWrites, pushActions };
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
	WriteRouteGate: "gate",
	WriteInternalPayload: "payload",
	WriteWorldActionMeta: "set",
	AddVote: "vote",
	PushAction: "push",
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
 * jump offsets to absolute line numbers, and action meta writes to param names.
 *
 * @param index - The instruction's position in the program (for computing jump targets).
 * @param actionCtx - Pre-computed action context connecting meta writes to push actions.
 */
export function formatReadableInstruction(
	instruction: VmInstruction,
	inputRefs: InputReference[],
	constants: number[],
	index: number,
	actionCtx?: ActionContext,
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
	const operands = formatOperands(name, payload, inputRefs, constants, index, actionCtx);
	return { label, operands, badges };
}

/** Format string-type instructions (Noop, PopAction, ExecuteActionQueue, Halt). */
function formatStringInstruction(
	instruction: string,
	badges: RuntimeIoBadge[],
): ReadableInstruction {
	switch (instruction) {
		case "Noop":
			return { label: "nop", operands: "", badges };
		case "PopAction":
			return { label: "pop", operands: "remove last action", badges };
		case "ExecuteActionQueue":
			return { label: "emit", operands: "emit queued actions", badges };
		case "Halt":
			return { label: "halt", operands: "", badges };
		default:
			return { label: instruction.toLowerCase(), operands: "", badges };
	}
}

/** Label the vote sink at a catalog index (T19.F03); out of range reads as invalid. */
export function voteSinkLabel(index: number): string {
	if (index === 0) return "Eat";
	if (index < 9) return `Move[${index - 1}]`;
	if (index < 17) return `Reproduce[${index - 9}]`;
	if (index < 25) return `StealEnergy[${index - 17}]`;
	if (index === 25) return "Terminate";
	if (index === 26) return "Decide";
	return `invalid(${index})`;
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
	return formatInputRefWithSubIndex(ref, subIdx);
}

function formatOperands(
	name: string,
	p: Record<string, number>,
	inputRefs: InputReference[],
	constants: number[],
	index: number,
	actionCtx?: ActionContext,
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
			return `gate[${f(p, "slot")}] ← ${reg(f(p, "src"))}`;
		case "WriteInternalPayload":
			return `payload[${f(p, "slot_idx")}] ← ${reg(f(p, "src"))}`;
		case "AddVote": {
			const sinkIdx = f(p, "sink");
			const sink = voteSinkLabel(sinkIdx);
			return `vote[${sink}] += ${reg(f(p, "src"))}`;
		}
		case "WriteWorldActionMeta": {
			const slotIdx = f(p, "slot_idx");
			const metaCtx = actionCtx?.metaWrites.get(index);
			const slotLabel = metaCtx?.paramName ?? `action[${slotIdx}]`;
			return `${slotLabel} ← ${reg(f(p, "src"))}`;
		}
		case "PushAction": {
			const actionIdx = f(p, "action_type");
			const actionName = ACTION_TYPE_NAMES[actionIdx] ?? `type(${actionIdx})`;
			const paramNames = ACTION_PARAM_NAMES[actionIdx];
			const pushCtx = actionCtx?.pushActions.get(index);
			if (paramNames && paramNames.length > 0) {
				const parts = paramNames.map((pName, i) => {
					const src = pushCtx?.paramSources[i];
					return src ? `${pName}=${src}` : pName;
				});
				return `push ${actionName}(${parts.join(", ")})`;
			}
			return `push ${actionName}`;
		}
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

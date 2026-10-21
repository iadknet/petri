import catalog from "../components/inspector/voteSinks.json";

/**
 * Action types — serialized as string variant names from Rust's `ActionType`
 * enum; the values come from the Rust-pinned `voteSinks.json` catalog.
 */
export type ActionType = "NoOp" | "Eat" | "Move" | "Reproduce" | "StealEnergy" | "GiveEnergy";

export const ActionType = Object.freeze(
	Object.fromEntries(catalog.actionTypes.map(({ name }) => [name, name])),
) as { readonly [K in ActionType]: K };

/**
 * Action results — serialized as string variant names from Rust's `ActionResult` enum.
 */
export const ActionResult = {
	Success: "Success",
	NoFood: "NoFood",
	Blocked: "Blocked",
	InvalidTarget: "InvalidTarget",
	EnergyConstraints: "EnergyConstraints",
	PopulationCap: "PopulationCap",
	TransferredAndKilled: "TransferredAndKilled",
	NoVictim: "NoVictim",
	NoRecipient: "NoRecipient",
} as const;

export type ActionResult = (typeof ActionResult)[keyof typeof ActionResult];

export interface ActionLogEntry {
	tick: number;
	action_type: ActionType;
	result: ActionResult;
	direction: number;
	energy_before: number;
	energy_after: number;
	amount: number;
	/** Selected food type for Eat actions; null for every other action type. */
	food_type: number | null;
	priority_bid: number;
}

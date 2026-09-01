/**
 * Action types — serialized as string variant names from Rust's `ActionType` enum.
 */
export const ActionType = {
	NoOp: "NoOp",
	Eat: "Eat",
	Move: "Move",
	Reproduce: "Reproduce",
	StealEnergy: "StealEnergy",
} as const;

export type ActionType = (typeof ActionType)[keyof typeof ActionType];

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
	NutritionConstraints: "NutritionConstraints",
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

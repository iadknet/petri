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
	priority_bid: number;
}

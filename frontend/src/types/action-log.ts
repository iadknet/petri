/**
 * Action types — serialized as u8 discriminants from Rust's `ActionType` enum.
 */
export const ActionType = {
	NoOp: 0,
	Eat: 1,
	Move: 2,
	Reproduce: 3,
	StealEnergy: 4,
} as const;

export type ActionType = (typeof ActionType)[keyof typeof ActionType];

/**
 * Action results — serialized as u8 discriminants from Rust's `ActionResult` enum.
 */
export const ActionResult = {
	Success: 0,
	NoFood: 1,
	Blocked: 2,
	InvalidTarget: 3,
	EnergyConstraints: 4,
	PopulationCap: 5,
	TransferredAndKilled: 6,
	NoVictim: 7,
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

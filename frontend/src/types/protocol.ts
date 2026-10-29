import type { CreatureSnapshot } from "./generated/CreatureSnapshot.ts";
import type { FoodCell } from "./generated/FoodCell.ts";

/** The wire protocol this frontend speaks: `v3-server`'s `PROTOCOL_VERSION`. */
export const PROTOCOL_VERSION = "v3alpha19" as const;

/** A byte grid as the frontend holds it: decoded from a wire `number[]` or
 * built locally as a typed array. */
export type ByteArrayLike = Uint8Array | number[];

/** One rendered world frame, assembled from the view payloads. */
export interface Frame {
	width: number;
	height: number;
	creatures: CreatureSnapshot[];
	food: FoodCell[];
	barriers: { x: number; y: number }[];
}

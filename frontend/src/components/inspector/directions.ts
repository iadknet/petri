const DIRECTION_NAMES = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;
const DIRECTION_WORDS = [
	"north",
	"north-east",
	"east",
	"south-east",
	"south",
	"south-west",
	"west",
	"north-west",
] as const;

/** Map a ring sensor sub_idx to a direction name (N=0, NE=1, ..., NW=7). */
export function directionName(subIdx: number): string {
	return DIRECTION_NAMES[subIdx % 8] ?? `?${subIdx}`;
}

/** The spelled-out direction for a `Direction::ALL` index, for tooltips. */
export function directionWord(subIdx: number): string {
	return DIRECTION_WORDS[subIdx % 8] ?? `direction ${subIdx}`;
}

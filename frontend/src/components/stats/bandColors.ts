import type { BandId } from "./readings.ts";

/** Band identity hues, validated together on the panel surface for colour-vision deficiency. */
export const BAND_COLOR: Record<BandId, string> = {
	world: "#0d9488",
	evolution: "#d97706",
	brains: "#8b5cf6",
};

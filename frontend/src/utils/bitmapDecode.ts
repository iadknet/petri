import type { PatternBounds } from "../types/api.ts";

/**
 * Decode a base64-encoded pattern bitmap into a Set of "x,y" cell keys.
 *
 * The bitmap is row-major, MSB-first: bit 0 = top-left cell within bounds,
 * bit (width-1) = top-right, etc. Bit = 1 means barrier cell.
 */
export function decodeBitmap(bitmap: string, bounds: PatternBounds): Set<string> {
	const cells = new Set<string>();
	const bytes = base64ToBytes(bitmap);
	const totalBits = bounds.width * bounds.height;

	for (let i = 0; i < totalBits; i++) {
		const byteIdx = i >> 3; // Math.floor(i / 8)
		const bitIdx = 7 - (i & 7); // MSB-first: bit 7 is the leftmost
		if (byteIdx < bytes.length && (bytes[byteIdx]! >> bitIdx) & 1) {
			const x = bounds.x + (i % bounds.width);
			const y = bounds.y + Math.floor(i / bounds.width);
			cells.add(`${x},${y}`);
		}
	}

	return cells;
}

function base64ToBytes(base64: string): Uint8Array {
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i);
	}
	return bytes;
}

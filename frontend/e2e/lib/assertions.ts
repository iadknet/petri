import { waitFor } from "./wait.ts";

export function assert(condition: unknown, message: string): asserts condition {
	if (!condition) {
		throw new Error(message);
	}
}

export function parseTickValue(text: string): number {
	const match = text.match(/([0-9][0-9,]*)/u);
	if (!match) {
		throw new Error(`Unable to parse tick value from: ${text}`);
	}
	return Number(match[1].replaceAll(",", ""));
}

export async function waitForCondition(
	predicate: () => Promise<boolean> | boolean,
	description: string,
	timeoutMs: number,
): Promise<void> {
	await waitFor(predicate, {
		timeoutMs,
		description,
	});
}

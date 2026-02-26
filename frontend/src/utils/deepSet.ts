export function deepSet<T extends Record<string, unknown>>(obj: T, path: string, value: unknown): T {
	const clone = structuredClone(obj);
	const keys = path.split(".");
	let current: Record<string, unknown> = clone;
	for (let i = 0; i < keys.length - 1; i++) {
		const key = keys[i]!;
		if (typeof current[key] !== "object" || current[key] === null) {
			current[key] = {};
		}
		current = current[key] as Record<string, unknown>;
	}
	current[keys[keys.length - 1]!] = value;
	return clone;
}

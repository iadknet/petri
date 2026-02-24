export function getByPath(obj: unknown, path: string): unknown {
	let current = obj;
	for (const key of path.split(".")) {
		if (current == null || typeof current !== "object") {
			return undefined;
		}
		current = (current as Record<string, unknown>)[key];
	}
	return current;
}

export function buildPatch(path: string, value: number): Record<string, unknown> {
	const keys = path.split(".");
	const result: Record<string, unknown> = {};
	let current = result;
	for (let i = 0; i < keys.length - 1; i++) {
		const next: Record<string, unknown> = {};
		current[keys[i]!] = next;
		current = next;
	}
	current[keys[keys.length - 1]!] = value;
	return result;
}

export function mergePatch(target: Record<string, unknown>, source: Record<string, unknown>): void {
	for (const [key, value] of Object.entries(source)) {
		if (value && typeof value === "object" && !Array.isArray(value)) {
			if (
				!target[key] ||
				typeof target[key] !== "object" ||
				target[key] === null ||
				Array.isArray(target[key])
			) {
				target[key] = {};
			}
			mergePatch(target[key] as Record<string, unknown>, value as Record<string, unknown>);
			continue;
		}
		target[key] = value;
	}
}

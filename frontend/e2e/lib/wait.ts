export function sleep(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitFor(
	predicate: () => Promise<boolean> | boolean,
	options: {
		timeoutMs: number;
		intervalMs?: number;
		description: string;
	},
): Promise<void> {
	const startedAt = Date.now();
	const intervalMs = options.intervalMs ?? 250;

	while (Date.now() - startedAt < options.timeoutMs) {
		const pass = await predicate();
		if (pass) {
			return;
		}
		await sleep(intervalMs);
	}

	throw new Error(`Timed out waiting for ${options.description}`);
}

export async function waitForHttpOk(url: string, timeoutMs: number): Promise<void> {
	await waitFor(
		async () => {
			try {
				const response = await fetch(url);
				return response.ok;
			} catch {
				return false;
			}
		},
		{
			timeoutMs,
			intervalMs: 300,
			description: `HTTP OK from ${url}`,
		},
	);
}

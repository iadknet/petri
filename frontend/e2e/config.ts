import path from "node:path";
import type { CliArgs, E2EConfig } from "./types.ts";

const DEFAULT_STARTUP_TIMEOUT_MS = 60_000;
const DEFAULT_COMMAND_TIMEOUT_MS = 30_000;

function parseIntegerEnv(name: string): number | undefined {
	const raw = process.env[name];
	if (raw === undefined || raw === "") {
		return undefined;
	}
	const value = Number(raw);
	if (!Number.isInteger(value) || value <= 0) {
		throw new Error(`${name} must be a positive integer, got: ${raw}`);
	}
	return value;
}

export function parseCliArgs(argv: string[]): CliArgs {
	let scenario: string | undefined;
	let headed = false;

	for (let i = 0; i < argv.length; i += 1) {
		const arg = argv[i];
		if (arg === "--scenario") {
			const next = argv[i + 1];
			if (!next) {
				throw new Error("--scenario requires a value");
			}
			scenario = next;
			i += 1;
			continue;
		}
		if (arg === "--headed") {
			headed = true;
			continue;
		}
		throw new Error(`Unknown argument: ${arg}`);
	}

	return { scenario, headed };
}

export function loadE2EConfig(frontendDir: string): E2EConfig {
	const repoRoot = path.resolve(frontendDir, "..");

	return {
		frontendDir,
		repoRoot,
		startupTimeoutMs: parseIntegerEnv("E2E_STARTUP_TIMEOUT_MS") ?? DEFAULT_STARTUP_TIMEOUT_MS,
		commandTimeoutMs: parseIntegerEnv("E2E_CMD_TIMEOUT_MS") ?? DEFAULT_COMMAND_TIMEOUT_MS,
		artifactBaseDir:
			process.env.E2E_ARTIFACT_DIR ?? path.resolve(frontendDir, ".artifacts", "e2e"),
		fixedBackendPort: parseIntegerEnv("E2E_BACKEND_PORT"),
		fixedFrontendPort: parseIntegerEnv("E2E_FRONTEND_PORT"),
	};
}

import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { ScenarioStatus } from "../types.ts";

export interface MaybeWriteFailureArtifactsInput {
	scenarioId: string;
	status: ScenarioStatus;
	baseDir: string;
	jsonArtifacts: Record<string, unknown>;
	textArtifacts: Record<string, string>;
}

export async function ensureRunDirectory(baseDir: string): Promise<string> {
	const runDir = path.join(baseDir, `run-${new Date().toISOString().replaceAll(":", "-")}`);
	await mkdir(runDir, { recursive: true });
	return runDir;
}

export async function maybeWriteFailureArtifacts(
	input: MaybeWriteFailureArtifactsInput,
): Promise<string | null> {
	if (input.status !== "failed") {
		return null;
	}

	const scenarioDir = path.join(input.baseDir, input.scenarioId.toLowerCase());
	await mkdir(scenarioDir, { recursive: true });

	for (const [fileName, content] of Object.entries(input.jsonArtifacts)) {
		await writeFile(path.join(scenarioDir, fileName), JSON.stringify(content, null, 2), "utf8");
	}
	for (const [fileName, content] of Object.entries(input.textArtifacts)) {
		await writeFile(path.join(scenarioDir, fileName), content, "utf8");
	}

	return scenarioDir;
}

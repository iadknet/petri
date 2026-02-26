#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import path from "node:path";
import { AgentBrowserClient } from "./lib/agentBrowser.ts";
import { ensureRunDirectory, maybeWriteFailureArtifacts } from "./lib/artifacts.ts";
import { allocatePorts } from "./lib/ports.ts";
import { startLocalStack, withManagedStack } from "./lib/processManager.ts";
import { waitForHttpOk } from "./lib/wait.ts";
import { loadE2EConfig, parseCliArgs } from "./config.ts";
import { scenarioBootConnectivity } from "./scenarios/e2e-01-boot-connectivity.ts";
import { scenarioStartupLifecycleStep } from "./scenarios/e2e-02-startup-lifecycle-step.ts";
import { scenarioConfigApplyResetLocks } from "./scenarios/e2e-03-config-apply-reset-locks.ts";
import { scenarioStatsAndPanels } from "./scenarios/e2e-04-stats-and-panels.ts";
import { scenarioViewportSmoke } from "./scenarios/e2e-05-viewport-smoke.ts";
import { scenarioMutationConfig } from "./scenarios/e2e-06-mutation-config.ts";
import { scenarioPaintDrawing } from "./scenarios/e2e-07-paint-drawing.ts";
import type { ScenarioDefinition } from "./types.ts";

const scenarios: ScenarioDefinition[] = [
	scenarioBootConnectivity,
	scenarioStartupLifecycleStep,
	scenarioConfigApplyResetLocks,
	scenarioStatsAndPanels,
	scenarioViewportSmoke,
	scenarioMutationConfig,
	scenarioPaintDrawing,
];

async function main(): Promise<void> {
	const cli = parseCliArgs(process.argv.slice(2));
	const config = loadE2EConfig(process.cwd());
	const runDir = await ensureRunDirectory(config.artifactBaseDir);
	const ports = await allocatePorts({
		fixedBackendPort: config.fixedBackendPort,
		fixedFrontendPort: config.fixedFrontendPort,
	});

	const selected = cli.scenario
		? scenarios.filter((scenario) => scenario.id === cli.scenario)
		: scenarios;
	if (selected.length === 0) {
		throw new Error(`Unknown scenario: ${cli.scenario}`);
	}

	const failures: Array<{ id: string; reason: string; artifactsPath?: string }> = [];

	await withManagedStack(
		() =>
			startLocalStack({
				repoRoot: config.repoRoot,
				artifactBaseDir: runDir,
				backendPort: ports.backendPort,
				frontendPort: ports.frontendPort,
			}),
		async (stack) => {
			await waitForHttpOk(`${stack.backendUrl}/v3/simulation/status`, config.startupTimeoutMs);
			await waitForHttpOk(stack.frontendUrl, config.startupTimeoutMs);

			for (const scenario of selected) {
				const commands: string[] = [];
				const session = `petri-${scenario.id.toLowerCase()}-${Date.now()}`;
				const browser = new AgentBrowserClient({
					session,
					headed: cli.headed,
					timeoutMs: config.commandTimeoutMs,
					onCommand: (command) => commands.push(command),
				});

				let status: "passed" | "failed" = "passed";
				let errorReason = "";

				try {
					console.log(`Running ${scenario.id}: ${scenario.description}`);
					await scenario.run({
						frontendUrl: stack.frontendUrl,
						backendUrl: stack.backendUrl,
						startupTimeoutMs: config.startupTimeoutMs,
						browser,
					});
					console.log(`PASS ${scenario.id}`);
				} catch (error) {
					status = "failed";
					errorReason = error instanceof Error ? error.stack ?? error.message : String(error);
					console.error(`FAIL ${scenario.id}: ${errorReason}`);
				} finally {
					let artifactsPath: string | null = null;
					if (status === "failed") {
						const diagnostics = await collectDiagnostics(browser);
						const stackLog = await readFile(stack.stackLogPath, "utf8").catch(() => "");
						artifactsPath = await maybeWriteFailureArtifacts({
							scenarioId: scenario.id,
							status,
							baseDir: runDir,
							jsonArtifacts: {
								"snapshot-interactive.json": diagnostics.snapshotInteractive,
								"snapshot-interactive-cursor.json": diagnostics.snapshotInteractiveCursor,
								"diff-snapshot.json": diagnostics.diffSnapshot,
								"errors.json": diagnostics.errors,
								"console.json": diagnostics.console,
							},
							textArtifacts: {
								"commands.log": commands.join("\n"),
								"stack.log": stackLog,
								"failure.txt": errorReason,
							},
						});
						if (artifactsPath) {
							const screenshotPath = path.join(artifactsPath, "annotated.png");
							await browser.screenshotAnnotated(screenshotPath).catch(() => undefined);
						}
						failures.push({ id: scenario.id, reason: errorReason, artifactsPath: artifactsPath ?? undefined });
					}
					await browser.close().catch(() => undefined);
				}
			}
		},
	);

	console.log(`\nE2E Summary: ${selected.length - failures.length}/${selected.length} passed`);
	if (failures.length > 0) {
		for (const failure of failures) {
			console.error(`- ${failure.id}: ${failure.reason}`);
			if (failure.artifactsPath) {
				console.error(`  artifacts: ${failure.artifactsPath}`);
			}
		}
		process.exitCode = 1;
	}
}

async function collectDiagnostics(browser: AgentBrowserClient): Promise<{
	snapshotInteractive: unknown;
	snapshotInteractiveCursor: unknown;
	diffSnapshot: unknown;
	errors: unknown;
	console: unknown;
}> {
	const snapshotInteractive = await browser.snapshotInteractive().catch((error) => toErrorArtifact(error));
	const snapshotInteractiveCursor = await browser
		.snapshotInteractiveCursor()
		.catch((error) => toErrorArtifact(error));
	const diffSnapshot = await browser.diffSnapshot().catch((error) => toErrorArtifact(error));
	const errors = await browser.errors().catch((error) => toErrorArtifact(error));
	const consoleMessages = await browser.consoleMessages().catch((error) => toErrorArtifact(error));

	return {
		snapshotInteractive,
		snapshotInteractiveCursor,
		diffSnapshot,
		errors,
		console: consoleMessages,
	};
}

function toErrorArtifact(error: unknown): { error: string } {
	if (error instanceof Error) {
		return { error: error.message };
	}
	return { error: String(error) };
}

main().catch((error) => {
	console.error(error instanceof Error ? error.stack ?? error.message : String(error));
	process.exitCode = 1;
});

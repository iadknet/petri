// @vitest-environment node
import { spawn } from "node:child_process";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { terminateProcessTree, withManagedStack } from "../lib/processManager.ts";

describe("withManagedStack", () => {
	it("always stops stack when scenario succeeds", async () => {
		let stopCalls = 0;
		const result = await withManagedStack(
			async () => ({
				backendUrl: "http://localhost:1",
				frontendUrl: "http://localhost:2",
				stackLogPath: "/tmp/stack.log",
				stop: async () => {
					stopCalls += 1;
				},
			}),
			async () => "ok",
		);

		expect(result).toBe("ok");
		expect(stopCalls).toBe(1);
	});

	it("always stops stack when scenario throws", async () => {
		let stopCalls = 0;
		await expect(() =>
			withManagedStack(
				async () => ({
					backendUrl: "http://localhost:1",
					frontendUrl: "http://localhost:2",
					stackLogPath: "/tmp/stack.log",
					stop: async () => {
						stopCalls += 1;
					},
				}),
				async () => {
					throw new Error("fail");
				},
			),
		).rejects.toThrow("fail");

		expect(stopCalls).toBe(1);
	});

	it("kills descendant processes when terminating a detached stack process", async () => {
		const tempDir = await mkdtemp(path.join(tmpdir(), "petri-process-manager-"));
		const childPidPath = path.join(tempDir, "child.pid");
		const workerScriptPath = path.join(tempDir, "worker.js");
		const parentScriptPath = path.join(tempDir, "parent.js");

		await writeFile(
			workerScriptPath,
			[
				'import { writeFileSync } from "node:fs";',
				"writeFileSync(process.argv[2], String(process.pid));",
				"setInterval(() => {}, 1_000);",
			].join("\n"),
		);
		await writeFile(
			parentScriptPath,
			[
				'import { spawn } from "node:child_process";',
				'spawn(process.execPath, [process.argv[2], process.argv[3]], { stdio: "ignore" });',
				'process.on("SIGTERM", () => process.exit(0));',
				"setInterval(() => {}, 1_000);",
			].join("\n"),
		);

		const parent = spawn(process.execPath, [parentScriptPath, workerScriptPath, childPidPath], {
			detached: true,
			stdio: "ignore",
		});

		const childPid = Number.parseInt(await waitForFileContents(childPidPath), 10);
		expect(Number.isFinite(childPid)).toBe(true);

		await terminateProcessTree(parent, 250);

		await waitForCondition(() => !isPidAlive(childPid), `child ${childPid} still alive`);
	});
});

async function waitForFileContents(filePath: string): Promise<string> {
	for (let attempt = 0; attempt < 40; attempt += 1) {
		try {
			return (await readFile(filePath, "utf8")).trim();
		} catch {
			// Keep polling until the child has written its PID.
		}
		await new Promise((resolve) => setTimeout(resolve, 50));
	}

	throw new Error(`Timed out waiting for ${filePath}`);
}

function isPidAlive(pid: number): boolean {
	try {
		process.kill(pid, 0);
		return true;
	} catch {
		return false;
	}
}

async function waitForCondition(check: () => boolean, message: string): Promise<void> {
	for (let attempt = 0; attempt < 40; attempt += 1) {
		if (check()) {
			return;
		}
		await new Promise((resolve) => setTimeout(resolve, 50));
	}

	throw new Error(message);
}

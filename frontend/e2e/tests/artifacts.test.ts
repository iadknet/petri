// @vitest-environment node
import { mkdtemp, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { maybeWriteFailureArtifacts } from "../lib/artifacts.ts";

describe("maybeWriteFailureArtifacts", () => {
	it("does nothing for passed scenarios", async () => {
		const tmp = await mkdtemp(path.join(os.tmpdir(), "petri-e2e-pass-"));
		const out = await maybeWriteFailureArtifacts({
			scenarioId: "E2E-01",
			status: "passed",
			baseDir: tmp,
			jsonArtifacts: {},
			textArtifacts: {},
		});
		expect(out).toBeNull();
	});

	it("writes artifacts for failed scenarios", async () => {
		const tmp = await mkdtemp(path.join(os.tmpdir(), "petri-e2e-fail-"));
		const out = await maybeWriteFailureArtifacts({
			scenarioId: "E2E-02",
			status: "failed",
			baseDir: tmp,
			jsonArtifacts: {
				"errors.json": { errors: [] },
			},
			textArtifacts: {
				"commands.log": "cmd1\\ncmd2",
			},
		});

		expect(out).not.toBeNull();
		const commands = await readFile(path.join(out!, "commands.log"), "utf8");
		expect(commands).toContain("cmd1");
	});
});

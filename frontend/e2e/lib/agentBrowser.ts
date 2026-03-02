import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

export interface AgentBrowserEnvelope<T = unknown> {
	success: boolean;
	data: T;
	error: string | null;
}

export class AgentBrowserCommandError extends Error {
	readonly command: string;
	readonly output?: string;

	constructor(message: string, command: string, output?: string) {
		super(message);
		this.name = "AgentBrowserCommandError";
		this.command = command;
		this.output = output;
	}
}

function extractJsonLine(raw: string): string {
	const lines = raw
		.split(/\r?\n/u)
		.map((line) => line.trim())
		.filter((line) => line.length > 0);
	if (lines.length === 0) {
		throw new Error("no output");
	}
	return lines[lines.length - 1] ?? "";
}

export function parseAgentBrowserJson(raw: string): AgentBrowserEnvelope {
	let line: string;
	try {
		line = extractJsonLine(raw);
	} catch (error) {
		throw new AgentBrowserCommandError(
			`agent-browser returned empty output: ${String(error)}`,
			"parse",
			raw,
		);
	}

	try {
		return JSON.parse(line) as AgentBrowserEnvelope;
	} catch (error) {
		throw new AgentBrowserCommandError(
			`agent-browser returned malformed JSON: ${String(error)}`,
			"parse",
			raw,
		);
	}
}

export function assertAgentBrowserSuccess(
	envelope: AgentBrowserEnvelope,
	command: string,
): AgentBrowserEnvelope {
	if (!envelope.success) {
		throw new AgentBrowserCommandError(
			envelope.error ?? "agent-browser command failed",
			command,
			JSON.stringify(envelope),
		);
	}
	return envelope;
}

export function unwrapAgentBrowserEvalResult<T>(data: unknown): T {
	if (typeof data === "object" && data !== null && "result" in data) {
		return (data as { result: T }).result;
	}
	return data as T;
}

interface AgentBrowserOptions {
	session: string;
	headed: boolean;
	timeoutMs: number;
	onCommand?: (command: string) => void;
}

export class AgentBrowserClient {
	private readonly session: string;
	private readonly headed: boolean;
	private readonly timeoutMs: number;
	private readonly onCommand?: (command: string) => void;

	constructor(options: AgentBrowserOptions) {
		this.session = options.session;
		this.headed = options.headed;
		this.timeoutMs = options.timeoutMs;
		this.onCommand = options.onCommand;
	}

	private async run<T = unknown>(args: string[]): Promise<T> {
		const commandArgs = ["--session", this.session, "--json"];
		if (this.headed) {
			commandArgs.push("--headed");
		}
		commandArgs.push(...args);

		const commandString = `agent-browser ${commandArgs.map(quoteArg).join(" ")}`;
		this.onCommand?.(commandString);

		try {
			const { stdout, stderr } = await execFileAsync("agent-browser", commandArgs, {
				timeout: this.timeoutMs,
				maxBuffer: 2 * 1024 * 1024,
			});
			const output = stdout || stderr;
			const envelope = assertAgentBrowserSuccess(parseAgentBrowserJson(output), commandString);
			return envelope.data as T;
		} catch (error) {
			if (error instanceof AgentBrowserCommandError) {
				throw error;
			}
			const execError = error as {
				stdout?: string;
				stderr?: string;
				message: string;
			};
			const combinedOutput = [execError.stdout, execError.stderr].filter(Boolean).join("\n");
			if (combinedOutput) {
				const parsed = parseAgentBrowserJson(combinedOutput);
				assertAgentBrowserSuccess(parsed, commandString);
				return parsed.data as T;
			}
			throw new AgentBrowserCommandError(execError.message, commandString, combinedOutput);
		}
	}

	open(url: string): Promise<{ title?: string; url: string }> {
		return this.run(["open", url]);
	}

	waitLoad(state: "load" | "domcontentloaded" | "networkidle" = "networkidle"): Promise<unknown> {
		return this.run(["wait", "--load", state]);
	}

	snapshotInteractive(): Promise<{ snapshot: string; refs?: Record<string, unknown> }> {
		return this.run(["snapshot", "-i"]);
	}

	snapshotInteractiveCursor(): Promise<{ snapshot: string; refs?: Record<string, unknown> }> {
		return this.run(["snapshot", "-i", "-C"]);
	}

	click(selector: string): Promise<unknown> {
		return this.run(["click", selector]);
	}

	dblclick(selector: string): Promise<unknown> {
		return this.run(["dblclick", selector]);
	}

	hover(selector: string): Promise<unknown> {
		return this.run(["hover", selector]);
	}

	fill(selector: string, value: string): Promise<unknown> {
		return this.run(["fill", selector, value]);
	}

	findRoleClick(role: string, name: string): Promise<unknown> {
		return this.run(["find", "role", role, "click", "--name", name]);
	}

	press(key: string): Promise<unknown> {
		return this.run(["press", key]);
	}

	scroll(direction: "up" | "down" | "left" | "right", pixels: number): Promise<unknown> {
		return this.run(["scroll", direction, String(pixels)]);
	}

	mouseMove(x: number, y: number): Promise<unknown> {
		return this.run(["mouse", "move", String(Math.round(x)), String(Math.round(y))]);
	}

	mouseDown(button: "left" | "right" | "middle" = "left"): Promise<unknown> {
		return this.run(["mouse", "down", button]);
	}

	mouseUp(button: "left" | "right" | "middle" = "left"): Promise<unknown> {
		return this.run(["mouse", "up", button]);
	}

	async clickAt(x: number, y: number): Promise<void> {
		await this.mouseMove(x, y);
		await this.mouseDown();
		await this.mouseUp();
	}

	eval<T = unknown>(script: string): Promise<T> {
		return this.run<{ result: T } | T>(["eval", script]).then((data) =>
			unwrapAgentBrowserEvalResult<T>(data),
		);
	}

	isEnabled(selector: string): Promise<{ enabled: boolean }> {
		return this.run(["is", "enabled", selector]);
	}

	isVisible(selector: string): Promise<{ visible: boolean }> {
		return this.run(["is", "visible", selector]);
	}

	getText(selector: string): Promise<{ text: string }> {
		return this.run(["get", "text", selector]);
	}

	getValue(selector: string): Promise<{ value: string }> {
		return this.run(["get", "value", selector]);
	}

	getCount(selector: string): Promise<{ count: number }> {
		return this.run(["get", "count", selector]);
	}

	waitText(text: string): Promise<unknown> {
		return this.run(["wait", "--text", text]);
	}

	diffSnapshot(): Promise<{ changed: boolean; diff: string }> {
		return this.run(["diff", "snapshot"]);
	}

	errors(): Promise<{ errors: unknown[] }> {
		return this.run(["errors"]);
	}

	consoleMessages(): Promise<{ messages: unknown[] }> {
		return this.run(["console"]);
	}

	screenshotAnnotated(path: string): Promise<unknown> {
		return this.run(["screenshot", path, "--annotate"]);
	}

	close(): Promise<unknown> {
		return this.run(["close"]);
	}
}

function quoteArg(value: string): string {
	if (/^[a-zA-Z0-9_./:-]+$/u.test(value)) {
		return value;
	}
	return JSON.stringify(value);
}

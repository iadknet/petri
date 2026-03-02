import { spawn } from "node:child_process";
import { createWriteStream } from "node:fs";
import { mkdir } from "node:fs/promises";
import path from "node:path";

export interface ManagedStack {
	backendUrl: string;
	frontendUrl: string;
	stackLogPath: string;
	stop: () => Promise<void>;
}

export async function withManagedStack<T>(
	start: () => Promise<ManagedStack>,
	run: (stack: ManagedStack) => Promise<T>,
): Promise<T> {
	const stack = await start();
	try {
		return await run(stack);
	} finally {
		await stack.stop();
	}
}

export interface StartStackOptions {
	repoRoot: string;
	artifactBaseDir: string;
	backendPort: number;
	frontendPort: number;
}

export async function startLocalStack(options: StartStackOptions): Promise<ManagedStack> {
	const backendUrl = `http://localhost:${options.backendPort}`;
	const frontendUrl = `http://localhost:${options.frontendPort}`;
	const logDir = path.join(options.artifactBaseDir, "runtime");
	await mkdir(logDir, { recursive: true });
	const stackLogPath = path.join(logDir, `stack-${Date.now()}.log`);
	const logStream = createWriteStream(stackLogPath, { flags: "a" });

	const scriptPath = path.join(options.repoRoot, "scripts", "dev.sh");
	const child = spawn(scriptPath, {
		cwd: options.repoRoot,
		detached: true,
		env: {
			...process.env,
			BACKEND_PORT: String(options.backendPort),
			FRONTEND_PORT: String(options.frontendPort),
			FRONTEND_PROXY_TARGET: backendUrl,
		},
		stdio: ["ignore", "pipe", "pipe"],
	});

	child.stdout?.on("data", (chunk) => logStream.write(chunk));
	child.stderr?.on("data", (chunk) => logStream.write(chunk));

	let stopped = false;
	const stop = async () => {
		if (stopped) {
			return;
		}
		stopped = true;
		await terminateProcessTree(child, 5_000);
		logStream.end();
	};

	return {
		backendUrl,
		frontendUrl,
		stackLogPath,
		stop,
	};
}

export async function terminateProcessTree(
	child: ReturnType<typeof spawn>,
	graceMs: number,
): Promise<void> {
	if (child.exitCode !== null || child.killed) {
		return;
	}

	sendSignal(child, "SIGTERM");
	const exited = await waitForExit(child, graceMs);
	if (exited) {
		return;
	}

	sendSignal(child, "SIGKILL");
	await waitForExit(child, graceMs);
}

function sendSignal(child: ReturnType<typeof spawn>, signal: NodeJS.Signals): void {
	const pid = child.pid;
	if (pid === undefined) {
		child.kill(signal);
		return;
	}

	try {
		process.kill(-pid, signal);
	} catch {
		child.kill(signal);
	}
}

async function waitForExit(child: ReturnType<typeof spawn>, timeoutMs: number): Promise<boolean> {
	if (child.exitCode !== null) {
		return true;
	}

	return new Promise((resolve) => {
		let settled = false;
		const timer = setTimeout(() => {
			if (settled) {
				return;
			}
			settled = true;
			resolve(false);
		}, timeoutMs);

		child.once("exit", () => {
			if (settled) {
				return;
			}
			settled = true;
			clearTimeout(timer);
			resolve(true);
		});
	});
}

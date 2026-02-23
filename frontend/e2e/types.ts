import type { AgentBrowserClient } from "./lib/agentBrowser.ts";

export interface CliArgs {
	scenario?: string;
	headed: boolean;
}

export interface E2EConfig {
	frontendDir: string;
	repoRoot: string;
	startupTimeoutMs: number;
	commandTimeoutMs: number;
	artifactBaseDir: string;
	fixedBackendPort?: number;
	fixedFrontendPort?: number;
}

export interface RuntimeContext {
	frontendUrl: string;
	backendUrl: string;
	startupTimeoutMs: number;
	browser: AgentBrowserClient;
}

export interface ScenarioDefinition {
	id: string;
	description: string;
	run: (ctx: RuntimeContext) => Promise<void>;
}

export type ScenarioStatus = "passed" | "failed";

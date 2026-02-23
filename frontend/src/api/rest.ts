import type {
	ApiError,
	ConfigResponse,
	FrameResponse,
	LifecycleResponse,
	SimulationConfig,
	StartupRequest,
	StartupResponse,
	StatusResponse,
	StepRequest,
	StepResponse,
} from "../types/api.ts";

const BASE_URL = import.meta.env.VITE_API_URL ?? "";

class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = BASE_URL) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(path: string, options?: RequestInit): Promise<T> {
		const res = await fetch(`${this.baseUrl}${path}`, {
			headers: { "Content-Type": "application/json" },
			...options,
		});
		if (!res.ok) {
			const body = (await res.json()) as ApiError;
			throw new ApiRequestError(res.status, body);
		}
		return res.json() as Promise<T>;
	}

	async startup(req: StartupRequest): Promise<StartupResponse> {
		return this.request("/v3/simulation/startup", {
			method: "POST",
			body: JSON.stringify(req),
		});
	}

	async start(): Promise<LifecycleResponse> {
		return this.request("/v3/simulation/start", { method: "POST" });
	}

	async pause(): Promise<LifecycleResponse> {
		return this.request("/v3/simulation/pause", { method: "POST" });
	}

	async step(req?: StepRequest): Promise<StepResponse> {
		return this.request("/v3/simulation/step", {
			method: "POST",
			body: req ? JSON.stringify(req) : undefined,
		});
	}

	async getStatus(): Promise<StatusResponse> {
		return this.request("/v3/simulation/status");
	}

	async getFrame(): Promise<FrameResponse> {
		return this.request("/v3/simulation/frame");
	}

	async getConfig(): Promise<ConfigResponse> {
		return this.request("/v3/simulation/config");
	}

	async patchConfig(patch: DeepPartial<SimulationConfig>): Promise<ConfigResponse> {
		return this.request("/v3/simulation/config", {
			method: "PATCH",
			body: JSON.stringify(patch),
		});
	}
}

export class ApiRequestError extends Error {
	status: number;
	body: ApiError;

	constructor(status: number, body: ApiError) {
		super(body.error.message);
		this.name = "ApiRequestError";
		this.status = status;
		this.body = body;
	}
}

/** Deep partial utility for nested config patches */
export type DeepPartial<T> = {
	[P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export const api = new ApiClient();

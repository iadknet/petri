import type {
	ApiError,
	ConfigResponse,
	CreatureDetail,
	LifecycleResponse,
	PaintRequest,
	PaintResponse,
	SampleResponse,
	SimulationConfig,
	SnapshotResponse,
	StartSampleResponse,
	StartupRequest,
	StartupResponse,
	StatusResponse,
	StepRequest,
	StepResponse,
	ZoomTier,
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

	async getSnapshot(query?: {
		x?: number;
		y?: number;
		width?: number;
		height?: number;
		canvas_width?: number;
		canvas_height?: number;
		zoom_tier?: ZoomTier;
	}): Promise<SnapshotResponse> {
		const params = new URLSearchParams();
		if (query) {
			for (const [key, value] of Object.entries(query)) {
				if (value === undefined) continue;
				params.set(key, String(value));
			}
		}
		const suffix = params.size > 0 ? `?${params.toString()}` : "";
		return this.request(`/v3/simulation/snapshot${suffix}`);
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

	async getCreature(id: number, signal?: AbortSignal): Promise<CreatureDetail> {
		return this.request(`/v3/simulation/creature/${id}`, { signal });
	}

	async paint(req: PaintRequest): Promise<PaintResponse> {
		return this.request("/v3/simulation/paint", {
			method: "POST",
			body: JSON.stringify(req),
		});
	}

	async startSample(id: number, ticks = 5, signal?: AbortSignal): Promise<StartSampleResponse> {
		return this.request(`/v3/simulation/creature/${id}/sample`, {
			method: "POST",
			body: JSON.stringify({ ticks }),
			signal,
		});
	}

	async getSample(id: number, signal?: AbortSignal): Promise<SampleResponse> {
		return this.request(`/v3/simulation/creature/${id}/sample`, { signal });
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

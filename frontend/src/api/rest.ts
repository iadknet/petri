import type {
	ConfigResponse,
	CreatureDetail,
	LifecycleResponse,
	PaintRequest,
	PaintResponse,
	PatternApplyResponse,
	PatternPreviewResponse,
	PatternRequest,
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
import type { ApiError, FieldError } from "../types/errors.ts";

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

	async getCreature(
		id: number,
		signal?: AbortSignal,
		query?: { since_tick?: number; exclude?: string },
	): Promise<CreatureDetail> {
		const params = new URLSearchParams();
		if (query?.since_tick !== undefined) {
			params.set("since_tick", String(query.since_tick));
		}
		if (query?.exclude) {
			params.set("exclude", query.exclude);
		}
		const suffix = params.size > 0 ? `?${params.toString()}` : "";
		return this.request(`/v3/simulation/creature/${id}${suffix}`, { signal });
	}

	async paint(req: PaintRequest): Promise<PaintResponse> {
		return this.request("/v3/simulation/paint", {
			method: "POST",
			body: JSON.stringify(req),
		});
	}

	async patternPreview(req: PatternRequest, signal?: AbortSignal): Promise<PatternPreviewResponse> {
		return this.request("/v3/simulation/pattern/preview", {
			method: "POST",
			body: JSON.stringify(req),
			signal,
		});
	}

	async patternApply(req: PatternRequest): Promise<PatternApplyResponse> {
		return this.request("/v3/simulation/pattern/apply", {
			method: "POST",
			body: JSON.stringify(req),
		});
	}

	async startSample(id: number, ticks = 10, signal?: AbortSignal): Promise<StartSampleResponse> {
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

/**
 * Builds a non-empty message for a failed request. An empty message renders as
 * no message at all, which is how config rejections used to go unnoticed.
 */
function errorMessage(status: number, body: ApiError): string {
	const message = body?.error?.message;
	if (message) return message;
	const code = body?.error?.code;
	return code ? `${code} (HTTP ${status})` : `Request failed (HTTP ${status})`;
}

export class ApiRequestError extends Error {
	status: number;
	body: ApiError;
	/** Per-field reasons from `error.details.field_errors`; empty when absent. */
	fieldErrors: FieldError[];

	constructor(status: number, body: ApiError) {
		super(errorMessage(status, body));
		this.name = "ApiRequestError";
		this.status = status;
		this.body = body;
		this.fieldErrors = body?.error?.details?.field_errors ?? [];
	}
}

/** Deep partial utility for nested config patches */
export type DeepPartial<T> = {
	[P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export const api = new ApiClient();

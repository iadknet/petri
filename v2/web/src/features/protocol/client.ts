import {
  decodeErrorEnvelope,
  decodeFramePayload,
  decodeLifecycleResponse,
  decodePaintResponse,
  decodeStartupResponse,
  decodeStatusPayload,
  decodeWsEventEnvelope,
} from "./decoders";
import type {
  ErrorEnvelope,
  FramePayload,
  LifecycleResponse,
  PaintRequest,
  PaintResponse,
  StartupRequest,
  StartupResponse,
  StatusPayload,
  WsEventEnvelope,
} from "./models";

export class ProtocolClient {
  constructor(
    private readonly baseUrl = "",
    private readonly wsBaseUrl = ""
  ) {}

  async startup(request: StartupRequest): Promise<StartupResponse> {
    const json = await this.fetchJson("/v2/simulation/startup", {
      method: "POST",
      body: JSON.stringify(request),
    });
    return decodeStartupResponse(json);
  }

  async start(): Promise<LifecycleResponse> {
    const json = await this.fetchJson("/v2/simulation/start", { method: "POST" });
    return decodeLifecycleResponse(json);
  }

  async pause(): Promise<LifecycleResponse> {
    const json = await this.fetchJson("/v2/simulation/pause", { method: "POST" });
    return decodeLifecycleResponse(json);
  }

  async step(steps = 1): Promise<LifecycleResponse> {
    const json = await this.fetchJson("/v2/simulation/step", {
      method: "POST",
      body: JSON.stringify({ steps }),
    });
    return decodeLifecycleResponse(json);
  }

  async paint(request: PaintRequest): Promise<PaintResponse> {
    const json = await this.fetchJson("/v2/simulation/world/paint", {
      method: "POST",
      body: JSON.stringify(request),
    });
    return decodePaintResponse(json);
  }

  async fetchStatus(): Promise<StatusPayload> {
    const json = await this.fetchJson("/v2/simulation/status");
    return decodeStatusPayload(json);
  }

  async fetchFrame(): Promise<FramePayload> {
    const json = await this.fetchJson("/v2/simulation/frame");
    return decodeFramePayload(json);
  }

  decodeWsMessage(raw: string): WsEventEnvelope {
    return decodeWsEventEnvelope(JSON.parse(raw));
  }

  connectWs(
    onEvent: (event: WsEventEnvelope) => void,
    onError: (error: Error) => void
  ): () => void {
    const ws = new WebSocket(this.wsUrl("/v2/ws"));
    ws.onmessage = (message) => {
      try {
        onEvent(this.decodeWsMessage(String(message.data)));
      } catch (error) {
        onError(error instanceof Error ? error : new Error(String(error)));
      }
    };
    ws.onerror = () => {
      onError(new Error("websocket connection error"));
    };

    return () => {
      ws.close();
    };
  }

  private wsUrl(path: string): string {
    if (this.wsBaseUrl) {
      const wsBase = this.wsBaseUrl.endsWith("/")
        ? this.wsBaseUrl.slice(0, -1)
        : this.wsBaseUrl;
      if (wsBase.startsWith("http://") || wsBase.startsWith("https://")) {
        return `${wsBase.replace(/^http/, "ws")}${path}`;
      }
      return `${wsBase}${path}`;
    }

    if (this.baseUrl) {
      return `${this.baseUrl.replace(/^http/, "ws")}${path}`;
    }

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.host}${path}`;
  }

  private async fetchJson(path: string, init: RequestInit = {}): Promise<unknown> {
    const method = init.method?.toUpperCase() ?? "GET";
    const defaultHeaders =
      method === "GET" ? {} : ({ "content-type": "application/json" } as Record<string, string>);

    const response = await fetch(`${this.baseUrl}${path}`, {
      ...init,
      headers: {
        ...defaultHeaders,
        ...(init.headers ?? {}),
      },
    });

    const raw = await response.text();
    const parsed = parseJsonBody(raw);
    if (!response.ok) {
      if (!parsed.ok) {
        throw new Error(`request failed with HTTP ${response.status}: non-json response`);
      }
      throw protocolErrorFromEnvelope(parsed.value, response.status);
    }

    if (!parsed.ok) {
      throw new Error(
        `request failed with HTTP ${response.status}: invalid JSON response`
      );
    }
    return parsed.value;
  }
}

function protocolErrorFromEnvelope(raw: unknown, status: number): Error {
  let envelope: ErrorEnvelope;
  try {
    envelope = decodeErrorEnvelope(raw);
  } catch {
    return new Error(
      `request failed with HTTP ${status}: invalid protocol error envelope`
    );
  }
  return new Error(
    `request failed with HTTP ${status}: ${envelope.error.code}: ${envelope.error.message}`
  );
}

function parseJsonBody(raw: string): { ok: true; value: unknown } | { ok: false } {
  if (raw.trim().length === 0) {
    return { ok: false };
  }

  try {
    return { ok: true, value: JSON.parse(raw) as unknown };
  } catch {
    return { ok: false };
  }
}

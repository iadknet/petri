import {
  decodeErrorEnvelope,
  decodeFramePayload,
  decodeStatusPayload,
  decodeWsEventEnvelope,
} from "./decoders";
import type {
  ErrorEnvelope,
  FramePayload,
  StatusPayload,
  WsEventEnvelope,
} from "./models";

export class ProtocolClient {
  constructor(private readonly baseUrl: string) {}

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

  private async fetchJson(path: string): Promise<unknown> {
    const response = await fetch(`${this.baseUrl}${path}`);
    const json = (await response.json()) as unknown;
    if (!response.ok) {
      throw protocolErrorFromEnvelope(json);
    }
    return json;
  }
}

function protocolErrorFromEnvelope(raw: unknown): Error {
  let envelope: ErrorEnvelope;
  try {
    envelope = decodeErrorEnvelope(raw);
  } catch {
    return new Error("request failed and response was not a valid protocol envelope");
  }
  return new Error(`${envelope.error.code}: ${envelope.error.message}`);
}

import { decode } from "@msgpack/msgpack";
import type { ClientMessage, ServerMessage } from "../types/api.ts";

export type WsConnectionStatus = "disconnected" | "connecting" | "connected";

export type WsClientEvent =
	| { type: "connection"; status: WsConnectionStatus }
	| { type: "message"; message: ServerMessage };

function isServerMessage(value: unknown): value is ServerMessage {
	return (
		typeof value === "object" &&
		value !== null &&
		"type" in value &&
		typeof (value as { type?: unknown }).type === "string"
	);
}

export function decodeServerMessage(data: ArrayBuffer): ServerMessage | null {
	try {
		const decoded = decode(new Uint8Array(data));
		return isServerMessage(decoded) ? decoded : null;
	} catch {
		return null;
	}
}

export function encodeClientMessage(message: ClientMessage): string {
	return JSON.stringify(message);
}

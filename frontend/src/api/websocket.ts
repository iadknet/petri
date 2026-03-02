import type { ClientMessage } from "../types/api.ts";
import {
	type WsClientEvent,
	type WsConnectionStatus,
	decodeServerMessage,
	encodeClientMessage,
} from "./protocol.ts";

const BACKOFF_BASE = 1000;
const BACKOFF_CAP = 10000;

type WsListener = (event: WsClientEvent) => void;

export class WsClient {
	private ws: WebSocket | null = null;
	private reconnectAttempt = 0;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private disposed = false;
	private readonly url: string;
	private readonly listeners = new Set<WsListener>();

	constructor(url?: string) {
		const base = import.meta.env.VITE_API_URL ?? window.location.origin;
		const wsBase = base.replace(/^http/, "ws");
		this.url = url ?? `${wsBase}/v3/ws`;
	}

	subscribe(listener: WsListener): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	connect(): void {
		this.disposed = false;
		if (this.ws) return;

		this.emitConnection("connecting");

		const socket = new WebSocket(this.url);
		socket.binaryType = "arraybuffer";
		this.ws = socket;

		socket.onopen = () => {
			if (this.ws !== socket) return;
			this.reconnectAttempt = 0;
			this.emitConnection("connected");
		};

		socket.onmessage = (event) => {
			if (this.ws !== socket) return;
			this.handleMessage(event.data as ArrayBuffer);
		};

		socket.onclose = () => {
			if (this.ws !== socket) return;
			this.ws = null;
			this.emitConnection("disconnected");
			this.scheduleReconnect();
		};

		socket.onerror = () => {
			if (this.ws !== socket) return;
			socket.close();
		};
	}

	disconnect(): void {
		this.disposed = true;
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}

		if (!this.ws) {
			return;
		}

		this.ws.onopen = null;
		this.ws.onmessage = null;
		this.ws.onclose = null;
		this.ws.onerror = null;
		this.ws.close();

		this.ws = null;
		this.emitConnection("disconnected");
	}

	send(message: ClientMessage): boolean {
		if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
			return false;
		}

		this.ws.send(encodeClientMessage(message));
		return true;
	}

	private handleMessage(data: ArrayBuffer): void {
		const message = decodeServerMessage(data);
		if (!message) return;
		this.emit({ type: "message", message });
	}

	private emit(event: WsClientEvent): void {
		for (const listener of this.listeners) {
			listener(event);
		}
	}

	private emitConnection(status: WsConnectionStatus): void {
		this.emit({ type: "connection", status });
	}

	private scheduleReconnect(): void {
		if (this.disposed) return;
		const delay = Math.min(BACKOFF_BASE * 2 ** this.reconnectAttempt, BACKOFF_CAP);
		this.reconnectAttempt++;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
		}, delay);
	}
}

export const wsClient = new WsClient();

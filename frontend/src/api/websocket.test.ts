import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WsClient } from "./websocket.ts";

class MockWebSocket {
	static instances: MockWebSocket[] = [];

	url: string;
	onopen: ((event: Event) => void) | null = null;
	onmessage: ((event: MessageEvent<string>) => void) | null = null;
	onclose: ((event: CloseEvent) => void) | null = null;
	onerror: ((event: Event) => void) | null = null;

	constructor(url: string | URL) {
		this.url = url.toString();
		MockWebSocket.instances.push(this);
	}

	close(): void {}
}

describe("WsClient", () => {
	const originalWebSocket = globalThis.WebSocket;

	beforeEach(() => {
		MockWebSocket.instances = [];
		globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket;
	});

	afterEach(() => {
		globalThis.WebSocket = originalWebSocket;
		vi.restoreAllMocks();
	});

	it("reconnects after a disconnect and second connect call", () => {
		const client = new WsClient("ws://localhost:9999/v3/ws");

		client.connect();
		expect(MockWebSocket.instances).toHaveLength(1);

		client.disconnect();
		client.connect();

		expect(MockWebSocket.instances).toHaveLength(2);
	});
});

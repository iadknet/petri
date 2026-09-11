import { encode } from "@msgpack/msgpack";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useSimulationStore } from "../stores/simulation.ts";
import type { ServerMessage } from "../types/api.ts";
import { WsClient } from "./websocket.ts";

class MockWebSocket {
	static instances: MockWebSocket[] = [];
	static readonly CONNECTING = 0;
	static readonly OPEN = 1;
	static readonly CLOSING = 2;
	static readonly CLOSED = 3;

	url: string;
	binaryType = "blob";
	readyState = MockWebSocket.CONNECTING;
	sent: Array<string | ArrayBufferLike | Blob | ArrayBufferView> = [];
	onopen: ((event: Event) => void) | null = null;
	onmessage: ((event: MessageEvent<ArrayBuffer>) => void) | null = null;
	onclose: ((event: CloseEvent) => void) | null = null;
	onerror: ((event: Event) => void) | null = null;

	constructor(url: string | URL) {
		this.url = url.toString();
		MockWebSocket.instances.push(this);
	}

	send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
		this.sent.push(data);
	}

	close(): void {}
}

function openSocket(socket: MockWebSocket | undefined): void {
	expect(socket).toBeTruthy();
	if (!socket) {
		return;
	}
	socket.readyState = MockWebSocket.OPEN;
	socket.onopen?.(new Event("open"));
}

describe("WsClient", () => {
	const originalWebSocket = globalThis.WebSocket;

	beforeEach(() => {
		MockWebSocket.instances = [];
		globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket;
		useSimulationStore.getState().reset();
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

	it("does not emit disconnected when disconnect is called cold", () => {
		const client = new WsClient("ws://localhost:9999/v3/ws");
		const events: Array<{ type: string; status?: string }> = [];
		client.subscribe((event) => {
			if (event.type === "connection") {
				events.push({ type: event.type, status: event.status });
				return;
			}
			events.push({ type: event.type });
		});

		client.disconnect();

		expect(events).toEqual([]);
	});

	it("emits typed connection and message events without mutating stores directly", () => {
		const client = new WsClient("ws://localhost:9999/v3/ws");
		const events: Array<{ type: string }> = [];
		client.subscribe((event) => {
			events.push({ type: event.type });
		});

		client.connect();
		const socket = MockWebSocket.instances[0];
		expect(socket).toBeTruthy();

		openSocket(socket);

		const message: ServerMessage = {
			type: "status",
			protocol_version: "v3alpha2",
			projection_revision: 7,
			world_static_revision: 3,
			tick: 42,
			payload: {
				state: "running",
				population: 99,
				mean_energy: 18,
				last_tick_actions: { move: 1, eat: 2, reproduce: 3, noop: 4, steal: 0, predation_kills: 0 },
				reproduction_actions_attempted_total: 10,
				reproduction_actions_spawned_total: 3,
				reproduction_actions_rejected_total: 7,
				predation_actions_attempted_total: 2,
				predation_actions_transferred_total: 1,
				predation_actions_rejected_total: 1,
				predation_kills_total: 0,
				predation_actions_by_result: {},
				mutation_events_attempted_total: 5,
				mutation_events_applied_total: 4,
				mutation_events_skipped_total: 1,
				mutation_events_attempted_total_by_domain: {},
				mutation_events_applied_total_by_domain: {},
				mutation_events_attempted_total_by_operator: {},
				mutation_events_applied_total_by_operator: {},
				last_tick_compute_energy_total_mean: 1,
				last_tick_compute_energy_total_min: 1,
				last_tick_compute_energy_total_max: 1,
				last_tick_compute_energy_vm_mean: 1,
				last_tick_compute_energy_graph_mean: 1,
				perf: { projection_publish_ms: 1, ws_frame_publish_ms: 1, subscriber_count: 1 },
			},
		};

		const encoded = encode(message);
		socket?.onmessage?.({
			data: encoded.buffer.slice(
				encoded.byteOffset,
				encoded.byteOffset + encoded.byteLength,
			) as ArrayBuffer,
		} as MessageEvent<ArrayBuffer>);

		expect(events).toEqual([{ type: "connection" }, { type: "connection" }, { type: "message" }]);
		expect(useSimulationStore.getState().status).toBeNull();
	});

	it("serializes subscribe_view messages onto the socket", () => {
		const client = new WsClient("ws://localhost:9999/v3/ws");

		client.connect();
		const socket = MockWebSocket.instances[0];
		openSocket(socket);

		client.send({
			type: "subscribe_view",
			request_id: 9,
			x: 1,
			y: 2,
			width: 30,
			height: 40,
			canvas_width: 300,
			canvas_height: 200,
			zoom_tier: "detail",
		});

		expect(socket?.sent).toEqual([
			JSON.stringify({
				type: "subscribe_view",
				request_id: 9,
				x: 1,
				y: 2,
				width: 30,
				height: 40,
				canvas_width: 300,
				canvas_height: 200,
				zoom_tier: "detail",
			}),
		]);
	});

	it("does not send while the socket is still connecting", () => {
		const client = new WsClient("ws://localhost:9999/v3/ws");

		client.connect();
		const socket = MockWebSocket.instances[0];
		expect(socket).toBeTruthy();

		expect(
			client.send({
				type: "subscribe_view",
				request_id: 1,
				x: 0,
				y: 0,
				width: 10,
				height: 10,
				canvas_width: 100,
				canvas_height: 100,
				zoom_tier: "overview",
			}),
		).toBe(false);
		expect(socket?.sent).toEqual([]);
	});
});

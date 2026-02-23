import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import { PROTOCOL_VERSION } from "../types/api.ts";
import type { Frame, HealthPayload, StatusPayload, WsEnvelope } from "../types/api.ts";
import { api } from "./rest.ts";

const BACKOFF_BASE = 1000;
const BACKOFF_CAP = 10000;

export class WsClient {
	private ws: WebSocket | null = null;
	private reconnectAttempt = 0;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private disposed = false;
	private url: string;

	constructor(url?: string) {
		const base = import.meta.env.VITE_API_URL ?? window.location.origin;
		const wsBase = base.replace(/^http/, "ws");
		this.url = url ?? `${wsBase}/v3/ws`;
	}

	connect(): void {
		this.disposed = false;
		if (this.ws) return;

		const sim = useSimulationStore.getState();
		sim.setConnectionStatus("connecting");

		const socket = new WebSocket(this.url);
		this.ws = socket;

		socket.onopen = () => {
			if (this.ws !== socket) return;
			this.reconnectAttempt = 0;
			useSimulationStore.getState().setConnectionStatus("connected");
			this.resync();
		};

		socket.onmessage = (event) => {
			if (this.ws !== socket) return;
			this.handleMessage(event.data as string);
		};

		socket.onclose = () => {
			if (this.ws !== socket) return;
			this.ws = null;
			useSimulationStore.getState().setConnectionStatus("disconnected");
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
		if (this.ws) {
			this.ws.onopen = null;
			this.ws.onmessage = null;
			this.ws.onclose = null;
			this.ws.onerror = null;
			this.ws.close();
		}
		this.ws = null;
	}

	private handleMessage(data: string): void {
		let envelope: WsEnvelope;
		try {
			envelope = JSON.parse(data) as WsEnvelope;
		} catch {
			return;
		}

		if (envelope.protocol_version !== PROTOCOL_VERSION) {
			console.warn(
				`Protocol mismatch: expected ${PROTOCOL_VERSION}, got ${envelope.protocol_version}`,
			);
			return;
		}

		const sim = useSimulationStore.getState();
		const stats = useStatsHistoryStore.getState();

		switch (envelope.event) {
			case "status": {
				const payload = envelope.payload as StatusPayload;
				sim.setStatus(envelope.tick, payload);
				stats.pushStats(envelope.tick, payload.population, payload.mean_energy);
				stats.pushActions(envelope.tick, payload.last_tick_actions);
				break;
			}
			case "frame": {
				const payload = envelope.payload as Frame;
				sim.setFrame(envelope.tick, payload);
				break;
			}
			case "health": {
				const payload = envelope.payload as HealthPayload;
				sim.setHealth(envelope.tick, payload);
				stats.setReproStats(
					payload.reproduction_actions_attempted_total,
					payload.reproduction_actions_spawned_total,
					payload.reproduction_actions_rejected_total,
					payload.reproduction_actions_rejected_total_by_reason,
				);
				stats.setMutationStats(
					payload.mutation_events_attempted_total,
					payload.mutation_events_applied_total,
					payload.mutation_events_skipped_total,
				);
				break;
			}
		}
	}

	private async resync(): Promise<void> {
		try {
			const [status, frame] = await Promise.all([api.getStatus(), api.getFrame()]);
			const sim = useSimulationStore.getState();
			sim.setStatus(status.tick, {
				state: status.state,
				population: status.population,
				mean_energy: status.mean_energy,
				last_tick_actions: status.last_tick_actions,
				reproduction_actions_attempted_total: status.reproduction_actions_attempted_total,
				reproduction_actions_spawned_total: status.reproduction_actions_spawned_total,
				reproduction_actions_rejected_total: status.reproduction_actions_rejected_total,
			});
			sim.setFrame(frame.tick, {
				width: frame.width,
				height: frame.height,
				creatures: frame.creatures,
				food: frame.food,
				barriers: frame.barriers,
			});
		} catch {
			// Resync failure is non-fatal; next WS events will update state
		}
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

/** Singleton instance */
export const wsClient = new WsClient();

import { decode } from "@msgpack/msgpack";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import type { WsFrame } from "../types/api.ts";
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
		socket.binaryType = "arraybuffer";
		this.ws = socket;

		socket.onopen = () => {
			if (this.ws !== socket) return;
			this.reconnectAttempt = 0;
			useSimulationStore.getState().setConnectionStatus("connected");
			this.resync();
		};

		socket.onmessage = (event) => {
			if (this.ws !== socket) return;
			this.handleMessage(event.data as ArrayBuffer);
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

	private handleMessage(data: ArrayBuffer): void {
		let frame: WsFrame;
		try {
			frame = decode(new Uint8Array(data)) as WsFrame;
		} catch {
			return;
		}

		const sim = useSimulationStore.getState();
		const stats = useStatsHistoryStore.getState();

		sim.setStatus(frame.tick, frame.status);
		stats.pushStats(frame.tick, frame.status.population, frame.status.mean_energy);
		stats.pushActions(frame.tick, frame.status.last_tick_actions);
		stats.pushCompute(
			frame.tick,
			frame.status.last_tick_compute_total_mean,
			frame.status.last_tick_compute_total_min,
			frame.status.last_tick_compute_total_max,
			frame.status.last_tick_compute_vm_mean,
			frame.status.last_tick_compute_graph_mean,
		);

		sim.setFrame(frame.tick, frame.frame);

		sim.setHealth(frame.tick, frame.health);
		stats.setReproStats(
			frame.health.reproduction_actions_attempted_total,
			frame.health.reproduction_actions_spawned_total,
			frame.health.reproduction_actions_rejected_total,
			frame.health.reproduction_actions_rejected_total_by_reason,
		);
		stats.setMutationStats(
			frame.health.mutation_events_attempted_total,
			frame.health.mutation_events_applied_total,
			frame.health.mutation_events_skipped_total,
		);
		stats.pushComplexity(
			frame.tick,
			frame.health.genome_complexity_mean,
			frame.health.genome_complexity_min,
			frame.health.genome_complexity_max,
		);
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
				last_tick_compute_total_mean: status.last_tick_compute_total_mean,
				last_tick_compute_total_min: status.last_tick_compute_total_min,
				last_tick_compute_total_max: status.last_tick_compute_total_max,
				last_tick_compute_vm_mean: status.last_tick_compute_vm_mean,
				last_tick_compute_graph_mean: status.last_tick_compute_graph_mean,
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

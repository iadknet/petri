import { useEffect, useEffectEvent, useRef } from "react";
import type { WsClientEvent, WsConnectionStatus } from "../api/protocol.ts";
import { api } from "../api/rest.ts";
import type { WsClient } from "../api/websocket.ts";
import { wsClient } from "../api/websocket.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import type { ViewRequest } from "../stores/viewport.ts";
import { deriveViewRequest, useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import type {
	HealthPayload,
	ServerMessage,
	SnapshotResponse,
	StatusPayload,
} from "../types/api.ts";

function applyStatusToStores(
	projectionRevision: number,
	tick: number,
	status: StatusPayload,
	options?: { recordHistory?: boolean },
): void {
	const sim = useSimulationStore.getState();
	const stats = useStatsHistoryStore.getState();
	const isStaleProjection = projectionRevision < sim.projectionRevision;
	const isStaleTick = projectionRevision === sim.projectionRevision && tick < sim.tick;
	if (isStaleProjection || isStaleTick) {
		return;
	}

	sim.setStatus(projectionRevision, tick, status);
	if (options?.recordHistory === false) {
		return;
	}
	stats.pushStats(tick, status.population, status.mean_energy);
	stats.pushActions(tick, status.last_tick_actions);
	stats.pushCompute(
		tick,
		status.last_tick_compute_energy_total_mean,
		status.last_tick_compute_energy_total_min,
		status.last_tick_compute_energy_total_max,
		status.last_tick_compute_energy_vm_mean,
		status.last_tick_compute_energy_graph_mean,
	);
}

function applyHealthToStores(
	projectionRevision: number,
	tick: number,
	health: HealthPayload,
	options?: { recordHistory?: boolean },
): void {
	const sim = useSimulationStore.getState();
	const stats = useStatsHistoryStore.getState();
	const isStaleProjection = projectionRevision < sim.projectionRevision;
	const isStaleTick = projectionRevision === sim.projectionRevision && tick < sim.tick;
	if (isStaleProjection || isStaleTick) {
		return;
	}

	sim.setHealth(projectionRevision, tick, health);
	stats.setReproStats(
		health.reproduction_actions_attempted_total,
		health.reproduction_actions_spawned_total,
		health.reproduction_actions_rejected_total,
		health.reproduction_actions_rejected_total_by_reason,
	);
	stats.setMutationStats(
		health.mutation_events_attempted_total,
		health.mutation_events_applied_total,
		health.mutation_events_skipped_total,
	);
	stats.setPredationStats(
		health.predation_actions_attempted_total,
		health.predation_actions_transferred_total,
		health.predation_actions_rejected_total,
		health.predation_kills_total,
		health.predation_actions_by_result,
	);
	if (options?.recordHistory === false) {
		return;
	}
	stats.pushComplexity(
		tick,
		health.genome_complexity_mean,
		health.genome_complexity_min,
		health.genome_complexity_max,
	);
}

export function defaultRequestFromSnapshot(snapshot: SnapshotResponse): ViewRequest {
	const canvasSize = useViewportStore.getState().canvasSize;
	const fallbackCanvas =
		canvasSize.width > 0 && canvasSize.height > 0
			? canvasSize
			: {
					// Cold boot can fetch the bootstrap snapshot before the first measured
					// canvas size arrives from the viewport shell. Fall back to world-space
					// dimensions only as a temporary coarse approximation until the first
					// real viewport subscription is emitted.
					width: snapshot.world_static.width,
					height: snapshot.world_static.height,
				};

	return {
		rect: snapshot.view.rect,
		canvas: fallbackCanvas,
		zoomTier: snapshot.view.kind,
	};
}

function sameViewRequest(a: ViewRequest | null, b: ViewRequest | null): boolean {
	if (a === b) return true;
	if (!a || !b) return false;

	return (
		a.zoomTier === b.zoomTier &&
		a.canvas.width === b.canvas.width &&
		a.canvas.height === b.canvas.height &&
		a.rect.x === b.rect.x &&
		a.rect.y === b.rect.y &&
		a.rect.width === b.rect.width &&
		a.rect.height === b.rect.height
	);
}

export function applySnapshotToStores(snapshot: SnapshotResponse, requestIdFloor = 0): void {
	applyStatusToStores(snapshot.projection_revision, snapshot.tick, snapshot.status, {
		recordHistory: false,
	});
	applyHealthToStores(snapshot.projection_revision, snapshot.tick, snapshot.health, {
		recordHistory: false,
	});
	useWorldViewStore.getState().applySnapshot(snapshot, requestIdFloor);
	useViewportStore
		.getState()
		.setWorldSize(snapshot.world_static.width, snapshot.world_static.height);
}

export function applyServerMessageToStores(message: ServerMessage): void {
	switch (message.type) {
		case "status":
			applyStatusToStores(message.projection_revision, message.tick, message.payload);
			return;
		case "health":
			applyHealthToStores(message.projection_revision, message.tick, message.payload);
			return;
		case "world_static":
			useWorldViewStore.getState().applyWorldStatic({
				projectionRevision: message.projection_revision,
				worldStaticRevision: message.world_static_revision,
				tick: message.tick,
				payload: message.payload,
			});
			useViewportStore.getState().setWorldSize(message.payload.width, message.payload.height);
			return;
		case "view_overview":
			useWorldViewStore.getState().applyOverviewView({
				requestId: message.request_id,
				projectionRevision: message.projection_revision,
				worldStaticRevision: message.world_static_revision,
				tick: message.tick,
				payload: message.payload,
			});
			return;
		case "view_detail":
			useWorldViewStore.getState().applyDetailView({
				requestId: message.request_id,
				projectionRevision: message.projection_revision,
				worldStaticRevision: message.world_static_revision,
				tick: message.tick,
				payload: message.payload,
			});
			return;
	}
}

function setConnectionStatus(status: WsConnectionStatus): void {
	useSimulationStore.getState().setConnectionStatus(status);
}

export function useViewSubscription(client: WsClient = wsClient): void {
	const nextRequestIdRef = useRef(1);

	const sendViewRequest = useEffectEvent((request: ViewRequest | null, requestId?: number) => {
		if (!request) {
			client.send({ type: "unsubscribe_view" });
			return;
		}

		const nextRequestId = requestId ?? nextRequestIdRef.current;
		const didSend = client.send({
			type: "subscribe_view",
			request_id: nextRequestId,
			x: request.rect.x,
			y: request.rect.y,
			width: request.rect.width,
			height: request.rect.height,
			canvas_width: request.canvas.width,
			canvas_height: request.canvas.height,
			zoom_tier: request.zoomTier,
		});

		if (didSend) {
			nextRequestIdRef.current = Math.max(nextRequestIdRef.current, nextRequestId + 1);
		}
	});

	const bootstrapSnapshot = useEffectEvent(async () => {
		try {
			const snapshot = await api.getSnapshot();
			const requestIdFloor = nextRequestIdRef.current;
			applySnapshotToStores(snapshot, requestIdFloor);

			const request =
				useViewportStore.getState().getViewRequest() ?? defaultRequestFromSnapshot(snapshot);
			sendViewRequest(request, requestIdFloor);
		} catch {
			// Bootstrap failures are non-fatal; the next successful reconnect will retry.
		}
	});

	const handleClientEvent = useEffectEvent((event: WsClientEvent) => {
		if (event.type === "connection") {
			setConnectionStatus(event.status);
			if (event.status === "connected") {
				void bootstrapSnapshot();
			}
			return;
		}

		applyServerMessageToStores(event.message);
	});

	// biome-ignore lint/correctness/useExhaustiveDependencies: useEffectEvent returns stable callbacks by contract.
	useEffect(() => {
		const unsubscribeClient = client.subscribe(handleClientEvent);
		const unsubscribeViewport = useViewportStore.subscribe((state, prev) => {
			const nextRequest = deriveViewRequest(state);
			const previousRequest = deriveViewRequest(prev);
			if (!sameViewRequest(nextRequest, previousRequest)) {
				sendViewRequest(nextRequest);
			}
		});

		client.connect();

		return () => {
			unsubscribeViewport();
			unsubscribeClient();
			client.disconnect();
		};
	}, [client]);
}

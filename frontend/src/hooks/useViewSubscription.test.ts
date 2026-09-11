import { beforeEach, describe, expect, it } from "vitest";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import type {
	HealthPayload,
	ServerMessage,
	SnapshotResponse,
	StatusPayload,
	ViewDetailPayload,
	ViewOverviewPayload,
	WorldStaticPayload,
} from "../types/api.ts";
import {
	applyServerMessageToStores,
	applySnapshotToStores,
	defaultRequestFromSnapshot,
} from "./useViewSubscription.ts";

function buildStatusPayload(): StatusPayload {
	return {
		state: "running",
		population: 7,
		mean_energy: 15,
		last_tick_actions: { move: 2, eat: 1, reproduce: 0, noop: 0, steal: 0, predation_kills: 0 },
		reproduction_actions_attempted_total: 4,
		reproduction_actions_spawned_total: 1,
		reproduction_actions_rejected_total: 3,
		predation_actions_attempted_total: 2,
		predation_actions_transferred_total: 1,
		predation_actions_rejected_total: 1,
		predation_kills_total: 0,
		predation_actions_by_result: {},
		mutation_events_attempted_total: 3,
		mutation_events_applied_total: 2,
		mutation_events_skipped_total: 1,
		mutation_events_attempted_total_by_domain: {},
		mutation_events_applied_total_by_domain: {},
		mutation_events_attempted_total_by_operator: {},
		mutation_events_applied_total_by_operator: {},
		mutation_events_skipped_total_by_operator: {},
		mutation_target_reachability_total: {
			reachable: 2,
			unreachable: 0,
			executed: 2,
			not_applicable: 0,
		},
		move_actions_blocked_total_by_cause: {},
		reproduction_actions_rejected_invalid_target_total_by_cause: {},
		last_tick_compute_energy_total_mean: 2,
		last_tick_compute_energy_total_min: 1,
		last_tick_compute_energy_total_max: 4,
		last_tick_compute_energy_vm_mean: 1,
		last_tick_compute_energy_graph_mean: 1,
		perf: { projection_publish_ms: 1, ws_frame_publish_ms: 2, subscriber_count: 1 },
	};
}

function buildHealthPayload(): HealthPayload {
	return {
		population: 7,
		mean_energy: 15,
		mutation_events_attempted_total: 3,
		mutation_events_applied_total: 2,
		mutation_events_skipped_total: 1,
		mutation_events_attempted_total_by_domain: {},
		mutation_events_applied_total_by_domain: {},
		mutation_events_attempted_total_by_operator: {},
		mutation_events_applied_total_by_operator: {},
		mutation_events_skipped_total_by_operator: { duplicate_mutation: 1 },
		mutation_target_reachability_total: {
			reachable: 1,
			unreachable: 1,
			executed: 1,
			not_applicable: 0,
		},
		move_actions_blocked_total_by_cause: { barrier: 2 },
		reproduction_actions_rejected_invalid_target_total_by_cause: { occupied: 1 },
		reproduction_actions_attempted_total: 4,
		reproduction_actions_spawned_total: 1,
		reproduction_actions_rejected_total: 3,
		reproduction_actions_rejected_total_by_reason: {},
		mutation_events_skipped_total_by_reason: {},
		predation_actions_attempted_total: 2,
		predation_actions_transferred_total: 1,
		predation_actions_rejected_total: 1,
		predation_kills_total: 0,
		predation_actions_by_result: {},
		genome_complexity_mean: 8,
		genome_complexity_min: 7,
		genome_complexity_max: 9,
	};
}

function buildWorldStaticPayload(partial?: Partial<WorldStaticPayload>): WorldStaticPayload {
	return {
		width: 12,
		height: 10,
		barrier_mask: new Array(15).fill(0),
		food_types: [
			{
				type_idx: 0,
				name: "base",
				color: "#22c55e",
				growth_inhibitor: 0.2,
			},
		],
		...partial,
	};
}

function buildOverviewPayload(partial?: Partial<ViewOverviewPayload>): ViewOverviewPayload {
	return {
		rect: { x: 0, y: 0, width: 12, height: 10 },
		grid_width: 6,
		grid_height: 5,
		food: [],
		creature_count_u16: new Array(30).fill(0),
		...partial,
	};
}

function buildDetailPayload(partial?: Partial<ViewDetailPayload>): ViewDetailPayload {
	return {
		rect: { x: 2, y: 3, width: 4, height: 2 },
		width: 4,
		height: 2,
		food: [],
		creatures: [
			{
				id: 42,
				x: 3,
				y: 4,
				energy: 8,
				generation: 1,
				phenotype_rgb: [1, 2, 3],
			},
		],
		predation_events: [],
		...partial,
	};
}

function buildSnapshot(partial?: Partial<SnapshotResponse>): SnapshotResponse {
	return {
		protocol_version: "v3alpha2",
		projection_revision: 5,
		world_static_revision: 2,
		tick: 11,
		status: buildStatusPayload(),
		health: buildHealthPayload(),
		world_static: buildWorldStaticPayload(),
		view: { kind: "overview", ...buildOverviewPayload() },
		...partial,
	};
}

describe("useViewSubscription store helpers", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
		useStatsHistoryStore.getState().reset();
		useViewportStore.getState().reset();
		useWorldViewStore.getState().reset();
	});

	it("hydrates stores from a bootstrap snapshot", () => {
		applySnapshotToStores(buildSnapshot());

		expect(useSimulationStore.getState().tick).toBe(11);
		expect(useSimulationStore.getState().status?.population).toBe(7);
		expect(useWorldViewStore.getState().projectionRevision).toBe(5);
		expect(useViewportStore.getState().worldSize).toEqual({ width: 12, height: 10 });
		expect(useStatsHistoryStore.getState().statsHistory).toEqual([]);
		expect(useStatsHistoryStore.getState().reproAttempted).toBe(4);
		expect(useStatsHistoryStore.getState().mutationSkippedByOperator.duplicate_mutation).toBe(1);
		expect(useStatsHistoryStore.getState().mutationTargetReachabilityTotal.unreachable).toBe(1);
	});

	it("uses measured canvas size for the bootstrap fallback request when available", () => {
		useViewportStore.getState().setCanvasSize(1920, 1080);

		const request = defaultRequestFromSnapshot(buildSnapshot());

		expect(request.canvas).toEqual({ width: 1920, height: 1080 });
		expect(request.rect).toEqual({ x: 0, y: 0, width: 12, height: 10 });
		expect(request.zoomTier).toBe("overview");
	});

	it("routes typed view_detail events into the world view store", () => {
		applySnapshotToStores(buildSnapshot());

		const message: ServerMessage = {
			type: "view_detail",
			protocol_version: "v3alpha2",
			request_id: 12,
			projection_revision: 6,
			world_static_revision: 2,
			tick: 12,
			payload: buildDetailPayload(),
		};

		applyServerMessageToStores(message);

		const currentView = useWorldViewStore.getState().currentView;
		expect(currentView?.kind).toBe("detail");
		expect(currentView?.requestId).toBe(12);
		if (!currentView || currentView.kind !== "detail") {
			throw new Error("expected detail view");
		}
		expect(currentView.payload.creatures[0]?.id).toBe(42);
	});

	it("ignores stale status events from an older projection after a restart snapshot resets tick", () => {
		applySnapshotToStores(buildSnapshot({ projection_revision: 7, tick: 25 }));
		applySnapshotToStores(
			buildSnapshot({
				projection_revision: 8,
				tick: 0,
				status: { ...buildStatusPayload(), state: "idle", population: 0, mean_energy: 0 },
				health: { ...buildHealthPayload(), population: 0, mean_energy: 0 },
			}),
		);

		applyServerMessageToStores({
			type: "status",
			protocol_version: "v3alpha2",
			projection_revision: 7,
			world_static_revision: 2,
			tick: 26,
			payload: buildStatusPayload(),
		});

		const simulation = useSimulationStore.getState();
		expect(simulation.projectionRevision).toBe(8);
		expect(simulation.tick).toBe(0);
		expect(simulation.simState).toBe("idle");
	});
});

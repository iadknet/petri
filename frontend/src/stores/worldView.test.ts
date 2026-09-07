import { beforeEach, describe, expect, it } from "vitest";
import type {
	HealthPayload,
	SnapshotResponse,
	StatusPayload,
	ViewDetailPayload,
	ViewOverviewPayload,
	WorldStaticPayload,
} from "../types/api.ts";
import { useWorldViewStore } from "./worldView.ts";

function buildStatusPayload(): StatusPayload {
	return {
		state: "running",
		population: 12,
		mean_energy: 42,
		last_tick_actions: { move: 3, eat: 4, reproduce: 1, noop: 2, steal: 0, predation_kills: 0 },
		reproduction_actions_attempted_total: 10,
		reproduction_actions_spawned_total: 4,
		reproduction_actions_rejected_total: 6,
		predation_actions_attempted_total: 3,
		predation_actions_transferred_total: 2,
		predation_actions_rejected_total: 1,
		predation_kills_total: 1,
		predation_actions_by_result: { transferred: 2, rejected: 1 },
		mutation_events_attempted_total: 5,
		mutation_events_applied_total: 4,
		mutation_events_skipped_total: 1,
		mutation_events_attempted_total_by_domain: { genome: 5 },
		mutation_events_applied_total_by_domain: { genome: 4 },
		mutation_events_attempted_total_by_operator: { insert_node: 5 },
		mutation_events_applied_total_by_operator: { insert_node: 4 },
		mutation_events_applied_total_semantic_noop: 1,
		mutation_events_applied_total_semantic_change: 3,
		last_tick_compute_energy_total_mean: 7,
		last_tick_compute_energy_total_min: 4,
		last_tick_compute_energy_total_max: 9,
		last_tick_compute_energy_vm_mean: 3,
		last_tick_compute_energy_graph_mean: 4,
		perf: {
			projection_publish_ms: 1.2,
			ws_frame_publish_ms: 2.4,
			subscriber_count: 1,
		},
	};
}

function buildHealthPayload(): HealthPayload {
	return {
		population: 12,
		mean_energy: 42,
		mutation_events_attempted_total: 5,
		mutation_events_applied_total: 4,
		mutation_events_skipped_total: 1,
		mutation_events_attempted_total_by_domain: { genome: 5 },
		mutation_events_applied_total_by_domain: { genome: 4 },
		mutation_events_attempted_total_by_operator: { insert_node: 5 },
		mutation_events_applied_total_by_operator: { insert_node: 4 },
		mutation_events_applied_total_semantic_noop: 1,
		mutation_events_applied_total_semantic_change: 3,
		reproduction_actions_attempted_total: 10,
		reproduction_actions_spawned_total: 4,
		reproduction_actions_rejected_total: 6,
		reproduction_actions_rejected_total_by_reason: { blocked: 6 },
		mutation_events_skipped_total_by_reason: { duplicate: 1 },
		predation_actions_attempted_total: 3,
		predation_actions_transferred_total: 2,
		predation_actions_rejected_total: 1,
		predation_kills_total: 1,
		predation_actions_by_result: { transferred: 2, rejected: 1 },
		genome_complexity_mean: 11,
		genome_complexity_min: 8,
		genome_complexity_max: 15,
	};
}

function buildWorldStaticPayload(partial?: Partial<WorldStaticPayload>): WorldStaticPayload {
	return {
		width: 8,
		height: 6,
		barrier_mask: [0, 1, 2, 3, 4, 5],
		food_types: [
			{
				type_idx: 0,
				name: "Primary Food",
				color: "#22c55e",
				growth_inhibitor: 0.2,
			},
			{
				type_idx: 1,
				name: "Secondary Food",
				color: "#0ea5e9",
				growth_inhibitor: 0.3,
			},
		],
		...partial,
	};
}

function buildOverviewPayload(partial?: Partial<ViewOverviewPayload>): ViewOverviewPayload {
	return {
		rect: { x: 0, y: 0, width: 8, height: 6 },
		grid_width: 4,
		grid_height: 3,
		food: [],
		creature_count_u16: new Array(12).fill(0),
		...partial,
	};
}

function buildDetailPayload(partial?: Partial<ViewDetailPayload>): ViewDetailPayload {
	return {
		rect: { x: 2, y: 1, width: 4, height: 3 },
		width: 4,
		height: 3,
		food: [],
		creatures: [
			{
				id: 77,
				x: 3,
				y: 2,
				energy: 9,
				generation: 1,
				phenotype_rgb: [1, 2, 3],
			},
		],
		predation_events: [
			{ attacker_x: 3, attacker_y: 2, victim_x: 4, victim_y: 2, energy_stolen: 1, killed: false },
		],
		...partial,
	};
}

function buildSnapshot(partial?: Partial<SnapshotResponse>): SnapshotResponse {
	return {
		protocol_version: "v3alpha2",
		projection_revision: 5,
		world_static_revision: 2,
		tick: 19,
		status: buildStatusPayload(),
		health: buildHealthPayload(),
		world_static: buildWorldStaticPayload(),
		view: { kind: "overview", ...buildOverviewPayload() },
		...partial,
	};
}

describe("WorldViewStore", () => {
	beforeEach(() => {
		useWorldViewStore.getState().reset();
	});

	it("hydrates from snapshots and rejects stale projection revisions", () => {
		useWorldViewStore.getState().applySnapshot(buildSnapshot());
		expect(useWorldViewStore.getState().projectionRevision).toBe(5);
		expect(useWorldViewStore.getState().worldStaticRevision).toBe(2);
		expect(useWorldViewStore.getState().currentView?.kind).toBe("overview");

		useWorldViewStore.getState().applySnapshot(
			buildSnapshot({
				projection_revision: 4,
				world_static_revision: 9,
				view: { kind: "detail", ...buildDetailPayload() },
			}),
		);

		expect(useWorldViewStore.getState().projectionRevision).toBe(5);
		expect(useWorldViewStore.getState().worldStaticRevision).toBe(2);
		expect(useWorldViewStore.getState().currentView?.kind).toBe("overview");
	});

	it("rejects stale request ids and older static revisions", () => {
		const store = useWorldViewStore.getState();
		store.applySnapshot(buildSnapshot());

		store.applyDetailView({
			requestId: 11,
			projectionRevision: 6,
			worldStaticRevision: 2,
			tick: 20,
			payload: buildDetailPayload(),
		});

		store.applyDetailView({
			requestId: 10,
			projectionRevision: 7,
			worldStaticRevision: 2,
			tick: 21,
			payload: buildDetailPayload({
				creatures: [
					{
						id: 99,
						x: 4,
						y: 2,
						energy: 3,
						generation: 8,
						phenotype_rgb: [9, 9, 9],
					},
				],
			}),
		});

		store.applyWorldStatic({
			projectionRevision: 8,
			worldStaticRevision: 1,
			tick: 22,
			payload: buildWorldStaticPayload({ barrier_mask: [9, 9, 9] }),
		});

		const current = useWorldViewStore.getState();
		expect(current.currentView?.requestId).toBe(11);
		expect(current.currentView?.kind).toBe("detail");
		if (!current.currentView || current.currentView.kind !== "detail") {
			throw new Error("expected detail view");
		}
		expect(current.currentView.payload.creatures[0]?.id).toBe(77);
		expect(current.worldStatic?.barrier_mask).toEqual([0, 1, 2, 3, 4, 5]);
	});

	it("uses the snapshot request floor to reject delayed views from an older websocket session", () => {
		const store = useWorldViewStore.getState();
		store.applySnapshot(buildSnapshot({ projection_revision: 10 }), 12);

		store.applyDetailView({
			requestId: 11,
			projectionRevision: 10,
			worldStaticRevision: 2,
			tick: 20,
			payload: buildDetailPayload(),
		});

		const current = useWorldViewStore.getState();
		expect(current.currentView?.kind).toBe("overview");
		expect(current.currentView?.requestId).toBe(12);
	});

	it("tracks the latest worldStaticRevision from incoming view updates", () => {
		const store = useWorldViewStore.getState();
		store.applySnapshot(buildSnapshot({ world_static_revision: 2 }), 4);

		store.applyDetailView({
			requestId: 4,
			projectionRevision: 6,
			worldStaticRevision: 3,
			tick: 20,
			payload: buildDetailPayload(),
		});

		expect(useWorldViewStore.getState().worldStaticRevision).toBe(3);
	});

	it("maps detail payload food cells into frame food preserving typed density entries", () => {
		const store = useWorldViewStore.getState();
		store.applySnapshot(buildSnapshot());
		store.applyDetailView({
			requestId: 12,
			projectionRevision: 6,
			worldStaticRevision: 2,
			tick: 20,
			payload: buildDetailPayload({
				food: [
					{ x: 3, y: 2, type_idx: 0, density: 0.6 },
					{ x: 3, y: 2, type_idx: 1, density: 0.2 },
					{ x: 4, y: 2, type_idx: 99, density: 0.4 },
				],
			}),
		});

		expect(useWorldViewStore.getState().frame?.food).toEqual([
			{ x: 3, y: 2, type_idx: 0, density: 0.6 },
			{ x: 3, y: 2, type_idx: 1, density: 0.2 },
			{ x: 4, y: 2, type_idx: 99, density: 0.4 },
		]);
	});
});

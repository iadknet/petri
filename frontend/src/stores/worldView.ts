import { create } from "zustand";
import type {
	Frame,
	PredationEvent,
	SnapshotResponse,
	ViewDetailPayload,
	ViewOverviewPayload,
	WorldStaticPayload,
} from "../types/api.ts";

interface WorldStaticEvent {
	projectionRevision: number;
	worldStaticRevision: number;
	tick: number;
	payload: WorldStaticPayload;
}

interface ViewEventBase<TPayload> {
	requestId: number;
	projectionRevision: number;
	worldStaticRevision: number;
	tick: number;
	payload: TPayload;
}

interface StoredViewBase<TKind extends "overview" | "detail", TPayload> {
	kind: TKind;
	requestId: number;
	projectionRevision: number;
	worldStaticRevision: number;
	tick: number;
	payload: TPayload;
}

type StoredOverviewView = StoredViewBase<"overview", ViewOverviewPayload>;
type StoredDetailView = StoredViewBase<"detail", ViewDetailPayload>;
type StoredView = StoredOverviewView | StoredDetailView;

export interface WorldViewState {
	projectionRevision: number;
	worldStaticRevision: number;
	worldStatic: WorldStaticPayload | null;
	currentView: StoredView | null;
	frame: Frame | null;
	predationEvents: PredationEvent[];
	applySnapshot: (snapshot: SnapshotResponse, requestIdFloor?: number) => void;
	applyWorldStatic: (event: WorldStaticEvent) => void;
	applyOverviewView: (event: ViewEventBase<ViewOverviewPayload>) => void;
	applyDetailView: (event: ViewEventBase<ViewDetailPayload>) => void;
	setPredationEvents: (events: PredationEvent[]) => void;
	reset: () => void;
}

const initialState = {
	projectionRevision: 0,
	worldStaticRevision: 0,
	worldStatic: null as WorldStaticPayload | null,
	currentView: null as StoredView | null,
	frame: null as Frame | null,
	predationEvents: [] as PredationEvent[],
};

function asNumberArray(values: Uint8Array | number[]): number[] {
	return Array.from(values);
}

function decodeBarrierMask(payload: WorldStaticPayload): Frame["barriers"] {
	const barriers: Frame["barriers"] = [];
	const bytes = asNumberArray(payload.barrier_mask);
	const cellCount = payload.width * payload.height;

	for (let index = 0; index < cellCount; index++) {
		const byte = bytes[Math.floor(index / 8)] ?? 0;
		const mask = 1 << (index % 8);
		if ((byte & mask) === 0) continue;
		barriers.push({
			x: index % payload.width,
			y: Math.floor(index / payload.width),
		});
	}

	return barriers;
}

function buildVisibleFoodCells(payload: ViewDetailPayload): Frame["food"] {
	const food: Frame["food"] = [];
	const densities = asNumberArray(payload.food_density_u8);

	for (let index = 0; index < densities.length; index++) {
		const density = densities[index] ?? 0;
		if (density <= 0) continue;

		food.push({
			x: payload.rect.x + (index % payload.width),
			y: payload.rect.y + Math.floor(index / payload.width),
			density: density / 255,
		});
	}

	return food;
}

function buildLegacyFrame(
	worldStatic: WorldStaticPayload | null,
	view: StoredView | null,
): Frame | null {
	if (!worldStatic) return null;

	return {
		width: worldStatic.width,
		height: worldStatic.height,
		barriers: decodeBarrierMask(worldStatic),
		creatures: view?.kind === "detail" ? view.payload.creatures : [],
		food: view?.kind === "detail" ? buildVisibleFoodCells(view.payload) : [],
	};
}

function applyViewUpdate(state: WorldViewState, nextView: StoredView): Partial<WorldViewState> {
	if (nextView.projectionRevision < state.projectionRevision) {
		return {};
	}

	if (state.currentView && nextView.requestId < state.currentView.requestId) {
		return {};
	}

	const worldStaticRevision = Math.max(state.worldStaticRevision, nextView.worldStaticRevision);

	return {
		projectionRevision: nextView.projectionRevision,
		worldStaticRevision,
		currentView: {
			...nextView,
			worldStaticRevision,
		},
		frame: buildLegacyFrame(state.worldStatic, nextView),
		predationEvents: nextView.kind === "detail" ? nextView.payload.predation_events : [],
	};
}

export const useWorldViewStore = create<WorldViewState>()((set) => ({
	...initialState,

	applySnapshot: (snapshot, requestIdFloor = 0) =>
		set((state) => {
			if (snapshot.projection_revision < state.projectionRevision) {
				return state;
			}

			const worldStatic =
				snapshot.world_static_revision >= state.worldStaticRevision
					? snapshot.world_static
					: state.worldStatic;
			const worldStaticRevision =
				worldStatic === state.worldStatic
					? state.worldStaticRevision
					: snapshot.world_static_revision;

			// Snapshots borrow the next live request id as a floor so any delayed
			// view message from a previous websocket session cannot supersede the
			// freshly bootstrapped snapshot before the current subscription replies.
			const currentView: StoredView =
				snapshot.view.kind === "overview"
					? {
							kind: "overview",
							requestId: requestIdFloor,
							projectionRevision: snapshot.projection_revision,
							worldStaticRevision: snapshot.world_static_revision,
							tick: snapshot.tick,
							payload: snapshot.view,
						}
					: {
							kind: "detail",
							requestId: requestIdFloor,
							projectionRevision: snapshot.projection_revision,
							worldStaticRevision: snapshot.world_static_revision,
							tick: snapshot.tick,
							payload: snapshot.view,
						};

			return {
				projectionRevision: snapshot.projection_revision,
				worldStaticRevision,
				worldStatic,
				currentView,
				frame: buildLegacyFrame(worldStatic, currentView),
				predationEvents: snapshot.view.kind === "detail" ? snapshot.view.predation_events : [],
			};
		}),

	applyWorldStatic: (event) =>
		set((state) => {
			if (event.projectionRevision < state.projectionRevision) {
				return state;
			}
			if (event.worldStaticRevision < state.worldStaticRevision) {
				return state;
			}

			return {
				projectionRevision: event.projectionRevision,
				worldStaticRevision: event.worldStaticRevision,
				worldStatic: event.payload,
				frame: buildLegacyFrame(event.payload, state.currentView),
			};
		}),

	applyOverviewView: (event) =>
		set((state) =>
			applyViewUpdate(state, {
				kind: "overview",
				requestId: event.requestId,
				projectionRevision: event.projectionRevision,
				worldStaticRevision: event.worldStaticRevision,
				tick: event.tick,
				payload: event.payload,
			}),
		),

	applyDetailView: (event) =>
		set((state) =>
			applyViewUpdate(state, {
				kind: "detail",
				requestId: event.requestId,
				projectionRevision: event.projectionRevision,
				worldStaticRevision: event.worldStaticRevision,
				tick: event.tick,
				payload: event.payload,
			}),
		),

	setPredationEvents: (predationEvents) => set({ predationEvents }),
	reset: () => set(initialState),
}));

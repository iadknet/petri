import { create } from "zustand";
import { buildFrameFromTransport } from "../adapters/worldFrame.ts";
import type { PredationEventSnapshot } from "../types/generated/PredationEventSnapshot.ts";
import type { SnapshotResponse } from "../types/generated/SnapshotResponse.ts";
import type { ViewDetailPayload } from "../types/generated/ViewDetailPayload.ts";
import type { ViewOverviewPayload } from "../types/generated/ViewOverviewPayload.ts";
import type { WorldStaticPayload } from "../types/generated/WorldStaticPayload.ts";
import type { Frame } from "../types/protocol.ts";

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
	predationEvents: PredationEventSnapshot[];
	applySnapshot: (snapshot: SnapshotResponse, requestIdFloor?: number) => void;
	applyWorldStatic: (event: WorldStaticEvent) => void;
	applyOverviewView: (event: ViewEventBase<ViewOverviewPayload>) => void;
	applyDetailView: (event: ViewEventBase<ViewDetailPayload>) => void;
	setPredationEvents: (events: PredationEventSnapshot[]) => void;
	reset: () => void;
}

const initialState = {
	projectionRevision: 0,
	worldStaticRevision: 0,
	worldStatic: null as WorldStaticPayload | null,
	currentView: null as StoredView | null,
	frame: null as Frame | null,
	predationEvents: [] as PredationEventSnapshot[],
};

function buildLegacyFrame(
	worldStatic: WorldStaticPayload | null,
	view: StoredView | null,
): Frame | null {
	return buildFrameFromTransport(worldStatic, view?.kind === "detail" ? view.payload : null);
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

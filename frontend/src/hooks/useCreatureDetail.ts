import { useEffect, useRef } from "react";
import { ApiRequestError, api } from "../api/rest.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";

export function useCreatureDetail() {
	const abortRef = useRef<AbortController | null>(null);
	const currentIdRef = useRef<number | null>(null);
	const lastFetchRef = useRef<number>(0);
	/** Tracks the latest_tick from the most recent response for incremental fetching. */
	const latestTickRef = useRef<number | null>(null);
	/** Whether we have completed at least one full (non-incremental) fetch. */
	const hasFullFetchRef = useRef<boolean>(false);
	const MIN_FETCH_INTERVAL = 100; // ms — cap at ~10Hz

	// Sync the ref with the store value
	useEffect(() => {
		// Initialize with current value (subscribe only fires on future changes)
		currentIdRef.current = useCreatureInspectorStore.getState().selectedCreatureId;
		return useCreatureInspectorStore.subscribe((state) => {
			currentIdRef.current = state.selectedCreatureId;
		});
	}, []);

	// Fetch creature detail
	useEffect(() => {
		const fetchCreature = async (id: number, incremental: boolean) => {
			// Cancel any in-flight request
			abortRef.current?.abort();
			const controller = new AbortController();
			abortRef.current = controller;

			// Build query params for incremental fetch.
			const query: { since_tick?: number; exclude?: string } = {};
			if (incremental && hasFullFetchRef.current && latestTickRef.current !== null) {
				query.since_tick = latestTickRef.current;
				query.exclude = "genome";
			}

			try {
				const detail = await api.getCreature(id, controller.signal, query);
				// Only apply if this ID is still selected
				if (currentIdRef.current !== id) return;

				// Update latest_tick cursor for next incremental fetch.
				latestTickRef.current = detail.latest_tick;

				useCreatureInspectorStore.getState().setDetail({
					id: detail.id,
					position: detail.position,
					energy: detail.energy,
					maxEnergy: detail.max_energy,
					age: detail.age,
					generation: detail.generation,
					complexity: detail.complexity,
					phenotype: detail.phenotype,
					genome: detail.genome,
					sharedMemory: detail.shared_memory,
					actionLog: detail.action_log,
					incremental: incremental && hasFullFetchRef.current,
				});

				// After first successful full fetch, enable incremental mode.
				if (!hasFullFetchRef.current) {
					hasFullFetchRef.current = true;
				}
			} catch (err) {
				if (controller.signal.aborted) return;
				if (currentIdRef.current !== id) return;
				if (err instanceof ApiRequestError && err.status === 404) {
					useCreatureInspectorStore.getState().setDead();
				} else {
					useCreatureInspectorStore
						.getState()
						.setError(err instanceof Error ? err.message : "failed to fetch creature");
				}
			}
		};

		// Reset incremental state when creature changes.
		const resetIncrementalState = () => {
			latestTickRef.current = null;
			hasFullFetchRef.current = false;
		};

		// Fetch on selectedCreatureId change
		const unsubInspector = useCreatureInspectorStore.subscribe((state, prev) => {
			if (
				state.selectedCreatureId !== prev.selectedCreatureId &&
				state.selectedCreatureId !== null
			) {
				resetIncrementalState();
				fetchCreature(state.selectedCreatureId, false);
			}
		});

		// Re-fetch on tick change (live updates), throttled to ~10Hz
		const unsubSim = useSimulationStore.subscribe((state, prev) => {
			if (state.tick !== prev.tick && currentIdRef.current !== null) {
				const now = Date.now();
				if (now - lastFetchRef.current >= MIN_FETCH_INTERVAL) {
					lastFetchRef.current = now;
					fetchCreature(currentIdRef.current, true);
				}
			}
		});

		// Initial fetch if already selected
		const id = useCreatureInspectorStore.getState().selectedCreatureId;
		if (id !== null) {
			resetIncrementalState();
			fetchCreature(id, false);
		}

		return () => {
			unsubInspector();
			unsubSim();
			abortRef.current?.abort();
		};
	}, []);
}

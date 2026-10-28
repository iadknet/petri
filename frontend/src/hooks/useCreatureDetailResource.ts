import { useEffect, useRef } from "react";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";
export function useCreatureDetailResource() {
	// The request in flight, cleared when it settles. A tick never replaces
	// it: with responses slower than ticks, every poll would abort its
	// predecessor and none would land.
	const abortRef = useRef<AbortController | null>(null);
	const currentIdRef = useRef<number | null>(null);
	const lastFetchRef = useRef<number>(0);
	const latestTickRef = useRef<number | null>(null);
	const hasFullFetchRef = useRef(false);
	// The held brain resolves under another context than the latest
	// poll's: every fetch carries the brain until one lands.
	const brainStaleRef = useRef(false);
	const MIN_FETCH_INTERVAL = 100;

	useEffect(() => {
		currentIdRef.current = creatureInspectorSelectors.selectedCreatureId(
			useCreatureInspectorStore.getState(),
		);
		return useCreatureInspectorStore.subscribe((state) => {
			currentIdRef.current = creatureInspectorSelectors.selectedCreatureId(state);
		});
	}, []);

	useEffect(() => {
		const fetchCreature = async (id: number, incremental: boolean) => {
			abortRef.current?.abort();
			const controller = new AbortController();
			abortRef.current = controller;

			const query: { since_tick?: number; exclude?: string } = {};
			if (incremental && hasFullFetchRef.current && latestTickRef.current !== null) {
				query.since_tick = latestTickRef.current;
				// `learning` carries the weights in force, which move every
				// tick, so polls exclude the brain only.
				if (!brainStaleRef.current) query.exclude = "brain";
			}

			try {
				const detail = await api.getCreature(id, controller.signal, query);
				if (currentIdRef.current !== id) return;

				latestTickRef.current = detail.latest_tick;

				useCreatureInspectorStore.getState().setDetail({
					id: detail.id,
					position: detail.position,
					energy: detail.energy,
					maxEnergy: detail.max_energy,
					age: detail.age,
					generation: detail.generation,
					heading: detail.heading,
					complexity: detail.complexity,
					genomeSize: detail.genome_size,
					phenotype: detail.phenotype,
					brain: detail.brain,
					brainContext: detail.brain_context,
					learning: detail.learning,
					sharedMemory: detail.shared_memory,
					actionLog: detail.action_log,
					diagnostics: detail.diagnostics,
					incremental: incremental && hasFullFetchRef.current,
				});

				if (!hasFullFetchRef.current) {
					hasFullFetchRef.current = true;
				}

				// The held brain resolves under another context identity:
				// refetch it under the current one.
				const held = useCreatureInspectorStore.getState().staticDetail.brainContext;
				brainStaleRef.current = held !== null && held !== detail.brain_context;
				if (brainStaleRef.current) fetchCreature(id, true);
			} catch (err) {
				if (controller.signal.aborted || currentIdRef.current !== id) return;
				if (err instanceof ApiRequestError && err.status === 404) {
					useCreatureInspectorStore.getState().setDead();
					return;
				}
				useCreatureInspectorStore
					.getState()
					.setError(err instanceof Error ? err.message : "failed to fetch creature");
			} finally {
				if (abortRef.current === controller) abortRef.current = null;
			}
		};

		const resetIncrementalState = () => {
			latestTickRef.current = null;
			hasFullFetchRef.current = false;
			brainStaleRef.current = false;
		};

		const unsubInspector = useCreatureInspectorStore.subscribe((state, prev) => {
			const selectedCreatureId = creatureInspectorSelectors.selectedCreatureId(state);
			const prevSelectedCreatureId = creatureInspectorSelectors.selectedCreatureId(prev);
			if (selectedCreatureId !== prevSelectedCreatureId && selectedCreatureId !== null) {
				resetIncrementalState();
				fetchCreature(selectedCreatureId, false);
			}
		});

		const unsubSim = useSimulationStore.subscribe((state, prev) => {
			if (
				state.tick !== prev.tick &&
				currentIdRef.current !== null &&
				abortRef.current === null &&
				!creatureInspectorSelectors.isDead(useCreatureInspectorStore.getState())
			) {
				const now = Date.now();
				if (now - lastFetchRef.current >= MIN_FETCH_INTERVAL) {
					lastFetchRef.current = now;
					fetchCreature(currentIdRef.current, true);
				}
			}
		});

		// A committed config change may move the brain context with no tick
		// while paused: poll, replacing a request sent under the old config,
		// and let the poll's context identity decide the brain refetch.
		const unsubConfig = useConfigStore.subscribe((state, prev) => {
			if (
				prev.serverConfig !== null &&
				state.serverConfig !== prev.serverConfig &&
				currentIdRef.current !== null &&
				!creatureInspectorSelectors.isDead(useCreatureInspectorStore.getState())
			) {
				fetchCreature(currentIdRef.current, true);
			}
		});

		const selectedCreatureId = creatureInspectorSelectors.selectedCreatureId(
			useCreatureInspectorStore.getState(),
		);
		if (selectedCreatureId !== null) {
			resetIncrementalState();
			fetchCreature(selectedCreatureId, false);
		}

		return () => {
			unsubInspector();
			unsubSim();
			unsubConfig();
			abortRef.current?.abort();
		};
	}, []);
}

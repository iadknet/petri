import { useEffect, useRef } from "react";
import { ApiRequestError, api } from "../api/rest.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";

export function useCreatureDetailResource() {
	const abortRef = useRef<AbortController | null>(null);
	const currentIdRef = useRef<number | null>(null);
	const lastFetchRef = useRef<number>(0);
	const latestTickRef = useRef<number | null>(null);
	const hasFullFetchRef = useRef(false);
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
				query.exclude = "genome";
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
					reproductiveReserve: detail.reproductive_reserve,
					reproductiveReserveCapacity: detail.reproductive_reserve_capacity,
					age: detail.age,
					generation: detail.generation,
					complexity: detail.complexity,
					genomeSize: detail.genome_size,
					phenotype: detail.phenotype,
					genome: detail.genome,
					meshAnnotations: detail.mesh_annotations,
					sharedMemory: detail.shared_memory,
					actionLog: detail.action_log,
					diagnostics: detail.diagnostics,
					incremental: incremental && hasFullFetchRef.current,
				});

				if (!hasFullFetchRef.current) {
					hasFullFetchRef.current = true;
				}
			} catch (err) {
				if (controller.signal.aborted || currentIdRef.current !== id) return;
				if (err instanceof ApiRequestError && err.status === 404) {
					useCreatureInspectorStore.getState().setDead();
					return;
				}
				useCreatureInspectorStore
					.getState()
					.setError(err instanceof Error ? err.message : "failed to fetch creature");
			}
		};

		const resetIncrementalState = () => {
			latestTickRef.current = null;
			hasFullFetchRef.current = false;
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
				!creatureInspectorSelectors.isDead(useCreatureInspectorStore.getState())
			) {
				const now = Date.now();
				if (now - lastFetchRef.current >= MIN_FETCH_INTERVAL) {
					lastFetchRef.current = now;
					fetchCreature(currentIdRef.current, true);
				}
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
			abortRef.current?.abort();
		};
	}, []);
}

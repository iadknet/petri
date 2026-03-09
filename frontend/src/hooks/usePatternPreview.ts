import { type RefObject, useCallback, useEffect, useRef } from "react";
import { api } from "../api/rest.ts";
import type { WorldRenderer } from "../canvas/renderer.ts";
import { usePaintStore } from "../stores/paint.ts";
import { usePatternStore } from "../stores/pattern.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { decodeBitmap } from "../utils/bitmapDecode.ts";
import { buildSnapshotQuery } from "./usePaintInteraction.ts";
import { applySnapshotToStores } from "./useViewSubscription.ts";

const DEBOUNCE_MS = 300;

export interface PatternPreviewHandlers {
	applyPattern: () => void;
	cancelPattern: () => void;
	isPreviewLoading: boolean;
}

export function usePatternPreview(
	rendererRef: RefObject<WorldRenderer | null>,
): PatternPreviewHandlers {
	const abortRef = useRef<AbortController | null>(null);
	const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const previewCellsRef = useRef<Set<string> | null>(null);
	const isLoadingRef = useRef(false);

	const clearDebounce = useCallback(() => {
		if (debounceRef.current !== null) {
			clearTimeout(debounceRef.current);
			debounceRef.current = null;
		}
	}, []);

	const abortInFlight = useCallback(() => {
		if (abortRef.current) {
			abortRef.current.abort();
			abortRef.current = null;
		}
	}, []);

	const clearPreview = useCallback(() => {
		clearDebounce();
		abortInFlight();
		previewCellsRef.current = null;
		rendererRef.current?.setPreview(null, null);
		rendererRef.current?.invalidate();
		isLoadingRef.current = false;
	}, [rendererRef, clearDebounce, abortInFlight]);

	const requestPreview = useCallback(() => {
		const { areaBounds, patternParams, patternSeed } = usePatternStore.getState();
		if (!areaBounds) {
			clearPreview();
			return;
		}

		// Abort any in-flight request
		abortInFlight();

		const controller = new AbortController();
		abortRef.current = controller;
		isLoadingRef.current = true;

		api.patternPreview(
			{
				params: patternParams,
				bounds: areaBounds,
				seed: patternSeed,
			},
			controller.signal,
		)
			.then((response) => {
				if (controller.signal.aborted) return;
				const cells = decodeBitmap(response.bitmap, response.bounds);
				previewCellsRef.current = cells;
				rendererRef.current?.setPreview(cells, "barrier");
				rendererRef.current?.invalidate();
				isLoadingRef.current = false;
			})
			.catch((err) => {
				if (err instanceof DOMException && err.name === "AbortError") return;
				console.error("[PatternPreview] Preview error:", err);
				isLoadingRef.current = false;
			});
	}, [rendererRef, clearPreview, abortInFlight]);

	const schedulePreview = useCallback(() => {
		clearDebounce();
		debounceRef.current = setTimeout(() => {
			debounceRef.current = null;
			requestPreview();
		}, DEBOUNCE_MS);
	}, [clearDebounce, requestPreview]);

	// Subscribe to pattern store changes to trigger preview
	useEffect(() => {
		return usePatternStore.subscribe((state, prev) => {
			// Only react when in pattern mode
			const paintState = usePaintStore.getState();
			if (!paintState.paintMode || paintState.mode !== "pattern") return;

			// Check if anything preview-relevant changed
			const boundsChanged = state.areaBounds !== prev.areaBounds;
			const paramsChanged = state.patternParams !== prev.patternParams;
			const seedChanged = state.patternSeed !== prev.patternSeed;

			if (boundsChanged && state.areaBounds === null) {
				clearPreview();
				return;
			}

			if (boundsChanged || paramsChanged || seedChanged) {
				if (state.areaBounds) {
					schedulePreview();
				}
			}
		});
	}, [schedulePreview, clearPreview]);

	// Clean up on paint mode exit or pattern mode deselection
	useEffect(() => {
		return usePaintStore.subscribe((state, prev) => {
			const exitedPaint = prev.paintMode && !state.paintMode;
			const exitedPattern = prev.mode === "pattern" && state.mode !== "pattern";
			if (exitedPaint || exitedPattern) {
				clearPreview();
			}
		});
	}, [clearPreview]);

	// Cleanup on unmount
	useEffect(() => {
		return () => {
			clearDebounce();
			abortInFlight();
		};
	}, [clearDebounce, abortInFlight]);

	const applyPattern = useCallback(async () => {
		const { areaBounds, patternParams, patternSeed } = usePatternStore.getState();
		if (!areaBounds) return;

		clearDebounce();
		abortInFlight();

		try {
			await api.patternApply({
				params: patternParams,
				bounds: areaBounds,
				seed: patternSeed,
			});

			// Refresh world state
			const snapshot = await api.getSnapshot(
				buildSnapshotQuery(useViewportStore.getState().getViewRequest()),
			);
			applySnapshotToStores(snapshot);
			rendererRef.current?.invalidate();
		} catch (err) {
			console.error("[PatternPreview] Apply error:", err);
		}

		// Clear preview and selection
		previewCellsRef.current = null;
		rendererRef.current?.setPreview(null, null);
		usePatternStore.getState().clearPattern();

		// Clear the selection overlay
		const canvas = document.querySelector<HTMLCanvasElement>(
			'[data-testid="pattern-selection-overlay"]',
		);
		if (canvas) {
			const ctx = canvas.getContext("2d");
			ctx?.clearRect(0, 0, canvas.width, canvas.height);
		}
	}, [rendererRef, clearDebounce, abortInFlight]);

	const cancelPattern = useCallback(() => {
		clearPreview();
		usePatternStore.getState().clearPattern();

		// Clear the selection overlay
		const canvas = document.querySelector<HTMLCanvasElement>(
			'[data-testid="pattern-selection-overlay"]',
		);
		if (canvas) {
			const ctx = canvas.getContext("2d");
			ctx?.clearRect(0, 0, canvas.width, canvas.height);
		}
	}, [clearPreview]);

	return {
		applyPattern,
		cancelPattern,
		isPreviewLoading: isLoadingRef.current,
	};
}

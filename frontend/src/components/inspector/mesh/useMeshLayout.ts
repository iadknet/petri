import { startTransition, useEffect, useRef, useState } from "react";
import type { MeshAnalysis } from "./meshAnalysis.ts";
import { type MeshLayout, layoutMesh } from "./meshLayout.ts";

interface MeshLayoutState {
	layout: MeshLayout | null;
	error: string | null;
}

export function useMeshLayout(analysis: MeshAnalysis | null): MeshLayoutState {
	const [state, setState] = useState<MeshLayoutState>({
		layout: null,
		error: null,
	});
	const analysisRef = useRef<MeshAnalysis | null>(analysis);
	analysisRef.current = analysis;

	// biome-ignore lint/correctness/useExhaustiveDependencies: layout recomputes only when topology changes.
	useEffect(() => {
		const currentAnalysis = analysisRef.current;
		if (!currentAnalysis) {
			startTransition(() => {
				setState({
					layout: null,
					error: null,
				});
			});
			return undefined;
		}

		let cancelled = false;
		startTransition(() => {
			setState((current) => ({
				layout: current.layout?.topologyKey === currentAnalysis.topologyKey ? current.layout : null,
				error: null,
			}));
		});

		void layoutMesh(currentAnalysis)
			.then((layout) => {
				if (cancelled) {
					return;
				}
				startTransition(() => {
					setState({
						layout,
						error: null,
					});
				});
			})
			.catch(() => {
				if (cancelled) {
					return;
				}
				startTransition(() => {
					setState({
						layout: null,
						error: "Failed to compute a stable mesh layout.",
					});
				});
			});

		return () => {
			cancelled = true;
		};
	}, [analysis?.topologyKey]);

	return state;
}

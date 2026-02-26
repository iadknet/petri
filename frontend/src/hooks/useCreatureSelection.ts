import { type RefObject, useCallback, useRef } from "react";
import type { WorldRenderer } from "../canvas/renderer.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";

const DRAG_THRESHOLD = 5;

export function useCreatureSelection(
	rendererRef: RefObject<WorldRenderer | null>,
) {
	const mouseDownPos = useRef<{ x: number; y: number } | null>(null);

	const handleMouseDown = useCallback((e: React.MouseEvent) => {
		if (e.button === 0) {
			mouseDownPos.current = { x: e.clientX, y: e.clientY };
		}
	}, []);

	const handleMouseUp = useCallback(
		(e: React.MouseEvent) => {
			const down = mouseDownPos.current;
			mouseDownPos.current = null;
			if (!down || e.button !== 0) return;

			const dx = e.clientX - down.x;
			const dy = e.clientY - down.y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			if (distance >= DRAG_THRESHOLD) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const world = renderer.canvasToWorld(e.clientX, e.clientY);
			const frame = useSimulationStore.getState().frame;
			if (!frame) return;

			// Find creature at world position
			const creature = frame.creatures.find(
				(c) => c.x === world.x && c.y === world.y,
			);

			if (creature) {
				useCreatureInspectorStore.getState().selectCreature(creature.id);
			} else {
				useCreatureInspectorStore.getState().clearSelection();
			}
		},
		[rendererRef],
	);

	return { handleMouseDown, handleMouseUp };
}

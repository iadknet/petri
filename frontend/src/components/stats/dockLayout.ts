/** Height and collapsed state of the stats dock, remembered per browser. */

export interface DockLayout {
	height: number;
	collapsed: boolean;
}

export const MIN_DOCK_HEIGHT = 160;
export const DEFAULT_DOCK: DockLayout = { height: 340, collapsed: false };
const KEY = "petri.statsDock";

export function clampDockHeight(height: number, windowHeight: number): number {
	return Math.round(Math.min(Math.max(height, MIN_DOCK_HEIGHT), windowHeight * 0.75));
}

function browserStorage(): Storage | null {
	try {
		return typeof window === "undefined" ? null : window.localStorage;
	} catch {
		return null;
	}
}

export function loadDock(storage: Storage | null = browserStorage()): DockLayout {
	try {
		const raw = storage?.getItem(KEY);
		if (!raw) return DEFAULT_DOCK;
		const parsed: unknown = JSON.parse(raw);
		if (
			typeof parsed === "object" &&
			parsed !== null &&
			"height" in parsed &&
			"collapsed" in parsed &&
			typeof parsed.height === "number" &&
			Number.isFinite(parsed.height) &&
			typeof parsed.collapsed === "boolean"
		) {
			return { height: parsed.height, collapsed: parsed.collapsed };
		}
	} catch {
		// Private windows and blocked site data fall back to the default.
	}
	return DEFAULT_DOCK;
}

export function saveDock(layout: DockLayout, storage: Storage | null = browserStorage()): void {
	try {
		storage?.setItem(KEY, JSON.stringify(layout));
	} catch {
		// Remembering the height is a convenience; losing it is fine.
	}
}

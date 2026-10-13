import { useSyncExternalStore } from "react";

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function reducedMotionQuery(): MediaQueryList | null {
	if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
		return null;
	}
	return window.matchMedia(REDUCED_MOTION_QUERY);
}

function subscribe(onChange: () => void): () => void {
	const query = reducedMotionQuery();
	if (!query) {
		return () => {};
	}
	query.addEventListener("change", onChange);
	return () => query.removeEventListener("change", onChange);
}

/** Whether the user currently asks the OS to reduce motion. */
export function prefersReducedMotion(): boolean {
	return reducedMotionQuery()?.matches ?? false;
}

/** Subscribes to the user's prefers-reduced-motion setting. */
export function usePrefersReducedMotion(): boolean {
	return useSyncExternalStore(subscribe, prefersReducedMotion, () => false);
}

import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { prefersReducedMotion, usePrefersReducedMotion } from "./usePrefersReducedMotion.ts";

function stubMatchMedia(initial: boolean) {
	const listeners = new Set<() => void>();
	const mql = {
		matches: initial,
		addEventListener: (_: string, listener: () => void) => listeners.add(listener),
		removeEventListener: (_: string, listener: () => void) => listeners.delete(listener),
	};
	const matchMedia = vi.fn((query: string) => {
		expect(query).toBe("(prefers-reduced-motion: reduce)");
		return mql;
	});
	vi.stubGlobal("matchMedia", matchMedia);
	return {
		set(matches: boolean) {
			mql.matches = matches;
			for (const listener of listeners) listener();
		},
	};
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe("prefersReducedMotion", () => {
	it("is false when matchMedia is unavailable", () => {
		vi.stubGlobal("matchMedia", undefined);
		expect(prefersReducedMotion()).toBe(false);
	});

	it("reads the reduced-motion media query", () => {
		stubMatchMedia(true);
		expect(prefersReducedMotion()).toBe(true);
	});
});

describe("usePrefersReducedMotion", () => {
	it("tracks changes to the media query", () => {
		const media = stubMatchMedia(false);
		const { result } = renderHook(() => usePrefersReducedMotion());
		expect(result.current).toBe(false);

		act(() => media.set(true));
		expect(result.current).toBe(true);
	});
});

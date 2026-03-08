import "@testing-library/jest-dom/vitest";

class TestResizeObserver {
	observe() {}
	unobserve() {}
	disconnect() {}
}

if (!("ResizeObserver" in globalThis)) {
	Object.defineProperty(globalThis, "ResizeObserver", {
		writable: true,
		configurable: true,
		value: TestResizeObserver,
	});
}

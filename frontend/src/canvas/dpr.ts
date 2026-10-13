export interface BackingStoreSize {
	width: number;
	height: number;
}

/** Current devicePixelRatio, or 1 outside a browser or for an invalid value. */
export function currentDevicePixelRatio(): number {
	const ratio = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
	return Number.isFinite(ratio) && ratio > 0 ? ratio : 1;
}

/** Backing-store pixel size for a canvas of the given CSS size. */
export function backingStoreSize(
	cssWidth: number,
	cssHeight: number,
	ratio: number,
): BackingStoreSize {
	const dpr = Number.isFinite(ratio) && ratio > 0 ? ratio : 1;
	return {
		width: Math.max(1, Math.round(cssWidth * dpr)),
		height: Math.max(1, Math.round(cssHeight * dpr)),
	};
}

/**
 * Call `onChange` with the new ratio whenever devicePixelRatio changes
 * (browser zoom, moving the window between displays). Returns an unsubscribe.
 */
export function watchDevicePixelRatio(onChange: (ratio: number) => void): () => void {
	if (typeof matchMedia !== "function") return () => {};
	let query: MediaQueryList | null = null;
	const listener = () => {
		arm();
		onChange(currentDevicePixelRatio());
	};
	const arm = () => {
		query?.removeEventListener("change", listener);
		query = matchMedia(`(resolution: ${currentDevicePixelRatio()}dppx)`);
		query.addEventListener("change", listener);
	};
	arm();
	return () => {
		query?.removeEventListener("change", listener);
		query = null;
	};
}

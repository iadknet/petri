/**
 * The server stores most config values as f32 and serializes them widened to
 * f64, so 0.7 arrives as 0.699999988079071. Seven significant digits is what
 * an f32 holds; rounding to it recovers the value the user actually set.
 */
export function cleanConfigNumber(value: number): number {
	if (!Number.isFinite(value) || Number.isInteger(value)) return value;
	return Number(value.toPrecision(7));
}

/** Plain decimal text for a config value, without f32 noise or exponents. */
export function formatConfigNumber(value: number): string {
	const clean = cleanConfigNumber(value);
	if (clean === 0 || !Number.isFinite(clean)) return String(clean);
	const magnitude = Math.abs(clean);
	if (magnitude >= 1e-6 && magnitude < 1e21) {
		// toFixed avoids exponent notation for the small costs (1e-6), then the
		// trailing zeros it pads with are trimmed.
		const decimals = Math.min(20, Math.max(0, 7 - Math.floor(Math.log10(magnitude))));
		return clean
			.toFixed(decimals)
			.replace(/(\.\d*?)0+$/, "$1")
			.replace(/\.$/, "");
	}
	return String(clean);
}

/** Equality that ignores f32 widening, so a value typed as shown is not "changed". */
export function sameConfigNumber(a: number, b: number): boolean {
	return a === b || Math.fround(a) === Math.fround(b);
}

/** Decimal places in a step such as 0.001, used to round stepped values. */
export function stepDecimals(step: number): number {
	const text = formatConfigNumber(step);
	const dot = text.indexOf(".");
	return dot < 0 ? 0 : text.length - dot - 1;
}

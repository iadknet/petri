import type { FieldError } from "./generated/FieldError.ts";

/** Renders a request failure and its per-field reasons as one visible line. */
export function describeApiFailure(message: string, fieldErrors: readonly FieldError[]): string {
	return [message, ...fieldErrors.map(({ field, reason }) => `${field}: ${reason}`)].join(" — ");
}

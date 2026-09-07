import type { SimState } from "./protocol.ts";

export interface FieldError {
	field: string;
	reason: string;
}

export interface ErrorDetails {
	endpoint?: string;
	field_errors?: FieldError[];
	expected_state?: SimState;
	current_state?: SimState;
}

/** Renders a request failure and its per-field reasons as one visible line. */
export function describeApiFailure(message: string, fieldErrors: FieldError[]): string {
	return [message, ...fieldErrors.map(({ field, reason }) => `${field}: ${reason}`)].join(" — ");
}

export interface ApiError {
	protocol_version: string;
	error: {
		code: "invalid_request" | "invalid_state_transition" | "validation_rejected" | "internal_error";
		message: string;
		details?: ErrorDetails;
	};
}

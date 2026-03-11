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

export interface ApiError {
	protocol_version: string;
	error: {
		code: "invalid_request" | "invalid_state_transition" | "validation_rejected" | "internal_error";
		message: string;
		details?: ErrorDetails;
	};
}

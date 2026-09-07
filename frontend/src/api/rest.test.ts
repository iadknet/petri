import { describe, expect, it } from "vitest";
import type { ApiError } from "../types/errors.ts";
import { ApiRequestError } from "./rest.ts";

function envelope(error: Partial<ApiError["error"]>): ApiError {
	return {
		protocol_version: "v3alpha2",
		error: error as ApiError["error"],
	};
}

describe("ApiRequestError", () => {
	it("carries the server message and its field errors", () => {
		const error = new ApiRequestError(
			422,
			envelope({
				code: "validation_rejected",
				message: "validation failed for patch_config",
				details: {
					endpoint: "patch_config",
					field_errors: [{ field: "population.max_creatures", reason: "requested 5000" }],
				},
			}),
		);

		expect(error.status).toBe(422);
		expect(error.message).toBe("validation failed for patch_config");
		expect(error.fieldErrors).toEqual([
			{ field: "population.max_creatures", reason: "requested 5000" },
		]);
	});

	it("defaults field errors to an empty list", () => {
		const error = new ApiRequestError(
			409,
			envelope({ code: "invalid_state_transition", message: "current state is 'idle'" }),
		);

		expect(error.fieldErrors).toEqual([]);
	});

	it("falls back to the error code when the server message is empty", () => {
		const error = new ApiRequestError(422, envelope({ code: "validation_rejected", message: "" }));

		expect(error.message).toBe("validation_rejected (HTTP 422)");
	});

	it("falls back to a generic message when the body carries no error payload", () => {
		const error = new ApiRequestError(500, {} as ApiError);

		expect(error.message).toBe("Request failed (HTTP 500)");
		expect(error.fieldErrors).toEqual([]);
	});
});

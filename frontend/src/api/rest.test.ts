import { afterEach, describe, expect, it, vi } from "vitest";
import type { ApiError } from "../types/errors.ts";
import { ApiRequestError, api } from "./rest.ts";

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

describe("recipe transport", () => {
	afterEach(() => vi.unstubAllGlobals());
	it("exports fresh raw config and preserves imported large seeds", async () => {
		const recipe =
			' {"world": {"world_seed": 18446744073709551615, "terrain": [{"seed": 18446744073709551615}], "food": {"fertility": {"layers": [{"algorithm": {"Fbm": {"seed": 18446744073709551615}}}]}}}} ';
		const fetchMock = vi.fn().mockImplementation(() => Promise.resolve(new Response(recipe)));
		vi.stubGlobal("fetch", fetchMock);
		expect(await api.getRecipe()).toBe(recipe);
		expect(fetchMock.mock.calls[0]?.[0]).toBe("/v3/simulation/config?format=recipe");
		await api.loadRecipe(recipe, 42);
		expect(String(fetchMock.mock.calls[1]?.[1]?.body).match(/18446744073709551615/g)).toHaveLength(
			3,
		);
		expect(JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body)).seed).toBe(42);
	});
	it("accepts empty objects and rejects nonobjects, malformed JSON and embedded run seeds", async () => {
		const fetchMock = vi.fn().mockImplementation(() => Promise.resolve(new Response("{}")));
		vi.stubGlobal("fetch", fetchMock);
		await api.loadRecipe("  {  } \n", 7);
		expect(JSON.parse(String(fetchMock.mock.calls[0]?.[1]?.body))).toEqual({ seed: 7 });
		for (const bad of ["[]", "null", "1", "{", '{"seed":99}']) {
			await expect(api.loadRecipe(bad, 7)).rejects.toThrow();
		}
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
});

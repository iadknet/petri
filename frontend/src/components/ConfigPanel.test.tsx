import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import type { FieldError } from "../types/errors.ts";
import { ConfigPanel } from "./ConfigPanel.tsx";

vi.mock("../api/rest.ts", async (importOriginal) => ({
	...(await importOriginal<typeof import("../api/rest.ts")>()),
	api: {
		patchConfig: vi.fn(),
	},
}));

function rejection(fieldErrors: FieldError[]): ApiRequestError {
	return new ApiRequestError(422, {
		protocol_version: "v3alpha2",
		error: {
			code: "validation_rejected",
			message: "validation failed for patch_config",
			details: { endpoint: "patch_config", field_errors: fieldErrors },
		},
	});
}

describe("ConfigPanel", () => {
	beforeEach(() => {
		useConfigStore.getState().reset();
		useSimulationStore.getState().reset();
		useStartupConfigStore.getState().reset();
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		useSimulationStore.getState().setSimState("paused");
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
		vi.clearAllMocks();
		vi.mocked(api.patchConfig).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "paused",
			config: MOCK_CONFIG,
		});
	});

	it("saves the shared live Eat Reward", async () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-energy-costs-eat-reward-per-food"), {
			target: { value: "7.5" },
		});
		expect(useConfigStore.getState().localDraft?.energy.costs.eat_reward_per_food).toBe(7.5);
		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));
		await waitFor(() =>
			expect(api.patchConfig).toHaveBeenCalledWith(
				expect.objectContaining({
					energy: expect.objectContaining({
						costs: expect.objectContaining({ eat_reward_per_food: 7.5 }),
					}),
				}),
			),
		);
	});

	it("edits mutation event continuation probability in the runtime draft", () => {
		render(<ConfigPanel />);
		const field = screen.getByTestId("config-field-mutation-event-continuation-probability");
		expect(field).toHaveValue(0.2);
		fireEvent.change(field, { target: { value: "0.35" } });
		expect(
			useConfigStore.getState().localDraft?.mutation
				.per_birth_mutation_event_continuation_probability,
		).toBe(0.35);
	});

	it("renders startup and runtime top-level sections with food subgroup", () => {
		render(<ConfigPanel />);

		expect(screen.getByText("Startup Config")).toBeInTheDocument();
		expect(screen.getByText("Runtime (Live) Config")).toBeInTheDocument();
		expect(screen.getByTestId("startup-seed-randomize")).toBeInTheDocument();
		expect(screen.getByText("Food > Occupancy Depletion")).toBeInTheDocument();
		expect(screen.getAllByText("Food Types").length).toBeGreaterThan(0);
		expect(screen.getByTestId("startup-food-type-add")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-initial-density")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-initial-coverage")).toBeInTheDocument();
		expect(
			screen.queryByTestId("startup-field-food-type-0-metabolic-yield"),
		).not.toBeInTheDocument();
		expect(screen.queryByTestId("startup-field-food-type-0-reserve-yield")).not.toBeInTheDocument();
		expect(
			screen.queryByTestId("startup-field-nutrition-reserve-capacity"),
		).not.toBeInTheDocument();
		expect(screen.queryByTestId("startup-field-nutrition-reserve-cost")).not.toBeInTheDocument();
		expect(screen.getByTestId("startup-field-energy-initial-energy")).toBeInTheDocument();
	});

	it("renders the primary food defaults", () => {
		act(() => useStartupConfigStore.getState().reset());
		render(<ConfigPanel />);
		expect(screen.getByTestId("startup-field-food-type-0-initial-coverage")).toHaveValue(0.54);
		expect(screen.queryByTestId("startup-food-type-card-1")).not.toBeInTheDocument();
		expect(screen.getByTestId("config-field-energy-costs-eat-reward-per-food")).toHaveValue(5);
	});

	it("startup edits do not modify runtime local draft for startup-only fields", () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("startup-field-food-type-0-initial-density"), {
			target: { value: "0.75" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.types[0]!.initial_density).toBe(0.75);
		expect(useConfigStore.getState().localDraft?.world.food.shared.initial_density).toBe(1.0);
	});

	it("applies runtime field disable rules by simulation state", () => {
		render(<ConfigPanel />);
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("idle");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("running");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).toBeDisabled();
	});

	it("shows runtime unavailable message when runtime config is missing", () => {
		useConfigStore.getState().reset();
		render(<ConfigPanel />);

		expect(
			screen.getByText("Runtime config unavailable. Restart to initialize the simulation."),
		).toBeInTheDocument();
	});

	it("omits retired newborn backend controls", () => {
		render(<ConfigPanel />);
		expect(
			screen.queryByTestId("config-field-mutation-birth-graph-backend-chance"),
		).not.toBeInTheDocument();
	});

	it("supports occupancy depletion runtime controls", () => {
		render(<ConfigPanel />);

		const enabled = screen.getByTestId("config-field-food-occupancy-depletion-enabled");
		const rate = screen.getByTestId(
			"config-field-food-occupancy-depletion-deposit-per-occupied-tick",
		);

		expect(enabled).toBeChecked();
		expect(rate).toHaveValue(0.08);

		fireEvent.click(enabled);
		fireEvent.change(rate, { target: { value: "0.25" } });

		expect(useConfigStore.getState().localDraft?.world.food.shared.occupancy_depletion).toEqual({
			enabled: false,
			deposit_per_occupied_tick: 0.25,
		});
	});

	it("does not render legacy input auto-connect runtime field", () => {
		render(<ConfigPanel />);
		expect(
			screen.queryByTestId("config-field-mutation-input-auto-connect"),
		).not.toBeInTheDocument();
	});

	it("locks runtime failed action penalty while startup ramp is active", () => {
		useConfigStore.getState().setServerConfig(
			{
				...MOCK_CONFIG,
				startup: {
					ramps: {
						failed_action_penalty: {
							enabled: true,
							start: 5,
							end: 30,
							target_tick: 1000,
						},
					},
				},
			},
			"paused",
		);
		useSimulationStore.getState().setTick(10);

		render(<ConfigPanel />);
		expect(screen.getByTestId("config-field-energy-costs-failed-action-penalty")).toBeDisabled();
	});

	it("supports fertility startup controls and persists values in startup preset", () => {
		render(<ConfigPanel />);

		const fertilityToggle = screen.getByTestId("startup-field-fertility-enabled");
		expect(fertilityToggle).not.toBeChecked();
		expect(screen.queryByTestId("startup-field-fertility-min")).not.toBeInTheDocument();

		fireEvent.click(fertilityToggle);

		const minField = screen.getByTestId("startup-field-fertility-min");
		const maxField = screen.getByTestId("startup-field-fertility-max");
		expect(minField).toHaveValue(0);
		expect(maxField).toHaveValue(2);

		fireEvent.change(minField, { target: { value: "0.6" } });
		fireEvent.change(maxField, { target: { value: "1.7" } });

		const annealingToggle = screen.getByTestId("startup-field-annealing-enabled");
		expect(annealingToggle).not.toBeChecked();
		fireEvent.click(annealingToggle);

		fireEvent.change(screen.getByTestId("startup-field-annealing-ramp-ticks"), {
			target: { value: "9000" },
		});
		fireEvent.change(screen.getByTestId("startup-field-annealing-initial-min"), {
			target: { value: "0.8" },
		});
		fireEvent.change(screen.getByTestId("startup-field-annealing-initial-max"), {
			target: { value: "1.4" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.fertility).toMatchObject({
			enabled: true,
			min_fertility: 0.6,
			max_fertility: 1.7,
		});
		expect(useStartupConfigStore.getState().preset.world.food.annealing).toMatchObject({
			enabled: true,
			ramp_ticks: 9000,
			initial_min_fertility: 0.8,
			initial_max_fertility: 1.4,
		});
	});

	it("supports adding and removing food types in the startup panel", () => {
		render(<ConfigPanel />);

		expect(screen.getByTestId("startup-food-type-card-0")).toBeInTheDocument();
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		expect(screen.getByTestId("startup-food-type-card-1")).toBeInTheDocument();
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-name"), {
			target: { value: "Blue Food" },
		});
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-color"), {
			target: { value: "#3b82f6" },
		});
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-growth-inhibitor"), {
			target: { value: "0.45" },
		});
		expect(useStartupConfigStore.getState().preset.world.food.types[1]!.growth_inhibitor).toBe(
			0.45,
		);
		fireEvent.click(screen.getByTestId("startup-food-type-remove-1"));

		expect(screen.queryByTestId("startup-food-type-card-1")).not.toBeInTheDocument();
		expect(useStartupConfigStore.getState().preset.world.food.types).toHaveLength(1);
		expect(useStartupConfigStore.getState().preset.world.food.types[0]).toMatchObject({
			name: "Primary Food",
			color: "#22c55e",
			growth_inhibitor: 0.2,
		});
	});

	it("updates fertility targets using the current food type list", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		const target = screen.getByTestId("startup-field-fertility-layer-0-target");
		expect(target).toHaveValue("all_foods");
		expect(within(target).getByRole("option", { name: "Type 2: Food 2" })).toBeInTheDocument();

		fireEvent.change(target, { target: { value: "single_type:1" } });

		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]!.target).toEqual({
			SingleType: { type_idx: 1 },
		});
	});

	it("retargets fertility layers to all foods when a targeted type is removed", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-target"), {
			target: { value: "single_type:1" },
		});
		fireEvent.click(screen.getByTestId("startup-food-type-remove-1"));

		expect(screen.getByTestId("startup-field-fertility-layer-0-target")).toHaveValue("all_foods");
		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]!.target).toEqual(
			"AllFoods",
		);
	});

	it("supports editing fertility layer algorithm settings", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));

		expect(screen.getByTestId("startup-field-fertility-layer-0-algorithm")).toBeInTheDocument();
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Fbm" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-weight"), {
			target: { value: "0.65" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-octaves"), {
			target: { value: "5" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-frequency"), {
			target: { value: "0.08" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-lacunarity"), {
			target: { value: "2.2" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-persistence"), {
			target: { value: "0.47" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed-enabled"));
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed"), {
			target: { value: "4242" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]).toEqual({
			weight: 0.65,
			algorithm: {
				Fbm: {
					octaves: 5,
					frequency: 0.08,
					lacunarity: 2.2,
					persistence: 0.47,
					seed: 4242,
				},
			},
		});
	});

	it("shows ELI5 tooltips for FBM fertility controls", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Fbm" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed-enabled"));

		expect(
			screen.getByText(/detail layers are stacked together for richer texture/i),
		).toBeInTheDocument();
		expect(screen.getByText(/how zoomed in the pattern is/i)).toBeInTheDocument();
		expect(screen.getByText(/how much smaller each next detail layer gets/i)).toBeInTheDocument();
		expect(screen.getByText(/how strong those smaller detail layers stay/i)).toBeInTheDocument();
		expect(screen.getByText(/lock the random pattern so it repeats exactly/i)).toBeInTheDocument();
		expect(screen.getByText(/same seed gives the same pattern/i)).toBeInTheDocument();
	});

	it("shows ELI5 tooltips for Uniform and Poisson fertility controls", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Uniform" },
		});
		expect(screen.getByText(/single flat fertility value everywhere/i)).toBeInTheDocument();

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "PoissonBlobs" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-poisson-seed-enabled"));

		expect(
			screen.getAllByText(/how many fertility islands to drop on the map/i).length,
		).toBeGreaterThan(0);
		expect(screen.getAllByText(/smallest island size allowed/i).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/largest island size allowed/i).length).toBeGreaterThan(0);
		expect(
			screen.getAllByText(/how softly each island fades at the edges/i).length,
		).toBeGreaterThan(0);
		expect(
			screen.getAllByText(/lock island placement so it repeats exactly/i).length,
		).toBeGreaterThan(0);
		expect(screen.getAllByText(/same seed gives the same island layout/i).length).toBeGreaterThan(
			0,
		);
	});

	it("renders a rejected field under its own row and clears it on the next Apply", async () => {
		vi.mocked(api.patchConfig).mockRejectedValueOnce(
			rejection([
				{
					field: "population.max_creatures",
					reason: "requested 5000; canonical value is 100000",
				},
			]),
		);
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-energy-costs-eat-reward-per-food"), {
			target: { value: "7.5" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));

		const rowError = await screen.findByTestId("field-error-population-max_creatures");
		expect(rowError).toHaveTextContent("requested 5000; canonical value is 100000");
		expect(screen.getByTestId("config-error")).toHaveTextContent(
			"validation failed for patch_config",
		);

		vi.mocked(api.patchConfig).mockResolvedValueOnce({
			protocol_version: "v3alpha2",
			state: "paused",
			config: MOCK_CONFIG,
		});
		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));

		await waitFor(() => {
			expect(screen.queryByTestId("field-error-population-max_creatures")).not.toBeInTheDocument();
		});
		expect(screen.queryByTestId("config-error")).not.toBeInTheDocument();
	});

	it("renders field errors that match no runtime row in the panel message", async () => {
		vi.mocked(api.patchConfig).mockRejectedValueOnce(
			rejection([{ field: "config", reason: "unknown field `bogus`" }]),
		);
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-energy-costs-eat-reward-per-food"), {
			target: { value: "7.5" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));

		const panelError = await screen.findByTestId("config-error");
		expect(panelError).toHaveTextContent("config: unknown field `bogus`");
		expect(screen.queryByTestId("field-error-config")).not.toBeInTheDocument();
	});

	it("falls back to a panel message when the failure is not an API error", async () => {
		vi.mocked(api.patchConfig).mockRejectedValueOnce("network down");
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-energy-costs-eat-reward-per-food"), {
			target: { value: "7.5" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));

		expect(await screen.findByTestId("config-error")).toHaveTextContent("Config update failed");
	});

	it("clears field errors when the draft is reset", async () => {
		vi.mocked(api.patchConfig).mockRejectedValueOnce(
			rejection([{ field: "action_log.capacity", reason: "requested 0; canonical value is 500" }]),
		);
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-energy-costs-eat-reward-per-food"), {
			target: { value: "7.5" },
		});
		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));
		await screen.findByTestId("field-error-action_log-capacity");

		fireEvent.click(screen.getByTestId("config-reset"));

		expect(screen.queryByTestId("field-error-action_log-capacity")).not.toBeInTheDocument();
		expect(screen.queryByTestId("config-error")).not.toBeInTheDocument();
	});

	it("keeps a typed startup value above the static max after commit", () => {
		render(<ConfigPanel />);
		const input = screen.getByTestId("startup-field-population-initial-creatures");

		fireEvent.change(input, { target: { value: "20000" } });
		fireEvent.blur(input);

		expect(useStartupConfigStore.getState().preset.population.initial_creatures).toBe(20000);
	});

	it("clamps the slider to the draft-derived bound on every change", () => {
		render(<ConfigPanel />);

		fireEvent.change(screen.getByLabelText("Max Creatures slider"), { target: { value: "10" } });

		expect(useConfigStore.getState().localDraft?.population.max_creatures).toBe(
			MOCK_CONFIG.population.initial_creatures,
		);
	});

	it("clamps the number input to the draft-derived bound only once the edit is committed", () => {
		render(<ConfigPanel />);
		const input = screen.getByRole("spinbutton", { name: /Max Creatures/ });

		fireEvent.change(input, { target: { value: "10" } });
		expect(useConfigStore.getState().localDraft?.population.max_creatures).toBe(10);

		fireEvent.blur(input);
		expect(useConfigStore.getState().localDraft?.population.max_creatures).toBe(
			MOCK_CONFIG.population.initial_creatures,
		);
	});

	it("commits a number edit on Enter", () => {
		render(<ConfigPanel />);
		const input = screen.getByRole("spinbutton", { name: /Log Capacity/ });

		fireEvent.change(input, { target: { value: "0" } });
		fireEvent.keyDown(input, { key: "Enter" });

		expect(useConfigStore.getState().localDraft?.action_log.capacity).toBe(1);
	});

	it("blurs the focused number input before applying so the sent value is in bounds", async () => {
		render(<ConfigPanel />);
		const input = screen.getByRole<HTMLInputElement>("spinbutton", { name: /Max Creatures/ });
		input.focus();
		fireEvent.change(input, { target: { value: "10" } });

		fireEvent.click(screen.getByRole("button", { name: "Apply Changes" }));

		await waitFor(() =>
			expect(api.patchConfig).toHaveBeenCalledWith(
				expect.objectContaining({
					population: { max_creatures: MOCK_CONFIG.population.initial_creatures },
				}),
			),
		);
	});
});

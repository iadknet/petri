import { useRef, useState } from "react";
import { ApiRequestError, api } from "../../../api/rest.ts";
import { useConfigStore } from "../../../stores/config.ts";
import { useSimulationStore } from "../../../stores/simulation.ts";
import { useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { useStatsHistoryStore } from "../../../stores/stats.ts";
import { describeApiFailure } from "../../../types/errors.ts";
import { Tooltip } from "../shared/Tooltip.tsx";

function failure(error: unknown): string {
	return error instanceof ApiRequestError
		? describeApiFailure(error.message, error.fieldErrors)
		: error instanceof Error
			? error.message
			: "Recipe operation failed. Please try again.";
}

export function RecipeControls() {
	const fileInput = useRef<HTMLInputElement>(null);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [message, setMessage] = useState("");

	async function save() {
		setBusy(true);
		setError(null);
		setMessage("");
		try {
			const recipe = await api.getRecipe();
			const url = URL.createObjectURL(new Blob([recipe], { type: "application/json" }));
			const anchor = document.createElement("a");
			anchor.href = url;
			anchor.download = "world-recipe.json";
			document.body.append(anchor);
			anchor.click();
			anchor.remove();
			setTimeout(() => URL.revokeObjectURL(url), 0);
			setMessage("Recipe downloaded.");
		} catch (error) {
			setError(failure(error));
		} finally {
			setBusy(false);
		}
	}

	async function load(file: File | undefined) {
		if (!file) return;
		const state = useSimulationStore.getState().simState;
		if (
			(state === "running" || state === "paused") &&
			!window.confirm("Load this recipe and restart the simulation with the current Run Seed?")
		)
			return;
		setBusy(true);
		setError(null);
		setMessage("");
		let restarted = false;
		try {
			const seed = useStartupConfigStore.getState().preset.seed;
			const response = await api.loadRecipe(await file.text(), seed);
			restarted = true;
			useSimulationStore.getState().setSimState(response.state);
			useSimulationStore.getState().setTick(response.tick);
			useStatsHistoryStore.getState().reset();
			const config = await api.getConfig();
			useConfigStore.getState().commitServerConfig(config.config, config.state);
			useStartupConfigStore.getState().applyRecipeConfig(config.config);
			setMessage("Recipe loaded at tick zero.");
		} catch (error) {
			setError(
				`${restarted ? "World restarted, but refreshing controls failed. " : ""}${failure(error)}`,
			);
		} finally {
			setBusy(false);
		}
	}

	const button =
		"flex h-7 flex-1 items-center justify-center rounded border border-slate-700 px-2 text-xs text-slate-200 transition-colors hover:border-slate-600 hover:bg-slate-800 disabled:opacity-50";

	return (
		<div className="flex flex-col gap-1.5 border-b border-slate-800/80 px-3 py-2.5">
			<div className="flex items-center gap-1.5" aria-busy={busy}>
				<button type="button" disabled={busy} onClick={save} className={button}>
					Save Recipe
				</button>
				<button
					type="button"
					disabled={busy}
					onClick={() => fileInput.current?.click()}
					className={button}
				>
					Load Recipe
				</button>
				<Tooltip text="Save writes the applied procedural config. Load restarts at tick zero with the current Run Seed, so the same seed regenerates the same world; painted changes and living state are not saved. Recipe files keep large seeds exactly; the numeric editor handles seeds up to 9007199254740991." />
				<input
					ref={fileInput}
					type="file"
					accept=".json,application/json"
					aria-label="Recipe file"
					className="hidden"
					disabled={busy}
					onChange={(event) => {
						const file = event.currentTarget.files?.[0];
						event.currentTarget.value = "";
						void load(file);
					}}
				/>
			</div>
			{error ? (
				<p role="alert" className="text-[11px] text-red-400">
					{error}
				</p>
			) : null}
			<output className="block text-[11px] text-slate-400 empty:hidden">
				{busy ? "Working…" : message}
			</output>
		</div>
	);
}

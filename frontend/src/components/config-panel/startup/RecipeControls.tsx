import { useRef, useState } from "react";
import { ApiRequestError, api } from "../../../api/rest.ts";
import { useConfigStore } from "../../../stores/config.ts";
import { useSimulationStore } from "../../../stores/simulation.ts";
import { useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { useStatsHistoryStore } from "../../../stores/stats.ts";
import { describeApiFailure } from "../../../types/errors.ts";

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

	return (
		<div className="space-y-2 px-3 py-2 mb-3">
			<div className="flex gap-2" aria-busy={busy}>
				<button
					type="button"
					disabled={busy}
					onClick={save}
					className="min-h-9 flex-1 rounded bg-slate-700 px-3 text-xs text-slate-200 hover:bg-slate-600 disabled:opacity-50"
				>
					Save Recipe
				</button>
				<button
					type="button"
					disabled={busy}
					onClick={() => fileInput.current?.click()}
					className="min-h-9 flex-1 rounded bg-slate-700 px-3 text-xs text-slate-200 hover:bg-slate-600 disabled:opacity-50"
				>
					Load Recipe
				</button>
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
			<p className="text-xs text-slate-400">
				Save the applied procedural config. Load restarts at tick zero. Use the same Run Seed to
				regenerate the same world; painted changes and living state are not saved.
			</p>
			<p className="text-xs text-slate-400">
				Recipe files preserve large seeds exactly. The numeric editor supports seeds up to
				9007199254740991.
			</p>
			{error ? (
				<p role="alert" className="text-xs text-red-400">
					{error}
				</p>
			) : null}
			<output className="block text-xs text-slate-300">{busy ? "Working…" : message}</output>
		</div>
	);
}

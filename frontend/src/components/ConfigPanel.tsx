import { useCallback, useState } from "react";
import type { DeepPartial } from "../api/rest.ts";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import type { SimulationConfig } from "../types/api.ts";
import { type FieldError, describeApiFailure } from "../types/errors.ts";
import {
	RUNTIME_PATCH_FIELDS,
	RuntimeConfigPanel,
} from "./config-panel/runtime/RuntimeConfigPanel.tsx";
import { buildPatch, getByPath, mergePatch } from "./config-panel/shared/pathUtils.ts";
import { StartupConfigPanel } from "./config-panel/startup/StartupConfigPanel.tsx";

/** Paths that have a runtime row of their own, so a field error can be shown there. */
const RUNTIME_ROW_PATHS = new Set(
	RUNTIME_PATCH_FIELDS.filter((field) => "min" in field).map((field) => field.path),
);

export function ConfigPanel() {
	const localDraft = useConfigStore((s) => s.localDraft);
	const serverConfig = useConfigStore((s) => s.serverConfig);
	const isDirty = useConfigStore((s) => s.isDirty);
	const simState = useSimulationStore((s) => s.simState);
	const tick = useSimulationStore((s) => s.tick);
	const updateDraft = useConfigStore((s) => s.updateDraft);
	const resetDraft = useConfigStore((s) => s.resetDraft);
	const commitServerConfig = useConfigStore((s) => s.commitServerConfig);

	const startupPreset = useStartupConfigStore((s) => s.preset);
	const updateStartupPreset = useStartupConfigStore((s) => s.updatePreset);
	const addFoodType = useStartupConfigStore((s) => s.addFoodType);
	const removeFoodType = useStartupConfigStore((s) => s.removeFoodType);
	const updateFoodType = useStartupConfigStore((s) => s.updateFoodType);
	const addFertilityLayer = useStartupConfigStore((s) => s.addFertilityLayer);
	const updateFertilityLayerTarget = useStartupConfigStore((s) => s.updateFertilityLayerTarget);
	const randomizeSeed = useStartupConfigStore((s) => s.randomizeSeed);

	const [error, setError] = useState<string | null>(null);
	const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
	const [applying, setApplying] = useState(false);

	const handleApply = useCallback(async () => {
		// Commit any in-progress number edit, which clamps it to the resolved
		// bounds, before reading the draft this Apply will send.
		if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
		const { localDraft: draft, serverConfig: server } = useConfigStore.getState();
		if (!draft || !server) return;
		setApplying(true);
		setError(null);
		setFieldErrors({});

		try {
			const patch: Record<string, unknown> = {};
			for (const field of RUNTIME_PATCH_FIELDS) {
				const draftValue = getByPath(draft, field.path);
				if (draftValue !== getByPath(server, field.path)) {
					mergePatch(patch, buildPatch(field.path, draftValue as number | boolean));
				}
			}

			const res = await api.patchConfig(patch as DeepPartial<SimulationConfig>);
			commitServerConfig(res.config, res.state);
		} catch (e) {
			if (e instanceof ApiRequestError) {
				const rowErrors: Record<string, string> = {};
				const unmatched: FieldError[] = [];
				for (const fieldError of e.fieldErrors) {
					if (RUNTIME_ROW_PATHS.has(fieldError.field)) {
						rowErrors[fieldError.field] = fieldError.reason;
					} else {
						unmatched.push(fieldError);
					}
				}
				setFieldErrors(rowErrors);
				setError(describeApiFailure(e.message, unmatched));
			} else {
				setError(e instanceof Error ? e.message : "Config update failed");
			}
		} finally {
			setApplying(false);
		}
	}, [commitServerConfig]);

	const handleReset = useCallback(() => {
		setError(null);
		setFieldErrors({});
		resetDraft();
	}, [resetDraft]);

	return (
		<aside
			data-testid="config-panel"
			className="w-80 bg-petri-panel border-r border-petri-border overflow-y-auto flex flex-col"
		>
			<div className="flex-1 overflow-y-auto">
				<StartupConfigPanel
					startupPreset={startupPreset}
					updateStartupPreset={updateStartupPreset}
					addFoodType={addFoodType}
					removeFoodType={removeFoodType}
					updateFoodType={updateFoodType}
					addFertilityLayer={addFertilityLayer}
					updateFertilityLayerTarget={updateFertilityLayerTarget}
					randomizeSeed={randomizeSeed}
				/>
				<RuntimeConfigPanel
					localDraft={localDraft}
					serverConfig={serverConfig}
					simState={simState}
					tick={tick}
					updateDraft={updateDraft}
					fieldErrors={fieldErrors}
				/>
			</div>

			{localDraft && serverConfig && (
				<div className="p-3 border-t border-petri-border flex flex-col gap-2">
					{error && (
						<p data-testid="config-error" className="text-xs text-red-400">
							{error}
						</p>
					)}
					<div className="flex gap-2">
						<button
							type="button"
							data-testid="config-apply"
							disabled={!isDirty || applying}
							onClick={handleApply}
							className="flex-1 px-3 py-1.5 text-xs font-medium bg-emerald-600 text-white rounded hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed"
						>
							{applying ? "Applying..." : "Apply Changes"}
						</button>
						<button
							type="button"
							data-testid="config-reset"
							disabled={!isDirty}
							onClick={handleReset}
							className="px-3 py-1.5 text-xs text-slate-400 bg-slate-800 rounded hover:bg-slate-700 disabled:opacity-40"
						>
							Reset
						</button>
					</div>
				</div>
			)}
		</aside>
	);
}

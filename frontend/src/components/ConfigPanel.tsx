import { useCallback, useState } from "react";
import type { DeepPartial } from "../api/rest.ts";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import type { SimulationConfig } from "../types/api.ts";
import {
	RUNTIME_PATCH_FIELDS,
	RuntimeConfigPanel,
} from "./config-panel/runtime/RuntimeConfigPanel.tsx";
import { buildPatch, getByPath, mergePatch } from "./config-panel/shared/pathUtils.ts";
import { StartupConfigPanel } from "./config-panel/startup/StartupConfigPanel.tsx";

export function ConfigPanel() {
	const localDraft = useConfigStore((s) => s.localDraft);
	const serverConfig = useConfigStore((s) => s.serverConfig);
	const isDirty = useConfigStore((s) => s.isDirty);
	const simState = useSimulationStore((s) => s.simState);
	const updateDraft = useConfigStore((s) => s.updateDraft);
	const resetDraft = useConfigStore((s) => s.resetDraft);
	const commitServerConfig = useConfigStore((s) => s.commitServerConfig);

	const startupPreset = useStartupConfigStore((s) => s.preset);
	const updateStartupPreset = useStartupConfigStore((s) => s.updatePreset);
	const randomizeSeed = useStartupConfigStore((s) => s.randomizeSeed);

	const [error, setError] = useState<string | null>(null);
	const [applying, setApplying] = useState(false);

	const handleApply = useCallback(async () => {
		if (!localDraft || !serverConfig) return;
		setApplying(true);
		setError(null);

		try {
			const patch: Record<string, unknown> = {};
			for (const field of RUNTIME_PATCH_FIELDS) {
				const draft = getByPath(localDraft, field.path);
				const server = getByPath(serverConfig, field.path);
				if (draft !== server) {
					mergePatch(patch, buildPatch(field.path, draft as number));
				}
			}

			const res = await api.patchConfig(patch as DeepPartial<SimulationConfig>);
			commitServerConfig(res.config, res.state);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Config update failed");
		} finally {
			setApplying(false);
		}
	}, [commitServerConfig, localDraft, serverConfig]);

	return (
		<aside
			data-testid="config-panel"
			className="w-80 bg-petri-panel border-r border-petri-border overflow-y-auto flex flex-col"
		>
			<div className="flex-1 overflow-y-auto">
				<StartupConfigPanel
					startupPreset={startupPreset}
					updateStartupPreset={updateStartupPreset}
					randomizeSeed={randomizeSeed}
				/>
				<RuntimeConfigPanel
					localDraft={localDraft}
					serverConfig={serverConfig}
					simState={simState}
					updateDraft={updateDraft}
				/>
			</div>

			{localDraft && serverConfig && (
				<div className="p-3 border-t border-petri-border flex flex-col gap-2">
					{error && <p className="text-xs text-red-400">{error}</p>}
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
							onClick={resetDraft}
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

import { useCallback, useState } from "react";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";

export function StartupDialog({ onClose }: { onClose: () => void }) {
	const [seed, setSeed] = useState(() => Math.floor(Math.random() * 2 ** 32));
	const [population, setPopulation] = useState(50);
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const handleStartup = useCallback(async () => {
		setSubmitting(true);
		setError(null);
		try {
			const res = await api.startup({
				seed,
				population: { initial_creatures: population },
			});
			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
			useStatsHistoryStore.getState().reset();

			// Fetch fresh config after startup
			const configRes = await api.getConfig();
			useConfigStore.getState().setServerConfig(configRes.config, configRes.state);

			onClose();
		} catch (e) {
			setError(e instanceof Error ? e.message : "Startup failed");
		} finally {
			setSubmitting(false);
		}
	}, [seed, population, onClose]);

	return (
		<div
			data-testid="startup-modal"
			className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
		>
			<div className="bg-petri-panel border border-petri-border rounded-lg shadow-2xl w-96 p-6">
				<h2 className="text-lg font-semibold text-slate-100 mb-4">New Simulation</h2>

				<div className="flex flex-col gap-4">
					<label className="flex flex-col gap-1">
						<span className="text-xs text-slate-400">Seed</span>
						<input
							data-testid="startup-seed"
							type="number"
							value={seed}
							onChange={(e) => setSeed(Number(e.target.value))}
							className="px-3 py-2 font-mono text-sm bg-slate-800 border border-slate-700 rounded text-slate-200"
						/>
					</label>

					<label className="flex flex-col gap-1">
						<span className="text-xs text-slate-400">Initial Population</span>
						<input
							data-testid="startup-population"
							type="number"
							value={population}
							min={1}
							max={10000}
							onChange={(e) => setPopulation(Number(e.target.value))}
							className="px-3 py-2 font-mono text-sm bg-slate-800 border border-slate-700 rounded text-slate-200"
						/>
					</label>

					{error && (
						<p data-testid="startup-error" className="text-xs text-red-400">
							{error}
						</p>
					)}

					<div className="flex gap-2 mt-2">
						<button
							type="button"
							data-testid="startup-initialize"
							disabled={submitting}
							onClick={handleStartup}
							className="flex-1 px-4 py-2 text-sm font-medium bg-emerald-600 text-white rounded hover:bg-emerald-500 disabled:opacity-50"
						>
							{submitting ? "Starting..." : "Initialize"}
						</button>
						<button
							type="button"
							data-testid="startup-cancel"
							onClick={onClose}
							className="px-4 py-2 text-sm text-slate-400 bg-slate-800 rounded hover:bg-slate-700"
						>
							Cancel
						</button>
					</div>
				</div>
			</div>
		</div>
	);
}

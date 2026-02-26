import { useCallback, useState } from "react";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { usePanelLayout } from "../stores/layout.tsx";
import { useSimulationStore } from "../stores/simulation.ts";
import { buildStartupRequest, useStartupConfigStore } from "../stores/startupConfig.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import type { SimState } from "../types/api.ts";

function formatTps(tps: number): string {
	if (tps === 0) return "0";
	if (tps < 10) return tps.toFixed(1);
	return Math.round(tps).toLocaleString();
}

function ConnectionDot({ status }: { status: string }) {
	const color =
		status === "connected"
			? "bg-emerald-500"
			: status === "connecting"
				? "bg-amber-500"
				: "bg-red-500";

	return (
		<span
			data-testid="connection-status"
			className="flex items-center gap-1.5 text-xs text-slate-400"
		>
			<span className={`inline-block w-2 h-2 rounded-full ${color}`} />
			{status}
		</span>
	);
}

function SimButton({
	label,
	disabled,
	active,
	onClick,
	pulse,
	testId,
}: {
	label: string;
	disabled: boolean;
	active?: boolean;
	onClick: () => void;
	pulse?: boolean;
	testId?: string;
}) {
	return (
		<button
			type="button"
			data-testid={testId}
			disabled={disabled}
			onClick={onClick}
			className={`
				px-3 py-1 text-sm font-medium rounded transition-colors
				${
					disabled
						? "bg-slate-800 text-slate-500 opacity-50 cursor-not-allowed"
						: active
							? "bg-emerald-600 text-white hover:bg-emerald-500"
							: "bg-slate-700 text-slate-200 hover:bg-slate-600"
				}
			`}
		>
			{pulse ? (
				<span className="flex items-center gap-1.5">
					<span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
					{label}
				</span>
			) : (
				label
			)}
		</button>
	);
}

function buttonEnabled(state: SimState) {
	return {
		start: state === "idle" || state === "paused",
		restart: true,
		pause: state === "running",
		step: state === "paused",
	};
}

export function ControlBar() {
	const simState = useSimulationStore((s) => s.simState);
	const tick = useSimulationStore((s) => s.tick);
	const population = useSimulationStore((s) => s.status?.population ?? 0);
	const ticksPerSecond = useSimulationStore((s) => s.ticksPerSecond);
	const connectionStatus = useSimulationStore((s) => s.connectionStatus);
	const { toggleConfig, toggleStats } = usePanelLayout();
	const [restarting, setRestarting] = useState(false);

	const enabled = buttonEnabled(simState);

	const handleStart = useCallback(async () => {
		try {
			const res = await api.start();
			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
		} catch (e) {
			console.error("Start failed:", e);
		}
	}, []);

	const handlePause = useCallback(async () => {
		try {
			const res = await api.pause();
			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
		} catch (e) {
			console.error("Pause failed:", e);
		}
	}, []);

	const handleStep = useCallback(async () => {
		try {
			const res = await api.step({ steps: 1 });
			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
		} catch (e) {
			console.error("Step failed:", e);
		}
	}, []);

	const handleRestart = useCallback(async () => {
		if (
			(simState === "running" || simState === "paused") &&
			!window.confirm("Restart the simulation with the current startup settings?")
		) {
			return;
		}

		setRestarting(true);
		try {
			const startup = useStartupConfigStore.getState().preset;
			const res = await api.startup(buildStartupRequest(startup));

			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
			useStatsHistoryStore.getState().reset();

			const configRes = await api.getConfig();
			useConfigStore.getState().commitServerConfig(configRes.config, configRes.state);
		} catch (e) {
			console.error("Restart failed:", e);
		} finally {
			setRestarting(false);
		}
	}, [simState]);

	return (
		<header className="flex items-center gap-3 px-4 h-12 bg-petri-panel border-b border-petri-border">
			{/* Brand */}
			<span className="font-semibold text-slate-200 tracking-tight text-sm mr-2">PETRI</span>

			{/* Divider */}
			<div className="w-px h-6 bg-petri-border" />

			{/* Lifecycle buttons */}
			<div className="flex items-center gap-1.5">
				<SimButton
					label="Start"
					testId="control-start"
					disabled={!enabled.start}
					active={simState === "running"}
					pulse={simState === "running"}
					onClick={handleStart}
				/>
				<SimButton
					label={restarting ? "Restarting..." : "Restart"}
					testId="control-restart"
					disabled={!enabled.restart || restarting}
					onClick={handleRestart}
				/>
				<SimButton
					label="Pause"
					testId="control-pause"
					disabled={!enabled.pause}
					onClick={handlePause}
				/>
				<SimButton
					label="Step"
					testId="control-step"
					disabled={!enabled.step}
					onClick={handleStep}
				/>
			</div>

			{/* Divider */}
			<div className="w-px h-6 bg-petri-border" />

			{/* Tick counter */}
			<span data-testid="tick-value" className="font-mono text-sm text-slate-300 tabular-nums">
				Tick: {tick.toLocaleString()}
			</span>

			{/* Population */}
			<span className="font-mono text-xs text-slate-400 tabular-nums">
				Pop: {population.toLocaleString()}
			</span>

			{/* TPS */}
			<span data-testid="tps-value" className="font-mono text-xs text-slate-300 tabular-nums">
				TPS: {formatTps(ticksPerSecond)}
			</span>

			{/* Spacer */}
			<div className="flex-1" />

			{/* Connection indicator */}
			<ConnectionDot status={connectionStatus} />

			{/* Panel toggles */}
			<button
				type="button"
				data-testid="toggle-config"
				onClick={toggleConfig}
				className="px-2 py-1 text-xs text-slate-400 hover:text-slate-200 bg-slate-800 rounded"
			>
				Config
			</button>
			<button
				type="button"
				data-testid="toggle-stats"
				onClick={toggleStats}
				className="px-2 py-1 text-xs text-slate-400 hover:text-slate-200 bg-slate-800 rounded"
			>
				Stats
			</button>
		</header>
	);
}

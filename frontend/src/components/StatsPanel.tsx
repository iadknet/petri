import { useCallback, useEffect, useRef, useState } from "react";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";

type Tab = "overview" | "actions" | "evolution";

function Gauge({
	label,
	value,
	delta,
	format,
}: { label: string; value: number; delta?: number; format?: (v: number) => string }) {
	const fmt = format ?? ((v: number) => v.toLocaleString());
	return (
		<div className="flex flex-col">
			<span className="text-[11px] font-sans text-slate-400">{label}</span>
			<div className="flex items-baseline gap-1.5">
				<span className="text-xl font-mono text-slate-100 tabular-nums">{fmt(value)}</span>
				{delta !== undefined && delta !== 0 && (
					<span
						className={`text-[11px] font-mono tabular-nums ${
							delta > 0 ? "text-emerald-400" : "text-red-400"
						}`}
					>
						{delta > 0 ? "+" : ""}
						{fmt(delta)}
					</span>
				)}
			</div>
		</div>
	);
}

function ActionBar({
	move,
	eat,
	reproduce,
	noop,
}: { move: number; eat: number; reproduce: number; noop: number }) {
	const total = move + eat + reproduce + noop || 1;
	const pct = (n: number) => `${((n / total) * 100).toFixed(1)}%`;

	return (
		<div className="flex flex-col gap-1">
			<span className="text-[11px] text-slate-400">Actions</span>
			<div className="flex h-3 rounded overflow-hidden">
				<div className="bg-blue-500" style={{ width: pct(move) }} title={`Move: ${move}`} />
				<div className="bg-emerald-500" style={{ width: pct(eat) }} title={`Eat: ${eat}`} />
				<div
					className="bg-amber-500"
					style={{ width: pct(reproduce) }}
					title={`Reproduce: ${reproduce}`}
				/>
				<div className="bg-slate-600" style={{ width: pct(noop) }} title={`Noop: ${noop}`} />
			</div>
			<div className="flex gap-3 text-[10px] text-slate-500">
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-blue-500 rounded-sm" />
					Move {move}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-emerald-500 rounded-sm" />
					Eat {eat}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-amber-500 rounded-sm" />
					Repro {reproduce}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-slate-600 rounded-sm" />
					Noop {noop}
				</span>
			</div>
		</div>
	);
}

function MiniChart({
	data,
	color,
	height = 60,
}: { data: { tick: number; value: number }[]; color: string; height?: number }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas || data.length < 2) return;

		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		const w = canvas.width;
		const h = canvas.height;
		ctx.clearRect(0, 0, w, h);

		const values = data.map((d) => d.value);
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min || 1;

		ctx.strokeStyle = color;
		ctx.lineWidth = 1.5;
		ctx.beginPath();

		for (let i = 0; i < data.length; i++) {
			const x = (i / (data.length - 1)) * w;
			const y = h - ((values[i]! - min) / range) * (h - 4) - 2;
			if (i === 0) ctx.moveTo(x, y);
			else ctx.lineTo(x, y);
		}
		ctx.stroke();
	}, [data, color]);

	return (
		<canvas
			ref={canvasRef}
			width={300}
			height={height}
			className="w-full"
			style={{ height: `${height}px` }}
		/>
	);
}

function OverviewTab() {
	const status = useSimulationStore((s) => s.status);
	const statsHistory = useStatsHistoryStore((s) => s.statsHistory);

	const pop = status?.population ?? 0;
	const energy = status?.mean_energy ?? 0;
	const actions = status?.last_tick_actions ?? { move: 0, eat: 0, reproduce: 0, noop: 0 };

	// Delta from 10 ticks ago
	const delta =
		statsHistory.length >= 10
			? pop - (statsHistory[statsHistory.length - 10]?.population ?? pop)
			: undefined;

	return (
		<div className="flex flex-col gap-3 p-3">
			<div className="flex gap-6">
				<Gauge label="Population" value={pop} delta={delta} />
				<Gauge label="Mean Energy" value={energy} format={(v) => v.toFixed(1)} />
			</div>
			<ActionBar {...actions} />
			<div>
				<span className="text-[11px] text-slate-400">Population Trend</span>
				<MiniChart
					data={statsHistory.map((s) => ({ tick: s.tick, value: s.population }))}
					color="#10b981"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Energy Trend</span>
				<MiniChart
					data={statsHistory.map((s) => ({ tick: s.tick, value: s.meanEnergy }))}
					color="#3b82f6"
				/>
			</div>
		</div>
	);
}

function ActionsTab() {
	const actionsHistory = useStatsHistoryStore((s) => s.actionsHistory);
	const reproAttempted = useStatsHistoryStore((s) => s.reproAttempted);
	const reproSpawned = useStatsHistoryStore((s) => s.reproSpawned);

	const successRate =
		reproAttempted > 0 ? ((reproSpawned / reproAttempted) * 100).toFixed(1) : "0.0";

	return (
		<div className="flex flex-col gap-3 p-3">
			<div>
				<span className="text-[11px] text-slate-400">Move Actions Over Time</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.move }))}
					color="#3b82f6"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Eat Actions Over Time</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.eat }))}
					color="#10b981"
				/>
			</div>
			<div className="flex gap-4">
				<Gauge label="Repro Attempted" value={reproAttempted} />
				<Gauge label="Repro Spawned" value={reproSpawned} />
				<Gauge label="Success Rate" value={Number(successRate)} format={(v) => `${v}%`} />
			</div>
		</div>
	);
}

function EvolutionTab() {
	const reproAttempted = useStatsHistoryStore((s) => s.reproAttempted);
	const reproSpawned = useStatsHistoryStore((s) => s.reproSpawned);
	const reproRejected = useStatsHistoryStore((s) => s.reproRejected);
	const reproRejectedByReason = useStatsHistoryStore((s) => s.reproRejectedByReason);
	const mutationAttempted = useStatsHistoryStore((s) => s.mutationAttempted);
	const mutationApplied = useStatsHistoryStore((s) => s.mutationApplied);
	const mutationSkipped = useStatsHistoryStore((s) => s.mutationSkipped);

	return (
		<div className="flex flex-col gap-3 p-3">
			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">Reproduction</span>
				<div className="flex gap-4">
					<Gauge label="Attempted" value={reproAttempted} />
					<Gauge label="Spawned" value={reproSpawned} />
					<Gauge label="Rejected" value={reproRejected} />
				</div>
				{Object.keys(reproRejectedByReason).length > 0 && (
					<div className="mt-1 pl-2 border-l border-slate-700">
						<span className="text-[11px] text-slate-500">Rejection Reasons</span>
						{Object.entries(reproRejectedByReason).map(([reason, count]) => (
							<div key={reason} className="flex justify-between text-[11px]">
								<span className="text-slate-400">{reason}</span>
								<span className="font-mono text-slate-300">{count.toLocaleString()}</span>
							</div>
						))}
					</div>
				)}
			</div>

			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">Mutations</span>
				<div className="flex gap-4">
					<Gauge label="Attempted" value={mutationAttempted} />
					<Gauge label="Applied" value={mutationApplied} />
					<Gauge label="Skipped" value={mutationSkipped} />
				</div>
			</div>
		</div>
	);
}

export function StatsPanel() {
	const [tab, setTab] = useState<Tab>("overview");

	const tabButton = useCallback(
		(t: Tab, label: string, testId: string) => (
			<button
				type="button"
				data-testid={testId}
				onClick={() => setTab(t)}
				className={`px-3 py-1 text-xs font-medium rounded-t ${
					tab === t ? "bg-petri-panel text-slate-200" : "text-slate-500 hover:text-slate-300"
				}`}
			>
				{label}
			</button>
		),
		[tab],
	);

	return (
		<div
			data-testid="stats-panel"
			className="bg-petri-panel border-t border-petri-border flex flex-col"
			style={{ height: "280px" }}
		>
			{/* Tab bar */}
			<div className="flex gap-1 px-3 pt-1 bg-slate-950">
				{tabButton("overview", "Overview", "stats-tab-overview")}
				{tabButton("actions", "Actions", "stats-tab-actions")}
				{tabButton("evolution", "Evolution", "stats-tab-evolution")}
			</div>

			{/* Tab content */}
			<div className="flex-1 overflow-y-auto">
				{tab === "overview" && <OverviewTab />}
				{tab === "actions" && <ActionsTab />}
				{tab === "evolution" && <EvolutionTab />}
			</div>
		</div>
	);
}

import { memo, useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { ActionResult, ActionType } from "../../types/action-log.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";

const ACTION_COLORS: Record<ActionType, string> = {
	[ActionType.Move]: "#60a5fa",
	[ActionType.Eat]: "#34d399",
	[ActionType.Reproduce]: "#fbbf24",
	[ActionType.StealEnergy]: "#f87171",
	[ActionType.NoOp]: "#475569",
};

const ACTION_NAMES: Record<ActionType, string> = {
	[ActionType.Move]: "Move",
	[ActionType.Eat]: "Eat",
	[ActionType.Reproduce]: "Reproduce",
	[ActionType.StealEnergy]: "Steal Energy",
	[ActionType.NoOp]: "No-Op",
};

const RESULT_NAMES: Record<ActionResult, string> = {
	[ActionResult.Success]: "Success",
	[ActionResult.NoFood]: "No Food",
	[ActionResult.Blocked]: "Blocked",
	[ActionResult.InvalidTarget]: "Invalid Target",
	[ActionResult.EnergyConstraints]: "Energy Constraints",
	[ActionResult.PopulationCap]: "Population Cap",
	[ActionResult.TransferredAndKilled]: "Transferred & Killed",
	[ActionResult.NoVictim]: "No Victim",
};

const DIRECTIONS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;

export function directionLabel(dir: number): string {
	if (dir === 255) return "N/A";
	return DIRECTIONS[dir] ?? `${dir}`;
}

const SEGMENT_WIDTH = 6;
const TICK_LABEL_INTERVAL = 50;

interface ActionTimelineProps {
	actionLog: ActionLogEntry[];
	maxEnergy: number;
}

const ENERGY_OVERLAY_HEIGHT = 20; // matches TimelineBar height

export const ActionTimeline = memo(function ActionTimeline({
	actionLog,
	maxEnergy,
}: ActionTimelineProps) {
	const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
	const scrollRef = useRef<HTMLDivElement>(null);
	const isFollowingRef = useRef(true);

	// Reset selection when the log data changes (new fetch cycle)
	useEffect(() => {
		setSelectedIndex(null);
	}, [actionLog]);

	// "Following" auto-scroll: only scroll to right edge when user is already there
	useLayoutEffect(() => {
		const el = scrollRef.current;
		if (el && isFollowingRef.current) {
			el.scrollLeft = el.scrollWidth - el.clientWidth;
		}
	}, [actionLog]);

	const handleScroll = useCallback(() => {
		const el = scrollRef.current;
		if (!el) return;
		// Consider "at right edge" if within 2px tolerance
		const atEnd = el.scrollLeft + el.clientWidth >= el.scrollWidth - 2;
		isFollowingRef.current = atEnd;
	}, []);

	const selectedEntry = selectedIndex !== null ? actionLog[selectedIndex] ?? null : null;
	const totalWidth = actionLog.length * SEGMENT_WIDTH;

	return (
		<div className="px-4 py-3 space-y-2">
			<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">
				Action Timeline
			</span>
			<div
				ref={scrollRef}
				className="overflow-x-auto"
				onScroll={handleScroll}
			>
				<div style={{ width: totalWidth, minWidth: "100%" }}>
					<TickAxis entries={actionLog} />
					<div className="relative">
						<TimelineBar
							entries={actionLog}
							selectedIndex={selectedIndex}
							onSelect={setSelectedIndex}
						/>
						<EnergyOverlay entries={actionLog} maxEnergy={maxEnergy} />
					</div>
				</div>
			</div>
			{selectedEntry ? <ActionDetail entry={selectedEntry} /> : null}
		</div>
	);
});

export function tickLabels(entries: ActionLogEntry[]): { tick: number; offsetPx: number }[] {
	const labels: { tick: number; offsetPx: number }[] = [];
	for (let i = 0; i < entries.length; i++) {
		const entry = entries[i];
		if (entry && entry.tick % TICK_LABEL_INTERVAL === 0) {
			labels.push({ tick: entry.tick, offsetPx: i * SEGMENT_WIDTH });
		}
	}
	return labels;
}

function TickAxis({ entries }: { entries: ActionLogEntry[] }) {
	const labels = tickLabels(entries);

	return (
		<div className="relative h-3 text-[9px] text-slate-500 font-mono select-none" aria-hidden="true">
			{labels.map((label) => (
				<span
					key={label.tick}
					className="absolute whitespace-nowrap"
					style={{ left: label.offsetPx }}
				>
					{label.tick}
				</span>
			))}
		</div>
	);
}

function TimelineBar({
	entries,
	selectedIndex,
	onSelect,
}: {
	entries: ActionLogEntry[];
	selectedIndex: number | null;
	onSelect: (index: number | null) => void;
}) {
	return (
		<div
			className="flex"
			style={{ minHeight: 20 }}
			role="toolbar"
			aria-label="Action timeline"
		>
			{entries.map((entry, i) => {
				const color = ACTION_COLORS[entry.action_type] ?? ACTION_COLORS[ActionType.NoOp];
				const failed = entry.result !== ActionResult.Success;
				const isSelected = i === selectedIndex;
				return (
					<div
						key={i}
						role="button"
						tabIndex={0}
						aria-label={`Tick ${entry.tick}: ${ACTION_NAMES[entry.action_type] ?? "Unknown"}`}
						onClick={() => onSelect(isSelected ? null : i)}
						onKeyDown={(e) => {
							if (e.key === "Enter" || e.key === " ") {
								e.preventDefault();
								onSelect(isSelected ? null : i);
							}
						}}
						style={{
							width: SEGMENT_WIDTH,
							minWidth: SEGMENT_WIDTH,
							height: 20,
							backgroundColor: color,
							borderTop: failed ? "2px solid #ef4444" : undefined,
							opacity: isSelected ? 1 : selectedIndex !== null ? 0.6 : 1,
							cursor: "pointer",
						}}
						title={`Tick ${entry.tick}`}
					/>
				);
			})}
		</div>
	);
}

export function energyPath(entries: ActionLogEntry[], maxEnergy: number, height: number): string {
	if (entries.length === 0 || maxEnergy <= 0) return "";
	const points: string[] = [];
	for (let i = 0; i < entries.length; i++) {
		const entry = entries[i];
		if (!entry) continue;
		const x = (i * SEGMENT_WIDTH + SEGMENT_WIDTH / 2).toFixed(1);
		const ratio = Math.min(entry.energy_after / maxEnergy, 1);
		const y = (height * (1 - ratio)).toFixed(1);
		points.push(`${x},${y}`);
	}
	if (points.length === 0) return "";
	return `M${points.join("L")}`;
}

function EnergyOverlay({ entries, maxEnergy }: { entries: ActionLogEntry[]; maxEnergy: number }) {
	const totalWidth = entries.length * SEGMENT_WIDTH;
	const d = energyPath(entries, maxEnergy, ENERGY_OVERLAY_HEIGHT);
	if (!d) return null;

	return (
		<svg
			className="absolute inset-0 pointer-events-none"
			width={totalWidth}
			height={ENERGY_OVERLAY_HEIGHT}
			aria-hidden="true"
			data-testid="energy-overlay"
		>
			<path d={d} fill="none" stroke="#facc15" strokeWidth={1.5} strokeOpacity={0.6} />
		</svg>
	);
}

function ActionDetail({ entry }: { entry: ActionLogEntry }) {
	const actionName = ACTION_NAMES[entry.action_type] ?? "Unknown";
	const resultName = RESULT_NAMES[entry.result] ?? "Unknown";
	const dir = directionLabel(entry.direction);
	const delta = entry.energy_after - entry.energy_before;
	const deltaSign = delta >= 0 ? "+" : "";
	const deltaColor = delta >= 0 ? "text-emerald-400" : "text-red-400";

	return (
		<div className="rounded bg-slate-800 p-2 text-xs space-y-1" data-testid="action-detail" role="region" aria-label={`Details for tick ${entry.tick}`}>
			<div className="flex justify-between">
				<span className="text-slate-400">Tick</span>
				<span className="text-slate-200 font-mono">{entry.tick}</span>
			</div>
			<div className="flex justify-between">
				<span className="text-slate-400">Action</span>
				<span className="text-slate-200">
					{actionName}{" "}
					<span className={entry.result === ActionResult.Success ? "text-emerald-400" : "text-red-400"}>
						({resultName})
					</span>
				</span>
			</div>
			<div className="flex justify-between">
				<span className="text-slate-400">Direction</span>
				<span className="text-slate-200 font-mono">{dir}</span>
			</div>
			<div className="flex justify-between">
				<span className="text-slate-400">Energy</span>
				<span className="text-slate-200 font-mono">
					{entry.energy_before.toFixed(1)} → {entry.energy_after.toFixed(1)}{" "}
					<span className={deltaColor}>
						({deltaSign}{delta.toFixed(1)})
					</span>
				</span>
			</div>
			{entry.amount !== 0 ? (
				<div className="flex justify-between">
					<span className="text-slate-400">Amount</span>
					<span className="text-slate-200 font-mono">{entry.amount.toFixed(1)}</span>
				</div>
			) : null}
			<div className="flex justify-between">
				<span className="text-slate-400">Priority</span>
				<span className="text-slate-200 font-mono">{entry.priority_bid.toFixed(2)}</span>
			</div>
		</div>
	);
}

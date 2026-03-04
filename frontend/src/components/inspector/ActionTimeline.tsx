import { memo, useState } from "react";
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

interface ActionTimelineProps {
	actionLog: ActionLogEntry[];
	maxEnergy: number;
}

export const ActionTimeline = memo(function ActionTimeline({
	actionLog,
}: ActionTimelineProps) {
	const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
	const selectedEntry = selectedIndex !== null ? actionLog[selectedIndex] ?? null : null;

	return (
		<div className="px-4 py-3 space-y-2">
			<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">
				Action Timeline
			</span>
			<TimelineBar
				entries={actionLog}
				selectedIndex={selectedIndex}
				onSelect={setSelectedIndex}
			/>
			{selectedEntry ? <ActionDetail entry={selectedEntry} /> : null}
		</div>
	);
});

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
			className="flex overflow-x-auto"
			style={{ minHeight: 20 }}
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

function ActionDetail({ entry }: { entry: ActionLogEntry }) {
	const actionName = ACTION_NAMES[entry.action_type] ?? "Unknown";
	const resultName = RESULT_NAMES[entry.result] ?? "Unknown";
	const dir = directionLabel(entry.direction);
	const delta = entry.energy_after - entry.energy_before;
	const deltaSign = delta >= 0 ? "+" : "";
	const deltaColor = delta >= 0 ? "text-emerald-400" : "text-red-400";

	return (
		<div className="rounded bg-slate-800 p-2 text-xs space-y-1" data-testid="action-detail">
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

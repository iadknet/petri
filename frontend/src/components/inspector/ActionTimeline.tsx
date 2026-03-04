import { memo } from "react";
import { ActionResult, ActionType } from "../../types/action-log.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";

const ACTION_COLORS: Record<ActionType, string> = {
	[ActionType.Move]: "#60a5fa",
	[ActionType.Eat]: "#34d399",
	[ActionType.Reproduce]: "#fbbf24",
	[ActionType.StealEnergy]: "#f87171",
	[ActionType.NoOp]: "#475569",
};

const SEGMENT_WIDTH = 6;

interface ActionTimelineProps {
	actionLog: ActionLogEntry[];
	maxEnergy: number;
}

export const ActionTimeline = memo(function ActionTimeline({
	actionLog,
}: ActionTimelineProps) {
	return (
		<div className="px-4 py-3 space-y-2">
			<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">
				Action Timeline
			</span>
			<TimelineBar entries={actionLog} />
		</div>
	);
});

function TimelineBar({ entries }: { entries: ActionLogEntry[] }) {
	return (
		<div
			className="flex overflow-x-auto"
			style={{ minHeight: 20 }}
		>
			{entries.map((entry, i) => {
				const color = ACTION_COLORS[entry.action_type] ?? ACTION_COLORS[ActionType.NoOp];
				const failed = entry.result !== ActionResult.Success;
				return (
					<div
						key={i}
						style={{
							width: SEGMENT_WIDTH,
							minWidth: SEGMENT_WIDTH,
							height: 20,
							backgroundColor: color,
							borderTop: failed ? "2px solid #ef4444" : undefined,
						}}
						title={`Tick ${entry.tick}`}
					/>
				);
			})}
		</div>
	);
}

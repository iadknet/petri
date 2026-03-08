import { Handle, type NodeProps, Position } from "@xyflow/react";
import { memo } from "react";

export interface GraphInternalsNodeData extends Record<string, unknown> {
	index: number;
	kindLabel: string;
	category: "input" | "constant" | "memory" | "processing" | "output";
	isLive: boolean;
	/** Output value from first pass, null when no trace active */
	initialValue: number | null;
	/** Output value from current pass, null when no trace active */
	outputValue: number | null;
	/** State change string for stateful nodes, e.g. "0.30→0.51" */
	stateChange: string | null;
	/** For RouterOutput: mesh target node IDs, e.g. [3, 5, 7] */
	routeTargets: number[] | null;
	/** For RouterOutput during execution: which target was selected */
	selectedTarget: number | null;
}

const CATEGORY_COLORS = {
	input: { r: 52, g: 211, b: 153, bgAlpha: 0.15, borderAlpha: 0.4, text: "#6ee7b7" },
	constant: { r: 56, g: 189, b: 248, bgAlpha: 0.12, borderAlpha: 0.35, text: "#7dd3fc" },
	memory: { r: 167, g: 139, b: 250, bgAlpha: 0.12, borderAlpha: 0.35, text: "#c4b5fd" },
	processing: { r: 148, g: 163, b: 184, bgAlpha: 0.12, borderAlpha: 0.3, text: "#cbd5e1" },
	output: { r: 251, g: 191, b: 36, bgAlpha: 0.15, borderAlpha: 0.4, text: "#fcd34d" },
};

export const GraphInternalsNode = memo(function GraphInternalsNode(
	props: NodeProps,
) {
	const data = props.data as GraphInternalsNodeData;
	const c = CATEGORY_COLORS[data.category];

	// Modulate category color intensity by output value during execution
	const hasTrace = data.outputValue !== null;
	const intensity = hasTrace
		? Math.min(Math.abs(data.outputValue ?? 0), 1)
		: 0;

	const bgAlpha = hasTrace ? c.bgAlpha + intensity * 0.25 : c.bgAlpha;
	const borderAlpha = hasTrace ? c.borderAlpha + intensity * 0.3 : c.borderAlpha;
	const bgColor = `rgba(${c.r},${c.g},${c.b},${bgAlpha})`;
	const borderColor = `rgba(${c.r},${c.g},${c.b},${borderAlpha})`;

	const hasRouteTargets = data.routeTargets && data.routeTargets.length > 0;

	return (
		<div
			className={`flex flex-col justify-center h-full rounded px-1.5 text-[9px] font-mono leading-none transition-all ${
				data.isLive ? "" : "opacity-35"
			}`}
			style={{
				backgroundColor: bgColor,
				border: `1px solid ${borderColor}`,
			}}
		>
			<Handle
				type="target"
				position={Position.Left}
				isConnectable={false}
				className="!h-1.5 !w-1.5 !border-0 !bg-transparent !opacity-0"
			/>
			<Handle
				type="source"
				position={Position.Right}
				isConnectable={false}
				className="!h-1.5 !w-1.5 !border-0 !bg-transparent !opacity-0"
			/>
			<Handle
				type="source"
				position={Position.Bottom}
				id="bottom-out"
				isConnectable={false}
				className="!h-1.5 !w-1.5 !border-0 !bg-transparent !opacity-0"
			/>
			<Handle
				type="target"
				position={Position.Bottom}
				id="bottom-in"
				isConnectable={false}
				className="!h-1.5 !w-1.5 !border-0 !bg-transparent !opacity-0"
			/>
			<div className="flex items-center gap-1 min-w-0 overflow-hidden">
				<span className="text-slate-500 shrink-0">{data.index}</span>
				<span
					className="truncate"
					style={{ color: c.text }}
				>
					{data.kindLabel}
				</span>
				{hasTrace && data.outputValue !== null ? (() => {
					const initial = data.initialValue ?? data.outputValue;
					const changed = Math.abs(initial - data.outputValue) > 0.005;
					return changed ? (
						<>
							<span className="shrink-0 text-[8px] text-slate-500">
								{initial.toFixed(2)}
							</span>
							<span className="shrink-0 text-slate-600">→</span>
							<span className="shrink-0 text-[9px] text-cyan-300">
								{data.outputValue.toFixed(2)}
							</span>
						</>
					) : (
						<>
							<span className="shrink-0 text-slate-600">=</span>
							<span className="shrink-0 text-[9px] text-cyan-300">
								{data.outputValue.toFixed(2)}
							</span>
						</>
					);
				})() : null}
				{data.stateChange ? (
					<span className="shrink-0 text-[8px] text-amber-400/80">
						{data.stateChange}
					</span>
				) : null}
			</div>
			{hasRouteTargets ? (
				<div className="flex items-center gap-0.5 mt-px text-[8px] overflow-hidden">
					<span className="text-slate-600 shrink-0">→</span>
					{data.routeTargets!.map((id, i) => (
						<span
							key={id}
							className={`shrink-0 ${
								data.selectedTarget === id
									? "text-cyan-300"
									: "text-slate-600"
							}`}
						>
							#{id}{i < data.routeTargets!.length - 1 ? "," : ""}
						</span>
					))}
				</div>
			) : null}
		</div>
	);
});

import { Handle, type NodeProps, Position } from "@xyflow/react";
import { memo } from "react";
import type { NodeCategory } from "./graphNodeCategories.ts";

export interface GraphInternalsNodeData extends Record<string, unknown> {
	index: number;
	kindLabel: string;
	subtitle?: string;
	category: NodeCategory;
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
	// Output sink trace
	weightedSum: number | null;
	appliedValue: number | null;
	applied: boolean | null;
	// Action slot trace
	fired: boolean | null;
	emittedAction: string | null;
	queueDelta: string | null;
	// Execute gate trace
	gateFired: boolean | null;
	queueNonEmpty: boolean | null;
}

const CATEGORY_COLORS: Record<
	NodeCategory,
	{ r: number; g: number; b: number; bgAlpha: number; borderAlpha: number; text: string }
> = {
	input: { r: 16, g: 185, b: 129, bgAlpha: 0.15, borderAlpha: 0.4, text: "#6ee7b7" },
	arithmetic: { r: 148, g: 163, b: 184, bgAlpha: 0.12, borderAlpha: 0.3, text: "#cbd5e1" },
	activation: { r: 56, g: 189, b: 248, bgAlpha: 0.12, borderAlpha: 0.35, text: "#7dd3fc" },
	logic: { r: 167, g: 139, b: 250, bgAlpha: 0.12, borderAlpha: 0.35, text: "#c4b5fd" },
	stateful: { r: 196, g: 130, b: 250, bgAlpha: 0.12, borderAlpha: 0.35, text: "#d8b4fe" },
	constant: { r: 100, g: 116, b: 139, bgAlpha: 0.12, borderAlpha: 0.35, text: "#94a3b8" },
	output_value: { r: 251, g: 191, b: 36, bgAlpha: 0.15, borderAlpha: 0.4, text: "#fcd34d" },
	output_action: { r: 249, g: 115, b: 22, bgAlpha: 0.15, borderAlpha: 0.4, text: "#fdba74" },
	output_gate: { r: 239, g: 68, b: 68, bgAlpha: 0.15, borderAlpha: 0.4, text: "#fca5a5" },
};

export const GraphInternalsNode = memo(function GraphInternalsNode(props: NodeProps) {
	const data = props.data as GraphInternalsNodeData;
	const c = CATEGORY_COLORS[data.category];

	// Modulate category color intensity by output value during execution
	const hasComputeTrace = data.outputValue !== null;
	const intensity = hasComputeTrace ? Math.min(Math.abs(data.outputValue ?? 0), 1) : 0;

	const bgAlpha = hasComputeTrace ? c.bgAlpha + intensity * 0.25 : c.bgAlpha;
	const borderAlpha = hasComputeTrace ? c.borderAlpha + intensity * 0.3 : c.borderAlpha;
	const bgColor = `rgba(${c.r},${c.g},${c.b},${bgAlpha})`;
	const borderColor = `rgba(${c.r},${c.g},${c.b},${borderAlpha})`;

	const isDashed = data.category === "constant";
	const isStateful = data.category === "stateful";
	const hasRouteTargets = data.routeTargets && data.routeTargets.length > 0;

	return (
		<div
			className={`flex flex-col justify-center h-full rounded px-1.5 text-[9px] font-mono leading-none transition-all ${
				data.isLive ? "" : "opacity-35"
			}`}
			style={{
				backgroundColor: bgColor,
				border: `1px ${isDashed ? "dashed" : "solid"} ${borderColor}`,
				boxShadow: isStateful ? `inset 0 0 0 2px rgba(${c.r},${c.g},${c.b},0.3)` : undefined,
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
				{data.index >= 0 ? <span className="text-slate-500 shrink-0">{data.index}</span> : null}
				<span className="truncate" style={{ color: c.text }}>
					{data.kindLabel}
				</span>
				{hasComputeTrace && data.outputValue !== null
					? (() => {
							const initial = data.initialValue ?? data.outputValue;
							const changed = Math.abs(initial - data.outputValue) > 0.005;
							return changed ? (
								<>
									<span className="shrink-0 text-[8px] text-slate-500">{initial.toFixed(2)}</span>
									<span className="shrink-0 text-slate-600">{"\u2192"}</span>
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
						})()
					: null}
				{data.stateChange ? (
					<span className="shrink-0 text-[8px] text-amber-400/80">{data.stateChange}</span>
				) : null}
				{/* Output sink trace */}
				{data.weightedSum !== null ? (
					<span className="shrink-0 text-[8px] text-cyan-300">
						{"\u03A3"}={data.weightedSum.toFixed(2)}
					</span>
				) : null}
				{/* Action slot trace */}
				{data.fired !== null ? (
					<span
						className={`shrink-0 text-[8px] ${data.fired ? "text-orange-300" : "text-slate-600"}`}
					>
						{data.fired ? (data.emittedAction ?? "fired") : "\u2014"}
					</span>
				) : null}
				{/* Execute gate trace */}
				{data.gateFired !== null ? (
					<span
						className={`shrink-0 text-[8px] ${data.gateFired ? "text-red-300" : "text-slate-600"}`}
					>
						{data.gateFired ? "fired" : "\u2014"}
					</span>
				) : null}
			</div>
			{data.subtitle ? (
				<div className="text-[8px] text-slate-600 truncate mt-px">{data.subtitle}</div>
			) : null}
			{hasRouteTargets ? (
				<div className="flex items-center gap-0.5 mt-px text-[8px] overflow-hidden">
					<span className="text-slate-600 shrink-0">{"\u2192"}</span>
					{data.routeTargets!.map((id, i) => (
						<span
							key={id}
							className={`shrink-0 ${
								data.selectedTarget === id ? "text-cyan-300" : "text-slate-600"
							}`}
						>
							#{id}
							{i < data.routeTargets!.length - 1 ? "," : ""}
						</span>
					))}
				</div>
			) : null}
		</div>
	);
});

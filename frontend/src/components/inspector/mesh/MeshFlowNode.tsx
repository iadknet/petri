import { Handle, type NodeProps, Position } from "@xyflow/react";
import { memo } from "react";
import {
	MESH_FLOW_SOURCE_HANDLE_ID,
	MESH_FLOW_TARGET_HANDLE_ID,
	type MeshFlowNodeData,
} from "./meshFlowAdapter.ts";

export const MeshFlowNode = memo(function MeshFlowNode(props: NodeProps) {
	const data = props.data as MeshFlowNodeData;
	const selected = data.isSelected || props.selected;

	return (
		<div
			data-testid={`mesh-node-${data.nodeId}`}
			data-dimmed={data.dimmed ? "true" : "false"}
			data-selected={selected ? "true" : "false"}
			className={`group relative h-full w-full rounded-lg border transition-all cursor-pointer ${
				selected
					? "border-cyan-400/60 bg-slate-900"
					: data.region === "unreachable"
						? "border-slate-700 bg-slate-900/60"
						: "border-white/10 bg-slate-900/90"
			} ${data.dimmed ? "opacity-40" : ""}`}
		>
			<Handle
				type="target"
				id={MESH_FLOW_TARGET_HANDLE_ID}
				position={Position.Left}
				isConnectable={false}
				className="!h-2 !w-2 !border-0 !bg-transparent !opacity-0"
			/>
			<Handle
				type="source"
				id={MESH_FLOW_SOURCE_HANDLE_ID}
				position={Position.Right}
				isConnectable={false}
				className="!h-2 !w-2 !border-0 !bg-transparent !opacity-0"
			/>
			{data.isEntry ? (
				<div
					className="pointer-events-none absolute inset-0 rounded-lg"
					style={{ boxShadow: "inset 0 0 0 1px rgba(52,211,153,0.4)" }}
				/>
			) : null}
			{data.active ? (
				<div className="pointer-events-none absolute inset-[-4px] rounded-[10px] border border-amber-400/60 animate-[pulse_1.5s_ease-in-out_infinite] shadow-[0_0_12px_rgba(250,204,21,0.25)]" />
			) : null}
			{data.hasSharedMemory ? (
				<div
					className="absolute -top-1 -right-1 h-2 w-2 rounded-full bg-violet-400 border border-slate-900"
					title="Shared memory"
				/>
			) : null}
			<div className="flex h-full flex-col justify-center gap-0.5 px-3 py-2">
				<div className="flex items-baseline gap-1.5 overflow-hidden whitespace-nowrap">
					<span className="text-[10px] font-mono text-slate-500">#{data.nodeId}</span>
					<span className="text-[10px] font-mono uppercase" style={{ color: data.backendMuted }}>
						{data.backendLabel}
					</span>
					{data.isEntry ? (
						<span className="text-[9px] font-mono text-emerald-400">entry</span>
					) : null}
					<span className="text-[9px] font-mono text-slate-600">{data.summaryDetail}</span>
				</div>
				<p className="text-xs font-medium text-slate-100 leading-tight line-clamp-2">
					{data.shortLabel}
				</p>
				{data.inputPreview ? (
					<p className="text-[9px] font-mono text-slate-500 line-clamp-1 mt-0.5">
						{data.inputPreview}
					</p>
				) : null}
			</div>
		</div>
	);
});

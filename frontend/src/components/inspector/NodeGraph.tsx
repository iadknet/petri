import { memo, useCallback, useMemo, useState } from "react";
import type { BackendDef, CreatureGenome, GraphNodeKind, NodeGenome } from "../../types/api.ts";
import { formatInputRef, inputRefColor } from "./inputRefUtils.ts";

interface NodeGraphProps {
	genome: CreatureGenome;
	activeNodeId?: number | null;
}

// Layout constants
const SVG_PADDING = 16;
const NODE_WIDTH = 120;
const NODE_HEIGHT = 56;
const NODE_GAP_X = 40;
const NODE_GAP_Y = 32;
const INPUT_DOT_R = 3;

// Colors
const GRAPH_NODE_COLOR = "#8b5cf6"; // purple
const VM_NODE_COLOR = "#f59e0b"; // amber
const ENTRY_RING_COLOR = "#34d399"; // emerald

interface LayoutNode {
	nodeGenome: NodeGenome;
	x: number;
	y: number;
	col: number;
	row: number;
	isEntry: boolean;
}

interface Connection {
	fromId: number;
	toId: number;
}

function computeLayout(genome: CreatureGenome): {
	nodes: LayoutNode[];
	connections: Connection[];
	width: number;
	height: number;
} {
	const nodeMap = new Map<number, NodeGenome>();
	for (const n of genome.nodes) {
		nodeMap.set(n.node_id, n);
	}

	// BFS from entry node to assign columns (depth levels)
	const visited = new Set<number>();
	const depthMap = new Map<number, number>();
	const queue: { id: number; depth: number }[] = [{ id: genome.entry_node_id, depth: 0 }];
	visited.add(genome.entry_node_id);

	while (queue.length > 0) {
		const { id, depth } = queue.shift()!;
		depthMap.set(id, depth);

		const node = nodeMap.get(id);
		if (!node) continue;

		for (const targetId of node.targets) {
			if (!visited.has(targetId)) {
				visited.add(targetId);
				queue.push({ id: targetId, depth: depth + 1 });
			}
		}
	}

	// Any unvisited nodes get assigned to the last depth + 1
	const maxDepth = Math.max(0, ...depthMap.values());
	for (const n of genome.nodes) {
		if (!depthMap.has(n.node_id)) {
			depthMap.set(n.node_id, maxDepth + 1);
		}
	}

	// Group by column
	const columns = new Map<number, NodeGenome[]>();
	for (const n of genome.nodes) {
		const col = depthMap.get(n.node_id) ?? 0;
		const existing = columns.get(col);
		if (existing) {
			existing.push(n);
		} else {
			columns.set(col, [n]);
		}
	}

	const layoutNodes: LayoutNode[] = [];
	const totalCols = Math.max(0, ...columns.keys()) + 1;

	for (let col = 0; col < totalCols; col++) {
		const nodesInCol = columns.get(col) ?? [];
		for (let row = 0; row < nodesInCol.length; row++) {
			const ng = nodesInCol[row];
			if (!ng) continue;
			layoutNodes.push({
				nodeGenome: ng,
				x: SVG_PADDING + col * (NODE_WIDTH + NODE_GAP_X),
				y: SVG_PADDING + row * (NODE_HEIGHT + NODE_GAP_Y),
				col,
				row,
				isEntry: ng.node_id === genome.entry_node_id,
			});
		}
	}

	// Collect connections (target routing)
	const connections: Connection[] = [];
	for (const n of genome.nodes) {
		for (const targetId of n.targets) {
			connections.push({ fromId: n.node_id, toId: targetId });
		}
	}

	const maxRow = layoutNodes.length > 0 ? Math.max(...layoutNodes.map((n) => n.row)) : 0;

	return {
		nodes: layoutNodes,
		connections,
		width: SVG_PADDING * 2 + totalCols * NODE_WIDTH + (totalCols - 1) * NODE_GAP_X,
		height: SVG_PADDING * 2 + (maxRow + 1) * NODE_HEIGHT + maxRow * NODE_GAP_Y,
	};
}

function getBackendLabel(def: BackendDef): { type: string; detail: string } {
	if ("Vm" in def) {
		const vm = def.Vm;
		return {
			type: "VM",
			detail: `${vm.program.length} ops, ${vm.register_count} regs`,
		};
	}
	const graph = def.Graph;
	return {
		type: "Graph",
		detail: `${graph.internal_nodes.length} nodes`,
	};
}

function getNodeColor(def: BackendDef): string {
	return "Vm" in def ? VM_NODE_COLOR : GRAPH_NODE_COLOR;
}

function getGraphNodeKindLabel(kind: GraphNodeKind): string {
	if (typeof kind === "string") return kind;
	return Object.keys(kind)[0] ?? "?";
}

export const NodeGraph = memo(function NodeGraph({ genome, activeNodeId }: NodeGraphProps) {
	const [hoveredNodeId, setHoveredNodeId] = useState<number | null>(null);

	const layout = useMemo(() => computeLayout(genome), [genome]);
	const nodePositions = useMemo(() => {
		const map = new Map<number, LayoutNode>();
		for (const n of layout.nodes) {
			map.set(n.nodeGenome.node_id, n);
		}
		return map;
	}, [layout]);

	const handleNodeHover = useCallback((id: number) => {
		setHoveredNodeId(id);
	}, []);
	const handleNodeLeave = useCallback(() => {
		setHoveredNodeId(null);
	}, []);

	const hoveredNode = hoveredNodeId !== null ? nodePositions.get(hoveredNodeId) : null;

	return (
		<div className="px-4 py-3 space-y-2">
			<div className="flex items-baseline justify-between">
				<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">
					Genome Mesh
				</span>
				<span className="text-[10px] text-slate-600 font-mono">
					{genome.nodes.length} node{genome.nodes.length !== 1 ? "s" : ""}
				</span>
			</div>

			<div className="overflow-x-auto rounded bg-slate-950/50 border border-slate-800/50">
				<svg
					width={Math.max(layout.width, 200)}
					height={Math.max(layout.height, 80)}
					className="block"
					aria-hidden="true"
				>
					{/* Connections */}
					{layout.connections.map((conn) => {
						const from = nodePositions.get(conn.fromId);
						const to = nodePositions.get(conn.toId);
						if (!from || !to) return null;

						const x1 = from.x + NODE_WIDTH;
						const y1 = from.y + NODE_HEIGHT / 2;
						const x2 = to.x;
						const y2 = to.y + NODE_HEIGHT / 2;
						const cpx = (x1 + x2) / 2;

						return (
							<path
								key={`${conn.fromId}-${conn.toId}`}
								d={`M${x1},${y1} C${cpx},${y1} ${cpx},${y2} ${x2},${y2}`}
								fill="none"
								stroke="#475569"
								strokeWidth={1.5}
								strokeDasharray={from.col === to.col ? "4 3" : undefined}
								opacity={0.6}
							/>
						);
					})}

					{/* Nodes */}
					{layout.nodes.map((ln) => {
						const color = getNodeColor(ln.nodeGenome.backend_def);
						const backend = getBackendLabel(ln.nodeGenome.backend_def);

						return (
							<g
								key={ln.nodeGenome.node_id}
								onMouseEnter={() => handleNodeHover(ln.nodeGenome.node_id)}
								onMouseLeave={handleNodeLeave}
								className="cursor-default"
							>
								{/* Entry node ring */}
								{ln.isEntry && (
									<rect
										x={ln.x - 3}
										y={ln.y - 3}
										width={NODE_WIDTH + 6}
										height={NODE_HEIGHT + 6}
										rx={10}
										ry={10}
										fill="none"
										stroke={ENTRY_RING_COLOR}
										strokeWidth={2}
										opacity={0.6}
									/>
								)}

								{/* Active node glow */}
								{activeNodeId != null && activeNodeId === ln.nodeGenome.node_id && (
									<rect
										x={ln.x - 4}
										y={ln.y - 4}
										width={NODE_WIDTH + 8}
										height={NODE_HEIGHT + 8}
										rx={12}
										ry={12}
										fill="none"
										stroke="#38bdf8"
										strokeWidth={2}
										style={{ opacity: 0.7, transition: "opacity 0.3s" }}
									/>
								)}

								{/* Node body */}
								<rect
									x={ln.x}
									y={ln.y}
									width={NODE_WIDTH}
									height={NODE_HEIGHT}
									rx={8}
									ry={8}
									fill={`${color}18`}
									stroke={color}
									strokeWidth={1.2}
									opacity={
										activeNodeId != null
											? activeNodeId === ln.nodeGenome.node_id
												? 1
												: 0.3
											: hoveredNodeId !== null && hoveredNodeId !== ln.nodeGenome.node_id
												? 0.4
												: 1
									}
								/>

								{/* Node ID */}
								<text
									x={ln.x + 8}
									y={ln.y + 18}
									fontSize={11}
									fontFamily="ui-monospace, monospace"
									fontWeight={600}
									fill="#e2e8f0"
								>
									Node {ln.nodeGenome.node_id}
								</text>

								{/* Backend type badge */}
								<text
									x={ln.x + 8}
									y={ln.y + 34}
									fontSize={9}
									fontFamily="ui-monospace, monospace"
									fill={color}
								>
									{backend.type}
								</text>

								{/* Detail */}
								<text
									x={ln.x + 8}
									y={ln.y + 46}
									fontSize={8}
									fontFamily="ui-monospace, monospace"
									fill="#64748b"
								>
									{backend.detail}
								</text>

								{/* Input dots on left edge */}
								{ln.nodeGenome.input_refs.map((ref, i) => {
									const dotY =
										ln.y +
										12 +
										(i * (NODE_HEIGHT - 24)) / Math.max(1, ln.nodeGenome.input_refs.length - 1);
									return (
										<circle
											key={`in${ln.nodeGenome.node_id}-${i}`}
											cx={ln.x}
											cy={dotY}
											r={INPUT_DOT_R}
											fill={inputRefColor(ref)}
										/>
									);
								})}

								{/* Target output dot on right edge */}
								{ln.nodeGenome.targets.length > 0 && (
									<circle
										cx={ln.x + NODE_WIDTH}
										cy={ln.y + NODE_HEIGHT / 2}
										r={INPUT_DOT_R + 1}
										fill="#94a3b8"
									/>
								)}
							</g>
						);
					})}
				</svg>
			</div>

			{/* Hover tooltip */}
			{hoveredNode && <NodeTooltip node={hoveredNode.nodeGenome} isEntry={hoveredNode.isEntry} />}

			{/* Legend */}
			<div className="flex items-center gap-3 text-[9px] text-slate-600 font-mono">
				<span className="flex items-center gap-1">
					<span
						className="inline-block w-2 h-2 rounded-sm"
						style={{ backgroundColor: GRAPH_NODE_COLOR }}
					/>
					Graph
				</span>
				<span className="flex items-center gap-1">
					<span
						className="inline-block w-2 h-2 rounded-sm"
						style={{ backgroundColor: VM_NODE_COLOR }}
					/>
					VM
				</span>
				<span className="flex items-center gap-1">
					<span
						className="inline-block w-2 h-2 rounded-sm border"
						style={{ borderColor: ENTRY_RING_COLOR }}
					/>
					Entry
				</span>
			</div>
		</div>
	);
});

function NodeTooltip({
	node,
	isEntry,
}: {
	node: NodeGenome;
	isEntry: boolean;
}) {
	const backend = getBackendLabel(node.backend_def);
	const color = getNodeColor(node.backend_def);

	return (
		<div className="px-2.5 py-2 bg-slate-800/90 border border-slate-700 rounded text-[10px] font-mono space-y-1.5">
			<div className="flex items-center gap-2">
				<span className="text-slate-200 font-semibold">Node {node.node_id}</span>
				{isEntry && (
					<span className="text-[8px] px-1 py-px rounded bg-emerald-950 text-emerald-400 border border-emerald-800">
						entry
					</span>
				)}
				<span
					className="text-[8px] px-1 py-px rounded border"
					style={{
						color,
						borderColor: `${color}40`,
						backgroundColor: `${color}10`,
					}}
				>
					{backend.type}
				</span>
			</div>

			{/* Inputs */}
			{node.input_refs.length > 0 && (
				<div>
					<span className="text-slate-500">Inputs ({node.input_refs.length}):</span>
					<div className="pl-2 space-y-px">
						{node.input_refs.slice(0, 8).map((ref, i) => (
							<div key={`ref-${formatInputRef(ref)}-${i}`} className="flex items-center gap-1">
								<span
									className="inline-block w-1.5 h-1.5 rounded-full"
									style={{
										backgroundColor: inputRefColor(ref),
									}}
								/>
								<span className="text-slate-400">{formatInputRef(ref)}</span>
							</div>
						))}
						{node.input_refs.length > 8 && (
							<span className="text-slate-600">+{node.input_refs.length - 8} more</span>
						)}
					</div>
				</div>
			)}

			{/* Backend details */}
			<div className="text-slate-500">
				{backend.detail}
				{" · "}
				{node.targets.length} target{node.targets.length !== 1 ? "s" : ""}
			</div>

			{/* Graph internal nodes breakdown */}
			{"Graph" in node.backend_def && <GraphNodesBreakdown backendDef={node.backend_def} />}
		</div>
	);
}

function GraphNodesBreakdown({ backendDef }: { backendDef: BackendDef }) {
	if (!("Graph" in backendDef)) return null;
	const { internal_nodes } = backendDef.Graph;
	if (internal_nodes.length === 0) return null;

	return (
		<div className="text-slate-600 pl-2">
			{internal_nodes.slice(0, 6).map((n, i) => (
				<span key={`gn-${getGraphNodeKindLabel(n.kind)}-${i}`}>
					{getGraphNodeKindLabel(n.kind)}
					{i < Math.min(5, internal_nodes.length - 1) ? ", " : ""}
				</span>
			))}
			{internal_nodes.length > 6 && <span> +{internal_nodes.length - 6}</span>}
		</div>
	);
}

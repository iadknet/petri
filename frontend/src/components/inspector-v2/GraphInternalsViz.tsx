import {
	type Edge,
	MarkerType,
	type Node,
	ReactFlow,
	ReactFlowProvider,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { memo, useEffect, useMemo, useState } from "react";
import type {
	GraphInternalNode,
	GraphNodeKind,
	InputReference,
} from "../../types/genome.ts";
import type { GraphTrace } from "../../types/trace.ts";
import { formatInputRef } from "../inspector/inputRefUtils.ts";
import {
	type GraphInternalsNodeData,
	GraphInternalsNode,
} from "./GraphInternalsNode.tsx";
import {
	BackwardWeightEdge,
	WeightEdge,
	type WeightEdgeData,
} from "./graphInternalsEdges.tsx";
import {
	type GraphInternalsLayoutResult,
	layoutGraphInternals,
} from "./graphInternalsLayout.ts";

interface GraphInternalsVizProps {
	internalNodes: GraphInternalNode[];
	liveIndices: number[];
	inputRefs: InputReference[];
	targets: number[];
	routeTargetIdx: number | null;
	trace: GraphTrace | null;
	detailIndex: number;
}

const STATEFUL_KINDS = new Set([
	"DecayIntegrator",
	"Momentum",
	"Oscillator",
	"AdaptiveGain",
]);

const INPUT_KINDS = new Set(["InputRef"]);

const CONSTANT_KINDS = new Set(["Constant"]);

const MEMORY_KINDS = new Set(["ReadSlot", "ReadSlotPrev"]);

const OUTPUT_KINDS = new Set([
	"RouterOutput",
	"CustomOutput",
	"WriteSlot",
	"ClearSlot",
	"WriteActionMeta",
	"PushAction",
	"PopAction",
	"ExecuteActionQueue",
]);

export function getKindName(kind: GraphNodeKind): string {
	if (typeof kind === "string") return kind;
	const [name] = Object.entries(kind)[0] ?? ["?"];
	return name;
}

/** Produce a human-readable label for a graph internal node kind. */
export function getKindLabel(
	kind: GraphNodeKind,
	inputRefs: InputReference[],
): string {
	if (typeof kind === "string") {
		// Friendlier names for common string-type kinds
		switch (kind) {
			case "RouterOutput":
				return "Route Out";
			case "WeightedSum":
				return "Σ Weight";
			case "Clamp01":
				return "Clamp";
			case "GreaterThan":
				return "GT";
			case "ExecuteActionQueue":
				return "ExecQueue";
			case "PopAction":
				return "PopAct";
			default:
				return kind;
		}
	}
	const [name, value] = Object.entries(kind)[0] ?? ["?", ""];
	switch (name) {
		case "InputRef": {
			const { ref_idx, sub_idx } = value as { ref_idx: number; sub_idx: number };
			// u16::MAX (65535) is a sentinel for invalidated input refs
			if (ref_idx === 0xFFFF) {
				return sub_idx > 0 ? `Dead[${sub_idx}]` : "Dead";
			}
			const ref = inputRefs[ref_idx];
			const label = ref ? formatInputRef(ref) : `In(${ref_idx})`;
			return sub_idx > 0 ? `${label}[${sub_idx}]` : label;
		}
		case "Constant":
			return `Const(${(value as number).toFixed(2)})`;
		case "CustomOutput":
			return `Out[${value}]`;
		case "WriteSlot":
			return `Write s[${value}]`;
		case "ClearSlot":
			return `Clear s[${value}]`;
		case "ReadSlot":
			return `Read s[${value}]`;
		case "ReadSlotPrev":
			return `Prev s[${value}]`;
		case "WriteActionMeta":
			return `ActMeta[${value}]`;
		case "PushAction":
			return `Push(${value})`;
		case "Threshold":
			return `Thresh(${(value as number).toFixed(2)})`;
		case "DecayIntegrator":
			return `Decay(${(value as number).toFixed(2)})`;
		case "Momentum":
			return `Momentum(${(value as number).toFixed(2)})`;
		case "Oscillator":
			return `Osc(${(value as number).toFixed(2)})`;
		default:
			if (typeof value === "number") {
				return `${name}(${Number.isInteger(value) ? value : (value as number).toFixed(2)})`;
			}
			return name;
	}
}

export function categorize(kind: GraphNodeKind): "input" | "constant" | "memory" | "processing" | "output" {
	const name = getKindName(kind);
	if (INPUT_KINDS.has(name)) return "input";
	if (CONSTANT_KINDS.has(name)) return "constant";
	if (MEMORY_KINDS.has(name)) return "memory";
	if (OUTPUT_KINDS.has(name)) return "output";
	return "processing";
}

const nodeTypes = { graphInternal: GraphInternalsNode };

const edgeTypes = {
	weightEdge: memo(WeightEdge),
	backwardEdge: memo(BackwardWeightEdge),
};

/** Compute which target node ID was selected given a route value and targets array. */
export function resolveSelectedTarget(routeTargetIdx: number, targets: number[]): number | null {
	if (targets.length === 0) return null;
	const idx = Number.isNaN(routeTargetIdx)
		? -1
		: Math.floor(routeTargetIdx);
	const pos = ((idx % targets.length) + targets.length) % targets.length;
	return targets[pos] ?? null;
}

export const GraphInternalsViz = memo(function GraphInternalsViz({
	internalNodes,
	liveIndices,
	inputRefs,
	targets,
	routeTargetIdx,
	trace,
	detailIndex,
}: GraphInternalsVizProps) {
	const [layout, setLayout] = useState<GraphInternalsLayoutResult | null>(
		null,
	);

	const liveSet = useMemo(() => new Set(liveIndices), [liveIndices]);

	// Pre-compute labels and categories for layout
	const nodeLabels = useMemo(
		() =>
			internalNodes.map((node) => getKindLabel(node.kind, inputRefs)),
		[internalNodes, inputRefs],
	);

	const nodeCategories = useMemo(
		() => internalNodes.map((node) => categorize(node.kind)),
		[internalNodes],
	);

	// Compute first-pass values for showing deltas
	const firstPass = trace?.passes[0] ?? null;
	const initialByIndex = useMemo(() => {
		if (!firstPass) return null;
		const map = new Map<number, number>();
		for (const evalNode of firstPass.node_evaluations) {
			map.set(evalNode.node_index, evalNode.output);
		}
		return map;
	}, [firstPass]);

	// Compute current pass data from trace
	const currentPass = trace?.passes[detailIndex] ?? null;
	const evalByIndex = useMemo(() => {
		if (!currentPass) return null;
		const map = new Map<
			number,
			{ output: number; stateChange: string | null }
		>();
		for (const evalNode of currentPass.node_evaluations) {
			const isStateful = STATEFUL_KINDS.has(evalNode.kind);
			const stateChanged =
				isStateful && evalNode.state_before !== evalNode.state_after;
			map.set(evalNode.node_index, {
				output: evalNode.output,
				stateChange: stateChanged
					? `${evalNode.state_before.toFixed(2)}→${evalNode.state_after.toFixed(2)}`
					: null,
			});
		}
		return map;
	}, [currentPass]);

	// Re-layout when trace presence changes (not on every pass change)
	const hasTrace = trace !== null;

	// Run ELK layout with dynamic widths/heights based on label length and content
	useEffect(() => {
		let cancelled = false;
		const hasTargets = targets.length > 0;
		const targetRowWidth = hasTargets
			? targets.map((id) => `#${id}`).join(", ").length * 6 + 24
			: 0;
		// Extra width for value display during execution ("= X.XX" or "X.XX → X.XX")
		const traceValueWidth = hasTrace ? 80 : 0;
		const widths = nodeLabels.map((label, i) => {
			const labelWidth = Math.max(72, Math.min(label.length * 7 + 24, 140)) + traceValueWidth;
			// RouterOutput nodes may need extra width for target labels
			if (hasTargets && getKindName(internalNodes[i]?.kind ?? "Add") === "RouterOutput") {
				return Math.max(labelWidth, targetRowWidth);
			}
			return labelWidth;
		});
		const heights = internalNodes.map((node) =>
			hasTargets && getKindName(node.kind) === "RouterOutput" ? 36 : 24,
		);
		layoutGraphInternals(internalNodes, widths, nodeCategories, heights).then((result) => {
			if (!cancelled) setLayout(result);
		});
		return () => {
			cancelled = true;
		};
	}, [internalNodes, nodeLabels, nodeCategories, hasTrace, targets]);

	const { nodes, edges } = useMemo(() => {
		if (!layout) return { nodes: [], edges: [] };

		const flowNodes: Node<GraphInternalsNodeData>[] = layout.nodes.map(
			(ln) => {
				const internalNode = internalNodes[ln.index];
				const kind = internalNode?.kind;
				const evalData = evalByIndex?.get(ln.index) ?? null;

				return {
					id: String(ln.index),
					type: "graphInternal",
					position: { x: ln.x, y: ln.y },
					draggable: false,
					connectable: false,
					selectable: false,
					style: { width: ln.width, height: ln.height },
					data: {
						index: ln.index,
						kindLabel: nodeLabels[ln.index] ?? (kind ? getKindLabel(kind, inputRefs) : "?"),
						category: kind ? categorize(kind) : "processing",
						isLive: liveSet.has(ln.index),
						initialValue: initialByIndex?.get(ln.index) ?? null,
						outputValue: evalData?.output ?? null,
						stateChange: evalData?.stateChange ?? null,
						routeTargets: kind && getKindName(kind) === "RouterOutput" ? targets : null,
						selectedTarget: kind && getKindName(kind) === "RouterOutput" && routeTargetIdx !== null
							? resolveSelectedTarget(routeTargetIdx, targets)
							: null,
					},
				};
			},
		);

		const flowEdges: Edge<WeightEdgeData>[] = layout.edges.map((le) => {
			const opacity = Math.max(0.2, Math.min(Math.abs(le.weight), 1));
			if (le.isBackward) {
				return {
					id: le.id,
					source: String(le.fromIndex),
					target: String(le.toIndex),
					sourceHandle: "bottom-out",
					targetHandle: "bottom-in",
					type: "backwardEdge",
					selectable: false,
					focusable: false,
					markerEnd: {
						type: MarkerType.ArrowClosed,
						color: `rgba(251,191,36,${opacity})`,
						width: 10,
						height: 10,
					},
					data: {
						weight: le.weight,
					},
				};
			}
			return {
				id: le.id,
				source: String(le.fromIndex),
				target: String(le.toIndex),
				type: "weightEdge",
				selectable: false,
				focusable: false,
				markerEnd: {
					type: MarkerType.ArrowClosed,
					color: `rgba(148,163,184,${opacity})`,
					width: 10,
					height: 10,
				},
				style: {
					stroke: `rgba(148,163,184,${opacity})`,
				},
				data: {
					weight: le.weight,
				},
			};
		});

		return { nodes: flowNodes, edges: flowEdges };
	}, [layout, internalNodes, liveSet, evalByIndex, initialByIndex, nodeLabels, inputRefs, targets, routeTargetIdx]);

	if (!layout) {
		return (
			<div className="flex items-center justify-center py-4">
				<p className="text-[10px] font-mono text-slate-500">
					Computing layout…
				</p>
			</div>
		);
	}

	const height = Math.max(Math.min(layout.height + 24, 300), 120);

	return (
		<div className="px-3 py-2">
			{trace ? (
				<div className="mb-1 flex items-baseline gap-2 text-[10px] font-mono">
					<span className="text-slate-500 uppercase tracking-wider font-medium">
						Pass {detailIndex + 1}/{trace.passes.length}
					</span>
					{currentPass ? (
						<span className="text-slate-600">
							Δ={currentPass.max_delta.toFixed(4)}
						</span>
					) : null}
					<span
						className={
							trace.converged ? "text-emerald-400" : "text-amber-400"
						}
					>
						{trace.converged ? "converged" : "not converged"}
					</span>
				</div>
			) : (
				<div className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					Graph Internals ({internalNodes.length})
				</div>
			)}
			<div
				className="rounded-lg border border-white/6 overflow-hidden bg-slate-950"
				style={{ height }}
			>
				<ReactFlowProvider>
					<ReactFlow
						nodes={nodes}
						edges={edges}
						nodeTypes={nodeTypes}
						edgeTypes={edgeTypes}
						fitView
						fitViewOptions={{ padding: 0.15 }}
						minZoom={0.3}
						maxZoom={2}
						panOnDrag
						zoomOnScroll
						zoomOnPinch
						preventScrolling={false}
						proOptions={{ hideAttribution: true }}
						className="!bg-transparent"
					/>
				</ReactFlowProvider>
			</div>
		</div>
	);
});

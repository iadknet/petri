import { type Edge, MarkerType, type Node, ReactFlow, ReactFlowProvider } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { memo, useEffect, useMemo, useState } from "react";
import type {
	ComputeNode,
	ComputeNodeKind,
	GraphSource,
	InputReference,
} from "../../types/genome.ts";
import type { GraphTrace } from "../../types/trace.ts";
import { GraphInternalsNode, type GraphInternalsNodeData } from "./GraphInternalsNode.tsx";
import { BackwardWeightEdge, WeightEdge, type WeightEdgeData } from "./graphInternalsEdges.tsx";
import { type GraphInternalsLayoutResult, layoutGraphInternals } from "./graphInternalsLayout.ts";
import { directionName, formatInputRef, isRingSensor } from "./inputRefUtils.ts";

interface GraphInternalsVizProps {
	computeNodes: ComputeNode[];
	liveIndices: number[];
	inputRefs: InputReference[];
	targets: number[];
	routeTargetIdx: number | null;
	trace: GraphTrace | null;
	detailIndex: number;
}

const STATEFUL_KINDS = new Set(["DecayIntegrator", "Momentum", "Oscillator", "AdaptiveGain"]);

export function getKindName(kind: ComputeNodeKind): string {
	if (typeof kind === "string") return kind;
	const [name] = Object.entries(kind)[0] ?? ["?"];
	return name;
}

/** Produce a human-readable label for a compute node kind. */
export function getKindLabel(kind: ComputeNodeKind, _inputRefs: InputReference[]): string {
	if (typeof kind === "string") {
		switch (kind) {
			case "WeightedSum":
				return "\u03A3 Weight";
			case "Clamp01":
				return "Clamp";
			case "GreaterThan":
				return "GT";
			default:
				return kind;
		}
	}
	const [name, value] = Object.entries(kind)[0] ?? ["?", ""];
	switch (name) {
		case "Constant":
			return `Const(${(value as number).toFixed(2)})`;
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

/** Format a GraphSource as a human-readable label. */
export function formatGraphSource(source: GraphSource, inputRefs: InputReference[]): string {
	if ("ComputeNode" in source) return `CN${source.ComputeNode}`;
	if ("SharedMemory" in source) {
		const { slot, previous } = source.SharedMemory;
		return previous ? `Prev s[${slot}]` : `Read s[${slot}]`;
	}
	if ("InputLeaf" in source) {
		const { ref_idx, sub_idx } = source.InputLeaf;
		if (ref_idx === 0xffff) return sub_idx > 0 ? `Dead[${sub_idx}]` : "Dead";
		const ref = inputRefs[ref_idx];
		const label = ref ? formatInputRef(ref) : `In(${ref_idx})`;
		if (ref && typeof ref !== "string" && "World" in ref && isRingSensor(ref.World)) {
			return `${label}[${directionName(sub_idx)}]`;
		}
		return sub_idx > 0 ? `${label}[${sub_idx}]` : label;
	}
	return "?";
}

export function categorize(kind: ComputeNodeKind): "constant" | "processing" {
	const name = getKindName(kind);
	if (name === "Constant") return "constant";
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
	const idx = Number.isNaN(routeTargetIdx) ? -1 : Math.floor(routeTargetIdx);
	const pos = ((idx % targets.length) + targets.length) % targets.length;
	return targets[pos] ?? null;
}

export const GraphInternalsViz = memo(function GraphInternalsViz({
	computeNodes,
	liveIndices,
	inputRefs,
	targets: _targets,
	routeTargetIdx: _routeTargetIdx,
	trace,
	detailIndex,
}: GraphInternalsVizProps) {
	const [layout, setLayout] = useState<GraphInternalsLayoutResult | null>(null);

	const liveSet = useMemo(() => new Set(liveIndices), [liveIndices]);

	// Pre-compute labels and categories for layout
	const nodeLabels = useMemo(
		() => computeNodes.map((node) => getKindLabel(node.kind, inputRefs)),
		[computeNodes, inputRefs],
	);

	const nodeCategories = useMemo(
		() => computeNodes.map((node) => categorize(node.kind)),
		[computeNodes],
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
		const map = new Map<number, { output: number; stateChange: string | null }>();
		for (const evalNode of currentPass.node_evaluations) {
			const isStateful = STATEFUL_KINDS.has(evalNode.kind);
			const stateChanged = isStateful && evalNode.state_before !== evalNode.state_after;
			map.set(evalNode.node_index, {
				output: evalNode.output,
				stateChange: stateChanged
					? `${evalNode.state_before.toFixed(2)}\u2192${evalNode.state_after.toFixed(2)}`
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
		// Extra width for value display during execution ("= X.XX" or "X.XX → X.XX")
		const traceValueWidth = hasTrace ? 80 : 0;
		const widths = nodeLabels.map((label) => {
			return Math.max(72, Math.min(label.length * 7 + 24, 140)) + traceValueWidth;
		});
		const heights = computeNodes.map(() => 24);
		layoutGraphInternals(computeNodes, widths, nodeCategories, heights).then((result) => {
			if (!cancelled) setLayout(result);
		});
		return () => {
			cancelled = true;
		};
	}, [computeNodes, nodeLabels, nodeCategories, hasTrace]);

	const { nodes, edges } = useMemo(() => {
		if (!layout) return { nodes: [], edges: [] };

		const flowNodes: Node<GraphInternalsNodeData>[] = layout.nodes.map((ln) => {
			const computeNode = computeNodes[ln.index];
			const kind = computeNode?.kind;
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
					routeTargets: null,
					selectedTarget: null,
				},
			};
		});

		const flowEdges: Edge<WeightEdgeData>[] = layout.edges.map((le) => {
			const endpointsLive = liveSet.has(le.fromIndex) && liveSet.has(le.toIndex);
			const weightOpacity = Math.max(0.2, Math.min(Math.abs(le.weight), 1));
			const opacity = endpointsLive ? weightOpacity : weightOpacity * 0.35;
			if (le.isBackward) {
				return {
					id: le.id,
					source: String(le.toIndex),
					target: String(le.fromIndex),
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
						opacity,
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
					opacity,
				},
			};
		});

		return { nodes: flowNodes, edges: flowEdges };
	}, [layout, computeNodes, liveSet, evalByIndex, initialByIndex, nodeLabels, inputRefs]);

	if (!layout) {
		return (
			<div className="flex items-center justify-center py-4">
				<p className="text-[10px] font-mono text-slate-500">Computing layout\u2026</p>
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
						<span className="text-slate-600">\u0394={currentPass.max_delta.toFixed(4)}</span>
					) : null}
					<span className={trace.converged ? "text-emerald-400" : "text-amber-400"}>
						{trace.converged ? "converged" : "not converged"}
					</span>
				</div>
			) : (
				<div className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					Compute Nodes ({computeNodes.length})
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

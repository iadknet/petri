import { ReactFlow, ReactFlowProvider } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { memo, useEffect, useMemo, useState } from "react";
import type { GraphBackendDef, InputReference, RouteTarget } from "../../types/genome.ts";
import type { GraphTrace } from "../../types/trace.ts";
import { GraphInternalsNode } from "./GraphInternalsNode.tsx";
import { buildGraphFlowScene } from "./graphFlowAdapter.ts";
import { buildFullGraphModel } from "./graphFullModel.ts";
import { BackwardWeightEdge, WeightEdge } from "./graphInternalsEdges.tsx";
import {
	type LayoutInputEdge,
	type LayoutInputNode,
	type LayoutOutput,
	layoutGraphInternals,
} from "./graphInternalsLayout.ts";
import { buildComputeTraceOverlay, buildOutputTraceOverlay } from "./graphTraceOverlay.ts";

interface GraphInternalsVizProps {
	graphDef: GraphBackendDef;
	liveIndices: number[];
	inputRefs: InputReference[];
	targets: RouteTarget[];
	routeTargetIdx: number | null;
	trace: GraphTrace | null;
	detailIndex: number;
}

const nodeTypes = { graphInternal: GraphInternalsNode };

const edgeTypes = {
	weightEdge: memo(WeightEdge),
	backwardEdge: memo(BackwardWeightEdge),
};

/** Compute which target node ID was selected given a route value and targets array. */
export function resolveSelectedTarget(routeTargetIdx: number, targets: RouteTarget[]): number | null {
	if (targets.length === 0) return null;
	if (routeTargetIdx < 0 || routeTargetIdx >= targets.length) return null;
	return targets[routeTargetIdx]?.target_id ?? null;
}

export const GraphInternalsViz = memo(function GraphInternalsViz({
	graphDef,
	liveIndices,
	inputRefs,
	targets: _targets, // TODO: wire up resolveSelectedTarget for RouterOutput nodes
	routeTargetIdx: _routeTargetIdx, // TODO: wire up resolveSelectedTarget for RouterOutput nodes
	trace,
	detailIndex,
}: GraphInternalsVizProps) {
	const [layout, setLayout] = useState<LayoutOutput | null>(null);

	const liveSet = useMemo(() => new Set(liveIndices), [liveIndices]);
	const hasTrace = trace !== null;

	// 1. Build domain model
	const model = useMemo(() => buildFullGraphModel(graphDef, inputRefs), [graphDef, inputRefs]);

	// 2. Compute layout input (nodes with layer constraints + edges)
	const layoutInput = useMemo(() => {
		const traceValueExtraWidth = hasTrace ? 80 : 0;
		const layoutNodes: LayoutInputNode[] = model.nodes.map((mn) => {
			let layerConstraint: "FIRST" | "LAST" | undefined;
			if (mn.nodeType === "input" || mn.category === "constant") {
				layerConstraint = "FIRST";
			} else if (
				mn.nodeType === "output_sink" ||
				mn.nodeType === "action_slot" ||
				mn.nodeType === "execute_gate"
			) {
				layerConstraint = "LAST";
			}
			return {
				id: mn.id,
				width: mn.baseWidth + traceValueExtraWidth,
				height: mn.height,
				layerConstraint,
			};
		});
		const layoutEdges: LayoutInputEdge[] = model.edges.map((me) => ({
			id: me.id,
			sourceId: me.sourceId,
			targetId: me.targetId,
		}));
		return { nodes: layoutNodes, edges: layoutEdges, traceValueExtraWidth };
	}, [model, hasTrace]);

	// 3. Run async ELK layout
	useEffect(() => {
		let cancelled = false;
		layoutGraphInternals(layoutInput.nodes, layoutInput.edges).then((result) => {
			if (!cancelled) setLayout(result);
		});
		return () => {
			cancelled = true;
		};
	}, [layoutInput]);

	// 4. Build trace overlay
	const traceOverlay = useMemo(() => {
		if (!trace) return null;
		const overlay = buildComputeTraceOverlay(trace, detailIndex);
		for (const [k, v] of buildOutputTraceOverlay(trace)) overlay.set(k, v);
		return overlay;
	}, [trace, detailIndex]);

	// 5. Build ReactFlow scene
	const scene = useMemo(() => {
		if (!layout) return null;
		return buildGraphFlowScene({
			model,
			layout,
			traceOverlay,
			liveSet,
		});
	}, [model, layout, traceOverlay, liveSet]);

	if (!layout || !scene) {
		return (
			<div className="flex items-center justify-center py-4">
				<p className="text-[10px] font-mono text-slate-500">Computing layout{"\u2026"}</p>
			</div>
		);
	}

	const inputCount = model.nodes.filter((n) => n.nodeType === "input").length;
	const computeCount = graphDef.compute_nodes.length;
	const outputCount = model.nodes.filter(
		(n) =>
			n.nodeType === "output_sink" || n.nodeType === "action_slot" || n.nodeType === "execute_gate",
	).length;

	const currentPass = trace?.passes[detailIndex] ?? null;
	const height = Math.max(Math.min(layout.totalHeight + 24, 450), 120);

	return (
		<div className="px-3 py-2">
			{trace ? (
				<div className="mb-1 flex items-baseline gap-2 text-[10px] font-mono">
					<span className="text-slate-500 uppercase tracking-wider font-medium">
						Pass {detailIndex + 1}/{trace.passes.length}
					</span>
					{currentPass ? (
						<span className="text-slate-600">
							{"\u0394"}={currentPass.max_delta.toFixed(4)}
						</span>
					) : null}
					<span className={trace.converged ? "text-emerald-400" : "text-amber-400"}>
						{trace.converged ? "converged" : "not converged"}
					</span>
				</div>
			) : (
				<div className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-500">
					Graph ({inputCount} in {"\u00b7"} {computeCount} compute {"\u00b7"} {outputCount} out)
				</div>
			)}
			<div
				className="rounded-lg border border-white/6 overflow-hidden bg-slate-950"
				style={{ height }}
			>
				<ReactFlowProvider>
					<ReactFlow
						nodes={scene.nodes}
						edges={scene.edges}
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

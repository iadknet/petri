import { BaseEdge, type EdgeProps } from "@xyflow/react";
import { memo } from "react";
import type { MeshFlowEdgeData } from "./meshFlowAdapter.ts";

export const MeshFlowEdge = memo(function MeshFlowEdge(props: EdgeProps) {
	const data = props.data as MeshFlowEdgeData | undefined;
	const points = data?.points ?? [];
	if (points.length < 2) {
		return null;
	}

	const [first, ...rest] = points;
	const path = [`M ${first?.x ?? 0} ${first?.y ?? 0}`];
	for (const point of rest) {
		path.push(`L ${point.x} ${point.y}`);
	}

	const isActive = data?.active ?? false;

	return (
		<BaseEdge
			path={path.join(" ")}
			markerEnd={props.markerEnd}
			style={{
				stroke: isActive
					? "rgba(250,204,21,0.9)"
					: data?.dimmed
						? "rgba(100,116,139,0.3)"
						: "rgba(148,163,184,0.68)",
				strokeWidth: isActive ? 2.5 : data?.dimmed ? 1.25 : 1.7,
				strokeDasharray: isActive ? "8 4" : undefined,
				animation: isActive ? "meshEdgeDash 0.6s linear infinite" : undefined,
			}}
		/>
	);
});

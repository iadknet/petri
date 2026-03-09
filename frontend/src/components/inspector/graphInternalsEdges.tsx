import { type EdgeProps, getBezierPath } from "@xyflow/react";

export interface WeightEdgeData extends Record<string, unknown> {
	weight: number;
	opacity: number;
}

export function WeightEdge({
	id,
	sourceX,
	sourceY,
	targetX,
	targetY,
	sourcePosition,
	targetPosition,
	data,
	style,
	markerEnd,
}: EdgeProps) {
	const [edgePath, labelX, labelY] = getBezierPath({
		sourceX,
		sourceY,
		targetX,
		targetY,
		sourcePosition,
		targetPosition,
	});

	const edgeData = data as WeightEdgeData | undefined;
	const weight = edgeData?.weight ?? 1;
	const showWeight = Math.abs(weight - 1.0) > 0.01;

	return (
		<>
			<path
				id={id}
				d={edgePath}
				fill="none"
				strokeWidth={1}
				markerEnd={markerEnd as string}
				style={style}
			/>
			{showWeight ? (
				<text
					x={labelX}
					y={labelY - 6}
					textAnchor="middle"
					className="text-[8px] font-mono"
					fill="rgba(148,163,184,0.6)"
				>
					{weight.toFixed(2)}
				</text>
			) : null}
		</>
	);
}

export function BackwardWeightEdge({
	id,
	sourceX,
	sourceY,
	targetX,
	targetY,
	data,
	markerEnd,
}: EdgeProps) {
	const edgeData = data as WeightEdgeData | undefined;
	const weight = edgeData?.weight ?? 1;
	const opacity = edgeData?.opacity ?? Math.max(0.2, Math.min(Math.abs(weight), 1));
	const showWeight = Math.abs(weight - 1.0) > 0.01;

	const isSelfLoop = Math.abs(sourceX - targetX) < 5 && Math.abs(sourceY - targetY) < 5;

	let edgePath: string;
	let labelX: number;
	let labelY: number;

	if (isSelfLoop) {
		// Arc that exits bottom, curves down ~20px, and returns
		const r = 14;
		edgePath = `M ${sourceX - 6} ${sourceY} A ${r} ${r} 0 1 0 ${sourceX + 6} ${sourceY}`;
		labelX = sourceX;
		labelY = sourceY + r * 2 + 6;
	} else {
		// Scale the drop with node distance so nearby nodes get a subtle
		// curve instead of the dramatic U-loop from Position.Bottom bezier
		const dx = Math.abs(targetX - sourceX);
		const dy = Math.abs(targetY - sourceY);
		const drop = Math.max(20, Math.min(50, dx * 0.15 + dy * 0.2));
		const bottomY = Math.max(sourceY, targetY) + drop;
		edgePath = `M ${sourceX} ${sourceY} C ${sourceX} ${bottomY}, ${targetX} ${bottomY}, ${targetX} ${targetY}`;
		labelX = (sourceX + targetX) / 2;
		labelY = bottomY;
	}

	return (
		<>
			<path
				id={id}
				d={edgePath}
				fill="none"
				strokeWidth={1}
				strokeDasharray="4 3"
				markerEnd={markerEnd as string}
				style={{ stroke: `rgba(251,191,36,${opacity})` }}
			/>
			{showWeight ? (
				<text
					x={labelX}
					y={labelY - 6}
					textAnchor="middle"
					className="text-[8px] font-mono"
					fill="rgba(251,191,36,0.6)"
				>
					{weight.toFixed(2)}
				</text>
			) : null}
		</>
	);
}

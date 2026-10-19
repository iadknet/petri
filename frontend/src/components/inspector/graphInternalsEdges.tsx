import { type EdgeProps, getBezierPath } from "@xyflow/react";
import type { MouseEvent, PointerEvent } from "react";
import { TipBubble, useTipAnchor } from "../config-panel/shared/Tooltip.tsx";
import { plasticityDetail } from "./PlasticEdgeList.tsx";
import { composeTerms } from "./edgeGroupWeights.ts";
import type { GraphModelEdge } from "./graphFullModel.ts";

/** An edge's tooltip facts: its endpoints' labels and the model's weights. */
export interface EdgeTip extends Pick<GraphModelEdge, "weight" | "own" | "member" | "plastic"> {
	source: string;
	target: string;
}

export interface WeightEdgeData extends Record<string, unknown> {
	/** `null` when the edge's weight in force is unavailable. */
	weight: number | null;
	opacity: number;
	tint?: { r: number; g: number; b: number };
	tip?: EdgeTip;
}

/** A plastic edge's weights: genomic, and applied when the payload carries it. */
function plasticWeights(plastic: NonNullable<EdgeTip["plastic"]>): string {
	const genomic = `genomic ${plastic.genomic.toFixed(3)}`;
	if (plastic.applied === undefined) return `${genomic} · applied unavailable`;
	return `${genomic} · applied ${plastic.applied.toFixed(3)}${
		plastic.initialized ? "" : " (not initialized)"
	}`;
}

/**
 * A member's weight in force and scope (T25.F07); without the payload the
 * applied weight is unavailable (a static member of a learning group
 * weighs through the learned scale).
 */
export function memberApplied(member: NonNullable<EdgeTip["member"]>): string {
	const applied =
		member.applied === undefined ? "applied unavailable" : `applied ${member.applied.toFixed(3)}`;
	return `${applied} · scope ${member.scope}`;
}

/** The tooltip body: endpoints, then a member's composition and a plastic edge's learning. */
function EdgeTipContent({ tip }: { tip: EdgeTip }) {
	const line = "block";
	return (
		<>
			<span className={`${line} font-mono text-slate-100`}>
				{tip.source} → {tip.target}
			</span>
			{tip.member ? (
				<>
					<span className={`${line} text-slate-400`}>
						group #{tip.member.group} member: scale × pattern + correction
					</span>
					<span className={`${line} font-mono`}>
						{composeTerms(tip.own, tip.member.scale, tip.member.pattern)}
					</span>
					<span className={`${line} font-mono`} data-testid="member-applied">
						{memberApplied(tip.member)}
					</span>
				</>
			) : null}
			{tip.plastic ? (
				<>
					<span className={`${line} font-mono`}>{plasticWeights(tip.plastic)}</span>
					{plasticityDetail(tip.plastic.config, tip.member).map((detail) => (
						<span key={detail} className={`${line} text-slate-400`}>
							{detail}
						</span>
					))}
				</>
			) : null}
			{!tip.member && !tip.plastic ? (
				<span className={`${line} font-mono`}>weight {tip.own.toFixed(3)}</span>
			) : null}
		</>
	);
}

/**
 * A wide invisible stroke along the edge that opens its tooltip at the
 * pointer on hover or a tap. It sets its own pointer events: React Flow
 * turns them off on an edge that is neither selectable nor clickable.
 */
function EdgeHit({ d, tip }: { d: string; tip: EdgeTip | undefined }) {
	const { anchor, show, hide } = useTipAnchor();
	if (tip === undefined) return null;
	const open = (event: MouseEvent<SVGPathElement> | PointerEvent<SVGPathElement>) =>
		show(new DOMRect(event.clientX, event.clientY, 0, 0));
	return (
		<>
			<path
				data-testid="edge-hit"
				d={d}
				fill="none"
				stroke="transparent"
				strokeWidth={10}
				style={{ pointerEvents: "stroke", cursor: "help" }}
				onMouseEnter={open}
				onMouseLeave={hide}
				onPointerDown={open}
			/>
			{anchor ? (
				<TipBubble anchor={anchor}>
					<EdgeTipContent tip={tip} />
				</TipBubble>
			) : null}
		</>
	);
}

/** The edge's weight label: `n/a` when unavailable, none at the unit weight. */
export function weightLabel(data: WeightEdgeData | undefined): string | null {
	if (data === undefined) return null;
	if (data.weight === null) return "n/a";
	return Math.abs(data.weight - 1.0) > 0.01 ? data.weight.toFixed(2) : null;
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
	const tint = edgeData?.tint;
	const label = weightLabel(edgeData);

	const labelFill = tint ? `rgba(${tint.r},${tint.g},${tint.b},0.6)` : "rgba(148,163,184,0.6)";

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
			<EdgeHit d={edgePath} tip={edgeData?.tip} />
			{label !== null ? (
				<text
					x={labelX}
					y={labelY - 6}
					textAnchor="middle"
					className="text-[8px] font-mono"
					fill={labelFill}
				>
					{label}
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
	const opacity = edgeData?.opacity ?? 1;
	const tint = edgeData?.tint;
	const label = weightLabel(edgeData);

	const strokeColor = tint
		? `rgba(${tint.r},${tint.g},${tint.b},${opacity})`
		: `rgba(251,191,36,${opacity})`;
	const labelFill = tint ? `rgba(${tint.r},${tint.g},${tint.b},0.6)` : "rgba(251,191,36,0.6)";

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
				style={{ stroke: strokeColor }}
			/>
			<EdgeHit d={edgePath} tip={edgeData?.tip} />
			{label !== null ? (
				<text
					x={labelX}
					y={labelY - 6}
					textAnchor="middle"
					className="text-[8px] font-mono"
					fill={labelFill}
				>
					{label}
				</text>
			) : null}
		</>
	);
}

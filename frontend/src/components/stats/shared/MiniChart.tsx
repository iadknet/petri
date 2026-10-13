import { memo, useEffect, useMemo, useRef, useState } from "react";
import { currentDevicePixelRatio, watchDevicePixelRatio } from "../../../canvas/dpr.ts";

const INSET = 2;

/**
 * Map a series to CSS-pixel points across `w` × `h`. The minimum sits on the
 * bottom inset and the maximum on the top inset; a constant series runs through
 * the vertical centre so it cannot be mistaken for zero.
 */
export function chartPoints(values: number[], w: number, h: number): { x: number; y: number }[] {
	let min = Number.POSITIVE_INFINITY;
	let max = Number.NEGATIVE_INFINITY;
	for (const v of values) {
		if (v < min) min = v;
		if (v > max) max = v;
	}
	const range = max - min;
	const last = Math.max(values.length - 1, 1);
	return values.map((v, i) => ({
		x: (i / last) * w,
		y: range === 0 ? h / 2 : h - INSET - ((v - min) / range) * (h - 2 * INSET),
	}));
}

function formatValue(v: number): string {
	if (Math.abs(v) >= 100) return Math.round(v).toLocaleString("en-US");
	return String(Number(v.toFixed(2)));
}

export const MiniChart = memo(function MiniChart({
	data,
	color,
	height = 60,
}: { data: { tick: number; value: number }[]; color: string; height?: number }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const [width, setWidth] = useState(0);
	const [dpr, setDpr] = useState(currentDevicePixelRatio);

	const summary = useMemo(() => {
		if (data.length < 2) return null;
		const values = data.map((d) => d.value);
		let min = values[0]!;
		let max = values[0]!;
		for (const v of values) {
			if (v < min) min = v;
			if (v > max) max = v;
		}
		const span = `ticks ${data[0]!.tick}–${data[data.length - 1]!.tick}`;
		return { values, min: formatValue(min), max: formatValue(max), constant: min === max, span };
	}, [data]);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas) return;
		setWidth(canvas.clientWidth);
		if (typeof ResizeObserver === "undefined") return;
		const ro = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) setWidth(entry.contentRect.width);
		});
		ro.observe(canvas);
		return () => ro.disconnect();
	}, []);

	// Browser zoom or a move to another display changes the ratio without a resize.
	useEffect(() => watchDevicePixelRatio(setDpr), []);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas || width <= 0) return;
		canvas.width = Math.round(width * dpr);
		canvas.height = Math.round(height * dpr);

		const ctx = canvas.getContext("2d");
		if (!ctx) return;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, width, height);
		if (!summary) return;

		ctx.strokeStyle = color;
		ctx.lineWidth = 1.5;
		ctx.beginPath();
		const points = chartPoints(summary.values, width, height);
		for (let i = 0; i < points.length; i++) {
			const p = points[i]!;
			if (i === 0) ctx.moveTo(p.x, p.y);
			else ctx.lineTo(p.x, p.y);
		}
		ctx.stroke();
	}, [summary, color, width, height, dpr]);

	const label = summary
		? summary.constant
			? `No change: ${summary.min} over ${summary.span}`
			: `Range ${summary.min} to ${summary.max} over ${summary.span}`
		: "No data yet";

	return (
		<div>
			<div className="flex gap-2">
				<canvas
					ref={canvasRef}
					role="img"
					aria-label={label}
					className="w-full min-w-0 flex-1"
					style={{ height: `${height}px` }}
				/>
				{summary && (
					<div
						aria-hidden="true"
						className={`flex min-w-16 shrink-0 flex-col text-right text-[10px] tabular-nums text-slate-400 ${summary.constant ? "justify-center" : "justify-between"}`}
						style={{ height: `${height}px` }}
					>
						{summary.constant ? (
							<span>no change: {summary.min}</span>
						) : (
							<>
								<span>max {summary.max}</span>
								<span>min {summary.min}</span>
							</>
						)}
					</div>
				)}
			</div>
			{summary && (
				<div aria-hidden="true" className="text-[10px] tabular-nums text-slate-400">
					{summary.span}
				</div>
			)}
		</div>
	);
});

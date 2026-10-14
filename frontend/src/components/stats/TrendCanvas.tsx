import { memo, useEffect, useRef, useState } from "react";
import { currentDevicePixelRatio, watchDevicePixelRatio } from "../../canvas/dpr.ts";
import { NOISE_FLOOR } from "./readings.ts";
import { type Point, sampleX, trendGeometry, xToIndex } from "./trend.ts";

const SURFACE = "#0f172a";
const SECONDARY = "#94a3b8";
const SCRUB = "#38bdf8";
const BASELINE = "#1e293b";

function strokeRuns(ctx: CanvasRenderingContext2D, runs: Point[][]) {
	ctx.beginPath();
	for (const run of runs) {
		run.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
		if (run.length === 1) ctx.lineTo(run[0]!.x + 0.01, run[0]!.y);
	}
	ctx.stroke();
}

function dot(ctx: CanvasRenderingContext2D, p: Point, color: string, radius: number) {
	ctx.beginPath();
	ctx.arc(p.x, p.y, radius + 2, 0, Math.PI * 2);
	ctx.fillStyle = SURFACE;
	ctx.fill();
	ctx.beginPath();
	ctx.arc(p.x, p.y, radius, 0, Math.PI * 2);
	ctx.fillStyle = color;
	ctx.fill();
}

function pointAt(runs: Point[][], x: number): Point | undefined {
	for (const run of runs) {
		const hit = run.find((p) => Math.abs(p.x - x) < 0.5);
		if (hit) return hit;
	}
	return undefined;
}

/** A reading's chart: its line and wash, an optional grey second line, the scrub mark. Fills a positioned parent. */
export const TrendCanvas = memo(function TrendCanvas({
	values,
	secondary,
	color,
	scrubIndex,
	onScrub,
}: {
	values: (number | null)[];
	secondary?: (number | null)[];
	color: string;
	scrubIndex: number | null;
	onScrub: (index: number | null) => void;
}) {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	// The chart fills whatever box its parent gives it.
	const [{ width, height }, setSize] = useState({ width: 0, height: 0 });
	const [dpr, setDpr] = useState(currentDevicePixelRatio);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas) return;
		setSize({ width: canvas.clientWidth, height: canvas.clientHeight });
		if (typeof ResizeObserver === "undefined") return;
		const ro = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
		});
		ro.observe(canvas);
		return () => ro.disconnect();
	}, []);

	useEffect(() => watchDevicePixelRatio(setDpr), []);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas || width <= 0 || height <= 0) return;
		canvas.width = Math.round(width * dpr);
		canvas.height = Math.round(height * dpr);
		const ctx = canvas.getContext("2d");
		if (!ctx) return;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, width, height);

		// Leave room on the right so the end dot and its ring are never clipped.
		const plotWidth = Math.max(width - 6, 1);
		const geo = trendGeometry(values, secondary, plotWidth, height, NOISE_FLOOR);
		ctx.fillStyle = BASELINE;
		ctx.fillRect(0, height - 1, width, 1);
		ctx.lineJoin = "round";
		ctx.lineCap = "round";

		ctx.fillStyle = `${color}1f`;
		for (const run of geo.runs) {
			if (run.length < 2) continue;
			ctx.beginPath();
			ctx.moveTo(run[0]!.x, height);
			for (const p of run) ctx.lineTo(p.x, p.y);
			ctx.lineTo(run[run.length - 1]!.x, height);
			ctx.closePath();
			ctx.fill();
		}

		if (geo.secondaryRuns.length) {
			ctx.strokeStyle = SECONDARY;
			ctx.lineWidth = 1.25;
			strokeRuns(ctx, geo.secondaryRuns);
		}
		ctx.strokeStyle = color;
		ctx.lineWidth = 1.75;
		strokeRuns(ctx, geo.runs);

		const lastRun = geo.runs.at(-1);
		const end = lastRun?.at(-1);
		if (scrubIndex == null && end && end.x >= plotWidth - 0.5) dot(ctx, end, color, 2.5);

		if (scrubIndex != null) {
			const x = sampleX(scrubIndex, values.length, plotWidth);
			ctx.strokeStyle = `${SCRUB}b3`;
			ctx.lineWidth = 1;
			ctx.beginPath();
			ctx.moveTo(Math.round(x) + 0.5, 0);
			ctx.lineTo(Math.round(x) + 0.5, height);
			ctx.stroke();
			const second = pointAt(geo.secondaryRuns, x);
			if (second) dot(ctx, second, SECONDARY, 2);
			const main = pointAt(geo.runs, x);
			if (main) dot(ctx, main, color, 2.5);
		}
	}, [values, secondary, color, scrubIndex, width, height, dpr]);

	return (
		<canvas
			ref={canvasRef}
			className="absolute inset-0 block h-full w-full cursor-crosshair touch-none"
			onPointerMove={(event) => {
				const rect = event.currentTarget.getBoundingClientRect();
				onScrub(xToIndex(event.clientX - rect.left, Math.max(rect.width - 6, 1), values.length));
			}}
			onPointerLeave={() => onScrub(null)}
		/>
	);
});

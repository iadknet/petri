import { useEffect, useRef } from "react";

export function MiniChart({
	data,
	color,
	height = 60,
}: { data: { tick: number; value: number }[]; color: string; height?: number }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas || data.length < 2) return;

		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		const w = canvas.width;
		const h = canvas.height;
		ctx.clearRect(0, 0, w, h);

		const values = data.map((d) => d.value);
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min || 1;

		ctx.strokeStyle = color;
		ctx.lineWidth = 1.5;
		ctx.beginPath();

		for (let i = 0; i < data.length; i++) {
			const x = (i / (data.length - 1)) * w;
			const y = h - ((values[i]! - min) / range) * (h - 4) - 2;
			if (i === 0) ctx.moveTo(x, y);
			else ctx.lineTo(x, y);
		}
		ctx.stroke();
	}, [data, color]);

	return (
		<canvas
			ref={canvasRef}
			width={300}
			height={height}
			className="w-full"
			style={{ height: `${height}px` }}
		/>
	);
}

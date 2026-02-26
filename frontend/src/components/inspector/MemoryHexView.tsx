import { memo, useCallback, useMemo, useState } from "react";

const BYTES_PER_ROW = 16;

interface MemoryHexViewProps {
	memory: number[];
}

function memoryEqual(prev: MemoryHexViewProps, next: MemoryHexViewProps): boolean {
	const a = prev.memory;
	const b = next.memory;
	if (a === b) return true;
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}
	return true;
}

export const MemoryHexView = memo(function MemoryHexView({ memory }: MemoryHexViewProps) {
	const [hoveredByte, setHoveredByte] = useState<{
		index: number;
		x: number;
		y: number;
	} | null>(null);

	const totalRows = Math.ceil(memory.length / BYTES_PER_ROW);
	const nonZeroCount = useMemo(() => memory.filter((b) => b !== 0).length, [memory]);

	const handleByteHover = useCallback((e: React.MouseEvent, index: number) => {
		const rect = (e.target as HTMLElement).getBoundingClientRect();
		setHoveredByte({
			index,
			x: rect.left + rect.width / 2,
			y: rect.top,
		});
	}, []);

	const handleByteLeave = useCallback(() => {
		setHoveredByte(null);
	}, []);

	return (
		<div className="px-4 py-3 space-y-2">
			<div className="flex items-baseline justify-between">
				<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">Memory</span>
				<span className="text-[10px] text-slate-600 font-mono">
					{nonZeroCount}/{memory.length} active
				</span>
			</div>

			{/* Hex grid */}
			<div
				className="max-h-[200px] overflow-y-auto rounded bg-slate-950/50 border border-slate-800/50"
				aria-label="Creature memory"
			>
				{/* Header */}
				<div className="flex items-center gap-0 px-2 py-1 border-b border-slate-800/50 sticky top-0 bg-slate-950/90 backdrop-blur-sm z-10">
					<span className="w-8 text-[9px] font-mono text-slate-600 shrink-0">Addr</span>
					<div className="flex-1 flex">
						{Array.from({ length: BYTES_PER_ROW }, (_, i) => {
							const hex = i.toString(16).toUpperCase().padStart(2, "0");
							return (
								<span
									key={hex}
									className="w-[18px] text-center text-[9px] font-mono text-slate-600"
								>
									{hex}
								</span>
							);
						})}
					</div>
					<span className="w-[68px] text-[9px] font-mono text-slate-600 text-right shrink-0">
						ASCII
					</span>
				</div>

				{/* Rows */}
				{Array.from({ length: totalRows }, (_, rowIdx) => {
					const offset = rowIdx * BYTES_PER_ROW;
					const rowBytes = memory.slice(offset, offset + BYTES_PER_ROW);

					return (
						<HexRow
							key={offset}
							offset={offset}
							bytes={rowBytes}
							onByteHover={handleByteHover}
							onByteLeave={handleByteLeave}
						/>
					);
				})}
			</div>

			{/* Hover tooltip */}
			{hoveredByte !== null && (
				<ByteTooltip
					value={memory[hoveredByte.index] ?? 0}
					address={hoveredByte.index}
					x={hoveredByte.x}
					y={hoveredByte.y}
				/>
			)}
		</div>
	);
}, memoryEqual);

function HexRow({
	offset,
	bytes,
	onByteHover,
	onByteLeave,
}: {
	offset: number;
	bytes: number[];
	onByteHover: (e: React.MouseEvent, index: number) => void;
	onByteLeave: () => void;
}) {
	return (
		<div className="flex items-center gap-0 px-2 py-px hover:bg-white/[0.02]">
			{/* Address */}
			<span className="w-8 text-[10px] font-mono text-slate-600 shrink-0">
				{offset.toString(16).toUpperCase().padStart(3, "0")}
			</span>

			{/* Hex values */}
			<div className="flex-1 flex">
				{bytes.map((byte, i) => {
					const addr = offset + i;
					return (
						<span
							key={`b${addr}`}
							className={`w-[18px] text-center text-[10px] font-mono cursor-default ${
								byte !== 0 ? "text-slate-200" : "text-slate-700"
							}`}
							onMouseEnter={(e) => onByteHover(e, addr)}
							onMouseLeave={onByteLeave}
						>
							{byte.toString(16).toUpperCase().padStart(2, "0")}
						</span>
					);
				})}
			</div>

			{/* ASCII */}
			<span className="w-[68px] text-[10px] font-mono text-slate-600 text-right shrink-0 select-none">
				{bytes.map((b) => (b >= 0x20 && b <= 0x7e ? String.fromCharCode(b) : "\u00B7")).join("")}
			</span>
		</div>
	);
}

function ByteTooltip({
	value,
	address,
	x,
	y,
}: {
	value: number;
	address: number;
	x: number;
	y: number;
}) {
	return (
		<div
			className="fixed z-50 px-2 py-1.5 bg-slate-800 border border-slate-700 rounded shadow-lg pointer-events-none"
			style={{
				left: x,
				top: y - 4,
				transform: "translate(-50%, -100%)",
			}}
		>
			<div className="text-[10px] font-mono text-slate-300 space-y-0.5">
				<div>
					<span className="text-slate-500">addr </span>
					0x{address.toString(16).toUpperCase().padStart(3, "0")}
				</div>
				<div>
					<span className="text-slate-500">dec </span>
					{value}
				</div>
				<div>
					<span className="text-slate-500">bin </span>
					{value.toString(2).padStart(8, "0")}
				</div>
			</div>
		</div>
	);
}

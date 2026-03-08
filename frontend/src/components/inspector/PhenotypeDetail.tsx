import { memo } from "react";
import type { CreaturePhenotype } from "../../types/genome.ts";

const CHANNEL_LABELS = ["H1", "H2", "S1", "S2", "L1", "L2"] as const;

const CHANNEL_COLORS = [
	"#f472b6", // H1 — pink
	"#c084fc", // H2 — purple
	"#38bdf8", // S1 — sky
	"#2dd4bf", // S2 — teal
	"#fbbf24", // L1 — amber
	"#fb923c", // L2 — orange
] as const;

interface PhenotypeDetailProps {
	phenotype: CreaturePhenotype;
}

export const PhenotypeDetail = memo(function PhenotypeDetail({ phenotype }: PhenotypeDetailProps) {
	const { channels, active_channel, polarity, rgb } = phenotype;

	return (
		<div className="px-4 py-3 space-y-3">
			<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">Phenotype</span>

			{/* Channel bars */}
			<div className="space-y-1.5">
				{channels.map((value, i) => {
					const label = CHANNEL_LABELS[i] as string;
					const color = CHANNEL_COLORS[i] as string;
					const pol = polarity[i] as boolean;
					return (
						<ChannelBar
							key={label}
							label={label}
							value={value}
							color={color}
							isActive={i === active_channel}
							polarity={pol}
						/>
					);
				})}
			</div>

			{/* RGB swatch */}
			<div className="flex items-center gap-2.5 pt-1">
				<div
					className="w-7 h-7 rounded border border-white/10"
					style={{
						backgroundColor: `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`,
					}}
				/>
				<span className="text-xs font-mono text-slate-400">
					rgb({rgb[0]}, {rgb[1]}, {rgb[2]})
				</span>
			</div>
		</div>
	);
});

function ChannelBar({
	label,
	value,
	color,
	isActive,
	polarity,
}: {
	label: string;
	value: number;
	color: string;
	isActive: boolean;
	polarity: boolean;
}) {
	const ratio = value / 255;

	return (
		<div
			className={`flex items-center gap-2 text-xs rounded px-1.5 py-0.5 transition-colors ${
				isActive ? "bg-white/[0.04] ring-1 ring-white/10" : ""
			}`}
		>
			<span className="w-5 font-mono font-medium shrink-0" style={{ color }}>
				{label}
			</span>
			<div className="flex-1 h-1.5 rounded-full bg-slate-800 overflow-hidden">
				<div
					className="h-full rounded-full transition-[width] duration-200"
					style={{
						width: `${ratio * 100}%`,
						backgroundColor: color,
						opacity: isActive ? 1 : 0.6,
					}}
				/>
			</div>
			<span className="w-7 text-right font-mono text-slate-400 shrink-0">{value}</span>
			<span
				className={`w-3 text-center shrink-0 ${isActive ? "text-slate-200" : "text-slate-600"}`}
				title={polarity ? "Adding" : "Subtracting"}
			>
				{polarity ? "\u2191" : "\u2193"}
			</span>
		</div>
	);
}

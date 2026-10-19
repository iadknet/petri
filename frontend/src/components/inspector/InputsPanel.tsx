import { memo } from "react";
import type { InputReference } from "../../types/genome.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { describeWholeInputRef, formatInputRef, inputRefColor } from "./inputRefUtils.ts";

interface InputsPanelProps {
	inputRefs: InputReference[];
	upstreamSlots: number[];
}

export const InputsPanel = memo(function InputsPanel({
	inputRefs,
	upstreamSlots,
}: InputsPanelProps) {
	return (
		<div>
			<div className="text-[10px] text-slate-400 uppercase tracking-wider font-medium mb-1">
				Inputs
			</div>
			<div className="grid grid-cols-2 gap-x-4 gap-y-0.5 text-[10px] font-mono">
				{inputRefs.map((ref, i) => {
					const { description } = describeWholeInputRef(ref);
					const label = formatInputRef(ref);
					return (
						// biome-ignore lint/suspicious/noArrayIndexKey: genome order is the identity
						<div key={`${label}-${i}`} className="flex justify-between">
							<span style={{ color: inputRefColor(ref) }}>
								{description ? <Tip content={description}>{label}</Tip> : label}
							</span>
							<span className="text-slate-400">{(upstreamSlots[i] ?? 0).toFixed(2)}</span>
						</div>
					);
				})}
			</div>
		</div>
	);
});

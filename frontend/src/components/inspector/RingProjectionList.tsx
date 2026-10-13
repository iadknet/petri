import type { InputReference, RingProjection } from "../../types/genome.ts";
import { formatInputRef } from "./inputRefUtils.ts";

interface RingProjectionListProps {
	projections: RingProjection[] | undefined;
	inputRefs: InputReference[];
}

/** The ring a projection reads, or its dangling index. */
function ringLabel(refIdx: number, inputRefs: InputReference[]): string {
	const ref = inputRefs[refIdx];
	return ref === undefined ? `ref ${refIdx} (missing)` : formatInputRef(ref);
}

/**
 * A Graph node's ring projections (T11.F31): each votes its kind in all eight
 * directions from the ring rotated by its offset, at one weight.
 */
export function RingProjectionList({ projections, inputRefs }: RingProjectionListProps) {
	if (!projections || projections.length === 0) return null;
	return (
		<section aria-label="Ring projections" className="mt-2">
			<h4 className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-400">
				Ring projections ({projections.length})
			</h4>
			<ul className="space-y-0.5 font-mono text-[10px] text-slate-300">
				{projections.map((projection, index) => (
					<li
						// biome-ignore lint/suspicious/noArrayIndexKey: duplicates are allowed and genome order is the identity
						key={index}
						data-testid="ring-projection"
						className="flex gap-2"
					>
						<span>{ringLabel(projection.ref_idx, inputRefs)}</span>
						<span className="text-slate-400">{projection.kind}</span>
						<span
							title={`Each ring cell votes ${projection.kind} toward the direction ${projection.offset % 8} steps clockwise from that cell (0 = same, 4 = opposite), scaled by the weight.`}
						>
							offset {projection.offset % 8}
						</span>
						<span>w={projection.weight.toFixed(3)}</span>
					</li>
				))}
			</ul>
		</section>
	);
}

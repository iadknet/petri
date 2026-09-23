import { memo } from "react";
import type { MeshHopTrace } from "../../types/trace.ts";
import { OUTCOME_CHANNELS, VOTE_KINDS } from "../../types/trace.ts";
import { voteSinkLabel } from "./graphNodeFormatters.ts";

interface SelectedHopBlockProps {
	hop: MeshHopTrace;
	/** The tick's `PreviousOutcome` channels, frozen for every dispatch. */
	previousOutcome: readonly number[];
}

/** The non-zero entries of a recorded vote vector, labeled by sink. */
function NonZeroSinks({ label, votes }: { label: string; votes: readonly number[] }) {
	const nonZero = votes
		.map((value, index) => ({ value, index }))
		.filter((entry) => entry.value !== 0);
	return (
		<div className="flex flex-wrap gap-1" data-testid={`hop-${label}`}>
			<span className="text-slate-500">{label}</span>
			{nonZero.length === 0 ? (
				<span className="text-slate-600">all zero</span>
			) : (
				nonZero.map((entry) => (
					<span key={entry.index} className="text-slate-300">
						{voteSinkLabel(entry.index)} {entry.value.toFixed(3)}
					</span>
				))
			)}
		</div>
	);
}

/** Labeled recorded values, one per catalog entry. */
function Labeled({
	label,
	names,
	values,
	digits,
}: {
	label: string;
	names: readonly string[];
	values: readonly number[];
	digits: number;
}) {
	return (
		<div className="flex flex-wrap gap-1" data-testid={`hop-${label}`}>
			<span className="text-slate-500">{label}</span>
			{names.map((name, index) => (
				<span key={name} className="text-slate-300">
					{name} {(values[index] ?? 0).toFixed(digits)}
				</span>
			))}
		</div>
	);
}

/**
 * The selected dispatch: the vote contribution it committed and the
 * decision-state values available to it, each as recorded by the runtime.
 * The values are what the node could read, not proof that it read them.
 */
export const SelectedHopBlock = memo(function SelectedHopBlock({
	hop,
	previousOutcome,
}: SelectedHopBlockProps) {
	const inputs = hop.decision_inputs;
	return (
		<div className="px-3 py-2 border-t border-white/[0.06] text-[10px] font-mono">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Hop {hop.hop_index} · node #{hop.node_id} · pass {hop.pass_index} ·{" "}
				{hop.route ? `→ #${hop.route.selected_target_id}` : "no route"}
			</div>
			<NonZeroSinks label="vote contribution" votes={hop.vote_contribution} />
			<div className="mt-1 text-slate-500">decision state available to this dispatch</div>
			<NonZeroSinks label="ActionVotes" votes={inputs.action_votes} />
			<NonZeroSinks label="PreviousPassVotes" votes={inputs.previous_pass_votes} />
			<Labeled label="CommitCounts" names={VOTE_KINDS} values={inputs.commit_counts} digits={0} />
			<div className="flex gap-1" data-testid="hop-HopsThisTick">
				<span className="text-slate-500">HopsThisTick</span>
				<span className="text-slate-300">{inputs.hops_this_tick}</span>
			</div>
			<Labeled
				label="PreviousOutcome"
				names={OUTCOME_CHANNELS}
				values={previousOutcome}
				digits={3}
			/>
		</div>
	);
});

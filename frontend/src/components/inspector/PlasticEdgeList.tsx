import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { EdgeLearner } from "../../types/generated/EdgeLearner.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { WeightSurface } from "../../types/generated/WeightSurface.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { joinLearning } from "./brainLearning.ts";
import { sourceLabel } from "./edgeBundleWeights.ts";
import { describeGraphSource, describeOutputSink } from "./graphNodeFormatters.ts";

interface PlasticEdgeListProps {
	graphDef: GraphBrain;
	inputRefs: InputRef[];
	/** This node's `learning` entry: the weights in force, refreshed every poll. */
	learning: NodeLearning | null;
}

export interface PlasticEdgeRow {
	surface: WeightSurface;
	container: number;
	edge: number;
	/** What the edge feeds: a compute node or a sink. */
	target: string;
	/** What the edge reads. */
	source: string;
	learner: EdgeLearner;
	genomic: number;
	/** The weight in force. */
	applied: number;
	/** The learned own quantity, once initialized. */
	learned: number | undefined;
	/** Whether the learner is initialized (visited or seeded). */
	initialized: boolean;
}

/**
 * The per-edge plasticity badge: rule, pathway, trace decay and inheritance
 * (T25.F06), and where a member's learning is stored (T25.F12).
 */
export function plasticityBadge(learner: EdgeLearner): string {
	const pathway = learner.modulation
		? `reward ${learner.modulation.reward_source} · decay ${learner.modulation.trace_decay.toFixed(2)}`
		: "hebbian";
	const inheritance = learner.lamarckian ? "lamarckian" : "darwinian";
	const badge = `${learner.rule} · ${pathway} · ${inheritance}`;
	return learner.source === "lone" ? badge : `${badge} · ${sourceLabel(learner.source)}`;
}

/** The badge spelled out, one sentence per attribute, for its tooltip. */
export function plasticityDetail(learner: EdgeLearner): string[] {
	const stored =
		learner.source === "lone"
			? []
			: [
					learner.source === "own"
						? "Own override: learns its modifier on top of the bundle term."
						: "Follows its bundle's member default: learns its modifier on top of the bundle term.",
				];
	return [
		...stored,
		`${learner.rule} rule, learning rate ${learner.learning_rate.toFixed(3)}, weight clamp ±${learner.weight_clamp.toFixed(2)}.`,
		learner.modulation
			? `Reward-modulated by ${learner.modulation.reward_source}; its eligibility trace keeps ${learner.modulation.trace_decay.toFixed(2)} of itself each tick.`
			: "Hebbian: learns from co-activity alone.",
		...(learner.retention < 1
			? [
					`Decays toward its birth weight: keeps ${learner.retention.toFixed(4)} of its learned change each tick.`,
				]
			: []),
		learner.lamarckian
			? "Lamarckian: offspring inherit the learned weight."
			: "Darwinian: offspring start from the genomic weight.",
	];
}

/** The badge with its spelled-out attributes in a tooltip. */
export function PlasticityBadge({ learner }: { learner: EdgeLearner }) {
	return (
		<Tip
			content={plasticityDetail(learner).map((line) => (
				<span key={line} className="block">
					{line}
				</span>
			))}
			plain
			className="rounded bg-violet-500/15 px-1 text-violet-200"
		>
			<span data-testid="plasticity-badge">{plasticityBadge(learner)}</span>
		</Tip>
	);
}

/** Every plastic edge of `graphDef` on both surfaces, in canonical order. */
export function collectPlasticEdges(
	graphDef: GraphBrain,
	inputRefs: InputRef[],
	learning: NodeLearning | null,
): PlasticEdgeRow[] {
	const join = joinLearning(learning);
	const rows: PlasticEdgeRow[] = [];
	const visit = (surface: WeightSurface, container: number, target: string, edges: BrainEdge[]) => {
		edges.forEach((edge, index) => {
			if (!edge.learner) return;
			const state = join.edge(surface, container, index, edge);
			rows.push({
				surface,
				container,
				edge: index,
				target,
				source: describeGraphSource(edge.source, inputRefs).label,
				learner: edge.learner,
				genomic: edge.genomic,
				applied: state.applied,
				learned: state.learned,
				initialized: state.initialized ?? false,
			});
		});
	};
	graphDef.compute_nodes.forEach((node, index) => {
		visit("compute", index, `CN${index}`, node.edges);
	});
	graphDef.output_sinks.forEach((sink, index) => {
		visit("sink", index, describeOutputSink(sink.kind).label, sink.edges);
	});
	return rows;
}

/** The row's weights: genomic and in force. */
function rowWeights(row: PlasticEdgeRow): string {
	return `genomic=${row.genomic.toFixed(3)} applied=${row.applied.toFixed(3)}${
		row.initialized ? "" : " (not initialized)"
	}`;
}

/**
 * A Graph node's plastic edges (T25.F06): one row per edge on either
 * surface with its target, source, weights and a badge naming its rule,
 * pathway and inheritance.
 */
export function PlasticEdgeList({ graphDef, inputRefs, learning }: PlasticEdgeListProps) {
	const rows = collectPlasticEdges(graphDef, inputRefs, learning);
	if (rows.length === 0) return null;
	return (
		<section aria-label="Plastic edges" className="mt-2">
			<h4 className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-400">
				Plastic edges ({rows.length})
			</h4>
			<ul className="space-y-0.5 font-mono text-[10px] text-slate-300">
				{rows.map((row) => (
					<li
						key={`${row.surface}:${row.container}:${row.edge}`}
						data-testid="plastic-edge"
						className="flex flex-wrap gap-2"
					>
						<span>
							{row.source} → {row.target}
						</span>
						<PlasticityBadge learner={row.learner} />
						<span className="text-slate-400">{rowWeights(row)}</span>
					</li>
				))}
			</ul>
		</section>
	);
}

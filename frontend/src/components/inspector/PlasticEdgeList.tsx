import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type {
	GraphBackendDef,
	GraphEdge,
	InputReference,
	PlasticityConfig,
} from "../../types/genome.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { scopeLabel } from "./edgeGroupWeights.ts";
import {
	type MemberLearning,
	edgeWeightLookup,
	learnedScaleLookup,
	learnerInitialized,
} from "./graphFullModel.ts";
import { describeGraphSource, describeOutputSink } from "./graphNodeFormatters.ts";

interface PlasticEdgeListProps {
	graphDef: GraphBackendDef;
	inputRefs: InputReference[];
	/** The server's weights for this node: `applied` and `learned` per plastic edge. */
	graphWeights: NodeGraphWeights | null;
}

export interface PlasticEdgeRow {
	surface: "compute" | "sink";
	container: number;
	edge: number;
	/** What the edge feeds: a compute node or a sink. */
	target: string;
	/** What the edge reads. */
	source: string;
	config: PlasticityConfig;
	/** A member's scope and its group's `inherit_scale` (T25.F07); absent for an ungrouped edge. */
	member: MemberLearning | undefined;
	genomic: number;
	/** The weight in force, when the payload carries it. */
	applied: number | undefined;
	/** The learned own quantity, once initialized; never for a group-scope member. */
	learned: number | undefined;
	/** Whether the learner is initialized, as `learnerInitialized`. */
	initialized: boolean;
}

/**
 * Whether offspring inherit the learned state: at group scope the group's
 * `inherit_scale`, the edge's `lamarckian` flag being inert (T25.F07).
 */
function inheritsLearned(config: PlasticityConfig, member?: MemberLearning): boolean {
	return member?.scope === "Group" ? member.inheritScale : config.lamarckian;
}

/**
 * The per-edge plasticity badge: rule, pathway, trace decay and inheritance
 * (T25.F06), and a member's scope (T25.F07).
 */
export function plasticityBadge(config: PlasticityConfig, member?: MemberLearning): string {
	const pathway = config.modulation
		? `reward ${config.modulation.reward_source} · decay ${config.modulation.trace_decay.toFixed(2)}`
		: "hebbian";
	const inheritance = inheritsLearned(config, member) ? "lamarckian" : "darwinian";
	const badge = `${config.rule} · ${pathway} · ${inheritance}`;
	return member === undefined ? badge : `${badge} · ${scopeLabel(member.scope)}`;
}

/** The inheritance sentence: a group learner's follows its group's `inherit_scale`. */
function inheritanceDetail(config: PlasticityConfig, member?: MemberLearning): string {
	if (member?.scope === "Group") {
		return member.inheritScale
			? "Inherits the scale: offspring start from the group's learned scale (the Lamarckian flag is inert at group scope)."
			: "Darwinian scale: offspring start from the group's genomic scale (the Lamarckian flag is inert at group scope).";
	}
	return config.lamarckian
		? "Lamarckian: offspring inherit the learned weight."
		: "Darwinian: offspring start from the genomic weight.";
}

/** The badge spelled out, one sentence per attribute, for its tooltip. */
export function plasticityDetail(config: PlasticityConfig, member?: MemberLearning): string[] {
	const scoped =
		member === undefined
			? []
			: [
					member.scope === "Group"
						? "Group scope: learns for its group, moving the group's shared scale."
						: "Edge scope: learns its own correction on top of the group term.",
				];
	return [
		...scoped,
		`${config.rule} rule, learning rate ${config.learning_rate.toFixed(3)}, weight clamp ±${config.weight_clamp.toFixed(2)}.`,
		config.modulation
			? `Reward-modulated by ${config.modulation.reward_source}; its eligibility trace keeps ${config.modulation.trace_decay.toFixed(2)} of itself each tick.`
			: "Hebbian: learns from co-activity alone.",
		...(config.retention < 1
			? [
					`Decays toward its birth weight: keeps ${config.retention.toFixed(4)} of its learned change each tick.`,
				]
			: []),
		inheritanceDetail(config, member),
	];
}

/** The badge with its spelled-out attributes in a tooltip. */
export function PlasticityBadge({
	config,
	member,
}: { config: PlasticityConfig; member?: MemberLearning }) {
	return (
		<Tip
			content={plasticityDetail(config, member).map((line) => (
				<span key={line} className="block">
					{line}
				</span>
			))}
			plain
			className="rounded bg-violet-500/15 px-1 text-violet-200"
		>
			<span data-testid="plasticity-badge">{plasticityBadge(config, member)}</span>
		</Tip>
	);
}

/** Every plastic edge of `graphDef` on both surfaces, in canonical order. */
export function collectPlasticEdges(
	graphDef: GraphBackendDef,
	inputRefs: InputReference[],
	graphWeights: NodeGraphWeights | null,
): PlasticEdgeRow[] {
	const weightEntry = edgeWeightLookup(graphWeights);
	const learnedScale = learnedScaleLookup(graphWeights);
	const rows: PlasticEdgeRow[] = [];
	const visit = (
		surface: "compute" | "sink",
		container: number,
		target: string,
		inputs: GraphEdge[],
	) => {
		inputs.forEach((edge, index) => {
			if (!edge.plasticity) return;
			const entry = weightEntry(surface, container, index);
			rows.push({
				surface,
				container,
				edge: index,
				target,
				source: describeGraphSource(edge.source, inputRefs).label,
				config: edge.plasticity,
				member: edge.membership && {
					scope: edge.membership.scope,
					inheritScale: graphDef.groups?.[edge.membership.group]?.inherit_scale ?? false,
				},
				genomic: entry?.genomic ?? edge.own_weight,
				applied: entry?.applied,
				learned: entry?.learned,
				initialized: learnerInitialized(edge.membership, entry, learnedScale),
			});
		});
	};
	graphDef.compute_nodes.forEach((node, index) => {
		visit("compute", index, `CN${index}`, node.inputs);
	});
	graphDef.output_sinks.forEach((sink, index) => {
		visit("sink", index, describeOutputSink(sink.kind).label, sink.inputs);
	});
	return rows;
}

/** The row's weights; without the runtime payload the applied weight is unavailable. */
function rowWeights(row: PlasticEdgeRow): string {
	const genomic = `genomic=${row.genomic.toFixed(3)}`;
	if (row.applied === undefined) return `${genomic} applied=unavailable`;
	return `${genomic} applied=${row.applied.toFixed(3)}${
		row.initialized ? "" : " (not initialized)"
	}`;
}

/**
 * A Graph node's plastic edges (T25.F06): one row per edge on either
 * surface with its target, source, weights and a badge naming its rule,
 * pathway and inheritance.
 */
export function PlasticEdgeList({ graphDef, inputRefs, graphWeights }: PlasticEdgeListProps) {
	const rows = collectPlasticEdges(graphDef, inputRefs, graphWeights);
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
						<PlasticityBadge config={row.config} member={row.member} />
						<span className="text-slate-400">{rowWeights(row)}</span>
					</li>
				))}
			</ul>
		</section>
	);
}

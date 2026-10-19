import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type {
	GraphBackendDef,
	GraphEdge,
	InputReference,
	PlasticityConfig,
} from "../../types/genome.ts";
import { edgeWeightLookup } from "./graphFullModel.ts";
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
	genomic: number;
	/** The weight in force, when the payload carries it. */
	applied: number | undefined;
	/** The learned weight, once the edge is initialized. */
	learned: number | undefined;
}

/** The per-edge plasticity badge: rule, pathway and inheritance (T25.F06). */
export function plasticityBadge(config: PlasticityConfig): string {
	const pathway = config.modulation ? `reward ${config.modulation.reward_source}` : "hebbian";
	return `${config.rule} · ${pathway} · ${config.lamarckian ? "lamarckian" : "darwinian"}`;
}

/** Every plastic edge of `graphDef` on both surfaces, in canonical order. */
export function collectPlasticEdges(
	graphDef: GraphBackendDef,
	inputRefs: InputReference[],
	graphWeights: NodeGraphWeights | null,
): PlasticEdgeRow[] {
	const weightEntry = edgeWeightLookup(graphWeights);
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
				genomic: entry?.genomic ?? edge.own_weight,
				applied: entry?.applied,
				learned: entry?.learned,
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

function rowWeights(row: PlasticEdgeRow): string {
	const applied = row.applied ?? row.genomic;
	return `genomic=${row.genomic.toFixed(3)} applied=${applied.toFixed(3)}${
		row.learned === undefined ? " (not initialized)" : ""
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
						<span
							data-testid="plasticity-badge"
							className="rounded bg-violet-500/15 px-1 text-violet-200"
						>
							{plasticityBadge(row.config)}
						</span>
						<span className="text-slate-400">{rowWeights(row)}</span>
					</li>
				))}
			</ul>
		</section>
	);
}

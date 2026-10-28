import { type ReactNode, useState } from "react";
import type { BrainBundle } from "../../types/generated/BrainBundle.ts";
import type { BundleCoordinate } from "../../types/generated/BundleCoordinate.ts";
import type { BundleSource } from "../../types/generated/BundleSource.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { Scope } from "../../types/generated/Scope.ts";
import type { Site } from "../../types/generated/Site.ts";
import type { Wiring } from "../../types/generated/Wiring.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { type LearningJoin, joinLearning } from "./brainLearning.ts";
import { directionName } from "./directions.ts";
import { composeTerms, scopeLabel } from "./edgeBundleWeights.ts";
import { describeInputRef, formatInputRef } from "./inputRefUtils.ts";
import { formatVoteSink } from "./voteSinks.ts";

interface EdgeBundleListProps {
	graphDef: GraphBrain;
	inputRefs: InputRef[];
	/** This node's `learning` entry: the weights in force and learned scales. */
	learning: NodeLearning | null;
}

/** A bundle member's weights as the brain view resolves them. */
export interface BundleMember {
	own: number;
	/** The member's genomic (composed) weight. */
	genomic: number;
	scale: number;
	pattern: number;
	/** How the member learns when plastic (T25.F07). */
	scope: Scope;
	/** The weight in force. */
	applied: number;
}

/** The member a birth coordinate names: the edge at its location, the
 * key the `learning` payload uses; `undefined` for a hole. */
export function coordinateMember(
	graphDef: GraphBrain,
	coordinate: BundleCoordinate,
	join: LearningJoin,
): BundleMember | undefined {
	const at = coordinate.member;
	if (at === undefined) return undefined;
	const containers = at.surface === "sink" ? graphDef.output_sinks : graphDef.compute_nodes;
	const edge = containers[at.container]?.edges[at.edge];
	const member = edge?.member;
	if (edge === undefined || member === undefined) return undefined;
	return {
		own: edge.own,
		genomic: edge.genomic,
		scale: member.scale,
		pattern: member.pattern,
		scope: member.scope,
		applied: join.edge(at.surface, at.container, at.edge, edge).applied,
	};
}

/** A geometric wiring's kind and offset; `null` for `Plain`. */
function geometry(wiring: Wiring): { shape: string; kind: string; offset: number } | null {
	if ("Ring" in wiring) return { shape: "Ring", ...wiring.Ring };
	if ("DirectionPair" in wiring) return { shape: "Pair", ...wiring.DirectionPair };
	return null;
}

/** The sink a site names under `wiring`. */
function siteLabel(site: Site, wiring: Wiring): string {
	if ("Plain" in wiring) return formatVoteSink(wiring.Plain.sink);
	const kind = geometry(wiring)?.kind ?? "?";
	if (site === "Drive") return `${kind} drive`;
	if (site === "Sink") return kind;
	return `${kind}[${directionName(site.Direction)}]`;
}

/** What a bundle reads: its sensor (and field set when it declares several)
 * or its memory bank. */
function sourceLabel(source: BundleSource, inputRefs: InputRef[]): string {
	if ("Memory" in source) return source.Memory.previous ? "memory (previous)" : "memory";
	const { ref_idx: refIdx, field_set: fieldSet } = source.Sensor;
	const ref = inputRefs[refIdx];
	const label = ref === undefined ? `ref ${refIdx} (missing)` : formatInputRef(ref);
	return fieldSet === 0 ? label : `${label} set ${fieldSet}`;
}

/** A field of the bundle's source, as the inspector names it. */
function fieldLabel(source: BundleSource, field: number, inputRefs: InputRef[]): string {
	if ("Memory" in source) return `${source.Memory.previous ? "prev" : "mem"}[${field}]`;
	const ref = inputRefs[source.Sensor.ref_idx];
	return ref === undefined ? `field ${field}` : describeInputRef(ref, field).label;
}

/** A coordinate list's presence: `count/total ●○…`; a coordinate is
 * present when it carries a member location. */
function presence(list: readonly BundleCoordinate[]): string {
	const marks = list.map(({ member }) => (member ? "●" : "○")).join("");
	return `${list.filter(({ member }) => member).length}/${list.length} ${marks}`;
}

/** A member's weights, its composition from the bundle in a tooltip. */
function MemberWeights({ member }: { member: BundleMember }) {
	return (
		<Tip
			content={
				<>
					<span className="block text-slate-400">scale × pattern + correction</span>
					<span className="block font-mono">{composeTerms(member)}</span>
				</>
			}
		>
			own={member.own.toFixed(3)} genomic={member.genomic.toFixed(3)} applied=
			{member.applied.toFixed(3)}
			<span data-testid="edge-bundle-member-scope"> {scopeLabel(member.scope)}</span>
		</Tip>
	);
}

/** "1 step" or "n steps". */
function steps(n: number): string {
	return `${n} step${n === 1 ? "" : "s"}`;
}

/** The facts every bundle carries, each explained in a tooltip. */
function BundleFacts({
	bundle,
	learnedScale,
}: {
	bundle: BrainBundle;
	/** The learned scale, for an initialized scale slot (T25.F07). */
	learnedScale: number | undefined;
}) {
	const geo = geometry(bundle.wiring);
	return (
		<>
			{geo ? (
				<>
					<span className="text-slate-400">{geo.kind}</span>
					<Tip
						content={
							geo.shape === "Ring"
								? `Each ring cell votes ${geo.kind} toward the direction ${steps(geo.offset)} clockwise from that cell (0 = same, 4 = opposite).`
								: `Votes ${geo.kind} toward the sensed offset, rotated ${steps(geo.offset)} clockwise (0 = toward, 4 = away); the facts vote on the drive.`
						}
					>
						offset {geo.offset}
					</Tip>
				</>
			) : null}
			<Tip content="The bundle's shared weight: each member's weight is scale × its pattern + its own correction.">
				scale={bundle.scale.toFixed(3)}
			</Tip>
			{learnedScale !== undefined ? (
				<Tip content="The learned scale the bundle's group-scope members moved; every member weighs through it.">
					<span data-testid="edge-bundle-learned-scale">learned={learnedScale.toFixed(3)}</span>
				</Tip>
			) : null}
			<Tip
				content={
					bundle.inherit_scale
						? "inherit_scale: a parent's learned scale seeds the child's."
						: "No inherit_scale: a child starts from the genomic scale."
				}
			>
				<span data-testid="edge-bundle-inherit-scale">
					{bundle.inherit_scale ? "inherits scale" : "genomic scale at birth"}
				</span>
			</Tip>
			{bundle.retention < 1 ? (
				<Tip content="The learned scale decays toward its birth value, keeping this share of its learned change each tick.">
					<span data-testid="edge-bundle-retention">retention={bundle.retention.toFixed(4)}</span>
				</Tip>
			) : null}
		</>
	);
}

/**
 * One bundle: its source and wiring, its facts, and its members listed by
 * birth coordinate `(field, site)`, a hole where no member sits (T25.F11).
 * Collapsible and open by default; only the name toggles.
 */
function BundleRow({
	bundle,
	index,
	graphDef,
	join,
	inputRefs,
}: {
	bundle: BrainBundle;
	index: number;
	graphDef: GraphBrain;
	join: LearningJoin;
	inputRefs: InputRef[];
}) {
	const [open, setOpen] = useState(true);
	const wiringName =
		"Plain" in bundle.wiring
			? `Plain → ${formatVoteSink(bundle.wiring.Plain.sink)}`
			: geometry(bundle.wiring)?.shape;
	const name: ReactNode = (
		<span>
			{wiringName} {sourceLabel(bundle.source, inputRefs)} #{index}
		</span>
	);
	return (
		<li data-testid="edge-bundle" className="space-y-0.5">
			<div className="flex flex-wrap gap-2">
				<button
					type="button"
					aria-expanded={open}
					onClick={() => setOpen((value) => !value)}
					className="flex gap-2 text-left hover:text-slate-100"
				>
					<span aria-hidden="true">{open ? "▾" : "▸"}</span>
					{name}
				</button>
				<BundleFacts bundle={bundle} learnedScale={join.learnedScale(index, bundle)} />
				<Tip
					data-testid="edge-bundle-members"
					content="Members present over the birth coordinates, in birth order (● present, ○ hole)."
				>
					{presence(bundle.coordinates)}
				</Tip>
			</div>
			{open ? (
				<ul className="ml-4 space-y-0.5 text-slate-400">
					{bundle.coordinates.map((coordinate) => {
						const { field, site } = coordinate;
						// A coordinate's identity is its `(field, site)`, not its label.
						const key = `${field}:${typeof site === "string" ? site : site.Direction}`;
						const member = coordinateMember(graphDef, coordinate, join);
						const at = `${fieldLabel(bundle.source, field, inputRefs)} → ${siteLabel(site, bundle.wiring)}`;
						return member ? (
							<li key={key} data-testid="edge-bundle-member">
								{at} <MemberWeights member={member} />
							</li>
						) : (
							<li key={key} data-testid="edge-bundle-hole" className="text-slate-400">
								{at} (hole)
							</li>
						);
					})}
				</ul>
			) : null}
		</li>
	);
}

/**
 * A Graph node's edge bundles (T25.F03, T25.F11): one row per bundle with
 * its source, wiring, scale and the members present over its birth
 * coordinates (holes shown), each row expandable to its members by
 * coordinate and open by default.
 */
export function EdgeBundleList({ graphDef, inputRefs, learning }: EdgeBundleListProps) {
	const bundles = graphDef.bundles;
	if (bundles.length === 0) return null;
	const join = joinLearning(learning);
	return (
		<section aria-label="Edge bundles" className="mt-2">
			<h4 className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-400">
				Edge bundles ({bundles.length})
			</h4>
			<ul className="space-y-1 font-mono text-[10px] text-slate-300">
				{bundles.map((bundle, index) => (
					<BundleRow
						// biome-ignore lint/suspicious/noArrayIndexKey: a BundleId is the bundle's index
						key={index}
						bundle={bundle}
						index={index}
						graphDef={graphDef}
						join={join}
						inputRefs={inputRefs}
					/>
				))}
			</ul>
		</section>
	);
}

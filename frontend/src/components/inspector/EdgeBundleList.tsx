import { type ReactNode, useState } from "react";
import type { BrainBundle } from "../../types/generated/BrainBundle.ts";
import type { BundleCoordinate } from "../../types/generated/BundleCoordinate.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { Scope } from "../../types/generated/Scope.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { joinLearning } from "./brainLearning.ts";
import { directionName } from "./directions.ts";
import { composeTerms, scopeLabel } from "./edgeBundleWeights.ts";
import { describeInputRef, directionPairs, formatInputRef } from "./inputRefUtils.ts";

interface EdgeBundleListProps {
	graphDef: GraphBrain;
	inputRefs: InputRef[];
	/** This node's `learning` entry: the weights in force and learned scales. */
	learning: NodeLearning | null;
}

export interface BundleMember {
	/** The field the member reads: a ring cell, or a pair's field. */
	field: number | undefined;
	/** The direction of the sink the member sits on; absent on the drive. */
	direction: number | undefined;
	sink: number;
	edge: number;
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

/** The reference a bundle reads, or its dangling index. */
function refLabel(refIdx: number, inputRefs: InputRef[]): string {
	const ref = inputRefs[refIdx];
	return ref === undefined ? `ref ${refIdx} (missing)` : formatInputRef(ref);
}

/** Each bundle's members in sink order, as the brain view lists them. */
export function collectBundleMembers(
	graphDef: GraphBrain,
	learning: NodeLearning | null,
): BundleMember[][] {
	const join = joinLearning(learning);
	const members: BundleMember[][] = graphDef.bundles.map(() => []);
	graphDef.output_sinks.forEach((sink, sinkIndex) => {
		sink.edges.forEach((edge, edgeIndex) => {
			const member = edge.member;
			const list = member ? members[member.bundle] : undefined;
			if (!member || !list) return;
			list.push({
				field: member.field,
				direction:
					member.site !== undefined && member.site !== "Drive" ? member.site.Direction : undefined,
				sink: sinkIndex,
				edge: edgeIndex,
				own: edge.own,
				genomic: edge.genomic,
				scale: member.scale,
				pattern: member.pattern,
				scope: member.scope,
				applied: join.edge("sink", sinkIndex, edgeIndex, edge).applied,
			});
		});
	});
	return members;
}

/** Whether `member` sits on one of `coordinates`. */
function atCoordinate(member: BundleMember, coordinates: readonly BundleCoordinate[]): boolean {
	return coordinates.some(
		(coordinate) => coordinate.field === member.field && coordinate.direction === member.direction,
	);
}

/** A coordinate list's presence: `count/total ●○…`. */
function presence(list: readonly { present: boolean }[]): string {
	const marks = list.map(({ present }) => (present ? "●" : "○")).join("");
	return `${list.filter(({ present }) => present).length}/${list.length} ${marks}`;
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

/** Bundle facts shared by both shapes, each explained in a tooltip. */
function BundleFacts({
	bundle,
	learnedScale,
	offsetTip,
}: {
	bundle: BrainBundle;
	/** The learned scale, for an initialized scale slot (T25.F07). */
	learnedScale: number | undefined;
	offsetTip: string;
}) {
	return (
		<>
			<span className="text-slate-400">{bundle.kind}</span>
			<Tip content={offsetTip}>offset {bundle.offset}</Tip>
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

/** "1 step" or "n steps". */
function steps(n: number): string {
	return `${n} step${n === 1 ? "" : "s"}`;
}

/**
 * A row's header and member lines, collapsible and open by default. Only
 * the name toggles; the facts beside it carry their own tooltips.
 */
function BundleRowShell({
	name,
	facts,
	children,
}: {
	name: ReactNode;
	facts: ReactNode;
	children: ReactNode;
}) {
	const [open, setOpen] = useState(true);
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
				{facts}
			</div>
			{open ? <ul className="ml-4 space-y-0.5 text-slate-400">{children}</ul> : null}
		</li>
	);
}

function RingRow({
	bundle,
	ringRef,
	index,
	members,
	inputRefs,
	learnedScale,
}: {
	bundle: BrainBundle;
	ringRef: number;
	index: number;
	members: BundleMember[];
	inputRefs: InputRef[];
	learnedScale: number | undefined;
}) {
	const byField = [...members].sort((a, b) => (a.field ?? 0) - (b.field ?? 0));
	return (
		<BundleRowShell
			name={
				<span>
					Ring {refLabel(ringRef, inputRefs)} #{index}
				</span>
			}
			facts={
				<>
					<BundleFacts
						bundle={bundle}
						learnedScale={learnedScale}
						offsetTip={`Each ring cell votes ${bundle.kind} toward the direction ${steps(bundle.offset)} clockwise from that cell (0 = same, 4 = opposite).`}
					/>
					<Tip
						data-testid="edge-bundle-members"
						content="Members present, by ring cell from N clockwise (● present, ○ hole)."
					>
						{presence(bundle.coordinates)}
					</Tip>
				</>
			}
		>
			{byField.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-bundle-member">
					cell {directionName(member.field ?? 0)} → {bundle.kind}[
					{directionName(member.direction ?? 0)}] <MemberWeights member={member} />
				</li>
			))}
		</BundleRowShell>
	);
}

function PairRow({
	bundle,
	refIdx,
	pair,
	index,
	members,
	inputRefs,
	learnedScale,
}: {
	bundle: BrainBundle;
	refIdx: number;
	pair: number;
	index: number;
	members: BundleMember[];
	inputRefs: InputRef[];
	learnedScale: number | undefined;
}) {
	const ref = inputRefs[refIdx];
	const fields = directionPairs(ref)[pair];
	const label =
		ref === undefined || fields === undefined
			? `${refLabel(refIdx, inputRefs)} pair ${pair}`
			: `${describeInputRef(ref, fields.x).label}/${
					describeInputRef(ref, fields.y).label.split(".").pop() ?? fields.y
				}`;
	const role = (member: BundleMember): "x" | "y" | "?" =>
		member.field === fields?.x ? "x" : member.field === fields?.y ? "y" : "?";
	// A fact member (T25.F10) sits on the kind's drive: no sink direction.
	const directions = members.filter((member) => member.direction !== undefined);
	const factMembers = members.filter((member) => member.direction === undefined);
	const factLabel = (field: number) =>
		ref === undefined
			? `field ${field}`
			: (describeInputRef(ref, field).label.split(".").pop() ?? `field ${field}`);
	const extra = directions.filter((member) => !atCoordinate(member, bundle.coordinates));
	return (
		<BundleRowShell
			name={
				<span>
					Pair {label} #{index}
				</span>
			}
			facts={
				<>
					<BundleFacts
						bundle={bundle}
						learnedScale={learnedScale}
						offsetTip={`Votes ${bundle.kind} toward the sensed offset, rotated ${steps(bundle.offset)} clockwise (0 = toward, 4 = away).`}
					/>
					<Tip
						data-testid="edge-bundle-members"
						content={`Members present over the birth coordinates, in birth order (● present, ○ hole${
							extra.length > 0 ? `; +${extra.length} outside them, zero pattern` : ""
						}).`}
					>
						{presence(bundle.coordinates)}
						{extra.length > 0 ? ` +${extra.length}` : ""}
					</Tip>
					{factMembers.length > 0 ? (
						<Tip
							data-testid="edge-bundle-facts"
							content="Fact members present over the pair's declared facts, wired into the kind's drive; each weighs its own weight alone (● present, ○ hole)."
						>
							facts {presence(bundle.facts)}
						</Tip>
					) : null}
				</>
			}
		>
			{factMembers.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-bundle-fact">
					{factLabel(member.field ?? 0)} → {bundle.kind} drive <MemberWeights member={member} />
				</li>
			))}
			{directions.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-bundle-member">
					{role(member)} → {bundle.kind}[{directionName(member.direction ?? 0)}]{" "}
					<MemberWeights member={member} />
					{atCoordinate(member, bundle.coordinates) ? "" : " (zero pattern)"}
				</li>
			))}
		</BundleRowShell>
	);
}

/**
 * A Graph node's edge bundles (T25.F03, T25.F04): one row per bundle with its
 * shape, kind, offset, scale and the members present over its birth
 * coordinates (holes shown), each row expandable to its member edges and
 * open by default.
 */
export function EdgeBundleList({ graphDef, inputRefs, learning }: EdgeBundleListProps) {
	const bundles = graphDef.bundles;
	if (bundles.length === 0) return null;
	const members = collectBundleMembers(graphDef, learning);
	const join = joinLearning(learning);
	return (
		<section aria-label="Edge bundles" className="mt-2">
			<h4 className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-400">
				Edge bundles ({bundles.length})
			</h4>
			<ul className="space-y-1 font-mono text-[10px] text-slate-300">
				{bundles.map((bundle, index) =>
					"Ring" in bundle.shape ? (
						<RingRow
							// biome-ignore lint/suspicious/noArrayIndexKey: a BundleId is the bundle's index
							key={index}
							bundle={bundle}
							ringRef={bundle.shape.Ring.ring_ref}
							index={index}
							members={members[index] ?? []}
							inputRefs={inputRefs}
							learnedScale={join.learnedScale(index, bundle)}
						/>
					) : (
						<PairRow
							// biome-ignore lint/suspicious/noArrayIndexKey: a BundleId is the bundle's index
							key={index}
							bundle={bundle}
							refIdx={bundle.shape.DirectionPair.ref_idx}
							pair={bundle.shape.DirectionPair.pair}
							index={index}
							members={members[index] ?? []}
							inputRefs={inputRefs}
							learnedScale={join.learnedScale(index, bundle)}
						/>
					),
				)}
			</ul>
		</section>
	);
}

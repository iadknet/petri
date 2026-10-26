import { type ReactNode, useState } from "react";
import type { BrainGroup } from "../../types/generated/BrainGroup.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { GroupCoordinate } from "../../types/generated/GroupCoordinate.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { Scope } from "../../types/generated/Scope.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { joinLearning } from "./brainLearning.ts";
import { directionName } from "./directions.ts";
import { composeTerms, scopeLabel } from "./edgeGroupWeights.ts";
import { describeInputRef, directionPairs, formatInputRef } from "./inputRefUtils.ts";

interface EdgeGroupListProps {
	graphDef: GraphBrain;
	inputRefs: InputRef[];
	/** This node's `learning` entry: the weights in force and learned scales. */
	learning: NodeLearning | null;
}

export interface GroupMember {
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

/** The reference a group reads, or its dangling index. */
function refLabel(refIdx: number, inputRefs: InputRef[]): string {
	const ref = inputRefs[refIdx];
	return ref === undefined ? `ref ${refIdx} (missing)` : formatInputRef(ref);
}

/** Each group's members in sink order, as the brain view lists them. */
export function collectGroupMembers(
	graphDef: GraphBrain,
	learning: NodeLearning | null,
): GroupMember[][] {
	const join = joinLearning(learning);
	const members: GroupMember[][] = graphDef.groups.map(() => []);
	graphDef.output_sinks.forEach((sink, sinkIndex) => {
		sink.edges.forEach((edge, edgeIndex) => {
			const member = edge.member;
			const list = member ? members[member.group] : undefined;
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
function atCoordinate(member: GroupMember, coordinates: readonly GroupCoordinate[]): boolean {
	return coordinates.some(
		(coordinate) => coordinate.field === member.field && coordinate.direction === member.direction,
	);
}

/** A coordinate list's presence: `count/total ●○…`. */
function presence(list: readonly { present: boolean }[]): string {
	const marks = list.map(({ present }) => (present ? "●" : "○")).join("");
	return `${list.filter(({ present }) => present).length}/${list.length} ${marks}`;
}

/** A member's weights, its composition from the group in a tooltip. */
function MemberWeights({ member }: { member: GroupMember }) {
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
			<span data-testid="edge-group-member-scope"> {scopeLabel(member.scope)}</span>
		</Tip>
	);
}

/** Group facts shared by both shapes, each explained in a tooltip. */
function GroupFacts({
	group,
	learnedScale,
	offsetTip,
}: {
	group: BrainGroup;
	/** The learned scale, for an initialized scale slot (T25.F07). */
	learnedScale: number | undefined;
	offsetTip: string;
}) {
	return (
		<>
			<span className="text-slate-400">{group.kind}</span>
			<Tip content={offsetTip}>offset {group.offset}</Tip>
			<Tip content="The group's shared weight: each member's weight is scale × its pattern + its own correction.">
				scale={group.scale.toFixed(3)}
			</Tip>
			{learnedScale !== undefined ? (
				<Tip content="The learned scale the group's group-scope members moved; every member weighs through it.">
					<span data-testid="edge-group-learned-scale">learned={learnedScale.toFixed(3)}</span>
				</Tip>
			) : null}
			<Tip
				content={
					group.inherit_scale
						? "inherit_scale: a parent's learned scale seeds the child's."
						: "No inherit_scale: a child starts from the genomic scale."
				}
			>
				<span data-testid="edge-group-inherit-scale">
					{group.inherit_scale ? "inherits scale" : "genomic scale at birth"}
				</span>
			</Tip>
			{group.retention < 1 ? (
				<Tip content="The learned scale decays toward its birth value, keeping this share of its learned change each tick.">
					<span data-testid="edge-group-retention">retention={group.retention.toFixed(4)}</span>
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
function GroupRowShell({
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
		<li data-testid="edge-group" className="space-y-0.5">
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
	group,
	ringRef,
	index,
	members,
	inputRefs,
	learnedScale,
}: {
	group: BrainGroup;
	ringRef: number;
	index: number;
	members: GroupMember[];
	inputRefs: InputRef[];
	learnedScale: number | undefined;
}) {
	const byField = [...members].sort((a, b) => (a.field ?? 0) - (b.field ?? 0));
	return (
		<GroupRowShell
			name={
				<span>
					Ring {refLabel(ringRef, inputRefs)} #{index}
				</span>
			}
			facts={
				<>
					<GroupFacts
						group={group}
						learnedScale={learnedScale}
						offsetTip={`Each ring cell votes ${group.kind} toward the direction ${steps(group.offset)} clockwise from that cell (0 = same, 4 = opposite).`}
					/>
					<Tip
						data-testid="edge-group-members"
						content="Members present, by ring cell from N clockwise (● present, ○ hole)."
					>
						{presence(group.coordinates)}
					</Tip>
				</>
			}
		>
			{byField.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					cell {directionName(member.field ?? 0)} → {group.kind}[
					{directionName(member.direction ?? 0)}] <MemberWeights member={member} />
				</li>
			))}
		</GroupRowShell>
	);
}

function PairRow({
	group,
	refIdx,
	pair,
	index,
	members,
	inputRefs,
	learnedScale,
}: {
	group: BrainGroup;
	refIdx: number;
	pair: number;
	index: number;
	members: GroupMember[];
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
	const role = (member: GroupMember): "x" | "y" | "?" =>
		member.field === fields?.x ? "x" : member.field === fields?.y ? "y" : "?";
	// A fact member (T25.F10) sits on the kind's drive: no sink direction.
	const directions = members.filter((member) => member.direction !== undefined);
	const factMembers = members.filter((member) => member.direction === undefined);
	const factLabel = (field: number) =>
		ref === undefined
			? `field ${field}`
			: (describeInputRef(ref, field).label.split(".").pop() ?? `field ${field}`);
	const extra = directions.filter((member) => !atCoordinate(member, group.coordinates));
	return (
		<GroupRowShell
			name={
				<span>
					Pair {label} #{index}
				</span>
			}
			facts={
				<>
					<GroupFacts
						group={group}
						learnedScale={learnedScale}
						offsetTip={`Votes ${group.kind} toward the sensed offset, rotated ${steps(group.offset)} clockwise (0 = toward, 4 = away).`}
					/>
					<Tip
						data-testid="edge-group-members"
						content={`Members present over the birth coordinates, in birth order (● present, ○ hole${
							extra.length > 0 ? `; +${extra.length} outside them, zero pattern` : ""
						}).`}
					>
						{presence(group.coordinates)}
						{extra.length > 0 ? ` +${extra.length}` : ""}
					</Tip>
					{factMembers.length > 0 ? (
						<Tip
							data-testid="edge-group-facts"
							content="Fact members present over the pair's declared facts, wired into the kind's drive; each weighs its own weight alone (● present, ○ hole)."
						>
							facts {presence(group.facts)}
						</Tip>
					) : null}
				</>
			}
		>
			{factMembers.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-fact">
					{factLabel(member.field ?? 0)} → {group.kind} drive <MemberWeights member={member} />
				</li>
			))}
			{directions.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					{role(member)} → {group.kind}[{directionName(member.direction ?? 0)}]{" "}
					<MemberWeights member={member} />
					{atCoordinate(member, group.coordinates) ? "" : " (zero pattern)"}
				</li>
			))}
		</GroupRowShell>
	);
}

/**
 * A Graph node's edge groups (T25.F03, T25.F04): one row per group with its
 * shape, kind, offset, scale and the members present over its birth
 * coordinates (holes shown), each row expandable to its member edges and
 * open by default.
 */
export function EdgeGroupList({ graphDef, inputRefs, learning }: EdgeGroupListProps) {
	const groups = graphDef.groups;
	if (groups.length === 0) return null;
	const members = collectGroupMembers(graphDef, learning);
	const join = joinLearning(learning);
	return (
		<section aria-label="Edge groups" className="mt-2">
			<h4 className="mb-1 text-[10px] font-medium uppercase tracking-wider text-slate-400">
				Edge groups ({groups.length})
			</h4>
			<ul className="space-y-1 font-mono text-[10px] text-slate-300">
				{groups.map((group, index) =>
					"Ring" in group.shape ? (
						<RingRow
							// biome-ignore lint/suspicious/noArrayIndexKey: a GroupId is the group's index
							key={index}
							group={group}
							ringRef={group.shape.Ring.ring_ref}
							index={index}
							members={members[index] ?? []}
							inputRefs={inputRefs}
							learnedScale={join.learnedScale(index, group)}
						/>
					) : (
						<PairRow
							// biome-ignore lint/suspicious/noArrayIndexKey: a GroupId is the group's index
							key={index}
							group={group}
							refIdx={group.shape.DirectionPair.ref_idx}
							pair={group.shape.DirectionPair.pair}
							index={index}
							members={members[index] ?? []}
							inputRefs={inputRefs}
							learnedScale={join.learnedScale(index, group)}
						/>
					),
				)}
			</ul>
		</section>
	);
}

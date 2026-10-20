import { type ReactNode, useState } from "react";
import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type { EdgeGroup, GraphBackendDef, InputReference, Scope } from "../../types/genome.ts";
import { Tip } from "../config-panel/shared/Tooltip.tsx";
import { directionName } from "./directions.ts";
import { composeTerms, memberPattern, scopeLabel, sinkDirection } from "./edgeGroupWeights.ts";
import { learnedScaleLookup } from "./graphFullModel.ts";
import { describeInputRef, directionPairs, formatInputRef } from "./inputRefUtils.ts";

const RING_CHANNELS = 8;

/**
 * The twelve `(field, relative direction)` coordinates a new pair group
 * materializes (T25.F04): `x` where the compass `x` is non-zero (NE, E, SE,
 * SW, W, NW), `y` where the compass `y` is (N, NE, SE, S, SW, NW).
 */
const PAIR_BIRTH: readonly (readonly ["x" | "y", number])[] = [
	...[1, 2, 3, 5, 6, 7].map((d) => ["x", d] as const),
	...[0, 1, 3, 4, 5, 7].map((d) => ["y", d] as const),
];
const PAIR_BIRTH_KEYS = PAIR_BIRTH.map(([field, d]) => `${field}:${d}`);
const PAIR_BIRTH_SET = new Set(PAIR_BIRTH_KEYS);

interface EdgeGroupListProps {
	graphDef: GraphBackendDef;
	inputRefs: InputReference[];
	/** The server's member weights and learned scales for this node. */
	graphWeights: NodeGraphWeights | null;
}

export interface GroupMember {
	/** The field the member reads: a ring cell, or a pair's `x` or `y`. */
	channel: number;
	/** The direction of the sink the member sits on, when it is one. */
	direction: number | undefined;
	sink: number;
	edge: number;
	ownWeight: number;
	/** How the member learns when plastic (T25.F07). */
	scope: Scope;
	/** The server's genomic weight, when the payload carries it. */
	genomic: number | undefined;
	/** The server's weight in force, when the payload carries it (T25.F07). */
	applied: number | undefined;
}

/** The reference a group reads, or its dangling index. */
function refLabel(refIdx: number, inputRefs: InputReference[]): string {
	const ref = inputRefs[refIdx];
	return ref === undefined ? `ref ${refIdx} (missing)` : formatInputRef(ref);
}

/** Each group's members in sink order, read from the genome's memberships. */
export function collectGroupMembers(
	graphDef: GraphBackendDef,
	graphWeights: NodeGraphWeights | null,
): GroupMember[][] {
	const weights = new Map(
		(graphWeights?.edges ?? [])
			.filter((entry) => entry.surface === "sink")
			.map((entry) => [`${entry.container}:${entry.edge}`, entry]),
	);
	const members: GroupMember[][] = (graphDef.groups ?? []).map(() => []);
	graphDef.output_sinks.forEach((sink, sinkIndex) => {
		sink.inputs.forEach((edge, edgeIndex) => {
			const membership = edge.membership;
			const list = membership ? members[membership.group] : undefined;
			if (!membership || !list || !("InputLeaf" in edge.source)) return;
			const entry = weights.get(`${sinkIndex}:${edgeIndex}`);
			list.push({
				channel: edge.source.InputLeaf.sub_idx,
				direction: sinkDirection(sink.kind),
				sink: sinkIndex,
				edge: edgeIndex,
				ownWeight: edge.own_weight,
				scope: membership.scope,
				genomic: entry?.genomic,
				applied: entry?.applied,
			});
		});
	});
	return members;
}

/** A member's weights, its composition from the group in a tooltip. */
function MemberWeights({
	member,
	group,
	inputRefs,
}: {
	member: GroupMember;
	group: EdgeGroup;
	inputRefs: InputReference[];
}) {
	const pattern = memberPattern(group, inputRefs, member.channel, member.direction);
	return (
		<Tip
			content={
				<>
					<span className="block text-slate-400">scale × pattern + correction</span>
					<span className="block font-mono">
						{composeTerms(member.ownWeight, group.scale, pattern)}
					</span>
				</>
			}
		>
			own={member.ownWeight.toFixed(3)}
			{member.genomic !== undefined ? ` genomic=${member.genomic.toFixed(3)}` : ""}
			{member.applied !== undefined ? ` applied=${member.applied.toFixed(3)}` : ""}
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
	group: EdgeGroup;
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
	group: EdgeGroup;
	ringRef: number;
	index: number;
	members: GroupMember[];
	inputRefs: InputReference[];
	learnedScale: number | undefined;
}) {
	const present = new Set(members.map((member) => member.channel));
	const byChannel = [...members].sort((a, b) => a.channel - b.channel);
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
						{present.size}/{RING_CHANNELS}{" "}
						{Array.from({ length: RING_CHANNELS }, (_, channel) =>
							present.has(channel) ? "●" : "○",
						).join("")}
					</Tip>
				</>
			}
		>
			{byChannel.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					cell {directionName(member.channel)} → {group.kind}[
					{directionName(member.channel + group.offset)}]{" "}
					<MemberWeights member={member} group={group} inputRefs={inputRefs} />
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
	group: EdgeGroup;
	refIdx: number;
	pair: number;
	index: number;
	members: GroupMember[];
	inputRefs: InputReference[];
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
		member.channel === fields?.x ? "x" : member.channel === fields?.y ? "y" : "?";
	const relative = (member: GroupMember) =>
		member.direction === undefined ? undefined : (member.direction - group.offset + 8) % 8;
	const coordinate = (member: GroupMember) => `${role(member)}:${relative(member)}`;
	const present = new Set(members.map(coordinate));
	const presentAtBirth = PAIR_BIRTH_KEYS.map((key) => present.has(key));
	const extra = members.filter((member) => !PAIR_BIRTH_SET.has(coordinate(member)));
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
						content={`Members present over the twelve birth coordinates: x at NE E SE SW W NW, then y at N NE SE S SW NW, relative to the offset (● present, ○ hole${
							extra.length > 0 ? `; +${extra.length} outside them, zero pattern` : ""
						}).`}
					>
						{presentAtBirth.filter(Boolean).length}/{PAIR_BIRTH.length}{" "}
						{presentAtBirth.map((here) => (here ? "●" : "○")).join("")}
						{extra.length > 0 ? ` +${extra.length}` : ""}
					</Tip>
				</>
			}
		>
			{members.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					{role(member)}:{directionName(relative(member) ?? 0)} → {group.kind}[
					{directionName(member.direction ?? 0)}]{" "}
					<MemberWeights member={member} group={group} inputRefs={inputRefs} />
					{PAIR_BIRTH_SET.has(coordinate(member)) ? "" : " (zero pattern)"}
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
export function EdgeGroupList({ graphDef, inputRefs, graphWeights }: EdgeGroupListProps) {
	const groups = graphDef.groups ?? [];
	if (groups.length === 0) return null;
	const members = collectGroupMembers(graphDef, graphWeights);
	const learned = learnedScaleLookup(graphWeights);
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
							learnedScale={learned(index)}
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
							learnedScale={learned(index)}
						/>
					),
				)}
			</ul>
		</section>
	);
}

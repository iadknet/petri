import { type ReactNode, useState } from "react";
import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type {
	EdgeGroup,
	GraphBackendDef,
	InputReference,
	OutputSinkKind,
} from "../../types/genome.ts";
import { directionName } from "./directions.ts";
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

interface EdgeGroupListProps {
	graphDef: GraphBackendDef;
	inputRefs: InputReference[];
	/** The server's genomic member weights for this node. */
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
	/** The server's genomic weight, when the payload carries it. */
	genomic: number | undefined;
}

/** The direction of a directed vote sink. */
function sinkDirection(kind: OutputSinkKind | undefined): number | undefined {
	if (kind === undefined || !("ActionVote" in kind)) return undefined;
	const vote = kind.ActionVote;
	if (typeof vote !== "object") return undefined;
	if ("Move" in vote) return vote.Move;
	if ("Reproduce" in vote) return vote.Reproduce;
	if ("StealEnergy" in vote) return vote.StealEnergy;
	return undefined;
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
	const genomic = new Map(
		(graphWeights?.edges ?? [])
			.filter((entry) => entry.surface === "sink")
			.map((entry) => [`${entry.container}:${entry.edge}`, entry.genomic]),
	);
	const members: GroupMember[][] = (graphDef.groups ?? []).map(() => []);
	graphDef.output_sinks.forEach((sink, sinkIndex) => {
		sink.inputs.forEach((edge, edgeIndex) => {
			const group = edge.membership?.group;
			if (group === undefined || !members[group] || !("InputLeaf" in edge.source)) return;
			members[group].push({
				channel: edge.source.InputLeaf.sub_idx,
				direction: sinkDirection(sink.kind),
				sink: sinkIndex,
				edge: edgeIndex,
				ownWeight: edge.own_weight,
				genomic: genomic.get(`${sinkIndex}:${edgeIndex}`),
			});
		});
	});
	return members;
}

function memberWeights(member: GroupMember): string {
	return `own=${member.ownWeight.toFixed(3)}${
		member.genomic !== undefined ? ` genomic=${member.genomic.toFixed(3)}` : ""
	}`;
}

/** A row's header and member lines, collapsible and open by default. */
function GroupRowShell({
	header,
	children,
}: {
	header: ReactNode;
	children: ReactNode;
}) {
	const [open, setOpen] = useState(true);
	return (
		<li data-testid="edge-group" className="space-y-0.5">
			<button
				type="button"
				aria-expanded={open}
				onClick={() => setOpen((value) => !value)}
				className="flex w-full flex-wrap gap-2 text-left hover:text-slate-100"
			>
				<span aria-hidden="true">{open ? "▾" : "▸"}</span>
				{header}
			</button>
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
}: {
	group: EdgeGroup;
	ringRef: number;
	index: number;
	members: GroupMember[];
	inputRefs: InputReference[];
}) {
	const present = new Set(members.map((member) => member.channel));
	const byChannel = [...members].sort((a, b) => a.channel - b.channel);
	return (
		<GroupRowShell
			header={
				<>
					<span>
						Ring {refLabel(ringRef, inputRefs)} #{index}
					</span>
					<span className="text-slate-400">{group.kind}</span>
					<span
						title={`Each ring cell votes ${group.kind} toward the direction ${group.offset} steps clockwise from that cell (0 = same, 4 = opposite).`}
					>
						offset {group.offset}
					</span>
					<span>scale={group.scale.toFixed(3)}</span>
					<span data-testid="edge-group-members" title="Members present, by ring cell">
						{present.size}/{RING_CHANNELS}{" "}
						{Array.from({ length: RING_CHANNELS }, (_, channel) =>
							present.has(channel) ? "●" : "○",
						).join("")}
					</span>
				</>
			}
		>
			{byChannel.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					cell {directionName(member.channel)} → {group.kind}[
					{directionName(member.channel + group.offset)}] {memberWeights(member)}
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
}: {
	group: EdgeGroup;
	refIdx: number;
	pair: number;
	index: number;
	members: GroupMember[];
	inputRefs: InputReference[];
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
	const birth = new Set(PAIR_BIRTH.map(([field, d]) => `${field}:${d}`));
	const extra = members.filter((member) => !birth.has(coordinate(member)));
	return (
		<GroupRowShell
			header={
				<>
					<span>
						Pair {label} #{index}
					</span>
					<span className="text-slate-400">{group.kind}</span>
					<span
						title={`Votes ${group.kind} toward the sensed offset, rotated ${group.offset} steps clockwise (0 = toward, 4 = away).`}
					>
						offset {group.offset}
					</span>
					<span>scale={group.scale.toFixed(3)}</span>
					<span
						data-testid="edge-group-members"
						title="Members present over the twelve birth coordinates: x at NE E SE SW W NW, then y at N NE SE S SW NW (relative to the offset)"
					>
						{PAIR_BIRTH.filter(([field, d]) => present.has(`${field}:${d}`)).length}/
						{PAIR_BIRTH.length}{" "}
						{PAIR_BIRTH.map(([field, d]) => (present.has(`${field}:${d}`) ? "●" : "○")).join("")}
						{extra.length > 0 ? ` +${extra.length}` : ""}
					</span>
				</>
			}
		>
			{members.map((member) => (
				<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
					{role(member)}:{directionName(relative(member) ?? 0)} → {group.kind}[
					{directionName(member.direction ?? 0)}] {memberWeights(member)}
					{birth.has(coordinate(member)) ? "" : " (zero pattern)"}
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
						/>
					),
				)}
			</ul>
		</section>
	);
}

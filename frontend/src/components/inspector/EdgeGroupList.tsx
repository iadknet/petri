import { useState } from "react";
import type { NodeGraphWeights } from "../../types/creature-detail.ts";
import type { EdgeGroup, GraphBackendDef, InputReference } from "../../types/genome.ts";
import { formatInputRef } from "./inputRefUtils.ts";

const DIRECTIONS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;
const RING_CHANNELS = 8;

interface EdgeGroupListProps {
	graphDef: GraphBackendDef;
	inputRefs: InputReference[];
	/** The server's genomic member weights for this node. */
	graphWeights: NodeGraphWeights | null;
}

export interface GroupMember {
	channel: number;
	sink: number;
	edge: number;
	ownWeight: number;
	/** The server's genomic weight, when the payload carries it. */
	genomic: number | undefined;
}

/** The ring a group reads, or its dangling index. */
function ringLabel(refIdx: number, inputRefs: InputReference[]): string {
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
				sink: sinkIndex,
				edge: edgeIndex,
				ownWeight: edge.own_weight,
				genomic: genomic.get(`${sinkIndex}:${edgeIndex}`),
			});
		});
	});
	return members;
}

function GroupRow({
	group,
	index,
	members,
	inputRefs,
}: {
	group: EdgeGroup;
	index: number;
	members: GroupMember[];
	inputRefs: InputReference[];
}) {
	const [open, setOpen] = useState(true);
	const present = new Set(members.map((member) => member.channel));
	const byChannel = [...members].sort((a, b) => a.channel - b.channel);
	return (
		<li data-testid="edge-group" className="space-y-0.5">
			<button
				type="button"
				aria-expanded={open}
				onClick={() => setOpen((value) => !value)}
				className="flex w-full flex-wrap gap-2 text-left hover:text-slate-100"
			>
				<span aria-hidden="true">{open ? "▾" : "▸"}</span>
				<span>
					Ring {ringLabel(group.shape.Ring.ring_ref, inputRefs)} #{index}
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
			</button>
			{open ? (
				<ul className="ml-4 space-y-0.5 text-slate-400">
					{byChannel.map((member) => (
						<li key={`${member.sink}:${member.edge}`} data-testid="edge-group-member">
							cell {DIRECTIONS[member.channel % RING_CHANNELS]} → {group.kind}[
							{DIRECTIONS[(member.channel + group.offset) % RING_CHANNELS]}] own=
							{member.ownWeight.toFixed(3)}
							{member.genomic !== undefined ? ` genomic=${member.genomic.toFixed(3)}` : ""}
						</li>
					))}
				</ul>
			) : null}
		</li>
	);
}

/**
 * A Graph node's edge groups (T25.F03): one row per group with its shape,
 * kind, offset, scale and the members present over eight (holes shown), each
 * row expandable to its member edges and open by default.
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
				{groups.map((group, index) => (
					<GroupRow
						// biome-ignore lint/suspicious/noArrayIndexKey: a GroupId is the group's index
						key={index}
						group={group}
						index={index}
						members={members[index] ?? []}
						inputRefs={inputRefs}
					/>
				))}
			</ul>
		</section>
	);
}

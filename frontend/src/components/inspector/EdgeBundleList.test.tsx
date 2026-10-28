import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { graphBrain, ringBundle } from "../../test/brainFixtures.ts";
import type { BrainBundle } from "../../types/generated/BrainBundle.ts";
import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { BrainSink } from "../../types/generated/BrainSink.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { Site } from "../../types/generated/Site.ts";
import {
	EdgeBundleList,
	collectBundleMembers,
	coordinateKey,
	indexMembers,
} from "./EdgeBundleList.tsx";

/** A member of bundle 0 as the brain view resolves it: every number given,
 * none composed here. */
function member(
	field: number,
	site: Site,
	terms: { own: number; genomic: number; pattern: number; scale: number },
): BrainEdge {
	return {
		source: { InputLeaf: { ref_idx: 0, sub_idx: field } },
		own: terms.own,
		genomic: terms.genomic,
		applied: terms.genomic,
		member: { bundle: 0, scope: "Edge", site, field, pattern: terms.pattern, scale: terms.scale },
	};
}

/** Eight `kind` direction sinks, empty. */
function directionSinks(kind: "Move" | "StealEnergy"): BrainSink[] {
	return Array.from({ length: 8 }, (_, d) => ({
		kind: { ActionVote: { Directed: [kind, d] } },
		edges: [] as BrainEdge[],
	}));
}

/** A Move ring bundle at offset 1 on ring 1, missing its channel-3 member. */
function grouped(): GraphBrain {
	const sinks = directionSinks("Move");
	for (let channel = 0; channel < 8; channel++) {
		if (channel === 3) continue;
		const own = channel === 0 ? 0.25 : 0;
		sinks[(channel + 1) % 8]?.edges.push(
			member(
				channel,
				{ Direction: (channel + 1) % 8 },
				{ own, genomic: 0.5 + own, pattern: 1, scale: 0.5 },
			),
		);
	}
	const bundle = ringBundle(1, { offset: 1, scale: 0.5 });
	const missing = bundle.coordinates[3];
	if (missing) missing.present = false;
	return graphBrain({ output_sinks: sinks, bundles: [bundle] });
}

const refs = [
	{ family: "world", key: "FoodHere" },
	{ family: "world", key: "NeighborBarrierRing" },
] as const;

/** The coordinates of a `DirectionPair` bundle on AreaFoodSummary pair 1
 * (x field 3, y field 4, facts 0, 5, 6) at `offset`, in birth order: the
 * twelve compass coordinates, then the facts on the drive; `absent` lists
 * the coordinates without a member. */
function pairCoordinates(
	offset: number,
	absent: (field: number, site: Site) => boolean = () => false,
): BrainBundle["coordinates"] {
	const compass = (field: number, relatives: number[]) =>
		relatives.map((r) => ({ field, site: { Direction: (r + offset) % 8 } as Site }));
	return [
		...compass(3, [1, 2, 3, 5, 6, 7]),
		...compass(4, [0, 1, 3, 4, 5, 7]),
		...[0, 5, 6].map((field) => ({ field, site: "Drive" as Site })),
	].map((c) => ({ ...c, present: !absent(c.field, c.site) }));
}

describe("EdgeBundleList", () => {
	it("lists each bundle's wiring, kind, offset, scale and members by coordinate with holes", () => {
		render(
			<EdgeBundleList
				graphDef={grouped()}
				inputRefs={[...refs]}
				learning={{
					node_id: 0,
					scales: [],
					edges: [{ surface: "sink", container: 1, edge: 0, applied: 0.75 }],
				}}
			/>,
		);
		const row = screen.getByTestId("edge-bundle");
		expect(row).toHaveTextContent("Ring");
		expect(row).toHaveTextContent("Move");
		expect(row).toHaveTextContent("offset 1");
		expect(row).toHaveTextContent("scale=0.500");
		expect(screen.getByTestId("edge-bundle-members")).toHaveTextContent("7/8 ●●●○●●●●");
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members).toHaveLength(7);
		expect(members[0]).toHaveTextContent("→ Move[NE] own=0.250 genomic=0.750");
		const holes = screen.getAllByTestId("edge-bundle-hole");
		expect(holes).toHaveLength(1);
		expect(holes[0]).toHaveTextContent("→ Move[S] (hole)");
		fireEvent.mouseEnter(screen.getAllByText(/own=0\.250/)[0] as HTMLElement);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(
			"scale × pattern + correction0.500 × 1.00 + 0.250 = 0.750",
		);
	});

	it("shows the learned scale, inherit_scale, and each member's scope and applied weight", () => {
		const def = grouped();
		const bundle = def.bundles[0];
		if (bundle) bundle.inherit_scale = true;
		const first = def.output_sinks[1]?.edges[0]?.member;
		if (first) first.scope = "Group";
		render(
			<EdgeBundleList
				graphDef={def}
				inputRefs={[...refs]}
				learning={{
					node_id: 0,
					scales: [{ bundle: 0, learned: 1.25, initialized: true }],
					edges: [{ surface: "sink", container: 1, edge: 0, applied: 1.5 }],
				}}
			/>,
		);
		expect(screen.getByTestId("edge-bundle")).toHaveTextContent("scale=0.500");
		expect(screen.getByTestId("edge-bundle-learned-scale")).toHaveTextContent("learned=1.250");
		expect(screen.getByTestId("edge-bundle-inherit-scale")).toHaveTextContent("inherits scale");
		expect(screen.queryByTestId("edge-bundle-retention")).toBeNull();
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members[0]).toHaveTextContent("own=0.250 genomic=0.750 applied=1.500 group scope");
		expect(members[1]).toHaveTextContent("applied=0.500 edge scope");
	});

	it("shows a bundle's retention when it decays (T25.F09)", () => {
		const def = grouped();
		const bundle = def.bundles[0];
		if (bundle) bundle.retention = 0.99;
		render(<EdgeBundleList graphDef={def} inputRefs={[]} learning={null} />);
		expect(screen.getByTestId("edge-bundle-retention")).toHaveTextContent("retention=0.9900");
	});

	it("takes the learned scale from the brain when the poll lists none", () => {
		const def = grouped();
		render(<EdgeBundleList graphDef={def} inputRefs={[]} learning={null} />);
		expect(screen.queryByTestId("edge-bundle-learned-scale")).toBeNull();
		expect(screen.getByTestId("edge-bundle-inherit-scale")).toHaveTextContent(
			"genomic scale at birth",
		);
		const bundle = def.bundles[0];
		if (bundle) bundle.learned_scale = 2;
		render(<EdgeBundleList graphDef={def} inputRefs={[]} learning={null} />);
		expect(screen.getByTestId("edge-bundle-learned-scale")).toHaveTextContent("learned=2.000");
	});

	it("explains the offset and the members glyphs in tooltips, outside the toggle", () => {
		render(<EdgeBundleList graphDef={grouped()} inputRefs={[]} learning={null} />);
		const toggle = screen.getByRole("button");
		const offset = screen.getByText("offset 1");
		expect(toggle).not.toContainElement(offset);
		expect(offset).not.toHaveAttribute("title");
		fireEvent.mouseEnter(offset);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/1 step clockwise/);
		fireEvent.mouseLeave(offset);
		fireEvent.focus(screen.getByTestId("edge-bundle-members"));
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/in birth order/);
	});

	it("collapses and reopens a bundle's member rows, open by default", () => {
		render(<EdgeBundleList graphDef={grouped()} inputRefs={[]} learning={null} />);
		const toggle = screen.getByRole("button");
		expect(toggle).toHaveAttribute("aria-expanded", "true");
		expect(screen.getAllByTestId("edge-bundle-member")).toHaveLength(7);
		expect(screen.getByTestId("edge-bundle")).toHaveTextContent("ref 1 (missing)");
		fireEvent.click(toggle);
		expect(toggle).toHaveAttribute("aria-expanded", "false");
		expect(screen.queryAllByTestId("edge-bundle-member")).toHaveLength(0);
		fireEvent.click(toggle);
		expect(screen.getAllByTestId("edge-bundle-member")).toHaveLength(7);
	});

	it("lists a pair bundle's compass and fact members by coordinate, the facts on the drive", () => {
		// The x coordinate on direction 4 and the fact `max` (field 6) are
		// holes; a fact member weighs `scale × 1 + own` like every member.
		const absent = (field: number, site: Site) =>
			(field === 3 && typeof site !== "string" && site.Direction === 4) ||
			(field === 6 && site === "Drive");
		const coordinates = pairCoordinates(2, absent);
		const sinks = directionSinks("StealEnergy");
		const drive: BrainSink = { kind: { ActionVote: { Drive: "StealEnergy" } }, edges: [] };
		for (const c of coordinates.filter((c) => c.present)) {
			if (c.site === "Drive") {
				const own = c.field === 5 ? 0.25 : 0;
				drive.edges.push(member(c.field, "Drive", { own, genomic: 2 + own, pattern: 1, scale: 2 }));
			} else if (typeof c.site !== "string") {
				sinks[c.site.Direction]?.edges.push(
					member(c.field, c.site, { own: 0, genomic: 1, pattern: 0.5, scale: 2 }),
				);
			}
		}
		const bundle: BrainBundle = {
			...ringBundle(0),
			source: { Sensor: { ref_idx: 0, field_set: 1 } },
			wiring: { DirectionPair: { kind: "StealEnergy", offset: 2 } },
			scale: 2,
			coordinates,
		};
		render(
			<EdgeBundleList
				graphDef={graphBrain({ output_sinks: [...sinks, drive], bundles: [bundle] })}
				inputRefs={[{ family: "world", key: "AreaFoodSummary", food_type: 0 }]}
				learning={null}
			/>,
		);
		const row = screen.getByTestId("edge-bundle");
		expect(row).toHaveTextContent("Pair");
		expect(row).toHaveTextContent("set 1 #0");
		expect(row).toHaveTextContent("offset 2");
		expect(row).toHaveTextContent("scale=2.000");
		expect(screen.getByTestId("edge-bundle-members")).toHaveTextContent("13/15 ●○●●●●●●●●●●●●○");
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members).toHaveLength(13);
		const drives = members.filter((m) => m.textContent?.includes("StealEnergy drive"));
		expect(drives).toHaveLength(2);
		expect(drives[1]).toHaveTextContent("own=0.250 genomic=2.250");
		expect(screen.getAllByTestId("edge-bundle-hole")).toHaveLength(2);
		fireEvent.mouseEnter(screen.getByText(/own=0\.250/));
		expect(screen.getByTestId("tooltip")).toHaveTextContent("2.000 × 1.00 + 0.250 = 2.250");
	});

	it("names a Plain bundle's sink and a memory bank's slots", () => {
		const sink: BrainSink = { kind: { ActionVote: "Eat" }, edges: [] };
		for (let slot = 0; slot < 2; slot++) {
			sink.edges.push(member(slot, "Sink", { own: 0, genomic: 0.5, pattern: 1, scale: 0.5 }));
		}
		const bundle: BrainBundle = {
			...ringBundle(0),
			source: { Memory: { previous: true } },
			wiring: { Plain: { sink: "Eat" } },
			scale: 0.5,
			coordinates: [0, 1, 2].map((field) => ({ field, site: "Sink", present: field < 2 })),
		};
		render(
			<EdgeBundleList
				graphDef={graphBrain({ output_sinks: [sink], bundles: [bundle] })}
				inputRefs={[]}
				learning={null}
			/>,
		);
		const row = screen.getByTestId("edge-bundle");
		expect(row).toHaveTextContent("Plain → Eat memory (previous) #0");
		expect(screen.queryByText(/offset/)).toBeNull();
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members[0]).toHaveTextContent("prev[0] → Eat own=0.000 genomic=0.500");
		expect(screen.getByTestId("edge-bundle-hole")).toHaveTextContent("prev[2] → Eat (hole)");
	});

	it("renders nothing without bundles", () => {
		const { container } = render(
			<EdgeBundleList graphDef={graphBrain()} inputRefs={[]} learning={null} />,
		);
		expect(container.innerHTML).toBe("");
	});
});

describe("indexMembers", () => {
	it("finds every member of the widest queue bundle by coordinate", () => {
		const width = 65_536;
		const sink: BrainSink = { kind: { ActionVote: "Eat" }, edges: [] };
		for (let field = 0; field < width; field++) {
			sink.edges.push(member(field, "Sink", { own: 0, genomic: 0, pattern: 1, scale: 0 }));
		}
		const graph = graphBrain({ output_sinks: [sink], bundles: [ringBundle(0)] });
		const index = indexMembers(collectBundleMembers(graph, null)[0] ?? []);
		let found = 0;
		for (let field = 0; field < width; field++) {
			if (index.get(coordinateKey(field, "Sink"))?.field === field) found++;
		}
		expect(found).toBe(width);
		expect(index.get(coordinateKey(0, { Direction: 0 }))).toBeUndefined();
	});
});

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { graphBrain, ringBundle } from "../../test/brainFixtures.ts";
import type { BrainBundle } from "../../types/generated/BrainBundle.ts";
import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { BrainSink } from "../../types/generated/BrainSink.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import type { Site } from "../../types/generated/Site.ts";
import { EdgeBundleList } from "./EdgeBundleList.tsx";

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
				{
					own,
					genomic: 0.5 + own,
					pattern: 1,
					scale: 0.5,
				},
			),
		);
	}
	return graphBrain({
		output_sinks: sinks,
		bundles: [
			ringBundle(1, {
				offset: 1,
				scale: 0.5,
				coordinates: Array.from({ length: 8 }, (_, c) => ({
					field: c,
					direction: (c + 1) % 8,
					present: c !== 3,
				})),
			}),
		],
	});
}

const refs = [
	{ family: "world", key: "FoodHere" },
	{ family: "world", key: "NeighborBarrierRing" },
] as const;

/** The pair coordinates the view lists for a Move gene at offset 2 on
 * AreaFoodSummary pair 1 (x field 3, y field 4), in birth order, with the
 * x coordinate on direction 4 absent. */
function pairCoordinates(offset: number, missing?: number): BrainBundle["coordinates"] {
	const x = [1, 2, 3, 5, 6, 7].map((r) => ({ field: 3, direction: (r + offset) % 8 }));
	const y = [0, 1, 3, 4, 5, 7].map((r) => ({ field: 4, direction: (r + offset) % 8 }));
	return [...x, ...y].map((c) => ({ ...c, present: !(c.field === 3 && c.direction === missing) }));
}

describe("EdgeBundleList", () => {
	it("lists each bundle's ring, kind, offset, scale and members present with holes", () => {
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
		expect(row).toHaveTextContent("Move");
		expect(row).toHaveTextContent("offset 1");
		expect(row).toHaveTextContent("scale=0.500");
		expect(screen.getByTestId("edge-bundle-members")).toHaveTextContent("7/8 ●●●○●●●●");
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members).toHaveLength(7);
		expect(members[0]).toHaveTextContent("cell N → Move[NE] own=0.250 genomic=0.750");
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
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/by ring cell/);
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

	it("renders a pair gene's coordinates from the view and a zero-pattern member outside them", () => {
		// Preservation case (T30.F01 invariant 4): a zero-pattern direction
		// member, x at relative N (sink 2), is listed as present outside the
		// coordinates; the x coordinate on sink 4 is a hole.
		const sinks = directionSinks("Move");
		for (const c of pairCoordinates(2, 4).filter((c) => c.present)) {
			sinks[c.direction]?.edges.push(
				member(
					c.field,
					{ Direction: c.direction },
					{ own: 0, genomic: 0.5, pattern: 0.5, scale: 1 },
				),
			);
		}
		sinks[2]?.edges.push(
			member(3, { Direction: 2 }, { own: 0.125, genomic: 0.125, pattern: 0, scale: 1 }),
		);
		const bundle: BrainBundle = {
			...ringBundle(0),
			shape: { DirectionPair: { ref_idx: 0, pair: 1 } },
			offset: 2,
			scale: 1,
			coordinates: pairCoordinates(2, 4),
			facts: [
				{ field: 0, present: false },
				{ field: 5, present: false },
				{ field: 6, present: false },
			],
		};
		render(
			<EdgeBundleList
				graphDef={graphBrain({ output_sinks: sinks, bundles: [bundle] })}
				inputRefs={[{ family: "world", key: "AreaFoodSummary", food_type: 0 }]}
				learning={null}
			/>,
		);
		const row = screen.getByTestId("edge-bundle");
		expect(row).toHaveTextContent("Pair");
		expect(row).toHaveTextContent("nearestDx/nearestDy #0");
		expect(row).toHaveTextContent("offset 2");
		expect(row).toHaveTextContent("scale=1.000");
		expect(screen.getByTestId("edge-bundle-members")).toHaveTextContent("11/12 ●○●●●●●●●●●● +1");
		const members = screen.getAllByTestId("edge-bundle-member");
		expect(members).toHaveLength(12);
		const zero = members.filter((m) => m.textContent?.includes("(zero pattern)"));
		expect(zero).toHaveLength(1);
		expect(zero[0]).toHaveTextContent("x → Move[E]");
		fireEvent.mouseEnter(screen.getByText(/own=0\.125/));
		expect(screen.getByTestId("tooltip")).toHaveTextContent("1.000 × 0.00 + 0.125 = 0.125");
	});

	it("lists a pair gene's fact members on a facts line with the view's presence", () => {
		// Preservation case (T30.F01 invariant 4): a fact member sits on the
		// drive at pattern 0 with `genomic == own`; the declared fact `max`
		// (field 6) is a hole.
		const sinks = directionSinks("StealEnergy");
		for (const c of pairCoordinates(0)) {
			sinks[c.direction]?.edges.push(
				member(c.field, { Direction: c.direction }, { own: 0, genomic: 1, pattern: 0.5, scale: 2 }),
			);
		}
		sinks.push({
			kind: { ActionVote: { Drive: "StealEnergy" } },
			edges: [
				{ source: { InputLeaf: { ref_idx: 0, sub_idx: 1 } }, own: 0.5, genomic: 0.5, applied: 0.5 },
				member(0, "Drive", { own: 0, genomic: 0, pattern: 0, scale: 2 }),
				member(5, "Drive", { own: 0.25, genomic: 0.25, pattern: 0, scale: 2 }),
			],
		});
		const bundle: BrainBundle = {
			...ringBundle(0),
			shape: { DirectionPair: { ref_idx: 0, pair: 1 } },
			kind: "StealEnergy",
			scale: 2,
			coordinates: pairCoordinates(0),
			facts: [
				{ field: 0, present: true },
				{ field: 5, present: true },
				{ field: 6, present: false },
			],
		};
		render(
			<EdgeBundleList
				graphDef={graphBrain({ output_sinks: sinks, bundles: [bundle] })}
				inputRefs={[{ family: "world", key: "AreaFoodSummary", food_type: 0 }]}
				learning={null}
			/>,
		);
		expect(screen.getByTestId("edge-bundle-members")).toHaveTextContent("12/12 ●●●●●●●●●●●●");
		expect(screen.getByTestId("edge-bundle-facts")).toHaveTextContent("facts 2/3 ●●○");
		expect(screen.getAllByTestId("edge-bundle-member")).toHaveLength(12);
		const facts = screen.getAllByTestId("edge-bundle-fact");
		expect(facts).toHaveLength(2);
		expect(facts[0]).toHaveTextContent("amount → StealEnergy drive own=0.000");
		expect(facts[1]).toHaveTextContent("nearestDist → StealEnergy drive own=0.250 genomic=0.250");
		fireEvent.mouseEnter(screen.getByText(/own=0\.250/));
		expect(screen.getByTestId("tooltip")).toHaveTextContent("2.000 × 0.00 + 0.250 = 0.250");
	});

	it("renders nothing without bundles", () => {
		const { container } = render(
			<EdgeBundleList graphDef={graphBrain()} inputRefs={[]} learning={null} />,
		);
		expect(container.innerHTML).toBe("");
	});
});

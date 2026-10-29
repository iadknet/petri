import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { ComputeNodeKind } from "../../types/generated/ComputeNodeKind.ts";
import { CATEGORY_COLORS } from "./GraphInternalsNode.tsx";
import { EDGE_TINTS } from "./graphFlowAdapter.ts";
import { categorizeComputeNode } from "./graphNodeCategories.ts";
import { describeComputeKind } from "./graphNodeFormatters.ts";

// The petri-visuals skill's renderer draws chat explainers in the inspector's
// style and recomputes compute nodes with v3-core's math. It is plain JS outside
// the app, so this test keeps it from drifting from the inspector and runtime.
const ROOT = join(__dirname, "../../../..");
const SKILL = join(ROOT, ".agents/skills/petri-visuals/viz");
// sha256 of crates/v3-core/src/runtime/cgp/eval.rs when evalKind and the math
// expectations below were last checked against it. When eval.rs changes, review
// both, then update this fingerprint.
const EVAL_RS_SHA256 = "fde4089551ef272cec1ecbcdbff439dff308a7c9cdc5a0edda194719aa64478f";
// sha256 of crates/v3-core/src/genome/cgp/bundle.rs, the composition
// source (`compose(own, scale, pattern)`, the ring and pair birth members)
// the renderer's edgeGroups mirrors. When bundle.rs changes, review it, then
// update this fingerprint.
const BUNDLE_RS_SHA256 = "8de3a56ffae0dfb8de0449171da5ca9c737bdc01281266a90092a4a95209d4dd";

type Rgb = [number, number, number];
interface PetriViz {
	mount(target: Element, spec: unknown): void;
	CAT: Record<string, [number, number, number, number, number, string]>;
	TINT: Record<"ic" | "io" | "co" | "cc" | "gm", Rgb>;
	kindLabel(kind: ComputeNodeKind): string;
	kindCat(kind: ComputeNodeKind): string;
	evalKind(kind: ComputeNodeKind, w: number[], state: number): { v: number; state: number };
}

function loadRenderer(file: string): PetriViz {
	const scope: { PetriViz?: PetriViz } = {};
	new Function("window", readFileSync(join(SKILL, file), "utf8"))(scope);
	if (!scope.PetriViz) throw new Error(`${file} did not define PetriViz`);
	return scope.PetriViz;
}

const KINDS: ComputeNodeKind[] = [
	"Add",
	"WeightedSum",
	"Multiply",
	"Negate",
	"Abs",
	"Min",
	"Max",
	"Sigmoid",
	"Tanh",
	"Relu",
	"Clamp01",
	{ Threshold: -0.25 },
	"GreaterThan",
	"Select",
	{ DecayIntegrator: 0.5 },
	{ Momentum: 0.25 },
	{ Oscillator: 0.125 },
	"AdaptiveGain",
	{ Constant: 1 },
];

describe("petri-visuals renderer sources", () => {
	it("checked its compute math against the current runtime evaluator", () => {
		const evalRs = readFileSync(join(ROOT, "crates/v3-core/src/runtime/cgp/eval.rs"));
		expect(createHash("sha256").update(evalRs).digest("hex")).toBe(EVAL_RS_SHA256);
	});

	it("checked its member composition against the current bundle source", () => {
		const bundleRs = readFileSync(join(ROOT, "crates/v3-core/src/genome/cgp/bundle.rs"));
		expect(createHash("sha256").update(bundleRs).digest("hex")).toBe(BUNDLE_RS_SHA256);
	});

	it("ships a minified copy built from the current source (sh viz/build.sh)", () => {
		// The same esbuild command as build.sh; esbuild cannot load inside jsdom.
		const built = execFileSync(join(ROOT, "frontend/node_modules/.bin/esbuild"), [
			join(SKILL, "petri-viz.js"),
			"--minify",
			"--target=es2017",
			"--log-level=warning",
		]).toString();
		expect(readFileSync(join(SKILL, "petri-viz.min.js"), "utf8")).toBe(built);
	});
});

describe.each(["petri-viz.js", "petri-viz.min.js"])("petri-visuals renderer %s", (file) => {
	const viz = loadRenderer(file);

	it("uses the inspector's node colors", () => {
		for (const [category, c] of Object.entries(CATEGORY_COLORS)) {
			expect(viz.CAT[category]).toEqual([c.r, c.g, c.b, c.bgAlpha, c.borderAlpha, c.text]);
		}
	});

	it("uses the inspector's edge tints, slate between compute nodes", () => {
		const rgb = (t: { r: number; g: number; b: number } | undefined) => (t ? [t.r, t.g, t.b] : t);
		expect(viz.TINT.ic).toEqual(rgb(EDGE_TINTS.input_to_compute));
		expect(viz.TINT.io).toEqual(rgb(EDGE_TINTS.input_to_output));
		expect(viz.TINT.co).toEqual(rgb(EDGE_TINTS.compute_to_output));
		expect(viz.TINT.cc).toEqual([148, 163, 184]);
		expect(viz.TINT.gm).toEqual(rgb(EDGE_TINTS.group_member));
	});

	it("votes an edge group's members at scale plus correction, holes silent", () => {
		const host = document.createElement("div");
		viz.mount(host, {
			views: [
				{
					nodes: [],
					edgeGroups: [
						{
							ring: "Ring",
							kind: "Move",
							offset: 1,
							scale: 0.5,
							members: ["N", "E"],
							own: { E: 0.25 },
						},
					],
					ticks: [{ name: "t", values: {}, rings: { Ring: [1, 1, 1, 0, 0, 0, 0, 0] } }],
				},
			],
			startTick: 0,
		});
		expect(host.textContent).toContain("2/8 ●○●○○○○○");
		expect(host.textContent).toContain("Move[NE] +0.50, Move[SE] +0.75");
		expect(host.textContent).not.toContain("Move[E]");
	});

	it("votes a pair group's members at scale times the compass plus correction", () => {
		const host = document.createElement("div");
		const pair = { family: "Food", x: "nearestDx", y: "nearestDy" };
		viz.mount(host, {
			views: [
				{
					nodes: [],
					edgeGroups: [
						{ pair, kind: "Move", offset: 0, scale: 1 },
						{
							pair,
							kind: "Move",
							offset: 0,
							scale: 1,
							members: ["x:E", "y:N"],
							own: { "x:E": 0.25 },
						},
					],
					// The thing lies due east: x = 1, y = 0.
					ticks: [{ name: "t", values: {}, pairs: { Food: [1, 0] } }],
				},
			],
			startTick: 0,
		});
		expect(host.textContent).toContain("Pair Food.nearestDx/nearestDy");
		expect(host.textContent).toContain("12/12 ●●●●●●●●●●●●");
		expect(host.textContent).toContain("Move[NE] +0.71, Move[E] +1.00, Move[SE] +0.71");
		expect(host.textContent).toContain("2/12 ○●○○○○●○○○○○");
		expect(host.textContent).toContain("Move[E] +1.25");
	});

	it.each(KINDS.map((kind) => [JSON.stringify(kind), kind] as const))(
		"labels and colors %s like the inspector",
		(_, kind) => {
			expect(viz.kindLabel(kind)).toBe(describeComputeKind(kind).label);
			expect(viz.kindCat(kind)).toBe(categorizeComputeNode(kind));
		},
	);

	// Expected values follow v3-core runtime/cgp/eval.rs evaluate_compute_kind.
	it.each([
		["Add", [1, 2], 0, 3, 0],
		["Multiply", [2, 3], 0, 6, 0],
		["Multiply", [], 0, 1, 0],
		["Negate", [1, 2], 0, -3, 0],
		["Abs", [-2], 0, 2, 0],
		["Min", [], 0, 0, 0],
		["Max", [3, 1], 0, 3, 0],
		[{ Threshold: 0.5 }, [0.5], 0, 0, 0],
		[{ Threshold: 0.5 }, [0.6], 0, 1, 0],
		["GreaterThan", [1, 2], 0, 0, 0],
		["Sigmoid", [0], 0, 0.5, 0],
		["Clamp01", [2], 0, 1, 0],
		["Select", [0.5, 7, 9], 0, 7, 0],
		["Select", [0.4, 7, 9], 0, 9, 0],
		[{ Constant: 3 }, [5], 0, 3, 0],
		[{ DecayIntegrator: 0.5 }, [0.6], 0, 0.3, 0.3],
		[{ Momentum: 0.25 }, [1], 0, 0.75, 0.75],
		[{ Oscillator: 0.25 }, [], 0, 1, 0.25],
		["AdaptiveGain", [2], 1, 2.04, 1.02],
	] as [ComputeNodeKind, number[], number, number, number][])(
		"evaluates %j on %j from state %d like the runtime",
		(kind, inputs, state, value, next) => {
			const result = viz.evalKind(kind, inputs, state);
			expect(result.v).toBeCloseTo(value, 6);
			expect(result.state).toBeCloseTo(next, 6);
		},
	);

	it("renders the example spec and evaluates its first tick", () => {
		const host = document.createElement("div");
		document.body.appendChild(host);
		viz.mount(host, JSON.parse(readFileSync(join(SKILL, "example.json"), "utf8")));
		const tick = [...host.querySelectorAll("button")].find(
			(b) => b.textContent === "Weak stranger to the E",
		);
		tick?.click();
		expect(host.querySelector("svg")).not.toBeNull();
		expect(host.textContent).toContain("Σ=2.00");
		expect(host.textContent).toContain("Σ=8.00");
		expect(host.textContent).toContain("StealEnergy[E] +0.50");
		host.remove();
	});

	it("runs compute nodes in listed order; later sources read their previous output", () => {
		const host = document.createElement("div");
		const view = (prevOut: Record<string, number>) => ({
			nodes: [
				{ id: "constructor", kind: "Add", from: [["c1", 1]] },
				{ id: "c1", kind: { Constant: 5 } },
				{ id: "o0", out: "Vote Eat", from: [["constructor", 1]] },
			],
			ticks: [{ name: "tick", values: {}, prevOut }],
		});
		for (const [prevOut, sigma] of [
			[{}, "\u03a3=0.00"],
			[{ c1: 2 }, "\u03a3=2.00"],
		] as const) {
			viz.mount(host, { views: [view(prevOut)], startTick: 0 });
			expect(host.textContent).toContain(sigma);
		}
	});
});

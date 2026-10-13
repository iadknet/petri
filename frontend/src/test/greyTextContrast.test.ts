import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// WCAG 1.4.3 contrast for the muted grey text tier, computed from the
// installed Tailwind theme so a Tailwind upgrade re-proves the pairs.
const THEME = readFileSync(join(__dirname, "../../node_modules/tailwindcss/theme.css"), "utf8");

type Rgb = [number, number, number];

function themeColour(name: string): Rgb {
	const match = THEME.match(
		new RegExp(`--color-${name}:\\s*oklch\\(([\\d.]+)%\\s+([\\d.]+)\\s+([\\d.]+)\\)`),
	);
	if (!match) throw new Error(`no oklch value for --color-${name}`);
	return oklchToSrgb(Number(match[1]) / 100, Number(match[2]), Number(match[3]));
}

// OKLCH -> gamma-encoded sRGB in [0, 1] (Bjorn Ottosson's OKLab matrices).
function oklchToSrgb(lightness: number, chroma: number, hue: number): Rgb {
	const h = (hue * Math.PI) / 180;
	const a = chroma * Math.cos(h);
	const b = chroma * Math.sin(h);
	const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
	const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
	const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
	const linear: Rgb = [
		4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
	];
	return linear.map((v) => {
		const c = Math.min(1, Math.max(0, v));
		return c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
	}) as Rgb;
}

function hex(value: string): Rgb {
	return [1, 3, 5].map((i) => Number.parseInt(value.slice(i, i + 2), 16) / 255) as Rgb;
}

function over(fg: Rgb, alpha: number, bg: Rgb): Rgb {
	return fg.map((v, i) => v * alpha + (bg[i] ?? 0) * (1 - alpha)) as Rgb;
}

function luminance(colour: Rgb): number {
	const [r, g, b] = colour.map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
	return 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0);
}

function contrast(a: Rgb, b: Rgb): number {
	const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
	return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

const slate950 = themeColour("slate-950");
const slate800 = themeColour("slate-800");

// The surfaces muted text sits on: app and inspector base (slate-950), panels
// (slate-900 and the petri-panel token), inspector cards (bg-white/[0.0x] and
// slate-800/80 over slate-950) and the worst case, solid slate-800.
const BACKGROUNDS: Record<string, Rgb> = {
	"slate-950": slate950,
	"slate-900": themeColour("slate-900"),
	"petri-panel #0f172a": hex("#0f172a"),
	"slate-800": slate800,
	"white/0.1 over slate-950": over([1, 1, 1], 0.1, slate950),
	"slate-800/80 over slate-950": over(slate800, 0.8, slate950),
};

describe("muted grey text contrast", () => {
	it("reproduces the audit's measured slate-500 failures", () => {
		expect(contrast(themeColour("slate-500"), hex("#0f172a"))).toBeCloseTo(3.75, 1);
		expect(contrast(themeColour("slate-500"), slate950)).toBeCloseTo(4.23, 1);
	});

	it.each(Object.entries(BACKGROUNDS))("text-slate-400 meets 4.5:1 on %s", (_name, bg) => {
		expect(contrast(themeColour("slate-400"), bg)).toBeGreaterThanOrEqual(4.5);
	});
});

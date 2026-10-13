import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const SRC = join(__dirname, "..");

function sourceFiles(dir: string): string[] {
	return readdirSync(dir).flatMap((entry) => {
		const path = join(dir, entry);
		if (statSync(path).isDirectory()) return sourceFiles(path);
		return /\.tsx$/.test(entry) && !/\.test\.tsx$/.test(entry) ? [path] : [];
	});
}

// text-slate-500 and text-slate-600 fall below WCAG 1.4.3's 4.5:1 on every
// panel surface (see greyTextContrast.test.ts); text-slate-400 is the muted
// tier. Text of a disabled control is exempt from 1.4.3, so these exact class
// strings, each applied only while the control is disabled, may keep it.
const DISABLED_ALLOWLIST: Array<{ file: string; classes: string }> = [
	// Restart/Step button: the branch taken when `disabled` is set on the button.
	{
		file: "components/ControlBar.tsx",
		classes: "bg-slate-800 text-slate-500 opacity-50 cursor-not-allowed",
	},
	// Start button: the `disabled:` variant applies only to the disabled button.
	{
		file: "components/ConfigPanel.tsx",
		classes: "disabled:bg-slate-800 disabled:text-slate-500",
	},
	// Sample button: the branch taken when `disabled={isDead}` is set.
	{
		file: "components/inspector/SamplerBar.tsx",
		classes: "bg-white/[0.03] text-slate-600 cursor-not-allowed",
	},
];

function faintGreyClasses(source: string): string[] {
	return source.split(/[\s"'`{}]+/).filter((token) => /(^|:)text-slate-(500|600)$/.test(token));
}

describe("muted grey text classes", () => {
	it("uses no sub-AA grey text outside disabled states", () => {
		const offenders = sourceFiles(SRC).flatMap((file) => {
			const name = relative(SRC, file);
			let source = readFileSync(file, "utf8");
			for (const entry of DISABLED_ALLOWLIST) {
				if (entry.file === name) source = source.split(entry.classes).join(" ");
			}
			return faintGreyClasses(source).map((cls) => `${name}: ${cls}`);
		});
		expect(offenders).toEqual([]);
	});

	it("keeps every allowlist entry pointing at a real disabled-state class string", () => {
		for (const entry of DISABLED_ALLOWLIST) {
			expect(readFileSync(join(SRC, entry.file), "utf8")).toContain(entry.classes);
		}
	});
});

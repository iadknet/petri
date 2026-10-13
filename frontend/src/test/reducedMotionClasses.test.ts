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

// Every Tailwind animation class in a className string must stop under
// prefers-reduced-motion, either by being motion-safe: or by pairing with
// motion-reduce:animate-none in the same string.
function unguardedAnimations(source: string): string[] {
	const offenders: string[] = [];
	for (const match of source.matchAll(/"([^"\n]*)"/g)) {
		const classes = (match[1] ?? "").split(/\s+/);
		const animated = classes.filter((cls) => /^animate-(?!none\b)/.test(cls));
		if (animated.length > 0 && !classes.includes("motion-reduce:animate-none")) {
			offenders.push(...animated);
		}
	}
	return offenders;
}

describe("reduced-motion guards", () => {
	it("pairs every animation class with motion-reduce:animate-none", () => {
		const offenders = sourceFiles(SRC).flatMap((file) =>
			unguardedAnimations(readFileSync(file, "utf8")).map(
				(cls) => `${relative(SRC, file)}: ${cls}`,
			),
		);
		expect(offenders).toEqual([]);
	});
});

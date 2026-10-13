import { readFileSync } from "node:fs";
import { join } from "node:path";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Tooltip } from "../components/config-panel/shared/Tooltip.tsx";
import { targetSize } from "./targetSize.ts";

const STYLES = readFileSync(join(__dirname, "../styles.css"), "utf8");

describe("pointer target size (WCAG 2.5.8)", () => {
	it("reads a box and its invisible hit area from Tailwind classes", () => {
		const el = document.createElement("button");
		el.className = "relative size-4 before:absolute before:-inset-1";
		expect(targetSize(el)).toEqual({ width: 24, height: 24 });
		el.className = "h-5 w-8 before:-inset-y-0.5";
		expect(targetSize(el)).toEqual({ width: 32, height: 24 });
	});

	it("gives the About this setting button a 24px target", () => {
		render(<Tooltip text="Explains the setting" />);
		const button = screen.getByRole("button", { name: "About this setting" });
		const { width, height } = targetSize(button);
		expect(width).toBeGreaterThanOrEqual(24);
		expect(height).toBeGreaterThanOrEqual(24);
		expect(button.className).toMatch(/(^|\s)relative(\s|$)/);
	});

	it("gives every config switch a centred hit area of at least 24px", () => {
		const rule = STYLES.match(/\.config-switch::before\s*\{([^}]*)\}/)?.[1] ?? "";
		expect(rule).toMatch(/content:\s*""/);
		expect(rule).toMatch(/position:\s*absolute/);
		expect(rule).toMatch(/width:\s*max\(100%,\s*24px\)/);
		expect(rule).toMatch(/height:\s*max\(100%,\s*24px\)/);
		expect(rule).toMatch(/translate\(-50%,\s*-50%\)/);
	});
});

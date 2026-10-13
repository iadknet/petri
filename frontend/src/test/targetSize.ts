// Smallest pointer target, in CSS px, that an element's Tailwind classes give
// it: its box (size-*, w-*, h-*) grown by any invisible before: hit area
// (before:-inset-*, before:-inset-x-*, before:-inset-y-*). jsdom does no
// layout, so WCAG 2.5.8 (24 by 24 px) is checked from the classes.
const SPACING_PX = 4;

export function targetSize(element: Element): { width: number; height: number } {
	const classes = (element.getAttribute("class") ?? "").split(/\s+/);
	const px = (pattern: RegExp): number => {
		for (const cls of classes) {
			const match = cls.match(pattern);
			if (match) return Number(match[1]) * SPACING_PX;
		}
		return 0;
	};
	const size = px(/^size-([\d.]+)$/);
	const inset = px(/^before:-inset-([\d.]+)$/);
	return {
		width: (px(/^w-([\d.]+)$/) || size) + 2 * (inset || px(/^before:-inset-x-([\d.]+)$/)),
		height: (px(/^h-([\d.]+)$/) || size) + 2 * (inset || px(/^before:-inset-y-([\d.]+)$/)),
	};
}

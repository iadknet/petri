/** One stroke weight and grid for every glyph in the config panel. */
const PATHS = {
	search: "M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13ZM20 20l-4.6-4.6",
	close: "M6 6l12 12M18 6 6 18",
	chevron: "m9 6 6 6-6 6",
	info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 11v5.5M12 7.6v.1",
	lock: "M6.5 10.5h11v9h-11zM8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5",
	undo: "M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11",
	reset: "M3.5 12a8.5 8.5 0 1 0 2.6-6.1M3.5 4v4.5H8",
	shuffle:
		"M3 7h3.5c4 0 6 10 10 10H21M3 17h3.5c1.7 0 3-1.7 4.2-4M21 7h-4.5c-1.7 0-3 1.7-4.2 4M18 4l3 3-3 3M18 14l3 3-3 3",
	pause: "M8.5 5v14M15.5 5v14",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, className = "size-3.5" }: { name: IconName; className?: string }) {
	return (
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth={1.9}
			strokeLinecap="round"
			strokeLinejoin="round"
			aria-hidden="true"
			focusable="false"
			className={`shrink-0 ${className}`}
		>
			<path d={PATHS[name]} />
		</svg>
	);
}

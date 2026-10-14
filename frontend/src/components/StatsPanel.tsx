import {
	type KeyboardEvent,
	type PointerEvent,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from "react";
import { useStatsHistoryStore } from "../stores/stats.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import { BandTab } from "./stats/BandTab.tsx";
import { MechanicsTab } from "./stats/MechanicsTab.tsx";
import { BAND_COLOR } from "./stats/bandColors.ts";
import { type DockLayout, clampDockHeight, loadDock, saveDock } from "./stats/dockLayout.ts";
import { type BandId, bandVerdict } from "./stats/readings.ts";

const KEY_STEP = 24;

type TabId = "mechanics" | BandId;

const TABS: { id: TabId; label: string }[] = [
	{ id: "mechanics", label: "Mechanics" },
	{ id: "world", label: "World" },
	{ id: "evolution", label: "Evolution" },
	{ id: "brains", label: "Brains" },
];

function windowHeight(): number {
	return typeof window === "undefined" ? 1000 : window.innerHeight;
}

/** The live stats dock under the map: tabs that each fill the panel, never scrolling. */
export function StatsPanel() {
	const samples = useStatsHistoryStore((s) => s.samples);
	const foodTypes = useWorldViewStore((s) => s.worldStatic?.food_types);
	const foodNames = useMemo(() => {
		const names: string[] = [];
		for (const type of foodTypes ?? []) names[type.type_idx] = type.name;
		return names;
	}, [foodTypes]);

	const [tab, setTab] = useState<TabId>("mechanics");
	const [scrub, setScrub] = useState<number | null>(null);
	const [layout, setLayout] = useState<DockLayout>(() => {
		const stored = loadDock();
		return { ...stored, height: clampDockHeight(stored.height, windowHeight()) };
	});
	const [dragging, setDragging] = useState(false);
	const drag = useRef<{ startY: number; startHeight: number } | null>(null);

	const last = samples.length - 1;
	const scrubbing = scrub != null && scrub < last;
	const index = scrubbing ? scrub : last;
	const span = last > 0 ? samples[last]!.tick - samples[0]!.tick : 0;

	const update = useCallback((next: DockLayout) => {
		setLayout(next);
		saveDock(next);
	}, []);

	useEffect(() => {
		const onResize = () =>
			setLayout((l) => ({ ...l, height: clampDockHeight(l.height, windowHeight()) }));
		window.addEventListener("resize", onResize);
		return () => window.removeEventListener("resize", onResize);
	}, []);

	const selectTab = useCallback(
		(next: TabId, focus = false) => {
			setTab(next);
			setScrub(null);
			if (layout.collapsed) update({ ...layout, collapsed: false });
			if (focus) document.getElementById(`stats-tab-${next}`)?.focus();
		},
		[layout, update],
	);

	const onTabKey = (event: KeyboardEvent<HTMLButtonElement>, at: number) => {
		if (event.ctrlKey || event.metaKey || event.altKey) return;
		const lastTab = TABS.length - 1;
		let next: number;
		if (event.key === "ArrowRight") next = at === lastTab ? 0 : at + 1;
		else if (event.key === "ArrowLeft") next = at === 0 ? lastTab : at - 1;
		else if (event.key === "Home") next = 0;
		else if (event.key === "End") next = lastTab;
		else return;
		event.preventDefault();
		selectTab(TABS[next]!.id, true);
	};

	const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
		if (event.button !== 0) return;
		event.currentTarget.setPointerCapture(event.pointerId);
		drag.current = { startY: event.clientY, startHeight: layout.collapsed ? 0 : layout.height };
		setDragging(true);
	};
	const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
		const start = drag.current;
		if (!start) return;
		const height = clampDockHeight(
			start.startHeight + start.startY - event.clientY,
			windowHeight(),
		);
		setLayout({ height, collapsed: false });
	};
	const onPointerUp = () => {
		if (!drag.current) return;
		drag.current = null;
		setDragging(false);
		saveDock(layout);
	};
	const onHandleKey = (event: KeyboardEvent<HTMLDivElement>) => {
		const max = windowHeight();
		let height: number;
		if (event.key === "ArrowUp") height = layout.height + KEY_STEP;
		else if (event.key === "ArrowDown") height = layout.height - KEY_STEP;
		else if (event.key === "Home") height = 0;
		else if (event.key === "End") height = max;
		else if (event.key === "Enter") {
			update({ ...layout, collapsed: !layout.collapsed });
			event.preventDefault();
			return;
		} else return;
		event.preventDefault();
		update({ height: clampDockHeight(height, max), collapsed: false });
	};

	const toggle = () => update({ ...layout, collapsed: !layout.collapsed });
	const bandTab = tab !== "mechanics";

	return (
		<section
			data-testid="stats-panel"
			aria-labelledby="stats-panel-heading"
			className={`relative flex shrink-0 flex-col border-t border-petri-border bg-petri-panel ${
				dragging ? "" : "transition-[height] duration-200 ease-out motion-reduce:transition-none"
			}`}
			style={{ height: layout.collapsed ? undefined : `${layout.height}px` }}
		>
			{/* biome-ignore lint/a11y/useSemanticElements: an <hr> cannot take focus or report its value */}
			<div
				role="separator"
				aria-orientation="horizontal"
				aria-label="Resize statistics"
				aria-valuemin={0}
				aria-valuemax={Math.round(windowHeight() * 0.75)}
				aria-valuenow={layout.collapsed ? 0 : layout.height}
				aria-controls="stats-panel-body"
				tabIndex={0}
				onPointerDown={onPointerDown}
				onPointerMove={onPointerMove}
				onPointerUp={onPointerUp}
				onPointerCancel={onPointerUp}
				onDoubleClick={toggle}
				onKeyDown={onHandleKey}
				className="group absolute inset-x-0 -top-1 z-20 flex h-2 cursor-row-resize touch-none justify-center outline-none"
			>
				<span
					className={`mt-[3px] h-0.5 w-10 rounded-full transition-colors duration-150 ${
						dragging
							? "bg-sky-400"
							: "bg-slate-600 group-hover:bg-slate-400 group-focus-visible:bg-sky-400"
					}`}
				/>
			</div>

			<header className="@container flex h-9 shrink-0 items-stretch gap-3 border-b border-petri-border bg-slate-950/70 pr-1.5 pl-1">
				<h2 id="stats-panel-heading" className="sr-only">
					Statistics
				</h2>
				<div role="tablist" aria-label="Statistics views" className="flex min-w-0 items-stretch">
					{TABS.map(({ id, label }, at) => {
						const selected = tab === id;
						const verdict = id === "mechanics" ? null : bandVerdict(id, samples);
						return (
							<button
								key={id}
								type="button"
								role="tab"
								id={`stats-tab-${id}`}
								aria-selected={selected}
								aria-controls="stats-panel-body"
								tabIndex={selected ? 0 : -1}
								title={verdict?.full}
								onClick={() => selectTab(id)}
								onKeyDown={(event) => onTabKey(event, at)}
								className={`-mb-px flex shrink-0 items-center gap-1.5 border-b-2 px-2.5 text-xs transition-colors duration-150 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-sky-400 ${
									selected
										? "border-sky-400 text-slate-100"
										: "border-transparent text-slate-400 hover:text-slate-200"
								}`}
							>
								{id !== "mechanics" && (
									<span
										aria-hidden="true"
										className="h-1.5 w-1.5 shrink-0 rounded-full"
										style={{ backgroundColor: BAND_COLOR[id] }}
									/>
								)}
								<span className="font-semibold">{label}</span>
								{verdict && (
									// The word needs room; narrow panels keep it in the tooltip.
									<span className="hidden text-slate-400 @2xl:inline">{verdict.word}</span>
								)}
							</button>
						);
					})}
				</div>
				<div className="flex-1" />
				{bandTab && !layout.collapsed && span > 0 && (
					<div className="relative flex min-w-0 items-center rounded px-1.5 has-[input:focus-visible]:outline-2 has-[input:focus-visible]:-outline-offset-2 has-[input:focus-visible]:outline-sky-400">
						<span
							className={`truncate font-mono text-[11px] tabular-nums ${scrubbing ? "text-sky-300" : "text-slate-400"}`}
						>
							{scrubbing
								? `At tick ${samples[index]!.tick.toLocaleString("en-US")}`
								: `Last ${span.toLocaleString("en-US")} ticks`}
						</span>
						<input
							type="range"
							aria-label="Scrub through time"
							aria-valuetext={`tick ${samples[index]!.tick}`}
							title="Hover a chart, or focus here and use the arrow keys, to read every chart at that tick"
							min={0}
							max={last}
							value={index}
							onChange={(event) => {
								const next = Number(event.target.value);
								setScrub(next >= last ? null : next);
							}}
							onBlur={() => setScrub(null)}
							onKeyDown={(event) => {
								if (event.key !== "Escape" || !scrubbing) return;
								// Back to now without also clearing the map selection.
								event.preventDefault();
								setScrub(null);
							}}
							className="absolute inset-0 h-full w-full cursor-default opacity-0"
						/>
					</div>
				)}
				<button
					type="button"
					onClick={toggle}
					aria-expanded={!layout.collapsed}
					aria-controls="stats-panel-body"
					aria-label={layout.collapsed ? "Expand statistics" : "Collapse statistics"}
					title={layout.collapsed ? "Expand" : "Collapse"}
					className="my-auto grid h-7 w-7 shrink-0 place-items-center rounded text-slate-400 transition-colors duration-150 hover:bg-slate-800 hover:text-slate-200 focus-visible:outline-2 focus-visible:outline-sky-400"
				>
					<svg
						aria-hidden="true"
						viewBox="0 0 12 12"
						className={`h-3 w-3 transition-transform duration-200 ${layout.collapsed ? "rotate-180" : ""}`}
					>
						<path d="M3 4.5l3 3 3-3" fill="none" stroke="currentColor" strokeWidth="1.5" />
					</svg>
				</button>
			</header>

			{!layout.collapsed && (
				<div
					id="stats-panel-body"
					role="tabpanel"
					aria-labelledby={`stats-tab-${tab}`}
					className="min-h-0 flex-1 overflow-hidden"
				>
					{tab === "mechanics" ? (
						<MechanicsTab />
					) : samples.length === 0 ? (
						<p className="px-3 py-4 text-[13px] text-slate-400">
							Readings appear here once the simulation runs. Start or step it from the top bar.
						</p>
					) : (
						<BandTab
							band={tab}
							samples={samples}
							foodNames={foodNames}
							index={index}
							scrubbing={scrubbing}
							onScrub={setScrub}
						/>
					)}
				</div>
			)}
		</section>
	);
}

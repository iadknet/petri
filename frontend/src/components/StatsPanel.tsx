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
import { CensusSheet } from "./stats/CensusSheet.tsx";
import { MechanicsDetails } from "./stats/MechanicsDetails.tsx";
import { BAND_COLOR } from "./stats/bandColors.ts";
import { type DockLayout, clampDockHeight, loadDock, saveDock } from "./stats/dockLayout.ts";
import { BANDS, bandVerdict } from "./stats/readings.ts";

const KEY_STEP = 24;

function windowHeight(): number {
	return typeof window === "undefined" ? 1000 : window.innerHeight;
}

/** The live stats dock under the map: a verdict strip that expands into the census sheet. */
export function StatsPanel() {
	const samples = useStatsHistoryStore((s) => s.samples);
	const foodTypes = useWorldViewStore((s) => s.worldStatic?.food_types);
	const foodNames = useMemo(() => {
		const names: string[] = [];
		for (const type of foodTypes ?? []) names[type.type_idx] = type.name;
		return names;
	}, [foodTypes]);

	const [layout, setLayout] = useState<DockLayout>(() => {
		const stored = loadDock();
		return { ...stored, height: clampDockHeight(stored.height, windowHeight()) };
	});
	const [dragging, setDragging] = useState(false);
	const [scrubbing, setScrubbing] = useState(false);
	const drag = useRef<{ startY: number; startHeight: number } | null>(null);

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

			<header className="flex min-h-9 items-center gap-3 border-b border-petri-border bg-slate-950/70 pl-3 pr-1.5">
				<h2 id="stats-panel-heading" className="sr-only">
					Statistics
				</h2>
				<ul
					data-testid="stats-verdicts"
					aria-label="Verdicts now"
					className={`flex min-w-0 flex-1 flex-wrap items-baseline gap-x-5 gap-y-0.5 py-1.5 text-xs transition-opacity duration-150 ${
						scrubbing ? "opacity-60" : ""
					}`}
				>
					{scrubbing && (
						<li className="rounded-sm bg-slate-800 px-1.5 text-[11px] font-semibold text-slate-200">
							Now
						</li>
					)}
					{BANDS.map((band) => {
						const verdict = bandVerdict(band.id, samples);
						return (
							<li
								key={band.id}
								className="flex min-w-0 max-w-full items-baseline gap-1.5"
								title={verdict.full}
							>
								<span
									aria-hidden="true"
									className="h-1.5 w-1.5 shrink-0 translate-y-[-1px] rounded-full"
									style={{ backgroundColor: BAND_COLOR[band.id] }}
								/>
								<span className="shrink-0 font-semibold text-slate-200">{band.label}</span>
								<span className="truncate text-slate-300">{verdict.short}</span>
							</li>
						);
					})}
				</ul>
				<button
					type="button"
					onClick={toggle}
					aria-expanded={!layout.collapsed}
					aria-controls="stats-panel-body"
					aria-label={layout.collapsed ? "Expand statistics" : "Collapse statistics"}
					title={layout.collapsed ? "Expand" : "Collapse"}
					className="grid h-7 w-7 shrink-0 place-items-center rounded text-slate-400 transition-colors duration-150 hover:bg-slate-800 hover:text-slate-200 focus-visible:outline-2 focus-visible:outline-sky-400"
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
				<div id="stats-panel-body" className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
					{samples.length === 0 ? (
						<p className="px-3 py-4 text-[13px] text-slate-400">
							Readings appear here once the simulation runs. Start or step it from the top bar.
						</p>
					) : (
						<CensusSheet samples={samples} foodNames={foodNames} onScrubbingChange={setScrubbing} />
					)}
					<MechanicsDetails />
				</div>
			)}
		</section>
	);
}
